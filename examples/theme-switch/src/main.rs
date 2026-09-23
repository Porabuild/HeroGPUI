//! Runtime theme switching: a Switch toggles the built-in light and dark
//! themes, and a row of buttons activates the preset JSON themes that ship
//! with the `serde` feature (`herogpui::presets`).
//!
//! ```sh
//! cargo run -p herogpui-example-theme-switch
//! ```

use herogpui::*;

struct ThemeSwitch;

impl Render for ThemeSwitch {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let dark = cx.theme().is_dark();
        let active = ThemeProvider::get(cx).active_id().clone();
        let presets = presets::PRESETS.iter().map(|(id, _)| {
            Button::new(SharedString::from(format!("preset-{id}")))
                .label(*id)
                .variant(if active.as_ref() == *id {
                    Variant::Primary
                } else {
                    Variant::Secondary
                })
                .on_press(move |_, _, cx| {
                    // Registered at startup, so the id always resolves.
                    let _ = use_theme(*id, cx);
                })
        });
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
                .child(format!("Active theme: {active}"))
                .child(
                    Switch::new("dark-mode")
                        .is_selected(dark)
                        .label("Dark mode")
                        .on_change(|_: &bool, _, cx| toggle_light_dark(cx)),
                )
                .child(div().flex().gap_2().children(presets)),
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
            presets::register_presets(cx);
            let bounds = Bounds::centered(None, size(px(560.), px(320.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_, cx| cx.new(|_| ThemeSwitch),
            )
            .expect("failed to open window");
            cx.activate(true);
        });
}
