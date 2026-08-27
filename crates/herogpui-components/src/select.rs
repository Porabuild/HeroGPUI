//! Select — port of `@heroui/select` with single and multiple selection.

use std::{cell::RefCell, collections::BTreeSet, rc::Rc};

use gpui::{
    prelude::*, px, App, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window,
};
use herogpui_core::{Color, FieldVariant, Placement, SelectionMode};
use herogpui_theme::ActiveTheme;

use crate::{icons, util};

type OnSelectionChange = std::sync::Arc<dyn Fn(Option<usize>, &mut Window, &mut App) + 'static>;

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

/// HeroUI Select (controlled).
#[derive(IntoElement)]
pub struct Select {
    /// `name` — the name this control submits under; read back by
    /// [`Self::form_field`].
    name: Option<SharedString>,
    id: gpui::ElementId,
    options: Vec<SharedString>,
    selected: Option<usize>,
    /// Whether `value` was supplied. `Option<usize>` cannot distinguish
    /// "controlled, nothing selected" from "uncontrolled" on its own.
    is_controlled: bool,
    default_value: Option<usize>,
    /// Backs `selectionMode="multiple"`; `selected` backs `single`.
    selected_indices: BTreeSet<usize>,
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
    /// `ListBox.Section` — the heading that precedes an option, by index.
    sections: Vec<(usize, SharedString)>,
    /// `ListBox.ItemIndicator` — draws the tick. The closure is handed whether
    /// the row is selected.
    indicator: Option<Box<dyn Fn(bool) -> gpui::AnyElement + 'static>>,
    /// `Select.Value` — draws the trigger's value. The closure is handed the
    /// selected index, or `None` while the placeholder shows.
    value_content: Option<Box<dyn Fn(util::SelectionValue<'_>) -> gpui::AnyElement + 'static>>,
    is_required: bool,
    disabled_keys: std::collections::HashSet<usize>,
    full_width: bool,
    on_open_change: Option<std::sync::Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>>,
    on_selection_change: Option<OnSelectionChange>,
    on_selection_change_all:
        Option<std::sync::Arc<dyn Fn(&[usize], &mut Window, &mut App) + 'static>>,
    /// Mirrors the current selection, validity, successful state, focus and
    /// reset behavior for a live [`crate::form::FormField`].
    form_state: Rc<RefCell<crate::form::LiveFormFieldState>>,
}

impl Select {
    /// `onChange` — the v3 name for [`Select::on_selection_change`].
    pub fn on_change(
        self,
        handler: impl Fn(Option<usize>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change(handler)
    }

    /// `disabledKeys` — indices that cannot be chosen.
    pub fn disabled_keys(mut self, keys: impl IntoIterator<Item = usize>) -> Self {
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

    /// `ListBox.Section` — a heading rendered above the option at `index`.
    pub fn section_before(mut self, index: usize, label: impl Into<SharedString>) -> Self {
        self.sections.push((index, label.into()));
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
    pub fn new(id: impl Into<gpui::ElementId>, options: Vec<SharedString>) -> Self {
        Self {
            name: None,
            id: id.into(),
            options,
            selected: None,
            is_controlled: false,
            default_value: None,
            selected_indices: BTreeSet::new(),
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
            disabled_keys: std::collections::HashSet::new(),
            full_width: false,
            on_open_change: None,
            on_selection_change: None,
            on_selection_change_all: None,
            form_state: live_form_state(),
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
    /// ```ignore
    /// let field = control.form_field();
    /// form.field(field.unwrap()).child(control)
    /// ```
    pub fn form_field(&self) -> Option<crate::form::FormField> {
        let name = self.name.clone()?;
        let selected = if self.is_controlled {
            self.selected
        } else {
            self.default_value
        };
        sync_select_form(
            &self.form_state,
            select_form_value(
                self.selection_mode,
                &self.options,
                selected,
                &self.selected_indices,
            ),
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

    /// The chosen indices under `selectionMode="multiple"`.
    pub fn selected_indices(mut self, indices: impl IntoIterator<Item = usize>) -> Self {
        self.selected_indices = indices.into_iter().collect();
        self
    }

    /// Reports the whole selection, for `selectionMode="multiple"`.
    pub fn on_selection_change_all(
        mut self,
        handler: impl Fn(&[usize], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change_all = Some(std::sync::Arc::new(handler));
        self
    }

    /// `value` — the selected option, by index. Supplying it makes the select
    /// controlled, even with `None`.
    pub fn value(mut self, i: Option<usize>) -> Self {
        self.selected = i;
        self.is_controlled = true;
        self
    }

    /// `defaultValue` — the uncontrolled initial selection.
    ///
    /// Only consulted when `value` is not supplied; the select then owns the
    /// selection and choosing an option moves it.
    pub fn default_value(mut self, i: Option<usize>) -> Self {
        self.default_value = i;
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

    pub fn on_open_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_open_change = Some(std::sync::Arc::new(f));
        self
    }

    pub fn on_selection_change(
        mut self,
        f: impl Fn(Option<usize>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change = Some(std::sync::Arc::new(f));
        self
    }
}

impl Select {
    fn value_text_single(&self, selected: Option<usize>) -> SharedString {
        selected
            .and_then(|i| self.options.get(i).cloned())
            .unwrap_or_else(|| self.placeholder.clone())
    }

    fn value_text_multiple(&self) -> SharedString {
        let names: Vec<String> = self
            .selected_indices
            .iter()
            .filter_map(|i| self.options.get(*i).map(ToString::to_string))
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
            gpui::ElementId::Name(format!("{:?}-open", self.id).into()),
            self.is_open,
            self.default_open,
        );
        let (overlay_phase, dismissal_token) = util::overlay_scope(
            window,
            cx,
            el_name(format!("select-{}-overlay", id_debug(&self.id))),
            is_open,
            true,
        );
        let overlay_active = overlay_phase != util::OverlayPhase::Closed;
        let (selected, value_own) = util::controlled(
            window,
            cx,
            gpui::ElementId::Name(format!("{:?}-value", self.id).into()),
            self.is_controlled.then_some(self.selected),
            self.default_value,
        );
        let multiple = self.selection_mode == SelectionMode::Multiple;
        let form_default_indices = if multiple {
            let slot = window.use_keyed_state(
                el_name(format!("select-{}-form-default", id_debug(&self.id))),
                cx,
                |_, _| self.selected_indices.clone(),
            );
            slot.read(cx).clone()
        } else {
            BTreeSet::new()
        };
        sync_select_form(
            &self.form_state,
            select_form_value(
                self.selection_mode,
                &self.options,
                selected,
                &self.selected_indices,
            ),
            self.is_invalid,
            !self.is_disabled,
        );
        let reset_own = value_own.clone();
        let reset_state = self.form_state.clone();
        let reset_change = self
            .is_controlled
            .then(|| self.on_selection_change.clone())
            .flatten();
        let reset_change_all = self.on_selection_change_all.clone();
        let reset_index = self.default_value;
        let reset_options = self.options.clone();
        let reset_mode = self.selection_mode;
        self.form_state.borrow_mut().restore = (reset_own.is_some()
            || reset_change.is_some()
            || (multiple && reset_change_all.is_some()))
        .then(|| {
            util::shared(move |window: &mut Window, cx: &mut App| {
                if reset_mode == SelectionMode::Multiple {
                    reset_state.borrow_mut().value =
                        select_form_value(reset_mode, &reset_options, None, &form_default_indices);
                    if let Some(on_change) = &reset_change_all {
                        let keys: Vec<usize> = form_default_indices.iter().copied().collect();
                        on_change(&keys, window, cx);
                    }
                } else {
                    reset_state.borrow_mut().value = select_form_value(
                        reset_mode,
                        &reset_options,
                        reset_index,
                        &BTreeSet::new(),
                    );
                    if let Some(held) = &reset_own {
                        held.update(cx, |selected, cx| {
                            *selected = reset_index;
                            cx.notify();
                        });
                    }
                    if let Some(on_change) = &reset_change {
                        on_change(reset_index, window, cx);
                    }
                }
            }) as std::sync::Arc<dyn Fn(&mut Window, &mut App)>
        });

        // The trigger is what holds focus, so the open list can be walked with
        // the arrows the way v3's is.
        let focus_handle = window.use_keyed_state(
            el_name(format!("select-{}-focus", id_debug(&self.id))),
            cx,
            |_, cx| cx.focus_handle().tab_stop(true),
        );
        let focus_handle = focus_handle.read(cx).clone();
        self.form_state.borrow_mut().focus = Some(focus_handle.clone());
        let cursor = window.use_keyed_state(
            el_name(format!("select-{}-cursor", id_debug(&self.id))),
            cx,
            |_, _| None::<usize>,
        );
        let cursor_at = *cursor.read(cx);
        // v3's list is `overflow-y-auto`, and React Aria keeps the focused
        // option in view. Both need a handle: the virtual list has its own kind,
        // and a plain scrolling div has the other. `use_keyed_state` takes `cx`
        // mutably, so they precede the theme.
        let list_scroll = window.use_keyed_state(
            el_name(format!("select-{}-list-scroll", id_debug(&self.id))),
            cx,
            |_, _| gpui::UniformListScrollHandle::new(),
        );
        let panel_scroll = window.use_keyed_state(
            el_name(format!("select-{}-panel-scroll", id_debug(&self.id))),
            cx,
            |_, _| gpui::ScrollHandle::new(),
        );
        let list_scroll_now = list_scroll.read(cx).clone();
        let panel_scroll_now = panel_scroll.read(cx).clone();
        // The letters typed so far, which a search resetting every frame could
        // not accumulate.
        let typeahead = window.use_keyed_state(
            el_name(format!("select-{}-typed", id_debug(&self.id))),
            cx,
            |_, _| crate::list_nav::Typeahead::default(),
        );

        // Pinned `usePopover` closes when focus leaves the trigger-plus-list
        // scope. Blur deliberately leaves focus on its destination.
        let blur_base = format!("select-{}", id_debug(&self.id));
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
                cb(false, window, cx);
            }
        });

        let sem = cx.role(Color::Accent);
        let colors = cx.colors();
        let layout = cx.layout();

        // `.select__trigger` is `min-h-9 ... text-sm`.
        let (h, text) = (util::FIELD_HEIGHT, util::FIELD_TEXT);

        let trigger_id = el_name(format!("select-{}", id_debug(&self.id)));
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
            .flex()
            .items_center()
            .justify_between()
            .gap(px(8.))
            .min_h(h)
            .px(px(12.))
            .text_size(text)
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
                FieldVariant::Secondary => colors.default.soft_hover(),
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
            let stops: Vec<usize> = (0..self.options.len())
                .filter(|i| !self.disabled_keys.contains(i))
                .collect();
            let held = cursor;
            let wrap = self.should_focus_wrap;
            // Every option's text, so a typed letter can find one.
            let labels: Vec<String> = self.options.iter().map(ToString::to_string).collect();
            let typed = typeahead;
            let open_own_keys = open_own.clone();
            let value_own_keys = value_own.clone();
            let form_state_keys = self.form_state.clone();
            let form_options_keys = self.options.clone();
            let on_open_change = self.on_open_change.clone();
            let on_select = self.on_selection_change.clone();
            let was_open = is_open;
            let virtual_rows = self.row_height.is_some();
            let key_list_scroll = list_scroll_now.clone();
            let key_panel_scroll = panel_scroll_now.clone();
            let fh = focus_handle.clone();
            field = field
                .track_focus(&focus_handle)
                .key_context("Select")
                .on_mouse_down(gpui::MouseButton::Left, move |_, window, _| {
                    window.focus(&fh);
                })
                .on_key_down(move |event, window, cx| {
                    let key = event.keystroke.key.as_str();
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
                                cb(true, window, cx);
                            }
                            return;
                        }
                        // A closed select still answers letters: React Aria
                        // picks the matching option where it stands rather than
                        // opening the list.
                        if !crate::list_nav::is_typeahead_key(key) {
                            return;
                        }
                        let now = std::time::Instant::now();
                        let (query, repeat) = typed.update(cx, |t, _| {
                            let query = t.push(key, now);
                            (query, t.is_repeat())
                        });
                        let Some(found) =
                            crate::list_nav::typeahead(&labels, &stops, selected, &query, repeat)
                        else {
                            return;
                        };
                        if let Some(held) = &value_own_keys {
                            form_state_keys.borrow_mut().value = crate::form::FormValue::Text(
                                form_options_keys.get(found).cloned().unwrap_or_default(),
                            );
                            held.update(cx, |v, cx| {
                                *v = Some(found);
                                cx.notify();
                            });
                        }
                        if let Some(cb) = &on_select {
                            cb(Some(found), window, cx);
                        }
                        return;
                    }
                    let from = *held.read(cx);
                    match crate::list_nav::resolve(&stops, from, key, wrap) {
                        crate::list_nav::Move::To(next) => {
                            held.update(cx, |v, cx| {
                                *v = Some(next);
                                cx.notify();
                            });
                            // React Aria keeps the focused option in view; the
                            // highlight walking off the bottom of the list looks
                            // like the arrows have stopped working.
                            if virtual_rows {
                                key_list_scroll.scroll_to_item(next, gpui::ScrollStrategy::Center);
                            } else {
                                key_panel_scroll.scroll_to_item(next);
                            }
                        }
                        crate::list_nav::Move::Activate => {
                            // Take the selection only. Closing is the trigger's
                            // click listener, which gpui fires from the same
                            // keystroke -- doing it here as well would toggle the
                            // list back open.
                            let Some(index) = from else { return };
                            if let Some(held) = &value_own_keys {
                                form_state_keys.borrow_mut().value = crate::form::FormValue::Text(
                                    form_options_keys.get(index).cloned().unwrap_or_default(),
                                );
                                held.update(cx, |v, cx| {
                                    *v = Some(index);
                                    cx.notify();
                                });
                            }
                            if let Some(cb) = &on_select {
                                cb(Some(index), window, cx);
                            }
                        }
                        crate::list_nav::Move::Ignore => {
                            // Typeahead moves the cursor over the open list.
                            if !crate::list_nav::is_typeahead_key(key) {
                                return;
                            }
                            let now = std::time::Instant::now();
                            let (query, repeat) = typed.update(cx, |t, _| {
                                let query = t.push(key, now);
                                (query, t.is_repeat())
                            });
                            if let Some(found) =
                                crate::list_nav::typeahead(&labels, &stops, from, &query, repeat)
                            {
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
            self.value_text_multiple()
        } else {
            self.value_text_single(selected)
        };
        let mut chosen: Vec<usize> = if multiple {
            self.selected_indices
                .iter()
                .copied()
                .filter(|index| self.options.get(*index).is_some())
                .collect()
        } else {
            selected
                .filter(|index| self.options.get(*index).is_some())
                .into_iter()
                .collect()
        };
        chosen.sort_unstable();
        let has_value = !chosen.is_empty();

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
                let items: Vec<SharedString> = chosen
                    .iter()
                    .filter_map(|i| self.options.get(*i).cloned())
                    .collect();
                let names = items.iter().map(ToString::to_string).collect::<Vec<_>>();
                let text = format_selected_names(&names);
                gpui::div()
                    .flex_1()
                    .min_w_0()
                    .child(render(util::SelectionValue {
                        selected_items: &items,
                        selected_indices: &chosen,
                        selected_text: &text,
                        is_placeholder: !has_value,
                        default_children,
                    }))
                    .into_any_element()
            }
            None => default_children,
        };
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
                .on_click(move |_, window, cx| {
                    // Uncontrolled: flip our own copy, or the trigger would be
                    // inert without a caller handler.
                    if let Some(held) = &own {
                        held.update(cx, |v, cx| {
                            *v = !open;
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &on_open_change {
                        cb(!open, window, cx);
                    }
                });
        }

        // listbox panel
        let mut root = gpui::div().relative().max_w(px(320.));
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
                    cb(false, window, cx);
                }
                util::DismissResult::Handled
            });

        if overlay_active && self.options.is_empty() {
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
                        cb(false, window, cx);
                    }
                    util::DismissResult::Handled
                },
            );
        }

        if overlay_active && !self.options.is_empty() {
            let base = format!("select-list-{}", id_debug(&self.id));
            let options_len = self.options.len();
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
                .id(el_name(format!("{base}-scroll")))
                .overflow_y_scroll()
                .track_scroll(&panel_scroll_now)
                .max_h(px(280.));

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
                        cb(false, window, cx);
                    }
                    util::DismissResult::Handled
                },
            );

            // The theme tokens the row draws with, copied out: `cx.colors()`
            // hands back a borrow of the app, which a `'static` closure cannot
            // hold.
            let row_muted = colors.muted;
            let row_fg = colors.foreground;
            let row_focus = colors.focus;
            let row_hover_bg = colors.default.soft();
            let row_accent = sem.color;
            let row_disabled_opacity = layout.disabled_opacity;
            // Everything a row reads, owned: `uniform_list`'s callback is
            // `'static` and is called again on every scroll, so it cannot
            // borrow `self` -- and one row builder for both paths is what keeps
            // a virtual list drawing the same row as a short one.
            let options = self.options.clone();
            let sections = self.sections.clone();
            let selected_indices = self.selected_indices.clone();
            let opt_disabled_keys = self.disabled_keys.clone();
            let indicator: Option<Rc<dyn Fn(bool) -> gpui::AnyElement>> =
                self.indicator.take().map(Rc::from);
            let on_change_all = self.on_selection_change_all.clone();
            let on_change_one = self.on_selection_change.clone();
            let value_own = value_own;
            let open_own = open_own;
            let form_state_rows = self.form_state.clone();
            let on_close = self.on_open_change.clone();
            let base_row = base.clone();
            let row = move |i: usize, fixed_h: Option<gpui::Pixels>, cx: &mut App| {
                let base = &base_row;
                let opt = &options[i];
                let mut rows = Vec::new();
                // `ListBox.Section`'s `Header`: `text-xs` in the muted colour,
                // above the option it introduces.
                if let Some((_, label)) = sections.iter().find(|(at, _)| *at == i) {
                    rows.push(
                        gpui::div()
                            .px(px(8.))
                            .pt(px(6.))
                            .pb(px(2.))
                            .text_size(px(12.))
                            .text_color(row_muted)
                            .child(label.to_string())
                            .into_any_element(),
                    );
                }
                let is_sel = if multiple {
                    selected_indices.contains(&i)
                } else {
                    selected == Some(i)
                };
                let opt_disabled = opt_disabled_keys.contains(&i);
                let mut item = gpui::div()
                        .id(el_name(format!("{base}-opt-{i}")))
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
                        .text_size(util::FIELD_TEXT);

                if opt_disabled {
                    item = item.opacity(row_disabled_opacity);
                } else if panel_interactive {
                    item = item.cursor_pointer().hover(move |s| s.bg(row_hover_bg));
                }

                if is_sel {
                    item = item
                        .text_color(row_accent)
                        .font_weight(gpui::FontWeight::MEDIUM);
                } else {
                    item = item.text_color(row_fg);
                }

                // `status-focused` on the row the keyboard is on.
                if cursor_at == Some(i) {
                    item = item.border_2().border_color(row_focus);
                }

                item = item.child(gpui::div().truncate().child(opt.to_string()));

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
                        if let Some(cb) = on_change_all.clone() {
                            let current = selected_indices.clone();
                            item = item.on_click(move |_, window, cx| {
                                let mut next = current.clone();
                                if !next.remove(&i) {
                                    next.insert(i);
                                }
                                let next: Vec<usize> = next.into_iter().collect();
                                cb(&next, window, cx);
                            });
                        }
                    } else if on_change_one.is_some() || value_own.is_some() || open_own.is_some() {
                        let on_select = on_change_one.clone();
                        let on_close = on_close.clone();
                        let value_own = value_own.clone();
                        let open_own = open_own.clone();
                        let form_state_pick = form_state_rows.clone();
                        let picked = opt.clone();
                        item = item.on_click(move |_, window, cx| {
                            // Uncontrolled: take the selection and close, or
                            // choosing an option would do nothing.
                            if let Some(held) = &value_own {
                                form_state_pick.borrow_mut().value =
                                    crate::form::FormValue::Text(picked.clone());
                                held.update(cx, |v, cx| {
                                    *v = Some(i);
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
                                cb(false, window, cx);
                            }
                            if let Some(f) = &on_select {
                                f(Some(i), window, cx);
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
                // a thousand options affordable.
                Some(row_height) => {
                    panel = panel.child(
                        gpui::uniform_list(
                            el_name(format!("{base}-rows")),
                            options_len,
                            move |range, _window, cx| {
                                range
                                    .map(|i| row(i, Some(row_height), cx))
                                    .collect::<Vec<_>>()
                            },
                        )
                        .track_scroll(list_scroll_now)
                        .h(px(280.))
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
                    el_name(format!("{base}-panel-out")),
                    zoom,
                    crate::anim::Motion::LIST_OUT,
                    cx,
                )
            } else {
                crate::anim::entering_zoom(
                    panel,
                    el_name(format!("{base}-panel")),
                    zoom,
                    crate::anim::Motion::LIST_IN,
                    cx,
                )
            };
            root = root.child(util::floating(
                util::placed_field_panel(self.placement, px(6.)).child(panel),
            ));
        }

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

fn select_form_value(
    mode: SelectionMode,
    options: &[SharedString],
    selected: Option<usize>,
    indices: &BTreeSet<usize>,
) -> crate::form::FormValue {
    if mode == SelectionMode::Multiple {
        crate::form::FormValue::Keys(
            indices
                .iter()
                .filter_map(|i| options.get(*i).cloned())
                .collect(),
        )
    } else {
        crate::form::FormValue::Text(
            selected
                .and_then(|i| options.get(i).cloned())
                .unwrap_or_default(),
        )
    }
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

fn el_name(s: String) -> gpui::ElementId {
    gpui::ElementId::Name(s.into())
}

fn id_debug(id: &gpui::ElementId) -> String {
    format!("{id:?}").trim_matches('"').to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_value_text_matches_pinned_select_value() {
        let select = Select::new(
            "select-default-text",
            vec!["Alpha".into(), "Beta".into(), "Gamma".into()],
        )
        .selection_mode(SelectionMode::Multiple)
        .selected_indices([0, 1, 2]);

        assert_eq!(select.value_text_multiple(), "Alpha, Beta, and Gamma");
        assert_eq!(
            format_selected_names(&["Alpha".into(), "Beta".into()]),
            "Alpha and Beta"
        );
        assert_eq!(
            Select::new("select-default-placeholder", Vec::new()).placeholder,
            "Select an item"
        );
    }
}
