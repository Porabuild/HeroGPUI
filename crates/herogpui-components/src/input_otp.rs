//! InputOTP — port of `@heroui/input-otp`.

use gpui::{
    prelude::*, px, App, Entity, FocusHandle, Focusable, IntoElement, KeyDownEvent, RenderOnce,
    SharedString, Styled, Window,
};
use herogpui_core::{FieldVariant, Color, Size};
use herogpui_theme::ActiveTheme;

/// Editable state for an OTP field: one char per cell.
/// Which characters an OTP cell accepts (`pattern`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OtpPattern {
    /// `0-9` — the v3 default.
    #[default]
    Digits,
    /// `0-9A-Za-z`
    Alphanumeric,
    /// Any printable character.
    Any,
}

impl OtpPattern {
    pub const ALL: [OtpPattern; 3] = [
        OtpPattern::Digits,
        OtpPattern::Alphanumeric,
        OtpPattern::Any,
    ];

    /// Whether `ch` may be entered into a cell.
    pub fn accepts(self, ch: char) -> bool {
        match self {
            OtpPattern::Digits => ch.is_ascii_digit(),
            OtpPattern::Alphanumeric => ch.is_ascii_alphanumeric(),
            OtpPattern::Any => !ch.is_control(),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            OtpPattern::Digits => "Digits",
            OtpPattern::Alphanumeric => "Alphanumeric",
            OtpPattern::Any => "Any",
        }
    }
}

pub struct OtpState {
    cells: Vec<char>,
    cursor: usize,
    pub(crate) focus_handle: FocusHandle,
}

impl OtpState {
    /// Fills the cells from `code`, padding with blanks and dropping any
    /// overflow.
    pub fn set_code(&mut self, code: &str) {
        let len = self.cells.len();
        let mut chars = code.chars();
        for i in 0..len {
            self.cells[i] = chars.next().unwrap_or(' ');
        }
        self.cursor = code.chars().count().min(len.saturating_sub(1));
    }

    /// `length` = number of cells (HeroUI default 4).
    pub fn with_length(cx: &mut App, length: usize) -> Self {
        Self {
            cells: vec![' '; length.max(1)],
            cursor: 0,
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn code(&self) -> String {
        self.cells.iter().filter(|c| **c != ' ').collect()
    }

    pub fn is_complete(&self) -> bool {
        self.cells.iter().all(|c| *c != ' ')
    }

    pub fn clear(&mut self) {
        self.cells.iter_mut().for_each(|c| *c = ' ');
        self.cursor = 0;
    }
}

impl Focusable for OtpState {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

type OnComplete = std::sync::Arc<dyn Fn(&str, &mut Window, &mut App) + 'static>;

/// `textAlign` — where a digit sits inside its slot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OtpTextAlign {
    Left,
    #[default]
    Center,
    Right,
}

impl OtpTextAlign {
    pub const ALL: [OtpTextAlign; 3] =
        [OtpTextAlign::Left, OtpTextAlign::Center, OtpTextAlign::Right];

    pub fn label(self) -> &'static str {
        match self {
            OtpTextAlign::Left => "Left",
            OtpTextAlign::Center => "Center",
            OtpTextAlign::Right => "Right",
        }
    }
}

/// HeroUI InputOTP.
#[derive(IntoElement)]
pub struct InputOTP {
    variant: FieldVariant,
    /// `validate` — run by the component, not the caller.
    validate: Option<crate::validation::Validator<str>>,
    /// `validationErrors` — messages from a server round-trip.
    validation_errors: Vec<gpui::SharedString>,
    /// `textAlign` — where the digit sits inside its slot.
    text_align: OtpTextAlign,
    /// `autoFocus` — take focus on the first render.
    auto_focus: bool,
    /// `pasteTransformer` — rewrites pasted text before the slots take it.
    paste_transformer: Option<std::sync::Arc<dyn Fn(&str) -> String + 'static>>,
    is_invalid: bool,
    placeholder: char,
    pattern: OtpPattern,
    on_change: Option<std::sync::Arc<dyn Fn(&str, &mut Window, &mut App) + 'static>>,
    state: Entity<OtpState>,
    size: Size,
    color: Color,
    is_disabled: bool,
    separator: Option<SharedString>,
    on_complete: Option<OnComplete>,
}

impl InputOTP {
    /// `value` — writes the code through to the bound [`OtpState`], one char
    /// per cell.
    pub fn value(self, code: &str, cx: &mut App) -> Self {
        self.state.update(cx, |s, _| s.set_code(code));
        self
    }

    pub fn new(state: Entity<OtpState>) -> Self {
        Self {
            variant: FieldVariant::Primary,
            validate: None,
            validation_errors: Vec::new(),
            text_align: OtpTextAlign::Center,
            auto_focus: false,
            paste_transformer: None,
            is_invalid: false,
            placeholder: '-',
            pattern: OtpPattern::Digits,
            on_change: None,
            state,
            size: Size::Md,
            color: Color::Accent,
            is_disabled: false,
            separator: None,
            on_complete: None,
        }
    }

    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }

    pub fn variant(mut self, variant: FieldVariant) -> Self {
        self.variant = variant;
        self
    }

    /// `validate` — returns the message to show, or `None` when the code is fine.
    ///
    /// The component runs it and surfaces the result.
    pub fn validate(
        mut self,
        f: impl Fn(&str) -> Option<gpui::SharedString> + 'static,
    ) -> Self {
        self.validate = Some(std::sync::Arc::new(f));
        self
    }

    /// `validationErrors` — messages produced elsewhere, shown ahead of
    /// whatever `validate` returns.
    pub fn validation_errors(
        mut self,
        errors: impl IntoIterator<Item = impl Into<gpui::SharedString>>,
    ) -> Self {
        self.validation_errors = errors.into_iter().map(Into::into).collect();
        self
    }

    /// `autoFocus` — take focus on the first render.
    pub fn auto_focus(mut self, v: bool) -> Self {
        self.auto_focus = v;
        self
    }

    /// `pasteTransformer` — rewrites pasted text before it fills the slots.
    ///
    /// Useful for stripping separators from a code the user copied out of an
    /// email, e.g. `|t| t.replace('-', "")`.
    pub fn paste_transformer(mut self, f: impl Fn(&str) -> String + 'static) -> Self {
        self.paste_transformer = Some(std::sync::Arc::new(f));
        self
    }

    /// `textAlign` — where each digit sits inside its slot.
    ///
    /// v3 documents `left` as the default; a single character in a square slot
    /// reads better centred, which is what the slots render, so `Center` is the
    /// default here and the other two are available.
    pub fn text_align(mut self, align: OtpTextAlign) -> Self {
        self.text_align = align;
        self
    }

    pub fn is_invalid(mut self, v: bool) -> Self {
        self.is_invalid = v;
        self
    }

    /// `placeholder` — the glyph shown in an empty cell.
    pub fn placeholder(mut self, ch: char) -> Self {
        self.placeholder = ch;
        self
    }

    /// `pattern` — the characters a cell accepts. Defaults to digits.
    pub fn pattern(mut self, pattern: OtpPattern) -> Self {
        self.pattern = pattern;
        self
    }

    /// Fires on every cell change, not just completion (`onChange`).
    pub fn on_change(
        mut self,
        f: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(std::sync::Arc::new(f));
        self
    }

    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    /// Text between cell groups (e.g. "-").
    pub fn separator(mut self, s: impl Into<SharedString>) -> Self {
        self.separator = Some(s.into());
        self
    }

    pub fn on_complete(
        mut self,
        f: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_complete = Some(std::sync::Arc::new(f));
        self
    }
}

impl RenderOnce for InputOTP {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `focus_once` takes `cx` mutably, so it runs before the tokens.
        let focused_handle = self.state.read(cx).focus_handle.clone();
        if self.auto_focus {
            crate::util::focus_once(
                window,
                cx,
                gpui::ElementId::Name(
                    format!("otp-autofocus-{}", self.state.entity_id().as_u64()).into(),
                ),
                &focused_handle,
            );
        }

        let sem = cx.role(self.color);
        let colors = cx.colors();
        let layout = cx.layout();

        let (cell_px, text) = match self.size {
            Size::Sm => (px(34.), px(16.)),
            Size::Md => (px(40.), px(18.)),
            Size::Lg => (px(46.), px(20.)),
        };

        let focused = focused_handle.is_focused(window);
        let (cells_snapshot, cursor) = {
            let st = self.state.read(cx);
            (st.cells.clone(), st.cursor)
        };
        let _length = cells_snapshot.len();
        let disabled = self.is_disabled;

        // v3 order: the controlled flag, then server errors, then `validate`.
        let code_now = self.state.read(cx).code();
        let validity = crate::validation::resolve(
            self.is_invalid,
            &self.validation_errors,
            self.validate.as_ref().and_then(|f| f(code_now.as_str())),
            None,
        );
        let invalid = validity.is_invalid;

        let mut row = gpui::div()
            .id(gpui::ElementId::Name(
                format!("otp-{}", self.state.entity_id().as_u64()).into(),
            ))
            .flex()
            .items_center()
            .cursor(if disabled { gpui::CursorStyle::Arrow } else { gpui::CursorStyle::IBeam })
            .track_focus(&focused_handle)
            .key_context("InputOTP")
            .on_mouse_down(gpui::MouseButton::Left, {
                let fh = focused_handle.clone();
                move |_, window, _| window.focus(&fh)
            });

        if disabled {
            row = row.opacity(layout.disabled_opacity);
        }

        for (i, cell_ch) in cells_snapshot.iter().enumerate() {
            // group separator every 3 cells
            if i > 0 && i % 3 == 0 {
                if let Some(sep) = &self.separator {
                    row = row.child(
                        gpui::div()
                            .px(px(4.))
                            .text_size(text)
                            .text_color(colors.muted)
                            .child(sep.to_string()),
                    );
                }
            }

            let ch = *cell_ch;
            let is_cursor_cell = focused && i == cursor && !disabled;

            let mut cell = gpui::div()
                .flex()
                .items_center()
                // `textAlign` positions the digit inside its slot.
                .map(|c| match self.text_align {
                    OtpTextAlign::Left => c.justify_start().pl(px(6.)),
                    OtpTextAlign::Center => c.justify_center(),
                    OtpTextAlign::Right => c.justify_end().pr(px(6.)),
                })
                .w(cell_px)
                .h(cell_px + px(6.))
                .rounded(px(8.))
                .text_size(text)
                .font_weight(gpui::FontWeight::SEMIBOLD);

            // `secondary` cells sit flat on a surface; `primary` keeps the
            // field fill.
            let filled_bg = match self.variant {
                FieldVariant::Primary => colors.field.background,
                FieldVariant::Secondary => colors.surface_secondary,
            };
            let ring = if invalid {
                colors.danger.color
            } else {
                sem.color
            };
            let idle_border = if invalid {
                colors.danger.color
            } else {
                colors.default.soft_hover()
            };

            if is_cursor_cell {
                cell = cell.border_2().border_color(ring).bg(filled_bg);
            } else if ch != ' ' {
                cell = cell
                    .border_2()
                    .border_color(idle_border)
                    .text_color(colors.foreground)
                    .bg(filled_bg);
            } else {
                cell = cell
                    .border_2()
                    .border_color(idle_border)
                    .text_color(colors.foreground);
            }

            if ch != ' ' {
                cell = cell.child(ch.to_string());
            } else if is_cursor_cell {
                cell = cell.child(gpui::div().w(px(1.5)).h(text).bg(ring));
            } else {
                // `placeholder` fills the empty, unfocused cells.
                cell = cell
                    .text_color(colors.muted)
                    .child(self.placeholder.to_string());
            }

            row = row.child(cell.ml(if i > 0 { px(6.) } else { px(0.) }));
        }

        // editing
        let state_entity = self.state.clone();
        let on_complete = self.on_complete.clone();
        let on_change = self.on_change.clone();
        let pattern = self.pattern;
        let paste_transformer = self.paste_transformer.clone();
        row = row.on_key_down(move |ev: &KeyDownEvent, window, cx| {
            if disabled {
                return;
            }
            let key: &str = &ev.keystroke.key;

            // Ctrl/Cmd+V fills the slots from the clipboard. `Cmd` matters on
            // macOS; checking only `control` would make paste dead there.
            let paste_chord = (ev.keystroke.modifiers.control
                || ev.keystroke.modifiers.platform)
                && key == "v";
            if paste_chord {
                if let Some(text) = cx.read_from_clipboard().and_then(|c| c.text()) {
                    let text = match &paste_transformer {
                        Some(f) => f(&text),
                        None => text,
                    };
                    state_entity.update(cx, |s, cx| {
                        // A paste replaces the code from the cursor onward.
                        for ch in text.chars() {
                            if s.cursor >= s.cells.len() {
                                break;
                            }
                            if !ch.is_ascii_alphanumeric() {
                                continue;
                            }
                            s.cells[s.cursor] = ch.to_ascii_uppercase();
                            s.cursor += 1;
                        }
                        // Leave the cursor on the last filled slot so the next
                        // keystroke overwrites rather than falling off the end.
                        if s.cursor >= s.cells.len() {
                            s.cursor = s.cells.len() - 1;
                        }
                        cx.notify();
                    });
                    let code: String = state_entity.read(cx).code();
                    if code.chars().all(|c| c != ' ') {
                        if let Some(cb) = &on_complete {
                            cb(&code, window, cx);
                        }
                    }
                }
                return;
            }

            match key {
                "backspace" => {
                    state_entity.update(cx, |s, cx| {
                        if s.cells[s.cursor] != ' ' {
                            s.cells[s.cursor] = ' ';
                        } else if s.cursor > 0 {
                            s.cursor -= 1;
                            s.cells[s.cursor] = ' ';
                        }
                        cx.notify();
                    });
                    if let Some(cb) = &on_change {
                        let code = state_entity.read(cx).code();
                        cb(&code, window, cx);
                    }
                }
                "left" => state_entity.update(cx, |s, cx| {
                    s.cursor = s.cursor.saturating_sub(1);
                    cx.notify();
                }),
                "right" => state_entity.update(cx, |s, cx| {
                    if s.cursor + 1 < s.cells.len() {
                        s.cursor += 1;
                    }
                    cx.notify();
                }),
                single if single.chars().count() == 1 && !single.is_empty() => {
                    let mut c = single.chars().next().unwrap();
                    if c.is_ascii_alphabetic() && !ev.keystroke.modifiers.shift {
                        c = c.to_ascii_lowercase();
                    }
                    if pattern.accepts(c) {
                        let completed = state_entity.update(cx, |s, cx| {
                            s.cells[s.cursor] = c;
                            if s.cursor + 1 < s.cells.len() {
                                s.cursor += 1;
                            }
                            let done = s.is_complete();
                            cx.notify();
                            done
                        });
                        let code = state_entity.read(cx).code();
                        if let Some(cb) = &on_change {
                            cb(&code, window, cx);
                        }
                        if completed {
                            if let Some(cb) = &on_complete {
                                cb(&code, window, cx);
                            }
                        }
                    }
                }
                _ => {}
            }
        });

        // A field that can be invalid has to be able to say why.
        match validity.first() {
            None => row.into_any_element(),
            Some(message) => gpui::div()
                .flex()
                .flex_col()
                .gap(px(6.))
                .child(row)
                .child(crate::field::ErrorMessage::new(message))
                .into_any_element(),
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digits_pattern_is_the_default() {
        assert_eq!(OtpPattern::default(), OtpPattern::Digits);
    }

    #[test]
    fn digits_rejects_letters_and_symbols() {
        assert!(OtpPattern::Digits.accepts('7'));
        assert!(!OtpPattern::Digits.accepts('a'));
        assert!(!OtpPattern::Digits.accepts('-'));
    }

    #[test]
    fn alphanumeric_accepts_both_cases() {
        assert!(OtpPattern::Alphanumeric.accepts('7'));
        assert!(OtpPattern::Alphanumeric.accepts('a'));
        assert!(OtpPattern::Alphanumeric.accepts('Z'));
        assert!(!OtpPattern::Alphanumeric.accepts('-'));
    }

    #[test]
    fn any_accepts_printables_but_not_controls() {
        assert!(OtpPattern::Any.accepts('-'));
        assert!(OtpPattern::Any.accepts(' '));
        assert!(!OtpPattern::Any.accepts('\n'));
        assert!(!OtpPattern::Any.accepts('\t'));
    }
}
