//! Meter — HeroUI v3 `Meter` (value within a known range).
//!
//! A meter reports a static measurement inside a known range (disk usage,
//! password strength), where a `ProgressBar` reports advancing work. The two
//! share their track geometry, so this delegates the bar rendering to
//! [`ProgressBar`].

use gpui::{App, IntoElement, RenderOnce, SharedString, Window};
use herogpui_core::{Color, Size};

use crate::progress::ProgressBar;

/// HeroUI Meter. Supports `value` in `0..max` with
/// optional label and fill color.
#[derive(IntoElement)]
pub struct Meter {
    value: f32,
    min_value: f32,
    max_value: f32,
    size: Size,
    color: Color,
    label: Option<SharedString>,
    value_label: Option<SharedString>,
    /// `Meter.ValueLabel`'s render props (`percentage`, `valueText`), forwarded
    /// to the bar that draws them.
    value_content: Option<std::sync::Arc<dyn Fn(f32, &str) -> gpui::AnyElement + 'static>>,
    show_value: bool,
    /// `formatOptions` — forwarded to the bar, which writes the label.
    format: Option<herogpui_core::NumberFormat>,
}

impl Meter {
    /// `value` — also accepted positionally by [`Meter::new`].
    pub fn value(mut self, value: f32) -> Self {
        self.value = value;
        self
    }

    /// `Meter.ValueLabel`'s render function — handed `percentage` (0-100) and
    /// `valueText`, like the bar's.
    pub fn value_content(
        mut self,
        render: impl Fn(f32, &str) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.value_content = Some(std::sync::Arc::new(render));
        self
    }

    pub fn new(value: f32) -> Self {
        Self {
            value_content: None,
            value,
            min_value: 0.0,
            max_value: 100.0,
            size: Size::Md,
            color: Color::Accent,
            label: None,
            value_label: None,
            show_value: false,
            format: None,
        }
    }

    pub fn min_value(mut self, v: f32) -> Self {
        self.min_value = v;
        self
    }

    pub fn max_value(mut self, v: f32) -> Self {
        self.max_value = v;
        self
    }

    /// `valueLabel` — replaces the generated percentage.
    pub fn value_label(mut self, text: impl Into<SharedString>) -> Self {
        self.value_label = Some(text.into());
        self
    }

    /// `formatOptions` — v3 defaults to `{style: "percent"}`.
    pub fn format_options(mut self, format: herogpui_core::NumberFormat) -> Self {
        self.format = Some(format);
        self
    }

    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }

    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = Some(l.into());
        self
    }

    /// Show `value/max` next to the label (like `Meter.Output`).
    pub fn show_value(mut self, v: bool) -> Self {
        self.show_value = v;
        self
    }
}

impl RenderOnce for Meter {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let mut p = ProgressBar::new()
            .value(self.value)
            .min_value(self.min_value)
            .max_value(self.max_value)
            .size(self.size)
            .color(self.color)
            .show_value_label(self.show_value);
        if let Some(format) = self.format.clone() {
            p = p.format_options(format);
        }
        if let Some(render) = self.value_content {
            p = p.value_content(move |percentage, text| render(percentage, text));
        }
        if let Some(vl) = self.value_label {
            p = p.value_label(vl);
        }
        if let Some(l) = self.label {
            p = p.label(l.to_string());
        }
        p.into_any_element()
    }
}
