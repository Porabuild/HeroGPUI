//! Tabs — port of `@heroui/tabs`.

use std::{cell::Cell, rc::Rc, time::Duration};

use gpui::{
    prelude::*, px, Animation, AnimationExt, AnyElement, App, InteractiveElement, IntoElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window,
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

/// `.tabs__separator` transitions its opacity for 150ms with `--ease-smooth`.
const SEPARATOR_TRANSITION_MS: u64 = 150;
/// `.tabs__indicator` transitions translate, width and height for 250ms with
/// `--ease-out-fluid`.
const INDICATOR_TRANSITION_MS: u64 = 250;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct IndicatorRect {
    x: gpui::Pixels,
    y: gpui::Pixels,
    width: gpui::Pixels,
    height: gpui::Pixels,
}

#[derive(Clone)]
struct IndicatorMotion {
    target: IndicatorRect,
    generation: usize,
    from: IndicatorRect,
    rect: Rc<Cell<IndicatorRect>>,
}

struct IndicatorMotionFrame {
    generation: usize,
    from: IndicatorRect,
    to: IndicatorRect,
    rect: Rc<Cell<IndicatorRect>>,
    animate: bool,
}

impl IndicatorMotionFrame {
    fn render(self, indicator: gpui::Div) -> AnyElement {
        if !self.animate {
            self.rect.set(self.to);
            return place_indicator(indicator, self.to).into_any_element();
        }

        let rect = self.rect;
        let from = self.from;
        let to = self.to;
        indicator
            .with_animation(
                gpui::ElementId::Name(format!("tabs-indicator-slide-{}", self.generation).into()),
                Animation::new(Duration::from_millis(INDICATOR_TRANSITION_MS))
                    .with_easing(|t| crate::anim::Curve::OutFluid.at(t)),
                move |indicator, delta| {
                    let next = IndicatorRect {
                        x: from.x + (to.x - from.x) * delta,
                        y: from.y + (to.y - from.y) * delta,
                        width: from.width + (to.width - from.width) * delta,
                        height: from.height + (to.height - from.height) * delta,
                    };
                    rect.set(next);
                    place_indicator(indicator, next)
                },
            )
            .into_any_element()
    }
}

fn place_indicator(indicator: gpui::Div, rect: IndicatorRect) -> gpui::Div {
    indicator
        .left(rect.x)
        .top(rect.y)
        .w(rect.width)
        .h(rect.height)
}

fn indicator_motion(
    id: gpui::ElementId,
    target: IndicatorRect,
    window: &mut Window,
    cx: &mut App,
) -> IndicatorMotionFrame {
    let state = window.use_keyed_state(id, cx, |_, _| IndicatorMotion {
        target,
        generation: 0,
        from: target,
        rect: Rc::new(Cell::new(target)),
    });
    let mut current = state.read(cx).clone();
    if current.target != target {
        current.target = target;
        current.generation = current.generation.wrapping_add(1);
        current.from = current.rect.get();
        state.update(cx, |stored, _| *stored = current.clone());
    }
    if cx.reduce_motion() && current.rect.get() != target {
        current.from = target;
        current.rect.set(target);
        state.update(cx, |stored, _| *stored = current.clone());
    }
    IndicatorMotionFrame {
        generation: current.generation,
        from: current.from,
        to: target,
        rect: current.rect,
        animate: current.generation != 0 && !cx.reduce_motion() && current.from != target,
    }
}

#[derive(Clone, Debug, Default)]
struct TabsGeometry {
    list: Option<gpui::Bounds<gpui::Pixels>>,
    tabs: Vec<(SharedString, gpui::Bounds<gpui::Pixels>)>,
}

fn indicator_target(
    geometry: &TabsGeometry,
    selected_key: &SharedString,
    vertical: bool,
    secondary: bool,
) -> Option<IndicatorRect> {
    let list = geometry.list?;
    let tab = geometry
        .tabs
        .iter()
        .find_map(|(key, bounds)| (key == selected_key).then_some(*bounds))?;
    let x = tab.origin.x - list.origin.x;
    let y = tab.origin.y - list.origin.y;
    Some(if secondary && vertical {
        IndicatorRect {
            x: px(0.),
            y,
            width: px(2.),
            height: tab.size.height,
        }
    } else if secondary {
        IndicatorRect {
            x,
            y: list.size.height - px(2.),
            width: tab.size.width,
            height: px(2.),
        }
    } else {
        IndicatorRect {
            x,
            y,
            width: tab.size.width,
            height: tab.size.height,
        }
    })
}

#[derive(Clone)]
struct SeparatorMotion {
    hidden: bool,
    generation: usize,
    from: f32,
    opacity: Rc<Cell<f32>>,
}

struct SeparatorMotionFrame {
    generation: usize,
    from: f32,
    to: f32,
    opacity: Rc<Cell<f32>>,
    animate: bool,
}

impl SeparatorMotionFrame {
    fn render(self, separator: gpui::Div) -> AnyElement {
        if !self.animate {
            self.opacity.set(self.to);
            return separator.opacity(self.to).into_any_element();
        }

        let opacity = self.opacity;
        let from = self.from;
        let to = self.to;
        separator
            .with_animation(
                gpui::ElementId::Name(format!("tabs-separator-fade-{}", self.generation).into()),
                Animation::new(Duration::from_millis(SEPARATOR_TRANSITION_MS))
                    .with_easing(|t| crate::anim::Curve::Smooth.at(t)),
                move |separator, delta| {
                    let next = from + (to - from) * delta;
                    opacity.set(next);
                    separator.opacity(next)
                },
            )
            .into_any_element()
    }
}

fn separator_motion(
    id: gpui::ElementId,
    hidden: bool,
    window: &mut Window,
    cx: &mut App,
) -> SeparatorMotionFrame {
    let target = if hidden { 0. } else { 1. };
    let state = window.use_keyed_state(id, cx, |_, _| SeparatorMotion {
        hidden,
        generation: 0,
        from: target,
        opacity: Rc::new(Cell::new(target)),
    });
    let mut current = state.read(cx).clone();
    if current.hidden != hidden {
        current.hidden = hidden;
        current.generation = current.generation.wrapping_add(1);
        current.from = current.opacity.get();
        state.update(cx, |stored, _| *stored = current.clone());
    }
    if cx.reduce_motion() && (current.opacity.get() - target).abs() > f32::EPSILON {
        current.from = target;
        current.opacity.set(target);
        state.update(cx, |stored, _| *stored = current.clone());
    }
    SeparatorMotionFrame {
        generation: current.generation,
        from: current.from,
        to: target,
        opacity: current.opacity,
        animate: current.generation != 0
            && !cx.reduce_motion()
            && (current.from - target).abs() > f32::EPSILON,
    }
}

/// When arrow-key focus changes become selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KeyboardActivation {
    /// Select as focus moves, which is React Aria's default.
    #[default]
    Automatic,
    /// Move focus only; Enter or Space selects the focused tab.
    Manual,
}

/// One tab: key + label + panel content.
pub struct TabItem {
    pub key: SharedString,
    pub label: SharedString,
    pub content: Option<AnyElement>,
    /// `Tabs.Tab.isDisabled` — removes this tab from activation and the roving
    /// keyboard stops without disabling its siblings.
    pub is_disabled: bool,
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
            is_disabled: false,
            separator: false,
        }
    }

    pub fn content(mut self, el: impl IntoElement) -> Self {
        self.content = Some(el.into_any_element());
        self
    }

    pub fn is_disabled(mut self, value: bool) -> Self {
        self.is_disabled = value;
        self
    }

    /// Composes a `Tabs.Separator` into this tab: a hairline before it.
    pub fn separator(mut self) -> Self {
        self.separator = true;
        self
    }
}

type OnChange = std::sync::Arc<dyn Fn(&SharedString, &mut Window, &mut App) + 'static>;

#[derive(Clone)]
struct TabFocusState {
    key: SharedString,
    selected_key: SharedString,
    enabled_keys: Vec<SharedString>,
}

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
    keyboard_activation: KeyboardActivation,
    on_selection_change: Option<OnChange>,
}

impl Tabs {
    /// `orientation` — a vertical tab list stacks its tabs.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// React Aria's inherited `keyboardActivation`.
    pub fn keyboard_activation(mut self, activation: KeyboardActivation) -> Self {
        self.keyboard_activation = activation;
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
            keyboard_activation: KeyboardActivation::Automatic,
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
        let first_enabled = self
            .items
            .iter()
            .find(|item| !item.is_disabled)
            .or_else(|| self.items.first())
            .map(|item| item.key.clone());
        let fallback = self
            .default_selected_key
            .clone()
            .filter(|key| self.items.iter().any(|item| item.key == *key))
            .or(first_enabled.clone())
            .unwrap_or_default();
        // One handle for the list: a tab list is one tab stop and the focused
        // tab claims it, which is how the stop roves. Flipping a handle's
        // `tab_stop` cannot do that -- it is fixed where the handle is made.
        let list_focus = crate::util::tab_stop_handle(
            gpui::ElementId::Name(format!("{base_id}-focus").into()),
            window,
            cx,
        );
        let (mut selected_key, selection_own) = crate::util::controlled(
            window,
            cx,
            gpui::ElementId::Name(format!("{base_id}-selected").into()),
            self.selected_key.clone(),
            fallback,
        );
        // Pinned `useTabListState` repairs an uncontrolled selection when its
        // item disappears from the collection. Use the repaired key in this
        // frame as well as storing it, so the replacement panel and roving
        // stop are never absent for a frame.
        if self.selected_key.is_none() && !self.items.iter().any(|item| item.key == selected_key) {
            if let (Some(next), Some(held)) = (first_enabled.clone(), selection_own.as_ref()) {
                selected_key = next.clone();
                held.update(cx, |value, cx| {
                    *value = next.clone();
                    cx.notify();
                });
                if let Some(cb) = &self.on_selection_change {
                    cb(&next, window, cx);
                }
            }
        }

        // React Aria keeps the roving focus key separate from selectedKey.
        // That distinction matters for controlled tabs: arrow keys move focus
        // and report each newly focused key without mutating the selection the
        // caller still owns.
        let focus_seed = self
            .items
            .iter()
            .find(|item| item.key == selected_key && !item.is_disabled)
            .map(|item| item.key.clone())
            .or(first_enabled)
            .unwrap_or_else(|| selected_key.clone());
        let enabled_keys: Vec<SharedString> = self
            .items
            .iter()
            .filter(|item| !item.is_disabled)
            .map(|item| item.key.clone())
            .collect();
        let focus_state = window.use_keyed_state(
            gpui::ElementId::Name(format!("{base_id}-focused-key").into()),
            cx,
            |_, _| TabFocusState {
                key: focus_seed.clone(),
                selected_key: selected_key.clone(),
                enabled_keys: enabled_keys.clone(),
            },
        );
        let mut focus_now = focus_state.read(cx).clone();
        let focus_valid = enabled_keys.contains(&focus_now.key);
        if focus_now.selected_key != selected_key && !list_focus.is_focused(window) {
            focus_now.key = focus_seed;
        } else if !focus_valid {
            let old_index = focus_now
                .enabled_keys
                .iter()
                .position(|key| key == &focus_now.key);
            focus_now.key = old_index
                .and_then(|index| {
                    focus_now.enabled_keys[index + 1..]
                        .iter()
                        .find(|key| enabled_keys.contains(key))
                        .or_else(|| {
                            focus_now.enabled_keys[..index]
                                .iter()
                                .rev()
                                .find(|key| enabled_keys.contains(key))
                        })
                        .cloned()
                })
                .unwrap_or(focus_seed);
        }
        if focus_now.selected_key != selected_key || focus_now.enabled_keys != enabled_keys {
            focus_now.selected_key = selected_key.clone();
            focus_now.enabled_keys = enabled_keys;
            focus_state.update(cx, |state, _| *state = focus_now.clone());
        }
        let focused_key = focus_now.key;

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
        let vertical = self.orientation == Orientation::Vertical;
        let secondary = self.variant == TabsVariant::Secondary;
        let geometry = window.use_keyed_state(
            gpui::ElementId::Name(format!("{base_id}-geometry").into()),
            cx,
            |_, _| TabsGeometry::default(),
        );
        let indicator_frame =
            indicator_target(geometry.read(cx), &selected_key, vertical, secondary).map(|target| {
                indicator_motion(
                    gpui::ElementId::Name(format!("{base_id}-indicator-motion").into()),
                    target,
                    window,
                    cx,
                )
            });
        let mut separator_motions = self
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                (self.variant == TabsVariant::Primary && index > 0 && item.separator).then(|| {
                    let hidden =
                        item.key == selected_key || self.items[index - 1].key == selected_key;
                    separator_motion(
                        gpui::ElementId::Name(format!("{base_id}-separator-{}", item.key).into()),
                        hidden,
                        window,
                        cx,
                    )
                })
            })
            .collect::<Vec<_>>();

        let colors = cx.colors();
        let layout = cx.layout();
        let key_stops: Vec<usize> = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| (!item.is_disabled).then_some(index))
            .collect();
        let key_keys: Vec<SharedString> = self.items.iter().map(|item| item.key.clone()).collect();

        // `.tabs__list` is `w-max min-w-full`: it grows with its content, which is
        // what lets the scroller overflow -- a shrinking row always fits and
        // never scrolls.
        let mut list = gpui::div().relative().flex().flex_shrink_0().child({
            let measured = geometry.clone();
            gpui::canvas(
                move |bounds, _window, cx| {
                    if measured.read(cx).list != Some(bounds) {
                        measured.update(cx, |geometry, cx| {
                            geometry.list = Some(bounds);
                            cx.notify();
                        });
                    }
                },
                |_, _, _, _| {},
            )
            .absolute()
            .inset_0()
        });
        if vertical {
            list = list.flex_col().items_start().gap(px(4.));
        } else {
            list = list.min_w_full();
        }

        // v3 keeps two indicator styles: `primary` fills a segment behind the
        // selected tab, `secondary` underlines it.
        let indicator_ready = indicator_frame.is_some();
        if let Some(frame) = indicator_frame {
            // `.tabs__indicator` is the absolute `rounded-3xl bg-segment
            // shadow-surface` pill; Secondary flattens it into an accent line.
            let mut indicator = gpui::div()
                .absolute()
                .debug_selector(|| format!("{base_id}-indicator"));
            indicator = if secondary {
                indicator.bg(colors.accent.color)
            } else {
                indicator
                    .rounded(crate::util::control_radius(cx))
                    .bg(colors.segment.background)
                    .when(!layout.surface_shadow.is_empty(), |indicator| {
                        indicator.shadow(layout.surface_shadow.clone())
                    })
            };
            list = list.child(frame.render(indicator));
        }
        let measure_tab = |key: SharedString| {
            let measured = geometry.clone();
            gpui::canvas(
                move |bounds, _window, cx| {
                    let current = measured
                        .read(cx)
                        .tabs
                        .iter()
                        .find_map(|(held, bounds)| (held == &key).then_some(*bounds));
                    if current != Some(bounds) {
                        measured.update(cx, |geometry, cx| {
                            if let Some((_, held)) =
                                geometry.tabs.iter_mut().find(|(held, _)| held == &key)
                            {
                                *held = bounds;
                            } else {
                                geometry.tabs.push((key.clone(), bounds));
                            }
                            cx.notify();
                        });
                    }
                },
                |_, _, _, _| {},
            )
            .absolute()
            .inset_0()
        };
        match self.variant {
            TabsVariant::Primary => {
                // `.tabs__list` is `p-1` and nothing else: the tabs sit
                // shoulder to shoulder, with no gap between them.
                list = list.p(px(4.));
                for (index, item) in self.items.iter().enumerate() {
                    let active = item.key == selected_key;
                    let focused = item.key == focused_key;
                    let disabled = self.is_disabled || item.is_disabled;
                    let mut tab = gpui::div()
                        .id(gpui::ElementId::Name(
                            format!("{base_id}-tab-{}", item.key).into(),
                        ))
                        .relative()
                        .when(!disabled && focused, |t| t.track_focus(&list_focus))
                        // `.tabs__tab` is `h-8 px-4 rounded-3xl text-sm
                        // font-medium`.
                        .h(px(32.))
                        .px(px(16.))
                        .when(vertical, |t| t.w_full().min_w(px(80.)))
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
                        .when(!disabled, |t| t.cursor_pointer())
                        // `status-disabled` is `--disabled-opacity`.
                        .when(disabled, |t| t.opacity(cx.layout().disabled_opacity));
                    tab = tab.child(measure_tab(item.key.clone()));
                    // `.tabs__separator` is the absolute `w-px h-1/2
                    // rounded-sm` hairline inside the tab that carries it. It
                    // turns into `h-px w-[90%]` in a vertical list and hides
                    // beside the selected segment.
                    if index > 0 && item.separator {
                        let separator = gpui::div()
                            .absolute()
                            .rounded(crate::util::hairline_radius(cx))
                            .bg(colors.muted.alpha(0.25));
                        let separator = if vertical {
                            separator
                                .left(gpui::relative(0.05))
                                .top(px(0.))
                                .w(gpui::relative(0.9))
                                .h(cx.layout().border_width)
                        } else {
                            separator
                                .left(px(0.))
                                .top(gpui::relative(0.25))
                                .w(cx.layout().border_width)
                                .h(gpui::relative(0.5))
                        };
                        let separator = separator_motions[index]
                            .take()
                            .expect("primary separators have motion state")
                            .render(separator);
                        tab = tab.child(separator);
                    }
                    if active {
                        tab = tab
                            .text_color(colors.segment.foreground)
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .when(!indicator_ready, |tab| {
                                tab.bg(colors.segment.background)
                                    .when(!layout.surface_shadow.is_empty(), |tab| {
                                        tab.shadow(layout.surface_shadow.clone())
                                    })
                            });
                    } else {
                        tab = tab.text_color(colors.muted);
                        if !disabled {
                            tab = tab.hover(|s| s.opacity(0.7));
                        }
                    }
                    if !disabled {
                        // A tab list is one stop and the arrows move within
                        // it. Automatic activation selects as focus moves;
                        // manual activation waits for a press.
                        let key_stops = key_stops.clone();
                        let key_keys = key_keys.clone();
                        let key_cb = self.on_selection_change.clone();
                        let key_own = selection_own.clone();
                        let key_focus = focus_state.clone();
                        let automatic = self.keyboard_activation == KeyboardActivation::Automatic;
                        tab = tab.on_key_down(move |event, window, cx| {
                            let key = match (vertical, event.keystroke.key.as_str()) {
                                (_, "right") | (true, "down") => "down",
                                (_, "left") | (true, "up") => "up",
                                (_, other @ ("home" | "end")) => other,
                                _ => return,
                            };
                            // The list owns its axis and Home/End. Cross-axis
                            // keys returned above remain available to an
                            // enclosing scroller or navigation control.
                            cx.stop_propagation();
                            let crate::list_nav::Move::To(next) =
                                crate::list_nav::resolve(&key_stops, Some(index), key, true)
                            else {
                                return;
                            };
                            let Some(next_key) = key_keys.get(next).cloned() else {
                                return;
                            };
                            key_focus.update(cx, |state, cx| {
                                state.key = next_key.clone();
                                cx.notify();
                            });
                            if automatic {
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
                            }
                            // No refocusing: the next render has the newly
                            // focused tab claim the list's handle.
                        });
                        let key = item.key.clone();
                        let cb = self.on_selection_change.clone();
                        let own = selection_own.clone();
                        let focus = focus_state.clone();
                        let list_focus_for_click = list_focus.clone();
                        tab = tab.on_click(move |_, window, cx| {
                            window.focus(&list_focus_for_click);
                            focus.update(cx, |state, cx| {
                                state.key = key.clone();
                                cx.notify();
                            });
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
                        focused
                            && list_focus.is_focused(window)
                            && crate::util::focus_visible(cx)
                            && !disabled,
                        true,
                        Vec::new(),
                        cx,
                    );
                    list = list.child(tab.child(item.label.to_string()));
                }
            }
            TabsVariant::Secondary => {
                // `.tabs--secondary` gives the container a trailing-axis
                // border: bottom when horizontal, start when vertical.
                for (index, item) in self.items.iter().enumerate() {
                    let active = item.key == selected_key;
                    let focused = item.key == focused_key;
                    let disabled = self.is_disabled || item.is_disabled;
                    let mut tab = gpui::div()
                        .id(gpui::ElementId::Name(
                            format!("{base_id}-tab-{}", item.key).into(),
                        ))
                        .relative()
                        .when(!disabled && focused, |t| t.track_focus(&list_focus))
                        // The same `h-8 px-4 text-sm` box, `rounded-none`, with
                        // the indicator as a 2px bar along the bottom.
                        .h(px(32.))
                        .px(px(16.))
                        .when(vertical, |t| t.w_full().min_w(px(80.)))
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
                        .when(!indicator_ready && vertical, |t| t.border_l_2())
                        .when(!indicator_ready && !vertical, |t| t.border_b_2())
                        .when(!disabled, |t| t.cursor_pointer())
                        // `status-disabled` is `--disabled-opacity`.
                        .when(disabled, |t| t.opacity(cx.layout().disabled_opacity));
                    tab = tab.child(measure_tab(item.key.clone()));
                    tab = if active {
                        tab.text_color(colors.foreground)
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .when(!indicator_ready, |tab| {
                                tab.border_color(colors.accent.color)
                            })
                    } else {
                        tab.text_color(colors.muted).when(!indicator_ready, |tab| {
                            tab.border_color(gpui::transparent_black())
                        })
                    };
                    if !active && !disabled {
                        tab = tab.hover(|tab| tab.opacity(0.7));
                    }
                    if !disabled {
                        // A tab list is one stop and the arrows move within
                        // it. Automatic activation selects as focus moves;
                        // manual activation waits for a press.
                        let key_stops = key_stops.clone();
                        let key_keys = key_keys.clone();
                        let key_cb = self.on_selection_change.clone();
                        let key_own = selection_own.clone();
                        let key_focus = focus_state.clone();
                        let automatic = self.keyboard_activation == KeyboardActivation::Automatic;
                        tab = tab.on_key_down(move |event, window, cx| {
                            let key = match (vertical, event.keystroke.key.as_str()) {
                                (_, "right") | (true, "down") => "down",
                                (_, "left") | (true, "up") => "up",
                                (_, other @ ("home" | "end")) => other,
                                _ => return,
                            };
                            // The list owns its axis and Home/End. Cross-axis
                            // keys returned above remain available to an
                            // enclosing scroller or navigation control.
                            cx.stop_propagation();
                            let crate::list_nav::Move::To(next) =
                                crate::list_nav::resolve(&key_stops, Some(index), key, true)
                            else {
                                return;
                            };
                            let Some(next_key) = key_keys.get(next).cloned() else {
                                return;
                            };
                            key_focus.update(cx, |state, cx| {
                                state.key = next_key.clone();
                                cx.notify();
                            });
                            if automatic {
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
                            }
                            // No refocusing: the next render has the newly
                            // focused tab claim the list's handle.
                        });
                        let key = item.key.clone();
                        let cb = self.on_selection_change.clone();
                        let own = selection_own.clone();
                        let focus = focus_state.clone();
                        let list_focus_for_click = list_focus.clone();
                        tab = tab.on_click(move |_, window, cx| {
                            window.focus(&list_focus_for_click);
                            focus.update(cx, |state, cx| {
                                state.key = key.clone();
                                cx.notify();
                            });
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
                        focused
                            && list_focus.is_focused(window)
                            && crate::util::focus_visible(cx)
                            && !disabled,
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
        // `.tabs__list-container__scroll-prev` and
        // `.tabs__list-container__scroll-next` are `size-4` circles at the
        // edges, shown only when there is something that way to scroll to.
        let arrow = |id: &str, icon: &'static str, direction: f32, handle: gpui::ScrollHandle| {
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
                    .text_color(colors.foreground)
                    .hover(|arrow| arrow.opacity(0.7))
                    .child(
                        gpui::svg()
                            .size(px(12.))
                            .path(icon)
                            .text_color(colors.foreground),
                    )
                    .on_click(move |_, _, _| {
                        let at = handle.offset();
                        let viewport = handle.bounds().size;
                        let step = if vertical {
                            viewport.height * 0.8
                        } else {
                            viewport.width * 0.8
                        };
                        let delta = step * direction;
                        let next = if vertical {
                            gpui::point(at.x, at.y + delta)
                        } else {
                            gpui::point(at.x + delta, at.y)
                        };
                        handle.set_offset(next);
                    })
        };
        let container_radius = layout.radius_lg() * 2.5;
        let container = gpui::div()
            .relative()
            .when(!secondary, |c| {
                c.bg(colors.default.color)
                    .rounded(container_radius.min(px(32.)))
            })
            .when(secondary, |c| {
                c.border_color(colors.border)
                    .when(vertical, |c| c.border_l_1())
                    .when(!vertical, |c| c.border_b_1())
            })
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
                    if vertical {
                        crate::icons::CHEVRON_UP
                    } else {
                        crate::icons::CHEVRON_LEFT
                    },
                    1.,
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
                    if vertical {
                        crate::icons::CHEVRON_DOWN
                    } else {
                        crate::icons::CHEVRON_RIGHT
                    },
                    -1.,
                    scroll.clone(),
                );
                c.child(if vertical {
                    a.bottom(px(4.)).left(gpui::relative(0.5)).ml(px(-8.))
                } else {
                    a.right(px(4.)).top(gpui::relative(0.5)).mt(px(-8.))
                })
            });

        // `.tabs` is `flex gap-2`: horizontal tabs stack their panel below;
        // vertical tabs place it beside the list.
        let mut el = gpui::div()
            .flex()
            .when(vertical, |root| root.flex_row())
            .when(!vertical, |root| root.flex_col())
            .gap(px(8.))
            .child(container);

        if let Some(idx) = active_idx {
            if let Some(content) = items.swap_remove(idx).content {
                // `.tabs__panel` is `w-full p-2`, with `mt-4` horizontally or
                // `ms-4` vertically.
                el = el.child(
                    gpui::div()
                        .w_full()
                        .p(px(8.))
                        .when(vertical, |panel| panel.ml(px(16.)))
                        .when(!vertical, |panel| panel.mt(px(16.)))
                        .child(content),
                );
            }
        }

        el
    }
}
