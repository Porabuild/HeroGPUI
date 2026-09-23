//! The smallest HeroGPUI application: one window, one button, one counter.
//!
//! ```sh
//! cargo run -p herogpui-example-hello-button
//! ```

use herogpui::*;

struct Hello {
    presses: usize,
}

impl Render for Hello {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let label = format!("Pressed {} times", self.presses);
        app_focus_root(
            div()
                .size_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_4()
                .bg(cx.colors().background)
                .text_color(cx.colors().foreground)
                .child(label)
                .child(Button::new("hello").label("Press me").on_press(cx.listener(
                    |this, _, _, cx| {
                        this.presses += 1;
                        cx.notify();
                    },
                ))),
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
            let bounds = Bounds::centered(None, size(px(480.), px(320.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_, cx| cx.new(|_| Hello { presses: 0 }),
            )
            .expect("failed to open window");
            cx.activate(true);
        });
}
