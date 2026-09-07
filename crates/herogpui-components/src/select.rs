//! Select — port of `@heroui/select` with single and multiple selection.
//!
//! Pinned v3.2.4 / React Aria Components 1.20.0 keep a stable `Key` separate
//! from each item's `textValue`: `value` / `defaultValue` / `selectedKeys` /
//! `disabledKeys`, the selection callbacks and the form value address items by
//! key, while typeahead and the visible text use the label. Items are
//! therefore [`crate::PickerItem`]s, exactly as in
//! [`crate::Autocomplete`] and [`crate::ComboBox`]: a label cannot serve as
//! that key — two items may share one, and then they alias each other's
//! selection, disabled state and row identity.
//!
//! The selection and the keyboard cursor are held as keys, so both follow an
//! item across a re-render that reorders the collection. Reports and the form
//! value walk the collection, so a selection reads in row order however it
//! was picked — the walk `Select.Value` has always followed.

use std::{cell::RefCell, collections::HashSet, rc::Rc};

use gpui::{
    prelude::*, px, App, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window,
};
use herogpui_core::{element_id, Color, FieldVariant, Placement, SelectionMode};
use herogpui_theme::ActiveTheme;

use crate::{
    a11y::{self, A11y as _},
    icons,
    picker_item::PickerItem,
    selection::toggle_key,
    util,
};

type OnSelectionChange =
    std::sync::Arc<dyn Fn(&Option<SharedString>, &mut Window, &mut App) + 'static>;

// This port approximates pinned RAC's en-US `Intl.ListFormat` conjunction.
fn format_selected_names(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [name] => name.clone(),
        [first, second] => format!("{first} and {second}"),
        _ => format!(
            "{}, and {}",
            names[..names.len() - 1].join(", "),
            names.last().expect("the multiple-item branch is non-empty")
        ),
    }
}

/// A selection re-ordered to collection order: every chosen key that still
/// resolves to an item, in row order. Select reports its selection through
/// this walk — the trigger text, the plural callback, the form value and
/// `Select.Value` all read the collection the way this does.
fn in_collection_order(selected: &[SharedString], keys: &[SharedString]) -> Vec<SharedString> {
    keys.iter()
        .filter(|key| selected.contains(*key))
        .cloned()
        .collect()
}

/// The chosen keys that resolve to collection items, in row order — the
/// single key and the multiple set are disjoint channels, so filtering on
/// both is safe. Keys the collection no longer holds resolve to nothing, the
/// way an out-of-range index always did.
fn resolved_keys(
    items: &[PickerItem],
    single: &Option<SharedString>,
    multiple: &[SharedString],
) -> Vec<SharedString> {
    items
        .iter()
        .map(|item| item.key())
        .filter(|key| single.as_ref() == Some(*key) || multiple.contains(*key))
        .cloned()
        .collect()
}

/// Pinned React Stately 3.49.0 `useMultipleSelectionState`'s anchor record,
/// on the option keys a Select's `stops` walk: where a Shift extension
/// reaches from, how far the last one went, and whether the selection is the
/// raw `all` a `selectAll` produced. It lives beside the cursor in keyed
/// state, so it survives closing and reopening the popover the way the
/// pinned hook survives a listbox remount.
#[derive(Clone, Debug, Default)]
struct SelectSelectionRange {
    anchor: Option<SharedString>,
    current: Option<SharedString>,
    is_all: bool,
}

/// The selection after extending to `target` from `range`'s anchor.
///
/// The old anchor..current range is *replaced* by anchor..target, so extending
/// backwards shrinks again; a raw `all` collapses to the new key; a first
/// extension without an anchor selects from the target itself, which is what
/// the pinned SelectionManager does when nothing anchors it yet. Only
/// `selectable` keys enter the range, so disabled options are skipped, and
/// the result walks the collection so it reports in row order.
fn extend_selection_range(
    current: &[SharedString],
    collection: &[SharedString],
    selectable: &[SharedString],
    range: &SelectSelectionRange,
    target: &SharedString,
) -> Vec<SharedString> {
    if range.is_all {
        return vec![target.clone()];
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
    let mut next: Vec<SharedString> = current.to_vec();
    if let Some(previous_range) = between(anchor_at, previous_at) {
        for at in previous_range {
            let stale = &collection[at];
            if let Some(held) = next.iter().position(|key| key == stale) {
                next.remove(held);
            }
        }
    }
    if let Some(target_range) = between(anchor_at, target_at) {
        for at in target_range {
            let key = &collection[at];
            if selectable.contains(key) && !next.contains(key) {
                next.push(key.clone());
            }
        }
    }
    in_collection_order(&next, collection)
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
/// outside the registration is entirely inert: no cursor move, no selection,
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
/// cursor alone -- so the platform is an explicit bool rather than a `cfg!`.
fn shift_home_end_extends(key_name: &str, control: bool, macos: bool) -> bool {
    !matches!(key_name, "home" | "end") || (!macos && control)
}

/// HeroUI Select (controlled).
#[derive(IntoElement)]
pub struct Select {
    /// `name` — the name this control submits under; read back by
    /// [`Self::form_field`].
    name: Option<SharedString>,
    id: gpui::ElementId,
    items: Vec<PickerItem>,
    selected: Option<SharedString>,
    /// Whether `value` was supplied. `Option<SharedString>` cannot distinguish
    /// "controlled, nothing selected" from "uncontrolled" on its own.
    is_controlled: bool,
    default_value: Option<SharedString>,
    /// Backs `selectionMode="multiple"`; `selected` backs `single`.
    selected_keys: Vec<SharedString>,
    is_multiple_controlled: bool,
    default_selected_keys: Vec<SharedString>,
    selection_mode: SelectionMode,
    /// `isOpen` — `None` leaves the component holding the flag, seeded from
    /// `defaultOpen`.
    is_open: Option<bool>,
    default_open: bool,
    placement: Placement,
    label: Option<SharedString>,
    placeholder: SharedString,
    description: Option<SharedString>,
    variant: FieldVariant,
    is_disabled: bool,
    is_invalid: bool,
    /// `shouldFocusWrap` — whether the arrow keys wrap at the ends of the list.
    should_focus_wrap: bool,
    /// `ListLayout`'s `rowHeight`, which virtualizes the popover list.
    row_height: Option<gpui::Pixels>,
    /// `ListBox.Section` — the heading that precedes an option, by item key.
    sections: Vec<(SharedString, SharedString)>,
    /// `ListBox.ItemIndicator` — draws the tick. The closure is handed whether
    /// the row is selected.
    indicator: Option<Box<dyn Fn(bool) -> gpui::AnyElement + 'static>>,
    /// `Select.Value` — draws the trigger's value. The closure is handed the
    /// selected key, or `None` while the placeholder shows.
    value_content: Option<Box<dyn Fn(util::SelectionValue<'_>) -> gpui::AnyElement + 'static>>,
    is_required: bool,
    disabled_keys: HashSet<SharedString>,
    full_width: bool,
    on_open_change: Option<std::sync::Arc<dyn Fn(&bool, &mut Window, &mut App) + 'static>>,
    on_selection_change: Option<OnSelectionChange>,
    on_selection_change_all:
        Option<std::sync::Arc<dyn Fn(&[SharedString], &mut Window, &mut App) + 'static>>,
    /// Mirrors the current selection, validity, successful state, focus and
    /// reset behavior for a live [`crate::form::FormField`].
    form_state: Rc<RefCell<crate::form::LiveFormFieldState>>,
    /// The `sx` slot, refined over the root style at the end of render.
    sx: Option<Box<gpui::StyleRefinement>>,
}

impl Select {
    /// `onChange` — the v3 name for [`Select::on_selection_change`].
    pub fn on_change(
        self,
        handler: impl Fn(&Option<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change(handler)
    }

    /// `disabledKeys` — keys of the items that cannot be chosen. Disabled
    /// state is per key, so one of two same-label items can be disabled
    /// alone.
    pub fn disabled_keys(mut self, keys: impl IntoIterator<Item = SharedString>) -> Self {
        self.disabled_keys = keys.into_iter().collect();
        self
    }

    /// `shouldFocusWrap` — whether the arrow keys wrap at the ends of the list.
    /// `ListLayout`'s `rowHeight` -- and what virtualizes the popover list.
    ///
    /// v3 wraps the list in `<Virtualizer layout={ListLayout}>` inside
    /// `Select.Popover`; gpui's `uniform_list` builds only the rows in view, and
    /// it can do that because every row is this tall.
    pub fn row_height(mut self, h: impl Into<gpui::Pixels>) -> Self {
        self.row_height = Some(h.into());
        self
    }

    pub fn should_focus_wrap(mut self, v: bool) -> Self {
        self.should_focus_wrap = v;
        self
    }

    /// `ListBox.Section` — a heading rendered above the item with this key.
    pub fn section_before(
        mut self,
        item: impl Into<SharedString>,
        label: impl Into<SharedString>,
    ) -> Self {
        self.sections.push((item.into(), label.into()));
        self
    }

    /// `ListBox.ItemIndicator` — draw the selected tick yourself.
    pub fn indicator(mut self, render: impl Fn(bool) -> gpui::AnyElement + 'static) -> Self {
        self.indicator = Some(Box::new(render));
        self
    }

    /// `Select.Value` — draw the trigger's value yourself.
    ///
    /// The closure is handed the render props v3 passes into
    /// `<Select.Value>{({defaultChildren, isPlaceholder, selectedItems}) => …}`,
    /// so a `multiple` select can draw all of them and a caller can fall back to
    /// what the trigger would have drawn.
    pub fn value_content(
        mut self,
        render: impl Fn(util::SelectionValue<'_>) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.value_content = Some(Box::new(render));
        self
    }

    pub fn is_invalid(mut self, v: bool) -> Self {
        self.is_invalid = v;
        self
    }

    pub fn is_required(mut self, v: bool) -> Self {
        self.is_required = v;
        self
    }

    /// `fullWidth` — stretch to the container width.
    pub fn full_width(mut self, v: bool) -> Self {
        self.full_width = v;
        self
    }

    /// The one slot for caller-owned low-level styling: GPUI's styling methods
    /// (`bg`, `text_color`, `w`, `h`, `p`, `rounded`, `border_color`, …)
    /// applied to the select's root element after every value the variant and
    /// the active theme chose, so they win. The trigger paints its own chrome,
    /// so this reaches the box that chrome sits in, not the chrome.
    pub fn sx(mut self, style: impl FnOnce(gpui::Div) -> gpui::Div) -> Self {
        self.sx = Some(util::capture_sx(style));
        self
    }

    pub fn new(id: impl Into<gpui::ElementId>, items: Vec<PickerItem>) -> Self {
        Self {
            name: None,
            id: id.into(),
            items,
            selected: None,
            is_controlled: false,
            default_value: None,
            selected_keys: Vec::new(),
            is_multiple_controlled: false,
            default_selected_keys: Vec::new(),
            selection_mode: SelectionMode::Single,
            is_open: None,
            default_open: false,
            placement: Placement::BottomStart,
            label: None,
            placeholder: "Select an item".into(),
            description: None,
            variant: FieldVariant::Primary,
            is_disabled: false,
            is_invalid: false,
            should_focus_wrap: false,
            row_height: None,
            sections: Vec::new(),
            indicator: None,
            value_content: None,
            is_required: false,
            disabled_keys: HashSet::new(),
            full_width: false,
            on_open_change: None,
            on_selection_change: None,
            on_selection_change_all: None,
            form_state: live_form_state(),
            sx: None,
        }
    }

    /// `name` — the name this control submits under.
    pub fn name(mut self, name: impl Into<SharedString>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// The `Form` field this control submits, when it has a `name`.
    ///
    /// v3 discovers a field through the DOM; gpui gives a child no way to reach
    /// its ancestor, so the control hands the pair over instead. Borrows, so the
    /// control is still yours to place:
    ///
    /// ```
    /// # use gpui::{prelude::*, Window};
    /// # use herogpui_components::{Form, PickerItem, Select};
    /// # struct Demo;
    /// # impl Render for Demo {
    /// #     fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    /// #         let form = Form::new();
    /// #         let control =
    /// #             Select::new("city", vec![PickerItem::new("kyiv", "Kyiv")]).name("city");
    /// let field = control.form_field();
    /// form.field(field.unwrap()).child(control)
    /// #     }
    /// # }
    /// # let mut tcx = gpui::TestAppContext::single();
    /// # tcx.update(herogpui_theme::ThemeProvider::init);
    /// # let _ = tcx.add_window_view(|_, _| Demo);
    /// ```
    pub fn form_field(&self) -> Option<crate::form::FormField> {
        let name = self.name.clone()?;
        let selected = if self.is_controlled {
            self.selected.clone()
        } else {
            self.default_value.clone()
        };
        let selected_keys = if self.is_multiple_controlled {
            self.selected_keys.clone()
        } else {
            self.default_selected_keys.clone()
        };
        sync_select_form(
            &self.form_state,
            select_form_value(&self.items, &selected, &selected_keys),
            self.is_invalid,
            !self.is_disabled,
        );
        Some(
            crate::form::FormField::live(name, self.form_state.clone())
                .is_required(self.is_required),
        )
    }

    /// `selectionMode`
    pub fn selection_mode(mut self, mode: SelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }

    /// The chosen item keys under `selectionMode="multiple"` — `selectedKeys`,
    /// the `ListBox` spelling of [`Select::value`]'s controlled set.
    pub fn selected_keys(mut self, keys: impl IntoIterator<Item = SharedString>) -> Self {
        self.selected_keys = keys.into_iter().collect();
        self.is_multiple_controlled = true;
        self
    }

    /// `defaultSelectedKeys` under `selectionMode="multiple"` — the
    /// uncontrolled initial selection.
    pub fn default_selected_keys(mut self, keys: impl IntoIterator<Item = SharedString>) -> Self {
        self.default_selected_keys = keys.into_iter().collect();
        self
    }

    /// Reports the whole selection, for `selectionMode="multiple"`.
    pub fn on_selection_change_all(
        mut self,
        handler: impl Fn(&[SharedString], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change_all = Some(std::sync::Arc::new(handler));
        self
    }

    /// `value` — the selected item, by key. Supplying it makes the select
    /// controlled, even with `None`.
    pub fn value(mut self, key: Option<SharedString>) -> Self {
        self.selected = key;
        self.is_controlled = true;
        self
    }

    /// `defaultValue` — the uncontrolled initial selection, by key.
    ///
    /// Only consulted when `value` is not supplied; the select then owns the
    /// selection and choosing an option moves it.
    pub fn default_value(mut self, key: Option<SharedString>) -> Self {
        self.default_value = key;
        self
    }

    /// `placement` on the popover.
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    pub fn is_open(mut self, v: bool) -> Self {
        self.is_open = Some(v);
        self
    }
    /// `defaultOpen` — the uncontrolled initial state.
    ///
    /// Only consulted when `is_open` is not supplied; the component then owns
    /// the flag and its trigger toggles it.
    pub fn default_open(mut self, v: bool) -> Self {
        self.default_open = v;
        self
    }

    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = Some(l.into());
        self
    }

    pub fn placeholder(mut self, p: impl Into<SharedString>) -> Self {
        self.placeholder = p.into();
        self
    }

    pub fn description(mut self, d: impl Into<SharedString>) -> Self {
        self.description = Some(d.into());
        self
    }

    pub fn variant(mut self, v: FieldVariant) -> Self {
        self.variant = v;
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    pub fn on_open_change(mut self, f: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_open_change = Some(std::sync::Arc::new(f));
        self
    }

    pub fn on_selection_change(
        mut self,
        f: impl Fn(&Option<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change = Some(std::sync::Arc::new(f));
        self
    }
}

impl Select {
    fn value_text_single(&self, selected: &Option<SharedString>) -> SharedString {
        selected
            .as_ref()
            .and_then(|key| self.items.iter().find(|item| item.key() == key))
            .map_or_else(|| self.placeholder.clone(), |item| item.label().clone())
    }

    fn value_text_multiple(&self, selected_keys: &[SharedString]) -> SharedString {
        let names: Vec<String> = self
            .items
            .iter()
            .filter(|item| selected_keys.contains(item.key()))
            .map(|item| item.label().to_string())
            .collect();
        if names.is_empty() {
            self.placeholder.clone()
        } else {
            SharedString::from(format_selected_names(&names))
        }
    }
}

impl RenderOnce for Select {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        let (is_open, open_own) = util::controlled(
            window,
            cx,
            element_id::scoped(&self.id, "open"),
            self.is_open,
            self.default_open,
        );
        let (overlay_phase, dismissal_token) = util::overlay_scope(
            window,
            cx,
            element_id::scoped(&self.id, "overlay"),
            is_open,
            true,
        );
        let overlay_active = overlay_phase != util::OverlayPhase::Closed;
        let (selected, value_own) = util::controlled(
            window,
            cx,
            element_id::scoped(&self.id, "value"),
            self.is_controlled.then_some(self.selected.clone()),
            self.default_value.clone(),
        );
        let multiple = self.selection_mode == SelectionMode::Multiple;
        let (selected_keys, indices_own) = util::controlled(
            window,
            cx,
            element_id::scoped(&self.id, "values"),
            self.is_multiple_controlled
                .then(|| crate::selection::normalize_selection(self.selected_keys.clone(), true)),
            crate::selection::normalize_selection(self.default_selected_keys.clone(), true),
        );
        let form_default_keys = if multiple {
            let reset_keys = if self.is_multiple_controlled {
                self.selected_keys.clone()
            } else {
                self.default_selected_keys.clone()
            };
            let slot = window.use_keyed_state(
                element_id::scoped(&self.id, "form-default"),
                cx,
                move |_, _| reset_keys,
            );
            slot.read(cx).clone()
        } else {
            Vec::new()
        };
        sync_select_form(
            &self.form_state,
            select_form_value(&self.items, &selected, &selected_keys),
            self.is_invalid,
            !self.is_disabled,
        );
        let reset_own = value_own.clone();
        let reset_keys_own = indices_own.clone();
        let reset_state = self.form_state.clone();
        let reset_change = self
            .is_controlled
            .then(|| self.on_selection_change.clone())
            .flatten();
        let reset_change_all = self
            .is_multiple_controlled
            .then(|| self.on_selection_change_all.clone())
            .flatten();
        let reset_key = self.default_value.clone();
        let reset_items = self.items.clone();
        let reset_mode = self.selection_mode;
        self.form_state.borrow_mut().restore = (reset_own.is_some()
            || (multiple && reset_keys_own.is_some())
            || reset_change.is_some()
            || (multiple && reset_change_all.is_some()))
        .then(|| {
            util::shared(move |window: &mut Window, cx: &mut App| {
                if reset_mode == SelectionMode::Multiple {
                    reset_state.borrow_mut().value =
                        select_form_value(&reset_items, &None, &form_default_keys);
                    if let Some(held) = &reset_keys_own {
                        held.update(cx, |selected, cx| {
                            *selected = form_default_keys.clone();
                            cx.notify();
                        });
                    }
                    if let Some(on_change) = &reset_change_all {
                        on_change(&form_default_keys, window, cx);
                    }
                } else {
                    reset_state.borrow_mut().value =
                        select_form_value(&reset_items, &reset_key, &[]);
                    if let Some(held) = &reset_own {
                        held.update(cx, |selected, cx| {
                            *selected = reset_key.clone();
                            cx.notify();
                        });
                    }
                    if let Some(on_change) = &reset_change {
                        on_change(&reset_key, window, cx);
                    }
                }
            }) as std::sync::Arc<dyn Fn(&mut Window, &mut App)>
        });

        // The collection's keys in row order — the channel the selection,
        // the cursor and every report address.
        let keys: Vec<SharedString> = self.items.iter().map(|item| item.key().clone()).collect();

        // The trigger is what holds focus, so the open list can be walked with
        // the arrows the way v3's is.
        let focus_handle =
            window.use_keyed_state(element_id::scoped(&self.id, "focus"), cx, |_, cx| {
                cx.focus_handle().tab_stop(true)
            });
        let focus_handle = focus_handle.read(cx).clone();
        self.form_state.borrow_mut().focus = Some(focus_handle.clone());
        // Which row the keyboard is on, held as the item's *key* so the cursor
        // stays on the same item when the caller reorders the collection.
        let cursor = window.use_keyed_state(element_id::scoped(&self.id, "cursor"), cx, |_, _| {
            None::<SharedString>
        });
        let cursor_key = cursor.read(cx).clone();
        let cursor_at = cursor_key
            .as_ref()
            .and_then(|key| keys.iter().position(|k| k == key));
        let keyboard_press_open = window.use_keyed_state(
            element_id::scoped(&self.id, "keyboard-press"),
            cx,
            |_, _| None::<bool>,
        );
        // The Shift-range anchor lives beside the cursor, keyed off the same
        // instance id, so two selects never share an anchor and a closed
        // popover leaves its anchor standing for the reopen.
        let selection_range =
            window.use_keyed_state(element_id::scoped(&self.id, "range"), cx, |_, _| {
                SelectSelectionRange::default()
            });
        // v3's list is `overflow-y-auto`, and React Aria keeps the focused
        // option in view. Both need a handle: the virtual list has its own kind,
        // and a plain scrolling div has the other. `use_keyed_state` takes `cx`
        // mutably, so they precede the theme.
        let list_scroll =
            window.use_keyed_state(element_id::scoped(&self.id, "list-scroll"), cx, |_, _| {
                gpui::UniformListScrollHandle::new()
            });
        let panel_scroll =
            window.use_keyed_state(element_id::scoped(&self.id, "panel-scroll"), cx, |_, _| {
                gpui::ScrollHandle::new()
            });
        let list_scroll_now = list_scroll.read(cx).clone();
        let panel_scroll_now = panel_scroll.read(cx).clone();
        // The letters typed so far, which a search resetting every frame could
        // not accumulate.
        let typeahead =
            window.use_keyed_state(element_id::scoped(&self.id, "typed"), cx, |_, _| {
                crate::list_nav::Typeahead::default()
            });

        // Pinned `usePopover` closes when focus leaves the trigger-plus-list
        // scope. Blur deliberately leaves focus on its destination.
        let blur_base = element_id::scoped(&self.id, "select");
        let blur_close_own = open_own.clone();
        let blur_open_change = self.on_open_change.clone();
        let blur_scope = util::close_on_blur(window, cx, &blur_base, is_open, move |window, cx| {
            if let Some(held) = &blur_close_own {
                held.update(cx, |v, cx| {
                    *v = false;
                    cx.notify();
                });
            }
            if let Some(cb) = &blur_open_change {
                cb(&false, window, cx);
            }
        });

        let sem = cx.role(Color::Accent);
        let colors = cx.colors();
        let layout = cx.layout();

        // `.select__trigger` is `min-h-9 ... text-sm`.
        let (h, text) = (util::FIELD_HEIGHT, util::FIELD_TEXT);

        let trigger_id = element_id::scoped(&self.id, "trigger");
        let trigger_selector = format!("select-trigger-{}", id_debug(&self.id));
        // Whether the pointer went down on the trigger. The panel's
        // outside-press dismissal treats the trigger as outside its own bounds,
        // so a press on the trigger of an *open* list would dismiss it on the
        // mouse-down *and* toggle it through the trigger's own click on the
        // mouse-up -- one press, two contradictory reports, and the list ended
        // up open. The trigger's capture-phase handler runs before the panel's
        // `on_mouse_down_out` in the same dispatch, so the dismissal can see it
        // and leave the close to the trigger's click.
        let trigger_pressed = Rc::new(std::cell::Cell::new(false));
        let mut field = gpui::div()
            .id(trigger_id)
            .debug_selector(move || trigger_selector)
            .flex()
            .items_center()
            .justify_between()
            .gap(px(8.))
            .min_h(h)
            .px(px(12.))
            .text_size(text)
            .line_height(px(20.))
            .cursor_pointer();

        let _border_color = if is_open { sem.color } else { colors.separator };
        // `.select__trigger:focus-visible` is `status-focused` -- the offset
        // ring, not a field's flush one, which is why the chrome is not told
        // about the focus here.
        field = util::apply_field_chrome(field, self.variant, self.is_invalid, false, cx);
        if !self.is_disabled {
            field = util::ring_if_focused(field, &focus_handle, true, Vec::new(), window, cx);
        }

        if self.is_invalid || (focus_handle.is_focused(window) && util::focus_visible(cx)) {
            field = field.bg(match self.variant {
                FieldVariant::Primary => colors.field.focus(),
                FieldVariant::Secondary => colors.default.color,
            });
        }

        if !self.is_disabled {
            let hover_bg = match self.variant {
                FieldVariant::Primary => colors.field.hover(),
                // `.select--secondary` hovers `--select-trigger-bg-hover: var(--default-hover)`.
                FieldVariant::Secondary => colors.default.hover(),
            };
            field = field.hover(move |s| s.bg(hover_bg));
        } else {
            field = field.opacity(layout.disabled_opacity);
        }

        if self.full_width {
            field = field.w_full();
        }

        // Down or Enter on a closed Select opens it, and the arrows then walk
        // the options -- the same keys React Aria binds.
        if !self.is_disabled {
            let stops: Vec<usize> = (0..self.items.len())
                .filter(|i| !self.disabled_keys.contains(&keys[*i]))
                .collect();
            // The full key list the range is resolved against -- disabled
            // keys keep their positions so range spans stay indexable, while
            // `stops` keeps their insertions out of the range.
            let collection: Vec<SharedString> = keys.clone();
            let selectable: Vec<SharedString> = stops.iter().map(|i| keys[*i].clone()).collect();
            let held = cursor.clone();
            let wrap = self.should_focus_wrap;
            // Every option's text, so a typed letter can find one.
            let labels: Vec<String> = self
                .items
                .iter()
                .map(|item| item.label().to_string())
                .collect();
            let typed = typeahead;
            let open_own_keys = open_own.clone();
            let value_own_keys = value_own.clone();
            let indices_own_keys = indices_own.clone();
            let selected_held = selected.clone();
            let selected_keys_held = selected_keys.clone();
            let form_state_keys = self.form_state.clone();
            let on_open_change = self.on_open_change.clone();
            let on_select = self.on_selection_change.clone();
            let on_select_all = self.on_selection_change_all.clone();
            let range_keys = selection_range.clone();
            let was_open = is_open;
            let virtual_rows = self.row_height.is_some();
            let key_list_scroll = list_scroll_now.clone();
            let key_panel_scroll = panel_scroll_now.clone();
            let fh = focus_handle.clone();
            let press_open = keyboard_press_open.clone();
            let row_keys = keys.clone();
            field = field
                .track_focus(&focus_handle)
                .key_context("Select")
                .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                    window.focus(&fh, cx);
                })
                .on_key_down(move |event, window, cx| {
                    let key = event.keystroke.key.as_str();
                    if matches!(key, "enter" | "space") {
                        // The browser's default newline can synthesize another
                        // Enter through beforeinput, including while held.
                        cx.stop_propagation();
                        if event.is_held {
                            return;
                        }
                        press_open.update(cx, |value, _| *value = Some(was_open));
                    }
                    if !was_open {
                        // Closed: Down and Up open the list. Enter and Space are
                        // *not* handled here -- the trigger has a click listener
                        // and gpui fires those on Enter and Space for a focused
                        // element, so handling them again would open and close
                        // the list in one keystroke.
                        if matches!(key, "down" | "up") {
                            if let Some(held) = &open_own_keys {
                                held.update(cx, |v, cx| {
                                    *v = true;
                                    cx.notify();
                                });
                            }
                            if let Some(cb) = &on_open_change {
                                cb(&true, window, cx);
                            }
                            return;
                        }
                        // A closed select still answers letters in single
                        // mode: React Aria picks the matching option where it
                        // stands rather than opening the list. A multiple
                        // select answers no typeahead on the closed trigger --
                        // the closed pick reports through the single-key
                        // callback, which a set-valued selection has no use
                        // for. Open, the RAC ListBox keeps its type-select and
                        // moves the cursor without selecting, exactly as in
                        // single mode.
                        if multiple {
                            return;
                        }
                        if !crate::list_nav::is_typeahead_key(key) {
                            return;
                        }
                        let now = web_time::Instant::now();
                        let (query, repeat) = typed.update(cx, |t, _| {
                            let query = t.push(key, now);
                            (query, t.is_repeat())
                        });
                        let selected_at = selected_held
                            .as_ref()
                            .and_then(|k| row_keys.iter().position(|key| key == k));
                        let Some(found) = crate::list_nav::typeahead(
                            &labels,
                            &stops,
                            selected_at,
                            &query,
                            repeat,
                        ) else {
                            return;
                        };
                        let found = row_keys[found].clone();
                        if let Some(held) = &value_own_keys {
                            form_state_keys.borrow_mut().value =
                                crate::form::FormValue::Keys(vec![found.clone()]);
                            held.update(cx, |v, cx| {
                                *v = Some(found.clone());
                                cx.notify();
                            });
                        }
                        if let Some(cb) = &on_select {
                            cb(&Some(found), window, cx);
                        }
                        return;
                    }
                    let from = held
                        .read(cx)
                        .as_ref()
                        .and_then(|k| row_keys.iter().position(|key| key == k));
                    let modifiers = event.keystroke.modifiers;
                    // Pinned React Aria 3.51.0 `useSelectableCollection`
                    // answers `Mod+A` with `selectAll` -- multiple mode only,
                    // and only while the list is open. Pinned SelectState
                    // drops the symbolic `all`: the uncontrolled set becomes
                    // every enabled key while `on_selection_change_all` stays
                    // silent, a repeat over a complete selection is not a
                    // toggle, and a controlled owner's state is not this
                    // keystroke's to mutate.
                    if key == "a"
                        && modifiers.secondary()
                        && !modifiers.shift
                        && !modifiers.alt
                        && if cfg!(target_os = "macos") {
                            !modifiers.control
                        } else {
                            !modifiers.platform
                        }
                        && multiple
                    {
                        let all: Vec<SharedString> =
                            stops.iter().map(|i| row_keys[*i].clone()).collect();
                        let complete = all.iter().all(|key| selected_keys_held.contains(key));
                        if !complete {
                            if let Some(held) = &indices_own_keys {
                                form_state_keys.borrow_mut().value =
                                    crate::form::FormValue::Keys(all.clone());
                                held.update(cx, |selected, cx| {
                                    *selected = all.clone();
                                    cx.notify();
                                });
                                range_keys.update(cx, |range, _| {
                                    *range = SelectSelectionRange {
                                        is_all: true,
                                        ..SelectSelectionRange::default()
                                    };
                                });
                            }
                        }
                        cx.stop_propagation();
                        return;
                    }
                    // Pinned React Aria 3.51.0 binds PageUp/PageDown only
                    // while the collection has a focused key: mouse-opening a
                    // selection-less Select leaves the cursor null, and the
                    // page keys must stay inert until keyboard navigation
                    // establishes one. With a cursor the list is
                    // non-scrollable -- HeroUI v3.2.4 puts the overflow
                    // scrolling on the Popover while the ListBox element is
                    // `overflow-clip` -- so a page takes the enabled end:
                    // `stops` already omits disabled rows, whatever the
                    // list's length, row height, or scroll state. A closed
                    // trigger answers no page key at all, so the closed
                    // branch above never sees them.
                    let page_move = match key {
                        "pagedown" if from.is_some() => stops.last().copied(),
                        "pageup" if from.is_some() => stops.first().copied(),
                        _ => None,
                    }
                    .filter(|next| Some(*next) != from);
                    match page_move.map_or_else(
                        || crate::list_nav::resolve(&stops, from, key, wrap),
                        crate::list_nav::Move::To,
                    ) {
                        crate::list_nav::Move::To(at) => {
                            // The pinned registrations install no Home/End
                            // handler for an unregistered chord -- Cmd- or
                            // Ctrl-bearing on macOS, Alt- or platform-bearing
                            // elsewhere -- so in either mode the whole event
                            // stays inert: no cursor move, no selection, no
                            // preventDefault.
                            if matches!(key, "home" | "end")
                                && !home_end_registered(modifiers, cfg!(target_os = "macos"))
                            {
                                return;
                            }
                            // A Shift extension reaches from the cursor, and a
                            // null cursor is nothing to reach from: pinned
                            // `useSelectableCollection` extends only from a
                            // focused key, so the registered Shift+Home/End is
                            // wholly inert rather than seating a fresh cursor.
                            if matches!(key, "home" | "end") && modifiers.shift && from.is_none() {
                                return;
                            }
                            let next = row_keys[at].clone();
                            held.update(cx, |v, cx| {
                                *v = Some(next.clone());
                                cx.notify();
                            });
                            // React Aria keeps the focused option in view; the
                            // highlight walking off the bottom of the list looks
                            // like the arrows have stopped working.
                            if virtual_rows {
                                key_list_scroll.scroll_to_item(at, gpui::ScrollStrategy::Center);
                            } else {
                                key_panel_scroll.scroll_to_item(at);
                            }
                            // Pinned `useSelectableCollection`: Shift extends a
                            // multiple selection over exactly the chords the
                            // platform's registration admits, so the extension
                            // gate reuses the Home/End registration map -- the
                            // pinned matcher reads the browser's canonical
                            // modifier flags only, and GPUI's `function` flag
                            // never reaches it. The pinned arrow delegates
                            // return null at an enabled boundary, so a
                            // Shift+Arrow that held ran no extension at all
                            // and must not report; Home and End resolve their
                            // end key again, so their repeated registered
                            // extension does report; and the page keys only
                            // reach here off the unchanged filter above.
                            let exact_shift_navigation =
                                home_end_registered(modifiers, cfg!(target_os = "macos"));
                            let extends_selection = multiple
                                && modifiers.shift
                                && exact_shift_navigation
                                && shift_home_end_extends(
                                    key,
                                    modifiers.control,
                                    cfg!(target_os = "macos"),
                                )
                                && (matches!(key, "home" | "end") || Some(at) != from);
                            if extends_selection {
                                let range = range_keys.read(cx).clone();
                                let next_selection = extend_selection_range(
                                    &selected_keys_held,
                                    &collection,
                                    &selectable,
                                    &range,
                                    &next,
                                );
                                range_keys.update(cx, |range, _| {
                                    if range.anchor.is_none() {
                                        range.anchor = Some(next.clone());
                                    }
                                    range.current = Some(next);
                                    range.is_all = false;
                                });
                                // The uncontrolled set and the form only move
                                // when the extension actually changed something,
                                // but pinned `useMultipleSelectionState` with
                                // Select's `allowDuplicateSelectionEvents`
                                // reports every extension it is handed -- so a
                                // repeated registered Shift+Home/End that
                                // resolved the end already held still reports.
                                if next_selection != selected_keys_held {
                                    if let Some(held) = &indices_own_keys {
                                        form_state_keys.borrow_mut().value =
                                            crate::form::FormValue::Keys(next_selection.clone());
                                        held.update(cx, |selected, cx| {
                                            *selected = next_selection.clone();
                                            cx.notify();
                                        });
                                    }
                                }
                                if let Some(cb) = &on_select_all {
                                    cb(&next_selection, window, cx);
                                }
                            }
                        }
                        crate::list_nav::Move::Activate => {
                            // Select on key-down; the trigger click owns closing
                            // on key-up.
                            let Some(at) = from else { return };
                            let key = &row_keys[at];
                            if multiple {
                                let added = !selected_keys_held.contains(key);
                                let mut next = selected_keys_held.clone();
                                toggle_key(&mut next, key);
                                let next = in_collection_order(&next, &row_keys);
                                // Pinned `toggleSelection` re-anchors on the
                                // add, and a deselect only ends a raw `all`
                                // so the next Shift move extends instead of
                                // collapsing to its target.
                                if added {
                                    range_keys.update(cx, |range, _| {
                                        range.anchor = Some(key.clone());
                                        range.current = Some(key.clone());
                                        range.is_all = false;
                                    });
                                } else {
                                    range_keys.update(cx, |range, _| {
                                        if range.is_all {
                                            *range = SelectSelectionRange::default();
                                        }
                                    });
                                }
                                if let Some(held) = &indices_own_keys {
                                    form_state_keys.borrow_mut().value =
                                        crate::form::FormValue::Keys(next.clone());
                                    held.update(cx, |selected, cx| {
                                        *selected = next.clone();
                                        cx.notify();
                                    });
                                }
                                if let Some(cb) = &on_select_all {
                                    cb(&next, window, cx);
                                }
                                return;
                            }
                            if let Some(held) = &value_own_keys {
                                form_state_keys.borrow_mut().value =
                                    crate::form::FormValue::Keys(vec![key.clone()]);
                                held.update(cx, |v, cx| {
                                    *v = Some(key.clone());
                                    cx.notify();
                                });
                            }
                            if let Some(cb) = &on_select {
                                cb(&Some(key.clone()), window, cx);
                            }
                        }
                        crate::list_nav::Move::Ignore => {
                            // Typeahead moves the cursor over the open list in
                            // either mode -- the open RAC ListBox keeps its
                            // type-select in multiple mode too -- and the move
                            // selects nothing.
                            if !crate::list_nav::is_typeahead_key(key) {
                                return;
                            }
                            let now = web_time::Instant::now();
                            let (query, repeat) = typed.update(cx, |t, _| {
                                let query = t.push(key, now);
                                (query, t.is_repeat())
                            });
                            if let Some(found) =
                                crate::list_nav::typeahead(&labels, &stops, from, &query, repeat)
                            {
                                let found = row_keys[found].clone();
                                held.update(cx, |v, cx| {
                                    *v = Some(found);
                                    cx.notify();
                                });
                            }
                        }
                    }
                });
        }

        let value_text = if multiple {
            self.value_text_multiple(&selected_keys)
        } else {
            self.value_text_single(&selected)
        };
        let chosen = resolved_keys(&self.items, &selected, &selected_keys);
        let has_value = !chosen.is_empty();
        // The chosen rows' labels and positions, walked in collection order so
        // the value slot's `selectedItems` and `selectedIndices` agree.
        let mut chosen_items: Vec<SharedString> = Vec::with_capacity(chosen.len());
        let mut chosen_at: Vec<usize> = Vec::with_capacity(chosen.len());
        for (at, item) in self.items.iter().enumerate() {
            if chosen.contains(item.key()) {
                chosen_items.push(item.label().clone());
                chosen_at.push(at);
            }
        }

        // What the trigger draws when the caller does not: v3's
        // `defaultChildren`, which a `Select.Value` closure can hand straight
        // back for the placeholder case.
        let default_children = gpui::div()
            .flex_1()
            .truncate()
            .text_color(if has_value {
                colors.foreground
            } else {
                colors.muted
            })
            .child(value_text.to_string())
            .into_any_element();

        // `Select.Value` — a caller-drawn value replaces the trigger's text.
        let value_slot = match &self.value_content {
            Some(render) => {
                let names = chosen_items
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>();
                let text = format_selected_names(&names);
                gpui::div()
                    .flex_1()
                    .min_w_0()
                    .child(render(util::SelectionValue {
                        selected_items: &chosen_items,
                        selected_indices: &chosen_at,
                        selected_keys: Some(&chosen),
                        selected_text: &text,
                        is_placeholder: !has_value,
                        default_children,
                    }))
                    .into_any_element()
            }
            None => default_children,
        };
        // `react-aria/dist/private/select/useSelect.mjs` derives the trigger
        // from `useMenuTrigger({type: 'listbox'})`, so
        // `.../overlays/useOverlayTrigger.mjs` gives it
        // `'aria-haspopup': 'listbox'`, `'aria-expanded': isOpen` and
        // `'aria-controls': isOpen ? overlayId : undefined`; HeroUI renders it
        // as an RAC `Button` (`select/select.js`), i.e. a native `<button>`.
        // Only `aria-expanded` ports: `aria-haspopup` and `aria-controls` have
        // no gpui builder and no id graph to point at (see `crate::a11y`).
        // `useSelect` names the trigger `aria-labelledby: [valueId, label]`,
        // which resolves to the field's label followed by the drawn value.
        field = field
            .a11y_named(
                a11y::Role::Button,
                &a11y::Name::maybe(self.label.clone()).described(Some(value_text.clone())),
            )
            .a11y_expanded(is_open);
        field = field.child(value_slot).child(
            gpui::svg()
                .size(px(16.))
                .path(if is_open {
                    // `.select__indicator` turns with the panel.
                    icons::CHEVRON_UP
                } else {
                    icons::CHEVRON_DOWN
                })
                .text_color(colors.muted)
                .flex_shrink_0(),
        );

        if !self.is_disabled && (self.on_open_change.is_some() || open_own.is_some()) {
            let on_open_change = self.on_open_change.clone();
            let own = open_own.clone();
            let open = is_open;
            let pressed = trigger_pressed.clone();
            field = field
                .capture_any_mouse_down(move |_, _, cx| {
                    pressed.set(true);
                    let pressed = pressed.clone();
                    cx.defer(move |_| pressed.set(false));
                })
                .on_click(move |event, window, cx| {
                    // Enter and Space activate the highlighted option before
                    // gpui synthesizes this trigger click. Multiple selection
                    // keeps the popover open for the next pick.
                    let keyboard = matches!(event, gpui::ClickEvent::Keyboard(_));
                    let press_open = keyboard_press_open.update(cx, |value, _| value.take());
                    let started_open = if keyboard {
                        press_open.unwrap_or(open)
                    } else {
                        open
                    };
                    if multiple && started_open && keyboard {
                        return;
                    }
                    // A selection callback can close a controlled popup and
                    // redraw between key-down and key-up. Finish that press's
                    // close rather than toggling the new frame back open.
                    let next_open = !started_open;
                    if next_open == open {
                        return;
                    }
                    // Uncontrolled: flip our own copy, or the trigger would be
                    // inert without a caller handler.
                    if let Some(held) = &own {
                        held.update(cx, |v, cx| {
                            *v = next_open;
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &on_open_change {
                        cb(&next_open, window, cx);
                    }
                });
        }

        // The popup anchors to the trigger bounds — not to the
        // label-to-description wrapper root — the way RAC's
        // `useOverlayPosition` positions against the trigger rect.
        // `scrollable_field_popover` below reads these bounds to flip and
        // cap the panel; the measure element itself only records them.
        let anchor_bounds = Rc::new(std::cell::Cell::new(None));
        let field = crate::popover::PopoverTriggerMeasure::new(field, anchor_bounds.clone());

        // listbox panel
        let mut root = gpui::div().relative();
        root = if self.full_width {
            root.w_full()
        } else {
            root.max_w(px(320.))
        };
        if self.label.is_some() || self.description.is_some() {
            let mut wrapper = gpui::div().flex().flex_col().gap(px(4.)).w_full();
            // Reuse the shared field slots so the required/invalid/disabled
            // treatments match every other control.
            if let Some(label) = &self.label {
                wrapper = wrapper.child(
                    crate::field::Label::new(label.clone())
                        .is_required(self.is_required)
                        .is_disabled(self.is_disabled)
                        .is_invalid(self.is_invalid),
                );
            }
            wrapper = wrapper.child(field);
            if !self.is_invalid {
                if let Some(desc) = &self.description {
                    wrapper = wrapper.child(crate::field::Description::new(desc.clone()));
                }
            }
            root = root.child(wrapper);
        } else {
            root = root.child(field);
        }

        let escape_own = open_own.clone();
        let escape_cb = self.on_open_change.clone();
        root =
            util::dismiss_on_escape_with_token(root, dismissal_token.clone(), move |window, cx| {
                if let Some(held) = &escape_own {
                    held.update(cx, |v, cx| {
                        *v = false;
                        cx.notify();
                    });
                }
                if let Some(cb) = &escape_cb {
                    cb(&false, window, cx);
                }
                util::DismissResult::Handled
            });

        if overlay_active && self.items.is_empty() {
            let dismiss_own = open_own.clone();
            let dismiss_cb = self.on_open_change.clone();
            root = util::dismiss_on_press_outside_with_token(
                root,
                dismissal_token.clone(),
                move |window, cx| {
                    if let Some(held) = &dismiss_own {
                        held.update(cx, |v, cx| {
                            *v = false;
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &dismiss_cb {
                        cb(&false, window, cx);
                    }
                    util::DismissResult::Handled
                },
            );
        }

        if overlay_active && !self.items.is_empty() {
            // The `debug_selector` spellings `pickers_deep.rs` queries on --
            // labels, not ids; the ids beside them derive from `self.id`.
            let base = format!("select-list-{}", id_debug(&self.id));
            let base_id = element_id::scoped(&self.id, "list");
            // `useListBox` is named through `useField`, i.e. by the same
            // `<Label>` the trigger points at.
            let list_name = self.label.clone();
            let options_len = self.items.len();
            let panel_interactive = overlay_phase == util::OverlayPhase::Open;
            let panel = gpui::div()
                .w_full()
                .flex()
                .flex_col()
                .p(px(6.))
                .bg(colors.overlay.background)
                .rounded(util::container_radius(cx))
                // v3 gives a floating panel no border: `.popover` and friends are
                // `bg-overlay shadow-overlay` and a radius, and dark mode's
                // inset hairline is what separates the panel from the page.
                .when_some(layout.overlay_hairline, |el, hairline| {
                el.border(layout.border_width).border_color(hairline)
                })
                .shadow(layout.overlay_shadow.clone())
                // `.select__popover` is `overflow-y-auto`: a long list scrolls
                // rather than being clipped. gpui needs an id for that.
                .id(element_id::scoped(&base_id, "scroll"))
                // RAC's `Select` puts a `ListBox` inside the popover, and
                // `react-aria/dist/private/listbox/useListBox.mjs` is one
                // literal `role: 'listbox'` with `'aria-orientation'`
                // defaulting to vertical. The scroller *is* that list here:
                // the rows are its children. `aria-multiselectable` has no
                // gpui builder (see `crate::a11y`).
                .a11y_named(a11y::Role::ListBox, &a11y::Name::maybe(list_name.clone()))
                .a11y_orientation(herogpui_core::Orientation::Vertical)
                .debug_selector({
                    let base = base.clone();
                    move || format!("{base}-panel")
                })
                .overflow_y_scroll()
                // `overscroll-contain` upstream: a wheel over the popup must
                // not scroll the page behind it (ColorPicker pattern).
                .occlude()
                .track_scroll(&panel_scroll_now)
                // RAC caps the popover at the available viewport height
                // (`calculatePosition`'s `getMaxHeight`); the positioner
                // below re-lays the panel out with that cap, so the panel
                // carries a viewport-relative bound rather than a fixed one.
                .max_h_full();

            // React Aria dismisses the list on a press outside it. Escape is
            // already read by the trigger's key handler, so only the press half
            // is added here. A press that started on the trigger is not an
            // outside press: the trigger's own click owns the close (and the
            // click only fires because the down was not stolen as a dismissal).
            let dismiss_own = open_own.clone();
            let dismiss_cb = self.on_open_change.clone();
            let mut panel = util::dismiss_on_press_outside_with_token(
                panel,
                dismissal_token,
                move |window, cx| {
                    if trigger_pressed.get() {
                        return util::DismissResult::Declined;
                    }
                    if let Some(held) = &dismiss_own {
                        held.update(cx, |v, cx| {
                            *v = false;
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &dismiss_cb {
                        cb(&false, window, cx);
                    }
                    util::DismissResult::Handled
                },
            );

            // The theme tokens the row draws with, copied out: `cx.colors()`
            // hands back a borrow of the app, which a `'static` closure cannot
            // hold.
            let row_muted = colors.muted;
            // `useOption` adds the set position only under virtualization;
            // `row_height` is what turns this list into a windowed one.
            let row_virtualized = self.row_height.is_some();
            let row_fg = colors.foreground;
            let row_focus = colors.focus;
            let row_hover_bg = colors.default.color;
            let row_accent = sem.color;
            let row_disabled_opacity = layout.disabled_opacity;
            // Everything a row reads, owned: `uniform_list`'s callback is
            // `'static` and is called again on every scroll, so it cannot
            // borrow `self` -- and one row builder for both paths is what keeps
            // a virtual list drawing the same row as a short one.
            let items = self.items.clone();
            let sections = self.sections.clone();
            let opt_disabled_keys = self.disabled_keys.clone();
            // The key list the range is resolved against and the enabled
            // keys that may join it, for the Shift-click extension.
            let collection_rows: Vec<SharedString> = keys.clone();
            let selectable_rows: Vec<SharedString> = (0..options_len)
                .filter(|i| !self.disabled_keys.contains(&keys[*i]))
                .map(|i| keys[i].clone())
                .collect();
            let range_rows = selection_range;
            let cursor_rows = cursor;
            let focus_rows = focus_handle;
            let indicator: Option<Rc<dyn Fn(bool) -> gpui::AnyElement>> =
                self.indicator.take().map(Rc::from);
            let on_change_all = self.on_selection_change_all.clone();
            let on_change_one = self.on_selection_change.clone();
            let value_own = value_own;
            let open_own = open_own;
            let form_state_rows = self.form_state.clone();
            let on_close = self.on_open_change.clone();
            let base_row = base;
            let base_row_id = base_id.clone();
            let row = move |i: usize, fixed_h: Option<gpui::Pixels>, cx: &mut App| {
                let base = &base_row;
                let base_id = &base_row_id;
                let opt = &items[i];
                let row_key = opt.key().clone();
                let focus_click = focus_rows.clone();
                let mut rows = Vec::new();
                // `ListBox.Section`'s `Header`: `text-xs` in the muted colour,
                // above the option it introduces.
                if let Some((_, label)) = sections.iter().find(|(at, _)| at == &row_key) {
                    rows.push(
                        gpui::div()
                            .px(px(8.))
                            .pt(px(6.))
                            .pb(px(4.))
                            .text_size(px(12.))
                            .line_height(px(16.))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(row_muted)
                            .child(label.to_string())
                            .into_any_element(),
                    );
                }
                let is_sel = if multiple {
                    selected_keys.contains(&row_key)
                } else {
                    selected.as_ref() == Some(&row_key)
                };
                let opt_disabled = opt_disabled_keys.contains(&row_key);
                let row_selector = format!("{base}-opt-{i}");
                let mut item = gpui::div()
                        .id(element_id::indexed(base_id, "opt", i))
                        // `useOption.mjs`: `role: 'option'` with
                        // `'aria-selected': selectionMode !== 'none' ?
                        // isSelected : undefined`. A Select's list always
                        // selects, so the flag is unconditional here. Its
                        // `aria-posinset`/`aria-setsize` pair is set only
                        // `if (isVirtualized)`, which is exactly this port's
                        // `row_height` path.
                        .a11y_named(
                            a11y::Role::ListBoxOption,
                            &a11y::Name::labelled(opt.label().clone()),
                        )
                        .a11y_selected(is_sel)
                        .when(row_virtualized, |item| {
                            item.a11y_set_position(i, options_len)
                        })
                        // The trigger keeps the real focus while the list is
                        // open, and a cursor walks the rows -- upstream's
                        // `shouldUseVirtualFocus`. gpui states that relation
                        // on the descendant rather than on the container.
                        .when(cursor_at == Some(i), |item| item.a11y_active_descendant())
                        .debug_selector(move || row_selector)
                        .flex()
                        .items_center()
                        .justify_between()
                        // Every menu row in v3 is a `.list-box-item`: `min-h-9
                        // rounded-2xl px-2 py-1.5 gap-3` at `text-sm`.
                        .min_h(util::FIELD_HEIGHT)
                        .rounded(util::soft_radius(cx))
                        .px(px(10.))
                        .py(px(6.))
                        .gap(px(12.))
                        .text_size(util::FIELD_TEXT)
                        .line_height(px(20.));

                if opt_disabled {
                    item = item.opacity(row_disabled_opacity);
                } else if panel_interactive {
                    item = item.cursor_pointer().hover(move |s| s.bg(row_hover_bg));
                }

                if is_sel {
                    item = item.text_color(row_accent);
                } else {
                    item = item.text_color(row_fg);
                }

                // `status-focused` on the row the keyboard is on.
                if cursor_at == Some(i) {
                    item = item.border_2().border_color(row_focus);
                }

                item = item.child(gpui::div().truncate().child(opt.label().to_string()));

                match &indicator {
                    Some(render) => item = item.child(render(is_sel)),
                    None if is_sel => {
                        item = item.child(
                            gpui::svg()
                                .size(px(13.))
                                .path(icons::CHECK)
                                .text_color(row_accent),
                        );
                    }
                    None => {}
                }

                if panel_interactive && !opt_disabled {
                    if multiple {
                        if indices_own.is_some() || on_change_all.is_some() {
                            let current = selected_keys.clone();
                            let own = indices_own.clone();
                            let cb = on_change_all.clone();
                            let form_state_pick = form_state_rows.clone();
                            let range_click = range_rows.clone();
                            let collection_click = collection_rows.clone();
                            let selectable_click = selectable_rows.clone();
                            let cursor_click = cursor_rows.clone();
                            let picked_key = row_key;
                            item = item.on_click(move |ev, window, cx| {
                                // Pinned `useSelectableItem` seats the cursor
                                // on pointer press, so a Shift+Arrow, page, or
                                // Enter that follows starts from the clicked
                                // row rather than from a null or stale cursor.
                                cursor_click.update(cx, |v, cx| {
                                    *v = Some(picked_key.clone());
                                    cx.notify();
                                });
                                // gpui's own focus-on-press would park focus
                                // on the row and deafen the trigger's key
                                // handler; the trigger is what holds focus so
                                // the open list stays keyboard-walkable.
                                window.focus(&focus_click, cx);
                                let mut next = current.clone();
                                // A Shift click extends from the anchor
                                // through `extendSelection`; an ordinary or
                                // platform-Mod click toggles and re-anchors on
                                // the add, the way pinned `toggleSelection`
                                // does -- a deselect only ends a raw `all`.
                                if ev.modifiers().shift {
                                    let range = range_click.read(cx).clone();
                                    next = extend_selection_range(
                                        &current,
                                        &collection_click,
                                        &selectable_click,
                                        &range,
                                        &picked_key,
                                    );
                                    range_click.update(cx, |range, _| {
                                        if range.anchor.is_none() {
                                            range.anchor = Some(picked_key.clone());
                                        }
                                        range.current = Some(picked_key.clone());
                                        range.is_all = false;
                                    });
                                } else {
                                    let added = !next.contains(&picked_key);
                                    toggle_key(&mut next, &picked_key);
                                    next = in_collection_order(&next, &collection_click);
                                    if added {
                                        range_click.update(cx, |range, _| {
                                            range.anchor = Some(picked_key.clone());
                                            range.current = Some(picked_key.clone());
                                            range.is_all = false;
                                        });
                                    } else {
                                        range_click.update(cx, |range, _| {
                                            if range.is_all {
                                                *range = SelectSelectionRange::default();
                                            }
                                        });
                                    }
                                }
                                if let Some(held) = &own {
                                    form_state_pick.borrow_mut().value =
                                        crate::form::FormValue::Keys(next.clone());
                                    held.update(cx, |selected, cx| {
                                        *selected = next.clone();
                                        cx.notify();
                                    });
                                }
                                if let Some(cb) = &cb {
                                    cb(&next, window, cx);
                                }
                            });
                        }
                    } else if on_change_one.is_some() || value_own.is_some() || open_own.is_some() {
                        let on_select = on_change_one.clone();
                        let on_close = on_close.clone();
                        let value_own = value_own.clone();
                        let open_own = open_own.clone();
                        let form_state_pick = form_state_rows.clone();
                        let picked_key = row_key;
                        item = item.on_click(move |_, window, cx| {
                            // Uncontrolled: take the selection and close, or
                            // choosing an option would do nothing.
                            if let Some(held) = &value_own {
                                form_state_pick.borrow_mut().value =
                                    crate::form::FormValue::Keys(vec![picked_key.clone()]);
                                held.update(cx, |v, cx| {
                                    *v = Some(picked_key.clone());
                                    cx.notify();
                                });
                            }
                            if let Some(held) = &open_own {
                                held.update(cx, |v, cx| {
                                    *v = false;
                                    cx.notify();
                                });
                            }
                            // A single-mode pick closes the popover, and a
                            // caller who drives `isOpen` has to hear about it:
                            // without this the keyed flag flipped while the
                            // callback stayed silent, so a controlled caller
                            // still saw the panel open and reopened it on the
                            // next render. Reported exactly here by a pointer
                            // pick; the Enter path's close is the trigger's
                            // own click listener (gpui fires a focused
                            // element's click on Enter), so it never reaches
                            // this closure.
                            if let Some(cb) = &on_close {
                                cb(&false, window, cx);
                            }
                            if let Some(f) = &on_select {
                                f(&Some(picked_key.clone()), window, cx);
                            }
                        });
                    }
                }

                rows.push(item.into_any_element());
                gpui::div()
                    .flex()
                    .flex_col()
                    .when_some(fixed_h, |el, h| el.h(h).w_full())
                    .children(rows)
                    .into_any_element()
            };

            match self.row_height {
                // Virtual: only the rows in view are built, which is what makes
                // a thousand options affordable. The list itself is the scroll
                // container: `Infer` sizes it from its rows — the full
                // natural height on the positioner's measure pass (so the
                // flip sees the real extent, like upstream's `overlaySize`),
                // capped to the available height on the capped pass — while
                // the ListBox stays `overflow-clip`, as in v3. A fixed inner
                // height plus an outer scroller would nest two scroll
                // containers and strand rows between them.
                Some(row_height) => {
                    panel = panel.child(
                        gpui::uniform_list(
                            element_id::scoped(&base_id, "rows"),
                            options_len,
                            move |range, _window, cx| {
                                range
                                    .map(|i| row(i, Some(row_height), cx))
                                    .collect::<Vec<_>>()
                            },
                        )
                        .track_scroll(&list_scroll_now)
                        .with_sizing_behavior(gpui::ListSizingBehavior::Infer)
                        .w_full(),
                    );
                }
                None => {
                    for i in 0..options_len {
                        panel = panel.child(row(i, None, cx));
                    }
                }
            }

            let zoom = crate::anim::ZoomBox::panel(px(6.), util::container_radius(cx));
            let panel = if overlay_phase == util::OverlayPhase::Exiting {
                crate::anim::exiting(
                    panel,
                    element_id::scoped(&base_id, "panel-out"),
                    zoom,
                    crate::anim::Motion::LIST_OUT,
                    cx,
                )
            } else {
                crate::anim::entering_zoom(
                    panel,
                    element_id::scoped(&base_id, "panel"),
                    zoom,
                    crate::anim::Motion::LIST_IN,
                    cx,
                )
            };
            // RAC positions the popover against the trigger with an 8px gap,
            // flips it when the other side has more room, and caps it at the
            // available viewport height past a 12px inset — which `Select`
            // inherits unchanged from `useOverlayPosition`/`Popover`.
            root = root.child(util::floating(crate::popover::scrollable_field_popover(
                anchor_bounds,
                self.placement,
                panel,
            )));
        }

        root = util::apply_sx(root, &self.sx);
        root.track_focus(&blur_scope)
    }
}

fn live_form_state() -> Rc<RefCell<crate::form::LiveFormFieldState>> {
    Rc::new(RefCell::new(crate::form::LiveFormFieldState {
        value: crate::form::FormValue::Text(SharedString::default()),
        is_invalid: false,
        is_successful: true,
        focus: None,
        restore: None,
    }))
}

/// The form value both modes submit: the selection's keys, each resolved to a
/// collection item, in row order — the channel `@heroui/autocomplete` submits
/// the same way, and what a keyed collection's hidden input carries upstream.
fn select_form_value(
    items: &[PickerItem],
    single: &Option<SharedString>,
    multiple: &[SharedString],
) -> crate::form::FormValue {
    crate::form::FormValue::Keys(resolved_keys(items, single, multiple))
}

fn sync_select_form(
    state: &Rc<RefCell<crate::form::LiveFormFieldState>>,
    value: crate::form::FormValue,
    is_invalid: bool,
    is_successful: bool,
) {
    let mut state = state.borrow_mut();
    state.value = value;
    state.is_invalid = is_invalid;
    state.is_successful = is_successful;
}

fn id_debug(id: &gpui::ElementId) -> String {
    format!("{id:?}").trim_matches('"').to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Home/End gate takes the platform as an explicit bool, so this
    /// truth table is free of `cfg!` and mechanically proves both maps from
    /// any host: no macOS chord ever extends -- Shift and Alt+Shift move the
    /// cursor alone -- while Windows and Linux extend exactly from
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
                "plain Shift+{key} must only move the cursor"
            );
        }
    }

    /// Arrows and page keys never consult the Home/End gate: their forbidden
    /// extra chords are rejected earlier, by `exact_shift_navigation`.
    #[test]
    fn arrows_and_pages_skip_the_home_end_gate() {
        assert!(shift_home_end_extends("down", false, true));
        assert!(shift_home_end_extends("up", true, false));
        assert!(shift_home_end_extends("pagedown", false, false));
        assert!(shift_home_end_extends("pageup", true, true));
    }

    #[test]
    fn home_end_registration_follows_the_platform_maps() {
        let none = gpui::Modifiers::none();
        let mut shift = none;
        shift.shift = true;
        let mut alt = none;
        alt.alt = true;
        let mut alt_shift = alt;
        alt_shift.shift = true;
        let mut control = none;
        control.control = true;
        let mut control_shift = control;
        control_shift.shift = true;
        let mut platform = none;
        platform.platform = true;
        let mut platform_shift = platform;
        platform_shift.shift = true;
        let mut function = none;
        function.function = true;
        let mut function_alt = function;
        function_alt.alt = true;

        for modifiers in [none, shift, alt, alt_shift, function, function_alt] {
            assert!(
                home_end_registered(modifiers, true),
                "macOS must register {modifiers:?}"
            );
        }
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

    #[test]
    fn default_value_text_matches_pinned_select_value() {
        let select = Select::new(
            "select-default-text",
            vec![
                PickerItem::new("alpha", "Alpha"),
                PickerItem::new("beta", "Beta"),
                PickerItem::new("gamma", "Gamma"),
            ],
        )
        .selection_mode(SelectionMode::Multiple)
        .selected_keys(["alpha", "beta", "gamma"].map(SharedString::from));

        assert_eq!(
            select.value_text_multiple(&select.selected_keys),
            "Alpha, Beta, and Gamma"
        );
        assert_eq!(
            format_selected_names(&["Alpha".into(), "Beta".into()]),
            "Alpha and Beta"
        );
        assert_eq!(
            Select::new("select-default-placeholder", Vec::new()).placeholder,
            "Select an item"
        );
    }

    /// The keyed selection channel: keys, not labels, address items, so two
    /// items may share a label without aliasing each other, the reads walk
    /// the collection however the keys were picked or listed, and keys the
    /// collection does not hold resolve to nothing.
    #[test]
    fn keys_address_items_and_reads_walk_the_collection() {
        let keys = |names: &[&str]| -> Vec<SharedString> {
            names.iter().copied().map(SharedString::from).collect()
        };
        let items = vec![
            PickerItem::new("a", "Paris"),
            PickerItem::new("b", "London"),
            PickerItem::new("c", "Paris"),
        ];
        // Same label, distinct keys: each key resolves to its own item.
        let select = Select::new("select-keyed", items).value(Some(SharedString::from("c")));
        assert_eq!(
            select.value_text_single(&select.selected),
            "Paris",
            "the key must resolve to its own item, never a same-label sibling"
        );
        // A multiple selection reports in collection order however it was
        // listed, and an unknown key resolves to nothing.
        assert_eq!(
            in_collection_order(&keys(&["c", "a", "zz"]), &keys(&["a", "b", "c"])),
            keys(&["a", "c"]),
            "reads must walk the collection and drop unresolvable keys"
        );
        assert_eq!(
            resolved_keys(&select.items, &None, &keys(&["c", "b"])),
            keys(&["b", "c"]),
            "the resolved set follows the collection's row order"
        );
        assert_eq!(
            resolved_keys(&select.items, &Some(SharedString::from("gone")), &[]),
            Vec::<SharedString>::new(),
            "a key with no item resolves to no selection"
        );
    }

    // The pinned `.select--secondary` hover fill is
    // `--select-trigger-bg-hover: var(--default-hover)` and the popup rows
    // fill with the full `bg-default`. The two accessors differ by one word
    // and the wrong one still looks plausible on screen, so the check is
    // mechanical.
    #[test]
    fn secondary_trigger_and_menu_rows_use_the_pinned_hover_tokens() {
        // Scan the implementation only; this test's own text names the
        // forbidden accessors.
        let source = include_str!("select.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        assert!(
            source.contains("FieldVariant::Secondary => colors.default.hover()"),
            "the secondary trigger hover must read `colors.default.hover()` \
             (pinned `--select-trigger-bg-hover: var(--default-hover)`)"
        );
        assert!(
            source.contains("let row_hover_bg = colors.default.color;"),
            "the popup rows must hover the full `bg-default` \
             (pinned `.list-box-item:hover`)"
        );
        assert!(
            !source.contains("soft_hover()") && !source.contains("default.soft()"),
            "no select surface may hover a soft token"
        );
    }
}
