//! Page routing and navigation registry for the HeroGPUI gallery.
//!
//! The route list mirrors HeroUI's getting-started, component and release
//! documentation, with fifteen component categories.

pub mod components;
pub mod docs;
pub mod docs_guides;
mod overview;
mod reference;
mod releases;

use gpui::{px, App};
use herogpui_theme::ActiveTheme;

use crate::app::Gallery;

/// Dispatches the active page to its renderer.
impl Gallery {
    pub fn render_current_page(&mut self, cx: &mut Context<'_, Self>) -> gpui::AnyElement {
        match self.page {
            // Overview
            Page::AllComponents => self.page_all_components(cx),

            // Releases
            Page::Releases => self.page_releases(cx),
            Page::ReleaseCurrent => self.page_release_current(cx),

            // Getting started
            Page::Introduction => self.page_introduction(cx),
            Page::Installation => self.page_installation(cx),
            Page::Theming => self.page_theming(cx),
            Page::DarkMode => self.page_dark_mode(cx),
            Page::Customization => self.page_customization(cx),
            Page::Styling => self.page_styling(cx),
            Page::DesignPrinciples => self.page_design_principles(cx),

            // Buttons
            Page::Button => self.page_button(cx),
            Page::ButtonGroup => self.page_button_group(cx),
            Page::CloseButton => self.page_close_button(cx),
            Page::ToggleButton => self.page_toggle_button(cx),

            // Collections
            Page::Dropdown => self.page_dropdown(cx),
            Page::ListBox => self.page_list_box(cx),
            Page::TagGroup => self.page_tag_group(cx),

            // Colors
            Page::ColorArea => self.page_color_area(cx),
            Page::ColorField => self.page_color_field(cx),
            Page::ColorPicker => self.page_color_picker(cx),
            Page::ColorSlider => self.page_color_slider(cx),
            Page::ColorSwatch => self.page_color_swatch(cx),
            Page::ColorSwatchPicker => self.page_color_swatch_picker(cx),

            // Controls
            Page::Slider => self.page_slider(cx),
            Page::Switch => self.page_switch(cx),

            // Data display
            Page::Badge => self.page_badge(cx),
            Page::Chip => self.page_chip(cx),
            Page::Table => self.page_table(cx),

            // Date and time
            Page::Calendar => self.page_calendar(cx),
            Page::DateField => self.page_date_field(cx),
            Page::DatePicker => self.page_date_picker(cx),
            Page::DateRangePicker => self.page_date_range_picker(cx),
            Page::RangeCalendar => self.page_range_calendar(cx),
            Page::TimeField => self.page_time_field(cx),

            // Feedback
            Page::Alert => self.page_alert(cx),
            Page::Meter => self.page_meter(cx),
            Page::ProgressBar => self.page_progress_bar(cx),
            Page::ProgressCircle => self.page_progress_circle(cx),
            Page::Skeleton => self.page_skeleton(cx),
            Page::Spinner => self.page_spinner(cx),

            // Forms
            Page::Checkbox => self.page_checkbox(cx),
            Page::CheckboxGroup => self.page_checkbox_group(cx),
            Page::Fieldset => self.page_fieldset(cx),
            Page::FieldSlots => self.page_field_slots(cx),
            Page::Form => self.page_form(cx),
            Page::Input => self.page_input(cx),
            Page::InputGroup => self.page_input_group(cx),
            Page::InputOtp => self.page_input_otp(cx),
            Page::NumberField => self.page_number_field(cx),
            Page::RadioGroup => self.page_radio_group(cx),
            Page::SearchField => self.page_search_field(cx),
            Page::TextArea => self.page_text_area(cx),
            Page::TextField => self.page_text_field(cx),

            // Layout
            Page::Card => self.page_card(cx),
            Page::Separator => self.page_separator(cx),
            Page::Surface => self.page_surface(cx),
            Page::Toolbar => self.page_toolbar(cx),

            // Media
            Page::Avatar => self.page_avatar(cx),

            // Navigation
            Page::Accordion => self.page_accordion(cx),
            Page::Breadcrumbs => self.page_breadcrumbs(cx),
            Page::Disclosure => self.page_disclosure(cx),
            Page::Link => self.page_link(cx),
            Page::Pagination => self.page_pagination(cx),
            Page::Tabs => self.page_tabs(cx),

            // Overlays
            Page::AlertDialog => self.page_alert_dialog(cx),
            Page::Drawer => self.page_drawer(cx),
            Page::Modal => self.page_modal(cx),
            Page::Popover => self.page_popover(cx),
            Page::Toast => self.page_toast(cx),
            Page::Tooltip => self.page_tooltip(cx),

            // Pickers
            Page::Autocomplete => self.page_autocomplete(cx),
            Page::ComboBox => self.page_combo_box(cx),
            Page::Select => self.page_select(cx),

            // Typography
            Page::Kbd => self.page_kbd(cx),
            Page::Typography => self.page_typography(cx),

            // Utilities
            Page::ScrollShadow => self.page_scroll_shadow(cx),
        }
    }
}

/// Every route of the gallery — mirrors heroui.com/docs/react/components.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    // Overview
    AllComponents,

    // Releases
    Releases,
    ReleaseCurrent,

    // Getting started
    Introduction,
    Installation,
    Theming,
    DarkMode,
    Customization,
    Styling,
    DesignPrinciples,

    // Buttons
    Button,
    ButtonGroup,
    CloseButton,
    ToggleButton,

    // Collections
    Dropdown,
    ListBox,
    TagGroup,

    // Colors
    ColorArea,
    ColorField,
    ColorPicker,
    ColorSlider,
    ColorSwatch,
    ColorSwatchPicker,

    // Controls
    Slider,
    Switch,

    // Data display
    Badge,
    Chip,
    Table,

    // Date and time
    Calendar,
    DateField,
    DatePicker,
    DateRangePicker,
    RangeCalendar,
    TimeField,

    // Feedback
    Alert,
    Meter,
    ProgressBar,
    ProgressCircle,
    Skeleton,
    Spinner,

    // Forms
    Checkbox,
    CheckboxGroup,
    Fieldset,
    /// `Label` / `Description` / `ErrorMessage` / `FieldError` on one page.
    FieldSlots,
    Form,
    Input,
    InputGroup,
    InputOtp,
    NumberField,
    RadioGroup,
    SearchField,
    TextArea,
    TextField,

    // Layout
    Card,
    Separator,
    Surface,
    Toolbar,

    // Media
    Avatar,

    // Navigation
    Accordion,
    Breadcrumbs,
    Disclosure,
    Link,
    Pagination,
    Tabs,

    // Overlays
    AlertDialog,
    Drawer,
    Modal,
    Popover,
    Toast,
    Tooltip,

    // Pickers
    Autocomplete,
    ComboBox,
    Select,

    // Typography
    Kbd,
    Typography,

    // Utilities
    ScrollShadow,
}

impl Page {
    pub fn title(self) -> &'static str {
        match self {
            Page::AllComponents => "All Components",
            Page::Releases => "Releases",
            Page::ReleaseCurrent => concat!("v", env!("CARGO_PKG_VERSION")),
            Page::Introduction => "Introduction",
            Page::Installation => "Installation",
            Page::Theming => "Theming",
            Page::DarkMode => "Dark Mode",
            Page::Customization => "Customization",
            Page::Styling => "Styling",
            Page::DesignPrinciples => "Design Principles",
            Page::Button => "Button",
            Page::ButtonGroup => "Button Group",
            Page::CloseButton => "Close Button",
            Page::ToggleButton => "Toggle Button",
            Page::Dropdown => "Dropdown",
            Page::ListBox => "List Box",
            Page::TagGroup => "Tag Group",
            Page::ColorArea => "Color Area",
            Page::ColorField => "Color Field",
            Page::ColorPicker => "Color Picker",
            Page::ColorSlider => "Color Slider",
            Page::ColorSwatch => "Color Swatch",
            Page::ColorSwatchPicker => "Color Swatch Picker",
            Page::Slider => "Slider",
            Page::Switch => "Switch",
            Page::Badge => "Badge",
            Page::Chip => "Chip",
            Page::Table => "Table",
            Page::Calendar => "Calendar",
            Page::DateField => "Date Field",
            Page::DatePicker => "Date Picker",
            Page::DateRangePicker => "Date Range Picker",
            Page::RangeCalendar => "Range Calendar",
            Page::TimeField => "Time Field",
            Page::Alert => "Alert",
            Page::Meter => "Meter",
            Page::ProgressBar => "Progress Bar",
            Page::ProgressCircle => "Progress Circle",
            Page::Skeleton => "Skeleton",
            Page::Spinner => "Spinner",
            Page::Checkbox => "Checkbox",
            Page::CheckboxGroup => "Checkbox Group",
            Page::Fieldset => "Fieldset",
            Page::FieldSlots => "Label & Messages",
            Page::Form => "Form",
            Page::Input => "Input",
            Page::InputGroup => "Input Group",
            Page::InputOtp => "Input OTP",
            Page::NumberField => "Number Field",
            Page::RadioGroup => "Radio Group",
            Page::SearchField => "Search Field",
            Page::TextArea => "Text Area",
            Page::TextField => "Text Field",
            Page::Card => "Card",
            Page::Separator => "Separator",
            Page::Surface => "Surface",
            Page::Toolbar => "Toolbar",
            Page::Avatar => "Avatar",
            Page::Accordion => "Accordion",
            Page::Breadcrumbs => "Breadcrumbs",
            Page::Disclosure => "Disclosure",
            Page::Link => "Link",
            Page::Pagination => "Pagination",
            Page::Tabs => "Tabs",
            Page::AlertDialog => "Alert Dialog",
            Page::Drawer => "Drawer",
            Page::Modal => "Modal",
            Page::Popover => "Popover",
            Page::Toast => "Toast",
            Page::Tooltip => "Tooltip",
            Page::Autocomplete => "Autocomplete",
            Page::ComboBox => "Combo Box",
            Page::Select => "Select",
            Page::Kbd => "Kbd",
            Page::Typography => "Typography",
            Page::ScrollShadow => "Scroll Shadow",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Page::AllComponents => {
                "Explore every component in the library, grouped by what it helps you build."
            }
            Page::Releases => {
                "Updates to HeroGPUI, including new components, fixes and documentation changes."
            }
            Page::ReleaseCurrent => {
                "The current workspace release, with its component, documentation and verification highlights."
            }
            Page::Introduction => "Beautiful, fast and modern cross-platform UI library for Rust GPUI. A faithful port of HeroUI v3.",
            Page::Installation => "Get HeroGPUI running in your GPUI application in minutes.",
            Page::Theming => "The OKLCH semantic token system shared by every component.",
            Page::DarkMode => "Switch between the light and dark appearance at runtime.",
            Page::Customization => "Build custom themes by overriding a handful of base tokens.",
            Page::Styling => "Where a v3 `className` goes when there are no classes: props, tokens and slots.",
            Page::DesignPrinciples => "The ten principles v3 is built on, and how each one lands in a gpui port.",
            Page::Button => "A pressable button with variants and states.",
            Page::ButtonGroup => "Group related buttons with a shared variant and merged edges.",
            Page::CloseButton => "A button for dismissing dialogs, modals and inline content.",
            Page::ToggleButton => "Toggle between a selected and unselected state.",
            Page::Dropdown => "Display a menu of actions anchored to a trigger.",
            Page::ListBox => "A list of options, nonselecting until a selection mode is set.",
            Page::TagGroup => "A focusable list of tags with selection and removal.",
            Page::ColorArea => "Pick saturation and lightness from a two-dimensional gradient.",
            Page::ColorField => "Enter a color value as text.",
            Page::ColorPicker => "A complete color picking surface in a popover.",
            Page::ColorSlider => "Adjust a single channel of a color.",
            Page::ColorSwatch => "Preview a single color value.",
            Page::ColorSwatchPicker => "Choose from a predefined set of colors.",
            Page::Slider => "Select a value from a range by dragging.",
            Page::Switch => "Toggle a single setting on or off.",
            Page::Badge => "Display a count or status marker on its children.",
            Page::Chip => "A compact element for tags and filters.",
            Page::Table => "Display rows of data in columns.",
            Page::Calendar => "A month grid for picking a single date.",
            Page::DateField => "Enter a date segment by segment.",
            Page::DatePicker => "A date field with a calendar popover.",
            Page::DateRangePicker => "Pick a start and end date together.",
            Page::RangeCalendar => "A month grid for picking a date range.",
            Page::TimeField => "Enter a time segment by segment.",
            Page::Alert => "Display an important inline message.",
            Page::Meter => "Show a value within a known range.",
            Page::ProgressBar => "Show determinate or indeterminate progress.",
            Page::ProgressCircle => "Show progress along a circular arc.",
            Page::Skeleton => "A placeholder shimmer shown while content loads.",
            Page::Spinner => "Indicate a busy state with a rotating arc.",
            Page::Checkbox => "Select one or more values from a set.",
            Page::CheckboxGroup => "A group of checkboxes with shared state.",
            Page::Fieldset => "Group related form controls under a legend.",
            Page::FieldSlots => "The label, description and error slots every field composes.",
            Page::Form => "Group fields with validation and submit handling.",
            Page::Input => "A single-line text input.",
            Page::InputGroup => "Combine an input with adjacent addons and controls.",
            Page::InputOtp => "A segmented one-time-password input.",
            Page::NumberField => "A numeric field with steppers, range and step.",
            Page::RadioGroup => "Select exactly one value from a set.",
            Page::SearchField => "A text input specialised for search, with a clear action.",
            Page::TextArea => "A multi-line text input.",
            Page::TextField => "A composition-friendly field with label and validation.",
            Page::Card => "Group related content and actions on a surface.",
            Page::Separator => "Separate content with a horizontal or vertical line.",
            Page::Surface => "A container that applies surface-level styling to its children.",
            Page::Toolbar => "A container for interactive controls with arrow-key navigation.",
            Page::Avatar => "Display an image or initials representing a user.",
            Page::Accordion => "Vertically collapsing panels.",
            Page::Breadcrumbs => "Show the path to the current resource.",
            Page::Disclosure => "A single collapsible section.",
            Page::Link => "Navigate or open external resources.",
            Page::Pagination => "Navigate between pages of content.",
            Page::Tabs => "Organise content into switchable panels.",
            Page::AlertDialog => "A modal for critical confirmations.",
            Page::Drawer => "A slide-over panel anchored to a window edge.",
            Page::Modal => "Display a dialog over the page content.",
            Page::Popover => "A floating panel anchored to a trigger.",
            Page::Toast => "Transient notifications stacked in a corner.",
            Page::Tooltip => "Contextual information shown on hover or focus.",
            Page::Autocomplete => {
                "An autocomplete combines a select with filtering, allowing users \
                 to search and select from a list of options."
            }
            Page::ComboBox => "A text input combined with a selectable list.",
            Page::Select => "Pick one value from a dropdown list.",
            Page::Kbd => "Display keyboard key combinations.",
            Page::Typography => "Semantic typography primitives for headings, body and code.",
            Page::ScrollShadow => "A scrollable area with soft fading edges.",
        }
    }

    pub fn import_line(self) -> &'static str {
        match self {
            Page::Button => "use herogpui::prelude::{Button, Size, Variant};",
            Page::ButtonGroup => "use herogpui::components::button_group::ButtonGroup;",
            Page::CloseButton => "use herogpui::components::close_button::CloseButton;",
            Page::ToggleButton => {
                "use herogpui::components::toggle_button::{ToggleButton, ToggleButtonGroup};"
            }
            Page::Dropdown => "use herogpui::components::dropdown::{Dropdown, MenuItem};",
            Page::ListBox => "use herogpui::components::list_box::{ListBox, ListBoxItem};",
            Page::TagGroup => "use herogpui::components::tag_group::{Tag, TagGroup};",
            Page::ColorArea => "use herogpui::components::color_picker::ColorArea;",
            Page::ColorField => "use herogpui::components::color_picker::ColorField;",
            Page::ColorPicker => "use herogpui::components::color_picker::ColorPicker;",
            Page::ColorSlider => {
                "use herogpui::components::color_picker::{ColorChannel, ColorSlider};"
            }
            Page::ColorSwatch => "use herogpui::components::color_picker::ColorSwatch;",
            Page::ColorSwatchPicker => "use herogpui::components::color_picker::ColorSwatchPicker;",
            Page::Slider => "use herogpui::components::slider::Slider;",
            Page::Switch => "use herogpui::components::switch::Switch;",
            Page::Badge => "use herogpui::components::badge::{Badge, BadgeAnchor, BadgeLabel};",
            Page::Chip => "use herogpui::components::chip::{Chip, ChipLabel};",
            Page::Table => "use herogpui::components::table::Table;",
            Page::Calendar => "use herogpui::components::calendar::{Calendar, CalendarState};",
            Page::DateField => "use herogpui::components::date_picker::DateField;",
            Page::DatePicker => "use herogpui::components::date_picker::DatePicker;",
            Page::DateRangePicker => {
                "use herogpui::components::date_picker::{DateRangePicker, DateRangeState};"
            }
            Page::RangeCalendar => "use herogpui::components::range_calendar::RangeCalendar;",
            Page::TimeField => "use herogpui::components::time_field::{TimeField, TimeState};",
            Page::Alert => "use herogpui::components::alert::Alert;",
            Page::Meter => "use herogpui::components::meter::Meter;",
            Page::ProgressBar => "use herogpui::components::progress::ProgressBar;",
            Page::ProgressCircle => "use herogpui::components::progress::ProgressCircle;",
            Page::Skeleton => "use herogpui::components::skeleton::Skeleton;",
            Page::Spinner => "use herogpui::components::spinner::Spinner;",
            Page::Checkbox => "use herogpui::components::checkbox::Checkbox;",
            Page::CheckboxGroup => "use herogpui::components::checkbox::CheckboxGroup;",
            Page::Fieldset => {
                "use herogpui::components::field::{Fieldset, FieldGroup, FieldsetLegend, FieldsetActions};"
            }
            Page::FieldSlots => {
                "use herogpui::components::field::{Description, ErrorMessage, FieldError, Label};"
            }
            Page::Form => "use herogpui::components::form::Form;",
            Page::Input => "use herogpui::components::input::{Input, InputState};",
            Page::InputGroup => "use herogpui::components::input_group::InputGroup;",
            Page::InputOtp => "use herogpui::components::input_otp::{InputOTP, OtpState};",
            Page::NumberField => {
                "use herogpui::components::number_field::{NumberField, NumberState};"
            }
            Page::RadioGroup => "use herogpui::components::radio_group::RadioGroup;",
            Page::SearchField => "use herogpui::components::input::SearchField;",
            Page::TextArea => "use herogpui::components::textarea::TextArea;",
            Page::TextField => "use herogpui::components::input::TextField;",
            Page::Card => {
                "use herogpui::components::card::{Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle};"
            }
            Page::Separator => "use herogpui::components::separator::Separator;",
            Page::Surface => "use herogpui::components::surface::Surface;",
            Page::Toolbar => "use herogpui::components::toolbar::Toolbar;",
            Page::Avatar => "use herogpui::components::avatar::Avatar;",
            Page::Accordion => "use herogpui::components::accordion::{Accordion, AccordionItem};",
            Page::Breadcrumbs => "use herogpui::components::breadcrumbs::{Breadcrumbs, Crumb};",
            Page::Disclosure => {
                "use herogpui::components::disclosure::{Disclosure, DisclosureGroup};"
            }
            Page::Link => "use herogpui::components::link::Link;",
            Page::Pagination => "use herogpui::components::pagination::Pagination;",
            Page::Tabs => "use herogpui::components::tabs::{TabItem, Tabs};",
            Page::AlertDialog => "use herogpui::components::alert_dialog::{AlertDialog, AlertDialogCloseTrigger};",
            Page::Drawer => "use herogpui::components::drawer::{Drawer, DrawerCloseTrigger, DrawerPlacement};",
            Page::Modal => "use herogpui::components::modal::{Modal, ModalCloseTrigger, ModalSize};",
            Page::Popover => "use herogpui::components::popover::{Popover, PopoverArrow, PopoverPlacement};",
            Page::Toast => "use herogpui::components::toast::{Toast, ToastViewport};",
            Page::Tooltip => "use herogpui::components::tooltip::Tooltip;",
            Page::Autocomplete => "use herogpui::components::autocomplete::Autocomplete;",
            Page::ComboBox => "use herogpui::components::combo_box::ComboBox;",
            Page::Select => "use herogpui::components::select::Select;",
            Page::Kbd => "use herogpui::components::kbd::Kbd;",
            Page::Typography => {
                "use herogpui::components::typography::{Typography, TypographyType};"
            }
            Page::ScrollShadow => "use herogpui::components::scroll_shadow::ScrollShadow;",
            _ => "",
        }
    }

    pub fn docs_root(self) -> Self {
        if matches!(self, Page::Releases | Page::ReleaseCurrent) {
            Page::Releases
        } else if self == Page::AllComponents || !self.import_line().is_empty() {
            Page::AllComponents
        } else {
            Page::Introduction
        }
    }
}

/// Sidebar section definition.
pub struct NavSection {
    pub title: &'static str,
    pub items: Vec<Page>,
}

pub fn nav_sections() -> Vec<NavSection> {
    vec![
        NavSection {
            title: "Overview",
            items: vec![Page::AllComponents],
        },
        NavSection {
            title: "Overview",
            items: vec![Page::Releases],
        },
        NavSection {
            title: "Versions",
            items: vec![Page::ReleaseCurrent],
        },
        NavSection {
            title: "Getting Started",
            items: vec![
                Page::Introduction,
                Page::Installation,
                Page::Theming,
                Page::DarkMode,
                Page::Customization,
                Page::Styling,
                Page::DesignPrinciples,
            ],
        },
        NavSection {
            title: "Buttons",
            items: vec![
                Page::Button,
                Page::ButtonGroup,
                Page::CloseButton,
                Page::ToggleButton,
            ],
        },
        NavSection {
            title: "Collections",
            items: vec![Page::Dropdown, Page::ListBox, Page::TagGroup],
        },
        NavSection {
            title: "Colors",
            items: vec![
                Page::ColorArea,
                Page::ColorField,
                Page::ColorPicker,
                Page::ColorSlider,
                Page::ColorSwatch,
                Page::ColorSwatchPicker,
            ],
        },
        NavSection {
            title: "Controls",
            items: vec![Page::Slider, Page::Switch],
        },
        NavSection {
            title: "Data Display",
            items: vec![Page::Badge, Page::Chip, Page::Table],
        },
        NavSection {
            title: "Date and Time",
            items: vec![
                Page::Calendar,
                Page::DateField,
                Page::DatePicker,
                Page::DateRangePicker,
                Page::RangeCalendar,
                Page::TimeField,
            ],
        },
        NavSection {
            title: "Feedback",
            items: vec![
                Page::Alert,
                Page::Meter,
                Page::ProgressBar,
                Page::ProgressCircle,
                Page::Skeleton,
                Page::Spinner,
            ],
        },
        NavSection {
            title: "Forms",
            items: vec![
                Page::Checkbox,
                Page::CheckboxGroup,
                Page::Fieldset,
                Page::FieldSlots,
                Page::Form,
                Page::Input,
                Page::InputGroup,
                Page::InputOtp,
                Page::NumberField,
                Page::RadioGroup,
                Page::SearchField,
                Page::TextArea,
                Page::TextField,
            ],
        },
        NavSection {
            title: "Layout",
            items: vec![Page::Card, Page::Separator, Page::Surface, Page::Toolbar],
        },
        NavSection {
            title: "Media",
            items: vec![Page::Avatar],
        },
        NavSection {
            title: "Navigation",
            items: vec![
                Page::Accordion,
                Page::Breadcrumbs,
                Page::Disclosure,
                Page::Link,
                Page::Pagination,
                Page::Tabs,
            ],
        },
        NavSection {
            title: "Overlays",
            items: vec![
                Page::AlertDialog,
                Page::Drawer,
                Page::Modal,
                Page::Popover,
                Page::Toast,
                Page::Tooltip,
            ],
        },
        NavSection {
            title: "Pickers",
            items: vec![Page::Autocomplete, Page::ComboBox, Page::Select],
        },
        NavSection {
            title: "Typography",
            items: vec![Page::Kbd, Page::Typography],
        },
        NavSection {
            title: "Utilities",
            items: vec![Page::ScrollShadow],
        },
    ]
}

// ---------------------------------------------------------------------------
// Shared documentation building blocks
// ---------------------------------------------------------------------------

use gpui::prelude::*;

const JSX_MARKERS: &[&str] = &[
    "className=",
    "<Button",
    "</",
    "<Slider",
    "<InputGroup",
    "import {",
    "import React",
    "from \"react\"",
    "from 'react'",
    "React.",
    "useState",
    "data-[",
    "key={",
];
const GENERIC_JSX_MARKER: &str = "uppercase JSX component tag";
const INVALID_CODE_BLOCK: &str = "// This example contains unsupported React/JSX syntax.";

fn jsx_marker(code: &str) -> Option<&'static str> {
    let bytes = code.as_bytes();
    let mut i = 0usize;
    let mut saw_generic_tag = false;

    while i < bytes.len() {
        if let Some(end) = skip_raw_string(code, i) {
            i = end;
            continue;
        }
        match bytes[i] {
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                i = skip_line_comment(code, i);
                continue;
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                i = skip_block_comment(code, i);
                continue;
            }
            b'"' => {
                i = skip_quoted(code, i, b'"');
                continue;
            }
            b'\'' => {
                i = skip_char_or_lifetime(code, i);
                continue;
            }
            _ => {}
        }

        let rest = &code[i..];
        if let Some(marker) = JSX_MARKERS.iter().find(|marker| rest.starts_with(**marker)) {
            return Some(marker);
        }
        if bytes[i] == b'<' && looks_like_jsx_component_tag(code, i) {
            saw_generic_tag = true;
        }
        i += code[i..].chars().next().map_or(1, char::len_utf8);
    }

    saw_generic_tag.then_some(GENERIC_JSX_MARKER)
}

fn checked_code(code: &str) -> &str {
    if jsx_marker(code).is_some() {
        INVALID_CODE_BLOCK
    } else {
        code
    }
}

fn skip_line_comment(code: &str, start: usize) -> usize {
    code[start..]
        .find('\n')
        .map_or(code.len(), |offset| start + offset)
}

fn skip_block_comment(code: &str, start: usize) -> usize {
    let bytes = code.as_bytes();
    let mut depth = 1usize;
    let mut i = start + 2;
    while i + 1 < bytes.len() {
        if bytes[i] == b'/' && bytes[i + 1] == b'*' {
            depth += 1;
            i += 2;
        } else if bytes[i] == b'*' && bytes[i + 1] == b'/' {
            depth -= 1;
            i += 2;
            if depth == 0 {
                return i;
            }
        } else {
            i += code[i..].chars().next().map_or(1, char::len_utf8);
        }
    }
    bytes.len()
}

fn skip_quoted(code: &str, start: usize, quote: u8) -> usize {
    let bytes = code.as_bytes();
    let mut i = start + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i = (i + 2).min(bytes.len()),
            value if value == quote => return i + 1,
            _ => i += code[i..].chars().next().map_or(1, char::len_utf8),
        }
    }
    bytes.len()
}

fn skip_char_or_lifetime(code: &str, start: usize) -> usize {
    let bytes = code.as_bytes();
    let Some(&next) = bytes.get(start + 1) else {
        return bytes.len();
    };
    if next.is_ascii_alphabetic() || next == b'_' {
        let char_end = start + 1 + code[start + 1..].chars().next().map_or(1, char::len_utf8);
        if bytes.get(char_end) == Some(&b'\'') {
            return char_end + 1;
        }
        return scan_ascii_ident(bytes, start + 1);
    }
    if !next.is_ascii() {
        let char_end = start + 1 + code[start + 1..].chars().next().map_or(1, char::len_utf8);
        if bytes.get(char_end) == Some(&b'\'') {
            return char_end + 1;
        }
    }
    skip_quoted(code, start, b'\'')
}

fn skip_raw_string(code: &str, start: usize) -> Option<usize> {
    let bytes = code.as_bytes();
    if bytes.get(start) != Some(&b'r') {
        return None;
    }
    let mut hashes = 0usize;
    let mut quote = start + 1;
    while bytes.get(quote) == Some(&b'#') {
        hashes += 1;
        quote += 1;
    }
    if bytes.get(quote) != Some(&b'"') {
        return None;
    }

    let mut i = quote + 1;
    while i < bytes.len() {
        if bytes[i] == b'"'
            && bytes.get(i + 1..i + 1 + hashes) == Some(&bytes[quote + 1..quote + 1 + hashes])
        {
            return Some(i + 1 + hashes);
        }
        i += code[i..].chars().next().map_or(1, char::len_utf8);
    }
    Some(bytes.len())
}

fn scan_ascii_ident(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
        i += 1;
    }
    i
}

fn looks_like_jsx_component_tag(code: &str, start: usize) -> bool {
    let bytes = code.as_bytes();
    let mut i = start + 1;
    let closing = bytes.get(i) == Some(&b'/');
    if closing {
        i += 1;
    }
    if !bytes.get(i).is_some_and(u8::is_ascii_uppercase) {
        return false;
    }

    let name_start = i;
    loop {
        i = scan_ascii_ident(bytes, i);
        if bytes.get(i) != Some(&b'.') {
            break;
        }
        i += 1;
        if !bytes.get(i).is_some_and(u8::is_ascii_uppercase) {
            return false;
        }
    }
    if i == name_start {
        return false;
    }

    while bytes.get(i).is_some_and(u8::is_ascii_whitespace) {
        i += 1;
    }
    if closing {
        return bytes.get(i) == Some(&b'>');
    }
    if bytes.get(i) == Some(&b'/') && bytes.get(i + 1) == Some(&b'>') {
        return true;
    }
    if bytes.get(i) == Some(&b'>') {
        return allows_jsx_open_context(code, start);
    }

    let mut brace_depth = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'"' | b'\'' => i = skip_quoted(code, i, bytes[i]),
            b'{' => {
                brace_depth += 1;
                i += 1;
            }
            b'}' if brace_depth > 0 => {
                brace_depth -= 1;
                i += 1;
            }
            b'=' if brace_depth == 0 => return true,
            b'>' if brace_depth == 0 => return false,
            b'/' if brace_depth == 0 && bytes.get(i + 1) == Some(&b'>') => return false,
            _ => i += code[i..].chars().next().map_or(1, char::len_utf8),
        }
    }
    false
}

fn allows_jsx_open_context(code: &str, start: usize) -> bool {
    let prefix = code[..start].trim_end_matches(|c: char| c.is_ascii_whitespace());
    let Some((_, previous)) = prefix.char_indices().next_back() else {
        return true;
    };
    if previous.is_ascii_alphanumeric() || previous == '_' {
        let prefix = prefix.trim_end_matches(|c: char| c.is_ascii_alphanumeric() || c == '_');
        let word = prefix
            .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .map_or(prefix, |index| &prefix[index + 1..]);
        return word == "return";
    }
    !matches!(previous, ':')
}

/// Standard docs page layout: title, description, optional import snippet and
/// a list of (heading, live-example) sections.
pub fn doc_page(
    title: &str,
    description: &str,
    import_line: &str,
    sections: Vec<(&str, gpui::AnyElement)>,
    cx: &App,
) -> gpui::AnyElement {
    let mut el = doc_page_shell(title, description, import_line, cx);

    // `HEROGPUI_SECTION=Sorting` renders only the sections whose title contains
    // that text, case-insensitively. A page is long, a window is at most a
    // monitor tall, and scrolling to a section by wheel notches to photograph it
    // is both slow and fragile -- naming it puts it at the top of an otherwise
    // empty page instead. Several names can be given, comma separated.
    //
    // The filter is a global rather than an env read, so the control file can
    // change it while the app runs (see `control.rs`).
    for (heading, body) in sections {
        if !crate::control::section_wanted(heading, cx) {
            continue;
        }
        el = el
            .mt(px(4.))
            .child(section_heading(heading))
            .child(example_frame(body, cx));
    }

    el.into_any_element()
}

fn doc_page_shell(title: &str, description: &str, import_line: &str, cx: &App) -> gpui::Div {
    let colors = cx.colors();
    let mut el = gpui::div()
        .w(px(860.))
        .max_w(gpui::relative(1.))
        .flex()
        .flex_col()
        .gap(px(20.))
        .child(
            gpui::div()
                .text_size(px(30.))
                .line_height(px(38.))
                .font_weight(gpui::FontWeight::BOLD)
                .child(title.to_owned()),
        )
        .child(
            gpui::div()
                .text_size(px(15.5))
                .line_height(px(26.))
                .text_color(colors.muted)
                .child(description.to_owned()),
        );

    if !import_line.is_empty() {
        el = el.child(code_block(import_line, cx));
    }
    el
}

/// Component docs use the same page layout, with the Rust expression that
/// renders each live example shown directly beneath it.
pub fn component_doc_page(
    title: &str,
    description: &str,
    import_line: &str,
    sections: Vec<(&str, gpui::AnyElement, &str)>,
    cx: &App,
) -> gpui::AnyElement {
    let references = reference::panels(import_line, &sections, cx);
    let mut el = doc_page_shell(title, description, import_line, cx);

    for (heading, body, code) in sections {
        if !crate::control::section_wanted(heading, cx) {
            continue;
        }
        el = el
            .mt(px(4.))
            .child(section_heading(heading))
            .child(example_frame_with_code(body, checked_code(code), cx));
    }

    for (heading, body) in references {
        if crate::control::section_wanted(heading, cx) {
            el = el.mt(px(4.)).child(section_heading(heading)).child(body);
        }
    }

    el.into_any_element()
}

pub fn section_heading(text: &str) -> gpui::Div {
    gpui::div().child(
        gpui::div()
            .text_size(px(20.))
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .child(text.to_owned()),
    )
}

/// Bordered live-demo container like HeroUI's "Usage" preview cards.
pub fn example_frame(content: gpui::AnyElement, cx: &App) -> gpui::AnyElement {
    let colors = cx.colors();
    gpui::div()
        .p(px(28.))
        .rounded(px(14.))
        .border_1()
        .border_color(colors.border)
        .bg(colors.background)
        .shadow(cx.layout().surface_shadow.clone())
        .flex()
        .flex_col()
        .gap(px(16.))
        .child(content)
        .into_any_element()
}

/// Quiet code-editor presentation shared by every docs code surface: a dark
/// panel with a hairline border, one mono size/leading pair, and no
/// decoration beyond the syntax colors.
const CODE_BG: u32 = 0x18181B;
const CODE_BORDER: u32 = 0x27272A;
const CODE_TEXT: u32 = 0xD4D4D8;
const CODE_TEXT_SIZE: f32 = 12.5;
const CODE_LINE_HEIGHT: f32 = 21.;
const CODE_PAD_X: f32 = 18.;
const CODE_PAD_Y: f32 = 14.;
const CODE_RADIUS: f32 = 12.;

/// The display-ready, highlighted text of a snippet: `stringify!` one-liners
/// are reflowed into rustfmt-shaped lines before being tokenized, so the
/// highlight spans land on the readable form the user sees.
fn highlighted_code(code: &str) -> gpui::StyledText {
    let shown = crate::highlight::format_rust_snippet(code);
    let highlights = crate::highlight::rust_highlights(&shown);
    gpui::StyledText::new(shown).with_highlights(highlights)
}

/// Live preview and its exact gallery source, presented as one example card.
pub fn example_frame_with_code(
    content: gpui::AnyElement,
    code: &str,
    cx: &App,
) -> gpui::AnyElement {
    let code = checked_code(code);
    let colors = cx.colors();
    gpui::div()
        .rounded(px(14.))
        .border_1()
        .border_color(colors.border)
        .bg(colors.background)
        .shadow(cx.layout().surface_shadow.clone())
        .overflow_hidden()
        .child(
            gpui::div()
                .p(px(28.))
                .flex()
                .flex_col()
                .gap(px(16.))
                .child(content),
        )
        .child(
            gpui::div()
                .id(gpui::ElementId::Name(code.to_owned().into()))
                .w_full()
                .border_t_1()
                .border_color(gpui::rgb(CODE_BORDER))
                .px(px(CODE_PAD_X))
                .py(px(CODE_PAD_Y))
                .max_h(px(220.))
                .focusable()
                .overflow_y_scroll()
                .overflow_x_scroll()
                .whitespace_nowrap()
                .bg(gpui::rgb(CODE_BG))
                .font_family(crate::app::MONO_FONT)
                .text_size(px(CODE_TEXT_SIZE))
                .line_height(px(CODE_LINE_HEIGHT))
                .text_color(gpui::rgb(CODE_TEXT))
                .child(highlighted_code(code)),
        )
        .into_any_element()
}

/// Multi-line code block.
pub fn code_block(code: &str, cx: &App) -> gpui::AnyElement {
    let _ = cx;
    let code = checked_code(code);
    gpui::div()
        .w_full()
        .px(px(CODE_PAD_X))
        .py(px(CODE_PAD_Y))
        .rounded(px(CODE_RADIUS))
        .border_1()
        .border_color(gpui::rgb(CODE_BORDER))
        .bg(gpui::rgb(CODE_BG))
        .font_family(crate::app::MONO_FONT)
        .text_size(px(CODE_TEXT_SIZE))
        .line_height(px(CODE_LINE_HEIGHT))
        .text_color(gpui::rgb(CODE_TEXT))
        .child(highlighted_code(code))
        .into_any_element()
}

/// A paragraph of body text.
pub fn para(text: &str, cx: &App) -> gpui::AnyElement {
    let colors = cx.colors();
    gpui::div()
        .text_size(px(14.5))
        .line_height(px(24.))
        .text_color(colors.foreground)
        .max_w(px(720.))
        .child(text.to_owned())
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Every string the gallery renders inside a code block. A hand-written
    /// snippet is registered here to be checked. Component sections are
    /// generated by `component_doc_page!`, so their source is included once
    /// rather than copied into a second 600-snippet registry; construction
    /// applies the same validator to each generated string.
    fn rendered_code_blocks() -> Vec<(String, String)> {
        let mut blocks: Vec<(String, String)> = docs::doc_code_blocks()
            .into_iter()
            .chain(docs_guides::doc_code_blocks())
            .map(|(name, code)| (name.to_owned(), code.to_owned()))
            .collect();
        blocks.push((
            "Toast/setup".to_owned(),
            components::toast_setup_block().to_owned(),
        ));
        blocks.push((
            "component_doc_page sections".to_owned(),
            include_str!("components.rs").to_owned(),
        ));
        blocks.extend(
            nav_sections()
                .into_iter()
                .flat_map(|section| section.items)
                .filter_map(|page| {
                    let line = page.import_line();
                    (!line.is_empty())
                        .then(|| (format!("import/{}", page.title()), line.to_owned()))
                }),
        );
        blocks
    }

    #[test]
    fn code_blocks_teach_only_herogpui_rust() {
        let blocks = rendered_code_blocks();
        assert!(
            blocks.len() >= 60,
            "code-block registry unexpectedly small: {}",
            blocks.len()
        );
        assert!(blocks.iter().any(|(name, _)| name == "Styling/variants"));
        assert!(blocks.iter().any(|(name, _)| name == "import/Button"));

        for (name, code) in &blocks {
            if let Some(marker) = jsx_marker(code) {
                panic!("code block {name} contains JSX marker {marker:?}:\n{code}");
            }
        }
    }

    #[test]
    fn marker_scan_catches_jsx() {
        assert_eq!(
            jsx_marker("<Button variant=\"secondary\" />"),
            Some("<Button")
        );
        assert_eq!(jsx_marker("<Card>\n  <Card.Header />\n</Card>"), Some("</"));
        assert_eq!(jsx_marker("className=\"w-full\""), Some("className="));
        assert_eq!(
            jsx_marker("import { Button } from \"react\""),
            Some("import {")
        );
        assert_eq!(
            jsx_marker("let x = 1;\nButton::new(\"b\").label(\"ok\")"),
            None
        );
    }

    #[test]
    fn marker_scan_catches_generic_component_tags_without_rust_false_positives() {
        assert!(jsx_marker("<Card />").is_some());
        assert!(jsx_marker("<Panel title=\"Details\">").is_some());
        assert!(jsx_marker("<Card.Header />").is_some());
        assert_eq!(jsx_marker("Vec<Card>"), None);
        assert_eq!(jsx_marker("Vec< Card >"), None);
        assert_eq!(jsx_marker("fn render<T>() -> Option<T>"), None);
        assert_eq!(jsx_marker("let result = left < Right;"), None);
        assert_eq!(jsx_marker("let result = left < Right > value;"), None);
        assert_eq!(jsx_marker("gpui::div().child(\"<Card />\")"), None);
        assert_eq!(jsx_marker("// <Card />"), None);
    }

    #[test]
    fn component_sections_are_in_the_integrity_registry() {
        let blocks = rendered_code_blocks();
        assert!(
            blocks
                .iter()
                .any(|(name, _)| name == "component_doc_page sections"),
            "component_doc_page sections are missing from the code registry"
        );
    }
}
