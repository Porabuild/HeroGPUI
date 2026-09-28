//! A validated form: two named text fields, a required check, and a submit
//! button that reports the collected `FormData`.
//!
//! ```sh
//! cargo run -p herogpui-example-form
//! ```

use herogpui::*;

struct SignUp {
    name: Entity<InputState>,
    email: Entity<InputState>,
    result: String,
}

impl Render for SignUp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let form = Form::new()
            .field(FormField::text(self.name.clone()).is_required(true))
            .field(FormField::text(self.email.clone()))
            .on_submit(cx.listener(|this, data: &FormData, _, cx| {
                this.result = data
                    .iter()
                    .map(|(name, value)| format!("{name}={}", value.as_text()))
                    .collect::<Vec<_>>()
                    .join(", ");
                cx.notify();
            }))
            .on_invalid(cx.listener(|this, _: &FormData, _, cx| {
                this.result = "Name is required".to_owned();
                cx.notify();
            }));
        let submit = form.submit_handler();
        app_focus_root(
            div()
                .size_full()
                .p_8()
                .flex()
                .flex_col()
                .gap_4()
                .bg(cx.colors().background)
                .text_color(cx.colors().foreground)
                .child(
                    form.child(
                        TextField::new(&self.name)
                            .name("name")
                            .label("Name")
                            .is_required(true),
                    )
                    .child(
                        TextField::new(&self.email)
                            .name("email")
                            .label("Email")
                            .placeholder("you@example.com"),
                    )
                    .child(
                        Button::new("submit")
                            .label("Submit")
                            .on_press(move |_, window, cx| submit(window, cx)),
                    ),
                )
                .child(self.result.clone()),
            window,
            cx,
        )
    }
}

fn main() {
    application()
        .with_assets(HeroGpuiAssets)
        .run(|cx: &mut App| {
            init(cx);
            let bounds = Bounds::centered(None, size(px(520.), px(420.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_, cx| {
                    cx.new(|cx| SignUp {
                        name: cx.new(|cx| InputState::new(cx)),
                        email: cx.new(|cx| InputState::new(cx)),
                        result: String::new(),
                    })
                },
            )
            .expect("failed to open window");
            cx.activate(true);
        });
}
