//! RadioGroup — port of `@heroui/radio`.

use std::{cell::RefCell, rc::Rc};

use gpui::{prelude::*, px, App, IntoElement, RenderOnce, SharedString, Styled, Window};
use herogpui_core::{Color, FieldVariant, Orientation};
use herogpui_theme::ActiveTheme;

/// One radio's visible label, submitted value, and local disabled state.
#[derive(Clone)]
pub struct RadioOption {
    label: SharedString,
    value: SharedString,
    is_disabled: bool,
    description: Option<SharedString>,
    error_message: Option<SharedString>,
}

impl RadioOption {
    pub fn new(label: impl Into<SharedString>) -> Self {
        let label = label.into();
        Self {
            value: label.clone(),
            label,
            is_disabled: false,
            description: None,
            error_message: None,
        }
    }

    /// `value` — submitted and reported independently of the visible label.
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = value.into();
        self
    }

    /// `isDisabled` — disables this option only.
    pub fn is_disabled(mut self, value: bool) -> Self {
        self.is_disabled = value;
        self
    }

    /// `Description` — help text below this option's clickable content.
    pub fn description(mut self, text: impl Into<SharedString>) -> Self {
        self.description = Some(text.into());
        self
    }

    /// `FieldError` — validation text below this option's clickable content.
    pub fn error_message(mut self, text: impl Into<SharedString>) -> Self {
        self.error_message = Some(text.into());
        self
    }
}

impl From<SharedString> for RadioOption {
    fn from(label: SharedString) -> Self {
        Self::new(label)
    }
}

impl From<String> for RadioOption {
    fn from(label: String) -> Self {
        Self::new(label)
    }
}

impl From<&str> for RadioOption {
    fn from(label: &str) -> Self {
        Self::new(label.to_owned())
    }
}

/// Field state handed to the root `Radio` and `Radio.Indicator` renderers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RadioOptionState {
    pub is_selected: bool,
    pub is_disabled: bool,
    pub is_read_only: bool,
    pub is_invalid: bool,
    pub is_required: bool,
}

/// HeroUI RadioGroup.
#[derive(IntoElement)]
pub struct RadioGroup {
    /// `name` — the name this control submits under; read back by
    /// [`Self::form_field`].
    name: Option<SharedString>,
    id: gpui::ElementId,
    options: Vec<RadioOption>,
    /// Mirrors the current value, validity, successful state, focus and reset
    /// behavior for a live [`crate::form::FormField`].
    form_state: Rc<RefCell<crate::form::LiveFormFieldState>>,
    /// `Radio`'s `children`-as-a-function: handed the option and its state.
    option_content:
        Option<std::sync::Arc<dyn Fn(&SharedString, RadioOptionState) -> gpui::AnyElement>>,
    /// `Radio.Indicator` children — replaces the built-in dot per option.
    indicator: Option<std::sync::Arc<dyn Fn(&SharedString, RadioOptionState) -> gpui::AnyElement>>,
    /// The `<Description>` v3 composes inside a `<Radio>`, per option and in the
    /// same order. `.radio` is `flex flex-col gap-1` around its content and this
    /// text, indented under the label by `ps-7`.
    descriptions: Vec<Option<SharedString>>,
    /// The group's own `<Label>`, `<Description>` and `<FieldError>`. v3
    /// composes all three inside `<RadioGroup>` -- every documented example
    /// opens with `<Label>Plan selection</Label>` -- and a monolithic group takes
    /// them as props, the way `CheckboxGroup` does.
    label: Option<SharedString>,
    description: Option<SharedString>,
    error_message: Option<SharedString>,
    selected: Option<usize>,
    /// Whether `value` was supplied. `Option<usize>` cannot distinguish
    /// "controlled, nothing selected" from "uncontrolled" on its own.
    is_controlled: bool,
    default_value: Option<usize>,
    orientation: Orientation,
    is_disabled: bool,
    variant: FieldVariant,
    is_invalid: bool,
    is_required: bool,
    is_read_only: bool,
    on_change: Option<std::sync::Arc<dyn Fn(&SharedString, &mut Window, &mut App) + 'static>>,
}

impl RadioGroup {
    pub fn variant(mut self, variant: FieldVariant) -> Self {
        self.variant = variant;
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

    /// `isReadOnly` — the value is shown but cannot be changed.
    pub fn is_read_only(mut self, v: bool) -> Self {
        self.is_read_only = v;
        self
    }

    /// `Radio`'s root render function — handed the option's label and v3's
    /// field state: selected, disabled, read-only, invalid and required.
    pub fn option_content(
        mut self,
        render: impl Fn(&SharedString, RadioOptionState) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.option_content = Some(std::sync::Arc::new(render));
        self
    }

    /// `Radio.Indicator` — draws each option's indicator from its field state.
    pub fn indicator(
        mut self,
        render: impl Fn(&SharedString, RadioOptionState) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.indicator = Some(std::sync::Arc::new(render));
        self
    }

    /// The per-option descriptions, in the order the options were given. v3
    /// writes one `<Description>` inside each `<Radio>`; a monolithic group
    /// takes the column instead.
    pub fn descriptions<T: Into<SharedString>>(
        mut self,
        text: impl IntoIterator<Item = Option<T>>,
    ) -> Self {
        self.descriptions = text.into_iter().map(|opt| opt.map(Into::into)).collect();
        self
    }

    pub fn new(id: impl Into<gpui::ElementId>, options: Vec<RadioOption>) -> Self {
        Self {
            name: None,
            id: id.into(),
            options,
            form_state: Rc::new(RefCell::new(crate::form::LiveFormFieldState {
                value: crate::form::FormValue::Text(SharedString::default()),
                is_invalid: false,
                is_successful: true,
                focus: None,
                restore: None,
            })),
            option_content: None,
            indicator: None,
            descriptions: Vec::new(),
            label: None,
            description: None,
            error_message: None,
            selected: None,
            is_controlled: false,
            default_value: None,
            orientation: Orientation::Vertical,
            is_disabled: false,
            variant: FieldVariant::Primary,
            is_invalid: false,
            is_required: false,
            is_read_only: false,
            on_change: None,
        }
    }

    /// The `<Label>` v3 composes inside the group.
    pub fn label(mut self, text: impl Into<SharedString>) -> Self {
        self.label = Some(text.into());
        self
    }

    /// The `<Description>` v3 composes inside the group, below its options.
    pub fn description(mut self, text: impl Into<SharedString>) -> Self {
        self.description = Some(text.into());
        self
    }

    /// The `<FieldError>` v3 composes inside the group; supplying it also marks
    /// the group invalid, as every other field in this port does.
    pub fn error_message(mut self, text: impl Into<SharedString>) -> Self {
        self.error_message = Some(text.into());
        self
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
        {
            let mut state = self.form_state.borrow_mut();
            state.value = crate::form::FormValue::Text(
                selected
                    .and_then(|index| self.options.get(index))
                    .map(|option| option.value.clone())
                    .unwrap_or_default(),
            );
            state.is_invalid = self.is_invalid
                || self.error_message.is_some()
                || self
                    .options
                    .iter()
                    .any(|option| option.error_message.is_some());
            state.is_successful = !self.is_disabled;
        }
        Some(
            crate::form::FormField::live(name, self.form_state.clone())
                .is_required(self.is_required),
        )
    }

    /// `value` — the selected option's value. Supplying it makes the group controlled.
    pub fn value(mut self, value: impl AsRef<str>) -> Self {
        self.selected = self
            .options
            .iter()
            .position(|option| option.value == value.as_ref());
        self.form_state.borrow_mut().value = crate::form::FormValue::Text(
            self.selected
                .and_then(|index| self.options.get(index))
                .map(|option| option.value.clone())
                .unwrap_or_default(),
        );
        self.is_controlled = true;
        self
    }

    /// `defaultValue` — the uncontrolled initial selection.
    ///
    /// Only consulted when `value` is not supplied; the group then owns the
    /// selection and a press moves it.
    pub fn default_value(mut self, value: impl AsRef<str>) -> Self {
        self.default_value = self
            .options
            .iter()
            .position(|option| option.value == value.as_ref());
        if !self.is_controlled {
            self.form_state.borrow_mut().value = crate::form::FormValue::Text(
                self.default_value
                    .and_then(|index| self.options.get(index))
                    .map(|option| option.value.clone())
                    .unwrap_or_default(),
            );
        }
        self
    }

    pub fn orientation(mut self, o: Orientation) -> Self {
        self.orientation = o;
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    pub fn on_change(mut self, f: impl Fn(&SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(std::sync::Arc::new(f));
        self
    }
}

impl RenderOnce for RadioGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        let (selected, own) = crate::util::controlled(
            window,
            cx,
            gpui::ElementId::Name(format!("{:?}-value", self.id).into()),
            self.is_controlled.then_some(self.selected),
            self.default_value,
        );
        let is_invalid = self.is_invalid
            || self.error_message.is_some()
            || self
                .options
                .iter()
                .any(|option| option.error_message.is_some());
        let selected_value = selected
            .and_then(|index| self.options.get(index))
            .map(|option| option.value.clone())
            .unwrap_or_default();
        {
            let mut state = self.form_state.borrow_mut();
            state.value = crate::form::FormValue::Text(selected_value);
            state.is_invalid = is_invalid;
            state.is_successful = !self.is_disabled;
        }

        let reset_own = own.clone();
        let reset_state = self.form_state.clone();
        let reset_change = self.is_controlled.then(|| self.on_change.clone()).flatten();
        let reset_index = self.default_value;
        let reset_value = reset_index
            .and_then(|index| self.options.get(index))
            .map(|option| option.value.clone())
            .unwrap_or_default();
        self.form_state.borrow_mut().restore = (reset_own.is_some() || reset_change.is_some())
            .then(|| {
                crate::util::shared(move |window: &mut Window, cx: &mut App| {
                    reset_state.borrow_mut().value =
                        crate::form::FormValue::Text(reset_value.clone());
                    if let Some(held) = &reset_own {
                        held.update(cx, |selected, cx| {
                            *selected = reset_index;
                            cx.notify();
                        });
                    }
                    if let Some(on_change) = &reset_change {
                        on_change(&reset_value, window, cx);
                    }
                }) as std::sync::Arc<dyn Fn(&mut Window, &mut App)>
            });

        // *One* handle for the whole group, because a radio group is one tab
        // stop. Which row claims it is what moves: a roving tab stop cannot be
        // done by flipping a handle's `tab_stop`, since that is fixed where the
        // handle is made. `use_keyed_state` takes `cx` mutably, so it precedes
        // the theme.
        // Every per-option id is prefixed with this one rendering of the group
        // id; formatting `{id:?}` per option per frame was O(n) allocations.
        let id_prefix = element_id_name(&self.id);
        let group_focus = crate::util::tab_stop_handle(
            gpui::ElementId::Name(format!("{id_prefix}-focus").into()),
            window,
            cx,
        );
        self.form_state.borrow_mut().focus = Some(group_focus.clone());

        // A radio group is *one* tab stop: Tab moves past the whole group and
        // the arrows choose within it, which is the ARIA radio-group pattern
        // React Aria implements. The stop is the selected option, or the first
        // when nothing is selected yet.
        //
        // `Radio.isDisabled` options are left out of `stops` -- the list the
        // arrows and Home/End walk -- so the cursor never lands on one. The
        // tab stop skips them too: a stop resting on a disabled option has no
        // row to claim the group's handle (AGENTS.md's roving tab stop), which
        // would take the whole group out of the tab order. So with nothing
        // selected and the first option disabled, the group is still reachable
        // by Tab, on the first *enabled* option. With every option disabled
        // `stops` is empty, no row tracks the handle, and the group leaves the
        // tab order exactly as the group-wide `is_disabled` does.
        // `Arc` because every enabled option's key handler captures the whole
        // list: a plain clone per option was O(n^2) per frame.
        let stops: std::sync::Arc<Vec<usize>> = std::sync::Arc::new(
            (0..self.options.len())
                .filter(|i| !self.options[*i].is_disabled)
                .collect(),
        );
        let initial_focus_index = selected
            .filter(|i| stops.contains(i))
            .or_else(|| stops.first().copied())
            .unwrap_or(0);
        // Actual focus and selection diverge in a read-only group: React
        // Aria's arrow handler focuses the next input first, then its stately
        // setter rejects the value change. Keep that roving cursor separately
        // from `selected`, keyed by the component id so two groups cannot
        // share it.
        let cursor = window.use_keyed_state(
            gpui::ElementId::Name(format!("{id_prefix}-cursor").into()),
            cx,
            move |_, _| initial_focus_index,
        );
        let held_cursor = *cursor.read(cx);
        let cursor_index = if group_focus.is_focused(window) && stops.contains(&held_cursor) {
            held_cursor
        } else {
            initial_focus_index
        };

        // One hover/press slot per option. The row's press grows the selected
        // indicator from 6px to 8px.
        let interaction: Vec<crate::util::Interaction> = (0..self.options.len())
            .map(|i| {
                crate::util::interaction(
                    gpui::ElementId::Name(format!("{id_prefix}-opt-{i}-interaction").into()),
                    window,
                    cx,
                )
            })
            .collect();

        let sem = cx.role(Color::Accent);
        let colors = cx.colors();
        let layout = cx.layout();
        // `.radio__control` is `size-4 rounded-lg` — a rounded square, not a
        // circle — and `.radio__indicator` fills it at `rounded-lg` too.
        // The selected dot is the indicator scaled to `0.4286` of the 16px
        // control, which v3's own comment rounds to 6px. (8px is its *pressed*
        // size, `scale: 0.5714`.)
        let (circle, dot, text, gap) = (px(16.), px(6.), px(14.), px(12.));

        // `.radio-group` spaces its options with `mt-4` when vertical and
        // `gap-4` when horizontal — 16px either way.
        let mut group = match self.orientation {
            Orientation::Horizontal => gpui::div().flex().items_center().flex_wrap().gap(px(16.)),
            Orientation::Vertical => gpui::div().flex().flex_col().gap(px(16.)),
        };
        // `.radio-group--secondary` is not a panel: it only repaints the
        // *control* with `--default` and drops its shadow. This used to wrap the
        // whole group in a padded `surface_secondary` card, which v3 has no rule
        // for.
        let control_bg = match self.variant {
            FieldVariant::Primary => colors.field.background,
            FieldVariant::Secondary => colors.default.color,
        };
        let control_shadow = (self.variant == FieldVariant::Primary
            && !layout.field_shadow.is_empty())
        .then(|| layout.field_shadow.clone());

        let option_values: std::sync::Arc<Vec<SharedString>> = std::sync::Arc::new(
            self.options
                .iter()
                .map(|option| option.value.clone())
                .collect(),
        );
        for (i, option) in self.options.into_iter().enumerate() {
            let label = option.label;
            let value = option.value;
            let description = option
                .description
                .or_else(|| self.descriptions.get(i).and_then(|text| text.clone()));
            let error_message = option.error_message;
            let is_selected = selected == Some(i);
            let option_invalid =
                self.is_invalid || self.error_message.is_some() || error_message.is_some();
            // `Radio.isDisabled` — the option's own switch, beside the
            // group-wide `is_disabled`: dimmed (`status-disabled`'s opacity,
            // v3's "reduced opacity, no pointer events"), no pointer
            // affordance, no click handler and no place in the tab order or
            // the arrow navigation.
            let row_disabled = self.is_disabled || option.is_disabled;
            let (_, is_pressed) = interaction
                .get(i)
                .map(|slot| *slot.read(cx))
                .unwrap_or_default();
            let option_state = RadioOptionState {
                is_selected,
                is_disabled: row_disabled,
                is_read_only: self.is_read_only,
                is_invalid: option_invalid,
                is_required: self.is_required,
            };
            // `.radio__control` has no border (`--field-border-width: 0`); it is
            // a filled square. Unselected it is `bg-field` plus `shadow-field`;
            // selected it fills with `bg-accent` and the indicator shrinks to a
            // 6px `bg-accent-foreground` dot (`scale: 0.4286` of 16px).
            let mut circle_el = gpui::div()
                .id(gpui::ElementId::Name(
                    format!("{id_prefix}-opt-{i}-control").into(),
                ))
                .flex()
                .items_center()
                .justify_center()
                .size(circle)
                .rounded(crate::util::key_radius(cx))
                .flex_shrink_0()
                .bg(if is_selected { sem.color } else { control_bg })
                .when_some(control_shadow.clone(), |el, shadows| el.shadow(shadows));

            if let Some(render) = &self.indicator {
                circle_el = circle_el.child(render(&label, option_state));
            } else if is_selected {
                circle_el = circle_el.child(
                    gpui::div()
                        .size(if is_pressed { px(8.) } else { dot })
                        .rounded(crate::util::key_radius(cx))
                        .bg(sem.foreground),
                );
            }
            // `status-invalid-field` is a 1px danger outline over whatever the
            // control already paints — it does not replace the fill, and v3
            // applies it whether or not the option is selected.
            if option_invalid {
                circle_el = circle_el.border_1().border_color(colors.danger.color);
            }

            // v3 focuses the radio and rings `.radio__control`: the row takes the
            // focus, the control shows it.
            let focused = i == cursor_index
                && group_focus.is_focused(window)
                && crate::util::focus_visible(cx);
            let circle_el = crate::util::with_focus_ring(
                circle_el,
                focused && !row_disabled,
                true,
                control_shadow.clone().unwrap_or_default(),
                cx,
            );

            // `.radio__control[data-pressed]` is `scale-95`, and a checked one
            // also fills with `bg-accent-hover`. A disabled option cannot be
            // pressed, so it skips the animation like a read-only one.
            let circle_el = if row_disabled || self.is_read_only {
                circle_el
            } else {
                let pressed_fill = sem.hover();
                let circle_el = crate::anim::pressed(
                    circle_el,
                    crate::anim::PressBox {
                        height: circle,
                        padding_x: None,
                        width: Some(circle),
                        min_width: None,
                        text_size: text,
                        line_height: text,
                        gap: px(0.),
                        radius: crate::util::key_radius(cx),
                        shrink_x: true,
                        scale: crate::anim::PRESSED_SCALE_DEEP,
                    },
                    cx,
                );
                if is_selected {
                    circle_el.active(move |s| s.bg(pressed_fill))
                } else {
                    circle_el
                }
            };

            let mut row = gpui::div()
                .id(gpui::ElementId::Name(format!("{id_prefix}-opt-{i}").into()))
                .when(!row_disabled && i == cursor_index, |r| {
                    r.track_focus(&group_focus)
                })
                .flex()
                .items_center()
                .gap(gap)
                .text_size(text)
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(colors.foreground)
                .when(!row_disabled && !self.is_read_only, |r| r.cursor_pointer())
                .when(row_disabled, |r| r.opacity(layout.disabled_opacity))
                .child(circle_el)
                .child(match &self.option_content {
                    Some(render) => render(&label, option_state),
                    None => label.into_any_element(),
                });
            if !row_disabled && !self.is_read_only {
                if let Some(slot) = interaction.get(i) {
                    row = crate::util::track_interaction(row, slot);
                }
            }

            if !row_disabled {
                let on_change = self.on_change.clone();
                let own = own.clone();
                let read_only = self.is_read_only;
                // The arrows always take focus with them. In a mutable group
                // they also select; read-only keeps the cursor movement and
                // rejects only that second step, matching the pinned hooks.
                let key_change = on_change.clone();
                let key_own = own.clone();
                let key_stops = stops.clone();
                let key_cursor = cursor.clone();
                let key_values = option_values.clone();
                let key_form_state = self.form_state.clone();
                row = row.on_key_down(move |event, window, cx| {
                    let key = match event.keystroke.key.as_str() {
                        "down" | "right" => "down",
                        "up" | "left" => "up",
                        _ => return,
                    };
                    // `useRadioGroup` owns all four arrows, but has no Home or
                    // End shortcut. Leave those and every other key available
                    // to the enclosing surface.
                    cx.stop_propagation();
                    let crate::list_nav::Move::To(next) =
                        crate::list_nav::resolve(&key_stops, Some(i), key, true)
                    else {
                        return;
                    };
                    key_cursor.update(cx, |v, cx| {
                        *v = next;
                        cx.notify();
                    });
                    if !read_only {
                        if let Some(held) = &key_own {
                            key_form_state.borrow_mut().value =
                                crate::form::FormValue::Text(key_values[next].clone());
                            held.update(cx, |v, cx| {
                                *v = Some(next);
                                cx.notify();
                            });
                        }
                        if let Some(f) = &key_change {
                            f(&key_values[next], window, cx);
                        }
                    }
                });
                let click_cursor = cursor.clone();
                let click_focus = group_focus.clone();
                let click_form_state = self.form_state.clone();
                row = row.on_click(move |_, window, cx| {
                    window.focus(&click_focus, cx);
                    click_cursor.update(cx, |v, cx| {
                        *v = i;
                        cx.notify();
                    });
                    if !read_only {
                        // Uncontrolled: move our own selection, or pressing a
                        // radio would do nothing.
                        if let Some(held) = &own {
                            click_form_state.borrow_mut().value =
                                crate::form::FormValue::Text(value.clone());
                            held.update(cx, |v, cx| {
                                *v = Some(i);
                                cx.notify();
                            });
                        }
                        if let Some(f) = &on_change {
                            f(&value, window, cx);
                        }
                    }
                });
            }

            // `.radio` is `flex flex-col gap-1` around its content and the
            // description, which `ps-7` indents under the label -- the control
            // plus the content gap.
            match (error_message, description) {
                (Some(message), _) => {
                    group = group.child(
                        gpui::div().flex().flex_col().gap(px(4.)).child(row).child(
                            gpui::div()
                                .pl(px(28.))
                                .child(crate::field::ErrorMessage::new(message)),
                        ),
                    );
                }
                (None, Some(text)) => {
                    group = group.child(
                        gpui::div().flex().flex_col().gap(px(4.)).child(row).child(
                            gpui::div()
                                .pl(px(28.))
                                .child(crate::field::Description::new(text)),
                        ),
                    );
                }
                (None, None) => group = group.child(row),
            }
        }

        // `.radio` is `flex flex-col gap-1`, and the group's own label,
        // description and error are its siblings. v3 marks `isRequired` on the
        // Label rather than adding a line of its own, which is what
        // `field::Label` draws.
        let mut root = gpui::div().flex().flex_col().gap(px(4.));
        if let Some(label) = &self.label {
            root = root.child(
                crate::field::Label::new(label.clone())
                    .is_required(self.is_required)
                    .is_disabled(self.is_disabled)
                    .is_invalid(is_invalid),
            );
        }
        // v3's order, from its own examples: the group's `<Description>` sits
        // between the label and the options, and its `<FieldError>` after them.
        // (A *field*'s description is replaced by its error; `radio-group.css`
        // has no rule hiding this one, so both can show.)
        if let Some(description) = &self.description {
            root = root.child(crate::field::Description::new(description.clone()));
        }
        root = root.child(group);
        if is_invalid {
            if let Some(message) = &self.error_message {
                root = root.child(crate::field::ErrorMessage::new(message.clone()));
            }
        }
        root
    }
}

fn element_id_name(id: &gpui::ElementId) -> String {
    format!("{id:?}").trim_matches('"').to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Render wraps the walk list and the value list in `Arc` so every enabled
    /// option's key handler clones the pointer, not the Vec.
    #[test]
    fn option_key_handlers_share_walk_and_value_lists() {
        let source = include_str!("radio_group.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        assert!(
            source.contains("let stops: std::sync::Arc<Vec<usize>> = std::sync::Arc::new("),
            "the walk list must be one Arc shared by every enabled option"
        );
        assert!(
            source.contains(
                "let option_values: std::sync::Arc<Vec<SharedString>> = std::sync::Arc::new("
            ),
            "the value list must be one Arc shared by every enabled option"
        );
        assert!(
            source.contains("let key_stops = stops.clone();"),
            "each enabled option must clone the shared walk list"
        );
        assert!(
            source.contains("let key_values = option_values.clone();"),
            "each enabled option must clone the shared value list"
        );

        // The clones above are `Arc::clone`: two handles to one allocation.
        let stops: std::sync::Arc<Vec<usize>> = std::sync::Arc::new(vec![0, 1, 2]);
        let values: std::sync::Arc<Vec<SharedString>> =
            std::sync::Arc::new(vec![SharedString::from("v0")]);
        let walk = std::sync::Arc::clone(&stops);
        let list = std::sync::Arc::clone(&values);
        assert!(
            std::sync::Arc::ptr_eq(&walk, &stops) && std::sync::Arc::ptr_eq(&list, &values),
            "Arc clones of the walk and value lists must share pointer identity"
        );
    }
}
