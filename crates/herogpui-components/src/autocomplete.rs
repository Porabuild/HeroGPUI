//! Autocomplete — port of `@heroui/autocomplete`.
//!
//! v3's Autocomplete is a **Select whose popover holds a search field**, not a
//! text field with suggestions under it. `autocomplete.css` says so directly:
//! `.autocomplete__trigger` is a field-shaped box (`min-h-9 rounded-field
//! bg-field px-3 shadow-field`) holding `.autocomplete__value` and a chevron
//! `.autocomplete__indicator`, and `.autocomplete__popover` stacks
//! `[data-slot="search-field"]` above the list. The text field *with* a list is
//! the [`crate::combo_box::ComboBox`], which is a separate component with its
//! own stylesheet.
//!
//! This port had it the other way round -- an [`Input`] with a suggestion panel
//! -- which drew none of that sheet and left the trigger nothing to show the
//! selection in. The trigger draws the selection now, and the popover searches.
//!
//! [`InputState`] therefore backs the **search field inside the popover**, which
//! is what `Autocomplete.Filter`'s `inputValue` and `onInputChange` address. The
//! selection is a set of item keys, held by `value` / `defaultValue`.
//!
//! Pinned v3.2.4 / React Aria Components 1.20.0 keep a stable `Key` separate
//! from each item's `textValue`: `value` / `defaultValue` / `disabledKeys`,
//! the selection callbacks and the form value address items by key, while
//! filtering and the visible text use the label. Items are therefore
//! [`crate::PickerItem`]s; using a label as the key made duplicate labels alias
//! each other's selection, disabled state and row identity.
//!
//! Pinned react-stately 3.49.0's `useSelectState` holds `selectedKeys` as a
//! JavaScript `Set`, which iterates in insertion order: `selectedItems`,
//! `selectedText`, the selection callbacks and the form value all follow the
//! *selection's* order, not the collection's. The selection is therefore an
//! ordered unique key list, and toggling removes in place or appends.

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use gpui::{
    prelude::*, px, App, Entity, IntoElement, RenderOnce, SharedString, StatefulInteractiveElement,
    Styled, Window,
};
use herogpui_core::{FieldVariant, Placement, SelectionMode};
use herogpui_theme::ActiveTheme;

use crate::{
    icons,
    input::{InputState, SearchField},
    picker_item::PickerItem,
    util,
};

type OnSelectionChange = std::sync::Arc<dyn Fn(&SharedString, &mut Window, &mut App) + 'static>;
type AutocompleteFormState = Rc<RefCell<crate::form::LiveFormFieldState>>;

thread_local! {
    static AUTOCOMPLETE_FORM_STATES: RefCell<
        HashMap<u64, std::rc::Weak<RefCell<crate::form::LiveFormFieldState>>>,
    > = RefCell::new(HashMap::new());
}

fn autocomplete_form_state(entity_id: u64) -> AutocompleteFormState {
    AUTOCOMPLETE_FORM_STATES.with(|states| {
        let mut states = states.borrow_mut();
        if let Some(state) = states.get(&entity_id).and_then(|state| state.upgrade()) {
            return state;
        }
        let state = Rc::new(RefCell::new(crate::form::LiveFormFieldState {
            value: crate::form::FormValue::Keys(Vec::new()),
            is_invalid: false,
            is_successful: true,
            focus: None,
            restore: None,
        }));
        states.insert(entity_id, Rc::downgrade(&state));
        state
    })
}

fn form_selection_value(selected: &[SharedString]) -> crate::form::FormValue {
    crate::form::FormValue::Keys(selected.to_vec())
}

/// Normalizes the selection to the shape of pinned react-stately 3.49.0's
/// `selectedKeys`: a `Set`, which collapses duplicates to the first insertion
/// and iterates in insertion order. A single-mode selection holds at most one
/// key — the first the owner (or default) listed.
fn normalize_selection(keys: Vec<SharedString>, multiple: bool) -> Vec<SharedString> {
    let mut out: Vec<SharedString> = Vec::with_capacity(keys.len());
    for key in keys {
        if out.contains(&key) {
            continue;
        }
        out.push(key);
        if !multiple {
            break;
        }
    }
    out
}

/// Toggles `key` in the ordered selection: removing takes it out in place so
/// the remaining keys keep their insertion order; adding appends — the
/// mutation a JS `Set` performs.
fn toggle_key(selection: &mut Vec<SharedString>, key: &SharedString) {
    if let Some(at) = selection.iter().position(|k| k == key) {
        selection.remove(at);
    } else {
        selection.push(key.clone());
    }
}

fn sync_form_state(
    state: &AutocompleteFormState,
    selected: &[SharedString],
    is_disabled: bool,
    is_invalid: bool,
) {
    let mut state = state.borrow_mut();
    state.value = form_selection_value(selected);
    state.is_successful = !is_disabled;
    state.is_invalid = is_invalid;
}

/// HeroUI Autocomplete.
#[derive(IntoElement)]
pub struct Autocomplete {
    /// `name` — the name this control submits under; read back by
    /// [`Self::form_field`].
    name: Option<SharedString>,
    /// The search field's text state — `Autocomplete.Filter`'s `inputValue`.
    state: Entity<InputState>,
    items: Vec<PickerItem>,
    max_items: usize,
    /// `ListLayout`'s `rowHeight`, which virtualizes the popover list.
    row_height: Option<gpui::Pixels>,
    label: Option<SharedString>,
    placeholder: Option<SharedString>,
    description: Option<SharedString>,
    error_message: Option<SharedString>,
    variant: FieldVariant,
    full_width: bool,
    is_disabled: bool,
    is_read_only: bool,
    is_invalid: bool,
    is_required: bool,
    /// `disabledKeys` — suggestions that render but cannot be chosen,
    /// addressed by item key so a disabled item never disables its
    /// same-label sibling.
    disabled_keys: std::collections::HashSet<SharedString>,
    /// `shouldFocusWrap` — whether the arrow keys wrap at the ends of the list.
    should_focus_wrap: bool,
    /// `ListBox.Section` — a heading above the item with this key.
    sections: Vec<(SharedString, SharedString)>,
    /// `Autocomplete.Indicator` — replaces the trigger chevron. The closure is
    /// handed whether the popover is open.
    indicator: Option<Box<dyn Fn(bool) -> gpui::AnyElement + 'static>>,
    /// Composed `ListBox.ItemIndicator` — draws the selection tick. The closure
    /// is handed whether the row is selected.
    item_indicator: Option<Box<dyn Fn(bool) -> gpui::AnyElement + 'static>>,
    /// `Autocomplete.Value` — draws the trigger's value.
    value_content: Option<Box<dyn Fn(util::SelectionValue<'_>) -> gpui::AnyElement + 'static>>,
    /// `allowsEmptyCollection` — whether the autocomplete may function when
    /// the collection has no items at all. It is the `useSelectState` open
    /// gate, not a close-on-filtered-empty flag: filtering an open popover
    /// to zero keeps it mounted with the empty state either way.
    allows_empty_collection: bool,
    selection_mode: SelectionMode,
    /// The selection as ordered unique item keys — react-stately 3.49.0's
    /// `selectedKeys` is a JS `Set`, which iterates in insertion order, so
    /// callbacks, the form value and the trigger all follow the order the
    /// keys were picked (or the owner listed) in.
    selected_keys: Vec<SharedString>,
    /// Whether the caller drives the selection. An unset `value` is not an empty
    /// controlled selection: without this flag every uncontrolled Autocomplete
    /// would hand its own clicks back to a set nobody owns, and picking an item
    /// would do nothing.
    is_controlled: bool,
    /// `defaultValue` — set it to hand this component its own selection.
    default_value: Option<Vec<SharedString>>,
    on_selection_change_all:
        Option<std::sync::Arc<dyn Fn(&[SharedString], &mut Window, &mut App) + 'static>>,
    /// `isOpen`. `None` lets the trigger own it, seeded from `defaultOpen`.
    is_open: Option<bool>,
    default_open: bool,
    placement: Placement,
    on_open_change: Option<std::sync::Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>>,
    on_selection_change: Option<OnSelectionChange>,
    /// `filter` — decides whether an item matches the query. Defaults to a
    /// case-insensitive substring test.
    filter: Option<std::sync::Arc<dyn Fn(&str, &str) -> bool + 'static>>,
    input_value: Option<String>,
    on_input_change: Option<std::sync::Arc<dyn Fn(&str, &mut Window, &mut App) + 'static>>,
    on_clear: Option<std::sync::Arc<dyn Fn(&mut Window, &mut App) + 'static>>,
    form_state: AutocompleteFormState,
}

impl Autocomplete {
    /// `selectionMode`
    pub fn selection_mode(mut self, mode: SelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }

    /// `defaultValue` — the uncontrolled initial selection.
    ///
    /// Supplying it hands the component its own selection set, seeded once;
    /// [`Self::value`] is the controlled spelling. The listed order is the
    /// selection's order, exactly as the owner listed it.
    pub fn default_value(
        mut self,
        keys: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.default_value = Some(keys.into_iter().map(Into::into).collect());
        self
    }

    /// `value` — the controlled selection, as item keys. The listed order is
    /// the owner's order and is preserved everywhere the selection is read.
    pub fn value(mut self, keys: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.selected_keys = keys.into_iter().map(Into::into).collect();
        self.is_controlled = true;
        self
    }

    /// The `ListBox`'s spelling of [`Self::value`], for a caller that has a set.
    pub fn selected_keys(mut self, keys: impl IntoIterator<Item = SharedString>) -> Self {
        self.selected_keys = keys.into_iter().collect();
        self.is_controlled = true;
        self
    }

    /// `onChange`'s complete `Key | Key[] | null` domain as a selection slice.
    /// Single selection reports zero or one key; multiple reports every key.
    pub fn on_selection_change_all(
        mut self,
        handler: impl Fn(&[SharedString], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change_all = Some(std::sync::Arc::new(handler));
        self
    }

    /// `placement` on `Autocomplete.Popover`.
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    /// `defaultOpen` — the popover starts open.
    pub fn default_open(mut self, v: bool) -> Self {
        self.default_open = v;
        self
    }

    /// `isOpen` — the controlled popover state.
    pub fn is_open(mut self, v: bool) -> Self {
        self.is_open = Some(v);
        self
    }

    /// `onOpenChange`
    pub fn on_open_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_change = Some(std::sync::Arc::new(handler));
        self
    }

    /// `filter` on `Autocomplete.Filter` — replaces the default
    /// case-insensitive substring match.
    ///
    /// Called as `filter(item_label, input)`: v3 filters on the item's
    /// `textValue`, not on its key.
    pub fn filter(mut self, f: impl Fn(&str, &str) -> bool + 'static) -> Self {
        self.filter = Some(std::sync::Arc::new(f));
        self
    }

    /// `inputValue` on `Autocomplete.Filter` — the controlled search text.
    ///
    /// Unlike the bound [`InputState`] this does not write through, so a caller
    /// can hold the query itself.
    pub fn input_value(mut self, value: impl Into<String>) -> Self {
        self.input_value = Some(value.into());
        self
    }

    /// `onInputChange` on `Autocomplete.Filter` — every keystroke in the search
    /// field.
    pub fn on_input_change(mut self, f: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_input_change = Some(std::sync::Arc::new(f));
        self
    }

    /// `onClear` — called after `Autocomplete.ClearButton` clears selection.
    /// The button's clearing behavior does not depend on this callback.
    pub fn on_clear(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_clear = Some(std::sync::Arc::new(f));
        self
    }

    /// Pick-only single-key convenience callback.
    ///
    /// Use [`Self::on_selection_change_all`] for v3's complete `onChange`
    /// domain, including multiple selection and the empty value from clear.
    pub fn on_change(
        self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change(handler)
    }

    pub fn new(state: Entity<InputState>, items: Vec<PickerItem>) -> Self {
        let form_state = autocomplete_form_state(state.entity_id().as_u64());
        Self {
            name: None,
            state,
            items,
            max_items: 100,
            row_height: None,
            label: None,
            placeholder: None,
            description: None,
            error_message: None,
            variant: FieldVariant::Primary,
            full_width: false,
            is_disabled: false,
            is_read_only: false,
            is_invalid: false,
            is_required: false,
            disabled_keys: std::collections::HashSet::new(),
            should_focus_wrap: false,
            sections: Vec::new(),
            indicator: None,
            item_indicator: None,
            value_content: None,
            allows_empty_collection: false,
            selection_mode: SelectionMode::Single,
            selected_keys: Vec::new(),
            is_controlled: false,
            default_value: None,
            on_selection_change_all: None,
            filter: None,
            input_value: None,
            on_input_change: None,
            on_clear: None,
            is_open: None,
            default_open: false,
            placement: Placement::BottomStart,
            on_open_change: None,
            on_selection_change: None,
            form_state,
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
    /// its ancestor, so the control hands the pair over instead. The live
    /// selection survives the next `Autocomplete::new` because it is keyed by
    /// the search-field entity, the way DateField keys its form state. A
    /// disabled control stays registered and is omitted from FormData.
    ///
    /// ```ignore
    /// let field = control.form_field();
    /// form.field(field.unwrap()).child(control)
    /// ```
    pub fn form_field(&self) -> Option<crate::form::FormField> {
        let name = self.name.clone()?;
        {
            let mut state = self.form_state.borrow_mut();
            state.is_successful = !self.is_disabled;
            state.is_invalid = self.is_invalid || self.error_message.is_some();
        }
        Some(
            crate::form::FormField::live(name, self.form_state.clone())
                .is_required(self.is_required),
        )
    }

    /// `ListLayout`'s `rowHeight` -- and what virtualizes the popover list.
    ///
    /// v3 wraps the list in `<Virtualizer layout={ListLayout}>` inside
    /// `Autocomplete.Popover`; gpui's `uniform_list` builds only the rows in
    /// view, and it can do that because every row is this tall.
    pub fn row_height(mut self, h: impl Into<gpui::Pixels>) -> Self {
        self.row_height = Some(h.into());
        self
    }

    pub fn max_items(mut self, n: usize) -> Self {
        self.max_items = n.max(1);
        self
    }

    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = Some(l.into());
        self
    }

    pub fn placeholder(mut self, p: impl Into<SharedString>) -> Self {
        self.placeholder = Some(p.into());
        self
    }

    pub fn description(mut self, text: impl Into<SharedString>) -> Self {
        self.description = Some(text.into());
        self
    }

    pub fn error_message(mut self, text: impl Into<SharedString>) -> Self {
        self.error_message = Some(text.into());
        self
    }

    pub fn variant(mut self, variant: FieldVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn full_width(mut self, v: bool) -> Self {
        self.full_width = v;
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    /// A read-only Autocomplete shows its selection and does not open.
    pub fn is_read_only(mut self, v: bool) -> Self {
        self.is_read_only = v;
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

    /// `disabledKeys` — keys of the suggestions that render but cannot be
    /// chosen. Disabled state is per key, so one of two same-label items can
    /// be disabled alone.
    pub fn disabled_keys(mut self, keys: impl IntoIterator<Item = SharedString>) -> Self {
        self.disabled_keys = keys.into_iter().collect();
        self
    }

    /// `shouldFocusWrap` — whether the arrow keys wrap at the ends of the list.
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

    /// `Autocomplete.Indicator` — draw the trigger indicator yourself.
    ///
    /// The closure receives the current open state, which is the GPUI analog of
    /// v3's `data-open` attribute on this part.
    pub fn indicator(mut self, render: impl Fn(bool) -> gpui::AnyElement + 'static) -> Self {
        self.indicator = Some(Box::new(render));
        self
    }

    /// Composed `ListBox.ItemIndicator` — draw the selected row tick yourself.
    pub fn item_indicator(mut self, render: impl Fn(bool) -> gpui::AnyElement + 'static) -> Self {
        self.item_indicator = Some(Box::new(render));
        self
    }

    /// `Autocomplete.Value` — draw the trigger's value yourself.
    ///
    /// The closure is handed the render props v3 passes into
    /// `<Autocomplete.Value>{({defaultChildren, isPlaceholder, selectedItems,
    /// selectedText}) => …}`, so a multiple selection can be drawn as tags.
    pub fn value_content(
        mut self,
        render: impl Fn(util::SelectionValue<'_>) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.value_content = Some(Box::new(render));
        self
    }

    /// `allowsEmptyCollection` — whether the autocomplete may function with a
    /// collection that has no items at all (v3: *"When true, the autocomplete
    /// can function even with no items."*).
    ///
    /// react-stately 3.49.0's `useSelectState` reads it as the `open`/`toggle`
    /// gate: a truly empty collection refuses to open without it. Filtering an
    /// open popover to zero is a different layer (the ListBox's empty-state
    /// slot), so this is not a close-on-filtered-empty flag.
    pub fn allows_empty_collection(mut self, v: bool) -> Self {
        self.allows_empty_collection = v;
        self
    }

    pub fn on_selection_change(
        mut self,
        f: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change = Some(std::sync::Arc::new(f));
        self
    }
}

fn el_name(s: String) -> gpui::ElementId {
    gpui::ElementId::Name(s.into())
}

impl RenderOnce for Autocomplete {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let base = format!("autocomplete-{}", self.state.entity_id().as_u64());
        // `Autocomplete.Filter.inputValue` is controlled state. Keep the bound
        // search field on the owner's value while still reporting proposed
        // edits through `onInputChange`.
        if let Some(input_value) = self.input_value.clone() {
            let current = self.state.read(cx).value().to_owned();
            if current != input_value {
                self.state.update(cx, |state, cx| {
                    state.set_value(input_value);
                    cx.notify();
                });
            }
        }
        // `defaultValue` opts into the component holding its own selection;
        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        // Both spellings normalize to the `Set` shape first: duplicates
        // collapse to their first insertion, and single mode keeps one key.
        // A controlled order is the owner's order — nothing is sorted.
        let multiple = self.selection_mode == SelectionMode::Multiple;
        self.selected_keys = normalize_selection(self.selected_keys.clone(), multiple);
        let (selection, selection_own) = util::controlled(
            window,
            cx,
            el_name(format!("{base}-selection")),
            self.is_controlled.then(|| self.selected_keys.clone()),
            normalize_selection(self.default_value.clone().unwrap_or_default(), multiple),
        );
        self.selected_keys = selection;

        // `isOpen` / `defaultOpen`: the trigger owns the popover, the way
        // `.autocomplete__trigger` does in v3 -- the old port opened on focus,
        // which is a ComboBox's behaviour, not this one's.
        let (is_open, open_own) = util::controlled(
            window,
            cx,
            el_name(format!("{base}-open")),
            self.is_open,
            self.default_open,
        );
        let open = is_open && !self.is_disabled;
        let (overlay_phase, dismissal_token) =
            util::overlay_scope(window, cx, el_name(format!("{base}-overlay")), open, true);
        let overlay_active = overlay_phase != util::OverlayPhase::Closed;
        let overlay_exiting = overlay_phase == util::OverlayPhase::Exiting;

        // `usePopover` closes when focus leaves the trigger-plus-panel scope.
        // Unlike Escape, blur leaves focus on its destination.
        let blur_close_own = open_own.clone();
        let blur_open_change = self.on_open_change.clone();
        let blur_scope = util::close_on_blur(window, cx, &base, open, move |window, cx| {
            if let Some(held) = &blur_close_own {
                held.update(cx, |v, cx| {
                    *v = false;
                    cx.notify();
                });
            }
            if let Some(cb) = &blur_open_change {
                cb(false, window, cx);
            }
        });

        // The trigger is what holds focus, so the open list can be walked with
        // the arrows. A disabled control leaves the tab order.
        let focus_handle = if self.is_disabled {
            None
        } else {
            Some(util::tab_stop_handle(
                el_name(format!("{base}-focus")),
                window,
                cx,
            ))
        };
        // Which row the keyboard is on, held as the item's *key* so the cursor
        // stays on the same item when the query filters or the caller reorders
        // the collection.
        let cursor = window.use_keyed_state(el_name(format!("{base}-cursor")), cx, |_, _| {
            None::<SharedString>
        });
        // React Aria keeps the focused row in view, and v3's list is
        // `overflow-y-auto`. The virtual list has its own handle kind, a
        // scrolling div the other. `use_keyed_state` takes `cx` mutably, so both
        // precede the theme tokens.
        let list_scroll =
            window.use_keyed_state(el_name(format!("{base}-list-scroll")), cx, |_, _| {
                gpui::UniformListScrollHandle::new()
            });
        let panel_scroll =
            window.use_keyed_state(el_name(format!("{base}-panel-scroll")), cx, |_, _| {
                gpui::ScrollHandle::new()
            });
        let list_scroll_now = list_scroll.read(cx).clone();
        let panel_scroll_now = panel_scroll.read(cx).clone();
        // v3 writes `<SearchField autoFocus>` inside `Autocomplete.Filter`, so
        // the query field takes the focus as the popover opens -- once per
        // opening, or it would take the focus back on every frame.
        let autofocused =
            window.use_keyed_state(el_name(format!("{base}-autofocus")), cx, |_, _| false);
        // SearchField's text callback and the bubbling key event cooperate to
        // classify the pending edit; see the block after `matches` below.
        let query_edit = window.use_keyed_state(
            el_name(format!("{base}-query-edit")),
            cx,
            |_, _| None::<bool>,
        );
        let plain_edit_key =
            window.use_keyed_state(el_name(format!("{base}-plain-edit-key")), cx, |_, _| false);
        let search_focus = self.state.read(cx).focus_handle.clone();
        if open && !*autofocused.read(cx) {
            window.focus(&search_focus);
            autofocused.update(cx, |v, _| *v = true);
        } else if !open && *autofocused.read(cx) {
            autofocused.update(cx, |v, _| *v = false);
        }

        // A controlled `inputValue` wins over whatever the search field holds.
        let raw_query = match &self.input_value {
            Some(v) => v.clone(),
            None => self.state.read(cx).value().to_owned(),
        };
        let query = raw_query.to_lowercase();

        // The list starts unfiltered: v3's popover shows the whole collection
        // until something is typed into the search field. Filtering reads the
        // items' labels, never their keys.
        let custom = self.filter.clone();
        let matches: Vec<PickerItem> = self
            .items
            .iter()
            .filter(|it| match &custom {
                // A custom filter owns the whole decision, including what an
                // empty query means.
                Some(f) => f(it.label(), &raw_query),
                None => query.is_empty() || it.label().to_lowercase().contains(&query),
            })
            .take(self.max_items)
            .cloned()
            .collect();
        // The stored cursor is the focused item's key; the row it lands on is
        // wherever that key sits in the filtered collection now.
        let cursor_at = cursor
            .read(cx)
            .as_ref()
            .and_then(|k| matches.iter().position(|it| it.key() == k));

        // Forward typing while the popover is open puts the collection cursor
        // on its first enabled filtered row. react-aria 3.51.0 does this from
        // `useAutocomplete.onChange` only for forward input types, and clears
        // virtual focus for deletion, paste and history edits. GPUI exposes
        // neither a DOM input type nor one combined callback, so the actual
        // SearchField change and its bubbling unmodified character key mark
        // the edit together. A controlled prop update fires neither and cannot
        // masquerade as typing.
        if let Some(forward) = *query_edit.read(cx) {
            let next = if open && forward {
                matches
                    .iter()
                    .position(|item| !self.disabled_keys.contains(item.key()))
            } else {
                None
            };
            if cursor_at != next {
                let next_key = next.and_then(|i| matches.get(i)).map(|it| it.key().clone());
                cursor.update(cx, |v, cx| {
                    *v = next_key;
                    cx.notify();
                });
                if let Some(next) = next {
                    if self.row_height.is_some() {
                        list_scroll_now.scroll_to_item(next, gpui::ScrollStrategy::Center);
                    } else {
                        panel_scroll_now.scroll_to_item(next);
                    }
                }
            }
            query_edit.update(cx, |v, cx| {
                *v = None;
                cx.notify();
            });
        }
        if *plain_edit_key.read(cx) {
            plain_edit_key.update(cx, |v, _| *v = false);
        }

        // The theme tokens borrow `cx`, so they are copied out only after the
        // keyed-state updates above.
        let colors = cx.colors();
        let layout = cx.layout();

        let is_invalid = self.is_invalid || self.error_message.is_some();
        sync_form_state(
            &self.form_state,
            &self.selected_keys,
            self.is_disabled,
            is_invalid,
        );
        self.form_state.borrow_mut().focus = focus_handle.clone();
        let restore_own = selection_own.clone();
        let restore_state = self.form_state.clone();
        let restore_default =
            normalize_selection(self.default_value.clone().unwrap_or_default(), multiple);
        let restore_all = self.on_selection_change_all.clone();
        self.form_state.borrow_mut().restore = (restore_own.is_some() || restore_all.is_some())
            .then(|| {
                util::shared(move |window: &mut Window, cx: &mut App| {
                    restore_state.borrow_mut().value = form_selection_value(&restore_default);
                    if let Some(held) = &restore_own {
                        let set = restore_default.clone();
                        held.update(cx, |v, cx| {
                            *v = set;
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &restore_all {
                        cb(&restore_default, window, cx);
                    }
                }) as std::sync::Arc<dyn Fn(&mut Window, &mut App)>
            });
        let can_open = !self.is_disabled;
        // Whether the trigger's open acts are allowed at all: react-stately
        // 3.49.0's `useSelectState` guards `open`/`toggle` — *"Don't open if
        // the collection is empty"* — and v3's Autocomplete root is a RAC
        // `Select`, whose trigger calls `state.toggle()`. A collection with no
        // items therefore refuses every trigger open/toggle unless
        // `allowsEmptyCollection` lets the autocomplete function with no
        // items. This is the *unfiltered* collection: a query that prunes an
        // open popover to zero never reaches this gate.
        let toggle_allowed = self.allows_empty_collection || !self.items.is_empty();

        // --- the trigger ----------------------------------------------------
        // Whether the pointer went down on the trigger (or on the clear
        // button inside it). The panel's outside-press dismissal treats the
        // trigger as outside its own bounds, so a press on an *open* popover's
        // trigger would dismiss it on the mouse-down *and* toggle it back open
        // through the trigger's own click on the mouse-up -- one press, two
        // contradictory reports. The trigger's capture-phase handler runs
        // before the panel's `on_mouse_down_out` in the same dispatch, so the
        // dismissal can see it and leave the close to the trigger's click.
        let trigger_pressed = Rc::new(std::cell::Cell::new(false));
        // `.autocomplete__trigger` is `relative isolate inline-flex min-h-9
        // rounded-field border bg-field px-3 py-2 text-sm shadow-field`, plus
        // `pe-7` because the indicator sits inside it.
        let mut field = gpui::div()
            .id(el_name(format!("{base}-trigger")))
            .relative()
            .flex()
            .items_center()
            .gap(px(8.))
            .min_h(util::FIELD_HEIGHT)
            .px(px(12.))
            .pr(px(28.))
            .text_size(util::FIELD_TEXT);
        field = util::apply_field_chrome(field, self.variant, is_invalid, false, cx);
        // `.autocomplete__trigger:focus-visible` is `status-focused` -- the
        // offset ring, not a field's flush one, which is why the chrome above is
        // not told about the focus.
        if let Some(handle) = &focus_handle {
            field = util::ring_if_focused(field, handle, true, Vec::new(), window, cx);
        }
        if self.is_disabled {
            field = field.opacity(layout.disabled_opacity);
        } else {
            let hover_bg = match self.variant {
                FieldVariant::Primary => colors.field.hover(),
                FieldVariant::Secondary => colors.default.soft_hover(),
            };
            field = field.hover(move |s| s.bg(hover_bg)).cursor_pointer();
        }
        if self.full_width {
            field = field.w_full();
        } else {
            // v3's trigger is `inline-flex` and every documented example sizes
            // it from the outside (`<Autocomplete className="w-[256px]">`).
            // There is no `className` here, so the trigger keeps a floor of its
            // own rather than collapsing onto the placeholder -- the same choice
            // `ComboBox` makes.
            field = field.min_w(px(180.));
        }

        // --- `.autocomplete__value` -----------------------------------------
        let has_selection = !self.selected_keys.is_empty();
        // The trigger renders the selection in the selection set's own order —
        // pinned react-stately 3.49.0's `selectedKeys` is a JS `Set`, whose
        // iteration order is insertion order, so `selectedItems`, the render
        // props' keys and `selectedText` follow the pick (or owner) order, not
        // the collection's. Each key resolves to its item wherever that item
        // now sits; a key whose item is not in the collection (still loading)
        // renders nothing, which is what v3's `selectedItems` does too.
        let mut selected_items: Vec<SharedString> = Vec::new();
        let mut selected_indices: Vec<usize> = Vec::new();
        for key in &self.selected_keys {
            if let Some((index, item)) = self
                .items
                .iter()
                .enumerate()
                .find(|(_, it)| it.key() == key)
            {
                selected_items.push(item.label().clone());
                selected_indices.push(index);
            }
        }
        let selected_key_order = self.selected_keys.clone();
        // `selectedText` — v3 joins with locale-aware separators; without CLDR
        // data this is a comma and a space.
        let selected_text = selected_items
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        let is_placeholder = selected_items.is_empty();
        let placeholder = self
            .placeholder
            .clone()
            // v3's own default for this prop.
            .unwrap_or_else(|| SharedString::from("Select an item"));
        // `.autocomplete__value` is `flex-1 text-start text-sm`, and
        // `text-field-placeholder` while nothing is chosen.
        let default_children = gpui::div()
            .flex_1()
            .min_w_0()
            .truncate()
            .text_size(util::FIELD_TEXT)
            .text_color(if is_placeholder {
                colors.field.placeholder
            } else {
                colors.field.foreground
            })
            .child(if is_placeholder {
                placeholder.to_string()
            } else {
                selected_text.clone()
            })
            .into_any_element();
        let value_slot = match self.value_content.take() {
            Some(render) => gpui::div()
                .flex_1()
                .min_w_0()
                .child(render(util::SelectionValue {
                    selected_items: &selected_items,
                    selected_indices: &selected_indices,
                    selected_keys: Some(&selected_key_order),
                    selected_text: &selected_text,
                    is_placeholder,
                    default_children,
                }))
                .into_any_element(),
            None => default_children,
        };
        field = field.child(value_slot);

        // `.autocomplete__clear-button` — present whenever there is a mutable
        // selection to clear (`data-empty` hides it otherwise). v3 clears the
        // selection first and only then calls the optional `onClear` handler.
        if has_selection && !self.is_disabled && !self.is_read_only {
            let own = selection_own.clone();
            let selection_cb = self.on_selection_change_all.clone();
            let clear_cb = self.on_clear.clone();
            let clear_form_state = self.form_state.clone();
            let hover_bg = colors.default.soft_hover();
            field = field.child(
                gpui::div()
                    .id(el_name(format!("{base}-clear")))
                    .flex()
                    .items_center()
                    .justify_center()
                    // `.autocomplete__clear-button` is `h-6 w-6`
                    // and then `size-5`, so 20px, `rounded-xl` and `p-1`
                    // -- which leaves the glyph 12.
                    .size(px(20.))
                    .p(px(4.))
                    .rounded(util::small_radius(cx))
                    .cursor_pointer()
                    .hover(move |st| st.bg(hover_bg))
                    .child(
                        gpui::svg()
                            .size(px(12.))
                            .path(icons::CLOSE)
                            .text_color(colors.muted),
                    )
                    .on_click(move |_, window, cx| {
                        // The button sits *inside* the trigger, so gpui
                        // dispatches its click up to the trigger's own
                        // `on_click` too -- and clearing is not an open
                        // gesture (React Aria's trigger press is
                        // pointer-bound, so a bubbled DOM click is inert
                        // there).
                        cx.stop_propagation();
                        // Uncontrolled: drop our own selection too, or the
                        // button would clear nothing.
                        if let Some(held) = &own {
                            held.update(cx, |v, cx| {
                                v.clear();
                                cx.notify();
                            });
                            clear_form_state.borrow_mut().value =
                                crate::form::FormValue::Keys(Vec::new());
                        }
                        if let Some(cb) = &selection_cb {
                            cb(&[], window, cx);
                        }
                        if let Some(cb) = &clear_cb {
                            cb(window, cx);
                        }
                    }),
            );
        }

        // `.autocomplete__indicator` is `absolute inset-y-0 end-2 my-auto`, and
        // its glyph is `size-4`. gpui 0.2.2 cannot rotate a div, so the chevron
        // is swapped rather than turned -- which is what v3's `rotate-180` looks
        // like on a symmetric glyph.
        let trigger_indicator = match self.indicator.take() {
            Some(render) => render(open),
            None => gpui::svg()
                .size(util::FIELD_ICON)
                .path(if open {
                    icons::CHEVRON_UP
                } else {
                    icons::CHEVRON_DOWN
                })
                .text_color(colors.field.placeholder)
                .into_any_element(),
        };
        field = field.child(
            gpui::div()
                .absolute()
                .right(px(8.))
                .top_0()
                .bottom_0()
                .flex()
                .items_center()
                .justify_center()
                .text_color(colors.field.placeholder)
                .child(trigger_indicator),
        );

        // Clicking the trigger opens and closes the popover. The toggle is
        // the `useSelectState.toggle()` act: an empty collection without the
        // prop refuses it in *both* directions, and the refusal reports
        // nothing (the guard sits before `triggerState.toggle()`, so
        // `onOpenChange` never fires).
        if can_open {
            let own = open_own.clone();
            let cb = self.on_open_change.clone();
            let was_open = open;
            let pressed = trigger_pressed.clone();
            let may_toggle = toggle_allowed;
            field = field
                .capture_any_mouse_down(move |_, _, cx| {
                    pressed.set(true);
                    let pressed = pressed.clone();
                    cx.defer(move |_| pressed.set(false));
                })
                .when_some(focus_handle.as_ref(), |el, handle| el.track_focus(handle))
                .on_click(move |_, window, cx| {
                    if !may_toggle {
                        return;
                    }
                    if let Some(held) = &own {
                        held.update(cx, |v, cx| {
                            *v = !was_open;
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &cb {
                        cb(!was_open, window, cx);
                    }
                });
        }

        // --- the wrapper: `.autocomplete` is `flex flex-col gap-1` -----------
        let mut wrapper = gpui::div().flex().flex_col().gap(px(4.)).w_full();
        if let Some(label) = &self.label {
            wrapper = wrapper.child(
                crate::field::Label::new(label.clone())
                    .is_required(self.is_required)
                    .is_disabled(self.is_disabled)
                    .is_invalid(is_invalid),
            );
        }
        wrapper = wrapper.child(field);
        if is_invalid {
            if let Some(message) = &self.error_message {
                wrapper = wrapper.child(crate::field::ErrorMessage::new(message.clone()));
            }
        } else if let Some(desc) = &self.description {
            wrapper = wrapper.child(crate::field::Description::new(desc.clone()));
        }

        let mut root = gpui::div().relative().child(wrapper);
        root = if self.full_width {
            root.w_full()
        } else {
            root.max_w(px(320.))
        };
        // The blur scope spans this one root, so a focus move between the
        // trigger and the search field inside the panel stays inside it.
        root = root.track_focus(&blur_scope);

        // Arrows, Home, End and Enter walk the list while the search field has
        // the focus: the input keeps left and right for the caret, so the rest
        // bubbles up to here. Escape closes.
        if can_open {
            let stops: Vec<usize> = (0..matches.len())
                .filter(|i| {
                    matches
                        .get(*i)
                        .is_some_and(|item| !self.disabled_keys.contains(item.key()))
                })
                .collect();
            let held = cursor.clone();
            let key_query_edit = query_edit.clone();
            let key_plain_edit = plain_edit_key.clone();
            let wrap = self.should_focus_wrap;
            let virtual_rows = self.row_height.is_some();
            let page_row_height = self.row_height;
            let key_list_scroll = list_scroll_now.clone();
            let key_panel_scroll = panel_scroll_now.clone();
            let key_page_stops = stops.clone();
            let rows = matches.clone();
            let key_open_own = open_own.clone();
            let key_open_change = self.on_open_change.clone();
            let may_open = toggle_allowed;
            let on_change_all = self.on_selection_change_all.clone();
            let on_change_one = self.on_selection_change.clone();
            let key_selection_own = selection_own.clone();
            let key_form_state = self.form_state.clone();
            let selected_now = self.selected_keys.clone();
            let was_open = open;
            root = root.on_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                let modifiers = event.keystroke.modifiers;
                let mut chars = key.chars();
                let plain_insert = was_open
                    && (key == "space"
                        || matches!(
                            (chars.next(), chars.next()),
                            (Some(ch), None) if !ch.is_control()
                        ))
                    && !modifiers.control
                    && !modifiers.alt
                    && !modifiers.platform
                    && !modifiers.function;
                key_plain_edit.update(cx, |v, cx| {
                    if *v != plain_insert {
                        *v = plain_insert;
                        cx.notify();
                    }
                });
                if plain_insert {
                    key_query_edit.update(cx, |edit, cx| {
                        if edit.is_some() {
                            *edit = Some(true);
                            cx.notify();
                        }
                    });
                }
                if !was_open {
                    // Closed: Down and Up open it. Enter and Space are *not*
                    // handled here -- the trigger has a click listener and gpui
                    // fires those for a focused element, so answering them again
                    // would open and close the popover in one keystroke.
                    if matches!(key, "down" | "up") {
                        // The keyboard open is the same
                        // `useSelectState.open()` act: an empty collection
                        // without `allowsEmptyCollection` refuses it and
                        // reports nothing.
                        if !may_open {
                            return;
                        }
                        if let Some(held) = &key_open_own {
                            held.update(cx, |v, cx| {
                                *v = true;
                                cx.notify();
                            });
                        }
                        if let Some(cb) = &key_open_change {
                            cb(true, window, cx);
                        }
                    }
                    return;
                }
                // The focused search field owns inserted characters. In
                // particular, the shared list navigator treats Space as an
                // activation key, but Autocomplete must insert it into the
                // query rather than select the current virtual row.
                if plain_insert || key == "space" {
                    return;
                }
                // The held cursor is the focused item's key; resolve it to the
                // row it occupies in the filtered collection now.
                let from = held
                    .read(cx)
                    .as_ref()
                    .and_then(|k| rows.iter().position(|it| it.key() == k));
                // Pinned React Aria 3.51.0 binds PageUp/PageDown through the
                // listbox's `useSelectableCollection`, which the closed branch
                // above never reaches. Those handlers require
                // `manager.focusedKey != null` -- a mouse-opened, selection-less
                // Autocomplete has a null cursor and must answer nothing until
                // an arrow establishes one.
                //
                // With a cursor this popup pages by viewport, unlike the
                // Select/ComboBox/Dropdown popups: `autocomplete.css` styles
                // the composed `[data-slot="list-box"]` itself
                // `max-h-[320px] min-h-0 overflow-y-auto`, so the list element
                // is its own scroller and pinned `ListKeyboardDelegate` walks
                // enabled rows from the cursor until one crosses a
                // one-viewport boundary, taking the enabled end only when the
                // walk runs out. The default rows are laid out, so the
                // boundary reads real `ScrollHandle` rects (the plain ListBox
                // shape); a `row_height` list is uniform and pages by
                // whole-row steps across its fixed 320px viewport (the fixed
                // ListBox shape).
                let fixed_page_step = page_row_height.map(|row_height| {
                    ((f32::from(px(320.)) / f32::from(row_height)).ceil() as usize)
                        .saturating_sub(1)
                });
                let page_target = |from: usize| -> Option<usize> {
                    if let Some(step) = fixed_page_step {
                        let boundary = match key {
                            "pagedown" => (from + step).min(rows.len().saturating_sub(1)),
                            "pageup" => from.saturating_sub(step),
                            _ => return None,
                        };
                        return match key {
                            "pagedown" => key_page_stops
                                .iter()
                                .copied()
                                .find(|stop| *stop >= boundary)
                                .or_else(|| key_page_stops.last().copied()),
                            "pageup" => key_page_stops
                                .iter()
                                .rev()
                                .copied()
                                .find(|stop| *stop <= boundary)
                                .or_else(|| key_page_stops.first().copied()),
                            _ => None,
                        };
                    }
                    let current = key_panel_scroll.bounds_for_item(from)?;
                    let viewport_height = key_panel_scroll.bounds().size.height;
                    let target = match key {
                        "pagedown" => current.top() - current.size.height + viewport_height,
                        "pageup" => current.top() + current.size.height - viewport_height,
                        _ => return None,
                    };
                    match key {
                        "pagedown" => key_page_stops
                            .iter()
                            .copied()
                            .filter(|stop| *stop >= from)
                            .find(|stop| {
                                key_panel_scroll
                                    .bounds_for_item(*stop)
                                    .is_some_and(|bounds| bounds.top() >= target)
                            })
                            .or_else(|| key_page_stops.last().copied()),
                        "pageup" => key_page_stops
                            .iter()
                            .rev()
                            .copied()
                            .filter(|stop| *stop <= from)
                            .find(|stop| {
                                key_panel_scroll
                                    .bounds_for_item(*stop)
                                    .is_some_and(|bounds| bounds.top() <= target)
                            })
                            .or_else(|| key_page_stops.first().copied()),
                        _ => None,
                    }
                };
                let page_move = from.and_then(page_target);
                let page_move = page_move.filter(|next| Some(*next) != from);
                match page_move.map_or_else(
                    || crate::list_nav::resolve(&stops, from, key, wrap),
                    crate::list_nav::Move::To,
                ) {
                    crate::list_nav::Move::To(next) => {
                        let next_key = rows.get(next).map(|item| item.key().clone());
                        held.update(cx, |v, cx| {
                            *v = next_key;
                            cx.notify();
                        });
                        if virtual_rows {
                            key_list_scroll.scroll_to_item(next, gpui::ScrollStrategy::Center);
                        } else {
                            key_panel_scroll.scroll_to_item(next);
                        }
                    }
                    crate::list_nav::Move::Activate => {
                        let Some(item) = from.and_then(|i| rows.get(i)) else {
                            return;
                        };
                        let item_key = item.key().clone();
                        let mut next = selected_now.clone();
                        if multiple {
                            toggle_key(&mut next, &item_key);
                        } else {
                            next.clear();
                            next.push(item_key.clone());
                        }
                        if let Some(own) = &key_selection_own {
                            let set = next.clone();
                            own.update(cx, |v, cx| {
                                *v = set;
                                cx.notify();
                            });
                            key_form_state.borrow_mut().value = form_selection_value(&next);
                        }
                        if let Some(cb) = &on_change_one {
                            cb(&item_key, window, cx);
                        }
                        if let Some(cb) = &on_change_all {
                            cb(&next, window, cx);
                        }
                        // A single selection closes the popover, as v3's does;
                        // a multiple one stays open for the next pick.
                        if !multiple {
                            if let Some(own) = &key_open_own {
                                own.update(cx, |v, cx| {
                                    *v = false;
                                    cx.notify();
                                });
                            }
                            if let Some(cb) = &key_open_change {
                                cb(false, window, cx);
                            }
                            // The focus is *not* moved back to the trigger here.
                            // gpui activates a focused element on Enter, so
                            // focusing the trigger inside this very keystroke
                            // fires its click listener and the popover reopens --
                            // observed, not theorised.
                        }
                    }
                    crate::list_nav::Move::Ignore => {}
                }
            });
        }

        let escape_own = open_own.clone();
        let escape_cb = self.on_open_change.clone();
        let escape_focus = focus_handle.clone();
        root =
            util::dismiss_on_escape_with_token(root, dismissal_token.clone(), move |window, cx| {
                if let Some(held) = &escape_own {
                    held.update(cx, |v, cx| {
                        *v = false;
                        cx.notify();
                    });
                }
                if let Some(cb) = &escape_cb {
                    cb(false, window, cx);
                }
                if let Some(handle) = &escape_focus {
                    window.focus(handle);
                }
                util::DismissResult::Handled
            });

        // --- the popover ----------------------------------------------------
        // The popover's presence is the Select's open state and nothing else.
        // Filtering happens inside `Autocomplete.Filter`, which prunes only
        // the ListBox's rows. At zero this port draws the "No results found"
        // empty state used by v3's examples; `allowsEmptyCollection` is not a
        // close-on-filtered-empty flag. The panel carries its own
        // outside-press dismissal, so there is nothing to attach to the root
        // when it is unmounted.
        let show_panel = overlay_active;
        if show_panel {
            let panel = gpui::div()
                .w_full()
                .flex()
                .flex_col()
                // `.autocomplete__popover` is `p-0 pt-2`: the search field and
                // the list bring their own padding.
                .pt(px(8.))
                .bg(colors.overlay.background)
                .rounded(util::container_radius(cx))
                // v3 gives a floating panel no border: `.popover` and friends are
                // `bg-overlay shadow-overlay` and a radius, and dark mode's
                // inset hairline is what separates the panel from the page.
                .when_some(layout.overlay_hairline, |el, hairline| {
                    el.border(layout.border_width).border_color(hairline)
                })
                .shadow(layout.overlay_shadow.clone());

            // React Aria dismisses the popover on a press outside it; Escape is
            // read by the key handler above. A press that started on the
            // trigger (or its clear button) is not an outside press: the
            // trigger's own click owns the close, and the click only fires
            // because the down was not stolen as a dismissal.
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
                        cb(false, window, cx);
                    }
                    util::DismissResult::Handled
                },
            );

            // The search field: v3's `[data-slot="search-field"]` inside the
            // popover is `shrink-0 px-3 py-1`, and `variant="secondary"` so it
            // reads as part of the panel rather than as a second field.
            let query_before_edit = raw_query;
            let edit_query = query_edit;
            let edit_key = plain_edit_key;
            let input_change = self.on_input_change.clone();
            let search = SearchField::new(self.state.clone())
                .variant(FieldVariant::Secondary)
                .placeholder("Search...")
                .is_read_only(self.is_read_only)
                .on_change(move |text, window, cx| {
                    if text != query_before_edit {
                        let forward = *edit_key.read(cx);
                        edit_query.update(cx, |edit, cx| {
                            *edit = Some(forward);
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &input_change {
                        cb(text, window, cx);
                    }
                });
            panel = panel.child(
                gpui::div()
                    .flex_shrink_0()
                    .px(px(12.))
                    .py(px(4.))
                    .child(search),
            );

            // Everything a row reads, owned: `uniform_list`'s callback is
            // `'static` and runs again on every scroll, so it cannot borrow
            // `self` or the theme -- and one row builder for both paths is what
            // keeps a virtual list drawing the same row as a short one.
            let matches_len = matches.len();
            let rows = matches.clone();
            let sections = self.sections.clone();
            let row_disabled_keys = self.disabled_keys.clone();
            let row_selected_keys = self.selected_keys.clone();
            let indicator: Option<Rc<dyn Fn(bool) -> gpui::AnyElement>> =
                self.item_indicator.take().map(Rc::from);
            let on_change_all = self.on_selection_change_all.clone();
            let on_change_one = self.on_selection_change.clone();
            let row_selection_own = selection_own;
            let row_form_state = self.form_state.clone();
            let row_open_own = open_own;
            let row_open_change = self.on_open_change.clone();
            let row_trigger_focus = focus_handle;
            let base_row = format!("{base}-list");
            let row_muted = colors.muted;
            let row_fg = colors.foreground;
            let row_hover_bg = colors.default.soft();
            let row_focus = colors.focus;
            let row_accent = colors.accent.color;
            let row_disabled_opacity = layout.disabled_opacity;
            // v3's `EmptyState` inside the popover is `text-center text-sm
            // text-overlay-foreground/60`. Copied out here because the row
            // builder below takes `cx` mutably, which ends the theme borrow.
            let mut empty_fg = colors.overlay.foreground;
            empty_fg.a *= 0.6;
            let row_of = move |index: usize, fixed_h: Option<gpui::Pixels>, cx: &mut App| {
                let base = base_row.as_str();
                let item = &rows[index];
                // A section header rides above the row it introduces, so the two
                // are one element -- a virtual row is one slot tall.
                let mut head: Vec<gpui::AnyElement> = Vec::new();
                let done = |head: Vec<gpui::AnyElement>, row: gpui::AnyElement| {
                    gpui::div()
                        .flex()
                        .flex_col()
                        .when_some(fixed_h, |el, h| el.h(h).w_full())
                        .children(head)
                        .child(row)
                        .into_any_element()
                };
                // `ListBox.Section`'s `Header`, above the item it introduces.
                if let Some((_, label)) = sections.iter().find(|(at, _)| at == item.key()) {
                    head.push(
                        gpui::div()
                            .px(px(10.))
                            .pt(px(6.))
                            .pb(px(2.))
                            .text_size(px(12.))
                            .text_color(row_muted)
                            .child(label.to_string())
                            .into_any_element(),
                    );
                }
                // The row's element id comes from the item's key, so two items
                // that share a label never share an interactive row.
                let item_disabled = row_disabled_keys.contains(item.key());
                let item_interactive = !item_disabled && !overlay_exiting;
                let row_selected = row_selected_keys.contains(item.key());
                let mut row = gpui::div()
                    .id(el_name(format!("{base}-{}", item.key())))
                    .flex()
                    .items_center()
                    .justify_between()
                    .w_full()
                    // Every menu row in v3 is a `.list-box-item`: `min-h-9
                    // rounded-2xl py-1.5 gap-3` at `text-sm`, and the
                    // Autocomplete's popover restates the padding as `px-2.5`.
                    .min_h(util::FIELD_HEIGHT)
                    .rounded(util::soft_radius(cx))
                    .px(px(10.))
                    .py(px(6.))
                    .gap(px(12.))
                    .text_size(util::FIELD_TEXT)
                    .child(gpui::div().truncate().child(item.label().to_string()));

                if item_disabled {
                    row = row.opacity(row_disabled_opacity);
                } else if item_interactive {
                    row = row.cursor_pointer().hover(move |s| s.bg(row_hover_bg));
                }
                if row_selected {
                    row = row
                        .text_color(row_accent)
                        .font_weight(gpui::FontWeight::MEDIUM);
                } else {
                    row = row.text_color(row_fg);
                }
                // `status-focused` on the row the keyboard is on.
                if cursor_at == Some(index) {
                    row = row.border_2().border_color(row_focus);
                }

                // The chosen rows are ticked, unless `ListBox.ItemIndicator` is
                // drawn by the caller.
                match &indicator {
                    Some(render) => row = row.child(render(row_selected)),
                    None if row_selected => {
                        row = row.child(
                            gpui::svg()
                                .size(px(13.))
                                .path(icons::CHECK)
                                .text_color(row_accent),
                        );
                    }
                    None => {}
                }

                if item_interactive {
                    let value = item.key().clone();
                    let current = row_selected_keys.clone();
                    let own = row_selection_own.clone();
                    let row_form_state = row_form_state.clone();
                    let cb_all = on_change_all.clone();
                    let cb_one = on_change_one.clone();
                    let open_own = row_open_own.clone();
                    let open_cb = row_open_change.clone();
                    let trigger_focus = row_trigger_focus.clone();
                    row = row.on_click(move |_, window, cx| {
                        let mut next = current.clone();
                        if multiple {
                            toggle_key(&mut next, &value);
                        } else {
                            next.clear();
                            next.push(value.clone());
                        }
                        // Uncontrolled: keep the new set, or picking an item
                        // would do nothing.
                        if let Some(held) = &own {
                            let set = next.clone();
                            held.update(cx, |v, cx| {
                                *v = set;
                                cx.notify();
                            });
                            row_form_state.borrow_mut().value = form_selection_value(&next);
                        }
                        if let Some(cb) = &cb_one {
                            cb(&value, window, cx);
                        }
                        if let Some(cb) = &cb_all {
                            cb(&next, window, cx);
                        }
                        // A single selection closes the popover; a multiple one
                        // stays open for the next pick.
                        if !multiple {
                            if let Some(held) = &open_own {
                                held.update(cx, |v, cx| {
                                    *v = false;
                                    cx.notify();
                                });
                            }
                            if let Some(cb) = &open_cb {
                                cb(false, window, cx);
                            }
                            if let Some(handle) = &trigger_focus {
                                window.focus(handle);
                            }
                        }
                    });
                }

                done(head, row.into_any_element())
            };

            // The list: `[data-slot="list-box"]` inside the popover is
            // `max-h-[320px] p-1.5 overflow-y-auto`.
            match self.row_height {
                // Virtual: only the rows in view are built, which is what makes
                // a thousand options affordable.
                Some(row_height) => {
                    panel = panel.child(
                        gpui::uniform_list(
                            el_name(format!("{base}-rows")),
                            matches_len,
                            move |range, _window, cx| {
                                range
                                    .map(|i| row_of(i, Some(row_height), cx))
                                    .collect::<Vec<_>>()
                            },
                        )
                        .track_scroll(list_scroll_now)
                        .h(px(320.))
                        .w_full()
                        .p(px(6.)),
                    );
                }
                None => {
                    let mut list = gpui::div()
                        .id(el_name(format!("{base}-list-scroll")))
                        .flex()
                        .flex_col()
                        .w_full()
                        .p(px(6.))
                        .max_h(px(320.))
                        .overflow_y_scroll()
                        .track_scroll(&panel_scroll_now);
                    for index in 0..matches_len {
                        list = list.child(row_of(index, None, cx));
                    }
                    panel = panel.child(list);
                }
            }

            if matches.is_empty() {
                panel = panel.child(
                    gpui::div()
                        .w_full()
                        .px(px(12.))
                        .py(px(12.))
                        .text_center()
                        .text_size(util::FIELD_TEXT)
                        .text_color(empty_fg)
                        .child("No results found"),
                );
            }

            let zoom = crate::anim::ZoomBox::panel(px(6.), util::container_radius(cx));
            let panel = if overlay_phase == util::OverlayPhase::Exiting {
                crate::anim::exiting(
                    panel,
                    el_name(format!("{base}-panel-out")),
                    zoom,
                    crate::anim::Motion::FLUID_OUT,
                    cx,
                )
            } else {
                crate::anim::entering_zoom(
                    panel,
                    el_name(format!("{base}-panel")),
                    zoom,
                    crate::anim::Motion::FLUID_IN,
                    cx,
                )
            };
            root = root.child(util::floating(
                util::placed_field_panel(self.placement, px(6.)).child(panel),
            ));
        }

        root
    }
}
