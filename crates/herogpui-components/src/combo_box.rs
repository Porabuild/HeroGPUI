//! ComboBox — port of `@heroui/combo-box` (v3).
//!
//! A text input combined with a selectable list. Unlike
//! [`Select`](crate::select::Select) the value is typed, and unlike
//! [`Autocomplete`](crate::autocomplete::Autocomplete) the list can be opened
//! without typing and `allowsCustomValue` decides whether input outside the
//! collection is accepted.
//!
//! Pinned v3.2.4 / React Aria Components 1.20.0 keep a stable `Key` separate
//! from each item's `textValue`: `selectedKey` / `defaultValue` /
//! `disabledKeys`, the selection callbacks and the form value address items
//! by key, while filtering and the visible text use the label. Items are
//! therefore [`crate::PickerItem`]s; using a label as the key made duplicate
//! labels alias each other's selection, disabled state and row identity.
//! Pinned HeroUI's ComboBox composition adds only slots, trigger and popover
//! chrome on the RAC primitive, so it overrides none of this.
//!
//! The input text is the query and the selected item's label, never the key:
//! picking a row fills the field with that row's label, and the cursor and
//! row identities ride the key so duplicate labels stay distinct. A committed
//! custom value carries a null selected key — pinned react-stately 3.49.0's
//! `commitCustomValue` sets the value to `null` and keeps the typed text —
//! which the [`ComboBox::on_selection_change_all`] slice reports as an empty
//! selection, and only when a selection actually existed; the single-key
//! [`ComboBox::on_selection_change`] cannot spell `null` and stays silent there.
//!
//! `formValue` (pinned React Aria Components 1.20.0) defaults to `"key"`: a
//! named field submits the selected key(s). `allowsCustomValue` forces
//! `"text"`, submitting the typed text instead.

use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
    sync::Arc,
};

use gpui::{
    div, prelude::*, px, App, Entity, InteractiveElement, IntoElement, RenderOnce, SharedString,
    Styled, Window,
};
use herogpui_core::{FieldVariant, Placement, SelectionMode};
use herogpui_theme::ActiveTheme;

use crate::{
    icons,
    input::{Input, InputState},
    picker_item::PickerItem,
    selection::{normalize_selection, toggle_key},
    util,
};

/// When the suggestion list opens.
///
/// v3's table reads `"focus" | "input" | "manual"` with **`"focus"`** as the
/// default, so a v3 ComboBox shows its list as soon as the field is focused.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuTrigger {
    /// Open as soon as the field gains focus. v3's default.
    #[default]
    Focus,
    /// Open on input, and when the trigger button is pressed.
    Input,
    /// Open only when the trigger button is pressed.
    Manual,
}

/// `formValue` — what a `name`d ComboBox submits.
///
/// Pinned React Aria Components 1.20.0 defaults to key and forces text when
/// `allowsCustomValue` is set: an input that accepts values outside the
/// collection always submits what was typed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ComboBoxFormValue {
    /// The selected item key(s). The pinned default.
    #[default]
    Key,
    /// The typed input text.
    Text,
}

type OnSelectionChange = Arc<dyn Fn(&SharedString, &mut Window, &mut App) + 'static>;
type OnOpenChange = Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>;
type ComboBoxFormState = Rc<RefCell<crate::form::LiveFormFieldState>>;

thread_local! {
    static COMBO_BOX_FORM_STATES: RefCell<HashMap<u64, Weak<RefCell<crate::form::LiveFormFieldState>>>> =
        RefCell::new(HashMap::new());
}

fn combo_box_form_state(entity_id: u64) -> ComboBoxFormState {
    COMBO_BOX_FORM_STATES.with(|states| {
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

/// Whether the pinned `formValue` serialization submits the input text:
/// `formValue="text"` explicitly, or `allowsCustomValue`, which pinned React
/// Aria Components 1.20.0 turns into text regardless of the prop.
fn form_submits_text(form_value: Option<ComboBoxFormValue>, allows_custom_value: bool) -> bool {
    allows_custom_value || form_value == Some(ComboBoxFormValue::Text)
}

/// The one-shot that keeps `MenuTrigger::Focus` from reopening a panel the
/// user dismissed while the field still has the focus.
///
/// `can_open` is true while a fresh focus session may open the list; opening
/// consumes it. `was_open` remembers the flag from the last frame, so a panel
/// that closes while the field stays focused (Escape, a pick, the chevron, a
/// press outside) reads as a dismissal rather than as a new focus to answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FocusOpen {
    can_open: bool,
    was_open: bool,
}

/// Which suggestion the keyboard is on, held as the item's *key* so the
/// cursor stays on the same item when the query filters the list or the
/// caller reorders the collection. `hidden_query` retains the cursor across a
/// multiple-mode pick, which clears the query before the next frame can
/// re-derive the row: while the query equals the retained value the cursor
/// stays alive even when the capped or filtered list no longer renders its
/// item.
#[derive(Clone, Debug, PartialEq, Eq)]
struct ComboCursor {
    key: SharedString,
    hidden_query: Option<String>,
}

fn cursor_for(rows: &[PickerItem], index: usize, hidden_query: Option<String>) -> ComboCursor {
    ComboCursor {
        key: rows[index].key().clone(),
        hidden_query,
    }
}

fn cursor_position(rows: &[PickerItem], cursor: &ComboCursor) -> Option<usize> {
    rows.iter().position(|item| item.key() == &cursor.key)
}

/// The label an item key resolves to, or `None` when the key has no item in
/// the collection (still loading, or already removed).
fn label_of_key<'a>(items: &'a [PickerItem], key: &SharedString) -> Option<&'a SharedString> {
    items
        .iter()
        .find(|item| item.key() == key)
        .map(|item| item.label())
}

/// HeroUI ComboBox (controlled open state).
#[derive(IntoElement)]
pub struct ComboBox {
    state: Entity<InputState>,
    items: Vec<PickerItem>,
    /// `isOpen` — `None` leaves the component holding the flag, seeded from
    /// `defaultOpen`.
    is_open: Option<bool>,
    default_open: bool,
    placement: Placement,
    label: Option<SharedString>,
    placeholder: Option<SharedString>,
    description: Option<SharedString>,
    error_message: Option<SharedString>,
    variant: FieldVariant,
    menu_trigger: MenuTrigger,
    /// `defaultFilter` — decides whether an item matches the query. Receives
    /// the item's label; the key never reaches it.
    filter: Option<Arc<dyn Fn(&str, &str) -> bool + 'static>>,
    allows_custom_value: bool,
    max_items: usize,
    full_width: bool,
    is_disabled: bool,
    is_invalid: bool,
    is_required: bool,
    is_read_only: bool,
    /// `autoFocus` — take focus on the first render.
    auto_focus: bool,
    /// `ListLayout`'s `rowHeight`, which virtualizes the popover list.
    row_height: Option<gpui::Pixels>,
    /// `validate` — run by the component, not the caller.
    validate: Option<crate::validation::Validator<str>>,
    /// `validationBehavior` — carried on the inner field.
    validation_behavior: Option<crate::form::ValidationBehavior>,
    /// `allowsEmptyCollection` — keeps the panel up with no matches.
    allows_empty_collection: bool,
    /// `name` — the name this field submits under.
    name: Option<SharedString>,
    /// `formValue` — `None` is the pinned `"key"` default; `allowsCustomValue`
    /// forces text whatever this says.
    form_value: Option<ComboBoxFormValue>,
    /// `shouldFocusWrap` — whether the arrow keys wrap at the ends of the list.
    should_focus_wrap: bool,
    /// `ListBox.Section` — a heading above the item with this key.
    sections: Vec<(SharedString, SharedString)>,
    /// `ListBox.ItemIndicator` — draws the tick. The closure is handed whether
    /// the row is the selected one.
    indicator: Option<Box<dyn Fn(bool) -> gpui::AnyElement + 'static>>,
    disabled_keys: std::collections::HashSet<SharedString>,
    selection_mode: SelectionMode,
    /// The selection as ordered unique item keys — pinned react-stately
    /// 3.49.0's `selectedKeys` is a JS `Set`, which iterates in insertion
    /// order, so the callbacks, the form value and `ComboBox.Value` follow
    /// the order the keys were picked (or the owner listed) in.
    selected_keys: Vec<SharedString>,
    /// Whether the caller drives the selection. An unset `selected_keys` is not
    /// an empty controlled selection: without this flag every plain ComboBox
    /// would hand its own picks back to a set nobody owns, and
    /// `ComboBox.Value` would never see them.
    is_controlled: bool,
    /// Whether `selected_key` owns the controlled single key. Only then does
    /// the render sync the input text to the key's label — and only when the
    /// key changes, never on the owner's other re-renders.
    selected_key_sync: bool,
    /// `ComboBox.Value` — draws the chosen item under the field.
    value_content: Option<Box<dyn Fn(util::SelectionValue<'_>) -> gpui::AnyElement + 'static>>,
    /// `defaultValue` — set it to hand this component its own selection.
    default_value: Option<Vec<SharedString>>,
    /// `defaultInputValue` — seeds the text state on the first render only.
    default_input_value: Option<SharedString>,
    on_selection_change_all: Option<Arc<dyn Fn(&[SharedString], &mut Window, &mut App) + 'static>>,
    on_input_change: Option<Arc<dyn Fn(&str, &mut Window, &mut App) + 'static>>,
    on_selection_change: Option<OnSelectionChange>,
    on_open_change: Option<OnOpenChange>,
    form_state: ComboBoxFormState,
}

impl ComboBox {
    /// `selectionMode`
    pub fn selection_mode(mut self, mode: SelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }

    /// `defaultValue` — the uncontrolled initial selection, as item keys.
    ///
    /// Supplying it hands the component its own selection set, seeded once;
    /// [`Self::selected_keys`] is the controlled spelling. The listed order is
    /// the selection's order, exactly as the owner listed it.
    /// `defaultInputValue` — the uncontrolled initial text.
    ///
    /// Written into the state on the first render only; [`Self::input_value`]
    /// is the controlled spelling.
    pub fn default_input_value(mut self, text: impl Into<SharedString>) -> Self {
        self.default_input_value = Some(text.into());
        self
    }

    pub fn default_value(
        mut self,
        keys: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.default_value = Some(keys.into_iter().map(Into::into).collect());
        self
    }

    /// `selectedKeys` — the controlled selection, as item keys. The listed
    /// order is the owner's order and is preserved everywhere the selection
    /// is read.
    pub fn selected_keys(mut self, keys: impl IntoIterator<Item = SharedString>) -> Self {
        self.selected_keys = keys.into_iter().collect();
        self.is_controlled = true;
        self
    }

    /// Reports the whole selection, including the empty single selection a
    /// cleared input or a committed custom value stands for (pinned React
    /// Stately reports `null` there). The empty slice fires only when a
    /// selection actually existed to clear.
    pub fn on_selection_change_all(
        mut self,
        handler: impl Fn(&[SharedString], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change_all = Some(Arc::new(handler));
        self
    }

    /// `selectedKey` — the controlled single-selection key, or `null` spelled
    /// as the empty string. The selection rides the key; the input text is
    /// synced to the key's label only when that key actually changes, so an
    /// owner that passes the same key every render never clobbers the text
    /// being typed — pinned react-stately resets the input value when the
    /// selected key changes and leaves the input alone otherwise.
    pub fn selected_key(mut self, key: impl Into<String>, _cx: &mut App) -> Self {
        let key = SharedString::from(key.into());
        self.selected_keys = if key.is_empty() {
            Vec::new()
        } else {
            vec![key]
        };
        self.is_controlled = true;
        self.selected_key_sync = true;
        self
    }

    /// `value` — the v3 alias of [`ComboBox::input_value`].
    pub fn value(self, value: impl Into<String>, cx: &mut App) -> Self {
        self.input_value(value, cx)
    }

    /// `disabledKeys` — keys of the items that render but cannot be chosen.
    /// Disabled state is per key, so one of two same-label items can be
    /// disabled alone.
    pub fn disabled_keys(mut self, keys: impl IntoIterator<Item = SharedString>) -> Self {
        self.disabled_keys = keys.into_iter().collect();
        self
    }

    /// `autoFocus` — take focus on the first render.
    /// `ListLayout`'s `rowHeight` -- and what virtualizes the popover list.
    ///
    /// v3 wraps the list in `<Virtualizer layout={ListLayout}>` inside the
    /// popover; gpui's `uniform_list` builds only the rows in view, and it can do
    /// that because every row is this tall.
    pub fn row_height(mut self, h: impl Into<gpui::Pixels>) -> Self {
        self.row_height = Some(h.into());
        self
    }

    pub fn auto_focus(mut self, v: bool) -> Self {
        self.auto_focus = v;
        self
    }

    /// `validate` — returns the message to show, or `None` when the text is
    /// fine. The component runs it and surfaces the result.
    pub fn validate(mut self, f: impl Fn(&str) -> Option<SharedString> + 'static) -> Self {
        self.validate = Some(Arc::new(f));
        self
    }

    /// `validationBehavior` — `Allow` shows the message without blocking a
    /// form submission.
    pub fn validation_behavior(mut self, behavior: crate::form::ValidationBehavior) -> Self {
        self.validation_behavior = Some(behavior);
        self
    }

    /// `allowsEmptyCollection` — keeps the panel open when nothing matches.
    pub fn allows_empty_collection(mut self, v: bool) -> Self {
        self.allows_empty_collection = v;
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

    /// `ListBox.ItemIndicator` — draw the selected tick yourself.
    pub fn indicator(mut self, render: impl Fn(bool) -> gpui::AnyElement + 'static) -> Self {
        self.indicator = Some(Box::new(render));
        self
    }

    /// `ComboBox.Value` — draw the chosen item under the field.
    ///
    /// v3's `.combo-box__value` is an optional part (`text-sm
    /// text-field-foreground empty:hidden`), and the closure is handed the same
    /// render props the component passes down: `selectedItems` reaches
    /// `selected_items`, and `defaultChildren` is the row this port would draw.
    pub fn value_content(
        mut self,
        render: impl Fn(util::SelectionValue<'_>) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.value_content = Some(Box::new(render));
        self
    }

    /// `name` — the name this field submits under.
    pub fn name(mut self, name: impl Into<SharedString>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// `formValue` — whether a `name`d field submits the selected key(s) or
    /// the typed text. `allowsCustomValue` forces the text whatever this says,
    /// as pinned React Aria Components 1.20.0 does.
    pub fn form_value(mut self, form_value: ComboBoxFormValue) -> Self {
        self.form_value = Some(form_value);
        self
    }

    /// The `Form` field this control submits, when it has a `name`.
    ///
    /// v3 discovers a field through the DOM; gpui gives a child no way to
    /// reach its ancestor, so the control hands the pair over instead. The
    /// live value follows the pinned `formValue` serialization — the selected
    /// key(s) by default, the typed text under `allowsCustomValue` — and the
    /// input keeps the implicit-Enter submission of the text control it is.
    /// A disabled control stays registered and is omitted from FormData.
    pub fn form_field(&self) -> Option<crate::form::FormField> {
        let name = self.name.clone()?;
        Some(
            crate::form::FormField::live_text(name, self.form_state.clone(), self.state.clone())
                .is_required(self.is_required),
        )
    }

    pub fn is_read_only(mut self, v: bool) -> Self {
        self.is_read_only = v;
        self
    }

    /// `inputValue` — writes the typed text through to the bound state.
    pub fn input_value(self, value: impl Into<String>, cx: &mut App) -> Self {
        self.state.update(cx, |s, _| s.set_value(value));
        self
    }

    /// `onInputChange` — fires as the text changes, where
    /// [`ComboBox::on_selection_change`] fires only on a pick.
    pub fn on_input_change(
        mut self,
        handler: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_input_change = Some(Arc::new(handler));
        self
    }

    /// `onChange` — the v3 name for [`ComboBox::on_selection_change`].
    pub fn on_change(
        self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change(handler)
    }

    pub fn new(state: Entity<InputState>, items: Vec<PickerItem>) -> Self {
        let form_state = combo_box_form_state(state.entity_id().as_u64());
        Self {
            state,
            items,
            is_open: None,
            default_open: false,
            placement: Placement::BottomStart,
            label: None,
            placeholder: None,
            description: None,
            error_message: None,
            variant: FieldVariant::Primary,
            menu_trigger: MenuTrigger::Focus,
            filter: None,
            allows_custom_value: false,
            max_items: 8,
            full_width: false,
            is_disabled: false,
            is_invalid: false,
            is_required: false,
            is_read_only: false,
            auto_focus: false,
            row_height: None,
            validate: None,
            validation_behavior: None,
            allows_empty_collection: false,
            name: None,
            form_value: None,
            should_focus_wrap: false,
            sections: Vec::new(),
            indicator: None,
            disabled_keys: std::collections::HashSet::new(),
            selection_mode: SelectionMode::Single,
            selected_keys: Vec::new(),
            is_controlled: false,
            selected_key_sync: false,
            value_content: None,
            default_value: None,
            default_input_value: None,
            on_selection_change_all: None,
            on_input_change: None,
            on_selection_change: None,
            on_open_change: None,
            form_state,
        }
    }

    /// `placement` on `ComboBox.Popover`.
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

    pub fn label(mut self, text: impl Into<SharedString>) -> Self {
        self.label = Some(text.into());
        self
    }

    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = Some(text.into());
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

    pub fn menu_trigger(mut self, trigger: MenuTrigger) -> Self {
        self.menu_trigger = trigger;
        self
    }

    /// `defaultFilter` — replaces the default case-insensitive substring
    /// match.
    ///
    /// Called as `filter(item_label, input)`: v3 filters on the item's
    /// `textValue`, never on its key, and owns the whole decision —
    /// including what an empty query means.
    pub fn filter(mut self, f: impl Fn(&str, &str) -> bool + 'static) -> Self {
        self.filter = Some(Arc::new(f));
        self
    }

    /// `allowsCustomValue` — accept text that matches no item. A committed
    /// custom value keeps the typed text and carries a null selected key.
    pub fn allows_custom_value(mut self, v: bool) -> Self {
        self.allows_custom_value = v;
        self
    }

    pub fn max_items(mut self, n: usize) -> Self {
        self.max_items = n.max(1);
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

    pub fn is_invalid(mut self, v: bool) -> Self {
        self.is_invalid = v;
        self
    }

    pub fn is_required(mut self, v: bool) -> Self {
        self.is_required = v;
        self
    }

    /// Pick-only single-key convenience callback.
    ///
    /// Use [`Self::on_selection_change_all`] for v3's complete
    /// `onChange` domain, including multiple selection and the `null` a
    /// cleared input or a committed custom value reports.
    pub fn on_selection_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change = Some(Arc::new(handler));
        self
    }

    pub fn on_open_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_change = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for ComboBox {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `defaultInputValue` seeds the text once, before anything reads it.
        if let Some(text) = self.default_input_value.clone() {
            let state = self.state.clone();
            util::seed_once(
                window,
                cx,
                gpui::ElementId::Name(
                    format!("combobox-{}-default-text", self.state.entity_id().as_u64()).into(),
                ),
                move |cx| {
                    state.update(cx, |s, cx| {
                        s.set_value(text.to_string());
                        cx.notify();
                    });
                },
            );
        }

        // `defaultValue` opts into the component holding its own selection;
        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        // Both spellings normalize to the `Set` shape first: duplicates
        // collapse to their first insertion, and single mode keeps one key.
        // A controlled order is the owner's order — nothing is sorted.
        let multiple = self.selection_mode == SelectionMode::Multiple;
        self.selected_keys = normalize_selection(self.selected_keys.clone(), multiple);
        let default_selection =
            normalize_selection(self.default_value.clone().unwrap_or_default(), multiple);
        let (selection, selection_own) = util::controlled(
            window,
            cx,
            gpui::ElementId::Name(
                format!("combobox-{}-selection", self.state.entity_id().as_u64()).into(),
            ),
            self.is_controlled.then(|| self.selected_keys.clone()),
            default_selection.clone(),
        );
        self.selected_keys = selection;

        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        let (open_state, open_own) = util::controlled(
            window,
            cx,
            gpui::ElementId::Name(
                format!("combobox-{}-open", self.state.entity_id().as_u64()).into(),
            ),
            self.is_open,
            self.default_open,
        );
        let (overlay_phase, dismissal_token) = util::overlay_scope(
            window,
            cx,
            gpui::ElementId::Name(
                format!("combobox-{}-overlay", self.state.entity_id().as_u64()).into(),
            ),
            open_state,
            false,
        );
        let overlay_active = overlay_phase != util::OverlayPhase::Closed;

        // Owned copies: `input.render` below needs `cx` mutably.
        let colors = cx.colors().clone();
        let layout = cx.layout().clone();
        let container_radius = util::container_radius(cx);
        let entity_id = self.state.entity_id().as_u64();
        let close_open = util::shared({
            let own = open_own.clone();
            let callback = self.on_open_change.clone();
            move |window: &mut Window, cx: &mut App| {
                let is_open = own.as_ref().map_or(open_state, |held| *held.read(cx));
                if !is_open {
                    return;
                }
                if let Some(held) = &own {
                    held.update(cx, |value, cx| {
                        *value = false;
                        cx.notify();
                    });
                }
                if let Some(callback) = &callback {
                    callback(false, window, cx);
                }
            }
        });
        let raw_query = self.state.read(cx).value().to_owned();
        let query = raw_query.to_lowercase();
        let is_invalid = self.is_invalid || self.error_message.is_some();
        let focus_handle = self.state.read(cx).focus_handle.clone();

        // The live form value follows the pinned `formValue` serialization:
        // the selected key(s) by default, the typed text when
        // `allowsCustomValue` forces text mode. Success and validity are the
        // inner input's own mirrors (`live_text` reads them there), so only
        // the value and the focus handle — the handle a blocked submit must
        // reach — live in this shared state.
        let form_text = form_submits_text(self.form_value, self.allows_custom_value);
        {
            let mut form = self.form_state.borrow_mut();
            form.value = if form_text {
                crate::form::FormValue::Text(SharedString::from(raw_query.clone()))
            } else {
                crate::form::FormValue::Keys(self.selected_keys.clone())
            };
            form.focus = Some(focus_handle.clone());
        }

        // `selected_key`'s controlled sync: the owner hands the same key to
        // the builder every render, so the label may move into the input only
        // when that key actually changes — writing it every frame would
        // clobber the text being typed. The empty string is v3's `null` and
        // clears the input; a key with no item resolves to no label.
        if self.selected_key_sync {
            let applied_key = window.use_keyed_state(
                gpui::ElementId::Name(format!("combobox-{entity_id}-applied-key").into()),
                cx,
                |_, _| None::<SharedString>,
            );
            let owned_key = self.selected_keys.first().cloned().unwrap_or_default();
            let first_apply = applied_key.read(cx).is_none();
            if applied_key.read(cx).clone() != Some(owned_key.clone()) {
                // Pinned `getDefaultInputValue` derives the text from the
                // selected key only when no `defaultInputValue` was given, so
                // the first application must leave the seeded default text in
                // place; later key changes still move their labels in.
                if !(first_apply && self.default_input_value.is_some()) {
                    let label = label_of_key(&self.items, &owned_key)
                        .cloned()
                        .unwrap_or_default();
                    self.state.update(cx, |state, cx| {
                        state.set_value(label.to_string());
                        cx.notify();
                    });
                }
                applied_key.update(cx, |v, _| *v = Some(owned_key));
            }
        }

        // A reset restores the default selection and the label it resolves
        // to, the way pinned react-stately resets `selectedKey` and lets
        // `resetInputValue` re-derive the text — reporting the restored text
        // through `onInputChange` when it actually changed, as the pinned
        // controlled input state does.
        let restore_own = selection_own.clone();
        let restore_state = self.form_state.clone();
        let restore_input = self.state.clone();
        let restore_items = self.items.clone();
        let restore_default = default_selection;
        let restore_input_change = self.on_input_change.clone();
        let restore_all = self.on_selection_change_all.clone();
        self.form_state.borrow_mut().restore = (restore_own.is_some() || restore_all.is_some())
            .then(|| {
                util::shared(move |window: &mut Window, cx: &mut App| {
                    let default_text = restore_default
                        .first()
                        .and_then(|key| label_of_key(&restore_items, key))
                        .cloned()
                        .unwrap_or_default();
                    restore_state.borrow_mut().value =
                        crate::form::FormValue::Keys(restore_default.clone());
                    let input_changed = restore_input.read(cx).value() != default_text.as_ref();
                    restore_input.update(cx, |state, cx| {
                        state.set_value(default_text.to_string());
                        cx.notify();
                    });
                    if input_changed {
                        if let Some(cb) = &restore_input_change {
                            cb(&default_text, window, cx);
                        }
                    }
                    if let Some(held) = &restore_own {
                        let keys = restore_default.clone();
                        held.update(cx, |v, cx| {
                            *v = keys;
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &restore_all {
                        cb(&restore_default, window, cx);
                    }
                }) as Arc<dyn Fn(&mut Window, &mut App)>
            });

        let show_all_items = window.use_keyed_state(
            gpui::ElementId::Name(format!("combobox-{entity_id}-show-all-items").into()),
            cx,
            |_, _| false,
        );
        let last_query = window.use_keyed_state(
            gpui::ElementId::Name(format!("combobox-{entity_id}-last-query").into()),
            cx,
            |_, _| raw_query.clone(),
        );
        if *last_query.read(cx) != raw_query {
            last_query.update(cx, |value, _| value.clone_from(&raw_query));
            show_all_items.update(cx, |value, _| *value = false);
        }
        let display_full_collection =
            *show_all_items.read(cx) || (self.menu_trigger == MenuTrigger::Manual && !open_state);

        // Focus and manual-action opens show the full collection until the
        // next edit. A custom `defaultFilter` owns the filtering decision,
        // including an empty query. Filtering reads the items' labels, never
        // their keys.
        let custom = self.filter.clone();
        let matches: Vec<PickerItem> = match &custom {
            _ if display_full_collection => {
                self.items.iter().take(self.max_items).cloned().collect()
            }
            Some(f) => self
                .items
                .iter()
                .filter(|item| f(item.label(), &raw_query))
                .take(self.max_items)
                .cloned()
                .collect(),
            _ if query.is_empty() => self.items.iter().take(self.max_items).cloned().collect(),
            _ => self
                .items
                .iter()
                .filter(|item| item.label().to_lowercase().contains(&query))
                .take(self.max_items)
                .cloned()
                .collect(),
        };

        // `MenuTrigger::Focus` opens the list when the field takes focus. The
        // check reads the focus handle every frame; the keyed state is the
        // one-shot that stops the panel from reopening behind a dismissal
        // (Escape, a pick, the chevron, a press outside) while the field keeps
        // the focus -- the frame after closing would otherwise re-answer the
        // still-held focus. The focus leaving resets it, so the *next* focus
        // session opens again.
        let focus_open =
            if self.menu_trigger == MenuTrigger::Focus && !self.is_disabled && !self.is_read_only {
                Some(window.use_keyed_state(
                    gpui::ElementId::Name(format!("combobox-{entity_id}-focus-open").into()),
                    cx,
                    |_, _| FocusOpen {
                        can_open: true,
                        was_open: false,
                    },
                ))
            } else {
                None
            };
        if let Some(focus_open) = &focus_open {
            let focused = self.state.read(cx).focus_handle.is_focused(window);
            let mut held = *focus_open.read(cx);
            let mut now_open = open_state;
            if focused && !open_state && held.was_open {
                // The list closed while the field still has the focus: the
                // user dismissed it, so it must not come back.
                held.can_open = false;
            } else if focused && !open_state && held.can_open {
                // A fresh focus session: the field taking focus is the
                // gesture, when there is something to show -- the panel's own
                // gate of a non-empty result or an allowed empty state.
                let can_show = !self.items.is_empty() || self.allows_empty_collection;
                if can_show {
                    show_all_items.update(cx, |v, _| *v = true);
                    // Opening from focus writes both halves, the way the
                    // chevron's handler does: the uncontrolled flag, and the
                    // report to a controlled caller.
                    if let Some(open) = &open_own {
                        open.update(cx, |v, cx| {
                            *v = true;
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &self.on_open_change {
                        cb(true, window, cx);
                    }
                    held.can_open = false;
                    now_open = true;
                }
            }
            if !focused {
                // The focus left; the next focus session may open again.
                held.can_open = true;
            }
            held.was_open = now_open;
            focus_open.update(cx, |v, _| *v = held);
        }

        let on_open_change = self.on_open_change.clone();
        let is_open = open_state;
        // Whether the pointer went down inside the input-plus-panel subtree.
        // The panel's outside-press listener reads only the panel bounds, so
        // the input is geometrically outside it even though both belong to the
        // same ComboBox. Capture the whole subtree for one dispatch: input
        // presses keep the list open, and the chevron or a row owns its click.
        let inside_pressed = Rc::new(std::cell::Cell::new(false));
        // `.combo-box__trigger` is `border-none bg-transparent
        // text-field-placeholder`, and its hover recolours the text to
        // `text-field-foreground`, filling nothing. `svg()` paints from its own
        // style rather than the inherited text color, so the hover refinement
        // goes on the glyph as well as the trigger that wraps it.
        let trigger_hover_fg = colors.field.foreground;
        let mut trigger = div()
            .id(gpui::ElementId::Name(
                format!("combobox-{entity_id}-trigger").into(),
            ))
            .flex()
            .items_center()
            .justify_center()
            .flex_shrink_0()
            .size(px(20.))
            .rounded(px(6.))
            .text_color(colors.muted)
            .child(
                gpui::svg()
                    .size(util::FIELD_ICON)
                    .path(icons::CHEVRON_DOWN)
                    .text_color(colors.muted)
                    .when(!self.is_disabled && !self.is_read_only, |glyph| {
                        glyph.hover(move |st| st.text_color(trigger_hover_fg))
                    }),
            );
        if !self.is_disabled && !self.is_read_only {
            trigger = trigger
                .cursor_pointer()
                .hover(move |s| s.text_color(trigger_hover_fg));
            if on_open_change.is_some() || open_own.is_some() {
                let own = open_own.clone();
                let focus_open = focus_open.clone();
                let show_all_items = show_all_items.clone();
                let close = close_open.clone();
                trigger = trigger
                    // The chevron is a separate React Aria button. Focus the
                    // input on press start, but do not let its down bubble into
                    // the input row and also trigger the default focus-open path.
                    .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                        if let Some(focus_open) = &focus_open {
                            focus_open.update(cx, |v, _| v.can_open = false);
                        }
                        window.focus(&focus_handle);
                        cx.stop_propagation();
                    })
                    .on_click(move |_, window, cx| {
                        if is_open {
                            close(window, cx);
                            return;
                        }
                        show_all_items.update(cx, |v, _| *v = true);
                        if let Some(held) = &own {
                            held.update(cx, |v, cx| {
                                *v = true;
                                cx.notify();
                            });
                        }
                        if let Some(cb) = &on_open_change {
                            cb(true, window, cx);
                        }
                    });
            }
        }

        // Which suggestion the keyboard is on. Input edits clear it, matching
        // React Stately's focused-key reset before the filtered list changes.
        let cursor = window.use_keyed_state(
            gpui::ElementId::Name(format!("combobox-{entity_id}-cursor").into()),
            cx,
            |_, _| None::<ComboCursor>,
        );
        let cursor_on_change = cursor.clone();
        let validate = self.validate.clone();
        let mut input = Input::new(self.state.clone())
            .variant(self.variant)
            .is_disabled(self.is_disabled)
            .is_invalid(is_invalid)
            .is_required(self.is_required)
            .is_read_only(self.is_read_only)
            .auto_focus(self.auto_focus)
            .when_some(self.validation_behavior, |i, b| i.validation_behavior(b))
            .when_some(validate, |i, f| i.validate(move |v| f(v)))
            .end_content(trigger);
        // Edits open Focus and Input triggers only when there is something to
        // show, and close an already-open filtered collection when it empties.
        // Manual stays closed until its trigger opens it, then edits switch it
        // from the full collection to filtered rows.
        let input_change = self.on_input_change.clone();
        let open_own_on_change = open_own.clone();
        let open_change_cb = self.on_open_change.clone();
        let focus_open_on_change = focus_open.clone();
        let show_all_items_on_change = show_all_items.clone();
        let items_on_change = self.items.clone();
        let filter_on_change = self.filter.clone();
        let menu_trigger_on_change = self.menu_trigger;
        let allows_empty_collection = self.allows_empty_collection;
        let was_open = open_state;
        let selection_own_on_change = selection_own.clone();
        let selected_on_change = self.selected_keys.clone();
        let selection_mode_on_change = self.selection_mode;
        let selection_change_on_input = self.on_selection_change_all.clone();
        let input_value_on_change = raw_query.clone();
        let close_on_empty = close_open.clone();
        input = input.on_change(move |text, window, cx| {
            if let Some(cb) = &input_change {
                cb(text, window, cx);
            }
            cursor_on_change.update(cx, |v, _| *v = None);

            if selection_mode_on_change == SelectionMode::Single
                && !input_value_on_change.is_empty()
                && text.is_empty()
                && !selected_on_change.is_empty()
            {
                if let Some(held) = &selection_own_on_change {
                    held.update(cx, |value, cx| {
                        value.clear();
                        cx.notify();
                    });
                }
                if let Some(cb) = &selection_change_on_input {
                    cb(&[], window, cx);
                }
            }

            let query = text.to_lowercase();
            let has_matches = match &filter_on_change {
                Some(f) => items_on_change.iter().any(|item| f(item.label(), text)),
                None if query.is_empty() => !items_on_change.is_empty(),
                None => items_on_change
                    .iter()
                    .any(|item| item.label().to_lowercase().contains(&query)),
            };
            let can_show = has_matches || allows_empty_collection;
            let is_open = open_own_on_change
                .as_ref()
                .map_or(was_open, |held| *held.read(cx));

            if !is_open && menu_trigger_on_change != MenuTrigger::Manual && can_show {
                if let Some(focus_open) = &focus_open_on_change {
                    focus_open.update(cx, |v, _| v.can_open = false);
                }
                if let Some(held) = &open_own_on_change {
                    held.update(cx, |v, cx| {
                        *v = true;
                        cx.notify();
                    });
                }
                if let Some(cb) = &open_change_cb {
                    cb(true, window, cx);
                }
            } else if is_open && !can_show {
                close_on_empty(window, cx);
            }

            show_all_items_on_change.update(cx, |v, cx| {
                *v = false;
                cx.notify();
            });
            if menu_trigger_on_change == MenuTrigger::Manual {
                cx.refresh_windows();
            }
        });

        if self.full_width {
            input = input.full_width();
        }
        if let Some(label) = self.label.clone() {
            input = input.label(label);
        }
        if let Some(placeholder) = self.placeholder.clone() {
            input = input.placeholder(placeholder);
        }
        if is_invalid {
            if let Some(message) = self.error_message.clone() {
                input = input.error_message(message);
            }
        } else if let Some(description) = self.description.clone() {
            input = input.description(description);
        }

        let stale_cursor = cursor.read(cx).as_ref().is_some_and(|focused| {
            let visible = cursor_position(&matches, focused).is_some();
            let retained_hidden = focused.hidden_query.as_deref() == Some(raw_query.as_str())
                && self.items.iter().any(|item| item.key() == &focused.key);
            self.disabled_keys.contains(&focused.key) || (!visible && !retained_hidden)
        });
        if stale_cursor {
            cursor.update(cx, |value, _| *value = None);
        }
        let cursor_at = cursor
            .read(cx)
            .as_ref()
            .and_then(|focused| cursor_position(&matches, focused));
        // React Aria keeps the focused row in view; the panel scrolls and the
        // virtual list scrolls itself. `use_keyed_state` takes `cx` mutably.
        let list_scroll = window.use_keyed_state(
            gpui::ElementId::Name(format!("combobox-{entity_id}-list-scroll").into()),
            cx,
            |_, _| gpui::UniformListScrollHandle::new(),
        );
        let panel_scroll = window.use_keyed_state(
            gpui::ElementId::Name(format!("combobox-{entity_id}-panel-scroll").into()),
            cx,
            |_, _| gpui::ScrollHandle::new(),
        );
        let list_scroll_now = list_scroll.read(cx).clone();
        let panel_scroll_now = panel_scroll.read(cx).clone();

        // ComboBox commits its text/selection when focus leaves even while the
        // list is closed. This is `useComboBoxState.setFocused(false)`, not the
        // generic popover close used by Select-family controls.
        let commit_value = util::shared({
            let state = self.state.clone();
            let selection_own = selection_own.clone();
            let selected = self.selected_keys.clone();
            let items = self.items.clone();
            let selection_change = self.on_selection_change_all.clone();
            let input_change = self.on_input_change.clone();
            let allows_custom = self.allows_custom_value;
            move |window: &mut Window, cx: &mut App| {
                let current = state.read(cx).value().to_owned();
                let selected_text = selected
                    .first()
                    .and_then(|key| label_of_key(&items, key))
                    .cloned()
                    .unwrap_or_default();

                if allows_custom {
                    // Pinned react-stately's commit path: text that no longer
                    // matches the selected item's label is a custom value,
                    // which carries a null selected key.
                    if !multiple && current != selected_text && !selected.is_empty() {
                        if let Some(held) = &selection_own {
                            held.update(cx, |value, cx| {
                                value.clear();
                                cx.notify();
                            });
                        }
                        if let Some(callback) = &selection_change {
                            callback(&[], window, cx);
                        }
                    }
                    return;
                }

                let committed = if multiple {
                    String::new()
                } else {
                    selected_text.to_string()
                };
                if current != committed {
                    state.update(cx, |state, cx| {
                        state.set_value(committed.clone());
                        cx.notify();
                    });
                    if let Some(callback) = &input_change {
                        callback(&committed, window, cx);
                    }
                }
            }
        });
        let blur_close = close_open.clone();
        let blur_commit = commit_value.clone();
        let blur_scope = util::on_focus_leave(
            window,
            cx,
            &format!("combobox-{entity_id}"),
            !self.is_disabled,
            move |window, cx| {
                blur_commit(window, cx);
                blur_close(window, cx);
            },
        );
        let blur_focus = blur_scope.focus_handle();

        // `.combo-box__input-group` is the field itself (`relative
        // inline-flex items-center`), and it is the panel's positioning
        // context: `.combo-box__value` sits below the field inside the root,
        // so a value row that appears under it must not push the popover down.
        let inside_pressed_for_group = inside_pressed.clone();
        let mut input_group = div()
            .relative()
            .capture_any_mouse_down(move |_, _, cx| {
                inside_pressed_for_group.set(true);
                let pressed = inside_pressed_for_group.clone();
                cx.defer(move |_| pressed.set(false));
            })
            .child(input.render(window, cx));
        let mut root = div()
            // Without a placeholder the inner input has no intrinsic width and
            // the trigger collapses to just its chevron, which is unclickable.
            .when(!self.full_width, |e| e.min_w(px(180.)))
            .flex()
            .flex_col()
            .gap(px(4.));

        // `ComboBox.Value` — `.combo-box__value` is `text-sm
        // text-field-foreground empty:hidden`, so it shows only once something
        // is chosen. The selection is read in its own order — pinned
        // react-stately 3.49.0's `selectedKeys` is a JS `Set`, which iterates
        // in insertion order — and each key resolves to its item wherever that
        // item now sits; a key with no item renders nothing.
        let mut value_content = None;
        if let Some(render) = self.value_content.take() {
            let mut items: Vec<SharedString> = Vec::new();
            let mut indices: Vec<usize> = Vec::new();
            for key in &self.selected_keys {
                if let Some((index, item)) = self
                    .items
                    .iter()
                    .enumerate()
                    .find(|(_, it)| it.key() == key)
                {
                    items.push(item.label().clone());
                    indices.push(index);
                }
            }
            let text = items
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            let default_children = div()
                .text_size(util::FIELD_TEXT)
                .text_color(colors.field.foreground)
                .child(text.clone())
                .into_any_element();
            value_content = Some(render(util::SelectionValue {
                selected_items: &items,
                selected_indices: &indices,
                selected_keys: Some(&self.selected_keys),
                selected_text: &text,
                is_placeholder: items.is_empty(),
                default_children,
            }));
        }

        // `allowsEmptyCollection` keeps the panel up with no matches. Without
        // it an empty result closes the list; `allowsCustomValue` only changes
        // what Enter commits while the list is closed.
        // Up, down, Home, End and Enter walk the suggestions; the inner input
        // keeps left and right for the caret.
        if !self.is_disabled && !self.is_read_only {
            let key_rows = if open_state || (self.allows_custom_value && !raw_query.is_empty()) {
                matches.clone()
            } else {
                self.items.iter().take(self.max_items).cloned().collect()
            };
            let stops: Vec<usize> = (0..key_rows.len())
                .filter(|i| {
                    key_rows
                        .get(*i)
                        .is_some_and(|item| !self.disabled_keys.contains(item.key()))
                })
                .collect();
            let held = cursor.clone();
            let wrap = self.should_focus_wrap;
            let virtual_rows = self.row_height.is_some();
            let key_list_scroll = list_scroll_now.clone();
            let key_panel_scroll = panel_scroll_now.clone();
            let rows = key_rows;
            let state = self.state.clone();
            let allows_custom_value = self.allows_custom_value;
            let was_open = open_state;
            let show_all_items = show_all_items.clone();
            let on_selection_change = self.on_selection_change.clone();
            let on_selection_change_all = self.on_selection_change_all.clone();
            let on_input_change = self.on_input_change.clone();
            let selected_now = self.selected_keys.clone();
            let key_query = raw_query.clone();
            let key_items = self.items.clone();
            let key_filter = self.filter.clone();
            let key_max_items = self.max_items;
            let key_disabled = self.disabled_keys.clone();
            let key_menu_trigger = self.menu_trigger;
            let open_own_keys = open_own.clone();
            let on_open_change = self.on_open_change.clone();
            let key_selection_own = selection_own.clone();
            let key_close = close_open.clone();
            let key_blur = blur_scope.clone();
            let key_multiple = multiple;
            root = root.on_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                let is_open = open_own_keys
                    .as_ref()
                    .map_or(was_open, |held| *held.read(cx));
                let stale_cursor = held.read(cx).as_ref().is_some_and(|focused| {
                    let current_query = state.read(cx).value().to_owned();
                    let lowered = current_query.to_lowercase();
                    let display_full = current_query == key_query && *show_all_items.read(cx)
                        || (key_menu_trigger == MenuTrigger::Manual && !is_open);
                    let current_matches: Vec<PickerItem> = match &key_filter {
                        _ if display_full => {
                            key_items.iter().take(key_max_items).cloned().collect()
                        }
                        Some(filter) => key_items
                            .iter()
                            .filter(|item| filter(item.label(), &current_query))
                            .take(key_max_items)
                            .cloned()
                            .collect(),
                        None if lowered.is_empty() => {
                            key_items.iter().take(key_max_items).cloned().collect()
                        }
                        None => key_items
                            .iter()
                            .filter(|item| item.label().to_lowercase().contains(&lowered))
                            .take(key_max_items)
                            .cloned()
                            .collect(),
                    };
                    let current_rows =
                        if is_open || (allows_custom_value && !current_query.is_empty()) {
                            current_matches
                        } else {
                            key_items.iter().take(key_max_items).cloned().collect()
                        };
                    let visible = cursor_position(&current_rows, focused).is_some();
                    let retained_hidden = focused.hidden_query.as_deref()
                        == Some(current_query.as_str())
                        && key_items.iter().any(|item| item.key() == &focused.key);
                    key_disabled.contains(&focused.key) || (!visible && !retained_hidden)
                });
                if stale_cursor {
                    held.update(cx, |value, _| *value = None);
                }
                // `allowsCustomValue` is the promise behind the drawn hint
                // "Press Enter to use this value". A no-match query has no
                // cursor row at all (an empty stop list makes `resolve` report
                // Ignore, so the Activate arm never runs). Pinned
                // react-stately's `commitCustomValue` keeps the typed text and
                // sets the value to null: the selection clears, and the slice
                // callback reports it only when a selection actually existed.
                // Text that still matches the selected item's label is not a
                // custom value at all — pinned `commitValue` re-runs
                // `commitSelection` there, so the selection stands and the
                // callbacks stay silent. The existing single-value commit
                // remains single-mode only; React Aria keeps multiple-mode
                // custom input independent from the selected items.
                if allows_custom_value
                    && key == "enter"
                    && !state.read(cx).value().is_empty()
                    && held.read(cx).as_ref().is_none_or(|focused| {
                        stale_cursor || cursor_position(&rows, focused).is_none()
                    })
                {
                    let selected_label = selected_now
                        .first()
                        .and_then(|item| label_of_key(&key_items, item))
                        .cloned()
                        .unwrap_or_default();
                    if !key_multiple && selected_label != state.read(cx).value() {
                        let had_selection = !selected_now.is_empty();
                        if let Some(held) = &key_selection_own {
                            held.update(cx, |v, cx| {
                                v.clear();
                                cx.notify();
                            });
                        }
                        if had_selection {
                            if let Some(cb) = &on_selection_change_all {
                                cb(&[], window, cx);
                            }
                        }
                    }
                    held.update(cx, |v, _| *v = None);
                    key_close(window, cx);
                    // Pinned React Aria 3.51.0's `Enter` shortcut prevents
                    // the default only while the menu is open
                    // (`shouldPreventDefault = state.isOpen`): a commit on a
                    // closed field — custom value or not — leaves Enter to
                    // bubble into an enclosing form's implicit submission.
                    if is_open {
                        cx.stop_propagation();
                    }
                    return;
                }
                if key == "enter" && (stale_cursor || held.read(cx).is_none()) {
                    let reset_value = if key_multiple {
                        String::new()
                    } else {
                        selected_now
                            .first()
                            .and_then(|item| label_of_key(&key_items, item))
                            .cloned()
                            .unwrap_or_default()
                            .to_string()
                    };
                    let input_changed = state.read(cx).value() != reset_value;
                    state.update(cx, |value, cx| {
                        value.set_value(reset_value.clone());
                        cx.notify();
                    });
                    if input_changed {
                        if let Some(cb) = &on_input_change {
                            cb(&reset_value, window, cx);
                        }
                    }
                    key_close(window, cx);
                    // Only an Enter the list acted on is kept from an
                    // enclosing form: an open list reverted, or a typed
                    // query discarded. A closed field with nothing to revert
                    // answers Enter with nothing here, and the keystroke may
                    // still bubble into the form's implicit submission.
                    if is_open || stale_cursor || input_changed {
                        cx.stop_propagation();
                    }
                    return;
                }
                let from = held
                    .read(cx)
                    .as_ref()
                    .and_then(|focused| cursor_position(&rows, focused));
                let nav_key = if key == "tab" && is_open && held.read(cx).is_some() {
                    "enter"
                } else {
                    key
                };
                // Pinned React Aria 3.51.0 binds PageUp/PageDown through the
                // listbox's `useSelectableCollection`, which a closed field
                // never runs: the suggestion list is not mounted, so the page
                // keys must not open it and must not move a retained cursor.
                // Open, those handlers require `manager.focusedKey != null` --
                // a mouse-opened, selection-less ComboBox has a null cursor
                // and must answer nothing until an arrow establishes one.
                // With a cursor the list is non-scrollable -- HeroUI v3.2.4
                // puts the overflow scrolling on the Popover while the ListBox
                // element is `overflow-clip` -- so a page takes the enabled
                // end: `stops` already omits disabled rows, whatever the
                // list's length or scroll state.
                let page_move = match nav_key {
                    "pagedown" if is_open && from.is_some() => stops.last().copied(),
                    "pageup" if is_open && from.is_some() => stops.first().copied(),
                    _ => None,
                }
                .filter(|next| Some(*next) != from);
                match page_move.map_or_else(
                    || crate::list_nav::resolve(&stops, from, nav_key, wrap),
                    crate::list_nav::Move::To,
                ) {
                    crate::list_nav::Move::To(next) => {
                        let next_cursor = Some(cursor_for(&rows, next, None));
                        held.update(cx, |v, cx| {
                            *v = next_cursor;
                            cx.notify();
                        });
                        if virtual_rows {
                            key_list_scroll.scroll_to_item(next, gpui::ScrollStrategy::Center);
                        } else {
                            key_panel_scroll.scroll_to_item(next);
                        }
                        // Walking the list opens it, which is what typing does.
                        if let Some(held) = &open_own_keys {
                            held.update(cx, |v, cx| {
                                *v = true;
                                cx.notify();
                            });
                        }
                        if !was_open {
                            show_all_items.update(cx, |v, _| *v = true);
                            if let Some(cb) = &on_open_change {
                                cb(true, window, cx);
                            }
                        }
                    }
                    crate::list_nav::Move::Activate => {
                        // The activation reads the held cursor's key, not the
                        // rendered row: a cursor retained across a query reset
                        // (the multiple-mode pick above) can sit on an item
                        // the capped or filtered list no longer renders, and
                        // Enter must still toggle it.
                        let Some(item_key) =
                            held.read(cx).as_ref().map(|focused| focused.key.clone())
                        else {
                            return;
                        };
                        let item_label = label_of_key(&key_items, &item_key)
                            .cloned()
                            .unwrap_or_default();
                        // An Enter that picks a row is the list's, not an
                        // enclosing form's. A Tab mapped here keeps its
                        // native motion: only Enter is stopped.
                        if key == "enter" {
                            cx.stop_propagation();
                        }
                        if key_multiple {
                            let had_query = !state.read(cx).value().is_empty();
                            state.update(cx, |st, cx| {
                                st.set_value(String::new());
                                cx.notify();
                            });
                            held.update(cx, |v, _| {
                                if let Some(focused) = v {
                                    focused.hidden_query = Some(String::new());
                                }
                            });
                            let mut next = selected_now.clone();
                            toggle_key(&mut next, &item_key);
                            if let Some(held) = &key_selection_own {
                                let next = next.clone();
                                held.update(cx, |v, cx| {
                                    *v = next;
                                    cx.notify();
                                });
                            }
                            if let Some(cb) = &on_selection_change_all {
                                cb(&next, window, cx);
                            }
                            if had_query {
                                if let Some(cb) = &on_input_change {
                                    cb("", window, cx);
                                }
                            }
                            return;
                        }
                        // Taking a suggestion fills the field with the row's
                        // label and closes the list, the way a click does.
                        state.update(cx, |st, cx| {
                            st.set_value(item_label.to_string());
                            cx.notify();
                        });
                        // Uncontrolled: record the pick in the selection too,
                        // or `ComboBox.Value` would never see it.
                        if let Some(held) = &key_selection_own {
                            let next = vec![item_key.clone()];
                            held.update(cx, |v, cx| {
                                *v = next;
                                cx.notify();
                            });
                        }
                        held.update(cx, |v, _| *v = None);
                        if key == "tab" {
                            key_blur.consume(cx);
                        }
                        if let Some(cb) = &on_selection_change {
                            cb(&item_key, window, cx);
                        }
                        key_close(window, cx);
                    }
                    crate::list_nav::Move::Ignore => {}
                }
            });
        }

        let show_list = overlay_active
            && !self.is_disabled
            && (!matches.is_empty() || self.allows_empty_collection);

        let escape_close = close_open.clone();
        let mut root = root;
        root =
            util::dismiss_on_escape_with_token(root, dismissal_token.clone(), move |window, cx| {
                escape_close(window, cx);
                util::DismissResult::Handled
            });
        if overlay_active && !show_list {
            let dismiss_close = close_open.clone();
            let dismiss_commit = commit_value.clone();
            let dismiss_blur = blur_scope.clone();
            root = util::dismiss_on_press_outside_with_token(
                root,
                dismissal_token.clone(),
                move |window, cx| {
                    dismiss_blur.consume(cx);
                    dismiss_commit(window, cx);
                    dismiss_close(window, cx);
                    util::DismissResult::Handled
                },
            );
        }
        if show_list {
            let panel = div()
                .id(gpui::ElementId::Name(
                    format!("combobox-{entity_id}-panel").into(),
                ))
                .w_full()
                .flex()
                .flex_col()
                .gap(px(2.))
                .p(px(4.))
                .max_h(px(240.))
                .overflow_y_scroll()
                .track_scroll(&panel_scroll_now)
                .rounded(container_radius)
                .bg(colors.overlay.background)
                // v3 gives a floating panel no border: `.popover` and friends are
                // `bg-overlay shadow-overlay` and a radius, and dark mode's
                // inset hairline is what separates the panel from the page.
                .when_some(layout.overlay_hairline, |el, hairline| {
                el.border(layout.border_width).border_color(hairline)
                })
                .text_color(colors.overlay.foreground)
                .when(
                    !layout.overlay_shadow.is_empty(),
                    |e: gpui::Stateful<gpui::Div>| e.shadow(layout.overlay_shadow.clone()),
                );

            // React Aria dismisses the list on a press outside it; Escape is
            // read in the field's own key handler above. A press that started
            // in the input-plus-panel subtree is not outside the ComboBox;
            // the input keeps the list open, while the chevron and rows own
            // their respective clicks.
            let dismiss_close = close_open.clone();
            let dismiss_commit = commit_value.clone();
            let dismiss_blur = blur_scope;
            let mut panel = util::dismiss_on_press_outside_with_token(
                panel,
                dismissal_token,
                move |window, cx| {
                    if inside_pressed.get() {
                        return util::DismissResult::Declined;
                    }
                    dismiss_blur.consume(cx);
                    dismiss_commit(window, cx);
                    dismiss_close(window, cx);
                    util::DismissResult::Handled
                },
            );

            if matches.is_empty() {
                // `allowsCustomValue` means an unmatched query is still valid,
                // so the empty state has to say something different.
                let message = if self.allows_custom_value {
                    "Press Enter to use this value"
                } else {
                    "No matching options"
                };
                panel = panel.child(
                    div()
                        .px(px(8.))
                        .py(px(6.))
                        .text_size(util::FIELD_TEXT)
                        .text_color(colors.muted)
                        .child(message),
                );
            }

            // Everything a row reads, owned: `uniform_list`'s callback is
            // `'static` and runs again on every scroll, so it cannot borrow
            // `self` or the theme -- and one row builder for both paths is what
            // keeps a virtual list drawing the same row as a short one.
            let matches_len = matches.len();
            let rows = matches;
            let sections = self.sections.clone();
            let row_disabled_keys = self.disabled_keys.clone();
            let row_selected_keys = self.selected_keys.clone();
            let indicator: Option<Rc<dyn Fn(bool) -> gpui::AnyElement>> =
                self.indicator.take().map(Rc::from);
            let on_change_all = self.on_selection_change_all.clone();
            let row_input_change = self.on_input_change.clone();
            let on_change_one = self.on_selection_change.clone();
            let row_state = self.state.clone();
            let row_cursor = cursor;
            let selection_own = selection_own;
            let row_open_own = open_own.clone();
            let row_open_state = open_state;
            let row_close_open = close_open.clone();
            let row_muted = colors.muted;
            let row_hover_bg = colors.default.color;
            let row_focus = colors.focus;
            let row_accent = colors.accent.color;
            let row_disabled_opacity = layout.disabled_opacity;
            let row_of = move |index: usize, fixed_h: Option<gpui::Pixels>, cx: &mut App| {
                let item = &rows[index];
                // A section header rides above the row it introduces, so the two
                // are one element -- a virtual row is one slot tall.
                let mut head: Vec<gpui::AnyElement> = Vec::new();
                let done = |head: Vec<gpui::AnyElement>, row: gpui::AnyElement| {
                    div()
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
                        div()
                            .px(px(8.))
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
                let hover_bg = row_hover_bg;
                let mut row = div()
                        .id(gpui::ElementId::Name(
                            format!("combobox-{entity_id}-item-{}", item.key()).into(),
                        ))
                        // `.list-box-item`: `min-h-9 rounded-2xl px-2 py-1.5 gap-3`.
                        .min_h(util::FIELD_HEIGHT)
                        .px(px(8.))
                        .py(px(6.))
                        .gap(px(12.))
                        .rounded(util::soft_radius(cx))
                        .flex()
                        .items_center()
                        .justify_between()
                        .text_size(util::FIELD_TEXT)
                        .child(item.label().to_string());

                if item_disabled {
                    row = row.opacity(row_disabled_opacity);
                } else {
                    row = row.cursor_pointer().hover(move |s| s.bg(hover_bg));
                }

                // `status-focused` on the row the keyboard is on.
                if cursor_at == Some(index) {
                    row = row.border_2().border_color(row_focus);
                }

                // `ListBox.ItemIndicator`: a caller-drawn tick replaces the
                // check glyph, and is asked for on every row so it can draw the
                // unselected state too.
                let row_selected = row_selected_keys.contains(item.key());
                match &indicator {
                    Some(render) => row = row.child(render(row_selected)),
                    None if multiple && row_selected => {
                        row = row.child(
                            gpui::svg()
                                .size(px(13.))
                                .path(icons::CHECK)
                                .text_color(row_accent),
                        );
                    }
                    None => {}
                }

                if item_disabled {
                    return done(head, row.into_any_element());
                }

                // Multiple mode toggles membership and leaves the panel open.
                if multiple {
                    if on_change_all.is_some() || selection_own.is_some() {
                        let cb = on_change_all.clone();
                        let own = selection_own.clone();
                        let current = row_selected_keys.clone();
                        let value = item.key().clone();
                        let state = row_state.clone();
                        let cursor = row_cursor.clone();
                        let next_cursor = cursor_for(&rows, index, Some(String::new()));
                        let input_change = row_input_change.clone();
                        row = row.on_click(move |_, window, cx| {
                            let had_query = !state.read(cx).value().is_empty();
                            state.read(cx).focus_handle.focus(window);
                            state.update(cx, |st, cx| {
                                st.set_value(String::new());
                                cx.notify();
                            });
                            cursor.update(cx, |v, _| *v = Some(next_cursor.clone()));
                            let mut next = current.clone();
                            toggle_key(&mut next, &value);
                            // Uncontrolled: keep the new set, or picking an
                            // item would do nothing.
                            if let Some(held) = &own {
                                let set = next.clone();
                                held.update(cx, |v, cx| {
                                    *v = set;
                                    cx.notify();
                                });
                            }
                            if let Some(cb) = &cb {
                                cb(&next, window, cx);
                            }
                            if had_query {
                                if let Some(cb) = &input_change {
                                    cb("", window, cx);
                                }
                            }
                        });
                    }
                    return done(head, row.into_any_element());
                }

                // Taking a suggestion fills the field with the row's label and
                // closes the list, the way a click does; the selection rides
                // the row's key.
                let value = item.key().clone();
                let label = item.label().clone();
                let state = row_state.clone();
                let on_selection_change = on_change_one.clone();
                let own = selection_own.clone();
                let open_own = row_open_own.clone();
                let close_open = row_close_open.clone();
                row = row.on_click(move |_, window, cx| {
                    let is_open = open_own
                        .as_ref()
                        .map_or(row_open_state, |held| *held.read(cx));
                    if !is_open {
                        return;
                    }
                    state.update(cx, |s, cx| {
                        s.set_value(label.to_string());
                        cx.notify();
                    });
                    // Uncontrolled: record the pick in the selection too, or
                    // `ComboBox.Value` would never see it.
                    if let Some(held) = &own {
                        let next = vec![value.clone()];
                        held.update(cx, |v, cx| {
                            *v = next;
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &on_selection_change {
                        cb(&value, window, cx);
                    }
                    close_open(window, cx);
                });

                done(head, row.into_any_element())
            };

            match self.row_height {
                // Virtual: only the rows in view are built, which is what makes
                // a thousand options affordable.
                Some(row_height) => {
                    panel = panel.child(
                        gpui::uniform_list(
                            gpui::ElementId::Name(format!("combobox-{entity_id}-rows").into()),
                            matches_len,
                            move |range, _window, cx| {
                                range
                                    .map(|i| row_of(i, Some(row_height), cx))
                                    .collect::<Vec<_>>()
                            },
                        )
                        .track_scroll(list_scroll_now)
                        .h(px(240.))
                        .w_full(),
                    );
                }
                None => {
                    for index in 0..matches_len {
                        panel = panel.child(row_of(index, None, cx));
                    }
                }
            }

            let panel = crate::anim::entering_zoom(
                panel,
                gpui::ElementId::Name(format!("combobox-{entity_id}-anim").into()),
                crate::anim::ZoomBox::panel(px(4.), container_radius).padding_x(px(4.)),
                crate::anim::Motion::LIST_IN,
                cx,
            );
            input_group = input_group.child(util::floating(
                util::placed_field_panel(self.placement, px(6.)).child(panel),
            ));
        }

        // The popover hangs off the input group it belongs to, so the group --
        // not the panel -- is the root's child; the deferred panel still paints
        // over the value row below it.
        root.child(input_group)
            .when_some(value_content, |root, value| root.child(value))
            .track_focus(&blur_focus)
    }
}

// The pinned `.combo-box__trigger` hovers `text-field-foreground` and stays
// `bg-transparent`; a filled hover looks plausible on screen, so the check is
// mechanical.
#[cfg(test)]
mod hover_tokens {
    #[test]
    fn the_trigger_hover_recolours_the_text_and_fills_nothing() {
        // Scan the implementation only; this test's own text names the
        // forbidden accessor.
        let source = include_str!("combo_box.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        assert!(
            source.contains(".hover(move |s| s.text_color(trigger_hover_fg))"),
            "the trigger hover must recolour the text to `text-field-foreground` \
             (pinned `.combo-box__trigger:hover`)"
        );
        assert!(
            !source.contains("let hover_bg = colors.default.hover();"),
            "the trigger hover must not fill a background \
             (pinned `.combo-box__trigger` is `bg-transparent`)"
        );
    }
}
