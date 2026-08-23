//! Color math for HeroGPUI.
//!
//! HeroUI v3 defines every theme token in `oklch()` and derives hover / soft /
//! surface-level variants with `color-mix(in oklab, …)`. This module provides
//! the exact equivalents so tokens can be transcribed verbatim from
//! `packages/styles/themes/default/variables.css`.

use gpui::{hsla, Hsla, Rgba};

/// Returns `color` with its alpha channel set to `alpha`.
pub fn with_alpha(color: Hsla, alpha: f32) -> Hsla {
    hsla(color.h, color.s, color.l, alpha)
}

// ---------------------------------------------------------------------------
// OKLCH / OKLab
// ---------------------------------------------------------------------------

/// Builds a color from CSS `oklch(l c h)`.
///
/// `l` is lightness in `0.0..=1.0` (CSS also allows `0%..=100%` — pass the
/// fraction), `c` is chroma and `h` is the hue angle in degrees.
pub fn oklch(l: f32, c: f32, h: f32) -> Hsla {
    oklcha(l, c, h, 1.0)
}

/// [`oklch`] with an explicit alpha.
pub fn oklcha(l: f32, c: f32, h: f32, alpha: f32) -> Hsla {
    let rad = h.to_radians();
    let lab = Oklab {
        l,
        a: c * rad.cos(),
        b: c * rad.sin(),
    };
    let (r, g, b) = lab.to_srgb();
    Hsla::from(Rgba {
        r,
        g,
        b,
        a: alpha,
    })
}

/// CSS `color-mix(in oklab, a (1-t), b t)`.
///
/// `t` is the weight of `b`, so `t = 0.0` yields `a` and `t = 1.0` yields `b`.
/// Alpha is handled the way CSS specifies it: channels are premultiplied before
/// interpolation and un-premultiplied afterwards. This makes mixing against a
/// fully transparent color produce the original hue at reduced alpha, which is
/// how every `*-soft` token in HeroUI v3 is defined.
pub fn mix_oklab(a: Hsla, b: Hsla, t: f32) -> Hsla {
    let t = t.clamp(0.0, 1.0);
    let (ra, ga, ba, aa) = rgba_parts(a);
    let (rb, gb, bb, ab) = rgba_parts(b);

    let lab_a = Oklab::from_srgb(ra, ga, ba);
    let lab_b = Oklab::from_srgb(rb, gb, bb);

    let alpha = aa + (ab - aa) * t;
    if alpha <= f32::EPSILON {
        return hsla(0.0, 0.0, 0.0, 0.0);
    }

    // Premultiplied interpolation, then divide the alpha back out.
    let wa = aa * (1.0 - t);
    let wb = ab * t;
    let lerp = |x: f32, y: f32| (x * wa + y * wb) / alpha;

    let mixed = Oklab {
        l: lerp(lab_a.l, lab_b.l),
        a: lerp(lab_a.a, lab_b.a),
        b: lerp(lab_a.b, lab_b.b),
    };
    let (r, g, bl) = mixed.to_srgb();
    Hsla::from(Rgba {
        r,
        g,
        b: bl,
        a: alpha,
    })
}

/// CSS `color-mix(in oklab, color p%, transparent)` — the `*-soft` family.
pub fn soft_mix(color: Hsla, percent: f32) -> Hsla {
    mix_oklab(color, hsla(0.0, 0.0, 0.0, 0.0), 1.0 - percent)
}

/// Blends two colors in premultiplied sRGB space. Retained for gradients and
/// other places where perceptual mixing is not wanted.
pub fn mix(a: Hsla, b: Hsla, t: f32) -> Hsla {
    let t = t.clamp(0.0, 1.0);
    let (ra, ga, ba, aa) = rgba_parts(a);
    let (rb, gb, bb, ab) = rgba_parts(b);
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    let alpha = lerp(aa, ab);
    if alpha <= f32::EPSILON {
        return hsla(0.0, 0.0, 0.0, 0.0);
    }
    Hsla::from(Rgba {
        r: lerp(ra, rb),
        g: lerp(ga, gb),
        b: lerp(ba, bb),
        a: alpha,
    })
}

/// A color in the Oklab space.
#[derive(Clone, Copy, Debug)]
struct Oklab {
    l: f32,
    a: f32,
    b: f32,
}

impl Oklab {
    fn from_srgb(r: f32, g: f32, b: f32) -> Self {
        let r = srgb_to_linear(r);
        let g = srgb_to_linear(g);
        let b = srgb_to_linear(b);

        let l = 0.412_221_47 * r + 0.536_332_55 * g + 0.051_445_995 * b;
        let m = 0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b;
        let s = 0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b;

        let l_ = l.cbrt();
        let m_ = m.cbrt();
        let s_ = s.cbrt();

        Self {
            l: 0.210_454_26 * l_ + 0.793_617_8 * m_ - 0.004_072_047 * s_,
            a: 1.977_998_5 * l_ - 2.428_592_2 * m_ + 0.450_593_7 * s_,
            b: 0.025_904_037 * l_ + 0.782_771_77 * m_ - 0.808_675_77 * s_,
        }
    }

    fn to_srgb(self) -> (f32, f32, f32) {
        let l_ = self.l + 0.396_337_78 * self.a + 0.215_803_76 * self.b;
        let m_ = self.l - 0.105_561_346 * self.a - 0.063_854_17 * self.b;
        let s_ = self.l - 0.089_484_18 * self.a - 1.291_485_5 * self.b;

        let l = l_ * l_ * l_;
        let m = m_ * m_ * m_;
        let s = s_ * s_ * s_;

        let r = 4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s;
        let g = -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s;
        let b = -0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s;

        (
            linear_to_srgb(r).clamp(0.0, 1.0),
            linear_to_srgb(g).clamp(0.0, 1.0),
            linear_to_srgb(b).clamp(0.0, 1.0),
        )
    }
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

fn rgba_parts(c: Hsla) -> (f32, f32, f32, f32) {
    let rgba = Rgba::from(c);
    (rgba.r, rgba.g, rgba.b, rgba.a)
}

/// Picks black or white for maximal contrast against `background`, matching
/// `readableColor` from color2k as used by HeroUI.
pub fn readable_color(background: Hsla) -> Hsla {
    let (r, g, b, _) = rgba_parts(background);
    let luminance =
        0.2126 * srgb_to_linear(r) + 0.7152 * srgb_to_linear(g) + 0.0722 * srgb_to_linear(b);
    let contrast_black = (luminance + 0.05) / 0.05;
    let contrast_white = 1.05 / (luminance + 0.05);
    if contrast_white > contrast_black {
        gpui::white()
    } else {
        gpui::black()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgb8(c: Hsla) -> (u8, u8, u8) {
        let r = Rgba::from(c);
        (
            (r.r * 255.0).round() as u8,
            (r.g * 255.0).round() as u8,
            (r.b * 255.0).round() as u8,
        )
    }

    #[test]
    fn oklch_white_and_black() {
        assert_eq!(rgb8(oklch(1.0, 0.0, 0.0)), (255, 255, 255));
        assert_eq!(rgb8(oklch(0.0, 0.0, 0.0)), (0, 0, 0));
    }

    /// The canonical Oklab coordinates of the sRGB primaries. If these three
    /// round-trip, the matrices and transfer functions are right.
    ///
    /// The published values carry more digits than `f32` can hold. They are
    /// kept verbatim so the constants are traceable to their source; the
    /// rounding `f32` applies is exactly what the conversion has to tolerate.
    #[allow(clippy::excessive_precision)]
    #[test]
    fn oklch_matches_srgb_primaries() {
        assert_eq!(rgb8(oklch(0.627_955, 0.257_683, 29.233_885)), (255, 0, 0));
        assert_eq!(rgb8(oklch(0.866_440, 0.294_827, 142.495_339)), (0, 255, 0));
        assert_eq!(rgb8(oklch(0.452_014, 0.313_214, 264.052_020)), (0, 0, 255));
    }

    #[test]
    fn srgb_oklab_round_trips() {
        for &(r, g, b) in &[
            (0.0_f32, 0.0_f32, 0.0_f32),
            (1.0, 1.0, 1.0),
            (0.2, 0.5, 0.9),
            (0.97, 0.31, 0.04),
        ] {
            let (r2, g2, b2) = Oklab::from_srgb(r, g, b).to_srgb();
            assert!((r - r2).abs() < 1e-3, "r {r} -> {r2}");
            assert!((g - g2).abs() < 1e-3, "g {g} -> {g2}");
            assert!((b - b2).abs() < 1e-3, "b {b} -> {b2}");
        }
    }

    #[test]
    fn soft_mix_keeps_hue_and_reduces_alpha() {
        let accent = oklch(0.6204, 0.195, 253.83);
        let soft = soft_mix(accent, 0.15);
        assert!((soft.a - 0.15).abs() < 1e-4, "alpha was {}", soft.a);
        assert!((soft.h - accent.h).abs() < 0.02);
    }

    #[test]
    fn mix_endpoints_are_exact() {
        let a = oklch(0.62, 0.19, 253.0);
        let b = oklch(0.73, 0.19, 150.0);
        assert_eq!(rgb8(mix_oklab(a, b, 0.0)), rgb8(a));
        assert_eq!(rgb8(mix_oklab(a, b, 1.0)), rgb8(b));
    }
}
