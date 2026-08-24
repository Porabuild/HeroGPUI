//! Tabs — port of `@heroui/tabs`.

use gpui::{
    prelude::*, px, AnyElement, App, InteractiveElement, IntoElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window,
};
use herogpui_core::Orientation;
use herogpui_theme::ActiveTheme;

/// Tab bar style (`variant`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabsVariant {
    /// Filled indicator behind the selected tab.
    #[default]
    Primary,
    /// Underline indicator beneath the selected tab.
    Secondary,
}

impl TabsVariant {
    pub const ALL: [TabsVariant; 2] = [TabsVariant::Primary, TabsVariant::Secondary];

    pub fn label(self) -> &'static str {
        match self {
            TabsVariant::Primary => "Primary",
            TabsVariant::Secondary => "Secondary",
        }
    }
}

/// One tab: key + label + panel content.
pub struct TabItem {
    pub key: SharedString,
    pub label: SharedString,
    pub content: Option<AnyElement>,
    /// `<Tabs.Separator />` inside this tab. v3 made the hairline between
    /// segments opt-in per tab in 3.0.0-beta.12, replacing the automatic
    /// pseudo-element and the `hideSeparator` prop it deleted.
    pub separator: bool,
}

impl TabItem {
    pub fn new(key: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            content: None,
            separator: false,
        }
    }

    pub fn content(mut self, el: impl IntoElement) -> Self {
        self.content = Some(el.into_any_element());
        self
    }

    /// Composes a `Tabs.Separator` into this tab: a hairline before it.
    pub fn separator(mut self) -> Self {
        self.separator = true;
        self
    }
}

type OnChange = std::sync::Arc<dyn Fn(&SharedString, &mut Window, &mut App) + 'static>;

/// HeroUI Tabs (controlled).
#[derive(IntoElement)]
pub struct Tabs {
    id: gpui::ElementId,
    items: Vec<TabItem>,
    /// `selectedKey` — `None` leaves the tabs holding the selection, seeded
    /// from `defaultSelectedKey`.
    selected_key: Option<SharedString>,
    default_selected_key: Option<SharedString>,
    variant: TabsVariant,
    is_disabled: bool,
    orientation: Orientation,
    on_selection_change: Option<OnChange>,
}

impl Tabs {
    /// `orientation` — a vertical tab list stacks its tabs.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// `selectedKey` — drives the tabs from outside; without it they hold
    /// their own selection, seeded positionally by [`Tabs::new`].
    pub fn selected_key(mut self, key: impl Into<SharedString>) -> Self {
        self.selected_key = Some(key.into());
        self
    }

    /// `defaultSelectedKey` — the uncontrolled initial tab, also accepted
    /// positionally by [`Tabs::new`].
    ///
    /// Only consulted when `selectedKey` is not supplied; the tabs then own the
    /// selection and switch themselves on press.
    pub fn default_selected_key(mut self, key: impl Into<SharedString>) -> Self {
        self.default_selected_key = Some(key.into());
        self
    }

    /// The positional key is `defaultSelectedKey`, not `selectedKey`: seeding
    /// the *controlled* prop leaves the tabs unable to switch themselves, so
    /// every demo that passed a literal was inert. Pass
    /// [`Tabs::selected_key`] to drive them from outside.
    pub fn new(
        id: impl Into<gpui::ElementId>,
        items: Vec<TabItem>,
        default_selected_key: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            items,
            selected_key: None,
            default_selected_key: Some(default_selected_key.into()),
            variant: TabsVariant::Primary,
            is_disabled: false,
            orientation: Orientation::Horizontal,
            on_selection_change: None,
        }
    }

    pub fn variant(mut self, v: TabsVariant) -> Self {
        self.variant = v;
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    /// `onSelectionChange` — reports the key of the tab the press moves to.
    pub fn on_selection_change(
        mut self,
        f: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change = Some(std::sync::Arc::new(f));
        self
    }
}

impl RenderOnce for Tabs {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let base_id = format!("{:?}", self.id);

        // `selectedKey` wins; without it the tabs hold the selection, seeded
        // from `defaultSelectedKey` (falling back to the first tab so something
        // is always active). `controlled` takes `cx` mutably, so it precedes
        // the theme tokens.
        let fallback = self
            .default_selected_key
            .clone()
            .or_else(|| self.items.first().map(|i| i.key.clone()))
            .unwrap_or_default();
        // One handle for the list: a tab list is one tab stop and the selected
        // tab claims it, which is how the stop roves. Flipping a handle's
        // `tab_stop` cannot do that -- it is fixed where the handle is made.
        let list_focus = crate::util::tab_stop_handle(
            gpui::ElementId::Name(format!("{base_id}-focus").into()),
            window,
            cx,
        );
        let (selected_key, selection_own) = crate::util::controlled(
            window,
            cx,
            gpui::ElementId::Name(format!("{base_id}-selected").into()),
            self.selected_key.clone(),
            fallback,
        );

        // `.tabs__list-container__scroller` is the box `.tabs__list` scrolls
        // inside; the handle is what says how far it has, which is what decides
        // whether each chevron shows.
        let scroll = window
            .use_keyed_state(
                gpui::ElementId::Name(format!("{base_id}-scroll").into()),
                cx,
                |_, _| gpui::ScrollHandle::new(),
            )
            .read(cx)
            .clone();

        // The two chevrons' visibility, measured a frame ago; `use_keyed_state`
        // takes `cx` mutably, so it precedes the theme borrow.
        let arrows = window.use_keyed_state(
            gpui::ElementId::Name(format!("{base_id}-arrows").into()),
            cx,
            |_, _| (false, false),
        );

        let colors = cx.colors();
        let layout = cx.layout();

        let vertical = self.orientation == Orientation::Vertical;
        // `.tabs__list` is `w-max min-w-full`: it grows with its content, which is
        // what lets the scroller overflow -- a shrinking row always fits and
        // never scrolls.
        let mut list = gpui::div().flex().flex_shrink_0();
        if vertical {
            list = list.flex_col().items_start();
        }

        // v3 keeps two indicator styles: `primary` fills a segment behind the
        // selected tab, `secondary` underlines it.
        match self.variant {
            TabsVariant::Primary => {
                // `.tabs__list` is `p-1` and nothing else: the tabs sit
                // shoulder to shoulder, with no gap between them.
                list = list
                    .p(px(4.))
                    .rounded(crate::util::control_radius(cx))
                    .bg(colors.surface_secondary);
                let selected_index = self.items.iter().position(|item| item.key == selected_key);
                for (index, item) in self.items.iter().enumerate() {
                    let active = item.key == selected_key;
                    // `.tabs__separator` is a `w-px h-1/2 rounded-sm bg-muted/25`
                    // hairline between segments, hidden on either side of the
                    // selected one (`&[data-selected] .tabs__separator` and
                    // `& + .tabs__tab .tabs__separator` are `opacity-0`).
                    if index > 0 && item.separator {
                        let touches_selected =
                            selected_index == Some(index) || selected_index == Some(index - 1);
                        list = list.child(
                            gpui::div()
                                .w(cx.layout().border_width)
                                .h(px(16.))
                                .flex_shrink_0()
                                .my_auto()
                                .rounded(crate::util::hairline_radius(cx))
                                .bg(if touches_selected {
                                    gpui::transparent_black()
                                } else {
                                    colors.muted.alpha(0.25)
                                }),
                        );
                    }
                    let mut tab = gpui::div()
                        .id(gpui::ElementId::Name(
                            format!("{base_id}-tab-{}", item.key).into(),
                        ))
                        .when(!self.is_disabled && active, |t| t.track_focus(&list_focus))
                        // `.tabs__tab` is `h-8 px-4 rounded-3xl text-sm
                        // font-medium`.
                        .h(px(32.))
                        .px(px(16.))
                        .flex_shrink_0()
                        // A tab's label does not wrap: `.tabs__list` is `w-max`,
                        // so the row is as wide as its labels and the scroller
                        // is what handles the overflow.
                        .whitespace_nowrap()
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(crate::util::control_radius(cx))
                        .text_size(px(14.))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .when(!self.is_disabled, |t| t.cursor_pointer())
                        // `status-disabled` is `--disabled-opacity`.
                        .when(self.is_disabled, |t| {
                            t.opacity(cx.layout().disabled_opacity)
                        });
                    if active {
                        tab = tab
                            .bg(colors.segment.background)
                            .text_color(colors.segment.foreground)
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .when(!layout.surface_shadow.is_empty(), |t| {
                                t.shadow(layout.surface_shadow.clone())
                            });
                    } else {
                        tab = tab.text_color(colors.muted);
                        if !self.is_disabled {
                            tab = tab.hover(move |s| s.text_color(colors.foreground));
                        }
                    }
                    if !self.is_disabled
                        && (self.on_selection_change.is_some() || selection_own.is_some())
                    {
                        // A tab list is one stop and the arrows move within
                        // it, selecting as they go -- React Aria's automatic
                        // activation, which is what v3 ships.
                        let key_stops: Vec<usize> = (0..self.items.len()).collect();
                        let key_keys: Vec<SharedString> =
                            self.items.iter().map(|i| i.key.clone()).collect();
                        let key_cb = self.on_selection_change.clone();
                        let key_own = selection_own.clone();
                        tab = tab.on_key_down(move |event, window, cx| {
                            let key = match event.keystroke.key.as_str() {
                                "right" | "down" => "down",
                                "left" | "up" => "up",
                                other @ ("home" | "end") => other,
                                _ => return,
                            };
                            let crate::list_nav::Move::To(next) =
                                crate::list_nav::resolve(&key_stops, Some(index), key, true)
                            else {
                                return;
                            };
                            let Some(next_key) = key_keys.get(next).cloned() else {
                                return;
                            };
                            if let Some(held) = &key_own {
                                let next_key = next_key.clone();
                                held.update(cx, |v, cx| {
                                    *v = next_key;
                                    cx.notify();
                                });
                            }
                            if let Some(f) = &key_cb {
                                f(&next_key, window, cx);
                            }
                            // No refocusing: the next render has the newly
                            // selected tab claim the list's handle.
                        });
                        let key = item.key.clone();
                        let cb = self.on_selection_change.clone();
                        let own = selection_own.clone();
                        tab = tab.on_click(move |_, window, cx| {
                            // Uncontrolled: move our own selection, or pressing
                            // a tab would do nothing.
                            if let Some(held) = &own {
                                held.update(cx, |v, cx| {
                                    *v = key.clone();
                                    cx.notify();
                                });
                            }
                            if let Some(f) = &cb {
                                f(&key, window, cx);
                            }
                        });
                    }
                    // `.tab:focus-visible` is `status-focused`.
                    let tab = crate::util::with_focus_ring(
                        tab,
                        active
                            && list_focus.is_focused(window)
                            && crate::util::focus_visible(cx)
                            && !self.is_disabled,
                        true,
                        Vec::new(),
                        cx,
                    );
                    list = list.child(tab.child(item.label.to_string()));
                }
            }
            TabsVariant::Secondary => {
                // `.tabs--secondary` gives the list `p-0` and the *container*
                // `border-b border-border`; the tabs keep their own box.
                list = list.border_b_1().border_color(colors.border);
                for (index, item) in self.items.iter().enumerate() {
                    let active = item.key == selected_key;
                    let mut tab = gpui::div()
                        .id(gpui::ElementId::Name(
                            format!("{base_id}-tab-{}", item.key).into(),
                        ))
                        .when(!self.is_disabled && active, |t| t.track_focus(&list_focus))
                        // The same `h-8 px-4 text-sm` box, `rounded-none`, with
                        // the indicator as a 2px bar along the bottom.
                        .h(px(32.))
                        .px(px(16.))
                        .flex_shrink_0()
                        // A tab's label does not wrap: `.tabs__list` is `w-max`,
                        // so the row is as wide as its labels and the scroller
                        // is what handles the overflow.
                        .whitespace_nowrap()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(14.))
                        .line_height(px(20.))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .border_b_2()
                        .when(!self.is_disabled, |t| t.cursor_pointer())
                        // `status-disabled` is `--disabled-opacity`.
                        .when(self.is_disabled, |t| {
                            t.opacity(cx.layout().disabled_opacity)
                        });
                    tab = if active {
                        tab.border_color(colors.accent.color)
                            .text_color(colors.foreground)
                            .font_weight(gpui::FontWeight::MEDIUM)
                    } else {
                        tab.border_color(gpui::transparent_black())
                            .text_color(colors.muted)
                    };
                    if !self.is_disabled
                        && (self.on_selection_change.is_some() || selection_own.is_some())
                    {
                        // A tab list is one stop and the arrows move within
                        // it, selecting as they go -- React Aria's automatic
                        // activation, which is what v3 ships.
                        let key_stops: Vec<usize> = (0..self.items.len()).collect();
                        let key_keys: Vec<SharedString> =
                            self.items.iter().map(|i| i.key.clone()).collect();
                        let key_cb = self.on_selection_change.clone();
                        let key_own = selection_own.clone();
                        tab = tab.on_key_down(move |event, window, cx| {
                            let key = match event.keystroke.key.as_str() {
                                "right" | "down" => "down",
                                "left" | "up" => "up",
                                other @ ("home" | "end") => other,
                                _ => return,
                            };
                            let crate::list_nav::Move::To(next) =
                                crate::list_nav::resolve(&key_stops, Some(index), key, true)
                            else {
                                return;
                            };
                            let Some(next_key) = key_keys.get(next).cloned() else {
                                return;
                            };
                            if let Some(held) = &key_own {
                                let next_key = next_key.clone();
                                held.update(cx, |v, cx| {
                                    *v = next_key;
                                    cx.notify();
                                });
                            }
                            if let Some(f) = &key_cb {
                                f(&next_key, window, cx);
                            }
                            // No refocusing: the next render has the newly
                            // selected tab claim the list's handle.
                        });
                        let key = item.key.clone();
                        let cb = self.on_selection_change.clone();
                        let own = selection_own.clone();
                        tab = tab.on_click(move |_, window, cx| {
                            // Uncontrolled: move our own selection, or pressing
                            // a tab would do nothing.
                            if let Some(held) = &own {
                                held.update(cx, |v, cx| {
                                    *v = key.clone();
                                    cx.notify();
                                });
                            }
                            if let Some(f) = &cb {
                                f(&key, window, cx);
                            }
                        });
                    }
                    // `.tab:focus-visible` is `status-focused`.
                    let tab = crate::util::with_focus_ring(
                        tab,
                        active
                            && list_focus.is_focused(window)
                            && crate::util::focus_visible(cx)
                            && !self.is_disabled,
                        true,
                        Vec::new(),
                        cx,
                    );
                    list = list.child(tab.child(item.label.to_string()));
                }
            }
        }

        // Active panel
        let mut items = self.items;
        let active_idx = items.iter().position(|i| i.key == selected_key);
        // `.tabs__list-container` is `relative`, holds the scroller, and hangs
        // the two `size-4` chevrons off its edges -- `hidden` until there is
        // something to scroll to in that direction (`start-1`/`end-1`, centred
        // on the cross axis).
        let (before, after) = *arrows.read(cx);
        let step = px(120.);
        // `.tabs__list-container__scroll-prev` and
        // `.tabs__list-container__scroll-next` are `size-4` circles at the
        // edges, shown only when there is something that way to scroll to.
        let arrow =
            |id: &str, icon: &'static str, delta: gpui::Pixels, handle: gpui::ScrollHandle| {
                gpui::div()
                    .id(gpui::ElementId::Name(format!("{base_id}-{id}").into()))
                    // gpui has no hitbox occlusion, so a chevron floating over
                    // the list hands its click to the tab underneath as well.
                    // v3's chevron is `z-2` above the `z-index: 1` tabs exactly
                    // so it takes the press; `occlude` stops the hit test at
                    // the button, which is that on-top layer.
                    .occlude()
                    .absolute()
                    .size(px(16.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .cursor_pointer()
                    .bg(colors.surface.background)
                    .text_color(colors.foreground)
                    .child(
                        gpui::svg()
                            .size(px(12.))
                            .path(icon)
                            .text_color(colors.foreground),
                    )
                    .on_click(move |_, _, _| {
                        let at = handle.offset();
                        let next = if vertical {
                            gpui::point(at.x, at.y + delta)
                        } else {
                            gpui::point(at.x + delta, at.y)
                        };
                        handle.set_offset(next);
                    })
            };
        let container = gpui::div()
            .relative()
            // A scroller only overflows if it is bounded: without `w_full` the
            // box grows to fit every tab and nothing ever scrolls.
            .when(!vertical, |c| c.w_full())
            .when(vertical, |c| c.h_full())
            .child(
                gpui::div()
                    .id(gpui::ElementId::Name(format!("{base_id}-scroller").into()))
                    // A flex box, so the `flex_shrink_0` list inside keeps its
                    // content width (`w-max`) instead of being stretched to the
                    // scroller -- a stretched list is never wider than its box
                    // and never scrolls.
                    .flex()
                    .when(!vertical, |e| e.w_full().overflow_x_scroll())
                    .when(vertical, |e| e.h_full().overflow_y_scroll())
                    .track_scroll(&scroll)
                    .child(list),
            )
            .child({
                // `max_offset` is written during prepaint, so the render that
                // decided whether to draw an arrow read the frame before. This
                // canvas reads it in place and stores what it found; the entity
                // update is what asks for the frame that draws them.
                let measured = arrows;
                let handle = scroll.clone();
                gpui::canvas(
                    move |_bounds, _window, cx| {
                        let offset = handle.offset();
                        let max = handle.max_offset();
                        let next = if vertical {
                            (
                                f32::from(offset.y) < -0.5,
                                f32::from(offset.y) - 0.5 > -f32::from(max.height),
                            )
                        } else {
                            (
                                f32::from(offset.x) < -0.5,
                                f32::from(offset.x) - 0.5 > -f32::from(max.width),
                            )
                        };
                        if *measured.read(cx) != next {
                            measured.update(cx, |flags, cx| {
                                *flags = next;
                                cx.notify();
                            });
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .size(px(0.))
            })
            .when(before, |c| {
                let a = arrow(
                    "scroll-prev",
                    crate::icons::CHEVRON_LEFT,
                    step,
                    scroll.clone(),
                );
                c.child(if vertical {
                    a.top(px(4.)).left(gpui::relative(0.5)).ml(px(-8.))
                } else {
                    // `start-1 top-1/2 -translate-y-1/2`.
                    a.left(px(4.)).top(gpui::relative(0.5)).mt(px(-8.))
                })
            })
            .when(after, |c| {
                let a = arrow(
                    "scroll-next",
                    crate::icons::CHEVRON_RIGHT,
                    -step,
                    scroll.clone(),
                );
                c.child(if vertical {
                    a.bottom(px(4.)).left(gpui::relative(0.5)).ml(px(-8.))
                } else {
                    a.right(px(4.)).top(gpui::relative(0.5)).mt(px(-8.))
                })
            });

        // `.tabs` is `flex gap-2`: the gap between the list and the panel.
        let mut el = gpui::div().flex().flex_col().gap(px(8.)).child(container);

        if let Some(idx) = active_idx {
            if let Some(content) = items.swap_remove(idx).content {
                // `.tabs__panel` is `w-full p-2`.
                el = el.child(gpui::div().w_full().p(px(8.)).child(content));
            }
        }

        el
    }
}
