"""Autocomplete: the suggestions answer the arrows."""
import io

P = 'crates/herogpui-components/src/autocomplete.rs'
s = io.open(P, encoding='utf-8', newline='').read()


def rep(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:70])
    s = s.replace(old, new)


rep("""        let mut root = gpui::div().relative().child(input);""",
    """        // Which suggestion the keyboard is on. The inner input keeps left and
        // right for the caret, so up, down, Home, End and Enter bubble to here.
        let cursor = window.use_keyed_state(
            el_name(format!("autocomplete-{}-cursor", self.state.entity_id().as_u64())),
            cx,
            |_, _| None::<usize>,
        );
        let cursor_at = *cursor.read(cx);

        let mut root = gpui::div().relative().child(input);
        if !self.is_disabled && !self.is_read_only {
            let stops: Vec<usize> = (0..matches.len())
                .filter(|i| {
                    matches
                        .get(*i)
                        .is_some_and(|item| !self.disabled_keys.contains(item))
                })
                .collect();
            let held = cursor.clone();
            let wrap = self.should_focus_wrap;
            let rows = matches.clone();
            let state = self.state.clone();
            let on_selection_change = self.on_selection_change.clone();
            root = root.on_key_down(move |event, window, cx| {
                let from = *held.read(cx);
                match crate::list_nav::resolve(
                    &stops,
                    from,
                    event.keystroke.key.as_str(),
                    wrap,
                ) {
                    crate::list_nav::Move::To(next) => {
                        held.update(cx, |v, cx| {
                            *v = Some(next);
                            cx.notify();
                        });
                    }
                    crate::list_nav::Move::Activate => {
                        let Some(item) = from.and_then(|i| rows.get(i).cloned()) else {
                            return;
                        };
                        // Taking a suggestion fills the field, as a click does.
                        state.update(cx, |st, cx| {
                            st.set_value(item.to_string());
                            cx.notify();
                        });
                        held.update(cx, |v, _| *v = None);
                        if let Some(cb) = &on_selection_change {
                            cb(&item, window, cx);
                        }
                    }
                    crate::list_nav::Move::Ignore => {}
                }
            });
        }""")

rep("""                if item_disabled {
                    row = row.opacity(layout.disabled_opacity);
                } else {
                    row = row
                        .cursor_pointer()
                        .hover(move |s| s.bg(colors.default.soft()));
                }""",
    """                if item_disabled {
                    row = row.opacity(layout.disabled_opacity);
                } else {
                    row = row
                        .cursor_pointer()
                        .hover(move |s| s.bg(colors.default.soft()));
                }

                // `status-focused` on the row the keyboard is on.
                if matches.iter().position(|m| m == item) == Some(cursor_index) {
                    row = row.border_2().border_color(colors.focus);
                }""")

io.open(P, 'w', encoding='utf-8', newline='').write(s)
print('patched autocomplete keys')
