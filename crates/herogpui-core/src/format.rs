//! Number formatting for the components that take v3's `formatOptions`.
//!
//! v3 hands `Intl.NumberFormatOptions` to `Meter`, `NumberField`, `ProgressBar`,
//! `ProgressCircle` and `Slider`, which is how a slider reads out `$1,200.00`
//! rather than `1200`. There is no `Intl` here, so this implements the subset
//! those components actually use, with the `en-US` conventions the default theme
//! already assumes: `,` between groups and `.` before the fraction.
//!
//! What is deliberately absent is `locale`. Choosing separators, digit systems
//! and currency placement per locale needs CLDR data, and inventing a partial
//! table would be worse than not offering the prop.

/// `Intl.NumberFormatOptions["style"]`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NumberStyle {
    #[default]
    Decimal,
    /// Multiplies by 100 and appends `%`.
    Percent,
    Currency,
    Unit,
}

/// `Intl.NumberFormatOptions["currencySign"]`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CurrencySign {
    #[default]
    Standard,
    /// Wraps a negative amount in parentheses instead of prefixing a minus.
    Accounting,
}

/// `Intl.NumberFormatOptions["unitDisplay"]`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UnitDisplay {
    Narrow,
    #[default]
    Short,
    Long,
}

/// The `formatOptions` subset these components use.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NumberFormat {
    pub style: NumberStyle,
    /// ISO 4217 code, e.g. `"USD"`. Rendered as a symbol when one is known.
    pub currency: Option<&'static str>,
    pub currency_sign: CurrencySign,
    /// A CLDR unit identifier, e.g. `"kilogram"`.
    pub unit: Option<&'static str>,
    pub unit_display: UnitDisplay,
    pub minimum_fraction_digits: Option<u8>,
    pub maximum_fraction_digits: Option<u8>,
    pub use_grouping: bool,
}

impl Default for NumberFormat {
    fn default() -> Self {
        Self {
            style: NumberStyle::Decimal,
            currency: None,
            currency_sign: CurrencySign::Standard,
            unit: None,
            unit_display: UnitDisplay::Short,
            minimum_fraction_digits: None,
            maximum_fraction_digits: None,
            use_grouping: true,
        }
    }
}

impl NumberFormat {
    pub fn decimal() -> Self {
        Self::default()
    }

    /// `{style: "percent"}` — v3's default for `ProgressBar` and `Meter`.
    pub fn percent() -> Self {
        Self {
            style: NumberStyle::Percent,
            ..Default::default()
        }
    }

    /// `{style: "currency", currency: code}`.
    pub fn currency(code: &'static str) -> Self {
        Self {
            style: NumberStyle::Currency,
            currency: Some(code),
            ..Default::default()
        }
    }

    /// `{style: "unit", unit: name}`.
    pub fn unit(name: &'static str) -> Self {
        Self {
            style: NumberStyle::Unit,
            unit: Some(name),
            ..Default::default()
        }
    }

    pub fn currency_sign(mut self, sign: CurrencySign) -> Self {
        self.currency_sign = sign;
        self
    }

    pub fn unit_display(mut self, display: UnitDisplay) -> Self {
        self.unit_display = display;
        self
    }

    pub fn minimum_fraction_digits(mut self, n: u8) -> Self {
        self.minimum_fraction_digits = Some(n);
        self
    }

    pub fn maximum_fraction_digits(mut self, n: u8) -> Self {
        self.maximum_fraction_digits = Some(n);
        self
    }

    pub fn use_grouping(mut self, v: bool) -> Self {
        self.use_grouping = v;
        self
    }

    /// The fraction-digit range `Intl` would use: currency defaults to 2, every
    /// other style to 0–3.
    fn fraction_range(&self) -> (u8, u8) {
        let (default_min, default_max) = match self.style {
            NumberStyle::Currency => (2, 2),
            _ => (0, 3),
        };
        let min = self.minimum_fraction_digits.unwrap_or(default_min);
        // An explicit minimum raises the maximum with it, as `Intl` does.
        let max = self
            .maximum_fraction_digits
            .unwrap_or(default_max.max(min));
        (min, max.max(min))
    }

    /// Formats `value`, applying the style's own scaling (`percent` × 100).
    pub fn format(&self, value: f64) -> String {
        let scaled = match self.style {
            NumberStyle::Percent => value * 100.0,
            _ => value,
        };
        let negative = scaled < 0.0 || (scaled == 0.0 && scaled.is_sign_negative());
        let digits = self.digits(scaled.abs());

        let body = match self.style {
            NumberStyle::Percent => format!("{digits}%"),
            NumberStyle::Currency => format!("{}{digits}", currency_symbol(self.currency)),
            NumberStyle::Unit => match (self.unit, self.unit_display) {
                (Some(u), UnitDisplay::Long) => {
                    format!("{digits} {}", long_unit(u, scaled.abs() == 1.0))
                }
                (Some(u), UnitDisplay::Narrow) => format!("{digits}{}", short_unit(u)),
                (Some(u), UnitDisplay::Short) => format!("{digits} {}", short_unit(u)),
                (None, _) => digits,
            },
            NumberStyle::Decimal => digits,
        };

        match (negative, self.style, self.currency_sign) {
            (true, NumberStyle::Currency, CurrencySign::Accounting) => format!("({body})"),
            (true, _, _) => format!("-{body}"),
            (false, _, _) => body,
        }
    }

    /// The digit run: rounded to the fraction range, then grouped.
    fn digits(&self, magnitude: f64) -> String {
        let (min, max) = self.fraction_range();
        let mut text = format!("{:.*}", max as usize, magnitude);
        if max > min {
            // Trim only what the minimum does not require.
            if text.contains('.') {
                let keep = text.len() - text.trim_end_matches('0').len();
                let removable = (max - min) as usize;
                text.truncate(text.len() - keep.min(removable));
                if text.ends_with('.') {
                    text.pop();
                }
            }
        }
        let (int, frac) = match text.split_once('.') {
            Some((i, f)) => (i.to_string(), Some(f.to_string())),
            None => (text, None),
        };
        let int = if self.use_grouping { group(&int) } else { int };
        match frac {
            Some(f) => format!("{int}.{f}"),
            None => int,
        }
    }
}

/// Inserts `,` every three digits from the right.
fn group(digits: &str) -> String {
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// The symbol for the currencies v3's examples use; anything else keeps its
/// code, which is what `Intl` falls back to for an unknown one.
fn currency_symbol(code: Option<&str>) -> String {
    match code {
        Some("USD") => "$".into(),
        Some("EUR") => "€".into(),
        Some("GBP") => "£".into(),
        Some("JPY") => "¥".into(),
        Some(other) => format!("{other} "),
        None => String::new(),
    }
}

fn short_unit(unit: &str) -> &str {
    match unit {
        "kilogram" => "kg",
        "gram" => "g",
        "pound" => "lb",
        "meter" => "m",
        "centimeter" => "cm",
        "kilometer" => "km",
        "mile" => "mi",
        "liter" => "L",
        "byte" => "byte",
        "kilobyte" => "kB",
        "megabyte" => "MB",
        "gigabyte" => "GB",
        "second" => "sec",
        "minute" => "min",
        "hour" => "hr",
        "day" => "day",
        "percent" => "%",
        "celsius" => "°C",
        "fahrenheit" => "°F",
        other => other,
    }
}

fn long_unit(unit: &str, singular: bool) -> String {
    if singular {
        unit.to_string()
    } else {
        // English plurals for the unit names above; none of them is irregular.
        match unit {
            "inch" => "inches".to_string(),
            other => format!("{other}s"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal_groups_thousands() {
        assert_eq!(NumberFormat::decimal().format(1234567.0), "1,234,567");
        assert_eq!(NumberFormat::decimal().format(999.0), "999");
        assert_eq!(
            NumberFormat::decimal().use_grouping(false).format(1234567.0),
            "1234567"
        );
    }

    #[test]
    fn percent_scales_by_a_hundred() {
        assert_eq!(NumberFormat::percent().format(0.42), "42%");
        // v3's ProgressBar hands a 0..1 fraction, so 1.0 must read as 100%.
        assert_eq!(NumberFormat::percent().format(1.0), "100%");
    }

    #[test]
    fn currency_defaults_to_two_fraction_digits() {
        assert_eq!(NumberFormat::currency("USD").format(1200.0), "$1,200.00");
        assert_eq!(NumberFormat::currency("EUR").format(0.5), "€0.50");
        // An unknown code keeps its code, as Intl does.
        assert_eq!(NumberFormat::currency("XYZ").format(3.0), "XYZ 3.00");
    }

    #[test]
    fn accounting_parenthesises_a_negative() {
        let f = NumberFormat::currency("EUR").currency_sign(CurrencySign::Accounting);
        assert_eq!(f.format(-12.0), "(€12.00)");
        assert_eq!(f.format(12.0), "€12.00");
        // `standard` keeps the minus sign in front of the symbol.
        assert_eq!(NumberFormat::currency("EUR").format(-12.0), "-€12.00");
    }

    #[test]
    fn fraction_digits_pad_and_trim() {
        let f = NumberFormat::decimal()
            .minimum_fraction_digits(2)
            .maximum_fraction_digits(2);
        assert_eq!(f.format(3.0), "3.00");
        assert_eq!(f.format(3.456), "3.46");
        // Trailing zeros above the minimum are dropped, as Intl drops them.
        assert_eq!(NumberFormat::decimal().format(2.50), "2.5");
        assert_eq!(NumberFormat::decimal().format(2.0), "2");
    }

    #[test]
    fn units_render_short_narrow_and_long() {
        let kg = NumberFormat::unit("kilogram");
        assert_eq!(kg.format(5.0), "5 kg");
        assert_eq!(kg.clone().unit_display(UnitDisplay::Narrow).format(5.0), "5kg");
        assert_eq!(
            kg.clone().unit_display(UnitDisplay::Long).format(5.0),
            "5 kilograms"
        );
        assert_eq!(
            kg.unit_display(UnitDisplay::Long).format(1.0),
            "1 kilogram"
        );
    }

    #[test]
    fn an_explicit_minimum_raises_the_maximum() {
        // Intl throws when max < min; raising max is the sane resolution.
        let f = NumberFormat::decimal().minimum_fraction_digits(4);
        assert_eq!(f.format(1.5), "1.5000");
    }
}
