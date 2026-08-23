"""TimeField: focus handle, autoFocus and the key handling v3 documents."""
import io

P = 'crates/herogpui-components/src/time_field.rs'
s = io.open(P, encoding='utf-8', newline='').read()


def rep(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:70])
    s = s.replace(old, new)


rep("""        let colors = cx.colors();
        let layout = cx.layout();
        let entity_id = self.state.entity_id().as_u64();
        let interactive = !self.is_disabled && !self.is_read_only;""",
    """        let entity_id = self.state.entity_id().as_u64();
        // A time field has no inner `Input`, so it owns its focus handle. Keyed
        // state keeps it across frames; `use_keyed_state` takes `cx` mutably, so
        // it precedes the theme tokens.
        let focus_handle = window.use_keyed_state(
            ElementId::Name(format!("timefield-{entity_id}-focus").into()),
            cx,
            |_, cx| cx.focus_handle(),
        );
        let focus_handle = focus_handle.read(cx).clone();
        if self.auto_focus {
            util::focus_once(
                window,
                cx,
                ElementId::Name(format!("timefield-{entity_id}-autofocus").into()),
                &focus_handle,
            );
        }
        // Digits typed into the focused segment but not yet complete, so `1` in
        // the hour segment can still become `12`.
        let typing = window.use_keyed_state(
            ElementId::Name(format!("timefield-{entity_id}-typing").into()),
            cx,
            |_, _| String::new(),
        );

        let colors = cx.colors();
        let layout = cx.layout();
        let interactive = !self.is_disabled && !self.is_read_only;""")

rep("""        group = util::apply_field_chrome(group, self.variant, is_invalid, false, cx);""",
    """        group = util::apply_field_chrome(group, self.variant, is_invalid, false, cx);

        // v3 drives a time field from the keyboard: the arrows step the focused
        // segment and walk between segments, and digits type into it.
        if interactive {
            let state = self.state.clone();
            let on_change = self.on_change.clone();
            let buffer = typing;
            let fh = focus_handle.clone();
            let order = TimeSegment::order(self.granularity, self.hour_cycle == HourCycle::H12);
            let twelve_hour = self.hour_cycle == HourCycle::H12;
            let seed = self.placeholder_value.unwrap_or(Time::new(9, 0));
            group = group
                .track_focus(&focus_handle)
                .key_context("TimeField")
                .on_mouse_down(gpui::MouseButton::Left, move |_, window, _| {
                    window.focus(&fh);
                })
                .on_key_down(move |event, window, cx| {
                    let key = event.keystroke.key.as_str();
                    let here = order.iter().position(|s| *s == focused).unwrap_or(0);
                    let commit = |time: Time, window: &mut Window, cx: &mut App| {
                        state.update(cx, |s, cx| {
                            s.value = Some(time);
                            cx.notify();
                        });
                        if let Some(cb) = &on_change {
                            cb(Some(time), window, cx);
                        }
                    };
                    match key {
                        "up" | "down" => {
                            let delta = if key == "up" { 1 } else { -1 };
                            buffer.update(cx, |b, _| b.clear());
                            state.update(cx, |s, cx| {
                                s.bump_focused_from(delta, seed);
                                cx.notify();
                            });
                            let next = state.read(cx).value;
                            if let (Some(cb), Some(time)) = (&on_change, next) {
                                cb(Some(time), window, cx);
                            }
                        }
                        "left" | "right" => {
                            let delta: i32 = if key == "right" { 1 } else { -1 };
                            let next = (here as i32 + delta)
                                .clamp(0, order.len() as i32 - 1) as usize;
                            buffer.update(cx, |b, _| b.clear());
                            let segment = order[next];
                            state.update(cx, |s, cx| {
                                s.focused = segment;
                                cx.notify();
                            });
                        }
                        "backspace" | "delete" => {
                            buffer.update(cx, |b, _| b.clear());
                            state.update(cx, |s, cx| {
                                s.value = None;
                                cx.notify();
                            });
                            if let Some(cb) = &on_change {
                                cb(None, window, cx);
                            }
                        }
                        // The meridiem segment answers `a` and `p`, the way
                        // React Aria's does.
                        "a" | "p" if focused == TimeSegment::Meridiem => {
                            let base = state.read(cx).value.unwrap_or(seed);
                            let hour = base.hour % 12 + if key == "p" { 12 } else { 0 };
                            commit(
                                Time::new(hour, base.minute).with_second(base.second),
                                window,
                                cx,
                            );
                        }
                        digit if digit.len() == 1 && digit.chars().all(|c| c.is_ascii_digit()) => {
                            let width = focused.digits();
                            if width == 0 {
                                return;
                            }
                            let text = buffer.update(cx, |b, _| {
                                if b.len() >= width {
                                    b.clear();
                                }
                                b.push_str(digit);
                                b.clone()
                            });
                            let Ok(value) = text.parse::<u32>() else {
                                return;
                            };
                            let base = state.read(cx).value.unwrap_or(seed);
                            commit(focused.with_value(base, value, twelve_hour), window, cx);
                            if text.len() >= width {
                                buffer.update(cx, |b, _| b.clear());
                                if let Some(segment) = order.get(here + 1).copied() {
                                    state.update(cx, |s, cx| {
                                        s.focused = segment;
                                        cx.notify();
                                    });
                                }
                            }
                        }
                        _ => {}
                    }
                });
        }""")

io.open(P, 'w', encoding='utf-8', newline='').write(s)
print('patched time field keyboard')
