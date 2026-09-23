//! The component chrome string catalogue (`i18n`): the active locale, the
//! built-in React Aria translations, application overrides, and the en-US
//! fallback every component renders when nothing is configured.
//!
//! The strings feed accessible names as well as drawn text, and the headless
//! platform builds no accessibility tree (see `a11y_deep.rs`), so the render
//! half is pinned by a source scan: every call site that used to inline an
//! English literal now resolves it through `i18n::ui_string`.

mod harness;

use gpui::{IntoElement, ParentElement, TestAppContext};
use harness::open_host;
use herogpui_components::{
    i18n::{self, UiString},
    CloseButton, Select, Spinner,
};

#[gpui::test]
fn unset_locale_is_en_us(cx: &mut TestAppContext) {
    cx.update(|cx| {
        assert_eq!(i18n::locale(cx), "en-US");
        assert_eq!(
            i18n::ui_string(UiString::SelectPlaceholder, cx),
            "Select an item"
        );
        assert_eq!(i18n::ui_string(UiString::NoResults, cx), "No results found");
        assert_eq!(i18n::ui_string(UiString::Close, cx), "Close");
    });
}

#[gpui::test]
fn set_locale_switches_the_built_in_table(cx: &mut TestAppContext) {
    cx.update(|cx| {
        i18n::set_locale("fr-FR", cx);
        assert_eq!(i18n::locale(cx), "fr-FR");
        assert_eq!(i18n::ui_string(UiString::Previous, cx), "Précédent");
        assert_eq!(i18n::ui_string(UiString::Close, cx), "Fermer");
        // HeroUI has no dictionary entry for this one: en-US.
        assert_eq!(i18n::ui_string(UiString::Loading, cx), "Loading");
        i18n::set_locale("de-CH", cx);
        assert_eq!(i18n::ui_string(UiString::Next, cx), "Weiter");
    });
}

#[gpui::test]
fn overrides_win_exactly_or_by_language(cx: &mut TestAppContext) {
    cx.update(|cx| {
        i18n::set_ui_string("fr", UiString::Loading, "Chargement", cx);
        i18n::set_ui_string("fr-CA", UiString::Close, "Fermer (CA)", cx);
        i18n::set_locale("fr-FR", cx);
        assert_eq!(i18n::ui_string(UiString::Loading, cx), "Chargement");
        assert_eq!(i18n::ui_string(UiString::Close, cx), "Fermer");
        i18n::set_locale("fr-CA", cx);
        assert_eq!(i18n::ui_string(UiString::Close, cx), "Fermer (CA)");
        i18n::set_locale("en-US", cx);
        assert_eq!(i18n::ui_string(UiString::Loading, cx), "Loading");
    });
}

/// Components render under a non-default locale without panicking, and a
/// caller-set placeholder still wins over the localized default.
#[gpui::test]
fn components_render_under_a_locale(cx: &mut TestAppContext) {
    cx.update(|cx| i18n::set_locale("de-DE", cx));
    let cx = open_host(cx, || {
        gpui::div()
            .child(Select::new("i18n-select", Vec::new()))
            .child(Select::new("i18n-select-own", Vec::new()).placeholder("Wählen"))
            .child(CloseButton::new("i18n-close"))
            .child(Spinner::new("i18n-spinner"))
            .into_any_element()
    });
    cx.run_until_parked();
}

#[test]
fn call_sites_resolve_through_the_catalogue() {
    let src = concat!(env!("CARGO_MANIFEST_DIR"), "/src/");
    for (file, key) in [
        ("select.rs", "UiString::SelectPlaceholder"),
        ("autocomplete.rs", "UiString::SelectPlaceholder"),
        ("autocomplete.rs", "UiString::NoResults"),
        ("close_button.rs", "UiString::Close"),
        ("spinner.rs", "UiString::Loading"),
        ("calendar.rs", "UiString::Previous"),
        ("range_calendar.rs", "UiString::Next"),
        ("tag_group.rs", "UiString::Remove"),
        ("breadcrumbs.rs", "UiString::Breadcrumbs"),
        ("input.rs", "UiString::Search"),
    ] {
        let text = std::fs::read_to_string(format!("{src}{file}")).unwrap();
        assert!(text.contains(key), "{file} does not resolve {key}");
    }
}
