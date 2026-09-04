//! InputGroup — port of `@heroui/input-group` (v3).
//!
//! Combines a field with adjacent addons and controls behind one shared piece
//! of field chrome, so a prefix label, the input itself and a trailing button
//! read as a single control.

use gpui::prelude::FluentBuilder;
use gpui::{
    div, px, AnyElement, App, InteractiveElement, IntoElement, MouseButton, ParentElement,
    RenderOnce, SharedString, Styled, Window,
};
use herogpui_core::FieldVariant;
use herogpui_theme::ActiveTheme;

use crate::util;

/// A static, non-interactive segment of an [`InputGroup`] — the `$` before an
/// amount, or a `.com` suffix.
#[derive(IntoElement)]
pub struct InputAddon {
    text: SharedString,
}

impl InputAddon {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }
}

impl RenderOnce for InputAddon {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `.input-group__prefix` / `__suffix`: `px-3`, transparent, and drawn in
        // `--field-placeholder`.
        div()
            .flex()
            .items_center()
            .flex_shrink_0()
            .px(px(12.))
            .text_color(cx.colors().field.placeholder)
            .child(self.text.to_string())
    }
}

/// HeroUI InputGroup.
#[derive(IntoElement)]
pub struct InputGroup {
    variant: FieldVariant,
    full_width: bool,
    is_disabled: bool,
    is_invalid: bool,
    is_required: bool,
    label: Option<SharedString>,
    description: Option<SharedString>,
    error_message: Option<SharedString>,
    /// `InputGroup.Prefix` — the leading addon.
    prefix: Option<AnyElement>,
    /// `InputGroup.Suffix` — the trailing addon.
    suffix: Option<AnyElement>,
    /// `InputGroup.Input` / `InputGroup.TextArea` — held rather than rendered
    /// so the group can strip its chrome and tell it which sides an addon
    /// occupies.
    input: Option<crate::input::Input>,
    /// Whether the held field came from [`InputGroup::text_area`], so the
    /// group can play the pinned `:has([data-slot="input-group-textarea"])`
    /// rules: top alignment, auto height, 8px addon top padding — and the
    /// `querySelector("input")` exception to root-click focusing.
    is_textarea: bool,
    children: Vec<AnyElement>,
}

impl InputGroup {
    pub fn new() -> Self {
        Self {
            variant: FieldVariant::Primary,
            full_width: false,
            is_disabled: false,
            is_invalid: false,
            is_required: false,
            label: None,
            description: None,
            error_message: None,
            prefix: None,
            suffix: None,
            input: None,
            is_textarea: false,
            children: Vec::new(),
        }
    }

    pub fn variant(mut self, variant: FieldVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn full_width(mut self, v: bool) -> Self {
        self.full_width = v;
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    /// `isRequired` — marks the group's label as required. v3's examples get
    /// this from the `TextField` around the group.
    pub fn is_required(mut self, v: bool) -> Self {
        self.is_required = v;
        self
    }

    pub fn is_invalid(mut self, v: bool) -> Self {
        self.is_invalid = v;
        self
    }

    pub fn label(mut self, text: impl Into<SharedString>) -> Self {
        self.label = Some(text.into());
        self
    }

    pub fn description(mut self, text: impl Into<SharedString>) -> Self {
        self.description = Some(text.into());
        self
    }

    pub fn error_message(mut self, text: impl Into<SharedString>) -> Self {
        self.error_message = Some(text.into());
        self
    }

    /// `InputGroup.Prefix` — content before the field.
    pub fn prefix(mut self, el: impl IntoElement) -> Self {
        self.prefix = Some(el.into_any_element());
        self
    }

    /// `InputGroup.Suffix` — content after the field.
    pub fn suffix(mut self, el: impl IntoElement) -> Self {
        self.suffix = Some(el.into_any_element());
        self
    }

    /// `InputGroup.Input` — the field itself.
    ///
    /// Taken as an [`crate::input::Input`] rather than an element so the group
    /// can strip its chrome: v3's group paints the box, and the inner input is
    /// transparent and flush against the addons. Passing one as a plain child
    /// instead leaves a second, smaller field drawn inside the group.
    pub fn input(mut self, input: crate::input::Input) -> Self {
        self.input = Some(input);
        self.is_textarea = false;
        self
    }

    /// `InputGroup.TextArea` — a multi-line field in the same shared chrome.
    pub fn text_area(mut self, text_area: crate::textarea::TextArea) -> Self {
        self.input = Some(text_area.into_group_input());
        self.is_textarea = true;
        self
    }
}

impl Default for InputGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for InputGroup {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for InputGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `.input-group` rings on `focus-within`, and what is inside it is a
        // real `Input` or `TextArea`, so their state is where the focus is.
        // A `TextArea` is converted to an `Input` by `text_area`, so there is one
        // slot to ask.
        let focus_within = self
            .input
            .as_ref()
            .is_some_and(|input| input.state_focus(cx).is_focused(window));
        let colors = cx.colors();
        let layout = cx.layout();
        let is_invalid = self.is_invalid || self.error_message.is_some();
        let (is_disabled, is_textarea) = (self.is_disabled, self.is_textarea);
        // The held field's state entity names this instance's probes; two
        // groups sharing one state would share the probes and the field both.
        let entity = self
            .input
            .as_ref()
            .map(|input| input.state().entity_id().as_u64());

        // `.input-group` is `inline-flex min-h-9 items-center` with no padding
        // of its own: the prefix, the input and the suffix each carry `px-3`,
        // which is what keeps the addons flush with the field's edges. With a
        // textarea inside, the pinned `:has` rule switches the group to
        // `items-start` with `height: auto`, so the box grows downward around
        // the multi-line field instead of centring it in a row.
        let mut group = div()
            .flex()
            .flex_row()
            .map(|g| {
                if is_textarea {
                    g.items_start()
                } else {
                    g.items_center()
                }
            })
            .min_h(util::FIELD_HEIGHT)
            .text_size(util::FIELD_TEXT)
            .text_color(colors.field.foreground);
        if let Some(entity) = entity {
            group = group.debug_selector(move || format!("input-group-{entity}-group"));
        }

        // v3 rings the *group* on `focus-within`, so the state comes from the
        // field inside it.
        group = util::apply_field_chrome(group, self.variant, is_invalid, focus_within, cx);
        if self.full_width {
            group = group.w_full();
        }
        // `status-disabled` is one dim over the whole group box, so the held
        // field, the addon slots and any composed children dim exactly once;
        // the propagated field skips its own coat (`group_dim`), and the
        // folded label dims itself beside the box. v3's `pointer-events:
        // none` cannot come with it — an arbitrary child of a disabled group
        // only dims here, it does not go inert.
        if is_disabled {
            group = group.opacity(layout.disabled_opacity);
        }

        // `.input-group:hover:not(:focus-within)` is `bg-field-hover` plus
        // `--field-border-hover`, and `.input-group--secondary` swaps only the
        // fill for `--input-group-bg-hover: var(--default-hover)`. The
        // refinement is baked off while the focus is inside, and a focus
        // change repaints through a re-render, so the suppressed hover never
        // paints over the focused chrome. v3's `status-disabled` is
        // `pointer-events: none` first, so a disabled group hovers never.
        if !focus_within && !is_disabled {
            let hover_bg = match self.variant {
                FieldVariant::Primary => colors.field.hover(),
                FieldVariant::Secondary => colors.default.hover(),
            };
            let hover_border = colors.field.border_hover();
            group = group.hover(move |style| style.bg(hover_bg).border_color(hover_border));
        }

        // v3.2.4 `InputGroupRoot.handleClick`: a click on the group outside
        // the contained input focuses the input (`target !== input &&
        // !input.contains(target)` -> `input.focus()`), so clicking the
        // prefix or the suffix starts typing in the field instead of
        // blurring it. The handler sits on this root box — no wrapper above
        // the field — and bubble order runs the deeper element first, so the
        // dispatch has already settled which handle the press lands on:
        // a click inside the input finds the field holding the focus (its
        // own mouse-down placed the caret) and must not write a second focus
        // behind it, while a click on a focusable suffix (a Button) finds
        // the button holding it and hands the focus to the field — the
        // button's click still fires on mouse-up, because gpui decides
        // clicks by hover, not focus, so the pinned order "action, then the
        // field" holds. That check is the focused handle itself, not
        // `default_prevented`: the input's own focus transfer prevented
        // default too, and skipping on that alone would exempt the very
        // suffix button v3 hands the focus past. `prevent_default` runs on
        // both arms — it is what keeps the press from falling through to a
        // focusable ancestor, which would otherwise take the focus the
        // moment the dispatch reaches the app focus root. Two pinned
        // exceptions keep the handler off the box: `querySelector("input")`
        // finds nothing in a textarea-only group, so a click there focuses
        // nothing; and a disabled field — the group's own flag, propagated
        // below, or the field's — never takes a browser focus.
        let field_disabled = self
            .input
            .as_ref()
            .is_some_and(|input| input.builder_is_disabled());
        if !is_textarea && !is_disabled && !field_disabled {
            if let Some(focus) = self.input.as_ref().map(|input| input.state_focus(cx)) {
                group = group.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    if !window.focused(cx).is_some_and(|held| held == focus) {
                        window.focus(&focus, cx);
                    }
                    window.prevent_default();
                });
            }
        }

        // Order matters: prefix, field, suffix, then anything else the caller
        // put in. The field is told which sides an addon occupies so it can
        // drop that padding. Group-disabled reaches the field itself — the
        // propagated field stops tracking focus and answering keys, exactly
        // as a browser's disabled `<input>` does, and skips its own dim
        // because the box above already carries it.
        let addon_slot = |el: AnyElement, name: &'static str| -> AnyElement {
            // The slot only exists for the pinned `:has(textarea)` rule; the
            // disabled dim lives on the group box now, so addons sit directly
            // in the row otherwise.
            if !is_textarea {
                return el;
            }
            // `:has([data-slot="input-group-textarea"])` top-aligns the
            // addons and gives each `padding-top: 0.5rem`, so the addon
            // text starts level with the textarea's first line.
            let mut slot = div().flex_shrink_0().pt(px(8.));
            if let Some(entity) = entity {
                slot = slot.debug_selector(move || format!("input-group-{entity}-{name}"));
            }
            slot.child(el).into_any_element()
        };
        let (has_prefix, has_suffix) = (self.prefix.is_some(), self.suffix.is_some());
        if let Some(prefix) = self.prefix {
            group = group.child(addon_slot(prefix, "prefix"));
        }
        if let Some(input) = self.input {
            let input = if is_disabled {
                input.is_disabled(true).group_dim(true)
            } else {
                input
            };
            group = group.child(input.in_group(has_prefix, has_suffix));
        }
        if let Some(suffix) = self.suffix {
            group = group.child(addon_slot(suffix, "suffix"));
        }
        group = group.children(self.children);

        // `.input-group` wrapper is `gap-1`, like every other field.
        let mut root = div().flex().flex_col().gap(px(4.));
        if self.full_width {
            // `.input-group--full-width` is `w-full`, and the wrapper above
            // the group has to carry the width too: gpui resolves a
            // percentage against the parent, and a content-sized parent
            // stretches nothing.
            root = root.w_full();
        }
        if let Some(label) = self.label {
            root = root.child(
                crate::field::Label::new(label)
                    .is_invalid(is_invalid)
                    .is_required(self.is_required)
                    .is_disabled(is_disabled),
            );
        }
        root = root.child(group);

        if is_invalid {
            if let Some(message) = self.error_message {
                root = root.child(crate::field::ErrorMessage::new(message));
            }
        } else if let Some(description) = self.description {
            root = root.child(crate::field::Description::new(description));
        }

        root
    }
}

#[cfg(test)]
mod tests {
    // The pinned `.input-group--secondary:hover` fill is
    // `--input-group-bg-hover: var(--default-hover)`, which
    // `RoleColor::hover()` computes; `soft_hover()` is a different, lighter
    // token. The two accessors differ by one word and the wrong one still
    // looks plausible on screen, so the check is mechanical: the source must
    // name the pinned accessor, and the wrong one must not appear at all.
    #[test]
    fn secondary_hover_uses_the_pinned_default_hover_token() {
        // Scan the implementation only; this test module's own text mentions
        // the wrong accessor to forbid it.
        let source = include_str!("input_group.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        assert!(
            source.contains("FieldVariant::Secondary => colors.default.hover()"),
            "the secondary group hover must read `colors.default.hover()` \
             (pinned `--input-group-bg-hover: var(--default-hover)`)"
        );
        assert!(
            !source.contains("soft_hover()"),
            "`soft_hover()` is not the pinned hover token for the group and \
             must not come back"
        );
    }

    // v3's `status-disabled` is `pointer-events: none` before it is an
    // opacity, so a disabled group paints the hover refinement never.
    #[test]
    fn the_disabled_group_paints_no_hover() {
        let source = include_str!("input_group.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        assert!(
            source.contains("if !focus_within && !is_disabled {"),
            "the hover refinement must be gated off while the group is \
             disabled, not only while the focus is inside"
        );
    }

    // One dim per box: the group carries the whole `status-disabled` opacity
    // and the propagated field skips its own coat, so the opacity must not
    // nest (`group_dim`), and the dim must sit on the group box so composed
    // children dim with it.
    #[test]
    fn the_disabled_dim_covers_the_box_once() {
        let source = include_str!("input_group.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        assert!(
            source.contains("input.is_disabled(true).group_dim(true)"),
            "the propagated field must be told the group already dims the box"
        );
        assert!(
            source.contains("group = group.opacity(layout.disabled_opacity)"),
            "the one dim must sit on the group box so composed children dim \
             with it"
        );
    }
}
