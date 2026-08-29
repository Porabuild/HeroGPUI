//! Behaviour tests for the collection and navigation components: ListBox,
//! TagGroup, Tabs, Accordion, Pagination and Breadcrumbs.
//!
//! Everything static about them is measured by the `.shots/*.py` audits; these
//! tests drive the controls and assert on recorded callbacks and behavioural
//! probes only — never on appearance.
//!
//! Geometry is derived from the components' own constants, not guessed. Where a
//! click target's x depends on a label's width (Tabs, Breadcrumbs), the width is
//! *measured* with the same text system the renderer shapes with
//! (`Window::text_system().shape_line`), so the derivation holds for the machine
//! the tests run on instead of assuming one font metric:
//!
//! - ListBox: `.list-box` is `p-1` (4px) with `mt-1` (4px) between children and
//!   each row is `min-h(util::FIELD_HEIGHT)` = 36px, so row *i*'s centre sits at
//!   y = 4 + i*(36+4) + 18 = 22 + 40i. Rows stretch the full window width.
//! - TagGroup: `.tag--md` is `px-2 py-1` (8/4), the list gaps chips by 6px, and
//!   the remove button is `size-3` (12px). The test's `tag_content` draws the
//!   label as a fixed 40x20 box so *every* distance is a constant: a chip is
//!   8+40+4+12+8 = 72px wide and 20+8 = 28px tall, and its remove button spans
//!   x 52..64 at y 8..20 — centre (58, 14); chip *i* starts at 78i.
//! - Tabs: the list is `p-1` with the tabs shoulder to shoulder, each tab
//!   `h-8 px-4` (32px tall, 16px side padding). Tab *i* is `w_i + 32` wide where
//!   `w_i` is the measured label width, so tab 1's centre is at
//!   4 + w_0 + 32 + (w_1 + 32)/2 at y = 4 + 16 = 20.
//! - Accordion: a trigger is `px-4 py-4` (16px all round) around a single line
//!   of `line_height(px(20.))`, so it is 52px tall; items are joined by a 1px
//!   separator. Closed headers *i* centre at 26 + 53i. In the single-expand test
//!   each body is given a fixed 40px-tall content so the second header's centre
//!   is exactly 52 + (2+40+16) + 1 + 26 = 137.
//! - Pagination: `size-md` cells are 32px squares; a nav button is `px-2.5`
//!   (10px each side) around a 14px glyph, so 34px wide; the row gaps items by
//!   4px. Prev spans x 0..34, then page *n*'s cell starts at 38+36(n-1), and the
//!   next button starts at 146 (three cells later) and centres at 163; y = 16.
//! - Breadcrumbs: each crumb row is `px-0.5` (2px) around a `px-0.5` (2px)
//!   measured label plus `gap-0.5` (2px) and a 12px separator slot, so the
//!   second label starts at w_0 + 26 and a label centres 4px into its row; the
//!   label line is `leading-5` = 20px, so the centre y is 10.
//!
//! Each instance gets its own element id; two components sharing an id share
//! their keyed state, which AGENTS.md documents as a silent failure.

mod harness;

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use gpui::{prelude::*, px, Font, FontFeatures, FontStyle, FontWeight, TestAppContext};
use herogpui_components::{
    util::FIELD_HEIGHT, Accordion, AccordionItem, Breadcrumbs, Crumb, ListBox, ListBoxItem,
    Pagination, SelectionMode, TabItem, Tabs, Tag, TagGroup,
};

use harness::{click, events, open_host, press};

/// The y centre of ListBox row `i`.
///
/// Derived from the component's own numbers, in the order the code applies
/// them: `.list-box` is `p-1` (4px top padding), each row is
/// `min-h(util::FIELD_HEIGHT)` = 36px tall, and the children are separated by
/// `mt-1` = 4px (`gap`), so row *i* sits at 4 + i*(36 + 4) and its centre is
/// half a row further down.
fn list_row_centre(i: usize) -> f32 {
    let row = f32::from(FIELD_HEIGHT);
    4. + i as f32 * (row + 4.) + row / 2.
}

/// The keys of a selection joined in a stable order.
///
/// A `HashSet` iterates in no particular order, so asserting on a raw join
/// would be flaky; sorting makes the recorded report deterministic.
fn sorted_join(keys: &HashSet<gpui::SharedString>) -> String {
    let mut keys: Vec<String> = keys.iter().map(ToString::to_string).collect();
    keys.sort();
    keys.join(",")
}

/// The advance width of `text` shaped the way the components shape it: gpui's
/// default `.SystemUIFont` stack at `size` px and `weight`.
///
/// Tabs labels are `text_size(14)` + MEDIUM; breadcrumb labels are 14px at
/// MEDIUM (`.breadcrumbs__link` is `font-medium`). Both are laid out by the
/// window's own `WindowTextSystem`, so this measurement is the render's
/// measurement.
fn text_width(system: &gpui::WindowTextSystem, text: &str, size: f32, weight: FontWeight) -> f32 {
    let run = gpui::TextRun {
        len: text.len(),
        font: Font {
            family: ".SystemUIFont".into(),
            features: FontFeatures::default(),
            weight,
            style: FontStyle::default(),
            fallbacks: None,
        },
        color: gpui::black(),
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let line = system.shape_line(text.to_owned().into(), px(size), &[run], None);
    f32::from(line.width)
}

// ---------------------------------------------------------------------------
// ListBox
// ---------------------------------------------------------------------------
//
// Row geometry, from ListBox::new's own comment (`.list-box` is `p-1` with
// `mt-1` between children) and the row code (`min-h(util::FIELD_HEIGHT)`):
// row *i* occupies y 4+40i .. 44+40i, centre y = 22 + 40i. All clicks use
// x = 60, well inside a full-width row.

#[gpui::test]
fn list_box_click_selects_and_arrows_move(cx: &mut TestAppContext) {
    let events = events();
    let recorded = events.clone();
    let cx = open_host(cx, move || {
        let events = events.clone();
        ListBox::new(
            "lb-single",
            vec![
                ListBoxItem::new("alpha", "Alpha"),
                ListBoxItem::new("beta", "Beta"),
                ListBoxItem::new("gamma", "Gamma"),
            ],
        )
        .on_selection_change(move |keys, _, _| events.borrow_mut().push(sorted_join(keys)))
        .into_any_element()
    });

    // `list_row_centre(0)` = 4 + 0 + 36/2 = y 22. The report carries the
    // row's *key* (the value v3 hands `onSelectionChange`), here "alpha".
    click(cx, 60., list_row_centre(0));
    assert_eq!(
        recorded.borrow().as_slice(),
        ["alpha"],
        "clicking the first row must select it"
    );

    // The click's mouse-down focused Alpha, so the arrows walk from that row.
    // Two Downs reach Gamma, which Enter then takes.
    press(cx, "down down");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["alpha", "gamma"],
        "Down Down Enter must move from Alpha to Gamma and choose it"
    );
}

#[gpui::test]
fn list_box_multiple_selection_accumulates(cx: &mut TestAppContext) {
    // ListBox is the *controlled* kind (like Select): `selected_keys` is a
    // prop and `on_selection_change` hands the full next set back, so the
    // caller owns the selection and feeds it to the next render. The test
    // plays that role, which is also what makes the third click's toggle
    // meaningful: the row can only remove Beta's pick if it was told Alpha and
    // Beta were selected.
    let selection: Rc<RefCell<HashSet<gpui::SharedString>>> = Rc::new(RefCell::new(HashSet::new()));
    let events = events();
    let recorded = events.clone();
    let cx = open_host(cx, move || {
        let selection = selection.clone();
        let events = events.clone();
        let held = selection.borrow().clone();
        ListBox::new(
            "lb-multi",
            vec![
                ListBoxItem::new("alpha", "Alpha"),
                ListBoxItem::new("beta", "Beta"),
                ListBoxItem::new("gamma", "Gamma"),
            ],
        )
        .selection_mode(SelectionMode::Multiple)
        .selected_keys(held)
        .on_selection_change(move |keys, _, _| {
            events.borrow_mut().push(sorted_join(keys));
            *selection.borrow_mut() = keys.clone();
        })
        .into_any_element()
    });

    // Rows 0 and 1 centre at y 22 and 62 (`list_row_centre`); reports carry
    // the row keys, and each one is the *whole* held selection.
    click(cx, 60., list_row_centre(0));
    assert_eq!(recorded.borrow().as_slice(), ["alpha"]);
    click(cx, 60., list_row_centre(1));
    assert_eq!(
        recorded.borrow().as_slice(),
        ["alpha", "alpha,beta"],
        "a second click must report the first pick and the second together"
    );
    click(cx, 60., list_row_centre(0));
    assert_eq!(
        recorded.borrow().as_slice(),
        ["alpha", "alpha,beta", "beta"],
        "re-clicking a picked row must toggle it off, which requires the \
         row to have been told it was already selected"
    );
}

#[gpui::test]
fn list_box_typeahead_moves_the_cursor(cx: &mut TestAppContext) {
    // The typeahead lives in `crate::list_nav`, which AGENTS.md describes as the
    // listbox/menu/select sharing one search. The cursor itself is internal, so
    // the assertion is behavioural: type a letter, then Enter must select the
    // row the letter found.
    let events = events();
    let recorded = events.clone();
    let cx = open_host(cx, move || {
        let events = events.clone();
        ListBox::new(
            "lb-typeahead",
            vec![
                ListBoxItem::new("rust", "Rust"),
                ListBoxItem::new("belgium", "Belgium"),
                ListBoxItem::new("brazil", "Brazil"),
                ListBoxItem::new("go", "Go"),
            ],
        )
        .on_selection_change(move |keys, _, _| events.borrow_mut().push(sorted_join(keys)))
        .into_any_element()
    });

    // Row 0 at `list_row_centre(0)`. Clicking also gives the list the focus
    // the keys need; the report carries the key "rust".
    click(cx, 60., list_row_centre(0));
    assert_eq!(recorded.borrow().as_slice(), ["rust"]);

    // "b" must search the *labels* and move the cursor to the first row
    // starting with b — Belgium is index 1, not the clicked row 0. Enter then
    // takes it, reporting the key "belgium".
    press(cx, "b");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["rust", "belgium"],
        "typing must move the keyboard to the matching row, which Enter then chooses"
    );
}

#[gpui::test]
fn list_box_disabled_key_cannot_be_chosen(cx: &mut TestAppContext) {
    let events = events();
    let recorded = events.clone();
    let cx = open_host(cx, move || {
        let events = events.clone();
        ListBox::new(
            "lb-disabled",
            vec![
                ListBoxItem::new("alpha", "Alpha"),
                ListBoxItem::new("beta", "Beta"),
                ListBoxItem::new("gamma", "Gamma"),
            ],
        )
        .disabled_keys(["beta".into()])
        .on_selection_change(move |keys, _, _| events.borrow_mut().push(sorted_join(keys)))
        .into_any_element()
    });

    // Row 1 (Beta, disabled) at `list_row_centre(1)`: nothing may record.
    click(cx, 60., list_row_centre(1));
    assert!(
        recorded.borrow().is_empty(),
        "clicking a disabledKeys row must record nothing"
    );

    // Row 0 at `list_row_centre(0)` selects Alpha and puts the list in the key
    // chain. From the top, the first Down lands the cursor on Alpha and the
    // second steps to the next *stop* — which skips disabled Beta and lands on
    // Gamma.
    click(cx, 60., list_row_centre(0));
    press(cx, "down down");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["alpha", "gamma"],
        "the arrows must skip a disabledKeys row: Enter after two Downs \
         chooses Gamma, never Beta"
    );
}

// ---------------------------------------------------------------------------
// TagGroup
// ---------------------------------------------------------------------------

#[gpui::test]
fn tag_group_remove_reports_the_key(cx: &mut TestAppContext) {
    let events = events();
    let removed = events.clone();
    let cx = open_host(cx, move || {
        let events = events.clone();
        // The label slot is drawn as a fixed 40x20 box (via `tag_content`, v3's
        // children-as-a-function) so every distance the click depends on is a
        // constant — see the header comment: a chip is 8+40+4+12+8 = 72px wide,
        // 28px tall, and its size-3 remove button spans x 52..64 at y 8..20.
        TagGroup::new(
            "tg-remove",
            vec![
                Tag::new("alpha", "Alpha"),
                Tag::new("beta", "Beta"),
                Tag::new("gamma", "Gamma"),
            ],
        )
        .tag_content(|_, _| gpui::div().w(px(40.)).h(px(20.)).into_any_element())
        .on_remove(move |keys, _, _| events.borrow_mut().push(sorted_join(keys)))
        .into_any_element()
    });

    // Chip 0's remove button centre: (58, 14); chip 1 starts at 72+6 = 78 and
    // its button centre is (58+78, 14) = (136, 14).
    click(cx, 58., 14.);
    assert_eq!(removed.borrow().as_slice(), ["alpha"]);
    click(cx, 136., 14.);
    assert_eq!(
        removed.borrow().as_slice(),
        ["alpha", "beta"],
        "each tag's own remove button must report that tag's key"
    );
}

#[gpui::test]
fn tag_group_roving_focus_survives_removal(cx: &mut TestAppContext) {
    // The group is ONE tab stop with a keyed cursor (AGENTS.md: "TagGroup keeps
    // its cursor in a keyed usize… clamp that cursor to the enabled tags —
    // Delete shortens the list, and a stop pointing past the end… takes the
    // group out of the tab order"). This test walks the cursor to the last tag,
    // deletes it, and then asks the group to keep answering the arrows: without
    // the clamp, nothing would claim the group's handle any more and the keys
    // would reach no tag at all.
    //
    // A TagGroup never removes tags itself — `on_remove` only reports the keys
    // and the *caller* drops it — so the test owns the list and shortens it
    // inside the handler, which is how the component is actually used.
    let tags = Rc::new(RefCell::new(vec![
        Tag::new("alpha", "Alpha"),
        Tag::new("beta", "Beta"),
        Tag::new("gamma", "Gamma"),
    ]));
    let events = events();
    let removed = events.clone();
    let cx = open_host(cx, move || {
        let tags = tags.clone();
        let events = events.clone();
        let current = tags.borrow().clone();
        TagGroup::new("tg-roving", current)
            .on_remove(move |keys, _, cx| {
                events.borrow_mut().push(sorted_join(keys));
                tags.borrow_mut().retain(|tag| !keys.contains(tag.key()));
                // A real app drops the tag from its own state here and asks for
                // a redraw; without the refresh the next keypress would arrive
                // against the frame that still shows the removed tag.
                cx.refresh_windows();
            })
            .into_any_element()
    });

    // Tab enters the group on the first tag; two Rights walk the cursor to the
    // last one (each render passes the handle to the tag the cursor moved to).
    press(cx, "tab");
    press(cx, "right right");

    // Delete removes the focused tag: the cursor was 2 (Gamma) and now points
    // past the end of the two remaining tags.
    press(cx, "delete");
    assert_eq!(removed.borrow().as_slice(), ["gamma"]);

    // The clamp has to have parked the roving stop back on the first enabled
    // tag; a stale cursor would leave the group outside the tab order and this
    // Right would reach nothing. The Right then Delete must take Beta.
    press(cx, "right");
    press(cx, "delete");
    assert_eq!(
        removed.borrow().as_slice(),
        ["gamma", "beta"],
        "after removing the last tag the group must still answer the arrows \
         and remove the newly focused tag"
    );
}

// ---------------------------------------------------------------------------
// Tabs
// ---------------------------------------------------------------------------

#[gpui::test]
fn tabs_uncontrolled_selection_changes(cx: &mut TestAppContext) {
    // The regression behind this test: `Tabs::new` used to seed the *controlled*
    // prop, `selected_key`, so `util::controlled` handed the caller's value back
    // with no state entity to write to — every demo that passed a literal was
    // inert while looking perfect. `Tabs::new`'s positional key is
    // `defaultSelectedKey`, so the tabs must switch themselves on press.
    let events = events();
    let selected = events.clone();
    let cx = open_host(cx, move || {
        let events = events.clone();
        Tabs::new(
            "tb-click",
            vec![
                TabItem::new("first", "First"),
                TabItem::new("second", "Second"),
            ],
            "first",
        )
        .on_selection_change(move |key, _, _| events.borrow_mut().push(key.to_string()))
        .into_any_element()
    });

    // The labels are measured with the window's own text system (see
    // `text_width`). Second tab centre: list p-1 (4) + the first tab's
    // label+2*16px padding, then half of the second tab's own label+2*16px.
    // Y: list p-1 top + h-8/2 = 4 + 16.
    let w_first =
        cx.update(|window, _| text_width(window.text_system(), "First", 14.0, FontWeight::MEDIUM));
    let w_second =
        cx.update(|window, _| text_width(window.text_system(), "Second", 14.0, FontWeight::MEDIUM));
    let tab1_centre_x = 4. + w_first + 32. + (w_second + 32.) / 2.;
    click(cx, tab1_centre_x, 20.);
    assert_eq!(
        selected.borrow().as_slice(),
        ["second"],
        "clicking the second tab must move the uncontrolled selection to it"
    );
}

#[gpui::test]
fn tabs_arrows_move_selection(cx: &mut TestAppContext) {
    // A tab list is one tab stop: the selected tab claims the group's handle,
    // and the arrows select as they rove (React Aria's automatic activation).
    let events = events();
    let selected = events.clone();
    let cx = open_host(cx, move || {
        let events = events.clone();
        Tabs::new(
            "tb-arrows",
            vec![
                TabItem::new("first", "First"),
                TabItem::new("second", "Second"),
                TabItem::new("third", "Third"),
            ],
            "first",
        )
        .on_selection_change(move |key, _, _| events.borrow_mut().push(key.to_string()))
        .into_any_element()
    });

    // Tab lands on the selected tab, whose handle the keyboard travels with as
    // the selection moves. Right walks first -> second -> third; a Left returns
    // to second. (The ends wrap, which is why a third Right is not tested.)
    press(cx, "tab");
    press(cx, "right");
    press(cx, "right");
    press(cx, "left");
    assert_eq!(
        selected.borrow().as_slice(),
        ["second", "third", "second"],
        "the arrows must move the selection: Right Right from the first tab \
         lands on third, Left brings it back to second"
    );
}

// ---------------------------------------------------------------------------
// Accordion
// ---------------------------------------------------------------------------

#[gpui::test]
fn accordion_toggles_one_item(cx: &mut TestAppContext) {
    let events = events();
    let expanded = events.clone();
    let cx = open_host(cx, move || {
        let events = events.clone();
        Accordion::new(vec![
            AccordionItem::new("one", "Item one"),
            AccordionItem::new("two", "Item two"),
        ])
        .id("acc-one")
        .on_expanded_change(move |keys, _, _| events.borrow_mut().push(sorted_join(keys)))
        .into_any_element()
    });

    // Header 0 is 52px tall starting at the origin (py-4 twice + a 20px line),
    // so its centre is y 26; the header spans the full window width.
    click(cx, 60., 26.);
    assert_eq!(
        expanded.borrow().as_slice(),
        ["one"],
        "clicking a closed trigger must expand that item"
    );
    click(cx, 60., 26.);
    assert_eq!(
        expanded.borrow().as_slice(),
        ["one", ""],
        "clicking the same trigger again must collapse it"
    );
}

#[gpui::test]
fn accordion_single_mode_closes_the_previous(cx: &mut TestAppContext) {
    let events = events();
    let expanded = events.clone();
    let cx = open_host(cx, move || {
        let events = events.clone();
        // `allows_multiple_expanded(false)` is v3's single-expand behaviour:
        // expanding one item collapses the rest (`next_expanded` in
        // accordion.rs). The bodies are given a fixed 40px content so the
        // second header's position is a constant while the first is open.
        Accordion::new(vec![
            AccordionItem::new("one", "Item one").content(gpui::div().h(px(40.))),
            AccordionItem::new("two", "Item two").content(gpui::div().h(px(40.))),
        ])
        .id("acc-single")
        .allows_multiple_expanded(false)
        .on_expanded_change(move |keys, _, _| events.borrow_mut().push(sorted_join(keys)))
        .into_any_element()
    });

    // Header 0 centre: y 26. With item one open, item two's header sits below
    // 52 (header) + 2+40+16 (body) + 1 (separator) + 26 (half header) = 137.
    click(cx, 60., 26.);
    assert_eq!(expanded.borrow().as_slice(), ["one"]);
    click(cx, 60., 137.);
    assert_eq!(
        expanded.borrow().as_slice(),
        ["one", "two"],
        "with allowsMultipleExpanded off, opening the second item must report \
         exactly that one — never both"
    );
}

// ---------------------------------------------------------------------------
// Pagination
// ---------------------------------------------------------------------------

#[gpui::test]
fn pagination_next_and_page_click(cx: &mut TestAppContext) {
    let events = events();
    let pages = events.clone();
    let cx = open_host(cx, move || {
        let events = events.clone();
        Pagination::new("pg-nav", 1, 3)
            .on_change(move |page, _, _| events.borrow_mut().push(page.to_string()))
            .into_any_element()
    });

    // size-md cells are 32px squares at y 0..32 (centre y 16). The prev button
    // spans x 0..34 (px-2.5 each side of a 14px glyph); with 4px gaps the page
    // cells start at x 38, 74 and 110, so page 3's centre is x 126, and the
    // next button starts at x 146 (centre 163). All maths in the header comment.
    click(cx, 163., 16.);
    assert_eq!(
        pages.borrow().as_slice(),
        ["2"],
        "clicking the next arrow must report page 2"
    );
    click(cx, 126., 16.);
    assert_eq!(
        pages.borrow().as_slice(),
        ["2", "3"],
        "clicking the third page cell must report page 3"
    );
}

// ---------------------------------------------------------------------------
// Breadcrumbs
// ---------------------------------------------------------------------------

#[gpui::test]
fn breadcrumbs_item_press_reports(cx: &mut TestAppContext) {
    let events = events();
    let navigated = events.clone();
    let cx = open_host(cx, move || {
        let events = events.clone();
        // The last crumb is the current page and never navigates (no `on_click`
        // is attached when `is_last`), so the clickable targets are the first
        // and the middle crumb.
        Breadcrumbs::new(vec![
            Crumb::new("Build"),
            Crumb::new("Deploy"),
            Crumb::new("Live"),
        ])
        .on_navigate(move |index, crumb, _, _, _| {
            events.borrow_mut().push(format!("{index}:{}", crumb.label));
        })
        .into_any_element()
    });

    // The labels are measured with the window's own text system at the link's
    // MEDIUM weight. The first label starts 4px into its row (2px row padding
    // + 2px link padding) on a 20px line, so its centre is (4 + w/2, 10). Each
    // non-last row is 2px paddings + 2px gap + a 12px separator wider than its
    // label, so the second label centres at (w_build + 26 + w_deploy/2, 10).
    let w_build =
        cx.update(|window, _| text_width(window.text_system(), "Build", 14.0, FontWeight::MEDIUM));
    let w_deploy =
        cx.update(|window, _| text_width(window.text_system(), "Deploy", 14.0, FontWeight::MEDIUM));
    click(cx, 4. + w_build / 2., 10.);
    assert_eq!(
        navigated.borrow().as_slice(),
        ["0:Build"],
        "clicking the first crumb must report its index and label"
    );
    click(cx, w_build + 26. + w_deploy / 2., 10.);
    assert_eq!(
        navigated.borrow().as_slice(),
        ["0:Build", "1:Deploy"],
        "clicking the second crumb must report its own index and label"
    );
}
