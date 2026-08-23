//! Getting Started documentation pages.

use gpui::{prelude::*, px, App, Context};
use herogpui_core::Color;
use herogpui_theme::ActiveTheme;

use crate::app::Gallery;
use crate::pages::{code_block, doc_page, example_frame, para};

impl Gallery {
    pub fn page_introduction(&mut self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let colors = cx.colors();
        let _ = colors;
        doc_page(
            "Introduction",
            "HeroGPUI is a beautiful, fast and modern cross-platform UI library for Rust. \
             It is a faithful port of the HeroUI (formerly NextUI) design system to GPUI — \
             Zed's GPU-accelerated UI framework — matching its component API, theming tokens \
             and capabilities.",
            "",
            vec![
                (
                    "Design Principles",
                    gpui::div()
                        .flex()
                        .flex_col()
                        .gap(px(8.))
                        .child(para("Beautiful — every component follows the HeroUI design language: soft radii, semantic colors and subtle shadows.", cx))
                        .child(para("Fast — components are plain data rendered through GPUI's immediate-mode pipeline on the GPU; no DOM, no layout thrash.", cx))
                        .child(para("Modern — a builder API that feels like React props: Button::new(\"save\").variant(Variant::Primary).on_press(...)", cx))
                        .child(para("Cross-platform — one codebase for Windows, macOS and Linux.", cx))
                        .into_any_element(),
                ),
                (
                    "Highlights",
                    gpui::div().flex().flex_col().gap(px(10.)).children(vec![
                        feature_row("69 v3 components", "Every component documented at heroui.com/docs/react/components — all themed.", cx),
                        feature_row("v3 OKLCH tokens", "Semantic roles (default/accent/success/warning/danger) with derived hover and soft variants, light & dark.", cx),
                        feature_row("Gallery & docs", "This app doubles as living documentation with runnable examples.", cx),
                    ]).into_any_element(),
                ),
            ],
            cx,
        )
    }

    pub fn page_installation(&mut self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let main_rs = r#"use gpui::{prelude::*, px, size, App, Application,
    Bounds, Render, Window, WindowBounds, WindowOptions};
use herogpui::theme::{ThemeProvider, ActiveTheme};
use herogpui_core::Color;

struct HelloWorld;

impl Render for HelloWorld {
    fn render(&mut self, _w: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let colors = cx.colors();
        gpui::div().size_full().bg(colors.background)
            .text_color(colors.foreground)
            .font_family("Segoe UI")
            .flex().items_center().justify_center()
            .child(
                herogpui::Button::new("hi")
                    .label("Hello HeroGPUI")
                    .variant(Variant::Primary)
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        herogpui::theme::ThemeProvider::init(cx); // registers light + dark
        let bounds = Bounds::centered(None, size(px(800.), px(600.)), cx);
        cx.open_window(WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            ..Default::default()
        }, |_, cx| cx.new(|_| HelloWorld)).unwrap();
    });
}"#;

        doc_page(
            "Installation",
            "Add HeroGPUI to your Cargo workspace, register the theme provider and render your first component.",
            "cargo add herogpui",
            vec![
                ("Setup GPUI", code_block(main_rs, cx)),
                (
                    "Assets",
                    para(
                        "Components that ship icons (Spinner, Checkbox, Accordion, ...) reference SVGs under `herogpui/icons/*`. \
                         Register an AssetSource that serves them — copy the folder from `gallery/assets/herogpui` into your app.",
                        cx,
                    ),
                ),
            ],
            cx,
        )
    }

    pub fn page_theming(&mut self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let colors = cx.colors();

        let mut palette = gpui::div().flex().flex_col().gap(px(10.));
        for role in Color::ALL {
            let sem = cx.role(role);
            palette = palette.child(
                gpui::div()
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .child(
                        gpui::div()
                            .size(px(28.))
                            .rounded(px(8.))
                            .border_1()
                            .border_color(colors.separator)
                            .bg(sem.color),
                    )
                    .child(
                        gpui::div()
                            .flex()
                            .flex_1()
                            .h(px(20.))
                            .rounded(px(6.))
                            .overflow_hidden()
                            .children([sem.soft(), sem.soft_hover(), sem.color, sem.hover()].map(|c| gpui::div().flex_1().bg(c))),
                    )
                    .child(
                        gpui::div()
                            .w(px(90.))
                            .text_size(px(12.5))
                            .text_color(colors.muted)
                            .child(format!("{role:?}")),
                    ),
            );
        }

        doc_page(
            "Theming",
            "Every color in HeroGPUI is a semantic token resolved from the active Theme global. Base values are \
             transcribed verbatim from HeroUI v3 in oklch(); hover and soft variants are derived with the same \
             color-mix(in oklab) weights.",
            "use herogpui::theme::{ThemeProvider, ActiveTheme};",
            vec![
                ("Semantic palette (active appearance)", palette.into_any_element()),
                (
                    "Reading tokens anywhere",
                    code_block(
                        "fn my_view(cx: &App) -> impl IntoElement {\n    let primary = cx.role(Color::Accent);\n    div().bg(primary.color).text_color(primary.foreground)\n}",
                        cx,
                    ),
                ),
            ],
            cx,
        )
    }

    pub fn page_dark_mode(&mut self, cx: &mut Context<Self>) -> gpui::AnyElement {
        doc_page(
            "Dark Mode",
            "ThemeProvider registers both `light` and `dark`. Toggle at runtime from any handler — \
             every themed component re-renders automatically on the next frame.",
            "herogpui::theme::toggle_light_dark(cx);",
            vec![
                (
                    "Try it",
                    example_frame(
                        gpui::div()
                            .text_size(px(14.5))
                            .line_height(px(22.))
                            .child("Use the sun / moon button in the top bar of this gallery to toggle the active appearance live.")
                            .into_any_element(),
                        cx,
                    ),
                ),
                (
                    "Programmatic switch",
                    code_block(
                        "herogpui::theme::use_theme(\"dark\", cx);\n// or\nherogpui::theme::set_theme(my_custom_dark_theme, cx);",
                        cx,
                    ),
                ),
            ],
            cx,
        )
    }

    pub fn page_customization(&mut self, cx: &mut Context<Self>) -> gpui::AnyElement {
        // Live violet custom theme preview
        // v3 themes override a base token and let every derived value follow.
        let preview_theme = herogpui_theme::Theme::builder(
            "violet-preview",
            herogpui_theme::Theme::light(),
        )
        .accent(herogpui_core::oklch(0.55, 0.23, 295.0))
        .build();

        let mut chips = gpui::div().flex().flex_wrap().gap(px(10.));
        for role in Color::ALL {
            let sem = match role {
                Color::Default => &preview_theme.colors.default,
                Color::Accent => &preview_theme.colors.accent,
                Color::Success => &preview_theme.colors.success,
                Color::Warning => &preview_theme.colors.warning,
                Color::Danger => &preview_theme.colors.danger,
            };
            chips = chips.child(
                gpui::div()
                    .px(px(14.))
                    .py(px(6.))
                    .rounded_full()
                    .bg(sem.color)
                    .text_color(sem.foreground)
                    .text_size(px(12.5))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child(format!("{role:?}")),
            );
        }

        doc_page(
            "Customization",
            "Build custom themes exactly like HeroUI's createTheme: start from a base appearance and override \
             semantic scales, single shades or layout tokens. Register the result with the provider.",
            "",
            vec![
                ("Custom theme builder", code_block(CUSTOM_THEME_SNIPPET, cx)),
                ("Preview: a violet theme", example_frame(chips.into_any_element(), cx)),
            ],
            cx,
        )
    }
}

const CUSTOM_THEME_SNIPPET: &str = r#"use herogpui::core::oklch;

let violet = herogpui::theme::Theme::builder("violet", Theme::light())
    .accent(oklch(0.55, 0.23, 295.0))   // hover / soft / focus all derive
    .role("success", oklch(0.73, 0.19, 150.0), snow())
    .radius(px(6.))                     // field_radius follows at 1.5x
    .build();

herogpui::theme::set_theme(violet, cx);"#;

fn feature_row(title: &str, desc: &str, cx: &App) -> gpui::AnyElement {
    let colors = cx.colors();
    gpui::div()
        .flex()
        .items_start()
        .gap(px(10.))
        .child(
            gpui::div()
                .mt(px(6.))
                .size(px(8.))
                .rounded_full()
                .bg(colors.accent.color),
        )
        .child(
            gpui::div().flex().flex_col().child(
                gpui::div()
                    .text_size(px(14.))
                    .line_height(px(22.))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child(format!("{} — {}", title, desc)),
            ),
        )
        .into_any_element()
}

// keep FONT_FAMILY referenced for docs readers
#[allow(unused_imports)]
use crate::app::FONT_FAMILY as _FONT;


