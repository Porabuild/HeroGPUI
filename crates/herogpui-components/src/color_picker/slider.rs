//! ColorSlider.

use super::*;

// ColorSlider
// ---------------------------------------------------------------------------

/// State handed to `ColorSlider.Thumb`'s render function.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ColorSliderThumbState {
    /// React Aria's ColorThumb render color excludes the alpha channel.
    pub color: PickerColor,
    pub is_dragging: bool,
    pub is_hovered: bool,
    pub is_focused: bool,
    pub is_focus_visible: bool,
    pub is_disabled: bool,
}

const COLOR_SLIDER_TRACK_INSET_PX: f32 = 10.0;

/// ColorSlider — adjusts a single channel along a gradient track.
#[derive(IntoElement)]
pub struct ColorSlider {
    /// `name` — the name this control submits under; read back by
    /// [`Self::form_field`].
    name: Option<SharedString>,
    /// `defaultValue` — set it to hand this component its own state.
    default_value: Option<PickerColor>,
    id: ElementId,
    value: PickerColor,
    channel: ColorChannel,
    /// `colorSpace` — only saturation differs between HSB and HSL, so this
    /// picks which one a saturation slider edits.
    color_space: ColorSpace,
    orientation: herogpui_core::Orientation,
    length: Pixels,
    show_label: bool,
    /// `ColorSlider.Output`'s render props: the closure is handed the current
    /// `color` and the formatted channel value.
    output: Option<Arc<dyn Fn(PickerColor, &str) -> gpui::AnyElement + 'static>>,
    thumb: Option<Arc<dyn Fn(ColorSliderThumbState) -> gpui::AnyElement + 'static>>,
    is_disabled: bool,
    on_change: Option<OnColorChange>,
    on_change_end: Option<OnColorChange>,
    form_state: Rc<RefCell<crate::form::LiveFormFieldState>>,
}

impl ColorSlider {
    pub fn new(id: impl Into<ElementId>, value: PickerColor, channel: ColorChannel) -> Self {
        Self {
            name: None,
            default_value: None,
            id: id.into(),
            value,
            channel,
            color_space: ColorSpace::default(),
            orientation: herogpui_core::Orientation::Horizontal,
            length: px(240.),
            show_label: true,
            output: None,
            thumb: None,
            is_disabled: false,
            on_change: None,
            on_change_end: None,
            form_state: live_color_form_state(crate::form::FormValue::Number(0.0)),
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
    /// # use herogpui_components::{ColorChannel, ColorSlider, Form, PickerColor};
    /// # struct Demo;
    /// # impl Render for Demo {
    /// #     fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    /// #         let form = Form::new();
    /// #         let color = PickerColor::hsb(210., 0.8, 0.9);
    /// #         let control = ColorSlider::new("hue", color, ColorChannel::Hue).name("hue");
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
        let value = self.default_value.unwrap_or(self.value);
        // A disabled range input is not a successful control in HTML, so the
        // field stays registered and is omitted from FormData.
        sync_color_form_state(
            &self.form_state,
            color_slider_form_value(value, self.channel, self.color_space),
            !self.is_disabled,
            false,
        );
        Some(crate::form::FormField::live(name, self.form_state.clone()).is_required(false))
    }

    /// `defaultValue` — the uncontrolled initial colour.
    ///
    /// Supplying it hands the component its own state: the constructor's
    /// `value` becomes the seed, and a change moves the component's copy.
    pub fn default_value(mut self, value: PickerColor) -> Self {
        self.default_value = Some(value);
        self
    }

    /// `orientation` — a vertical slider runs bottom to top.
    /// `colorSpace` — the space the channel is read in. Defaults to HSB, the
    /// space [`PickerColor`] stores.
    pub fn color_space(mut self, space: ColorSpace) -> Self {
        self.color_space = space;
        self
    }

    pub fn orientation(mut self, orientation: herogpui_core::Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// `onChangeEnd` — fires once when the drag finishes.
    pub fn on_change_end(
        mut self,
        handler: impl Fn(&PickerColor, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change_end = Some(Arc::new(handler));
        self
    }

    pub fn length(mut self, length: impl Into<Pixels>) -> Self {
        self.length = length.into();
        self
    }

    /// `ColorSlider.Output`'s render function — v3 hands it the `color`, which
    /// is what this closure takes along with the value as v3 formats it.
    pub fn output(
        mut self,
        render: impl Fn(PickerColor, &str) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.output = Some(Arc::new(render));
        self
    }

    /// `ColorSlider.Thumb`'s render function — the closure receives the shared
    /// ColorThumb interaction state while the built-in thumb retains its
    /// positioning, focus and drag behavior.
    pub fn thumb(
        mut self,
        render: impl Fn(ColorSliderThumbState) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.thumb = Some(Arc::new(render));
        self
    }

    pub fn show_label(mut self, v: bool) -> Self {
        self.show_label = v;
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(&PickerColor, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Arc::new(handler));
        self
    }

    /// The two ends of this channel's gradient, given the current color.
    fn gradient_ends(&self) -> (Hsla, Hsla) {
        let (min, max) = self.channel.range();
        (
            self.value
                .with_channel_in(self.channel, self.color_space, min)
                .to_hsla(),
            self.value
                .with_channel_in(self.channel, self.color_space, max)
                .to_hsla(),
        )
    }
}

pub(super) fn three_stop_gradient(
    vertical: bool,
    start: Hsla,
    middle: Hsla,
    end: Hsla,
) -> gpui::Div {
    if vertical {
        div()
            .absolute()
            .inset_0()
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .h(gpui::relative(0.5))
                    .bg(gpui::linear_gradient(
                        0.0,
                        gpui::linear_color_stop(middle, 0.0),
                        gpui::linear_color_stop(end, 1.0),
                    )),
            )
            .child(
                div()
                    .absolute()
                    .bottom_0()
                    .left_0()
                    .right_0()
                    .h(gpui::relative(0.5))
                    .bg(gpui::linear_gradient(
                        0.0,
                        gpui::linear_color_stop(start, 0.0),
                        gpui::linear_color_stop(middle, 1.0),
                    )),
            )
    } else {
        div()
            .absolute()
            .inset_0()
            .child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left_0()
                    .w(gpui::relative(0.5))
                    .bg(gpui::linear_gradient(
                        90.0,
                        gpui::linear_color_stop(start, 0.0),
                        gpui::linear_color_stop(middle, 1.0),
                    )),
            )
            .child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .right_0()
                    .w(gpui::relative(0.5))
                    .bg(gpui::linear_gradient(
                        90.0,
                        gpui::linear_color_stop(middle, 0.0),
                        gpui::linear_color_stop(end, 1.0),
                    )),
            )
    }
}

impl RenderOnce for ColorSlider {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `defaultValue` opts into the component holding its own colour;
        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        let (resolved, own) = util::controlled(
            window,
            cx,
            element_id::scoped(&self.id, "slider-value"),
            match self.default_value {
                Some(_) => None,
                None => Some(self.value),
            },
            self.default_value.unwrap_or(self.value),
        );
        self.value = resolved;
        let form_default = window.use_keyed_state(
            element_id::scoped(&self.id, "slider-form-default"),
            cx,
            |_, _| None::<PickerColor>,
        );
        if form_default.read(cx).is_none() {
            let initial = self.value;
            form_default.update(cx, |slot, cx| {
                *slot = Some(initial);
                cx.notify();
            });
        }
        let restore_default = form_default.read(cx).unwrap_or(self.value);
        // Submit the resolved colour's channel. Uncontrolled keyed state is
        // current after interaction; a controlled owner must accept the change
        // before the next render writes it here.
        sync_color_form_state(
            &self.form_state,
            color_slider_form_value(self.value, self.channel, self.color_space),
            !self.is_disabled,
            false,
        );
        let restore_own = own.clone();
        let restore_on_change = self.on_change.clone();
        let restore_form_state = self.form_state.clone();
        let restore_channel = self.channel;
        let restore_space = self.color_space;
        let restore_is_disabled = self.is_disabled;
        let restore: Arc<dyn Fn(&mut Window, &mut App)> = util::shared(move |window, cx| {
            if let Some(own) = &restore_own {
                own.update(cx, |current, cx| {
                    *current = restore_default;
                    cx.notify();
                });
            }
            if let Some(callback) = &restore_on_change {
                callback(&restore_default, window, cx);
            }
            sync_color_form_state(
                &restore_form_state,
                color_slider_form_value(restore_default, restore_channel, restore_space),
                !restore_is_disabled,
                false,
            );
        });
        self.form_state.borrow_mut().restore = Some(restore);
        // The handle the keys arrive on. `use_keyed_state` takes `cx` mutably, so
        // it precedes the theme tokens.
        let focus_handle =
            window.use_keyed_state(element_id::scoped(&self.id, "slider-focus"), cx, |_, cx| {
                cx.focus_handle().tab_stop(true)
            });
        let focus_handle = focus_handle.read(cx).clone();
        self.form_state.borrow_mut().focus = Some(focus_handle.clone());
        let bounds_slot =
            window.use_keyed_state(element_id::scoped(&self.id, "slider-bounds"), cx, |_, _| {
                Bounds::<f32> {
                    origin: gpui::point(0., 0.),
                    size: gpui::size(0., 0.),
                }
            });
        let dragging = window.use_keyed_state(
            element_id::scoped(&self.id, "slider-dragging"),
            cx,
            |_, _| false,
        );
        let thumb_hovered = window.use_keyed_state(
            element_id::scoped(&self.id, "slider-thumb-hovered"),
            cx,
            |_, _| false,
        );
        let colors = cx.colors();
        let border_width = f32::from(cx.layout().border_width);
        let (min, max) = self.channel.range();
        // Read in the requested space: HSL and HSB saturation are different
        // numbers for the same colour.
        let raw = self.value.channel_in(self.channel, self.color_space);
        let norm = ((raw - min) / (max - min)).clamp(0.0, 1.0);
        // `.color-slider__track` is `relative rounded-2xl` with the gradient
        // inside it; `.color-slider__output` is the value read-out above.
        let track_h = px(20.);

        let vertical = !self.orientation.is_horizontal();
        let mut track = div()
            .id(self.id.clone())
            .relative()
            .rounded(px(COLOR_SLIDER_TRACK_INSET_PX))
            .border(cx.layout().border_width)
            .border_color(colors.border);
        track = if vertical {
            track.w(track_h).h(self.length)
        } else {
            track.w(self.length).h(track_h)
        };

        let recorder_bounds = bounds_slot.clone();
        track = track.child(
            gpui::canvas(
                move |bounds: Bounds<Pixels>, _, cx| {
                    recorder_bounds.update(cx, |slot, _| {
                        *slot = Bounds {
                            origin: gpui::point(
                                f32::from(bounds.origin.x) - border_width,
                                f32::from(bounds.origin.y) - border_width,
                            ),
                            size: gpui::size(
                                f32::from(bounds.size.width) + border_width * 2.0,
                                f32::from(bounds.size.height) + border_width * 2.0,
                            ),
                        };
                    });
                    bounds
                },
                |_, _, _, _| {},
            )
            .absolute()
            .inset_0(),
        );

        // Hue needs the full spectrum, and lightness needs a midpoint so its
        // hue survives between black and white.
        track = if self.channel == ColorChannel::Lightness {
            let (start, middle, end) =
                lightness_gradient_colors(self.value, self.color_space, min, max);
            track.child(
                div()
                    .absolute()
                    .inset_0()
                    .rounded(px(COLOR_SLIDER_TRACK_INSET_PX))
                    .overflow_hidden()
                    .child(three_stop_gradient(vertical, start, middle, end)),
            )
        } else if self.channel == ColorChannel::Hue {
            let mut spectrum = div()
                .absolute()
                .inset_0()
                .rounded(px(COLOR_SLIDER_TRACK_INSET_PX))
                .overflow_hidden();
            // Six 60-degree bands approximate the continuous hue wheel.
            for i in 0..6 {
                let from = PickerColor::hsb(i as f32 * 60.0, 1.0, 1.0).to_hsla();
                let to = PickerColor::hsb((i as f32 + 1.0) * 60.0, 1.0, 1.0).to_hsla();
                spectrum = if vertical {
                    spectrum.child(
                        div()
                            .absolute()
                            .left_0()
                            .right_0()
                            .top(px(f32::from(self.length) * ((5 - i) as f32 / 6.0)))
                            .h(px(f32::from(self.length) / 6.0 + 1.0))
                            .bg(gpui::linear_gradient(
                                0.0,
                                gpui::linear_color_stop(from, 0.0),
                                gpui::linear_color_stop(to, 1.0),
                            )),
                    )
                } else {
                    spectrum.child(
                        div()
                            .absolute()
                            .top_0()
                            .bottom_0()
                            .left(px(f32::from(self.length) * (i as f32 / 6.0)))
                            .w(px(f32::from(self.length) / 6.0 + 1.0))
                            .bg(gpui::linear_gradient(
                                90.0,
                                gpui::linear_color_stop(from, 0.0),
                                gpui::linear_color_stop(to, 1.0),
                            )),
                    )
                };
            }
            track.child(spectrum)
        } else {
            let (from, to) = self.gradient_ends();
            track.bg(gpui::linear_gradient(
                if vertical { 0.0 } else { 90.0 },
                gpui::linear_color_stop(from, 0.0),
                gpui::linear_color_stop(to, 1.0),
            ))
        };

        // A vertical slider's zero end is at the bottom, so the offset is
        // measured from the far edge.
        let travel = (f32::from(self.length) - COLOR_SLIDER_TRACK_INSET_PX * 2.0).max(0.0);
        let thumb_offset = px(COLOR_SLIDER_TRACK_INSET_PX
            + travel * if vertical { 1.0 - norm } else { norm }
            - 8.0);
        let is_dragging = !self.is_disabled && *dragging.read(cx);
        let is_focused = !self.is_disabled && focus_handle.is_focused(window);
        let is_focus_visible = is_focused && util::focus_visible(cx);
        let thumb_state = ColorSliderThumbState {
            color: self.value.with_alpha(1.0),
            is_dragging,
            is_hovered: !self.is_disabled && *thumb_hovered.read(cx),
            is_focused,
            is_focus_visible,
            is_disabled: self.is_disabled,
        };
        let thumb_content = self.thumb.as_ref().map(|render| render(thumb_state));
        let mut thumb = util::with_focus_ring(
            div()
                .id(element_id::scoped(&self.id, "slider-thumb"))
                .absolute()
                .when(vertical, |t| t.left(px(2.)).top(thumb_offset))
                .when(!vertical, |t| t.top(px(2.)).left(thumb_offset))
                // `.color-slider__thumb` is `size-4`.
                .size(px(16.))
                .rounded(px(16.))
                .border(px(3.))
                .border_color(gpui::white())
                .bg(if self.is_disabled {
                    colors.default.color
                } else {
                    self.value.with_alpha(1.0).to_hsla()
                })
                .when_some(thumb_content, |thumb, content| thumb.child(content)),
            is_focus_visible,
            true,
            Vec::new(),
            cx,
        );
        if !self.is_disabled {
            let hovered = thumb_hovered;
            thumb = thumb.on_hover(move |is_hovered, _, cx| {
                hovered.update(cx, |value, cx| {
                    if *value != *is_hovered {
                        *value = *is_hovered;
                        cx.notify();
                    }
                });
            });
        }
        track = track.child(thumb);

        if self.is_disabled {
            track = track.opacity(cx.layout().disabled_opacity);
        } else {
            let value = self.value;
            let channel = self.channel;
            let space = self.color_space;
            track = track.cursor_pointer();
            // v3: the arrows step the channel, Home and End take it to its ends,
            // and Page Up/Down move by a tenth of the range -- React Aria's page
            // step. A colour slider with no keyboard is not the same control.
            let keys_value = self.value;
            let on_change_keys = self.on_change.clone();
            let end_keys = self.on_change_end.clone();
            let own_keys = own.clone();
            // One step per unit for a 0-360 hue or an 0-255 byte, and a
            // percentage point for the normalised channels.
            let step = if max - min > 2.0 { 1.0 } else { 0.01 };
            let page = ((max - min) / 10.0).max(step);
            track = track
                .track_focus(&focus_handle)
                .key_context("ColorSlider")
                .on_key_down(move |event, window, cx| {
                    let current = keys_value.channel_in(channel, space);
                    let next = match event.keystroke.key.as_str() {
                        "right" | "up" => current + step,
                        "left" | "down" => current - step,
                        "pageup" => current + page,
                        "pagedown" => current - page,
                        "home" => min,
                        "end" => max,
                        _ => return,
                    };
                    let next = keys_value.with_channel_in(channel, space, next.clamp(min, max));
                    if next == keys_value {
                        return;
                    }
                    if let Some(held) = &own_keys {
                        held.update(cx, |v, cx| {
                            *v = next;
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &on_change_keys {
                        cb(&next, window, cx);
                    }
                    // A keystroke is a finished change, so `onChangeEnd` fires
                    // with it rather than waiting for a release.
                    if let Some(cb) = &end_keys {
                        cb(&next, window, cx);
                    }
                });
            if self.on_change.is_some() || self.on_change_end.is_some() || own.is_some() {
                let down_bounds = bounds_slot.clone();
                let down_dragging = dragging.clone();
                let down_change = self.on_change.clone();
                let down_own = own.clone();
                let focus_for_press = focus_handle;
                track = track.on_mouse_down(
                    gpui::MouseButton::Left,
                    move |event: &MouseDownEvent, window, cx| {
                        if event.modifiers.alt
                            || event.modifiers.control
                            || event.modifiers.platform
                        {
                            return;
                        }
                        down_dragging.update(cx, |value, cx| {
                            if !*value {
                                *value = true;
                                cx.notify();
                            }
                        });
                        window.focus(&focus_for_press, cx);
                        if let Some(next) = slider_color_from_pointer(
                            &down_bounds,
                            event.position,
                            vertical,
                            value,
                            channel,
                            space,
                            cx,
                        ) {
                            if next.changed {
                                report_color_change(
                                    next.value,
                                    &down_own,
                                    &down_change,
                                    window,
                                    cx,
                                );
                            }
                        }
                    },
                );

                let global_bounds = bounds_slot;
                let global_dragging = dragging;
                let global_change = self.on_change;
                let global_end = self.on_change_end;
                let global_own = own;
                track = track.child(
                    gpui::canvas(
                        |bounds, _, _| bounds,
                        move |_, _, window, _| {
                            let move_bounds = global_bounds.clone();
                            let move_dragging = global_dragging.clone();
                            let move_change = global_change.clone();
                            let move_own = global_own.clone();
                            window.on_mouse_event(
                                move |event: &MouseMoveEvent, phase, window, cx| {
                                    if phase == gpui::DispatchPhase::Capture
                                        && event.pressed_button == Some(gpui::MouseButton::Left)
                                        && *move_dragging.read(cx)
                                    {
                                        if let Some(next) = slider_color_from_pointer(
                                            &move_bounds,
                                            event.position,
                                            vertical,
                                            value,
                                            channel,
                                            space,
                                            cx,
                                        ) {
                                            if next.changed {
                                                report_color_change(
                                                    next.value,
                                                    &move_own,
                                                    &move_change,
                                                    window,
                                                    cx,
                                                );
                                            }
                                        }
                                    }
                                },
                            );

                            let up_bounds = global_bounds.clone();
                            let up_dragging = global_dragging.clone();
                            let up_end = global_end.clone();
                            window.on_mouse_event(
                                move |event: &MouseUpEvent, phase, window, cx| {
                                    if phase == gpui::DispatchPhase::Capture
                                        && event.button == gpui::MouseButton::Left
                                    {
                                        finish_slider_drag(
                                            &up_dragging,
                                            &up_bounds,
                                            event.position,
                                            vertical,
                                            value,
                                            channel,
                                            space,
                                            &up_end,
                                            window,
                                            cx,
                                        );
                                    }
                                },
                            );
                        },
                    )
                    .absolute()
                    .inset_0(),
                );
            }
        }

        // `useColorSlider` is `useSlider` with one thumb: `role: 'slider'`
        // on the interactive track, named by the channel.
        track = track
            .a11y_named(a11y::Role::Slider, &a11y::Name::maybe(self.name.clone()))
            .a11y_orientation(self.orientation);

        if !self.show_label {
            return div().child(track);
        }

        let display = match self.channel {
            ColorChannel::Hue => format!("{}\u{00B0}", raw.round()),
            ColorChannel::Red | ColorChannel::Green | ColorChannel::Blue => {
                format!("{}", raw.round())
            }
            _ => format!("{}%", (raw * 100.0).round()),
        };
        let output = match &self.output {
            // `.color-slider__output`: v3's render prop is handed the colour,
            // which is what a caller needs to draw a swatch or a different
            // unit.
            Some(render) => render(self.value, &display),
            None => div()
                .text_color(colors.muted)
                .child(display)
                .into_any_element(),
        };

        div()
            .flex()
            .flex_col()
            // `.color-slider` is `grid w-full gap-1`.
            .gap(px(4.))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .w(self.length)
                    .text_size(px(14.))
                    .line_height(px(20.))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child(
                        div()
                            .text_color(colors.foreground)
                            .child(self.channel.label()),
                    )
                    // The disabled root dims Output through status-disabled,
                    // while the stylesheet restores Label to full opacity.
                    .child(
                        div()
                            .when(self.is_disabled, |output| {
                                output.opacity(cx.layout().disabled_opacity)
                            })
                            .child(output),
                    ),
            )
            .child(track)
    }
}

pub(super) fn lightness_gradient_colors(
    value: PickerColor,
    color_space: ColorSpace,
    min: f32,
    max: f32,
) -> (Hsla, Hsla, Hsla) {
    (
        value
            .with_channel_in(ColorChannel::Lightness, color_space, min)
            .to_hsla(),
        value
            .with_channel_in(ColorChannel::Lightness, color_space, (max - min) / 2.0)
            .to_hsla(),
        value
            .with_channel_in(ColorChannel::Lightness, color_space, max)
            .to_hsla(),
    )
}

#[allow(clippy::float_cmp)] // snapped channel values are exact state coordinates
pub(super) fn slider_color_from_pointer(
    bounds: &Entity<Bounds<f32>>,
    position: gpui::Point<Pixels>,
    vertical: bool,
    value: PickerColor,
    channel: ColorChannel,
    color_space: ColorSpace,
    cx: &App,
) -> Option<PointerColor> {
    let bounds = *bounds.read(cx);
    let (reach, extent) = if vertical {
        (
            bounds.origin.y + bounds.size.height
                - COLOR_SLIDER_TRACK_INSET_PX
                - f32::from(position.y),
            bounds.size.height - COLOR_SLIDER_TRACK_INSET_PX * 2.0,
        )
    } else {
        (
            f32::from(position.x) - bounds.origin.x - COLOR_SLIDER_TRACK_INSET_PX,
            bounds.size.width - COLOR_SLIDER_TRACK_INSET_PX * 2.0,
        )
    };
    if extent <= 0.0 {
        return None;
    }
    let fraction = (reach / extent).clamp(0.0, 1.0);
    let (min, max) = channel.range();
    let next = snap_color_channel(channel, min + fraction * (max - min));
    Some(PointerColor {
        changed: next != value.channel_in(channel, color_space),
        value: value.with_channel_in(channel, color_space, next),
    })
}

#[allow(clippy::too_many_arguments)] // the channel and callback are the color-slider drag state
pub(super) fn finish_slider_drag(
    dragging: &Entity<bool>,
    bounds: &Entity<Bounds<f32>>,
    position: gpui::Point<Pixels>,
    vertical: bool,
    value: PickerColor,
    channel: ColorChannel,
    color_space: ColorSpace,
    on_change_end: &Option<OnColorChange>,
    window: &mut Window,
    cx: &mut App,
) {
    if !*dragging.read(cx) {
        return;
    }
    dragging.update(cx, |value, cx| {
        *value = false;
        cx.notify();
    });
    if let (Some(callback), Some(next)) = (
        on_change_end,
        slider_color_from_pointer(bounds, position, vertical, value, channel, color_space, cx),
    ) {
        callback(&next.value, window, cx);
    }
}

// ---------------------------------------------------------------------------
