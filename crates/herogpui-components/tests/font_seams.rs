//! Phase 5 field font seam: the mono token stays the default and an explicit
//! `font_family` overrides it on the box only when set.
//!
//! Observed through the inherited text style: a style probe placed in a slot
//! the component renders *inside* the part that owns the family (a field's
//! `prefix`, an input's `start_content`, a picker row's indicator, the
//! typography run's children) reads the family text there is drawn with. The
//! headless `NoopTextSystem` resolves every family to one font, so glyph
//! metrics cannot tell families apart; the inherited style can.
//!
//! One seam stays a scoped source check: `ColorPicker`'s hex readout is a
//! bare text child with no caller slot beside it, so nothing a test composes
//! inherits its style.

mod harness;
mod source_scan;

use gpui::{prelude::*, AnyElement, TestAppContext};
use harness::{open_host, seen_style, settle, still, style_probe, style_sink, StyleSink};
use herogpui_components::{
    Autocomplete, ComboBox, DateField, Input, InputGroup, InputState, PickerItem, Select,
    TimeField, TimeState, Typography,
};

/// `util::MONO_FONT` on a native target.
const MONO: &str = "Consolas";
const CUSTOM: &str = "Custom Family";

fn family(cx: &mut TestAppContext, build: impl Fn(StyleSink) -> AnyElement + 'static) -> String {
    still();
    let sink = style_sink();
    let seen = sink.clone();
    let vcx = open_host(cx, move || build(sink.clone()));
    settle(vcx);
    seen_style(&seen, "font seam").font_family.to_string()
}

fn items() -> Vec<PickerItem> {
    vec![PickerItem::new("a", "Alpha"), PickerItem::new("b", "Beta")]
}

#[gpui::test]
fn time_and_date_fields_keep_mono_and_accept_a_family_override(cx: &mut TestAppContext) {
    for custom in [None, Some(CUSTOM)] {
        let state = cx.new(|cx| TimeState::new(cx));
        let seen = family(cx, move |sink| {
            let field = TimeField::new(state.clone()).prefix(style_probe(&sink));
            match custom {
                Some(f) => field.font_family(f),
                None => field,
            }
            .into_any_element()
        });
        assert_eq!(seen, custom.unwrap_or(MONO), "TimeField segment group");

        let state = cx.new(|cx| InputState::new(cx));
        let seen = family(cx, move |sink| {
            let field = DateField::new(state.clone()).prefix(style_probe(&sink));
            match custom {
                Some(f) => field.font_family(f),
                None => field,
            }
            .into_any_element()
        });
        assert_eq!(seen, custom.unwrap_or(MONO), "DateField segment group");
    }
}

#[gpui::test]
fn typography_code_keeps_mono_and_any_kind_accepts_a_family(cx: &mut TestAppContext) {
    let code = family(cx, |sink| {
        Typography::code("let x")
            .child(style_probe(&sink))
            .into_any_element()
    });
    assert_eq!(code, MONO, "the `Code` run is mono by default");

    let overridden = family(cx, |sink| {
        Typography::code("let x")
            .font_family(CUSTOM)
            .child(style_probe(&sink))
            .into_any_element()
    });
    assert_eq!(
        overridden, CUSTOM,
        "`font_family` refines after the mono run"
    );

    let body = family(cx, |sink| {
        Typography::new("Hello")
            .font_family(CUSTOM)
            .child(style_probe(&sink))
            .into_any_element()
    });
    assert_eq!(body, CUSTOM, "a non-mono kind takes the family too");
}

#[gpui::test]
fn input_group_forwards_its_family_to_the_held_input(cx: &mut TestAppContext) {
    let plain = cx.new(|cx| InputState::new(cx));
    let stock = family(cx, move |sink| {
        InputGroup::new()
            .input(Input::new(plain.clone()).start_content(style_probe(&sink)))
            .into_any_element()
    });
    assert_ne!(stock, CUSTOM, "no family is forced without the override");

    let state = cx.new(|cx| InputState::new(cx));
    let seen = family(cx, move |sink| {
        InputGroup::new()
            .font_family(CUSTOM)
            .input(Input::new(state.clone()).start_content(style_probe(&sink)))
            .into_any_element()
    });
    assert_eq!(
        seen, CUSTOM,
        "the group's family must reach the inner field"
    );
}

#[gpui::test]
fn picker_rows_consume_row_font_family(cx: &mut TestAppContext) {
    for custom in [None, Some(CUSTOM)] {
        let seen = family(cx, move |sink| {
            let select = Select::new("sel", items())
                .default_open(true)
                .indicator(move |_| style_probe(&sink));
            match custom {
                Some(f) => select.row_font_family(f),
                None => select,
            }
            .into_any_element()
        });
        assert_eq!(seen == CUSTOM, custom.is_some(), "Select rows: {seen}");

        let state = cx.new(|cx| InputState::new(cx));
        let seen = family(cx, move |sink| {
            let combo = ComboBox::new(state.clone(), items())
                .default_open(true)
                .indicator(move |_| style_probe(&sink));
            match custom {
                Some(f) => combo.row_font_family(f),
                None => combo,
            }
            .into_any_element()
        });
        assert_eq!(seen == CUSTOM, custom.is_some(), "ComboBox rows: {seen}");

        let state = cx.new(|cx| InputState::new(cx));
        let seen = family(cx, move |sink| {
            let auto = Autocomplete::new(state.clone(), items())
                .default_open(true)
                .item_indicator(move |_| style_probe(&sink));
            match custom {
                Some(f) => auto.row_font_family(f),
                None => auto,
            }
            .into_any_element()
        });
        assert_eq!(
            seen == CUSTOM,
            custom.is_some(),
            "Autocomplete rows: {seen}"
        );
    }
}

/// Remaining source-text check: the hex readout has no caller slot, so no
/// probe can inherit its style (see the module docs).
#[test]
fn color_picker_readout_keeps_mono_and_accepts_a_family() {
    let picker = source_scan::component_src("color_picker/picker.rs");
    source_scan::scope_contains(
        &picker,
        ".child(self.value.to_hex())",
        ".unwrap_or_else(|| util::MONO_FONT.into())",
    )
    .unwrap();
    source_scan::scope_contains(&picker, "pub fn font_family(", "self.font_family = Some(")
        .unwrap();
}
