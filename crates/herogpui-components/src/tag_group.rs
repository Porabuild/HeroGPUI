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

use crate::{icons, EscapeKeyBehavior};

/// Visual variant of the tags in a group.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TagVariant {
    /// Filled with `default`.
    #[default]
    Default,
    /// Flat on the surface.
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
type OnRemove = Arc<dyn Fn(&HashSet<SharedString>, &mut Window, &mut App) + 'static>;

/// Pinned React Stately `useMultipleSelectionState`'s anchor record: where a
/// Shift extension reaches from, how far the last one went, and whether the
/// selection is the raw `all` a `selectAll` produced.
#[derive(Clone, Debug, Default)]
struct TagSelectionRange {
    anchor: Option<SharedString>,
    current: Option<SharedString>,
    is_all: bool,
}

impl TagSelectionRange {
    /// Seats the range on `target`: the anchor stays whatever a first
    /// extension chose, the cursor lands on `target`, and a raw `all` ends.
    fn seat(&mut self, target: SharedString) {
        if self.anchor.is_none() {
            self.anchor = Some(target.clone());
        }
        self.current = Some(target);
        self.is_all = false;
    }
}

/// The selection after extending to `target` from `range`'s anchor.
///
/// The old anchor..current range is *replaced* by anchor..target, so extending
/// backwards shrinks again; a raw `all` collapses to the new key; a first
/// extension without an anchor selects from the target itself, which is what
/// the pinned SelectionManager does when nothing anchors it yet. Only
/// `selectable` keys enter the range, so disabled tags are skipped.
fn extend_selection_range(
    current: &HashSet<SharedString>,
    collection: &[SharedString],
    selectable: &HashSet<SharedString>,
    range: &TagSelectionRange,
    target: &SharedString,
) -> HashSet<SharedString> {
    if range.is_all {
        return HashSet::from([target.clone()]);
    }
    let anchor = range.anchor.as_ref().unwrap_or(target);
    let previous = range.current.as_ref().unwrap_or(target);
    let anchor_at = collection.iter().position(|key| key == anchor);
    let previous_at = collection.iter().position(|key| key == previous);
    let target_at = collection.iter().position(|key| key == target);
    let between = |from: Option<usize>, to: Option<usize>| {
        from.zip(to)
            .map(|(from, to)| if from <= to { from..=to } else { to..=from })
    };
    let mut next = current.clone();
    if let Some(previous_range) = between(anchor_at, previous_at) {
        for index in previous_range {
            next.remove(&collection[index]);
        }
    }
    if let Some(target_range) = between(anchor_at, target_at) {
        for index in target_range {
            let key = &collection[index];
            if selectable.contains(key) {
                next.insert(key.clone());
            }
        }
    }
    next
}

/// Pinned React Aria 3.51.0 `useSelectableCollection` registers Home and End
/// only for the chords each platform's handler admits: none, Shift, Alt, and
/// Alt+Shift on macOS -- no Meta or Control handler exists -- and none,
/// Shift, Control, and Control+Shift on Windows and Linux. The upstream
/// matcher reads exactly the browser's canonical modifier flags -- Alt,
/// Control, Meta, Shift -- so GPUI's `function` flag is ignored here: a
/// browser exposes no Fn state for it to read, so vetoing on the flag would
/// claim a pinned guard that does not exist, and the framework delivers an
/// Fn-bearing press with every matched modifier flag still false. A chord
/// outside the registration is entirely inert: no focus move, no selection,
/// no preventDefault. `macos` is simulated explicitly so every platform's
/// unit tests can prove both maps.
fn home_end_registered(modifiers: gpui::Modifiers, macos: bool) -> bool {
    if macos {
        !modifiers.control && !modifiers.platform
    } else {
        !modifiers.alt && !modifiers.platform
    }
}

/// Pinned `useSelectableCollection` (`isCtrlKeyPressed`): a Shift move
/// extends the range on the collection's navigation keys, while Home and
/// End extend only from Control+Shift on Windows and Linux. macOS registers
/// no Home/End extension at all -- its Shift and Alt+Shift chords move the
/// focus alone -- so the platform is an explicit bool rather than a `cfg!`.
fn shift_home_end_extends(key_name: &str, control: bool, macos: bool) -> bool {
    !matches!(key_name, "home" | "end") || (!macos && control)
}

/// HeroUI TagGroup.
#[derive(IntoElement)]
pub struct TagGroup {
    id: ElementId,
    tags: Vec<Tag>,
    label: Option<SharedString>,
    description: Option<SharedString>,
    selection_mode: SelectionMode,
    selected_keys: HashSet<SharedString>,
    default_selected_keys: HashSet<SharedString>,
    is_controlled: bool,
    disallow_empty_selection: bool,
    escape_key_behavior: EscapeKeyBehavior,
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
            default_selected_keys: HashSet::new(),
            is_controlled: false,
            disallow_empty_selection: false,
            escape_key_behavior: EscapeKeyBehavior::ClearSelection,
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
        self.is_controlled = true;
        self
    }

    /// `defaultSelectedKeys` — seeds the group's own selection state.
    pub fn default_selected_keys(mut self, keys: impl IntoIterator<Item = SharedString>) -> Self {
        self.default_selected_keys = keys.into_iter().collect();
        self
    }

    /// Prevents selection from becoming empty through a tag toggle or Escape.
    pub fn disallow_empty_selection(mut self, v: bool) -> Self {
        self.disallow_empty_selection = v;
        self
    }

    /// `escapeKeyBehavior` — whether unmodified Escape clears selection.
    pub fn escape_key_behavior(mut self, behavior: EscapeKeyBehavior) -> Self {
        self.escape_key_behavior = behavior;
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

    /// Adds a remove button to every tag. The button reports its own key;
    /// Delete or Backspace reports the whole selection when the focused tag is
    /// selected, and otherwise reports only the focused key.
    pub fn on_remove(
        mut self,
        handler: impl Fn(&HashSet<SharedString>, &mut Window, &mut App) + 'static,
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
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
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
        // The Shift-range anchor lives beside the cursor, keyed off the same
        // instance id so two groups never share an anchor.
        let selection_range = window.use_keyed_state(
            ElementId::Name(format!("{:?}-range", self.id).into()),
            cx,
            |_, _| TagSelectionRange::default(),
        );
        let (selected_keys, selection_own) = crate::util::controlled(
            window,
            cx,
            ElementId::Name(format!("{:?}-selected", self.id).into()),
            self.is_controlled.then(|| self.selected_keys.clone()),
            self.default_selected_keys.clone(),
        );
        self.selected_keys = selected_keys;
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
        let enabled_keys: HashSet<SharedString> = enabled
            .iter()
            .map(|index| self.tags[*index].key.clone())
            .collect();
        // The collection order the range is resolved against — every tag's
        // key, including disabled ones: disabled keys keep their collection
        // positions so range span traversal preserves indexes, while the
        // `selectable` filter keeps their insertions out of the range.
        let collection_keys: Vec<SharedString> =
            self.tags.iter().map(|tag| tag.key.clone()).collect();
        let owns_focus = window.is_window_active() && group_focus.is_focused(window);
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
        for (index, slot) in interaction.iter().enumerate() {
            let tag = &self.tags[index];
            if (self.selection_mode == SelectionMode::None
                || self.is_disabled
                || tag.is_disabled
                || self.disabled_keys.contains(&tag.key))
                && *slot.read(cx) != (false, false)
            {
                slot.update(cx, |state, _| *state = (false, false));
            }
        }
        let remove_focus_handles = if self.on_remove.is_some() {
            self.tags
                .iter()
                .map(|tag| {
                    crate::util::tab_stop_handle(
                        ElementId::Name(
                            format!("{:?}-tag-{:?}-remove-focus", self.id, tag.key).into(),
                        ),
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
        let mut root = div().relative().flex().flex_col().gap(px(4.));

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

        let mut list = div().relative().flex().flex_row().flex_wrap().gap(px(6.));

        for (index, tag) in self.tags.iter().enumerate() {
            let disabled =
                self.is_disabled || tag.is_disabled || self.disabled_keys.contains(&tag.key);
            let selected = self.selected_keys.contains(&tag.key);
            let selectable = self.selection_mode != SelectionMode::None;
            let interactive = selectable && !disabled;
            let tag_foreground = if selected {
                colors.accent.soft_foreground(colors.foreground)
            } else {
                match self.variant {
                    TagVariant::Default => colors.default.foreground,
                    TagVariant::Surface => colors.surface.foreground,
                }
            };

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
                .font_weight(gpui::FontWeight::MEDIUM)
                .whitespace_nowrap();

            chip = if selected {
                chip.bg(colors.accent.soft()).text_color(tag_foreground)
            } else {
                match self.variant {
                    TagVariant::Default => chip.bg(colors.default.color).text_color(tag_foreground),
                    TagVariant::Surface => chip
                        .bg(colors.surface.background)
                        .text_color(tag_foreground),
                }
            };

            if disabled {
                chip = chip.opacity(layout.disabled_opacity);
            } else if selectable {
                let hover = if selected {
                    colors.accent.soft_hover()
                } else {
                    match self.variant {
                        TagVariant::Default => colors.default.hover(),
                        TagVariant::Surface => colors.surface.hover(),
                    }
                };
                chip = chip.cursor_pointer().hover(move |s| s.bg(hover));
            }

            if let Some(path) = &tag.icon {
                chip = chip.child(
                    gpui::svg()
                        .size(px(12.))
                        .path(path.clone())
                        .flex_shrink_0()
                        .text_color(tag_foreground),
                );
            }

            chip = match &self.tag_content {
                Some(render) => {
                    let (is_hovered, is_pressed) = if interactive {
                        interaction
                            .get(index)
                            .map(|slot| *slot.read(cx))
                            .unwrap_or_default()
                    } else {
                        (false, false)
                    };
                    let focused = !disabled && owns_focus && cursor_index == Some(index);
                    chip.child(render(
                        tag,
                        crate::util::InteractiveState {
                            is_hovered,
                            is_pressed,
                            is_focused: focused,
                            is_focus_visible: focused && ring_visible,
                            is_selected: selected,
                            is_disabled: disabled,
                            is_pending: false,
                            is_indeterminate: false,
                        },
                    ))
                }
                None => chip.child(tag.label.to_string()),
            };
            if interactive {
                if let Some(slot) = interaction.get(index) {
                    chip = crate::util::track_interaction(chip, slot);
                }
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
                            .size(px(12.))
                            .path(icons::CLOSE)
                            .text_color(tag_foreground),
                    );
                if !disabled {
                    let hover_bg = colors.default.hover();
                    let remove_focus = &remove_focus_handles[index];
                    let focus_for_remove = group_focus.clone();
                    let cursor_for_remove = cursor.clone();
                    close = close
                        .track_focus(remove_focus)
                        .cursor_pointer()
                        .hover(move |s| s.bg(hover_bg))
                        // React Aria's grid-list stops row key handling while
                        // a child button owns the focus, except for Tab.
                        .on_key_down(|event, _, cx| {
                            if event.keystroke.key != "tab" {
                                cx.stop_propagation();
                            }
                        })
                        // The press belongs to the button. Stopping it here
                        // keeps the tag body's mouse-down -- which seats the
                        // group's focus and cursor -- out of a remove press.
                        .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| {
                            cx.stop_propagation();
                        })
                        .on_click(move |_, window, cx| {
                            // The remove button is an action inside a
                            // selectable tag. Its press belongs to the button,
                            // never to the tag behind it.
                            cx.stop_propagation();
                            on_remove(&HashSet::from([key.clone()]), window, cx);
                            // Pinned `useSelectableItem` only isolates the
                            // child's press -- DOM focus goes to the button.
                            // This port seats the owning tag itself because
                            // the report-only Rust model has no persisting
                            // native child and keyboard continuity needs a
                            // stable roving target. Removal only reports;
                            // the selection is not this click's to change.
                            window.focus(&focus_for_remove);
                            cursor_for_remove.update(cx, |v, cx| {
                                *v = index;
                                cx.notify();
                            });
                        });
                    close = crate::util::ring_if_focused(
                        close,
                        remove_focus,
                        true,
                        Vec::new(),
                        window,
                        cx,
                    );
                }
                chip = chip.child(close);
            }

            // React Aria's TagGroup: the arrows move between tags. Delete or
            // Backspace removes the selection when the focused tag belongs to
            // it, and otherwise removes only the focused tag.
            if !disabled {
                let stops = enabled.clone();
                let moved = cursor.clone();
                let remove = self.on_remove.clone();
                let key_for_remove = tag.key.clone();
                let mode = self.selection_mode;
                let disallow_empty = self.disallow_empty_selection;
                let escape_key_behavior = self.escape_key_behavior;
                let selected_now = self.selected_keys.clone();
                let selectable_keys = enabled_keys.clone();
                let collection_for_keys = collection_keys.clone();
                let range_for_keys = selection_range.clone();
                let on_selection_change = self.on_selection_change.clone();
                let selection_own_for_keys = selection_own.clone();
                let range_for_all = selection_range.clone();
                let range_for_escape = selection_range.clone();
                chip = chip.on_key_down(move |event, window, cx| {
                    let key_name = event.keystroke.key.as_str();
                    // Pinned React Aria 3.51 routes TagGroup through
                    // `useSelectableCollection`, including Mod+A select-all.
                    if key_name == "a"
                        && event.keystroke.modifiers.secondary()
                        && !event.keystroke.modifiers.shift
                        && !event.keystroke.modifiers.alt
                        && !event.keystroke.modifiers.function
                        && if cfg!(target_os = "macos") {
                            !event.keystroke.modifiers.control
                        } else {
                            !event.keystroke.modifiers.platform
                        }
                        && mode == SelectionMode::Multiple
                    {
                        let all_selected =
                            selectable_keys.iter().all(|key| selected_now.contains(key));
                        if !all_selected {
                            if let Some(held) = &selection_own_for_keys {
                                held.update(cx, |value, cx| {
                                    *value = selectable_keys.clone();
                                    cx.notify();
                                });
                            }
                            if let Some(change) = &on_selection_change {
                                change(&selectable_keys, window, cx);
                            }
                            // The raw `all`: the next Shift move collapses to
                            // its target instead of extending across
                            // everything. A redundant Mod+A over an already
                            // complete selection is idempotent and keeps the
                            // anchor, like pinned `SelectionManager::selectAll`.
                            range_for_all.update(cx, |range, _| {
                                *range = TagSelectionRange {
                                    is_all: true,
                                    ..TagSelectionRange::default()
                                };
                            });
                        }
                        cx.stop_propagation();
                        return;
                    }
                    // `useSelectableCollection` also clears a nonempty
                    // selection on unmodified Escape by default.
                    if key_name == "escape"
                        && !event.keystroke.modifiers.modified()
                        && escape_key_behavior == EscapeKeyBehavior::ClearSelection
                        && crate::selection::reports_changes(mode)
                        && !disallow_empty
                        && !selected_now.is_empty()
                    {
                        let next = HashSet::new();
                        if let Some(held) = &selection_own_for_keys {
                            held.update(cx, |value, cx| {
                                *value = next.clone();
                                cx.notify();
                            });
                        }
                        if let Some(change) = &on_selection_change {
                            change(&next, window, cx);
                        }
                        range_for_escape.update(cx, |range, _| {
                            *range = TagSelectionRange::default();
                        });
                        cx.stop_propagation();
                        return;
                    }
                    match key_name {
                        "delete" | "backspace" => {
                            if let Some(cb) = &remove {
                                cx.stop_propagation();
                                if selected_now.contains(&key_for_remove) {
                                    cb(&selected_now, window, cx);
                                } else {
                                    cb(&HashSet::from([key_for_remove.clone()]), window, cx);
                                }
                            }
                        }
                        key @ ("left" | "right" | "home" | "end") => {
                            // The pinned registrations install no Home/End
                            // handler for an unregistered chord -- Cmd- or
                            // Ctrl-bearing on macOS, Alt- or platform-bearing
                            // elsewhere -- so the whole event stays inert: no
                            // focus move, no selection, no preventDefault,
                            // and no consumption of the key either.
                            if matches!(key, "home" | "end")
                                && !home_end_registered(
                                    event.keystroke.modifiers,
                                    cfg!(target_os = "macos"),
                                )
                            {
                                return;
                            }
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
                                crate::list_nav::resolve(&stops, Some(index), key, true)
                            else {
                                return;
                            };
                            // Pinned `useSelectableCollection`: Shift extends a
                            // multiple selection from the anchor with no other
                            // chord, so plain Shift navigation is exact. A
                            // wrap-to-self move or a Home/End
                            // already at its target still extends: pinned
                            // `extendSelection` replaces the anchor..target
                            // range even when the cursor does not move, and
                            // the unchanged-selection guard below is what
                            // keeps true no-ops silent.
                            let modifiers = event.keystroke.modifiers;
                            let exact_shift_navigation = if cfg!(target_os = "macos") {
                                !modifiers.control && !modifiers.platform && !modifiers.function
                            } else {
                                !modifiers.alt && !modifiers.platform && !modifiers.function
                            };
                            let extends_selection = modifiers.shift
                                && mode == SelectionMode::Multiple
                                && exact_shift_navigation
                                && shift_home_end_extends(
                                    key_name,
                                    modifiers.control,
                                    cfg!(target_os = "macos"),
                                );
                            if extends_selection {
                                if let Some(target) = collection_for_keys.get(next) {
                                    let range = range_for_keys.read(cx).clone();
                                    let next_selection = extend_selection_range(
                                        &selected_now,
                                        &collection_for_keys,
                                        &selectable_keys,
                                        &range,
                                        target,
                                    );
                                    range_for_keys
                                        .update(cx, |range, _| range.seat(target.clone()));
                                    if next_selection != selected_now {
                                        if let Some(held) = &selection_own_for_keys {
                                            held.update(cx, |value, cx| {
                                                *value = next_selection.clone();
                                                cx.notify();
                                            });
                                        }
                                        if let Some(change) = &on_selection_change {
                                            change(&next_selection, window, cx);
                                        }
                                    }
                                }
                            }
                            // No refocusing: the next render has the tag
                            // at `next` claim the group's handle, so the
                            // focus goes with it.
                            moved.update(cx, |v, cx| {
                                *v = next;
                                cx.notify();
                            });
                        }
                        _ => {}
                    }
                });
                // React Aria seats a collection on pointer-down: pressing a
                // tag's body moves the roving cursor to it and takes the
                // group's focus, so the arrows and Space answer the tag the
                // user pressed with no Tab first. A child that prevented the
                // press -- the remove button -- keeps the body out of it, and
                // preventing the default here is the other half of the seat:
                // gpui's own focus-on-press for `track_focus` elements works
                // the same way, so an ancestor's -- the app root's -- press
                // focus cannot steal the handle back in the same dispatch.
                let focus_for_seat = group_focus.clone();
                let cursor_for_seat = cursor.clone();
                chip = chip.on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                    if window.default_prevented() {
                        return;
                    }
                    window.focus(&focus_for_seat);
                    window.prevent_default();
                    cursor_for_seat.update(cx, |v, cx| {
                        *v = index;
                        cx.notify();
                    });
                });
            }

            if selectable && !disabled {
                let key = tag.key.clone();
                let mode = self.selection_mode;
                let disallow_empty = self.disallow_empty_selection;
                let current = self.selected_keys.clone();
                let on_change = self.on_selection_change.clone();
                let selection_own = selection_own.clone();
                let collection_for_click = collection_keys.clone();
                let selectable_for_click = enabled_keys.clone();
                let range_for_click = selection_range.clone();
                chip = chip.on_click(move |ev, window, cx| {
                    let was_selected = current.contains(&key);
                    // A Shift click extends from the anchor in multiple mode;
                    // an ordinary click toggles and re-anchors instead.
                    let extends_selection = ev.modifiers().shift && mode == SelectionMode::Multiple;
                    let next = if extends_selection {
                        let range = range_for_click.read(cx).clone();
                        extend_selection_range(
                            &current,
                            &collection_for_click,
                            &selectable_for_click,
                            &range,
                            &key,
                        )
                    } else {
                        match mode {
                            SelectionMode::None => current.clone(),
                            SelectionMode::Single => {
                                if current.contains(&key) && !disallow_empty {
                                    HashSet::new()
                                } else {
                                    HashSet::from([key.clone()])
                                }
                            }
                            SelectionMode::Multiple => {
                                let mut set = current.clone();
                                if set.remove(&key) {
                                    if disallow_empty && set.is_empty() {
                                        set.insert(key.clone());
                                    }
                                } else {
                                    set.insert(key.clone());
                                }
                                set
                            }
                        }
                    };
                    // A controlled selection only reports; the owner's prop
                    // stays in charge until it feeds the value back.
                    if next != current {
                        if let Some(held) = &selection_own {
                            held.update(cx, |value, cx| {
                                *value = next.clone();
                                cx.notify();
                            });
                        }
                        if let Some(change) = &on_change {
                            change(&next, window, cx);
                        }
                    }
                    if extends_selection {
                        range_for_click.update(cx, |range, _| range.seat(key.clone()));
                    } else if mode == SelectionMode::Multiple && !was_selected {
                        range_for_click.update(cx, |range, _| {
                            range.anchor = Some(key.clone());
                            range.current = Some(key.clone());
                            range.is_all = false;
                        });
                    } else if mode == SelectionMode::Multiple {
                        range_for_click.update(cx, |range, _| {
                            if range.is_all {
                                *range = TagSelectionRange::default();
                            }
                        });
                    }
                });
            }

            // `.tag:focus-visible` is `status-focused`.
            let chip = crate::util::with_focus_ring(
                chip,
                !disabled && ring_visible && cursor_index == Some(index) && owns_focus,
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
                    .p(px(4.))
                    .text_size(px(12.))
                    .text_color(colors.muted)
                    .child(description.to_string()),
            );
        }

        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Home/End gate takes the platform as an explicit bool, so this
    /// truth table is free of `cfg!` and mechanically proves both maps from
    /// any host: no macOS chord ever extends -- Shift and Alt+Shift move the
    /// focus alone -- while Windows and Linux extend exactly from
    /// Control+Shift.
    #[test]
    fn shift_home_end_extends_only_from_control_outside_macos() {
        for key in ["home", "end"] {
            assert!(
                !shift_home_end_extends(key, true, true),
                "macOS registers no Home/End extension"
            );
            assert!(!shift_home_end_extends(key, false, true));
            assert!(
                shift_home_end_extends(key, true, false),
                "Control+Shift+{key} must extend on Windows and Linux"
            );
            assert!(
                !shift_home_end_extends(key, false, false),
                "plain Shift+{key} must only move the focus"
            );
        }
    }

    /// The horizontal delegate's arrows never consult the Home/End gate:
    /// their forbidden extra chords are rejected earlier, by
    /// `exact_shift_navigation`.
    #[test]
    fn shift_navigation_keys_do_not_consult_the_home_end_gate() {
        for key in ["left", "right"] {
            assert!(shift_home_end_extends(key, false, true));
            assert!(shift_home_end_extends(key, true, false));
        }
    }

    /// The registration gate takes `Modifiers`, so the pinned chord map can
    /// be spelled out: macOS registers none, Shift, Alt, and Alt+Shift and
    /// every Control- or Meta-bearing chord is entirely inert, while
    /// Windows and Linux register none, Shift, Control, and Control+Shift
    /// and reject every Alt- or Meta-bearing chord. The upstream matcher
    /// sees only the browser's Alt/Control/Meta/Shift flags, so GPUI's
    /// `function` flag is ignored: `fn` stays registered on both maps, and
    /// it never rescues a chord the platform itself rejects.
    #[test]
    fn home_end_registration_matches_the_pinned_chord_map() {
        let none = gpui::Modifiers::none();
        let shift = gpui::Modifiers {
            shift: true,
            ..none
        };
        let alt = gpui::Modifiers { alt: true, ..none };
        let alt_shift = gpui::Modifiers { shift: true, ..alt };
        let function = gpui::Modifiers {
            function: true,
            ..none
        };
        let function_alt = gpui::Modifiers {
            alt: true,
            ..function
        };
        for modifiers in [none, shift, alt, alt_shift, function, function_alt] {
            assert!(
                home_end_registered(modifiers, true),
                "macOS must register {modifiers:?}"
            );
        }
        let control = gpui::Modifiers {
            control: true,
            ..none
        };
        let control_shift = gpui::Modifiers {
            shift: true,
            ..control
        };
        let platform = gpui::Modifiers {
            platform: true,
            ..none
        };
        let platform_shift = gpui::Modifiers {
            shift: true,
            ..platform
        };
        for modifiers in [control, control_shift, platform, platform_shift] {
            assert!(
                !home_end_registered(modifiers, true),
                "macOS must not register {modifiers:?}"
            );
        }
        for modifiers in [none, shift, control, control_shift, function] {
            assert!(
                home_end_registered(modifiers, false),
                "Windows and Linux must register {modifiers:?}"
            );
        }
        for modifiers in [alt, alt_shift, function_alt, platform, platform_shift] {
            assert!(
                !home_end_registered(modifiers, false),
                "Windows and Linux must not register {modifiers:?}"
            );
        }
    }

    /// The keystroke spellings real events hand the gate: `ctrl` parses to
    /// the Control field the Windows/Linux registration admits and macOS
    /// vetoes, `cmd` to the platform field macOS vetoes, `alt-shift` to
    /// the chord that stays registered (focus-only) on macOS alone, and
    /// `fn` to the flag the browser matcher never sees, so it registers
    /// exactly like the bare key on both maps.
    #[test]
    fn keystroke_spellings_reach_the_registration_gate() {
        let ctrl_shift_home = gpui::Keystroke::parse("ctrl-shift-home").unwrap();
        assert!(home_end_registered(ctrl_shift_home.modifiers, false));
        assert!(!home_end_registered(ctrl_shift_home.modifiers, true));
        let cmd_shift_home = gpui::Keystroke::parse("cmd-shift-home").unwrap();
        assert!(!home_end_registered(cmd_shift_home.modifiers, true));
        let alt_shift_end = gpui::Keystroke::parse("alt-shift-end").unwrap();
        assert!(home_end_registered(alt_shift_end.modifiers, true));
        assert!(!home_end_registered(alt_shift_end.modifiers, false));
        let fn_home = gpui::Keystroke::parse("fn-home").unwrap();
        assert!(fn_home.modifiers.function);
        assert!(home_end_registered(fn_home.modifiers, true));
        assert!(home_end_registered(fn_home.modifiers, false));
    }
}
