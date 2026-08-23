//! TagGroup — port of `@heroui/tag-group` (v3).
//!
//! A focusable list of tags with optional selection and removal. Mirrors the
//! React API: `selectionMode`, `selectedKeys`, `disabledKeys`, `isDisabled`,
//! `onRemove`, `onSelectionChange`, `size` and the `default | surface` variant.

use std::collections::HashSet;
use std::sync::Arc;

use gpui::{
    div, prelude::*, px, App, ElementId, InteractiveElement, IntoElement, RenderOnce, SharedString,
    Styled, Window,
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

    fn metrics(size: Size) -> (gpui::Pixels, gpui::Pixels, gpui::Pixels) {
        match size {
            Size::Sm => (px(22.), px(8.), px(11.)),
            Size::Md => (px(26.), px(10.), px(12.)),
            Size::Lg => (px(32.), px(12.), px(14.)),
        }
    }
}

impl RenderOnce for TagGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // One tab stop per tag. `use_keyed_state` takes `cx` mutably, so the
        // handles come before the theme is borrowed.
        let tag_focus: Vec<gpui::FocusHandle> = (0..self.tags.len())
            .map(|index| {
                crate::util::tab_stop_handle(
                    ElementId::Name(format!("{:?}-tag-{index}-focus", self.id).into()),
                    window,
                    cx,
                )
            })
            .collect();
        let ring_visible = crate::util::focus_visible(cx);
        let colors = cx.colors();
        let layout = cx.layout();
        let (height, pad_x, text_size) = Self::metrics(self.size);

        let mut root = div().flex().flex_col().gap(px(6.));

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
                    .text_size(px(text_size.into()))
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
                .when_some(tag_focus.get(index).filter(|_| !disabled), |c, handle| {
                    c.track_focus(handle)
                })
                .flex()
                .flex_row()
                .items_center()
                .gap(px(4.))
                .h(height)
                .px(pad_x)
                .rounded(px(f32::from(height) / 2.))
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

            chip = chip.child(tag.label.to_string());

            if let Some(on_remove) = self.on_remove.clone() {
                let key = tag.key.clone();
                let mut close = div()
                    .id(ElementId::Name(
                        format!("{:?}-tag-{index}-remove", self.id).into(),
                    ))
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(14.))
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
                    && tag_focus.get(index).is_some_and(|h| h.is_focused(window)),
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
