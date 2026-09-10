//! Sparse theme documents, applied through [`ThemeBuilder`].
//!
//! A document records only the tokens a caller overrode, the same way v3
//! authors a `[data-theme]` block: the rest derive from the named `base`
//! (`light` or `dark`). Serializing a complete [`Theme`] would freeze every
//! derived hover / soft mix; going through the builder keeps those mixes live.
//!
//! The JSON keys are the [`ThemeBuilder`] methods. `.shots/theme_serde_audit.py`
//! fails if a builder method is added or renamed without a matching key here.

use std::fmt;

use gpui::{px, Hsla, Rgba};
use herogpui_core::oklcha;
use serde::{Deserialize, Serialize};

use crate::{Appearance, Theme, ThemeBuilder};

/// A sparse override document for a [`Theme`].
///
/// `id` and `base` are required. Every other field is optional and maps onto
/// one [`ThemeBuilder`] method of the same name (`role` is the map `roles`).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ThemeDocument {
    pub id: String,
    /// Which built-in theme the overrides extend: `"light"` or `"dark"`.
    pub base: Appearance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub appearance: Option<Appearance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radius: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_radius: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_width: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled_opacity: Option<f32>,
    /// The hover cursor for interactive controls, by gpui's `CursorStyle`
    /// variant name (`"PointingHand"` is v3's `cursor: pointer`, `"Arrow"`
    /// the platform default).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor_interactive: Option<gpui::CursorStyle>,
    /// The opacity a hovered `Tabs` item drops to; clamped to `0..=1`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tabs_hover_opacity: Option<f32>,
    /// The warm window after the pointer leaves a tooltip during which the
    /// next tip opens without its delay.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tooltip_cooldown_ms: Option<u64>,
    /// How long a `DropdownTrigger::LongPress` waits before it opens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub long_press_ms: Option<u64>,
    /// The background fade duration of `anim::hover_fade`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hover_fade_ms: Option<u64>,
    /// `--tooltip-delay`: how long a hover waits before the tip opens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tooltip_delay_ms: Option<u64>,
    /// `--tooltip-close-delay`: the per-tooltip close delay default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tooltip_close_delay_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub foreground: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub muted: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub separator: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backdrop: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface: Option<ColorPair>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface_levels: Option<SurfaceLevels>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overlay: Option<ColorPair>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub segment: Option<ColorPair>,
    /// Shorthand for [`ThemeBuilder::accent`]: sets `--accent` and derives
    /// the foreground. Conflicts with `roles.accent`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roles: Option<Roles>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<ColorPair>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_placeholder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_border: Option<String>,
}

/// A background / foreground pair, matching the two-argument builder methods.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ColorPair {
    pub background: String,
    pub foreground: String,
}

/// `--surface-secondary` and `--surface-tertiary`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SurfaceLevels {
    pub secondary: String,
    pub tertiary: String,
}

/// The five role slots [`ThemeBuilder::role`] accepts.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Roles {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<RoleOverride>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accent: Option<RoleOverride>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub success: Option<RoleOverride>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warning: Option<RoleOverride>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub danger: Option<RoleOverride>,
}

/// One role's base colour and its on-colour foreground.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RoleOverride {
    pub color: String,
    pub foreground: String,
}

/// Why a document could not become a [`Theme`].
#[derive(Debug)]
pub enum ThemeDocumentError {
    Json(serde_json::Error),
    Color {
        field: String,
        value: String,
        detail: String,
    },
    AccentConflict,
}

impl fmt::Display for ThemeDocumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(err) => write!(f, "theme document: {err}"),
            Self::Color {
                field,
                value,
                detail,
            } => {
                write!(
                    f,
                    "theme document: {field} value {value:?} is not a colour ({detail})"
                )
            }
            Self::AccentConflict => write!(
                f,
                "theme document: `accent` and `roles.accent` cannot both be set"
            ),
        }
    }
}

impl std::error::Error for ThemeDocumentError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json(err) => Some(err),
            Self::Color { .. } | Self::AccentConflict => None,
        }
    }
}

impl From<serde_json::Error> for ThemeDocumentError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

impl ThemeDocument {
    /// Parse a JSON document.
    pub fn from_json(json: &str) -> Result<Self, ThemeDocumentError> {
        Ok(serde_json::from_str(json)?)
    }

    /// Parse a JSON document and apply it through [`ThemeBuilder`].
    pub fn theme_from_json(json: &str) -> Result<Theme, ThemeDocumentError> {
        Self::from_json(json)?.to_theme()
    }

    /// Serialize this sparse document. Derived tokens are not written.
    pub fn to_json(&self) -> Result<String, ThemeDocumentError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Apply the overrides through [`ThemeBuilder`].
    pub fn to_theme(&self) -> Result<Theme, ThemeDocumentError> {
        let base = match self.base {
            Appearance::Light => Theme::light(),
            Appearance::Dark => Theme::dark(),
        };
        let mut builder = Theme::builder(self.id.clone(), base);
        if let Some(appearance) = self.appearance {
            builder = builder.appearance(appearance);
        }
        if let Some(radius) = self.radius {
            builder = builder.radius(px(radius));
        }
        if let Some(radius) = self.field_radius {
            builder = builder.field_radius(px(radius));
        }
        if let Some(width) = self.border_width {
            builder = builder.border_width(px(width));
        }
        if let Some(opacity) = self.disabled_opacity {
            builder = builder.disabled_opacity(opacity);
        }
        if let Some(cursor) = self.cursor_interactive {
            builder = builder.cursor_interactive(cursor);
        }
        if let Some(opacity) = self.tabs_hover_opacity {
            builder = builder.tabs_hover_opacity(opacity);
        }
        if let Some(ms) = self.tooltip_cooldown_ms {
            builder = builder.tooltip_cooldown_ms(ms);
        }
        if let Some(ms) = self.long_press_ms {
            builder = builder.long_press_ms(ms);
        }
        if let Some(ms) = self.hover_fade_ms {
            builder = builder.hover_fade_ms(ms);
        }
        if let Some(ms) = self.tooltip_delay_ms {
            builder = builder.tooltip_delay_ms(ms);
        }
        if let Some(ms) = self.tooltip_close_delay_ms {
            builder = builder.tooltip_close_delay_ms(ms);
        }
        builder = apply_color(
            builder,
            "background",
            self.background.as_deref(),
            ThemeBuilder::background,
        )?;
        builder = apply_color(
            builder,
            "foreground",
            self.foreground.as_deref(),
            ThemeBuilder::foreground,
        )?;
        builder = apply_color(builder, "muted", self.muted.as_deref(), ThemeBuilder::muted)?;
        builder = apply_color(
            builder,
            "border",
            self.border.as_deref(),
            ThemeBuilder::border,
        )?;
        builder = apply_color(
            builder,
            "separator",
            self.separator.as_deref(),
            ThemeBuilder::separator,
        )?;
        builder = apply_color(builder, "focus", self.focus.as_deref(), ThemeBuilder::focus)?;
        builder = apply_color(builder, "link", self.link.as_deref(), ThemeBuilder::link)?;
        builder = apply_color(
            builder,
            "backdrop",
            self.backdrop.as_deref(),
            ThemeBuilder::backdrop,
        )?;
        if let Some(pair) = &self.surface {
            builder = builder.surface(
                parse_color("surface.background", &pair.background)?,
                parse_color("surface.foreground", &pair.foreground)?,
            );
        }
        if let Some(levels) = &self.surface_levels {
            builder = builder.surface_levels(
                parse_color("surface_levels.secondary", &levels.secondary)?,
                parse_color("surface_levels.tertiary", &levels.tertiary)?,
            );
        }
        if let Some(pair) = &self.overlay {
            builder = builder.overlay(
                parse_color("overlay.background", &pair.background)?,
                parse_color("overlay.foreground", &pair.foreground)?,
            );
        }
        if let Some(pair) = &self.segment {
            builder = builder.segment(
                parse_color("segment.background", &pair.background)?,
                parse_color("segment.foreground", &pair.foreground)?,
            );
        }
        if self.accent.is_some() && self.roles.as_ref().is_some_and(|r| r.accent.is_some()) {
            return Err(ThemeDocumentError::AccentConflict);
        }
        if let Some(accent) = &self.accent {
            builder = builder.accent(parse_color("accent", accent)?);
        }
        if let Some(roles) = &self.roles {
            builder = apply_role(builder, "default", roles.default.as_ref())?;
            builder = apply_role(builder, "accent", roles.accent.as_ref())?;
            builder = apply_role(builder, "success", roles.success.as_ref())?;
            builder = apply_role(builder, "warning", roles.warning.as_ref())?;
            builder = apply_role(builder, "danger", roles.danger.as_ref())?;
        }
        if let Some(pair) = &self.field {
            builder = builder.field(
                parse_color("field.background", &pair.background)?,
                parse_color("field.foreground", &pair.foreground)?,
            );
        }
        builder = apply_color(
            builder,
            "field_placeholder",
            self.field_placeholder.as_deref(),
            ThemeBuilder::field_placeholder,
        )?;
        builder = apply_color(
            builder,
            "field_border",
            self.field_border.as_deref(),
            ThemeBuilder::field_border,
        )?;
        Ok(builder.build())
    }
}

fn apply_color(
    builder: ThemeBuilder,
    field: &str,
    raw: Option<&str>,
    apply: fn(ThemeBuilder, Hsla) -> ThemeBuilder,
) -> Result<ThemeBuilder, ThemeDocumentError> {
    match raw {
        Some(value) => Ok(apply(builder, parse_color(field, value)?)),
        None => Ok(builder),
    }
}

fn apply_role(
    builder: ThemeBuilder,
    name: &str,
    role: Option<&RoleOverride>,
) -> Result<ThemeBuilder, ThemeDocumentError> {
    match role {
        Some(role) => Ok(builder.role(
            name,
            parse_color(&format!("roles.{name}.color"), &role.color)?,
            parse_color(&format!("roles.{name}.foreground"), &role.foreground)?,
        )),
        None => Ok(builder),
    }
}

/// Colours are CSS `oklch()` / `oklcha()` or `#RGB` / `#RRGGBB` / `#RRGGBBAA`.
fn parse_color(field: &str, raw: &str) -> Result<Hsla, ThemeDocumentError> {
    let value = raw.trim();
    if let Some(hex) = value.strip_prefix('#') {
        return parse_hex(field, value, hex);
    }
    if let Some(inner) = value
        .strip_prefix("oklch(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return parse_oklch(field, value, inner);
    }
    if let Some(inner) = value
        .strip_prefix("oklcha(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return parse_oklch(field, value, inner);
    }
    Err(ThemeDocumentError::Color {
        field: field.to_owned(),
        value: value.to_owned(),
        detail: "expected oklch(...), oklcha(...) or #hex".into(),
    })
}

fn parse_oklch(field: &str, raw: &str, inner: &str) -> Result<Hsla, ThemeDocumentError> {
    let normalized = inner.replace('/', " ");
    let parts: Vec<&str> = normalized.split_whitespace().collect();
    if parts.len() < 3 || parts.len() > 4 {
        return Err(ThemeDocumentError::Color {
            field: field.to_owned(),
            value: raw.to_owned(),
            detail: "oklch takes L C H, optionally / alpha".into(),
        });
    }
    let l = parse_component(field, raw, parts[0], true)?;
    let c = parse_component(field, raw, parts[1], false)?;
    let h = parse_component(field, raw, parts[2], false)?;
    let a = match parts.get(3) {
        Some(part) => parse_component(field, raw, part, false)?,
        None => 1.0,
    };
    Ok(oklcha(l, c, h, a))
}

fn parse_component(
    field: &str,
    raw: &str,
    part: &str,
    lightness: bool,
) -> Result<f32, ThemeDocumentError> {
    let percent = part.ends_with('%');
    let number = part.trim_end_matches('%');
    let value: f32 = number.parse().map_err(|_| ThemeDocumentError::Color {
        field: field.to_owned(),
        value: raw.to_owned(),
        detail: format!("cannot parse {part:?} as a number"),
    })?;
    if percent || (lightness && value > 1.0) {
        Ok(value / 100.0)
    } else {
        Ok(value)
    }
}

fn parse_hex(field: &str, raw: &str, hex: &str) -> Result<Hsla, ThemeDocumentError> {
    let hex = hex.trim();
    let fail = |detail: &str| ThemeDocumentError::Color {
        field: field.to_owned(),
        value: raw.to_owned(),
        detail: detail.into(),
    };
    let nibble = |ch: u8| match ch {
        b'0'..=b'9' => Ok(ch - b'0'),
        b'a'..=b'f' => Ok(ch - b'a' + 10),
        b'A'..=b'F' => Ok(ch - b'A' + 10),
        _ => Err(fail("hex digit is not 0-9A-F")),
    };
    let byte =
        |hi: u8, lo: u8| -> Result<u8, ThemeDocumentError> { Ok((nibble(hi)? << 4) | nibble(lo)?) };
    let bytes = hex.as_bytes();
    let (r, g, b, a) = match bytes {
        [r, g, b] => (nibble(*r)? * 17, nibble(*g)? * 17, nibble(*b)? * 17, 255),
        [r, g, b, a] => (
            nibble(*r)? * 17,
            nibble(*g)? * 17,
            nibble(*b)? * 17,
            nibble(*a)? * 17,
        ),
        [r1, r2, g1, g2, b1, b2] => (byte(*r1, *r2)?, byte(*g1, *g2)?, byte(*b1, *b2)?, 255),
        [r1, r2, g1, g2, b1, b2, a1, a2] => (
            byte(*r1, *r2)?,
            byte(*g1, *g2)?,
            byte(*b1, *b2)?,
            byte(*a1, *a2)?,
        ),
        _ => return Err(fail("hex is #RGB, #RGBA, #RRGGBB or #RRGGBBAA")),
    };
    Ok(Hsla::from(Rgba {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: a as f32 / 255.0,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use herogpui_core::{oklch, with_alpha};

    #[test]
    fn an_empty_document_is_the_named_base_with_a_new_id() {
        let theme =
            ThemeDocument::theme_from_json(r#"{ "id": "brand", "base": "light" }"#).unwrap();
        let base = Theme::light();
        assert_eq!(theme.id.as_ref(), "brand");
        assert_eq!(theme.appearance, Appearance::Light);
        assert_eq!(theme.colors.background, base.colors.background);
        assert_eq!(theme.colors.accent.color, base.colors.accent.color);
        assert_eq!(theme.layout.radius, base.layout.radius);
    }

    #[test]
    fn overrides_go_through_the_builder_so_derived_mixes_stay_live() {
        let accent = oklch(0.55, 0.23, 295.0);
        let via_builder = Theme::builder("violet", Theme::light())
            .accent(accent)
            .foreground(oklch(0.30, 0.05, 120.0))
            .build();
        let via_json = ThemeDocument::theme_from_json(
            r#"{
                "id": "violet",
                "base": "light",
                "accent": "oklch(0.55 0.23 295)",
                "foreground": "oklch(0.30 0.05 120)"
            }"#,
        )
        .unwrap();
        assert_eq!(
            via_json.colors.accent.color,
            via_builder.colors.accent.color
        );
        assert_eq!(
            via_json.colors.accent.foreground,
            via_builder.colors.accent.foreground
        );
        assert_eq!(via_json.colors.scrollbar, via_builder.colors.scrollbar);
        assert_eq!(
            via_json.colors.scrollbar,
            with_alpha(oklch(0.30, 0.05, 120.0), 0.15)
        );
        assert!((via_json.colors.accent.soft().a - 0.15).abs() < 1e-4);
    }

    #[test]
    fn unknown_keys_are_rejected() {
        let err =
            ThemeDocument::from_json(r##"{ "id": "x", "base": "light", "primary": "#f00" }"##)
                .unwrap_err();
        let message = err.to_string();
        assert!(
            message.contains("primary") || message.contains("unknown"),
            "{message}"
        );
    }

    #[test]
    fn accent_and_roles_accent_cannot_both_be_set() {
        let err = ThemeDocument::theme_from_json(
            r##"{
                "id": "x",
                "base": "light",
                "accent": "#006FEE",
                "roles": { "accent": { "color": "#006FEE", "foreground": "#fff" } }
            }"##,
        )
        .unwrap_err();
        assert!(matches!(err, ThemeDocumentError::AccentConflict));
    }

    #[test]
    fn hex_and_percent_lightness_parse() {
        let theme = ThemeDocument::theme_from_json(
            r##"{
                "id": "x",
                "base": "dark",
                "background": "#111",
                "link": "oklch(55% 0.2 250 / 0.9)"
            }"##,
        )
        .unwrap();
        assert_eq!(theme.id.as_ref(), "x");
        assert_eq!(theme.appearance, Appearance::Dark);
        assert!((theme.colors.link.a - 0.9).abs() < 1e-4);
    }

    #[test]
    fn customisation_tokens_apply_from_json_through_the_builder() {
        let json = r#"{
                "id": "x",
                "base": "light",
                "tabs_hover_opacity": 0.2,
                "tooltip_cooldown_ms": 250,
                "long_press_ms": 350,
                "hover_fade_ms": 0,
                "tooltip_delay_ms": 50,
                "tooltip_close_delay_ms": 75
            }"#;
        let theme = ThemeDocument::theme_from_json(json).unwrap();
        assert!((theme.layout.tabs_hover_opacity - 0.2).abs() < 1e-6);
        assert_eq!(theme.layout.tooltip_cooldown_ms, 250);
        assert_eq!(theme.layout.long_press_ms, 350);
        assert_eq!(theme.layout.hover_fade_ms, 0);
        assert_eq!(theme.layout.tooltip_delay_ms, 50);
        assert_eq!(theme.layout.tooltip_close_delay_ms, 75);

        // Round-trip the sparse document: serializing must not drop a token
        // and re-parsing must apply the same values.
        let round_tripped = ThemeDocument::from_json(json).unwrap().to_json().unwrap();
        let again = ThemeDocument::theme_from_json(&round_tripped).unwrap();
        assert!((again.layout.tabs_hover_opacity - 0.2).abs() < 1e-6);
        assert_eq!(again.layout.tooltip_cooldown_ms, 250);
        assert_eq!(again.layout.long_press_ms, 350);
        assert_eq!(again.layout.hover_fade_ms, 0);
        assert_eq!(again.layout.tooltip_delay_ms, 50);
        assert_eq!(again.layout.tooltip_close_delay_ms, 75);

        let clamped = ThemeDocument::theme_from_json(
            r#"{ "id": "x", "base": "light", "tabs_hover_opacity": 3.0 }"#,
        )
        .unwrap();
        assert!((clamped.layout.tabs_hover_opacity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn a_document_round_trips_without_growing_derived_keys() {
        let original =
            ThemeDocument::from_json(r#"{ "id": "brand", "base": "light", "radius": 8 }"#).unwrap();
        let json = original.to_json().unwrap();
        assert!(!json.contains("scrollbar"));
        assert!(!json.contains("soft"));
        let again = ThemeDocument::from_json(&json).unwrap();
        assert_eq!(again.id, "brand");
        assert_eq!(again.radius, Some(8.0));
    }
}
