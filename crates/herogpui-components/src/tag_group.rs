//! TagGroup — port of `@heroui/tag-group` (v3).
//!
//! A focusable list of tags with optional selection and removal. Mirrors the
//! React API: `selectionMode`, `selectedKeys`, `disabledKeys`, `isDisabled`,
//! `onRemove`, `onSelectionChange`, `size` and the `default | surface` variant.

use std::collections::HashSet;
use std::sync::Arc;

use gpui::{
    div, prelude::*, px, AnyElement, App, ElementId, InteractiveElement, IntoElement, RenderOnce,
    SharedString, Styled, Window,
};
use herogpui_core::{SelectionMode, Size};
use herogpui_theme::ActiveTheme;

use crate::icons;

/// Visual variant of the tags in a group.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TagVariant {
    /// Filled with `default`.
    #[default]
    Default,
    /// Flat on the surface with a border.
    Surface,
}

impl TagVariant {
    pub const ALL: [TagVariant; 2] = [TagVariant::Default, TagVariant::Surface];

    pub fn label(self) -> &'static str {
        match self {
            TagVariant::Default => "Default",
            TagVariant::Surface => "Surface",
        }
    }
}

/// One tag in a [`TagGroup`].
#[derive(Clone)]
pub struct Tag {
    key: SharedString,
    label: SharedString,
    icon: Option<SharedString>,
    is_disabled: bool,
}

impl Tag {
    pub fn new(key: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            icon: None,
            is_disabled: false,
        }
    }

    pub fn icon(mut self, path: impl Into<SharedString>) -> Self {
        self.icon = Some(path.into());
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    pub fn key(&self) -> &SharedString {
        &self.key
    }
}

type OnSelectionChange = Arc<dyn Fn(&HashSet<SharedString>, &mut Window, &mut App) + 'static>;
type OnRemove = Arc<dyn Fn(&SharedString, &mut Window, &mut App) + 'static>;

/// HeroUI TagGroup.
#[derive(IntoElement)]
pub struct TagGroup {
    id: ElementId,
    tags: Vec<Tag>,
    label: Option<SharedString>,
    description: Option<SharedString>,
    selection_mode: SelectionMode,
    selected_keys: HashSet<SharedString>,
    disabled_keys: HashSet<SharedString>,
    is_disabled: bool,
    size: Size,
    variant: TagVariant,
    /// `Tag`'s `children`-as-a-function: handed the interactive state and drawn
    /// in place of the label.
    tag_content: Option<Arc<dyn Fn(&Tag, crate::util::InteractiveState) -> AnyElement + 'static>>,
    /// Shown in place of the list when `tags` is empty.
    empty_state: Option<SharedString>,
    on_selection_change: Option<OnSelectionChange>,
    on_remove: Option<OnRemove>,
}

impl TagGroup {
    pub fn new(id: impl Into<ElementId>, tags: Vec<Tag>) -> Self {
        Self {
            id: id.into(),
            tags,
            tag_content: None,
            label: None,
            description: None,
            selection_mode: SelectionMode::None,
            selected_keys: HashSet::new(),
            disabled_keys: HashSet::new(),
            is_disabled: false,
            size: Size::Md,
            variant: TagVariant::Default,
            empty_state: None,
            on_selection_change: None,
            on_remove: None,
        }
    }

    pub fn label(mut self, text: impl Into<SharedString>) -> Self {
        self.label = Some(text.into());
        self
    }

    pub fn description(mut self, text: impl Into<SharedString>) -> Self {
        self.description = Some(text.into());
        self
    }

    pub fn selection_mode(mut self, mode: SelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }

    pub fn selected_keys(mut self, keys: impl IntoIterator<Item = SharedString>) -> Self {
        self.selected_keys = keys.into_iter().collect();
        self
    }

    pub fn disabled_keys(mut self, keys: impl IntoIterator<Item = SharedString>) -> Self {
        self.disabled_keys = keys.into_iter().collect();
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// v3's render function for a tag's children, handed `isHovered`,
    /// `isPressed`, `isFocused`, `isFocusVisible` and `isSelected` -- and the tag
    /// itself, which the closure needs to know what it is drawing.
    ///
    /// The hover and the press are a frame behind the pointer: gpui reports both
    /// to a handler, not to the render that draws them.
    pub fn tag_content(
        mut self,
        render: impl Fn(&Tag, crate::util::InteractiveState) -> AnyElement + 'static,
    ) -> Self {
        self.tag_content = Some(Arc::new(render));
        self
    }

    pub fn variant(mut self, variant: TagVariant) -> Self {
        self.variant = variant;
        self
    }

    /// `TagGroup.List` renders this when there is nothing to show.
    pub fn empty_state(mut self, text: impl Into<SharedString>) -> Self {
        self.empty_state = Some(text.into());
        self
    }

    pub fn on_selection_change(
        mut self,
        handler: impl Fn(&HashSet<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change = Some(Arc::new(handler));
        self
    }

    /// Adds a remove button to every tag.
    pub fn on_remove(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_remove = Some(Arc::new(handler));
        self
    }

    /// `(px, py, text)` from `.tag--sm` / `--md` / `--lg`.
    ///
    /// v3 gives a tag no height: it is padding around one line, which is why
    /// this returns a vertical padding rather than the box it used to force.
    fn metrics(size: Size) -> (gpui::Pixels, gpui::Pixels, gpui::Pixels) {
        match size {
            Size::Sm => (px(8.), px(2.), px(12.)),
            Size::Md => (px(8.), px(4.), px(12.)),
            Size::Lg => (px(10.), px(6.), px(14.)),
        }
    }

    /// `rounded-xl` on `.tag`, `rounded-2xl` on `.tag--lg`.
    fn radius(size: Size, cx: &App) -> gpui::Pixels {
        match size {
            Size::Sm | Size::Md => crate::util::small_radius(cx),
            Size::Lg => crate::util::soft_radius(cx),
        }
    }
}

impl RenderOnce for TagGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // A tag group is *one* tab stop: React Aria roves the tabindex, so Tab
        // enters the group once and the arrows move inside it. Which tag claims
        // the handle is held here, because a handle's `tab_stop` is fixed where
        // the handle is made. `use_keyed_state` takes `cx` mutably, so both
        // precede the theme.
        let group_focus = crate::util::tab_stop_handle(
            ElementId::Name(format!("{:?}-focus", self.id).into()),
            window,
            cx,
        );
        let cursor = window.use_keyed_state(
            ElementId::Name(format!("{:?}-cursor", self.id).into()),
            cx,
            |_, _| 0usize,
        );
        // Removing a tag shortens the list and disabled tags take no focus, so
        // the stop lands on the first enabled tag at or after the cursor.
        let enabled: Vec<usize> = self
            .tags
            .iter()
            .enumerate()
            .filter(|(_, t)| {
                !(self.is_disabled || t.is_disabled || self.disabled_keys.contains(&t.key))
            })
            .map(|(i, _)| i)
            .collect();
        let at = *cursor.read(cx);
        let cursor_index = enabled
            .iter()
            .copied()
            .find(|i| *i >= at)
            .or_else(|| enabled.first().copied());
        // One hover/press slot per tag, for a `tag_content` closure.
        let interaction: Vec<crate::util::Interaction> = if self.tag_content.is_some() {
            (0..self.tags.len())
                .map(|index| {
                    crate::util::interaction(
                        ElementId::Name(format!("{:?}-tag-{index}-interaction", self.id).into()),
                        window,
                        cx,
                    )
                })
                .collect()
        } else {
            Vec::new()
        };
        let ring_visible = crate::util::focus_visible(cx);
        let colors = cx.colors();
        let layout = cx.layout();
        let (pad_x, pad_y, text_size) = Self::metrics(self.size);
        let tag_radius = Self::radius(self.size, cx);

        // `.tag-group` is `flex flex-col gap-1`: the label, the list and the
        // description.
        let mut root = div().flex().flex_col().gap(px(4.));

        if let Some(label) = &self.label {
            root = root.child(
                div()
                    .text_size(px(14.))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(colors.foreground)
                    .child(label.to_string()),
            );
        }

        if self.tags.is_empty() {
            let text = self
                .empty_state
                .unwrap_or_else(|| SharedString::from("No tags"));
            root = root.child(
                div()
                    // `.empty-state` is `p-2 text-sm text-muted`.
                    .p(px(8.))
                    .text_size(px(14.))
                    .text_color(colors.muted)
                    .child(text.to_string()),
            );
            return root;
        }

        let mut list = div().flex().flex_row().flex_wrap().gap(px(6.));

        for (index, tag) in self.tags.iter().enumerate() {
            let disabled =
                self.is_disabled || tag.is_disabled || self.disabled_keys.contains(&tag.key);
            let selected = self.selected_keys.contains(&tag.key);
            let selectable = self.selection_mode != SelectionMode::None;

            let mut chip = div()
                .id(ElementId::Name(format!("{:?}-tag-{index}", self.id).into()))
                .when(!disabled && cursor_index == Some(index), |c| {
                    c.track_focus(&group_focus)
                })
                .flex()
                .flex_row()
                .items_center()
                .gap(px(4.))
                .px(pad_x)
                .py(pad_y)
                .rounded(tag_radius)
                .text_size(text_size)
                .whitespace_nowrap();

            chip = if selected {
                chip.bg(colors.accent.soft())
                    .text_color(colors.accent.soft_foreground())
            } else {
                match self.variant {
                    TagVariant::Default => chip
                        .bg(colors.default.color)
                        .text_color(colors.default.foreground),
                    TagVariant::Surface => chip
                        .bg(colors.surface.background)
                        .border(layout.border_width)
                        .border_color(colors.border)
                        .text_color(colors.foreground),
                }
            };

            if disabled {
                chip = chip.opacity(layout.disabled_opacity);
            } else if selectable {
                let hover = if selected {
                    colors.accent.soft_hover()
                } else {
                    colors.default.hover()
                };
                chip = chip.cursor_pointer().hover(move |s| s.bg(hover));
            }

            if let Some(path) = &tag.icon {
                chip = chip.child(
                    gpui::svg()
                        .size(self.size.icon_size())
                        .path(path.clone())
                        .flex_shrink_0()
                        .text_color(colors.muted),
                );
            }

            chip = match &self.tag_content {
                Some(render) => {
                    let (is_hovered, is_pressed) = interaction
                        .get(index)
                        .map(|slot| *slot.read(cx))
                        .unwrap_or_default();
                    let focused = !disabled && cursor_index == Some(index);
                    chip.child(render(
                        tag,
                        crate::util::InteractiveState {
                            is_hovered,
                            is_pressed,
                            is_focused: focused,
                            is_focus_visible: focused && ring_visible,
                            is_selected: selected,
                            is_disabled: disabled,
                            is_indeterminate: false,
                        },
                    ))
                }
                None => chip.child(tag.label.to_string()),
            };
            if let Some(slot) = interaction.get(index) {
                chip = crate::util::track_interaction(chip, slot);
            }

            if let Some(on_remove) = self.on_remove.clone() {
                let key = tag.key.clone();
                let mut close = div()
                    .id(ElementId::Name(
                        format!("{:?}-tag-{index}-remove", self.id).into(),
                    ))
                    .flex()
                    .items_center()
                    .justify_center()
                    // `.tag__remove-button` is `size-3`.
                    .size(px(12.))
                    .rounded_full()
                    .flex_shrink_0()
                    // gpui svgs need an explicit color; they do not inherit.
                    .child(
                        gpui::svg()
                            .size(px(10.))
                            .path(icons::CLOSE)
                            .text_color(colors.muted),
                    );
                if !disabled {
                    let hover_bg = colors.default.hover();
                    close = close
                        .cursor_pointer()
                        .hover(move |s| s.bg(hover_bg))
                        .on_click(move |_, window, cx| on_remove(&key, window, cx));
                }
                chip = chip.child(close);
            }

            // React Aria's TagGroup: the arrows move between tags and Delete or
            // Backspace removes the focused one.
            if !disabled {
                let stops = enabled.clone();
                let moved = cursor.clone();
                let remove = self.on_remove.clone();
                let key_for_remove = tag.key.clone();
                chip =
                    chip.on_key_down(
                        move |event, window, cx| match event.keystroke.key.as_str() {
                            "delete" | "backspace" => {
                                if let Some(cb) = &remove {
                                    cx.stop_propagation();
                                    cb(&key_for_remove, window, cx);
                                }
                            }
                            key @ ("left" | "right" | "home" | "end") => {
                                let key = match key {
                                    "right" => "down",
                                    "left" => "up",
                                    other => other,
                                };
                                // React Aria gives TagGroup a horizontal
                                // keyboard delegate. The list owns its axis
                                // and Home/End; Up/Down fall through to an
                                // enclosing scroller.
                                cx.stop_propagation();
                                let crate::list_nav::Move::To(next) =
                                    crate::list_nav::resolve(&stops, Some(index), key, false)
                                else {
                                    return;
                                };
                                // No refocusing: the next render has the tag
                                // at `next` claim the group's handle, so the
                                // focus goes with it.
                                moved.update(cx, |v, cx| {
                                    *v = next;
                                    cx.notify();
                                });
                            }
                            _ => {}
                        },
                    );
            }

            if selectable && !disabled {
                let key = tag.key.clone();
                let mode = self.selection_mode;
                let current = self.selected_keys.clone();
                let on_change = self.on_selection_change.clone();
                chip = chip.on_click(move |_, window, cx| {
                    if let Some(change) = &on_change {
                        let next = match mode {
                            SelectionMode::None => current.clone(),
                            SelectionMode::Single => {
                                if current.contains(&key) {
                                    HashSet::new()
                                } else {
                                    HashSet::from([key.clone()])
                                }
                            }
                            SelectionMode::Multiple => {
                                let mut set = current.clone();
                                if !set.remove(&key) {
                                    set.insert(key.clone());
                                }
                                set
                            }
                        };
                        change(&next, window, cx);
                    }
                });
            }

            // `.tag:focus-visible` is `status-focused`.
            let chip = crate::util::with_focus_ring(
                chip,
                !disabled
                    && ring_visible
                    && cursor_index == Some(index)
                    && group_focus.is_focused(window),
                true,
                Vec::new(),
                cx,
            );
            list = list.child(chip);
        }

        root = root.child(list);

        if let Some(description) = &self.description {
            root = root.child(
                div()
                    .text_size(px(12.))
                    .text_color(colors.muted)
                    .child(description.to_string()),
            );
        }

        root
    }
}
