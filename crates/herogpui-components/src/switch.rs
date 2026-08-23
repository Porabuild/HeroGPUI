//! Switch — port of `@heroui/switch`.

use gpui::{
    prelude::*, px, AnyElement, App, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window,
};
use herogpui_core::{Color, Size};
use herogpui_theme::ActiveTheme;

/// HeroUI Switch (`<Switch>`).
#[derive(IntoElement)]
pub struct Switch {
    /// `value` — what this control submits when checked. HTML's default is
    /// `"on"`.
    value: Option<gpui::SharedString>,
    /// `validationBehavior` — carried on this control's form field.
    validation_behavior: crate::form::ValidationBehavior,
    /// `name` — the name this control submits under; read back by
    /// [`Self::form_field`].
    name: Option<gpui::SharedString>,
    id: gpui::ElementId,
    /// `isSelected` — `None` leaves the component holding the state, seeded
    /// from `defaultSelected`.
    checked: Option<bool>,
    default_checked: bool,
    size: Size,
    is_disabled: bool,
    is_invalid: bool,
    /// `validate` — run by the component, not the caller.
    validate: Option<crate::validation::Validator<bool>>,
    /// `validationErrors` — messages from a server round-trip.
    validation_errors: Vec<gpui::SharedString>,
    is_required: bool,
    is_read_only: bool,
    label: Option<AnyElement>,
    /// `Description` — v3 composes it as a sibling of the button row, indented
    /// to sit under the label.
    description: Option<gpui::SharedString>,
    /// `Switch.Thumb` children — v3 draws an icon inside the thumb, one per
    /// state (`.switch__thumb > *` is a centred, full-size box).
    thumb_off: Option<AnyElement>,
    thumb_on: Option<AnyElement>,
    /// Whether the label comes before the control. v3 gets this from the order
    /// of `Switch.Content`'s children.
    label_first: bool,
    /// `Arc` rather than `Box`: the handler is bound twice, once for the
    /// pointer and once for Enter and Space.
    on_change: Option<std::sync::Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>>,
}

impl Switch {
    /// `onPress` — the v3 name for [`Switch::on_change`], which already
    /// reports the next state.
    pub fn on_press(self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change(handler)
    }

    /// `validate` — returns the message to show, or `None` when the state is fine.
    ///
    /// The component runs it and surfaces the result, so a caller does not have
    /// to mirror the logic into `is_invalid`.
    pub fn validate(mut self, f: impl Fn(&bool) -> Option<gpui::SharedString> + 'static) -> Self {
        self.validate = Some(std::sync::Arc::new(f));
        self
    }

    /// `validationErrors` — messages produced elsewhere, shown ahead of
    /// whatever `validate` returns.
    pub fn validation_errors(
        mut self,
        errors: impl IntoIterator<Item = impl Into<gpui::SharedString>>,
    ) -> Self {
        self.validation_errors = errors.into_iter().map(Into::into).collect();
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

    pub fn is_read_only(mut self, v: bool) -> Self {
        self.is_read_only = v;
        self
    }

    pub fn new(id: impl Into<gpui::ElementId>) -> Self {
        Self {
            value: None,
            validation_behavior: crate::form::ValidationBehavior::Native,
            name: None,
            id: id.into(),
            checked: None,
            default_checked: false,
            size: Size::Md,
            is_disabled: false,
            is_invalid: false,
            validate: None,
            validation_errors: Vec::new(),
            is_required: false,
            is_read_only: false,
            label: None,
            description: None,
            thumb_off: None,
            thumb_on: None,
            label_first: false,
            on_change: None,
        }
    }

    /// `value` — what this control submits when checked.
    ///
    /// An HTML checkbox submits `"on"` unless told otherwise; this is that
    /// override, and it is read by [`Self::form_field`].
    pub fn value(mut self, value: impl Into<gpui::SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// `validationBehavior` — `Allow` shows the message without blocking form
    /// submission. Carried on the [`Self::form_field`] this control produces.
    pub fn validation_behavior(mut self, behavior: crate::form::ValidationBehavior) -> Self {
        self.validation_behavior = behavior;
        self
    }

    /// `name` — the name this control submits under.
    pub fn name(mut self, name: impl Into<gpui::SharedString>) -> Self {
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
        let checked = self.checked.unwrap_or(self.default_checked);
        let field = match (&self.value, checked) {
            // A checked box with an explicit `value` submits that text.
            (Some(v), true) => crate::form::FormField::text_value(name, v.clone()),
            _ => crate::form::FormField::flag(name, checked),
        };
        Some(
            field
                .is_required(self.is_required)
                .validation_behavior(self.validation_behavior),
        )
    }

    /// Controlled checked state.
    /// `isSelected` — the controlled state; `None` leaves the component
    /// holding it, seeded from `defaultSelected`.
    pub fn is_selected(mut self, v: bool) -> Self {
        self.checked = Some(v);
        self
    }

    /// `defaultSelected` — the uncontrolled initial state.
    ///
    /// Only consulted when `checked` is not supplied; the switch then owns the
    /// state and toggles itself.
    pub fn default_selected(mut self, v: bool) -> Self {
        self.default_checked = v;
        self
    }

    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    /// Text shown next to the track (children slot in React).
    pub fn label(mut self, el: impl IntoElement) -> Self {
        self.label = Some(el.into_any_element());
        self
    }

    /// `Description` — help text under the control and label.
    pub fn description(mut self, text: impl Into<gpui::SharedString>) -> Self {
        self.description = Some(text.into());
        self
    }

    /// `Switch.Thumb` children — what the thumb shows in each state.
    ///
    /// v3 composes an icon inside the thumb and swaps it on selection, which is
    /// its "With Icons" example.
    pub fn thumb_icons(mut self, off: impl IntoElement, on: impl IntoElement) -> Self {
        self.thumb_off = Some(off.into_any_element());
        self.thumb_on = Some(on.into_any_element());
        self
    }

    /// Puts the label before the control, which v3 does by ordering the
    /// children of `Switch.Content`.
    pub fn label_first(mut self, v: bool) -> Self {
        self.label_first = v;
        self
    }

    pub fn on_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(std::sync::Arc::new(f));
        self
    }
}

impl RenderOnce for Switch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        let (checked, own) = crate::util::controlled(
            window,
            cx,
            gpui::ElementId::Name(format!("{:?}-checked", self.id).into()),
            self.checked,
            self.default_checked,
        );

        // The keyboard's focus target. `use_keyed_state` takes `cx` mutably, so
        // it precedes every borrow of the theme.
        let focus_handle = crate::util::tab_stop_handle(
            gpui::ElementId::Name(format!("{:?}-focus", self.id).into()),
            window,
            cx,
        );
        let sem = cx.role(Color::Accent);
        let colors = cx.colors();
        let layout = cx.layout();

        // v3 order: the controlled flag, then server errors, then `validate`.
        let validity = crate::validation::resolve(
            self.is_invalid,
            &self.validation_errors,
            self.validate.as_ref().and_then(|f| f(&checked)),
            None,
        );

        // `.switch__control` and `.switch__thumb` state their sizes in `rem`,
        // so at a 16px root: track 32x16 / 40x20 / 48x24, and a thumb that is a
        // rounded *rectangle* 1.375x as wide as it is tall -- 16.5x12 / 22x16 /
        // 27.5x20 -- inset `ms-0.5` (2px) at each end. The track radii are
        // `rounded-lg` for `sm` and `rounded-xl` above it; the thumb's are
        // `rounded-md` / `rounded-lg` / `rounded-xl`.
        let (w, h, thumb_w, thumb_h, track_r, thumb_r) = match self.size {
            Size::Sm => (px(32.), px(16.), px(16.5), px(12.), px(8.), px(6.)),
            Size::Md => (px(40.), px(20.), px(22.), px(16.), px(12.), px(8.)),
            Size::Lg => (px(48.), px(24.), px(27.5), px(20.), px(12.), px(12.)),
        };
        let thumb_inset = px(2.);

        // `default` is the v3 unchecked track. A soft (alpha) mix vanishes on
        // a white overlay, so the track uses the solid role colour.
        let track_bg = if checked {
            sem.color
        } else {
            colors.default.color
        };

        let mut track = gpui::div()
            .id(self.id.clone())
            .when(!self.is_disabled, |el| el.track_focus(&focus_handle))
            .relative()
            .w(w)
            .h(h)
            .rounded(track_r)
            .bg(track_bg)
            .flex()
            .items_center()
            .px(thumb_inset)
            .when(self.is_disabled, |t| t.opacity(layout.disabled_opacity))
            .when(!self.is_disabled, |t| t.cursor_pointer());
        // v3's switch stylesheet has no invalid rule at all -- the state shows
        // in the field error below, not as a danger ring on the track, so the
        // ring this used to draw was an invention.

        if !self.is_disabled && !self.is_read_only {
            let hover_bg = if checked {
                sem.hover()
            } else {
                colors.default.hover()
            };
            // `--switch-control-bg-pressed` *is* `--switch-control-bg-hover`, and
            // a checked switch presses to `--accent-hover`, which is what its
            // hover already resolves to.
            track = track
                .hover(move |s| s.bg(hover_bg))
                .active(move |s| s.bg(hover_bg));
        }

        // Thumb sits at the end when checked, start when unchecked. v3 moves it
        // by margin rather than transform, which is what `ml_auto`/`mr_auto` do
        // here. It is `bg-white` unchecked and `bg-accent-foreground` checked --
        // not the page background and a soft grey, which is what this drew
        // before.
        let mut thumb_el = gpui::div()
            .w(thumb_w)
            .h(thumb_h)
            .rounded(thumb_r)
            .flex_shrink_0()
            // `.switch__thumb > *` is a centred, full-size box.
            .flex()
            .items_center()
            .justify_center();
        if let Some(glyph) = if checked {
            self.thumb_on
        } else {
            self.thumb_off
        } {
            thumb_el = thumb_el.child(glyph);
        }
        track = track.child(if checked {
            thumb_el
                .ml_auto()
                .bg(sem.foreground)
                // The checked thumb carries its own three-layer shadow.
                .shadow(vec![
                    gpui::BoxShadow {
                        color: gpui::black().alpha(0.02),
                        offset: gpui::point(px(0.), px(0.)),
                        blur_radius: px(5.),
                        spread_radius: px(0.),
                    },
                    gpui::BoxShadow {
                        color: gpui::black().alpha(0.06),
                        offset: gpui::point(px(0.), px(2.)),
                        blur_radius: px(10.),
                        spread_radius: px(0.),
                    },
                    gpui::BoxShadow {
                        color: gpui::black().alpha(0.3),
                        offset: gpui::point(px(0.), px(0.)),
                        blur_radius: px(1.),
                        spread_radius: px(0.),
                    },
                ])
        } else {
            thumb_el
                .mr_auto()
                .bg(herogpui_theme::white())
                .when(!layout.field_shadow.is_empty(), |t| {
                    t.shadow(layout.field_shadow.clone())
                })
        });

        if !self.is_disabled && (self.on_change.is_some() || own.is_some()) {
            let on_change = self.on_change;
            track = track.on_click(move |_, window, cx| {
                // Uncontrolled: flip our own copy, or nothing could change it.
                if let Some(held) = &own {
                    held.update(cx, |v, cx| {
                        *v = !checked;
                        cx.notify();
                    });
                }
                if let Some(cb) = &on_change {
                    cb(!checked, window, cx);
                }
            });
        }

        if !self.is_disabled {
            track =
                crate::util::ring_if_focused(track, &focus_handle, true, Vec::new(), window, cx);
        }

        // `.switch__content` is `gap-3`. v3 gets the label's side from the order
        // of its children, so `label_first` puts it before the control.
        let mut el = gpui::div()
            .flex()
            .items_center()
            .gap(px(12.))
            .text_size(px(14.));
        let label_row = self.label.map(|label| {
            gpui::div()
                .flex()
                .items_center()
                // `.switch__label` is `text-base`, a step larger than the
                // content around it.
                .text_size(px(16.))
                .gap(px(4.))
                .child(label)
                .when(self.is_required, |r| {
                    r.child(gpui::div().text_color(colors.danger.color).child("*"))
                })
        });
        match (self.label_first, label_row) {
            (true, Some(label)) => el = el.child(label).child(track),
            (false, Some(label)) => el = el.child(track).child(label),
            (_, None) => el = el.child(track),
        }

        // `& > [data-slot="description"]` is indented by the track width plus
        // the row's `gap-3`, so the text lines up under the label.
        if let Some(description) = self.description {
            let indent = w + px(12.);
            return gpui::div()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(el)
                .child(
                    gpui::div()
                        .pl(indent)
                        .child(crate::field::Description::new(description)),
                )
                .when_some(validity.first(), |c, message| {
                    c.child(
                        gpui::div()
                            .pl(indent)
                            .child(crate::field::ErrorMessage::new(message)),
                    )
                })
                .into_any_element();
        }

        // A switch that can be invalid has to be able to say why, so the row
        // becomes a column when there is a message.
        match validity.first() {
            None => el.into_any_element(),
            Some(message) => gpui::div()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(el)
                .child(crate::field::ErrorMessage::new(message))
                .into_any_element(),
        }
    }
}

/// `SwitchGroup` — the layout v3 wraps a set of switches in.
///
/// `.switch-group` is `flex flex-col gap-6` around a `.switch-group__items`
/// that is `flex gap-4`, and the orientation modifier is what turns that inner
/// row into a column. The outer gap is for the label and description a caller
/// puts beside the items.
#[derive(IntoElement)]
pub struct SwitchGroup {
    orientation: herogpui_core::Orientation,
    items: Vec<AnyElement>,
}

impl SwitchGroup {
    pub fn new() -> Self {
        Self {
            // v3 documents `vertical` as the default.
            orientation: herogpui_core::Orientation::Vertical,
            items: Vec::new(),
        }
    }

    pub fn orientation(mut self, orientation: herogpui_core::Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn child(mut self, el: impl IntoElement) -> Self {
        self.items.push(el.into_any_element());
        self
    }
}

impl Default for SwitchGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for SwitchGroup {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.items.extend(elements);
    }
}

impl RenderOnce for SwitchGroup {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let vertical = self.orientation == herogpui_core::Orientation::Vertical;
        gpui::div().flex().flex_col().gap(px(24.)).child(
            gpui::div()
                .flex()
                .map(|el| {
                    if vertical {
                        el.flex_col()
                    } else {
                        el.flex_row()
                    }
                })
                .gap(px(16.))
                .children(self.items),
        )
    }
}
