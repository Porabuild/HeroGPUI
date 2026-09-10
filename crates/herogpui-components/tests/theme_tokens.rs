//! The customisation tokens reach their consumers.
//!
//! Builder/JSON application is covered in `herogpui-theme`. These tests pin
//! the other half: `set_theme` publishes every token to the layout, and each
//! named consumer reads the token that replaced its literal. This is wiring
//! evidence, not a drawn-pixel measurement.

mod source_scan;

use gpui::TestAppContext;
use herogpui_theme::{set_theme, ActiveTheme, Theme, ThemeProvider};
use source_scan::{component_src, scope_contains};

struct Consumer {
    file: &'static str,
    part: &'static str,
    /// Source marker whose enclosing function must read the token.
    owner: &'static str,
    token: &'static str,
    /// The hardcoded literal this consumer used before the token existed, if
    /// any; it must be gone.
    removed: Option<&'static str>,
}

const CONSUMERS: &[Consumer] = &[
    Consumer {
        file: "tabs.rs",
        part: "primary tab hover dim",
        owner: "tab.hover(move |s| s.opacity(tabs_hover_opacity))",
        token: "tabs_hover_opacity",
        removed: Some("opacity(0.7)"),
    },
    Consumer {
        file: "tabs.rs",
        part: "secondary tab hover dim",
        owner: "tab.hover(move |tab| tab.opacity(tabs_hover_opacity))",
        token: "tabs_hover_opacity",
        removed: Some("opacity(0.7)"),
    },
    Consumer {
        file: "tabs.rs",
        part: "scroll-arrow hover dim",
        owner: "arrow.opacity(tabs_hover_opacity)",
        token: "tabs_hover_opacity",
        removed: Some("opacity(0.7)"),
    },
    Consumer {
        file: "tooltip.rs",
        part: "global cooldown floor",
        owner: "fn start_tooltip_cooldown",
        token: "tooltip_cooldown_ms",
        removed: Some("TOOLTIP_GLOBAL_COOLDOWN_MS"),
    },
    Consumer {
        file: "dropdown.rs",
        part: "long-press wait",
        owner: "DropdownTrigger::LongPress => {",
        token: "long_press_ms",
        removed: Some("LONG_PRESS_MS"),
    },
    Consumer {
        file: "anim.rs",
        part: "hover-fade duration",
        owner: "fn hover_fade_duration",
        token: "hover_fade_ms",
        removed: Some("Animation::new(Duration::from_millis(TRANSITION_MS))"),
    },
];

#[test]
fn every_named_consumer_reads_its_token_and_drops_the_literal() {
    let mut failures = Vec::new();
    for consumer in CONSUMERS {
        let source = component_src(consumer.file);
        if !source.contains(consumer.owner) {
            failures.push(format!(
                "{} [{}]: owner marker {:?} is gone",
                consumer.file, consumer.part, consumer.owner
            ));
            continue;
        }
        if let Err(err) = scope_contains(&source, consumer.owner, consumer.token) {
            failures.push(format!("{} [{}]: {err}", consumer.file, consumer.part));
        }
        if let Some(literal) = consumer.removed {
            if source.contains(literal) {
                failures.push(format!(
                    "{} [{}]: the replaced literal {:?} is still in the source",
                    consumer.file, consumer.part, literal
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "token consumer wiring failed:\n{}",
        failures.join("\n")
    );
}

#[gpui::test]
fn set_theme_publishes_every_customisation_token(cx: &mut TestAppContext) {
    cx.update(ThemeProvider::init);
    cx.update(|cx| {
        set_theme(
            Theme::builder("custom", Theme::light())
                .tabs_hover_opacity(0.25)
                .tooltip_cooldown_ms(111)
                .long_press_ms(222)
                .hover_fade_ms(333)
                .tooltip_delay_ms(444)
                .tooltip_close_delay_ms(555)
                .build(),
            cx,
        );
    });
    cx.update(|cx| {
        let layout = cx.layout();
        assert!((layout.tabs_hover_opacity - 0.25).abs() < 1e-6);
        assert_eq!(layout.tooltip_cooldown_ms, 111);
        assert_eq!(layout.long_press_ms, 222);
        assert_eq!(layout.hover_fade_ms, 333);
        assert_eq!(layout.tooltip_delay_ms, 444);
        assert_eq!(layout.tooltip_close_delay_ms, 555);
    });
}
