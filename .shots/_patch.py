"""ListBox: the arrow-key navigation."""
import io

P = 'crates/herogpui-components/src/list_box.rs'
s = io.open(P, encoding='utf-8', newline='').read()


def rep(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:70])
    s = s.replace(old, new)


rep("""        if let Some(max_h) = self.max_h {
            list = list.max_h(max_h).overflow_y_scroll();
        }""",
    """        if let Some(max_h) = self.max_h {
            list = list.max_h(max_h).overflow_y_scroll();
        }

        // The rows a keyboard can land on: an item that is not disabled.
        // Sections and separators are skipped, so the cursor never stops on
        // something that cannot be chosen.
        let stops: Vec<usize> = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| match item {
                ListBoxItem::Item { key, .. } => !self.disabled_keys.contains(key),
                _ => false,
            })
            .map(|(i, _)| i)
            .collect();

        if !stops.is_empty() {
            let held = cursor.clone();
            let stops_for_keys = stops.clone();
            let wrap = self.should_focus_wrap;
            let keys: Vec<SharedString> = self
                .items
                .iter()
                .map(|item| item.key().cloned().unwrap_or_default())
                .collect();
            let mode = self.selection_mode;
            let selected_now = self.selected_keys.clone();
            let on_selection_change = self.on_selection_change.clone();
            let on_action = self.on_action.clone();
            list = list.on_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                let here = stops_for_keys
                    .iter()
                    .position(|i| Some(*i) == *held.read(cx));
                let step = |delta: i32| -> Option<usize> {
                    let last = stops_for_keys.len() as i32 - 1;
                    let next = match here {
                        // With nothing focused, Down starts at the top and Up
                        // at the bottom, which is what React Aria does.
                        None if delta > 0 => 0,
                        None => last,
                        Some(pos) => {
                            let raw = pos as i32 + delta;
                            if raw < 0 {
                                if wrap {
                                    last
                                } else {
                                    0
                                }
                            } else if raw > last {
                                if wrap {
                                    0
                                } else {
                                    last
                                }
                            } else {
                                raw
                            }
                        }
                    };
                    stops_for_keys.get(next as usize).copied()
                };
                match key {
                    "down" | "up" => {
                        let next = step(if key == "down" { 1 } else { -1 });
                        held.update(cx, |v, cx| {
                            *v = next;
                            cx.notify();
                        });
                    }
                    "home" | "end" => {
                        let next = if key == "home" {
                            stops_for_keys.first().copied()
                        } else {
                            stops_for_keys.last().copied()
                        };
                        held.update(cx, |v, cx| {
                            *v = next;
                            cx.notify();
                        });
                    }
                    "enter" | "space" => {
                        let Some(index) = *held.read(cx) else {
                            return;
                        };
                        let Some(item_key) = keys.get(index).cloned() else {
                            return;
                        };
                        if let Some(cb) = &on_action {
                            cb(&item_key, window, cx);
                        }
                        if let Some(cb) = &on_selection_change {
                            let next = crate::selection::next_selection_set(
                                &selected_now,
                                &item_key,
                                mode,
                            );
                            cb(&next, window, cx);
                        }
                    }
                    _ => {}
                }
            });
        }""")

# The cursor row shows a focus ring, which is how the keyboard position reads.
rep("""                    if selected {
                        row = row.bg(match variant {
                            ListBoxItemVariant::Default => colors.accent.soft(),
                            ListBoxItemVariant::Danger => colors.danger.soft(),
                        });
                    }""",
    """                    if selected {
                        row = row.bg(match variant {
                            ListBoxItemVariant::Default => colors.accent.soft(),
                            ListBoxItemVariant::Danger => colors.danger.soft(),
                        });
                    }

                    // `status-focused` on the row the keyboard is on.
                    if cursor_at == Some(index) {
                        row = row.border_2().border_color(colors.focus);
                    }""")

io.open(P, 'w', encoding='utf-8', newline='').write(s)
print('patched list box keys')
