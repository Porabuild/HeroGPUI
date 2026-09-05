//! Tag line boxes must match v3.2.4 even outside the gallery's 14/20 text root.

mod harness;

use gpui::{prelude::*, px, TestAppContext};
use herogpui_components::{Size, Tag, TagGroup};

const SELECTORS: [&str; 3] = ["tag-sm", "tag-md", "tag-lg"];

#[gpui::test]
fn tag_sizes_do_not_inherit_host_line_height(cx: &mut TestAppContext) {
    for leading in [None, Some(48.)] {
        let cx = harness::open_host(cx, move || {
            let mut root = gpui::div().flex().flex_col().items_start();
            if let Some(leading) = leading {
                root = root.text_size(px(32.)).line_height(px(leading));
            }
            root.children(Size::ALL.into_iter().enumerate().map(|(index, size)| {
                gpui::div()
                    .debug_selector(move || SELECTORS[index].to_owned())
                    .child(
                        TagGroup::new(format!("tag-{index}"), vec![Tag::new("one", "Tag")])
                            .size(size),
                    )
            }))
            .into_any_element()
        });
        cx.run_until_parked();
        for (index, height) in [20., 24., 32.].into_iter().enumerate() {
            let bounds = cx.debug_bounds(SELECTORS[index]).unwrap();
            assert!(
                (f32::from(bounds.size.height) - height).abs() < 0.5,
                "size {index}, host leading {leading:?}: expected {height}px, got {bounds:?}"
            );
        }
    }
}

#[gpui::test]
fn tag_label_description_and_empty_state_own_their_line_boxes(cx: &mut TestAppContext) {
    let cx = harness::open_host(cx, || {
        gpui::div()
            .flex()
            .flex_col()
            .items_start()
            .text_size(px(32.))
            .line_height(px(48.))
            .child(
                gpui::div()
                    .debug_selector(|| "tag-labelled".to_owned())
                    .child(
                        TagGroup::new("labelled", vec![Tag::new("one", "Tag")])
                            .label("Label")
                            .description("Description"),
                    ),
            )
            .child(
                gpui::div()
                    .debug_selector(|| "tag-empty".to_owned())
                    .child(TagGroup::new("empty", vec![]).label("Label")),
            )
            .into_any_element()
    });
    cx.run_until_parked();
    // Label 20 + gaps 8 + medium tag 24 + description 16 with p-1 = 76.
    let labelled = cx.debug_bounds("tag-labelled").unwrap();
    assert_eq!(labelled.size.height, px(76.));
    // Label 20 + gap 4 + empty-state 20 with p-2 = 60.
    let empty = cx.debug_bounds("tag-empty").unwrap();
    assert_eq!(empty.size.height, px(60.));
}
