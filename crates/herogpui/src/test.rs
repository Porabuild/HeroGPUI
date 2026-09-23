//! A headless UI test kit for applications built on HeroGPUI (HeroGPUI
//! extension; `test-support` feature).
//!
//! It is the public form of the harness HeroGPUI's own behaviour tests use:
//! open one real window on GPUI's headless test platform with the theme
//! installed, find elements by their **debug selector**, and drive them with
//! simulated clicks, keys and text.
//!
//! Elements are addressed by `.debug_selector(|| "name".into())`, which GPUI
//! records in debug and `test-support` builds only (it is a no-op in release
//! builds). GPUI keeps no public element-id → bounds map, so an element you
//! want to address needs a selector; wrap a component in a `div()` that
//! carries one when the component sets none of its own.
//!
//! ```
//! # #[cfg(feature = "components")] {
//! use herogpui::test::{open_window, TestWindowExt};
//! use herogpui::*;
//!
//! struct Counter(usize);
//!
//! impl Render for Counter {
//!     fn render(&mut self, _: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
//!         div().child(
//!             div().debug_selector(|| "save".into()).child(
//!                 Button::new("save")
//!                     .label("Save")
//!                     .on_press(cx.listener(|this, _, _, cx| {
//!                         this.0 += 1;
//!                         cx.notify();
//!                     })),
//!             ),
//!         )
//!     }
//! }
//!
//! let mut app = TestAppContext::single();
//! let (view, cx) = open_window(&mut app, |_, _| Counter(0));
//! cx.click("save");
//! assert_eq!(view.read_with(cx, |c, _| c.0), 1);
//! # }
//! ```
//!
//! Harness facts worth knowing (they are GPUI's, not this kit's):
//!
//! - GPUI activates a focused element on key **up**; [`TestWindowExt::press`](crate::test::TestWindowExt::press)
//!   sends the matching key-up after the keystrokes, which bare
//!   `simulate_keystrokes` does not.
//! - Hit testing uses the last rendered frame, so every helper here redraws
//!   and parks the executor after its event.
//! - The test platform has no asset source, so `svg()` icons draw nothing;
//!   that does not affect layout or events.

use std::cell::RefCell;
use std::collections::HashSet;

use gpui::{
    point, px, Bounds, Context, KeyUpEvent, Keystroke, Modifiers, MouseButton, Pixels, Point,
    Render, ScrollDelta, ScrollWheelEvent, TestAppContext, TouchPhase, VisualTestContext, Window,
};

thread_local! {
    // `VisualTestContext::debug_bounds` takes a `&'static str`. Interning
    // leaks each distinct selector once per thread, which a test binary can
    // afford; a new allocation per lookup would leak on every call.
    static SELECTORS: RefCell<HashSet<&'static str>> = RefCell::new(HashSet::new());
}

fn intern(selector: &str) -> &'static str {
    SELECTORS.with_borrow_mut(|set| {
        if let Some(found) = set.get(selector) {
            return *found;
        }
        let leaked: &'static str = Box::leak(selector.to_owned().into_boxed_str());
        set.insert(leaked);
        leaked
    })
}

/// Initializes HeroGPUI ([`crate::init`]), opens one window whose root view
/// `build` returns, draws it, and parks the pointer outside the window so
/// nothing starts hovered. Returns the view and the window's context.
pub fn open_window<V, F>(
    cx: &mut TestAppContext,
    build: F,
) -> (gpui::Entity<V>, &mut VisualTestContext)
where
    V: Render + 'static,
    F: FnOnce(&mut Window, &mut Context<'_, V>) -> V,
{
    cx.update(crate::init);
    let (view, cx) = cx.add_window_view(build);
    cx.simulate_mouse_move(point(px(-100.), px(-100.)), None, Modifiers::none());
    cx.run_until_parked();
    (view, cx)
}

/// Selector-addressed input and queries on a test window.
pub trait TestWindowExt {
    /// Redraws the window and runs every pending task.
    fn settle(&mut self);
    /// The last-rendered bounds of the element with this debug selector.
    fn find(&mut self, selector: &str) -> Option<Bounds<Pixels>>;
    /// Like [`find`](Self::find), but panics naming the missing selector.
    fn expect(&mut self, selector: &str) -> Bounds<Pixels>;
    /// A left click at the centre of the element with this selector.
    fn click(&mut self, selector: &str);
    /// A left click at window coordinates.
    fn click_at(&mut self, position: Point<Pixels>);
    /// Moves the pointer to the centre of the element with this selector.
    fn hover(&mut self, selector: &str);
    /// Space-separated keystrokes (`"tab enter"`, `"cmd-a"`), then the
    /// key-up of the last one, so a focused control activates.
    fn press(&mut self, keys: &str);
    /// Types `text` into the focused input, as an IME commit would.
    fn type_text(&mut self, text: &str);
    /// Scrolls by `delta` pixels over the element with this selector
    /// (negative `y` scrolls down).
    fn scroll(&mut self, selector: &str, delta: Point<Pixels>);
}

impl TestWindowExt for VisualTestContext {
    fn settle(&mut self) {
        self.update(|window, _| window.refresh());
        self.run_until_parked();
    }

    fn find(&mut self, selector: &str) -> Option<Bounds<Pixels>> {
        self.settle();
        self.debug_bounds(intern(selector))
    }

    fn expect(&mut self, selector: &str) -> Bounds<Pixels> {
        self.find(selector)
            .unwrap_or_else(|| panic!("no element with debug selector {selector:?} was rendered"))
    }

    fn click(&mut self, selector: &str) {
        let center = self.expect(selector).center();
        self.click_at(center);
    }

    fn click_at(&mut self, position: Point<Pixels>) {
        self.simulate_mouse_move(position, None, Modifiers::none());
        self.simulate_mouse_down(position, MouseButton::Left, Modifiers::none());
        self.simulate_mouse_up(position, MouseButton::Left, Modifiers::none());
        self.settle();
    }

    fn hover(&mut self, selector: &str) {
        let center = self.expect(selector).center();
        self.simulate_mouse_move(center, None, Modifiers::none());
        self.settle();
    }

    fn press(&mut self, keys: &str) {
        self.simulate_keystrokes(keys);
        if let Some(last) = keys.split_whitespace().next_back() {
            let keystroke = Keystroke::parse(last)
                .unwrap_or_else(|err| panic!("unparseable keystroke {last:?}: {err}"));
            self.simulate_event(KeyUpEvent { keystroke });
        }
        self.settle();
    }

    fn type_text(&mut self, text: &str) {
        self.simulate_input(text);
        self.settle();
    }

    fn scroll(&mut self, selector: &str, delta: Point<Pixels>) {
        let center = self.expect(selector).center();
        self.simulate_mouse_move(center, None, Modifiers::none());
        self.simulate_event(ScrollWheelEvent {
            position: center,
            delta: ScrollDelta::Pixels(delta),
            modifiers: Modifiers::none(),
            touch_phase: TouchPhase::Moved,
        });
        self.settle();
    }
}
