//! Shared enums for HeroGPUI — the v3 prop vocabularies.
//!
//! v3 does not use one variant enum everywhere. It uses a small number of
//! distinct vocabularies, each modelled separately here so an invalid
//! combination cannot be expressed:
//!
//! * [`Variant`] — button emphasis (`primary | secondary | … | danger`)
//! * [`FieldVariant`] — form-control emphasis (`primary | secondary`)
//! * [`Prominence`] — container prominence (`transparent | default | …`)
//! * [`Backdrop`] — overlay scrim style (`opaque | blur | transparent`)
//! * [`Color`] — semantic color role (`default | accent | … | danger`)

/// Semantic color roles — HeroUI v3.
///
/// `Accent` is the brand color (v2 `primary`). `Secondary` as a *color* was
/// removed in v3; the `secondary` *variant* uses `default`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Color {
    #[default]
    Default,
    Accent,
    Success,
    Warning,
    Danger,
}

impl Color {
    pub const ALL: [Color; 5] = [
        Color::Default,
        Color::Accent,
        Color::Success,
        Color::Warning,
        Color::Danger,
    ];

    /// The v3 token name of this role.
    pub fn token(self) -> &'static str {
        match self {
            Color::Default => "default",
            Color::Accent => "accent",
            Color::Success => "success",
            Color::Warning => "warning",
            Color::Danger => "danger",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Color::Default => "Default",
            Color::Accent => "Accent",
            Color::Success => "Success",
            Color::Warning => "Warning",
            Color::Danger => "Danger",
        }
    }
}

/// Button emphasis variant — `Button`, and (minus [`Variant::Outline`])
/// `ButtonGroup`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Variant {
    /// Filled with `accent`.
    #[default]
    Primary,
    /// Filled with `default`, accent-tinted label.
    Secondary,
    /// Transparent until hovered.
    Tertiary,
    /// Bordered, transparent fill.
    Outline,
    /// No border, no fill; soft hover only.
    Ghost,
    /// Filled with `danger`.
    Danger,
    /// `danger` at 15% over the surface, with danger-colored text.
    DangerSoft,
}

impl Variant {
    pub const ALL: [Variant; 7] = [
        Variant::Primary,
        Variant::Secondary,
        Variant::Tertiary,
        Variant::Outline,
        Variant::Ghost,
        Variant::Danger,
        Variant::DangerSoft,
    ];

    /// The five variants `ButtonGroup` propagates — `outline` is not one.
    pub const GROUP: [Variant; 5] = [
        Variant::Primary,
        Variant::Secondary,
        Variant::Tertiary,
        Variant::Ghost,
        Variant::Danger,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Variant::Primary => "Primary",
            Variant::Secondary => "Secondary",
            Variant::Tertiary => "Tertiary",
            Variant::Outline => "Outline",
            Variant::Ghost => "Ghost",
            Variant::Danger => "Danger",
            Variant::DangerSoft => "Danger soft",
        }
    }
}

/// Form-control emphasis — `primary` carries the field shadow, `secondary` is
/// the flat low-emphasis style for use inside a `Surface`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FieldVariant {
    #[default]
    Primary,
    Secondary,
}

impl FieldVariant {
    pub const ALL: [FieldVariant; 2] = [FieldVariant::Primary, FieldVariant::Secondary];

    pub fn label(self) -> &'static str {
        match self {
            FieldVariant::Primary => "Primary",
            FieldVariant::Secondary => "Secondary",
        }
    }
}

/// Container prominence — `Surface` and `Card`. [`Separator`] uses the same
/// ladder minus `transparent`.
///
/// [`Separator`]: ../herogpui_components/struct.Separator.html
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Prominence {
    /// No background — for overlays and custom-painted containers.
    Transparent,
    /// `bg-surface`
    #[default]
    Default,
    /// `bg-surface-secondary`
    Secondary,
    /// `bg-surface-tertiary`
    Tertiary,
}

impl Prominence {
    pub const ALL: [Prominence; 4] = [
        Prominence::Transparent,
        Prominence::Default,
        Prominence::Secondary,
        Prominence::Tertiary,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Prominence::Transparent => "Transparent",
            Prominence::Default => "Default",
            Prominence::Secondary => "Secondary",
            Prominence::Tertiary => "Tertiary",
        }
    }
}

/// Overlay scrim style — `Modal`, `Drawer` and `AlertDialog`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Backdrop {
    #[default]
    Opaque,
    Blur,
    Transparent,
}

impl Backdrop {
    pub const ALL: [Backdrop; 3] = [Backdrop::Opaque, Backdrop::Blur, Backdrop::Transparent];

    pub fn label(self) -> &'static str {
        match self {
            Backdrop::Opaque => "Opaque",
            Backdrop::Blur => "Blur",
            Backdrop::Transparent => "Transparent",
        }
    }
}

/// The `sm | md | lg` scale used by most components.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Size {
    Sm,
    #[default]
    Md,
    Lg,
}

impl Size {
    pub const ALL: [Size; 3] = [Size::Sm, Size::Md, Size::Lg];

    /// Control height: sm 32px, md 36px, lg 40px.
    ///
    /// These are v3's *desktop* heights. Its sheet is mobile-first — `.button`
    /// is `h-10 md:h-9`, `.button--sm` is `h-9 md:h-8`, `.button--lg` is
    /// `h-11 md:h-10` — and a desktop app is past every breakpoint, so the `md`
    /// value is the one to match. Reading the base value made every control a
    /// step too tall.
    pub fn control_height(self) -> gpui::Pixels {
        match self {
            Size::Sm => gpui::px(32.0),
            Size::Md => gpui::px(36.0),
            Size::Lg => gpui::px(40.0),
        }
    }

    /// Icon-only controls are square at the control height.
    pub fn icon_control_size(self) -> gpui::Pixels {
        self.control_height()
    }

    /// Label size: sm 12px, md 14px, lg 16px.
    pub fn text_size(self) -> gpui::Pixels {
        match self {
            Size::Sm => gpui::px(12.0),
            Size::Md => gpui::px(14.0),
            Size::Lg => gpui::px(16.0),
        }
    }

    /// Line height matching [`text_size`](Self::text_size).
    pub fn line_height(self) -> gpui::Pixels {
        match self {
            Size::Sm => gpui::px(16.0),
            Size::Md => gpui::px(20.0),
            Size::Lg => gpui::px(24.0),
        }
    }

    /// Gap between a control's icon and its label.
    pub fn gap(self) -> gpui::Pixels {
        match self {
            Size::Sm | Size::Md => gpui::px(8.0),
            Size::Lg => gpui::px(12.0),
        }
    }

    /// Horizontal padding of a labelled control.
    pub fn padding_x(self) -> gpui::Pixels {
        match self {
            Size::Sm => gpui::px(12.0),
            Size::Md => gpui::px(16.0),
            Size::Lg => gpui::px(24.0),
        }
    }

    /// Glyph size for icons inside a control of this size.
    pub fn icon_size(self) -> gpui::Pixels {
        match self {
            Size::Sm => gpui::px(14.0),
            Size::Md => gpui::px(16.0),
            Size::Lg => gpui::px(20.0),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Size::Sm => "Sm",
            Size::Md => "Md",
            Size::Lg => "Lg",
        }
    }
}

/// The `xs | sm | md | lg | xl` scale used by `Spinner`, `ColorSwatch` and
/// `ColorSwatchPicker`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SizeXl {
    Xs,
    Sm,
    #[default]
    Md,
    Lg,
    Xl,
}

impl SizeXl {
    pub const ALL: [SizeXl; 5] = [SizeXl::Xs, SizeXl::Sm, SizeXl::Md, SizeXl::Lg, SizeXl::Xl];

    /// Square edge length: 16 / 20 / 24 / 32 / 40 px.
    pub fn px(self) -> gpui::Pixels {
        match self {
            SizeXl::Xs => gpui::px(16.0),
            SizeXl::Sm => gpui::px(20.0),
            SizeXl::Md => gpui::px(24.0),
            SizeXl::Lg => gpui::px(32.0),
            SizeXl::Xl => gpui::px(40.0),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SizeXl::Xs => "Xs",
            SizeXl::Sm => "Sm",
            SizeXl::Md => "Md",
            SizeXl::Lg => "Lg",
            SizeXl::Xl => "Xl",
        }
    }
}

/// Orientation for separators, toolbars, sliders and groups.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Orientation {
    #[default]
    Horizontal,
    Vertical,
}

impl Orientation {
    pub const ALL: [Orientation; 2] = [Orientation::Horizontal, Orientation::Vertical];

    pub fn is_horizontal(self) -> bool {
        matches!(self, Orientation::Horizontal)
    }

    pub fn label(self) -> &'static str {
        match self {
            Orientation::Horizontal => "Horizontal",
            Orientation::Vertical => "Vertical",
        }
    }
}

/// How many items a collection lets the user select.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SelectionMode {
    None,
    #[default]
    Single,
    Multiple,
}

/// A unique element id helper — most components require one for interactivity.
pub fn auto_id(seed: &str) -> gpui::ElementId {
    gpui::ElementId::Name(seed.to_owned().into())
}

/// `placement` — where a floating panel sits relative to its trigger.
///
/// v3 spells both physical (`"bottom left"`) and logical (`"bottom start"`)
/// forms. This port has no RTL mode, so start coincides with left and end with
/// right, and the two spellings collapse into one value each.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Placement {
    /// `"bottom"` — below the trigger, centred.
    Bottom,
    /// `"bottom start"` / `"bottom left"`.
    #[default]
    BottomStart,
    /// `"bottom end"` / `"bottom right"`.
    BottomEnd,
    /// `"top"` — above the trigger, centred.
    Top,
    /// `"top start"` / `"top left"`.
    TopStart,
    /// `"top end"` / `"top right"`.
    TopEnd,
    Left,
    Right,
}

/// How a panel lines up along the trigger's cross axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlacementAlign {
    Start,
    Center,
    End,
}

impl Placement {
    pub const ALL: [Placement; 8] = [
        Placement::Bottom,
        Placement::BottomStart,
        Placement::BottomEnd,
        Placement::Top,
        Placement::TopStart,
        Placement::TopEnd,
        Placement::Left,
        Placement::Right,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Placement::Bottom => "Bottom",
            Placement::BottomStart => "Bottom start",
            Placement::BottomEnd => "Bottom end",
            Placement::Top => "Top",
            Placement::TopStart => "Top start",
            Placement::TopEnd => "Top end",
            Placement::Left => "Left",
            Placement::Right => "Right",
        }
    }

    /// Whether the panel opens upward.
    pub fn is_above(self) -> bool {
        matches!(
            self,
            Placement::Top | Placement::TopStart | Placement::TopEnd
        )
    }

    /// Whether the panel sits beside the trigger rather than above or below.
    pub fn is_side(self) -> bool {
        matches!(self, Placement::Left | Placement::Right)
    }

    pub fn align(self) -> PlacementAlign {
        match self {
            Placement::BottomStart | Placement::TopStart => PlacementAlign::Start,
            Placement::BottomEnd | Placement::TopEnd => PlacementAlign::End,
            _ => PlacementAlign::Center,
        }
    }
}
