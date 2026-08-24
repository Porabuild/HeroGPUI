//! Interaction tests — driving the controls, not looking at them.
//!
//! Every static axis of this port is measured by the `.shots/*.py` audits;
//! what they cannot see is whether a control *functions*. These tests open a
//! real gpui window on the headless test platform (`test-support`, enabled in
//! this crate's dev-dependencies), render one component at the top-left corner
//! and simulate clicks and keystrokes against it.
//!
//! Two harness facts worth knowing before adding more tests here:
//!
//! - Component state lives in the *window's* keyed state
//!   (`window.use_keyed_state`), so one host window must survive a whole test.
//!   The builder closures re-run on every frame — that is how these components
//!   are always driven — and only the keyed state carries the value across
//!   frames.
//! - The test platform ships no asset source (`AssetSource for ()` answers
//!   `Ok(None)`), so every `svg()` glyph silently renders nothing. That path
//!   logs instead of panicking, which is why no stub source is installed here.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    point, prelude::*, px, AnyElement, Context, KeyUpEvent, Keystroke, Modifiers, Render,
    TestAppContext, VisualTestContext, Window,
};
use herogpui_components::{Checkbox, Select, Switch};
use herogpui_theme::ThemeProvider;

/// What the component callbacks recorded, cloned into each closure.
type Events = Rc<RefCell<Vec<String>>>;

/// Renders one component under test at the top-left corner of the window, with
/// no padding, so simulated click coordinates land where the layout says.
struct Host {
    content: Box<dyn Fn() -> AnyElement>,
}

impl Render for Host {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        // A minimal stand-in for `util::app_focus_root`: hold the focus when
        // nothing else does — with no focus there is no key-event chain for the
        // first Tab to travel down — and move it on Tab. gpui leaves both jobs
        // to the app root; the gallery installs one, so the harness must too.
        let root = window
            .use_keyed_state(
                gpui::ElementId::Name("host-focus-root".into()),
                cx,
                |_, cx| cx.focus_handle(),
            )
            .read(cx)
            .clone();
        if !root.contains_focused(window, cx) {
            window.focus(&root);
        }
        gpui::div()
            .track_focus(&root)
            .on_key_down(|event, window, _| {
                if event.keystroke.key == "tab" {
                    window.focus_next();
                }
            })
            .size_full()
            .child((self.content)())
    }
}

/// Installs the theme global and opens one host window on the test platform.
fn open_host(
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

#[gpui::test]
fn checkbox_click_toggles(cx: &mut TestAppContext) {
    let events: Events = Rc::new(RefCell::new(Vec::new()));
    let recorded = events.clone();
    let cx = open_host(cx, move || {
        let events = events.clone();
        Checkbox::new("cb")
            .label("Toggle")
            .on_change(move |checked, _, _| {
                events.borrow_mut().push(format!("change:{checked}"));
            })
            .into_any_element()
    });

    // The row starts at the window origin and the 16px control box is its
    // first child, centred at (8, 8).
    cx.simulate_click(point(px(8.), px(8.)), Modifiers::none());
    assert_eq!(
        recorded.borrow().as_slice(),
        ["change:true"],
        "first click must check the box"
    );

    cx.simulate_click(point(px(8.), px(8.)), Modifiers::none());
    assert_eq!(
        recorded.borrow().as_slice(),
        ["change:true", "change:false"],
        "second click must uncheck it"
    );
}

#[gpui::test]
fn switch_keyboard_activates(cx: &mut TestAppContext) {
    let events: Events = Rc::new(RefCell::new(Vec::new()));
    let recorded = events.clone();
    let cx = open_host(cx, move || {
        let events = events.clone();
        Switch::new("sw")
            .label("Wi-Fi")
            .on_change(move |checked, _, _| {
                events.borrow_mut().push(format!("change:{checked}"));
            })
            .into_any_element()
    });

    // Tab moves the focus from the harness root onto the track without firing
    // anything (a click would have toggled it once already). Space then
    // activates the focused track exactly once: gpui fires a focused element's
    // click listeners on Enter and Space, and a component that also bound its
    // own key handler would fire twice — the double-fire bug class this guards.
    //
    // The activation listens on key *up*, and `dispatch_keystroke` sends only
    // the down half, so the release is delivered explicitly.
    cx.simulate_keystrokes("tab space");
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("space").unwrap(),
    });

    assert_eq!(
        recorded.borrow().as_slice(),
        ["change:true"],
        "one keystroke must toggle the switch exactly once"
    );
}

#[gpui::test]
fn select_click_selects_and_closes(cx: &mut TestAppContext) {
    let events: Events = Rc::new(RefCell::new(Vec::new()));
    let recorded = events.clone();
    let cx = open_host(cx, move || {
        let selection = events.clone();
        let opening = events.clone();
        Select::new("sel", vec!["Rust".into(), "Go".into()])
            .on_selection_change(move |index, _, _| {
                selection.borrow_mut().push(format!("select:{index:?}"));
            })
            .on_open_change(move |open, _, _| opening.borrow_mut().push(format!("open:{open}")))
            .into_any_element()
    });

    // The trigger is a full-width row of `util::FIELD_HEIGHT` (36px) starting
    // at the origin; its centre is (60, 18).
    cx.simulate_click(point(px(60.), px(18.)), Modifiers::none());
    assert_eq!(recorded.borrow().as_slice(), ["open:true"]);

    // First option: `placed_field_panel` hangs the panel 6px below the trigger
    // (`top_full` + `mt(6px)` => y 42) with 6px vertical padding and
    // `min_h(FIELD_HEIGHT)` rows, so the row spans roughly y 48..84 and its
    // centre sits near y 66. The enter zoom animates only padding and radius,
    // never enough to move that band.
    cx.simulate_click(point(px(60.), px(66.)), Modifiers::none());
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:true", "select:Some(0)"],
        "clicking the first row must select index 0"
    );

    // Closed proof by behaviour: the same spot is bare page below the trigger
    // now, so the press must reach nothing. Were the popover still open, the
    // row would record a second `select:Some(0)` here.
    cx.simulate_click(point(px(60.), px(66.)), Modifiers::none());
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:true", "select:Some(0)"],
        "the popover must be closed after choosing an option"
    );
}
