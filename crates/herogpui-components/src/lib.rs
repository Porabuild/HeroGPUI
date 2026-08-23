//! HeroGPUI components — a faithful Rust/GPUI port of the HeroUI v3 component
//! library. One module per `@heroui/*` package.
#![allow(clippy::type_complexity)]

pub mod accordion;
pub mod alert;
pub mod alert_dialog;
pub mod anim;
pub mod autocomplete;
pub mod avatar;
pub mod badge;
pub mod breadcrumbs;
pub mod button;
pub mod button_group;
pub mod calendar;
pub mod calendar_view;
pub mod card;
pub mod checkbox;
pub mod chip;
pub mod close_button;
pub mod color_picker;
pub mod combo_box;
pub mod date_constraints;
pub mod date_picker;
pub mod disclosure;
pub mod drawer;
pub mod dropdown;
pub mod field;
pub mod form;
pub mod icons;
pub mod input;
pub mod input_group;
pub mod input_otp;
pub mod kbd;
pub mod link;
pub mod list_box;
pub mod list_nav;
pub mod meter;
pub mod modal;
pub mod number_field;
pub mod pagination;
pub mod popover;
pub mod progress;
pub mod radio_group;
pub mod range_calendar;
pub mod scroll_shadow;
pub mod select;
pub mod selection;
pub mod separator;
pub mod skeleton;
pub mod slider;
pub mod spinner;
pub mod surface;
pub mod switch;
pub mod table;
pub mod tabs;
pub mod tag_group;
pub mod textarea;
pub mod time_field;
pub mod toast;
pub mod toggle_button;
pub mod toolbar;
pub mod tooltip;
pub mod typography;
pub mod util;
pub mod validation;

// The shared v3 prop vocabularies, re-exported so `herogpui::components::*`
// is enough to build a UI.
pub use herogpui_core::{
    Backdrop, Color, FieldVariant, Orientation, Placement, PlacementAlign, Prominence,
    SelectionMode, Size, SizeXl, Variant,
};

pub use accordion::*;
pub use alert::*;
pub use alert_dialog::*;
pub use anim::*;
pub use autocomplete::*;
pub use avatar::*;
pub use badge::*;
pub use breadcrumbs::*;
pub use button::*;
pub use button_group::*;
pub use calendar::*;
pub use calendar_view::*;
pub use card::*;
pub use checkbox::*;
pub use chip::*;
pub use close_button::*;
pub use color_picker::*;
pub use combo_box::*;
pub use date_constraints::*;
pub use date_picker::*;
pub use disclosure::*;
pub use drawer::*;
pub use dropdown::*;
pub use field::*;
pub use form::*;

/// `formatOptions` for the components that take it, re-exported so a caller
/// reaches it beside the component it configures.
pub use herogpui_core::{CurrencySign, NumberFormat, NumberStyle, UnitDisplay};
pub use icons::*;
pub use input::*;
pub use input_group::*;
pub use input_otp::*;
pub use kbd::*;
pub use link::*;
pub use list_box::*;
pub use meter::*;
pub use modal::*;
pub use number_field::*;
pub use pagination::*;
pub use popover::*;
pub use progress::*;
pub use radio_group::*;
pub use range_calendar::*;
pub use scroll_shadow::*;
pub use select::*;
pub use selection::*;
pub use separator::*;
pub use skeleton::*;
pub use slider::*;
pub use spinner::*;
pub use surface::*;
pub use switch::*;
pub use table::*;
pub use tabs::*;
pub use tag_group::*;
pub use textarea::*;
pub use time_field::*;
pub use toast::*;
pub use toggle_button::*;
pub use toolbar::*;
pub use tooltip::*;
pub use typography::*;
pub use validation::*;
