//! ColorSwatchPicker.

use super::*;

// ColorSwatchPicker
// ---------------------------------------------------------------------------

/// Layout of a [`ColorSwatchPicker`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SwatchLayout {
    #[default]
    Grid,
    Stack,
}

/// State handed to `ColorSwatchPicker.Item`'s render function.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ColorSwatchPickerItemState {
    pub color: PickerColor,
    pub is_hovered: bool,
    pub is_pressed: bool,
    pub is_selected: bool,
    pub is_focused: bool,
    pub is_focus_visible: bool,
    pub is_disabled: bool,
}

/// ColorSwatchPicker — chooses from a predefined palette.
#[derive(IntoElement)]
pub struct ColorSwatchPicker {
    /// `defaultValue` — set it to hand this component its own state.
    default_value: Option<PickerColor>,
    id: ElementId,
    swatches: Vec<PickerColor>,
    value: Option<PickerColor>,
    size: SizeXl,
    shape: SwatchShape,
    layout: SwatchLayout,
    is_disabled: bool,
    /// `ColorSwatchPicker.Item.isDisabled` — swatches that cannot be chosen,
    /// by index.
    disabled_keys: std::collections::HashSet<usize>,
    item_content:
        Option<Arc<dyn Fn(usize, ColorSwatchPickerItemState) -> gpui::AnyElement + 'static>>,
    indicator: Option<Arc<dyn Fn(usize, ColorSwatchPickerItemState) -> gpui::AnyElement + 'static>>,
    on_change: Option<OnColorChange>,
}

impl ColorSwatchPicker {
    pub fn new(id: impl Into<ElementId>, swatches: Vec<PickerColor>) -> Self {
        Self {
            default_value: None,
            id: id.into(),
            swatches,
            value: None,
            // `.color-swatch` is `size-8` (32px), which is `SizeXl::Md` on v3's
            // own swatch scale (16/24/32/36/40).
            size: SizeXl::Md,
            shape: SwatchShape::Circle,
            layout: SwatchLayout::Grid,
            is_disabled: false,
            disabled_keys: std::collections::HashSet::new(),
            item_content: None,
            indicator: None,
            on_change: None,
        }
    }

    /// `defaultValue` — the uncontrolled initial colour.
    ///
    /// Supplying it hands the component its own state: the constructor's
    /// `value` becomes the seed, and a change moves the component's copy.
    pub fn default_value(mut self, value: PickerColor) -> Self {
        self.default_value = Some(value);
        self
    }

    pub fn value(mut self, value: PickerColor) -> Self {
        self.value = Some(value);
        self
    }

    pub fn size(mut self, size: SizeXl) -> Self {
        self.size = size;
        self
    }

    pub fn shape(mut self, shape: SwatchShape) -> Self {
        self.shape = shape;
        self
    }

    pub fn layout(mut self, layout: SwatchLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    /// `ColorSwatchPicker.Item.isDisabled` — swatches that cannot be chosen,
    /// by index.
    ///
    /// A disabled swatch draws dimmed (`status-disabled`'s reduced opacity),
    /// answers no click, and leaves the tab order, exactly as a disabled
    /// control must in this port. The dictionary is by palette position — the
    /// same projection `RadioGroup::disabled_keys` gives `Radio.isDisabled` —
    /// since the swatches are a plain list with no keys of their own.
    pub fn disabled_keys(mut self, keys: impl IntoIterator<Item = usize>) -> Self {
        self.disabled_keys = keys.into_iter().collect();
        self
    }

    /// `children` on `ColorSwatchPicker.Item` — replaces the built-in swatch
    /// and indicator while the stable item keeps selection and navigation.
    /// The closure receives the item's palette index and the complete pinned
    /// React Aria item render state.
    pub fn item_content(
        mut self,
        render: impl Fn(usize, ColorSwatchPickerItemState) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.item_content = Some(Arc::new(render));
        self
    }

    /// `ColorSwatchPicker.Indicator` — replaces the selected checkmark. The
    /// render function receives the same item state as its pinned compound
    /// part, including while the indicator is visually hidden.
    pub fn indicator(
        mut self,
        render: impl Fn(usize, ColorSwatchPickerItemState) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.indicator = Some(Arc::new(render));
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(&PickerColor, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for ColorSwatchPicker {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `defaultValue` opts into the component holding its own selection;
        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        let (resolved, own) = util::controlled(
            window,
            cx,
            element_id::scoped(&self.id, "swatches-value"),
            match self.default_value {
                Some(_) => None,
                None => Some(self.value),
            },
            self.default_value.or(self.value),
        );
        self.value = resolved;

        // One tab stop for the whole list: v3's picker sits on React Aria's
        // ListBox, which roves a listbox's tabindex, so Tab enters the picker
        // once and the arrows move inside it (AGENTS.md's roving tab stop).
        // Which swatch claims the group's handle is held in keyed state,
        // because a handle's `tab_stop` is fixed where the handle is made;
        // the cursor never rests on a disabled swatch, so a stop stranded on
        // one cannot take the picker out of the tab order. The state precedes
        // the theme borrow.
        let swatch_focus = util::tab_stop_handle(element_id::scoped(&self.id, "focus"), window, cx);
        let cursor =
            window.use_keyed_state(element_id::scoped(&self.id, "cursor"), cx, |_, _| 0usize);
        let enabled: Vec<usize> = (0..self.swatches.len())
            .filter(|index| !(self.is_disabled || self.disabled_keys.contains(index)))
            .collect();
        let at = *cursor.read(cx);
        let cursor_index = enabled
            .iter()
            .copied()
            .find(|i| *i >= at)
            .or_else(|| enabled.first().copied());
        let interactions: Vec<util::Interaction> =
            if self.item_content.is_some() || self.indicator.is_some() {
                (0..self.swatches.len())
                    .map(|index| {
                        util::interaction(
                            element_id::scoped(
                                &element_id::indexed(&self.id, "swatch", index),
                                "interaction",
                            ),
                            window,
                            cx,
                        )
                    })
                    .collect()
            } else {
                Vec::new()
            };
        let pointer_focus =
            window.use_keyed_state(element_id::scoped(&self.id, "pointer-focus"), cx, |_, _| {
                None::<usize>
            });
        let item_edge = self.size.swatch_px();
        let border_width = match self.size {
            SizeXl::Xs => px(1.),
            SizeXl::Sm | SizeXl::Md => px(2.),
            SizeXl::Lg | SizeXl::Xl => px(3.),
        };
        let item_radius = match self.shape {
            SwatchShape::Circle => match self.size {
                SizeXl::Xs => px(8.),
                SizeXl::Sm => px(12.),
                SizeXl::Md => px(16.),
                SizeXl::Lg | SizeXl::Xl => px(24.),
            },
            SwatchShape::Square => match self.size {
                SizeXl::Xs => px(6.),
                SizeXl::Sm => px(8.),
                SizeXl::Md | SizeXl::Lg | SizeXl::Xl => px(12.),
            },
        };
        // RAC's grid delegate moves Up and Down between the same column of the
        // adjacent row. A grid row holds the most cells of the active size
        // that fit under the picker's 280px maximum with its 8px gap.
        let grid_columns = if self.layout == SwatchLayout::Grid {
            (((280. + 8.) / (f32::from(item_edge) + 8.)) as usize).max(1)
        } else {
            0
        };
        let swatch_ring = util::focus_visible(cx);

        let clear_pointer_focus = pointer_focus.clone();
        let mut row = div()
            .id(element_id::scoped(&self.id, "swatches"))
            .flex()
            .items_center()
            .gap(px(8.))
            .on_mouse_down_out(move |_, _, cx| {
                clear_pointer_focus.update(cx, |focused, cx| {
                    if focused.take().is_some() {
                        cx.notify();
                    }
                });
            });
        row = match self.layout {
            SwatchLayout::Grid => row.flex_row().flex_wrap().max_w(px(280.)),
            SwatchLayout::Stack => row.flex_col(),
        };

        for (index, swatch) in self.swatches.iter().enumerate() {
            let selected = self.value.is_some_and(|v| v.to_hex() == swatch.to_hex());
            // `ColorSwatchPicker.Item.isDisabled` — the item's own flag
            // beside the group-wide one: dimmed, no press, and out of the tab
            // order (`track_focus` below is gated on it, and a stop resting
            // on nothing is what takes a control out of the order).
            let item_disabled = self.is_disabled || self.disabled_keys.contains(&index);
            let (recorded_hover, recorded_press) = interactions
                .get(index)
                .map(|slot| *slot.read(cx))
                .unwrap_or_default();
            let item_focused = !item_disabled
                && cursor_index == Some(index)
                && (swatch_focus.is_focused(window)
                    || *pointer_focus.read(cx) == Some(index)
                    || recorded_press);
            let state = ColorSwatchPickerItemState {
                color: *swatch,
                is_hovered: !item_disabled && recorded_hover,
                is_pressed: !item_disabled && recorded_press,
                is_selected: selected,
                is_focused: item_focused,
                is_focus_visible: item_focused && swatch_ring,
                is_disabled: item_disabled,
            };

            let mut cell = div()
                .id(element_id::indexed(&self.id, "swatch", index))
                .when(cursor_index == Some(index), |c| {
                    c.track_focus(&swatch_focus)
                })
                .relative()
                .flex()
                .items_center()
                .justify_center()
                .size(item_edge)
                .rounded(item_radius)
                .border(border_width);

            if let Some(render) = &self.item_content {
                cell = cell.child(render(index, state));
            } else {
                cell = cell.child({
                    // `.color-swatch-picker__swatch` is `size-full` inside the
                    // border, `scale(1.1)` on hover and `scale(0.77)` when the
                    // item is selected. gpui has no div transform, so each of
                    // those is the size it comes to.
                    let base_edge = f32::from(item_edge) - 2. * f32::from(border_width);
                    let edge = px(if selected {
                        base_edge * 0.77
                    } else {
                        base_edge
                    });
                    let radius = match (self.shape, self.size, selected) {
                        (SwatchShape::Circle, _, _) => item_radius,
                        (SwatchShape::Square, SizeXl::Xs, _) => px(6.),
                        (SwatchShape::Square, SizeXl::Sm, true) => px(6.),
                        (SwatchShape::Square, _, _) => px(8.),
                    };
                    let grown = px(f32::from(edge) * 1.1);
                    div()
                        .size(edge)
                        .rounded(radius)
                        .flex_shrink_0()
                        .overflow_hidden()
                        // The checkerboard that shows through a translucent
                        // colour, as on a plain `ColorSwatch`.
                        .bg(cx.colors().surface_secondary)
                        .when(!item_disabled && !selected, |el| {
                            el.hover(move |st| st.size(grown))
                        })
                        .child(div().size_full().rounded(radius).bg(swatch.to_hsla()))
                });

                // `.color-swatch-picker__indicator` spans the *item* (`absolute
                // inset-0`) and centres a checkmark at `size-1/3` of it -- white by
                // default, black over a light colour (`data-light-color`).
                if let Some(render) = &self.indicator {
                    cell = cell.child(
                        div()
                            .absolute()
                            .inset_0()
                            .flex()
                            .items_center()
                            .justify_center()
                            .when(!selected, |indicator| indicator.opacity(0.))
                            .child(render(index, state)),
                    );
                } else if selected {
                    cell = cell.child(
                        div()
                            .absolute()
                            .inset_0()
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                gpui::svg()
                                    .size(px(f32::from(item_edge) / 3.))
                                    .path(crate::icons::CHECK)
                                    .text_color(color_swatch_indicator_color(*swatch)),
                            ),
                    );
                }
            }

            // Selected: `border-color: var(--color-swatch-current)` -- the
            // border takes the swatch's own colour, and the gap the shrunk
            // swatch leaves is what reads as a ring.
            if selected {
                cell = cell.border_color(swatch.to_hsla());
            } else {
                cell = cell.border_color(gpui::transparent_black());
            }

            if item_disabled {
                // v3's sheet: `[data-disabled="true"]` is `status-disabled`,
                // reduced opacity, and no press handler is attached here at
                // all -- a disabled swatch cannot report a choice.
                cell = cell.opacity(cx.layout().disabled_opacity);
            } else if self.on_change.is_some() || own.is_some() {
                let on_change = self.on_change.clone();
                let own = own.clone();
                let value = *swatch;
                cell = cell.cursor_pointer().on_click(move |_, window, cx| {
                    // Uncontrolled: take the selection, or the press would do
                    // nothing.
                    if let Some(held) = &own {
                        held.update(cx, |v, cx| {
                            *v = Some(value);
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &on_change {
                        cb(&value, window, cx);
                    }
                });
            }

            if !item_disabled {
                let moved = cursor.clone();
                let focus = swatch_focus.clone();
                let pointer = pointer_focus.clone();
                if let Some(interaction) = interactions.get(index) {
                    cell = util::track_interaction_on_mouse_down(
                        cell,
                        interaction,
                        move |window, cx| {
                            moved.update(cx, |cursor, cx| {
                                *cursor = index;
                                cx.notify();
                            });
                            pointer.update(cx, |focused, cx| {
                                *focused = Some(index);
                                cx.notify();
                            });
                            window.focus(&focus, cx);
                        },
                    );
                } else {
                    cell = cell.on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                        moved.update(cx, |cursor, cx| {
                            *cursor = index;
                            cx.notify();
                        });
                        pointer.update(cx, |focused, cx| {
                            *focused = Some(index);
                            cx.notify();
                        });
                        window.focus(&focus, cx);
                    });
                }
            }

            // The collection keyboard, the ListBox's: the arrows rove the
            // focus over the enabled swatches and Home and End jump, while a
            // disabled swatch is never a stop. Enter and Space stay with gpui
            // -- a focused element's press fires on key up, so the click
            // handler above is the only path that selects; binding them again
            // here would select twice. Disabled swatches never see this
            // handler (they cannot hold the focus), so the cursor only moves
            // between `enabled`.
            if !item_disabled {
                let stops = enabled.clone();
                let moved = cursor.clone();
                let pointer = pointer_focus.clone();
                let columns = grid_columns;
                cell = cell.on_key_down(move |event, _window, cx| {
                    let key = event.keystroke.key.as_str();
                    if key == "tab" {
                        pointer.update(cx, |focused, cx| {
                            if focused.take().is_some() {
                                cx.notify();
                            }
                        });
                        return;
                    }
                    let next = match key {
                        // Grid rows: same column, adjacent row. The strided
                        // target must be a real enabled swatch; otherwise the
                        // row does not exist, which is what the geometric
                        // delegate reports for a ragged grid too.
                        "up" | "down" if columns > 0 => {
                            let j = index as isize
                                + if key == "down" {
                                    columns as isize
                                } else {
                                    -(columns as isize)
                                };
                            if j < 0 || !stops.contains(&(j as usize)) {
                                return;
                            }
                            j as usize
                        }
                        key @ ("up" | "down") if columns == 0 => {
                            let crate::list_nav::Move::To(next) =
                                crate::list_nav::resolve(&stops, Some(index), key, false)
                            else {
                                return;
                            };
                            next
                        }
                        key @ ("left" | "right" | "home" | "end") => {
                            let key = match key {
                                "right" => "down",
                                "left" => "up",
                                other => other,
                            };
                            let crate::list_nav::Move::To(next) =
                                crate::list_nav::resolve(&stops, Some(index), key, false)
                            else {
                                return;
                            };
                            next
                        }
                        _ => return,
                    };
                    cx.stop_propagation();
                    // No refocusing: the next render has the swatch at `next`
                    // claim the group's handle, so the focus goes with it.
                    moved.update(cx, |v, cx| {
                        *v = next;
                        cx.notify();
                    });
                });
            }

            let mut cell =
                util::with_focus_ring(cell, swatch_ring && item_focused, true, Vec::new(), cx);
            cell = cell
                .a11y_named(
                    a11y::Role::RadioButton,
                    &a11y::Name::labelled(swatch.to_hex()),
                )
                .a11y_selected(selected);
            row = row.child(cell);
        }

        row.a11y(a11y::Role::RadioGroup)
    }
}

// ---------------------------------------------------------------------------
