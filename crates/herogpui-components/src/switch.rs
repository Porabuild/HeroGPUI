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
    id: gpui::ElementId,
    /// `isSelected` — `None` leaves the component holding the state, seeded
    /// from `defaultSelected`.
    checked: Option<bool>,
    default_checked: bool,
    color: Color,
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
    on_change: Option<Box<dyn Fn(bool, &mut Window, &mut App) + 'static>>,
}

impl Switch {
    /// `value` — the v3 name for [`Switch::checked`].
    pub fn value(self, v: bool) -> Self {
        self.checked(v)
    }

    /// `onPress` — the v3 name for [`Switch::on_change`], which already
    /// reports the next state.
    pub fn on_press(self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change(handler)
    }

    /// `isSelected` — the v3 name for [`Switch::checked`].
    pub fn is_selected(self, v: bool) -> Self {
        self.checked(v)
    }

    /// `validate` — returns the message to show, or `None` when the state is fine.
    ///
    /// The component runs it and surfaces the result, so a caller does not have
    /// to mirror the logic into `is_invalid`.
    pub fn validate(
        mut self,
        f: impl Fn(&bool) -> Option<gpui::SharedString> + 'static,
    ) -> Self {
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
            id: id.into(),
            checked: None,
            default_checked: false,
            color: Color::Accent,
            size: Size::Md,
            is_disabled: false,
            is_invalid: false,
            validate: None,
            validation_errors: Vec::new(),
            is_required: false,
            is_read_only: false,
            label: None,
            on_change: None,
        }
    }

    /// Controlled checked state.
    pub fn checked(mut self, v: bool) -> Self {
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

    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
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

    pub fn on_change(
        mut self,
        f: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Box::new(f));
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

        let sem = cx.role(self.color);
        let colors = cx.colors();
        let layout = cx.layout();

        // v3 order: the controlled flag, then server errors, then `validate`.
        let validity = crate::validation::resolve(
            self.is_invalid,
            &self.validation_errors,
            self.validate.as_ref().and_then(|f| f(&checked)),
            None,
        );
        let validity_invalid = validity.is_invalid;

        // HeroUI switch dims: md 40x24 thumb 16; sm 32x20 thumb 12; lg 48x28 thumb 20
        let (w, h, thumb) = match self.size {
            Size::Sm => (px(32.), px(20.), px(12.)),
            Size::Md => (px(40.), px(24.), px(16.)),
            Size::Lg => (px(48.), px(28.), px(20.)),
        };

        // `default` is the v3 unchecked track. A soft (alpha) mix vanishes on
        // a white overlay, so the track uses the solid role colour.
        let track_bg = if checked {
            sem.color
        } else {
            colors.default.color
        };

        let mut track = gpui::div()
            .id(self.id.clone())
            .relative()
            .w(w)
            .h(h)
            .rounded_full()
            .bg(track_bg)
            .flex()
            .items_center()
            .px((h - thumb) / 2.)
            .when(self.is_disabled, |t| t.opacity(layout.disabled_opacity))
            .when(!self.is_disabled, |t| t.cursor_pointer());

        if validity_invalid {
            track = track.border_2().border_color(colors.danger.color);
        }

        if !self.is_disabled && !self.is_read_only {
            let hover_bg = if checked {
                sem.hover()
            } else {
                colors.default.hover()
            };
            track = track.hover(move |s| s.bg(hover_bg));
        }

        // Thumb sits at the end when checked, start when unchecked.
        track = track.child(if checked {
            gpui::div()
                .ml_auto()
                .size(thumb)
                .rounded_full()
                .bg(colors.background)
                .shadow(vec![gpui::BoxShadow {
                    color: gpui::black().alpha(0.25),
                    offset: gpui::point(px(0.), px(1.)),
                    blur_radius: px(3.),
                    spread_radius: px(0.),
                }])
                .flex_shrink_0()
        } else {
            gpui::div()
                .mr_auto()
                .size(thumb)
                .rounded_full()
                .bg(colors.default.soft())
                .shadow(vec![gpui::BoxShadow {
                    color: gpui::black().alpha(0.25),
                    offset: gpui::point(px(0.), px(1.)),
                    blur_radius: px(3.),
                    spread_radius: px(0.),
                }])
                .flex_shrink_0()
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

        let mut el = gpui::div().flex().items_center().gap(px(8.)).child(track);
        if let Some(label) = self.label {
            el = el.child(label);
            if self.is_required {
                el = el.child(
                    gpui::div()
                        .text_color(colors.danger.color)
                        .child("*"),
                );
            }
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

