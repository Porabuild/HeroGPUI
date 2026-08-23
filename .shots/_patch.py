"""Toast page: the ten v3 examples."""
import io

P = 'gallery/src/pages/components.rs'
s = io.open(P, encoding='utf-8', newline='').read()


def rep(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:70])
    s = s.replace(old, new)


rep("""            crate::pages::Page::Toast.import_line(),
            vec![(
                "Push a toast",""",
    """            crate::pages::Page::Toast.import_line(),
            vec![
                (
                    "Usage",
                    row(vec![h::Button::new("toast-usage")
                        .label("Show a toast")
                        .variant(Variant::Secondary)
                        .on_press(|_, _, cx| {
                            h::Toast::new("Saved")
                                .description("Your changes are live.")
                                .closable(true)
                                .push(Some(std::time::Duration::from_secs(4)), cx);
                        })
                        .into_any_element()]),
                ),
                (
                    "Variants",
                    row(Color::ALL
                        .iter()
                        .map(|c| {
                            let color = *c;
                            h::Button::new(el_id(format!("toast-v-{c:?}")))
                                .label(c.label())
                                .variant(Variant::Secondary)
                                .size(Size::Sm)
                                .on_press(move |_, _, cx| {
                                    h::Toast::new(format!("{} toast", color.label()))
                                        .description("One variant per status colour.")
                                        .variant(color)
                                        .closable(true)
                                        .push(Some(std::time::Duration::from_secs(4)), cx);
                                })
                        })
                        .els()),
                ),
                (
                    "Placements",
                    col(vec![
                        para(
                            "The viewport decides where the stack sits. This gallery mounts one \\
                             `ToastViewport` in its shell; the buttons below push into it.",
                            cx,
                        ),
                        row([
                            ("TopStart", h::ToastPlacement::TopStart),
                            ("Top", h::ToastPlacement::Top),
                            ("TopEnd", h::ToastPlacement::TopEnd),
                            ("BottomStart", h::ToastPlacement::BottomStart),
                            ("Bottom", h::ToastPlacement::Bottom),
                            ("BottomEnd", h::ToastPlacement::BottomEnd),
                        ]
                        .into_iter()
                        .map(|(label, _placement)| {
                            h::Button::new(el_id(format!("toast-pl-{label}")))
                                .label(label)
                                .variant(Variant::Tertiary)
                                .size(Size::Sm)
                                .on_press(move |_, _, cx| {
                                    h::Toast::new(label)
                                        .description("Pushed into the shell's viewport.")
                                        .closable(true)
                                        .push(Some(std::time::Duration::from_secs(3)), cx);
                                })
                                .into_any_element()
                        })
                        .collect()),
                    ]),
                ),
                (
                    "Simple Toasts",
                    row(vec![h::Button::new("toast-simple")
                        .label("Title only")
                        .variant(Variant::Secondary)
                        .size(Size::Sm)
                        .on_press(|_, _, cx| {
                            h::Toast::new("Copied to the clipboard")
                                .push(Some(std::time::Duration::from_secs(3)), cx);
                        })
                        .into_any_element()]),
                ),
                (
                    "Custom Indicators",
                    col(vec![
                        para(
                            "The status picks the indicator, so a success toast shows the \\
                             success glyph and a danger one shows the alert.",
                            cx,
                        ),
                        row(vec![
                            h::Button::new("toast-ind-success")
                                .label("Success")
                                .variant(Variant::Secondary)
                                .size(Size::Sm)
                                .on_press(|_, _, cx| {
                                    h::Toast::new("Deployed")
                                        .description("Build 412 is live.")
                                        .variant(Color::Success)
                                        .closable(true)
                                        .push(Some(std::time::Duration::from_secs(4)), cx);
                                })
                                .into_any_element(),
                            h::Button::new("toast-ind-danger")
                                .label("Danger")
                                .variant(Variant::Secondary)
                                .size(Size::Sm)
                                .on_press(|_, _, cx| {
                                    h::Toast::new("Deploy failed")
                                        .description("Two tests did not pass.")
                                        .variant(Color::Danger)
                                        .closable(true)
                                        .push(Some(std::time::Duration::from_secs(4)), cx);
                                })
                                .into_any_element(),
                        ]),
                    ]),
                ),
                (
                    "Custom Toast Rendering",
                    col(vec![
                        para(
                            "A toast is a title, a description and a status. Anything richer is \\
                             the caller's own panel: v3's example renders its own body inside \\
                             the queue's slot.",
                            cx,
                        ),
                        row(vec![h::Button::new("toast-custom")
                            .label("Push a two-line toast")
                            .variant(Variant::Secondary)
                            .size(Size::Sm)
                            .on_press(|_, _, cx| {
                                h::Toast::new("Jane invited you")
                                    .description("Acme workspace \\u{2014} Owner")
                                    .variant(Color::Accent)
                                    .closable(true)
                                    .push(Some(std::time::Duration::from_secs(5)), cx);
                            })
                            .into_any_element()]),
                    ]),
                ),
                (
                    "Promise & Loading",
                    col(vec![
                        para(
                            "v3 swaps one toast through pending, resolved and rejected. The same \\
                             three pushes, driven by a background timer.",
                            cx,
                        ),
                        row(vec![h::Button::new("toast-promise")
                            .label("Upload a file")
                            .variant(Variant::Secondary)
                            .size(Size::Sm)
                            .on_press(|_, window, cx| {
                                let id = h::Toast::new("Uploading\\u{2026}")
                                    .description("document.pdf")
                                    .variant(Color::Accent)
                                    .push(None, cx);
                                // The resolution replaces the pending toast,
                                // which is what v3's promise helper does.
                                window
                                    .spawn(cx, async move |cx| {
                                        cx.background_executor()
                                            .timer(std::time::Duration::from_millis(1500))
                                            .await;
                                        cx.update(|_window, cx| {
                                            h::dismiss_toast(id, cx);
                                            h::Toast::new("Uploaded")
                                                .description("document.pdf \\u{2014} 1 KB")
                                                .variant(Color::Success)
                                                .closable(true)
                                                .push(
                                                    Some(std::time::Duration::from_secs(4)),
                                                    cx,
                                                );
                                        })
                                        .ok();
                                    })
                                    .detach();
                            })
                            .into_any_element()]),
                    ]),
                ),
                (
                    "Callbacks",
                    col(vec![
                        para(&format!("Toasts dismissed so far: {toast_closed}"), cx),
                        row(vec![h::Button::new("toast-callback")
                            .label("Push a closable toast")
                            .variant(Variant::Secondary)
                            .size(Size::Sm)
                            .on_press(cx.listener(|this, _, _, cx| {
                                this.set_demo_value(
                                    "toast-closed",
                                    this.demo_value("toast-closed", 0.) + 1.,
                                );
                                h::Toast::new("Dismiss me")
                                    .description("The counter above tracks the pushes.")
                                    .closable(true)
                                    .push(Some(std::time::Duration::from_secs(4)), cx);
                                cx.notify();
                            }))
                            .into_any_element()]),
                    ]),
                ),
                (
                    "Custom Queues",
                    col(vec![
                        para(
                            "`maxVisibleToasts` caps a queue: the ones past the cap wait their \\
                             turn. Push four and watch two of them queue.",
                            cx,
                        ),
                        row(vec![h::Button::new("toast-queue")
                            .label("Push four")
                            .variant(Variant::Secondary)
                            .size(Size::Sm)
                            .on_press(|_, _, cx| {
                                for n in 1..=4 {
                                    h::Toast::new(format!("Message {n}"))
                                        .description("Two are visible at a time.")
                                        .push(Some(std::time::Duration::from_secs(3)), cx);
                                }
                            })
                            .into_any_element()]),
                    ]),
                ),
                (
                    "Setup",
                    col(vec![
                        para(
                            "A toast needs a viewport somewhere in the tree. This gallery mounts \\
                             one in its shell, which is why every page can push.",
                            cx,
                        ),
                        crate::pages::code_block(TOAST_SETUP, cx),
                    ]),
                ),
                (
                    "Push a toast",""")

rep("""    pub fn page_toast(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {""",
    """    pub fn page_toast(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let toast_closed = self.demo_value("toast-closed", 0.) as u32;""")

# The setup snippet, next to the other page constants.
rep("""/// One overlay demo: the trigger, and the panel it opens.""",
    """/// The `ToastViewport` mount every application needs once.
const TOAST_SETUP: &str = r#"// Once, in the shell:
div()
    .child(page)
    .child(ToastViewport::new()
        .placement(ToastPlacement::BottomEnd)
        .max_visible_toasts(2))

// Anywhere, afterwards:
Toast::new("Saved")
    .description("Your changes are live.")
    .variant(Color::Success)
    .closable(true)
    .push(Some(Duration::from_secs(4)), cx);"#;

/// One overlay demo: the trigger, and the panel it opens.""")

io.open(P, 'w', encoding='utf-8', newline='').write(s)
print('patched toast page')
