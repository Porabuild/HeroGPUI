//! Localizable component chrome strings (HeroGPUI extension).
//!
//! Dates, times and numbers already format per locale through ICU4X (each
//! picker's `.locale(..)`). This module covers the *fixed* strings the
//! components draw or announce themselves: the Select placeholder, the
//! calendar's Previous/Next buttons, the CloseButton's name, and so on.
//!
//! The application picks one locale with [`set_locale`]; every string then
//! resolves through [`ui_string`] as: an application override
//! ([`set_ui_string`]) → the built-in table for the locale (exact tag, then
//! its language) → en-US. Without any call, every component renders the same
//! en-US text it always has.
//!
//! The built-in translations are copied from the React Aria dictionaries
//! HeroUI v3.2.6 pins (`react-aria` 3.52.1 `dist/private/intl/*/<locale>.mjs`
//! and `react-aria-components` 1.21.1 `dist/private/intl/<locale>.mjs`), which
//! is where HeroUI's own strings come from. Strings HeroUI hard-codes in
//! English with no React Aria dictionary entry ([`UiString::NoResults`],
//! [`UiString::Loading`], [`UiString::Search`]) have only an en-US value;
//! supply other languages with [`set_ui_string`].
//!
//! ```
//! use herogpui_components::i18n::{self, UiString};
//!
//! assert_eq!(i18n::lookup("de-DE", UiString::Next), "Weiter");
//! // A region the table lacks falls back to its language...
//! assert_eq!(i18n::lookup("de-AT", UiString::Next), "Weiter");
//! // ...and an unknown language to en-US.
//! assert_eq!(i18n::lookup("xx", UiString::Next), "Next");
//! ```

use std::collections::HashMap;

use gpui::{App, Global, SharedString};

/// A component chrome string. See the [module docs](self).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum UiString {
    /// Select / Autocomplete placeholder (`selectPlaceholder`, RAC).
    SelectPlaceholder,
    /// Autocomplete's empty-collection message (HeroUI, en-US only).
    NoResults,
    /// CloseButton's accessible name (`close`, React Aria toast).
    Close,
    /// Spinner's accessible name (HeroUI, en-US only).
    Loading,
    /// Calendar previous-page button (`previous`, React Aria calendar).
    Previous,
    /// Calendar next-page button (`next`, React Aria calendar).
    Next,
    /// Tag remove button (`removeButtonLabel`, React Aria tag).
    Remove,
    /// Breadcrumbs list name (`breadcrumbs`, React Aria breadcrumbs).
    Breadcrumbs,
    /// SearchField placeholder (HeroUI, en-US only).
    Search,
}

impl UiString {
    /// Every key, in declaration order.
    pub const ALL: [UiString; 9] = [
        UiString::SelectPlaceholder,
        UiString::NoResults,
        UiString::Close,
        UiString::Loading,
        UiString::Previous,
        UiString::Next,
        UiString::Remove,
        UiString::Breadcrumbs,
        UiString::Search,
    ];

    fn index(self) -> usize {
        self as usize
    }
}

/// The locales with built-in translations; en-US first.
pub const LOCALES: [&str; 9] = [
    "en-US", "de-DE", "es-ES", "fr-FR", "it-IT", "nl-NL", "pl-PL", "pt-BR", "sv-SE",
];

const EN: [&str; 9] = [
    "Select an item",
    "No results found",
    "Close",
    "Loading",
    "Previous",
    "Next",
    "Remove",
    "Breadcrumbs",
    "Search",
];

// `None` falls back to en-US: HeroUI has no dictionary entry to copy.
type Row = [Option<&'static str>; 9];

const TABLE: [Row; 8] = [
    // de-DE
    [
        Some("Element wählen"),
        None,
        Some("Schließen"),
        None,
        Some("Zurück"),
        Some("Weiter"),
        Some("Entfernen"),
        Some("Breadcrumbs"),
        None,
    ],
    // es-ES
    [
        Some("Seleccionar un artículo"),
        None,
        Some("Cerrar"),
        None,
        Some("Anterior"),
        Some("Siguiente"),
        Some("Quitar"),
        Some("Migas de pan"),
        None,
    ],
    // fr-FR
    [
        Some("Sélectionner un élément"),
        None,
        Some("Fermer"),
        None,
        Some("Précédent"),
        Some("Suivant"),
        Some("Supprimer"),
        Some("Chemin de navigation"),
        None,
    ],
    // it-IT
    [
        Some("Seleziona un elemento"),
        None,
        Some("Chiudi"),
        None,
        Some("Precedente"),
        Some("Successivo"),
        Some("Rimuovi"),
        Some("Breadcrumb"),
        None,
    ],
    // nl-NL
    [
        Some("Selecteer een item"),
        None,
        Some("Sluiten"),
        None,
        Some("Vorige"),
        Some("Volgende"),
        Some("Verwijderen"),
        Some("Broodkruimels"),
        None,
    ],
    // pl-PL
    [
        Some("Wybierz element"),
        None,
        Some("Zamknij"),
        None,
        Some("Wstecz"),
        Some("Dalej"),
        Some("Usuń"),
        Some("Struktura nawigacyjna"),
        None,
    ],
    // pt-BR
    [
        Some("Selecione um item"),
        None,
        Some("Fechar"),
        None,
        Some("Anterior"),
        Some("Próximo"),
        Some("Remover"),
        Some("Caminho detalhado"),
        None,
    ],
    // sv-SE
    [
        Some("Välj en artikel"),
        None,
        Some("Stäng"),
        None,
        Some("Föregående"),
        Some("Nästa"),
        Some("Ta bort"),
        Some("Sökvägar"),
        None,
    ],
];

fn language(tag: &str) -> &str {
    tag.split(['-', '_']).next().unwrap_or(tag)
}

/// Resolves `tag` to one of [`LOCALES`]: the exact tag (case-insensitive,
/// `_` accepted for `-`), else the first built-in locale with the same
/// language, else `"en-US"`.
pub fn resolve_locale(tag: &str) -> &'static str {
    let normalized = tag.replace('_', "-");
    if let Some(exact) = LOCALES.iter().find(|l| l.eq_ignore_ascii_case(&normalized)) {
        return exact;
    }
    let lang = language(&normalized);
    LOCALES
        .iter()
        .find(|l| language(l).eq_ignore_ascii_case(lang))
        .copied()
        .unwrap_or("en-US")
}

/// The built-in string for `key` in `locale`, with the fallbacks
/// [`resolve_locale`] describes; en-US where the table has no entry.
/// Ignores application overrides; see [`ui_string`] for those.
pub fn lookup(locale: &str, key: UiString) -> &'static str {
    let resolved = resolve_locale(locale);
    LOCALES
        .iter()
        .position(|l| *l == resolved)
        .and_then(|row| row.checked_sub(1))
        .and_then(|row| TABLE[row][key.index()])
        .unwrap_or(EN[key.index()])
}

/// The application's string settings: the active locale and any overrides.
/// A GPUI global; absent means en-US with no overrides.
#[derive(Default)]
struct UiStrings {
    locale: Option<SharedString>,
    overrides: HashMap<(SharedString, UiString), SharedString>,
}

impl Global for UiStrings {}

/// Sets the locale component chrome strings resolve in, and repaints.
/// Accepts any BCP 47 tag; see [`resolve_locale`] for the fallback.
pub fn set_locale(locale: impl Into<SharedString>, cx: &mut App) {
    cx.default_global::<UiStrings>().locale = Some(locale.into());
    cx.refresh_windows();
}

/// The locale set with [`set_locale`], or `"en-US"`.
pub fn locale(cx: &App) -> SharedString {
    cx.try_global::<UiStrings>()
        .and_then(|s| s.locale.clone())
        .unwrap_or_else(|| SharedString::new_static("en-US"))
}

/// Overrides `key` for `locale`. An override keyed by a full tag (`"fr-CA"`)
/// applies to that tag only; one keyed by a bare language (`"fr"`) applies to
/// every locale of that language without an exact override. Use it to translate a string the built-in table
/// lacks, or to reword one.
pub fn set_ui_string(
    locale: impl Into<SharedString>,
    key: UiString,
    value: impl Into<SharedString>,
    cx: &mut App,
) {
    cx.default_global::<UiStrings>()
        .overrides
        .insert((locale.into(), key), value.into());
    cx.refresh_windows();
}

/// The string for `key` in the active locale: an override, else the
/// built-in table, else en-US.
pub fn ui_string(key: UiString, cx: &App) -> SharedString {
    let Some(strings) = cx.try_global::<UiStrings>() else {
        return SharedString::new_static(EN[key.index()]);
    };
    let active = strings
        .locale
        .clone()
        .unwrap_or_else(|| SharedString::new_static("en-US"));
    let lang = language(&active);
    let found = strings
        .overrides
        .get(&(active.clone(), key))
        .or_else(|| {
            strings
                .overrides
                .get(&(SharedString::from(lang.to_owned()), key))
        })
        .cloned();
    found.unwrap_or_else(|| SharedString::new_static(lookup(&active, key)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn en_us_matches_the_strings_components_drew_before() {
        for key in UiString::ALL {
            assert_eq!(lookup("en-US", key), EN[key.index()]);
        }
        assert_eq!(
            lookup("en-US", UiString::SelectPlaceholder),
            "Select an item"
        );
    }

    #[test]
    fn every_locale_resolves_every_key() {
        for locale in LOCALES {
            for key in UiString::ALL {
                assert!(!lookup(locale, key).is_empty(), "{locale} {key:?}");
            }
        }
    }

    #[test]
    fn fallback_chain() {
        assert_eq!(resolve_locale("fr_CA"), "fr-FR");
        assert_eq!(resolve_locale("PT-pt"), "pt-BR");
        assert_eq!(resolve_locale("ja-JP"), "en-US");
        assert_eq!(lookup("fr-FR", UiString::Loading), "Loading");
        assert_eq!(lookup("sv-SE", UiString::Previous), "Föregående");
    }
}
