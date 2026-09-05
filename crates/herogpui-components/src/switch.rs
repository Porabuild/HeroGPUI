//! Switch — port of `@heroui/switch`.

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

use gpui::{
    prelude::*, px, Animation, AnimationExt, AnyElement, App, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window,
};
use herogpui_core::{Color, Size};
use herogpui_theme::ActiveTheme;

/// State handed to Switch's children render function.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SwitchState {
    pub is_selected: bool,
    pub is_hovered: bool,
    pub is_pressed: bool,
    pub is_focused: bool,
    pub is_focus_visible: bool,
    pub is_disabled: bool,
    pub is_read_only: bool,
    pub is_invalid: bool,
    pub is_required: bool,
}

/// `.switch__thumb` transitions its margin for 300ms with
/// `--ease-out-fluid`. The current fraction lives outside the animation
/// element so reversing mid-flight starts from the rendered position rather
/// than jumping back to the previous endpoint.
const THUMB_TRANSITION_MS: u64 = 300;
/// `.switch__control` transitions its background color for 250ms with
/// `--ease-smooth`. The changing animation id belongs to the listener-free
/// fill child, so the track keeps its stable interaction path.
const TRACK_TRANSITION_MS: u64 = 250;

#[derive(Clone)]
struct ThumbMotion {
    selected: bool,
    generation: usize,
    from: f32,
    position: Rc<Cell<f32>>,
}

struct ThumbMotionFrame {
    generation: usize,
    from: f32,
    to: f32,
    position: Rc<Cell<f32>>,
    animate: bool,
}

impl ThumbMotionFrame {
    fn render(self, thumb: gpui::Div, travel: gpui::Pixels) -> AnyElement {
        if !self.animate {
            self.position.set(self.to);
            return thumb.ml(travel * self.to).into_any_element();
        }

        let position = self.position;
        let from = self.from;
        let to = self.to;
        thumb
            .with_animation(
                gpui::ElementId::Name(format!("switch-thumb-slide-{}", self.generation).into()),
                Animation::new(Duration::from_millis(THUMB_TRANSITION_MS))
                    .with_easing(|t| crate::anim::Curve::OutFluid.at(t)),
                move |thumb, delta| {
                    let fraction = from + (to - from) * delta;
                    position.set(fraction);
                    thumb.ml(travel * fraction)
                },
            )
            .into_any_element()
    }
}

#[derive(Clone)]
struct TrackMotion {
    target: gpui::Hsla,
    generation: usize,
    from: gpui::Hsla,
    color: Rc<Cell<gpui::Hsla>>,
}

struct TrackMotionFrame {
    generation: usize,
    from: gpui::Hsla,
    to: gpui::Hsla,
    color: Rc<Cell<gpui::Hsla>>,
    animate: bool,
}

impl TrackMotionFrame {
    fn render(self, fill: gpui::Div) -> AnyElement {
        if !self.animate {
            self.color.set(self.to);
            return fill.bg(self.to).into_any_element();
        }

        let color = self.color;
        let from = self.from;
        let to = self.to;
        fill.with_animation(
            gpui::ElementId::Name(format!("switch-track-background-{}", self.generation).into()),
            Animation::new(Duration::from_millis(TRACK_TRANSITION_MS))
                .with_easing(|t| crate::anim::Curve::Smooth.at(t)),
            move |fill, delta| {
                let next = herogpui_core::mix_oklab(from, to, delta);
                color.set(next);
                fill.bg(next)
            },
        )
        .into_any_element()
    }
}

fn thumb_motion(
    id: &gpui::ElementId,
    selected: bool,
    window: &mut Window,
    cx: &mut App,
) -> ThumbMotionFrame {
    let state = window.use_keyed_state(
        gpui::ElementId::Name(format!("{id:?}-thumb-motion").into()),
        cx,
        |_, _| ThumbMotion {
            selected,
            generation: 0,
            from: if selected { 1.0 } else { 0.0 },
            position: Rc::new(Cell::new(if selected { 1.0 } else { 0.0 })),
        },
    );
    let mut current = state.read(cx).clone();
    let to = if selected { 1.0 } else { 0.0 };
    if current.selected != selected {
        current.selected = selected;
        current.generation = current.generation.wrapping_add(1);
        current.from = current.position.get();
        state.update(cx, |stored, _| *stored = current.clone());
    }
    if ActiveTheme::reduce_motion(cx) && (current.position.get() - to).abs() > f32::EPSILON {
        current.from = to;
        current.position.set(to);
        state.update(cx, |stored, _| *stored = current.clone());
    }
    ThumbMotionFrame {
        generation: current.generation,
        from: current.from,
        to,
        position: current.position,
        animate: current.generation != 0
            && !ActiveTheme::reduce_motion(cx)
            && (current.from - to).abs() > f32::EPSILON,
    }
}

fn track_motion(
    id: &gpui::ElementId,
    target: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> TrackMotionFrame {
    let state = window.use_keyed_state(
        gpui::ElementId::Name(format!("{id:?}-track-motion").into()),
        cx,
        |_, _| TrackMotion {
            target,
            generation: 0,
            from: target,
            color: Rc::new(Cell::new(target)),
        },
    );
    let mut current = state.read(cx).clone();
    if current.target != target {
        current.target = target;
        current.generation = current.generation.wrapping_add(1);
        current.from = current.color.get();
        state.update(cx, |stored, _| *stored = current.clone());
    }
    let reduce_motion = ActiveTheme::reduce_motion(cx);
    if reduce_motion && current.color.get() != target {
        current.from = target;
        current.color.set(target);
        state.update(cx, |stored, _| *stored = current.clone());
    }
    let animate = !reduce_motion && current.generation != 0 && current.color.get() != target;
    TrackMotionFrame {
        generation: current.generation,
        from: current.from,
        to: target,
        color: current.color,
        animate,
    }
}

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
    /// v3's `children`-as-a-function: handed the interactive state and drawn in
    /// place of the label.
    content: Option<std::sync::Arc<dyn Fn(SwitchState) -> AnyElement + 'static>>,
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
    form_state: Rc<RefCell<crate::form::LiveFormFieldState>>,
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
            content: None,
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
            form_state: Rc::new(RefCell::new(crate::form::LiveFormFieldState {
                value: crate::form::FormValue::Flag(false),
                is_invalid: false,
                is_successful: true,
                focus: None,
                restore: None,
            })),
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
        let validity = crate::validation::resolve(
            self.is_invalid,
            &self.validation_errors,
            self.validate.as_ref().and_then(|f| f(&checked)),
            None,
        );
        {
            let mut state = self.form_state.borrow_mut();
            state.value = match (&self.value, checked) {
                (Some(value), true) => crate::form::FormValue::Text(value.clone()),
                _ => crate::form::FormValue::Flag(checked),
            };
            state.is_invalid = validity.is_invalid;
            state.is_successful = !self.is_disabled;
        }
        Some(
            crate::form::FormField::live(name, self.form_state.clone())
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
    /// v3's render function for a switch's children, handed its complete field
    /// state. Hover and press are a frame behind the pointer because gpui
    /// reports them to a handler.
    pub fn content(mut self, render: impl Fn(SwitchState) -> AnyElement + 'static) -> Self {
        self.content = Some(std::sync::Arc::new(render));
        self
    }

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
        let reset_own = own.clone();
        let reset_state = self.form_state.clone();
        let reset_value = self.value.clone();
        let reset_change = self
            .checked
            .is_some()
            .then(|| self.on_change.clone())
            .flatten();
        self.form_state.borrow_mut().restore = (reset_own.is_some() || reset_change.is_some())
            .then(|| {
                let default_checked = self.default_checked;
                crate::util::shared(move |window: &mut Window, cx: &mut App| {
                    reset_state.borrow_mut().value = match (&reset_value, default_checked) {
                        (Some(value), true) => crate::form::FormValue::Text(value.clone()),
                        _ => crate::form::FormValue::Flag(default_checked),
                    };
                    if let Some(held) = &reset_own {
                        held.update(cx, |checked, cx| {
                            *checked = default_checked;
                            cx.notify();
                        });
                    }
                    if let Some(on_change) = &reset_change {
                        on_change(default_checked, window, cx);
                    }
                }) as std::sync::Arc<dyn Fn(&mut Window, &mut App)>
            });

        // The keyboard's focus target. `use_keyed_state` takes `cx` mutably, so
        // it precedes every borrow of the theme.
        let focus_handle = crate::util::tab_stop_handle(
            gpui::ElementId::Name(format!("{:?}-focus", self.id).into()),
            window,
            cx,
        );
        self.form_state.borrow_mut().focus = Some(focus_handle.clone());
        // The track's background transition and an optional `content` closure
        // read the same one-frame-late hover/press state.
        let interaction = crate::util::interaction(
            gpui::ElementId::Name(format!("{:?}-interaction", self.id).into()),
            window,
            cx,
        );
        let thumb_motion = thumb_motion(&self.id, checked, window, cx);

        // v3 order: the controlled flag, then server errors, then `validate`.
        let validity = crate::validation::resolve(
            self.is_invalid,
            &self.validation_errors,
            self.validate.as_ref().and_then(|f| f(&checked)),
            None,
        );
        {
            let mut state = self.form_state.borrow_mut();
            state.value = match (&self.value, checked) {
                (Some(value), true) => crate::form::FormValue::Text(value.clone()),
                _ => crate::form::FormValue::Flag(checked),
            };
            state.is_invalid = validity.is_invalid;
            state.is_successful = !self.is_disabled;
        }

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
        let thumb_travel = w - thumb_w - thumb_inset * 2.;

        let (
            accent_color,
            accent_hover,
            accent_foreground,
            default_color,
            default_hover,
            default_foreground,
            danger_color,
        ) = {
            let sem = cx.role(Color::Accent);
            let colors = cx.colors();
            (
                sem.color,
                sem.hover(),
                sem.foreground,
                colors.default.color,
                colors.default.hover(),
                colors.default.foreground,
                colors.danger.color,
            )
        };

        // `default` is the v3 unchecked track. A soft (alpha) mix vanishes on
        // a white overlay, so the track uses the solid role colour.
        let track_bg = if checked { accent_color } else { default_color };
        let hover_bg = if checked { accent_hover } else { default_hover };
        let interaction_state = *interaction.read(cx);
        let track_target = if !self.is_disabled
            && !self.is_read_only
            && (interaction_state.0 || interaction_state.1)
        {
            hover_bg
        } else {
            track_bg
        };
        let track_motion_frame = track_motion(&self.id, track_target, window, cx);
        let layout = cx.layout();

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
            .when(!self.is_disabled, |t| t.cursor_pointer());
        if !self.is_disabled {
            track = crate::util::track_interaction(track, &interaction);
        }
        // v3's switch stylesheet has no invalid rule at all -- the state shows
        // in the field error below, not as a danger ring on the track, so the
        // ring this used to draw was an invention.

        track = track
            .child(track_motion_frame.render(gpui::div().absolute().inset_0().rounded(track_r)));

        // Thumb sits at the end when checked, start when unchecked. v3 moves it
        // by margin rather than transform; `ThumbMotionFrame` animates that
        // margin and preserves its current fraction across a reversal.
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
        let thumb_el = if checked {
            thumb_el
                .bg(accent_foreground)
                .when(self.is_disabled, |thumb| thumb.opacity(0.4))
                // The checked thumb carries its own three-layer shadow.
                .shadow(vec![
                    gpui::BoxShadow {
                        color: gpui::black().alpha(0.02),
                        offset: gpui::point(px(0.), px(0.)),
                        blur_radius: px(5.),
                        spread_radius: px(0.),
                        inset: false,
                    },
                    gpui::BoxShadow {
                        color: gpui::black().alpha(0.06),
                        offset: gpui::point(px(0.), px(2.)),
                        blur_radius: px(10.),
                        spread_radius: px(0.),
                        inset: false,
                    },
                    gpui::BoxShadow {
                        color: gpui::black().alpha(0.3),
                        offset: gpui::point(px(0.), px(0.)),
                        blur_radius: px(1.),
                        spread_radius: px(0.),
                        inset: false,
                    },
                ])
        } else {
            thumb_el
                .bg(if self.is_disabled {
                    default_foreground.alpha(0.2)
                } else {
                    herogpui_theme::white()
                })
                .when(!layout.field_shadow.is_empty(), |t| {
                    t.shadow(layout.field_shadow.clone())
                })
        };
        track = track.child(thumb_motion.render(thumb_el, thumb_travel));

        if !self.is_disabled && !self.is_read_only && (self.on_change.is_some() || own.is_some()) {
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
            .text_size(px(14.))
            .line_height(px(20.))
            .font_weight(gpui::FontWeight::MEDIUM);
        let content_row = self.content.clone().map(|render| {
            let (is_hovered, is_pressed) = *interaction.read(cx);
            let focused = focus_handle.is_focused(window);
            render(SwitchState {
                is_hovered,
                is_pressed,
                is_focused: focused,
                is_focus_visible: focused && crate::util::focus_visible(cx),
                is_selected: checked,
                is_disabled: self.is_disabled,
                is_read_only: self.is_read_only,
                is_invalid: validity.is_invalid,
                is_required: self.is_required,
            })
        });
        let label_row = self.label.map(|label| {
            gpui::div()
                .flex()
                .items_center()
                // `.switch__label` is `text-base`, a step larger than the
                // content around it.
                .text_size(px(16.))
                // Tailwind pairs that size with a 24px leading; without the
                // pair the label inherited the shell's 20px and read tighter
                // than the `.switch__content` beside it.
                .line_height(px(24.))
                .gap(px(4.))
                .child(label)
                .when(self.is_required, |r| {
                    r.child(gpui::div().text_color(danger_color).child("*"))
                })
        });
        match (self.label_first, label_row, content_row) {
            // A `content` closure stands in for the label wherever the label
            // would have gone.
            (true, _, Some(content)) => el = el.child(content).child(track),
            (false, _, Some(content)) => el = el.child(track).child(content),
            (true, Some(label), None) => el = el.child(label).child(track),
            (false, Some(label), None) => el = el.child(track).child(label),
            (_, None, None) => el = el.child(track),
        }

        // Description and FieldError are direct siblings of Switch.Content.
        // Both use the size-specific track width plus the 12px content gap.
        let indent = w + px(12.);
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .when(self.is_disabled, |root| {
                root.opacity(layout.disabled_opacity)
            })
            .child(el)
            .when_some(self.description, |root, description| {
                root.child(
                    gpui::div()
                        .pl(indent)
                        .child(crate::field::Description::new(description)),
                )
            })
            .when_some(validity.first(), |root, message| {
                root.child(
                    gpui::div()
                        .pl(indent)
                        .child(crate::field::ErrorMessage::new(message)),
                )
            })
            .into_any_element()
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
