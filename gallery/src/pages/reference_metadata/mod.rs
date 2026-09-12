//! Checked-in, build-time reference data sourced from HeroUI v3.
//!
//! This module is intentionally data-only. A later metadata wave can add a
//! `ReferenceMetadata` value without changing the renderer in `reference.rs`.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ImplementationStatus {
    Implemented,
    Partial,
    Unavailable,
}

impl ImplementationStatus {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Implemented => "Implemented",
            Self::Partial => "Partial",
            Self::Unavailable => "Not ported",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ApiDoc {
    // `owner` and `prop` are the pinned upstream spellings. The renderer no
    // longer shows them, but the checked-in literals stay for the .shots
    // audits and the pinned v3.2.5 contract.
    #[allow(dead_code)]
    pub(crate) owner: &'static str,
    #[allow(dead_code)]
    pub(crate) prop: &'static str,
    pub(crate) ty: &'static str,
    pub(crate) default: &'static str,
    pub(crate) description: &'static str,
    pub(crate) rust_owner: &'static str,
    pub(crate) rust: &'static str,
    pub(crate) status: ImplementationStatus,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PartDoc {
    pub(crate) name: &'static str,
    pub(crate) slot: &'static str,
    pub(crate) description: &'static str,
    pub(crate) rust_owner: &'static str,
    pub(crate) status: ImplementationStatus,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct StateDoc {
    pub(crate) state: &'static str,
    pub(crate) selector: &'static str,
    pub(crate) description: &'static str,
    pub(crate) rust: &'static str,
    pub(crate) status: ImplementationStatus,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct StyleDoc {
    pub(crate) class_or_token: &'static str,
    // Pinned upstream CSS value; hidden by the renderer, kept for the audits.
    #[allow(dead_code)]
    pub(crate) value: &'static str,
    pub(crate) description: &'static str,
    pub(crate) rust: &'static str,
    pub(crate) status: ImplementationStatus,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ReferenceMetadata {
    pub(crate) page: &'static str,
    pub(crate) import_line: &'static str,
    pub(crate) source_module: &'static str,
    pub(crate) version: &'static str,
    pub(crate) docs_source: &'static str,
    pub(crate) api_source: &'static str,
    pub(crate) style_source: &'static str,
    pub(crate) required_parts: &'static [&'static str],
    pub(crate) api: &'static [ApiDoc],
    pub(crate) parts: &'static [PartDoc],
    pub(crate) states: &'static [StateDoc],
    pub(crate) styling: &'static [StyleDoc],
}

mod buttons;
mod collections;
mod colors;
mod controls;
mod data_display;
mod date_and_time;
mod feedback;
mod forms;
mod layout;
mod media;
mod navigation;
mod overlays;
mod pickers;
mod typography;
mod utilities;

use buttons::*;
use collections::*;
use colors::*;
use controls::*;
use data_display::*;
use date_and_time::*;
use feedback::*;
use forms::*;
use layout::*;
use media::*;
use navigation::*;
use overlays::*;
use pickers::*;
use typography::*;
use utilities::*;

pub(crate) const ALL: &[ReferenceMetadata] = &[
    DROPDOWN,
    LIST_BOX,
    TAG_GROUP,
    BADGE,
    CHIP,
    COLOR_AREA,
    COLOR_SLIDER,
    COLOR_SWATCH,
    COLOR_SWATCH_PICKER,
    TOAST,
    COLOR_PICKER,
    COLOR_FIELD,
    SLIDER,
    BUTTON,
    BUTTON_GROUP,
    CHECKBOX,
    CHECKBOX_GROUP,
    RADIO_GROUP,
    CLOSE_BUTTON,
    TOGGLE_BUTTON,
    SWITCH,
    PAGINATION,
    INPUT,
    TEXT_FIELD,
    TEXT_AREA,
    SEARCH_FIELD,
    INPUT_OTP,
    INPUT_GROUP,
    TABS,
    CALENDAR,
    DATE_FIELD,
    DATE_PICKER,
    DATE_RANGE_PICKER,
    NUMBER_FIELD,
    TIME_FIELD,
    RANGE_CALENDAR,
    DRAWER,
    MODAL,
    COMBO_BOX,
    AUTOCOMPLETE,
    PROGRESS_BAR,
    PROGRESS_CIRCLE,
    METER,
    SKELETON,
    SPINNER,
    SEPARATOR,
    SELECT,
    POPOVER,
    TOOLTIP,
    TABLE,
    ACCORDION,
    DISCLOSURE,
    TOOLBAR,
    FORM,
    ALERT_DIALOG,
    BREADCRUMBS,
    CARD,
    SURFACE,
    KBD,
    TYPOGRAPHY,
    ALERT,
    LINK,
    AVATAR,
    SCROLL_SHADOW,
    FIELDSET,
    FIELD_SLOTS,
];

pub(crate) fn for_import(import_line: &str) -> Option<&'static ReferenceMetadata> {
    ALL.iter()
        .find(|metadata| metadata.import_line == import_line)
}

#[cfg(test)]
pub(crate) fn for_route(page: &str, import_line: &str) -> Option<&'static ReferenceMetadata> {
    ALL.iter()
        .find(|metadata| metadata.page == page && metadata.import_line == import_line)
}
