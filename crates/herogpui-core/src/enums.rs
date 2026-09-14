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

/// Button emphasis variant — `Button`, and the vocabulary `ButtonGroup`
/// inherits to its direct children (the pinned group context takes
/// `ButtonProps["variant"]`, every value on this enum).
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

    /// Every Button variant can be inherited by `ButtonGroup` members.
    pub const GROUP: [Variant; 7] = Self::ALL;

    pub fn label(self) -> &'static str {
        match self {
            Variant::Primary => "Primary",
            Variant::Secondary => "Secondary",
            Variant::Tertiary => "Tertiary",
            Variant::Outline => "Outline",
            Variant::Ghost => "Ghost",
            Variant::Danger => "Danger",
            Variant::DangerSoft => "Danger Soft",
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

    /// The `text-xs`/`text-sm`/`text-base` ladder: sm 12px, md 14px, lg 16px.
    ///
    /// Not every sized component steps its type on every rung — `.button` only
    /// steps at `lg` — so a component reads its own stylesheet rather than
    /// assuming this one.
    pub fn text_size(self) -> gpui::Pixels {
        match self {
            Size::Sm => gpui::px(12.0),
            Size::Md => gpui::px(14.0),
            Size::Lg => gpui::px(16.0),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Size::Sm => "Small",
            Size::Md => "Medium",
            Size::Lg => "Large",
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

    /// A colour swatch's edge: `size-4 / 6 / 8 / 9 / 10` from
    /// `color-swatch.css`, so 16 / 24 / 32 / 36 / 40.
    ///
    /// Named for the component on purpose: v3 declares its sizes per sheet, and
    /// a swatch's `sm` (24px) is not a spinner's (16px, and `Spinner` has its own
    /// `SpinnerSize` for exactly that reason). The shared `px()` this replaces
    /// was 16/20/24/32/40 and matched neither sheet.
    pub fn swatch_px(self) -> gpui::Pixels {
        match self {
            SizeXl::Xs => gpui::px(16.0),
            SizeXl::Sm => gpui::px(24.0),
            SizeXl::Md => gpui::px(32.0),
            SizeXl::Lg => gpui::px(36.0),
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

/// `placement` — where a floating panel sits relative to its trigger.
///
/// The full React Aria union v3 forwards: both physical spellings
/// (`"bottom left"`, `"bottom right"`) and logical aliases (`"start"`,
/// `"end top"`, …) for every side. This port has no RTL mode, so the logical
/// start/end aliases resolve to the same pixels as their left/right
/// spellings; the spellings stay distinct values so `ALL` enumerates the
/// whole 22-value vocabulary a caller can name.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Placement {
    /// `"bottom"` — below the trigger, centred.
    Bottom,
    /// `"bottom start"` — below the trigger, flush with its start edge.
    #[default]
    BottomStart,
    /// `"bottom left"` — below the trigger, flush with its left edge; the
    /// physical spelling of [`Placement::BottomStart`] in this LTR-only port.
    BottomLeft,
    /// `"bottom end"` — below the trigger, flush with its end edge.
    BottomEnd,
    /// `"bottom right"` — below the trigger, flush with its right edge; the
    /// physical spelling of [`Placement::BottomEnd`] here.
    BottomRight,
    /// `"top"` — above the trigger, centred.
    Top,
    /// `"top start"` — above the trigger, flush with its start edge.
    TopStart,
    /// `"top left"` — above the trigger, flush with its left edge; the
    /// physical spelling of [`Placement::TopStart`] here.
    TopLeft,
    /// `"top end"` — above the trigger, flush with its end edge.
    TopEnd,
    /// `"top right"` — above the trigger, flush with its right edge; the
    /// physical spelling of [`Placement::TopEnd`] here.
    TopRight,
    /// `"left"` — beside the trigger's left edge, vertically centred.
    Left,
    /// `"left top"` — beside the left edge, flush with the trigger's top.
    LeftTop,
    /// `"left bottom"` — beside the left edge, flush with the trigger's
    /// bottom.
    LeftBottom,
    /// `"right"` — beside the trigger's right edge, vertically centred.
    Right,
    /// `"right top"` — beside the right edge, flush with the trigger's top.
    RightTop,
    /// `"right bottom"` — beside the right edge, flush with the trigger's
    /// bottom.
    RightBottom,
    /// `"start"` — the logical spelling of [`Placement::Left`] in an
    /// LTR-only port.
    Start,
    /// `"start top"` — the logical spelling of [`Placement::LeftTop`] here.
    StartTop,
    /// `"start bottom"` — the logical spelling of [`Placement::LeftBottom`]
    /// here.
    StartBottom,
    /// `"end"` — the logical spelling of [`Placement::Right`] in an
    /// LTR-only port.
    End,
    /// `"end top"` — the logical spelling of [`Placement::RightTop`] here.
    EndTop,
    /// `"end bottom"` — the logical spelling of [`Placement::RightBottom`]
    /// here.
    EndBottom,
}

/// How a panel lines up along the trigger's cross axis.
///
/// The axis is relative to the side: for the top and bottom placements it is
/// the trigger's horizontal axis, for the side placements its vertical one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlacementAlign {
    Start,
    Center,
    End,
}

impl Placement {
    pub const ALL: [Placement; 22] = [
        Placement::Bottom,
        Placement::BottomStart,
        Placement::BottomLeft,
        Placement::BottomEnd,
        Placement::BottomRight,
        Placement::Top,
        Placement::TopStart,
        Placement::TopLeft,
        Placement::TopEnd,
        Placement::TopRight,
        Placement::Left,
        Placement::LeftTop,
        Placement::LeftBottom,
        Placement::Right,
        Placement::RightTop,
        Placement::RightBottom,
        Placement::Start,
        Placement::StartTop,
        Placement::StartBottom,
        Placement::End,
        Placement::EndTop,
        Placement::EndBottom,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Placement::Bottom => "Bottom",
            Placement::BottomStart => "Bottom start",
            Placement::BottomLeft => "Bottom left",
            Placement::BottomEnd => "Bottom end",
            Placement::BottomRight => "Bottom right",
            Placement::Top => "Top",
            Placement::TopStart => "Top start",
            Placement::TopLeft => "Top left",
            Placement::TopEnd => "Top end",
            Placement::TopRight => "Top right",
            Placement::Left => "Left",
            Placement::LeftTop => "Left top",
            Placement::LeftBottom => "Left bottom",
            Placement::Right => "Right",
            Placement::RightTop => "Right top",
            Placement::RightBottom => "Right bottom",
            Placement::Start => "Start",
            Placement::StartTop => "Start top",
            Placement::StartBottom => "Start bottom",
            Placement::End => "End",
            Placement::EndTop => "End top",
            Placement::EndBottom => "End bottom",
        }
    }

    /// Whether the panel opens upward.
    pub fn is_above(self) -> bool {
        matches!(
            self,
            Placement::Top
                | Placement::TopStart
                | Placement::TopLeft
                | Placement::TopEnd
                | Placement::TopRight
        )
    }

    /// Whether the panel sits beside the trigger rather than above or below.
    pub fn is_side(self) -> bool {
        matches!(
            self,
            Placement::Left
                | Placement::LeftTop
                | Placement::LeftBottom
                | Placement::Right
                | Placement::RightTop
                | Placement::RightBottom
                | Placement::Start
                | Placement::StartTop
                | Placement::StartBottom
                | Placement::End
                | Placement::EndTop
                | Placement::EndBottom
        )
    }

    /// Whether the panel opens on the trigger's start side — the left edge,
    /// because this port has no RTL mode.
    pub fn is_start_side(self) -> bool {
        matches!(
            self,
            Placement::Left
                | Placement::LeftTop
                | Placement::LeftBottom
                | Placement::Start
                | Placement::StartTop
                | Placement::StartBottom
        )
    }

    /// The alignment along the trigger's cross axis: horizontal for the top
    /// and bottom placements, vertical for the side ones.
    pub fn align(self) -> PlacementAlign {
        match self {
            Placement::BottomStart
            | Placement::BottomLeft
            | Placement::TopStart
            | Placement::TopLeft
            | Placement::LeftTop
            | Placement::RightTop
            | Placement::StartTop
            | Placement::EndTop => PlacementAlign::Start,
            Placement::BottomEnd
            | Placement::BottomRight
            | Placement::TopEnd
            | Placement::TopRight
            | Placement::LeftBottom
            | Placement::RightBottom
            | Placement::StartBottom
            | Placement::EndBottom => PlacementAlign::End,
            _ => PlacementAlign::Center,
        }
    }
}
