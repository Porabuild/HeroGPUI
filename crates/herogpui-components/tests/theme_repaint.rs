//! Theme provider repaint — global theme mutations must redraw every open window.
//!
//! `set_theme`, `use_theme` and `set_reduce_motion` mutate an app-level global.
//! gpui tracks no dependencies on globals, so a mutation alone marks no window
//! dirty: a second open window keeps painting stale tokens until an unrelated
//! event happens to redraw it. On the headless test platform the effect flush
//! literally redraws every window that `App::refresh_windows` marked dirty, so
//! two windows that count their frames and report the theme id each frame
//! painted prove the contract in both directions.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gpui::{div, prelude::*, Context, Render, TestAppContext, Window};
use herogpui_theme::{
    set_reduce_motion, set_theme, toggle_light_dark, use_theme, ActiveTheme, Theme, ThemeProvider,
};

/// Root view of a probe window. Counts its frames and records which theme id
/// the latest frame painted, so the assertions can tell "redrew at all" apart
/// from "redrew with the new theme".
struct Probe {
    frames: Rc<Cell<u32>>,
    seen_theme: Rc<RefCell<String>>,
}

impl Render for Probe {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        self.frames.set(self.frames.get() + 1);
        *self.seen_theme.borrow_mut() = cx.theme().id.to_string();
        div()
    }
}

#[gpui::test]
fn theme_mutations_repaint_every_open_window(cx: &mut TestAppContext) {
    cx.update(ThemeProvider::init);

    let frames_a = Rc::new(Cell::new(0));
    let frames_b = Rc::new(Cell::new(0));
    let seen_a = Rc::new(RefCell::new(String::new()));
    let seen_b = Rc::new(RefCell::new(String::new()));

    let _window_a = cx.add_window_view(|_, _| Probe {
        frames: frames_a.clone(),
        seen_theme: seen_a.clone(),
    });
    let _window_b = cx.add_window_view(|_, _| Probe {
        frames: frames_b.clone(),
        seen_theme: seen_b.clone(),
    });

    // Baseline: both windows painted the initial light theme and are idle.
    assert_eq!(seen_a.borrow().as_str(), "light");
    assert_eq!(seen_b.borrow().as_str(), "light");
    let baseline_a = frames_a.get();
    let baseline_b = frames_b.get();

    // Switching by id must repaint both windows with the new tokens.
    cx.update(|cx| use_theme("dark", cx));
    assert_eq!(
        frames_a.get(),
        baseline_a + 1,
        "window A must repaint after use_theme"
    );
    assert_eq!(
        frames_b.get(),
        baseline_b + 1,
        "window B must repaint after use_theme"
    );
    assert_eq!(seen_a.borrow().as_str(), "dark");
    assert_eq!(seen_b.borrow().as_str(), "dark");

    // Registering and activating a custom theme repaints both windows too.
    cx.update(|cx| set_theme(Theme::builder("dusk", Theme::dark()).build(), cx));
    assert_eq!(
        frames_a.get(),
        baseline_a + 2,
        "window A must repaint after set_theme"
    );
    assert_eq!(
        frames_b.get(),
        baseline_b + 2,
        "window B must repaint after set_theme"
    );
    assert_eq!(seen_a.borrow().as_str(), "dusk");
    assert_eq!(seen_b.borrow().as_str(), "dusk");

    // The reduced-motion preference repaints both windows without any token
    // changing: the frame count is the only witness, the painted id must stay.
    cx.update(|cx| set_reduce_motion(true, cx));
    assert_eq!(
        frames_a.get(),
        baseline_a + 3,
        "window A must repaint after set_reduce_motion"
    );
    assert_eq!(
        frames_b.get(),
        baseline_b + 3,
        "window B must repaint after set_reduce_motion"
    );
    assert_eq!(seen_a.borrow().as_str(), "dusk");
    assert_eq!(seen_b.borrow().as_str(), "dusk");
    assert!(
        cx.read(ActiveTheme::reduce_motion),
        "the preference itself must be readable app-wide"
    );

    // The light/dark toggle inherits the repaint through `use_theme`.
    cx.update(toggle_light_dark);
    assert_eq!(
        frames_a.get(),
        baseline_a + 4,
        "window A must repaint after toggle_light_dark"
    );
    assert_eq!(
        frames_b.get(),
        baseline_b + 4,
        "window B must repaint after toggle_light_dark"
    );
    assert_eq!(seen_a.borrow().as_str(), "light");
    assert_eq!(seen_b.borrow().as_str(), "light");
}
