//! A downstream application's UI test, written against `herogpui::test`
//! alone: open the window, click the button by its debug selector, activate
//! it from the keyboard, and read the view's state back.

use herogpui::test::{open_window, TestWindowExt};
use herogpui::*;
// With `test-support`, the root glob also carries GPUI's `test` attribute;
// restore the built-in one, which `#[gpui::test]` expands to.
use ::core::prelude::v1::test;

struct Hello {
    presses: usize,
}

impl Render for Hello {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        app_focus_root(
            div().size_full().child(
                div().debug_selector(|| "hello".into()).child(
                    Button::new("hello").label("Press me").on_press(cx.listener(
                        |this, _, _, cx| {
                            this.presses += 1;
                            cx.notify();
                        },
                    )),
                ),
            ),
            window,
            cx,
        )
    }
}

#[gpui::test]
fn click_and_keyboard_activation(cx: &mut TestAppContext) {
    let (view, cx) = open_window(cx, |_, _| Hello { presses: 0 });
    assert!(cx.find("missing").is_none());
    let bounds = cx.expect("hello");
    assert!(bounds.size.width > px(0.));

    cx.click("hello");
    assert_eq!(view.read_with(cx, |v, _| v.presses), 1);

    // The click focused the button; Enter activates it on key-up.
    cx.press("enter");
    assert_eq!(view.read_with(cx, |v, _| v.presses), 2);
}
