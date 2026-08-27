//! Shared harness for the behaviour tests.
//!
//! A test binary opts in with `mod harness;` — Cargo compiles this file as an
//! ordinary module of that binary, never as a test target of its own. The
//! helpers exist because a control can draw perfectly and not work: they open
//! one real gpui window on the headless test platform (`test-support`,
//! enabled in this crate's dev-dependencies) and drive it with simulated
//! clicks and keystrokes.
//!
//! Two harness facts worth knowing before adding more tests:
//!
//! - Component state lives in the *window's* keyed state
//!   (`window.use_keyed_state`), so one host window must survive a whole test.
//!   The builder closures re-run on every frame — that is how these components
//!   are always driven — and only the keyed state carries the value across
//!   frames.
//! - The test platform ships no asset source (`AssetSource for ()` answers
//!   `Ok(None)`), so every `svg()` glyph silently renders nothing. That path
//!   logs instead of panicking, which is why no stub source is installed here.

// Each test binary compiles this module separately, so a helper that one
// binary does not call is dead code *in that binary* even though its siblings
// use it. The allow keeps the shared surface from sprouting per-file copies.
#![allow(dead_code)]

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    canvas, point, prelude::*, px, AnyElement, Context, KeyUpEvent, Keystroke, Modifiers, Render,
    TestAppContext, VisualTestContext, Window,
};
use herogpui_components::{util, Tooltip, TooltipHover};
use herogpui_theme::ThemeProvider;

/// What the component callbacks recorded, cloned into each closure.
pub type Events = Rc<RefCell<Vec<String>>>;

/// An empty recorder, ready to be cloned into the builder closures.
pub fn events() -> Events {
    Rc::new(RefCell::new(Vec::new()))
}

/// Reads one tooltip's keyed open state from the component's render id path.
pub fn tooltip_open_probe(id: &'static str, seen: Events, focus_open: bool) -> AnyElement {
    canvas(
        move |_, window, cx| {
            let open = window.with_id(std::any::type_name::<Tooltip>(), |window| {
                let state = window
                    .use_keyed_state(gpui::ElementId::Name(id.into()), cx, |_, _| {
                        TooltipHover::closed()
                    })
                    .read(cx);
                if focus_open {
                    state.is_focus_open()
                } else {
                    state.is_open()
                }
            });
            seen.borrow_mut().push(format!("open:{open}"));
        },
        |_, _, _, _| {},
    )
    .size_0()
    .into_any_element()
}

/// Renders one component under test at the top-left corner of the window, with
/// no padding, so simulated click coordinates land where the layout says.
pub struct Host {
    content: Box<dyn Fn() -> AnyElement>,
}

impl Render for Host {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let root = gpui::div().size_full().child((self.content)());
        util::app_focus_root(root, window, cx)
    }
}

/// Installs the theme global and opens one host window on the test platform.
pub fn open_host(
    cx: &mut TestAppContext,
    content: impl Fn() -> AnyElement + 'static,
) -> &mut VisualTestContext {
    // Every component reads its tokens through the `ThemeProvider` global;
    // drawing without one panics.
    cx.update(ThemeProvider::init);
    let (_view, cx) = cx.add_window_view(|_, _| Host {
        content: Box::new(content),
    });
    cx
}

/// One hit-tested left click at window coordinates (`x`, `y`).
pub fn click(cx: &mut VisualTestContext, x: f32, y: f32) {
    cx.simulate_click(point(px(x), px(y)), Modifiers::none());
}

/// Types `keys` (a space-separated keystroke string) and releases the last key.
///
/// **The most surprising fact in this harness:** gpui activates a focused
/// element on key *up*. An element that holds the focus fires its click
/// listeners when Enter or Space is *released* — the listener is registered on
/// `KeyUpEvent` — and `dispatch_keystroke`, which backs
/// `simulate_keystrokes`, sends only the down half. Without the explicit
/// `KeyUpEvent` below, `"enter"` reaches every `on_key_down` along the focus
/// chain but activates nothing that waits for the click.
pub fn press(cx: &mut VisualTestContext, keys: &str) {
    cx.simulate_keystrokes(keys);
    if let Some(last) = keys.split_whitespace().next_back() {
        cx.simulate_event(KeyUpEvent {
            keystroke: Keystroke::parse(last).unwrap(),
        });
    }
}
