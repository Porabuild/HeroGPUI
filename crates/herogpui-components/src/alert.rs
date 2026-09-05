//! Alert — port of `@heroui/alert`.

use gpui::{
    px, AnyElement, App, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window,
};
use herogpui_core::Color;
use herogpui_theme::ActiveTheme;

use crate::icons;

/// HeroUI Alert.
///
/// v3.2.4's API table carries only `status`/`className`/`children`: the
/// migration guide explicitly removes `isClosable`, `onClose` and
/// `closeButtonProps`, so a close affordance is composed by the caller as an
/// ordinary child (a `CloseButton`) instead of being built in.
#[derive(IntoElement)]
pub struct Alert {
    title: SharedString,
    description: Option<SharedString>,
    color: Color,
    /// Composed children — v3's "Additional content like buttons, close
    /// button, etc.", appended after the content column.
    children: Vec<AnyElement>,
}

impl Alert {
    /// `status` — the v3 name for `color`; the values are the same
    /// semantic roles.
    pub fn status(mut self, status: Color) -> Self {
        self.color = status;
        self
    }

    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            description: None,
            color: Color::Default,
            children: Vec::new(),
        }
    }

    pub fn description(mut self, d: impl Into<SharedString>) -> Self {
        self.description = Some(d.into());
        self
    }
}

impl ParentElement for Alert {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        // End content; see the struct doc for the v3 composition rationale.
        self.children.extend(elements);
    }
}

impl RenderOnce for Alert {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sem = cx.role(self.color);
        let colors = cx.colors();
        let layout = cx.layout();

        // v3 dropped Alert's `variant`: the color role is the only axis.
        // `.alert` is `bg-surface` for every status -- the role never paints
        // the container.
        let bg = colors.surface.background;
        let fg = colors.foreground;
        // `.alert--default` paints the title *and* the indicator
        // `text-foreground`; every status paints `text-{role}-soft-foreground`.
        let role_fg = if self.color == Color::Default {
            colors.foreground
        } else {
            sem.soft_foreground(colors.foreground)
        };

        // `alert.tsx`'s `getDefaultIcon`: accent and the `default` fall-through
        // both draw the Info glyph; success the circled check (the pinned
        // `SuccessIcon`), warning the triangle, danger the circle-exclamation.
        let glyph = match self.color {
            Color::Default | Color::Accent => icons::INFO_CIRCLE,
            Color::Success => icons::CHECK_CIRCLE,
            Color::Warning => icons::WARNING_TRIANGLE,
            Color::Danger => icons::CIRCLE_EXCLAMATION,
        };

        let mut alert = gpui::div()
            .flex()
            .items_start()
            .justify_start()
            .gap(px(16.))
            .w_full()
            .px(px(16.))
            .py(px(12.))
            .rounded(crate::util::control_radius(cx))
            .bg(bg)
            .text_color(fg)
            .debug_selector(|| "alert-root".to_owned());
        // `shadow-surface` is the surface elevation token; dark mode leaves
        // the token empty, and GPUI 0.2.2 paints an empty shadow list as
        // nothing, so the token is applied unconditionally.
        alert = alert.shadow(layout.surface_shadow.clone());

        let indicator_glyph = gpui::svg()
            .size(px(16.))
            .path(glyph)
            .text_color(role_fg)
            .flex_shrink_0();
        alert = alert.child(
            // `.alert__indicator` is a `p-1` box around the 16px glyph, centered.
            gpui::div()
                .p(px(4.))
                .flex()
                .items_center()
                .justify_center()
                .flex_shrink_0()
                .debug_selector(|| "alert-indicator".to_owned())
                .child(indicator_glyph),
        );

        // `.alert__content` is the column that holds the title and the
        // description, beside the indicator. It carries no gap: the pinned
        // rule is only `flex h-full grow flex-col items-start`.
        // `min_w_0` is what lets the description wrap. A flex item's automatic
        // minimum size is its content's, and gpui measures a text child's
        // minimum as the whole string, so `grow` alone pushed the column --
        // and the copy -- straight out through the alert's right edge.
        let mut text_col = gpui::div()
            .flex()
            .flex_col()
            .items_start()
            .flex_1()
            .min_w_0();
        text_col = text_col.child(
            gpui::div()
                .text_size(px(14.)) // `.alert__title` is `text-sm leading-6 font-medium`.
                .line_height(px(24.))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(role_fg)
                .debug_selector(|| "alert-title".to_owned())
                .child(self.title.to_string()),
        );
        text_col = text_col.debug_selector(|| "alert-content".to_owned());
        if let Some(desc) = self.description {
            text_col = text_col.child(
                gpui::div()
                    // `.alert__description` is `text-sm`.
                    .text_size(px(14.))
                    // `text-sm`'s own line height is 20px.
                    .line_height(px(20.))
                    .text_color(colors.muted)
                    .debug_selector(|| "alert-description".to_owned())
                    .child(desc.to_string()),
            );
        }
        alert = alert.child(text_col);

        // Composed children go last; see the struct doc.
        alert = alert.children(self.children);

        alert
    }
}

// The pinned `.alert` is `bg-surface` for every status: the role paints the
// indicator and the title only, never the container. A soft wash looks
// plausible on screen, so the check is mechanical.
#[cfg(test)]
mod painted_tokens {
    fn implementation() -> &'static str {
        include_str!("alert.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present")
    }

    #[test]
    fn the_alert_container_is_always_surface() {
        let source = implementation();
        assert!(
            source.contains("let bg = colors.surface.background;"),
            "every status must paint `bg-surface` (pinned `.alert`)"
        );
        assert!(
            !source.contains("sem.soft()"),
            "no alert container may paint a role soft background"
        );
    }

    #[test]
    fn the_indicator_follows_the_status_soft_foreground() {
        let source = implementation();
        // The glyph's own `let` declaration is the structural unit: everything
        // between `let indicator_glyph = gpui::svg()` and the statement's
        // terminating `;` is the full builder chain that paints the glyph.
        let chain = source
            .split("let indicator_glyph = gpui::svg()")
            .nth(1)
            .expect("the indicator must paint the glyph as a declared svg element")
            .split(';')
            .next()
            .expect("the declaration must terminate");
        assert!(
            chain.contains(".text_color(role_fg)"),
            "the indicator must paint the same token as the title: \
             `text-foreground` on default, `text-{{role}}-soft-foreground` \
             otherwise (pinned `.alert__indicator`)"
        );
        assert!(
            !chain.contains("colors.muted"),
            "the default indicator is `text-foreground`, not the muted tone"
        );
    }
}
