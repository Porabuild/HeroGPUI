//! Buttons gallery pages.
#![allow(clippy::redundant_clone)]

use super::*;

impl Gallery {
    // ---------------------------------------------------------------------------
    // Buttons
    // ---------------------------------------------------------------------------

    pub fn page_button(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let clicks = self.button_clicks;
        let pending_foreground = cx.colors().accent.foreground;
        component_doc_page!(
            "Button",
            crate::pages::Page::Button.description(),
            crate::pages::Page::Button.import_line(),
            vec![
                (
                    "Usage",
                    "`radius` sets the corners; `sx` squares the top-left corner, including while pressed.",
                    specimen_body("btn-usage", row(vec![h::Button::new("btn-usage")
                        .radius(px(8.))
                        .sx(|el| el.rounded_tl(px(0.)))
                        .label("Click me")
                        .into_any_element()]), cx),
                ),
                (
                    "Variants",
                    row(Variant::ALL
                        .iter()
                        .filter_map(|v| {
                            let key = format!("btn-v-{v:?}");
                            crate::control::specimen_wanted(&key, cx).then(|| {
                                h::Button::new(el_id(key))
                                    .label(v.label())
                                    .variant(*v)
                            })
                        })
                        .els()),
                ),
                (
                    "Sizes",
                    row(Size::ALL
                        .iter()
                        .filter_map(|s| {
                            let key = format!("btn-s-{s:?}");
                            crate::control::specimen_wanted(&key, cx).then(|| {
                                h::Button::new(el_id(key))
                                    .label(s.label())
                                    .size(*s)
                            })
                        })
                        .els()),
                ),
                (
                    "With Icons",
                    row(vec![
                        h::Button::new("btn-i-1")
                            .child(icon(h::icons::GLOBE, cx))
                            .child("Search")
                            .into_any_element(),
                        h::Button::new("btn-i-2")
                            .variant(Variant::Secondary)
                            .child(icon(h::icons::PLUS, cx))
                            .child("Add Member")
                            .into_any_element(),
                        h::Button::new("btn-i-3")
                            .variant(Variant::Tertiary)
                            .child(icon(h::icons::MAIL, cx))
                            .child("Email")
                            .into_any_element(),
                        h::Button::new("btn-i-4")
                            .variant(Variant::Danger)
                            .child(icon(h::icons::TRASH, cx))
                            .child("Delete")
                            .into_any_element(),
                    ]),
                ),
                (
                    "Icon Only",
                    row(vec![
                        h::Button::new("btn-io-1")
                            .is_icon_only(true)
                            .variant(Variant::Tertiary)
                            .child(icon(h::icons::ELLIPSIS, cx))
                            .into_any_element(),
                        h::Button::new("btn-io-2")
                            .is_icon_only(true)
                            .variant(Variant::Secondary)
                            .child(icon(h::icons::GEAR, cx))
                            .into_any_element(),
                        h::Button::new("btn-io-3")
                            .is_icon_only(true)
                            .variant(Variant::Danger)
                            .child(icon(h::icons::TRASH, cx))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Loading", "A static pending button: the spinner replaces the label's leading slot while `is_pending` holds, and the caller decides when the work is done.",
                    specimen_body("btn-loading", row(vec![h::Button::new("btn-loading")
                        .is_pending(true)
                        .content(move |state| {
                            gpui::div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .when(state.is_pending, |content| {
                                    content.child(
                                        h::Spinner::new("btn-loading-spinner")
                                            .size(h::SpinnerSize::Sm)
                                            .current_color(pending_foreground),
                                    )
                                })
                                .child("Uploading...")
                                .into_any_element()
                        })
                        .into_any_element()]), cx),
                ),
                (
                    "Loading State", "The caller owns the pending window: pressing starts an upload and the state resets itself after two seconds.",
                    specimen_body("btn-loading-state", row(vec![h::Button::new("btn-loading-state")
                        .is_pending(self.button_upload_pending)
                        .content(move |state| {
                            gpui::div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .when(state.is_pending, |content| {
                                    content.child(
                                        h::Spinner::new("btn-loading-state-spinner")
                                            .size(h::SpinnerSize::Sm)
                                            .current_color(pending_foreground),
                                    )
                                })
                                .child(if state.is_pending {
                                    "Uploading..."
                                } else {
                                    "Upload File"
                                })
                                .into_any_element()
                        })
                        .on_press({
                            let gallery = cx.entity().downgrade();
                            move |_, window, cx: &mut gpui::App| {
                                if let Some(gallery) = gallery.upgrade() {
                                    gallery.update(cx, |gallery, cx| {
                                        gallery.button_upload_pending = true;
                                        cx.notify();
                                    });
                                }
                                let gallery = gallery.clone();
                                window
                                    .spawn(cx, async move |cx| {
                                        cx.background_executor()
                                            .timer(std::time::Duration::from_millis(2000))
                                            .await;
                                        cx.update(|_, cx| {
                                            if let Some(gallery) = gallery.upgrade() {
                                                gallery.update(cx, |gallery, cx| {
                                                    gallery.button_upload_pending = false;
                                                    cx.notify();
                                                });
                                            }
                                        })
                                        .ok();
                                    })
                                    .detach();
                            }
                        })
                        .into_any_element()]), cx),
                ),
                (
                    "Full Width",
                    // v3 stretches two `fullWidth` buttons inside a 400px
                    // column; `full_width` resolves against its container, so
                    // the container is what has to be definite.
                    col(vec![gpui::div()
                        .flex()
                        .flex_col()
                        .w(px(400.))
                        .gap(px(12.))
                        .child(
                            h::Button::new("btn-full")
                                .label("Primary Button")
                                .full_width(true),
                        )
                        .child(
                            h::Button::new("btn-full-icon")
                                .full_width(true)
                                .child(icon(h::icons::PLUS, cx))
                                .child("With Icon"),
                        )
                        .into_any_element()]),
                ),
                (
                    "Pixel Geometry", "`width`, `min_width`, `height` and `padding_x` set the box in place of the size ladder. An explicit pixel `width` beats `full_width`, and the pressed skin keeps the fixed box.",
                    specimen_body("btn-pixel-geometry", row(vec![gpui::div()
                        .flex()
                        .items_start()
                        .gap(px(12.))
                        .child(
                            h::Button::new("btn-geom-sized")
                                .label("200 x 56")
                                .width(px(200.))
                                .height(px(56.)),
                        )
                        .child(
                            h::Button::new("btn-geom-roomy")
                                .label("Roomy")
                                .padding_x(px(32.)),
                        )
                        .child(
                            h::Button::new("btn-geom-min")
                                .label("Min 160")
                                .min_width(px(160.)),
                        )
                        .into_any_element()]), cx),
                ),
                (
                    "Text Metrics", "`text_size` and `font_weight` restyle the label in place of the size step's `text-sm font-medium`; a Tailwind step keeps its paired leading.",
                    specimen_body("btn-text-metrics", row(vec![
                        h::Button::new("btn-metrics-large")
                            .label("18px SemiBold")
                            .text_size(px(18.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .into_any_element(),
                        h::Button::new("btn-metrics-small")
                            .label("12px medium")
                            .text_size(px(12.))
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Flex Fill", "`grow` is v3's `flex-1` plus `min-w-0`: the growing buttons share the row's free width and compress below their content width when the row is tight, while the fixed one keeps its own.",
                    specimen_body("btn-flex-fill", row(vec![gpui::div()
                        .flex()
                        .w(px(400.))
                        .gap(px(8.))
                        .child(h::Button::new("btn-grow-fill").label("Fill").grow(true))
                        .child(h::Button::new("btn-grow-share").label("Shares too").grow(true))
                        .child(h::Button::new("btn-grow-fixed").label("Fixed"))
                        .into_any_element()]), cx),
                ),
                (
                    "Disabled State",
                    row(Variant::ALL
                        .iter()
                        .filter_map(|v| {
                            let key = format!("btn-d-{v:?}");
                            crate::control::specimen_wanted(&key, cx).then(|| {
                                h::Button::new(el_id(key))
                                    .label(v.label())
                                    .variant(*v)
                                    .is_disabled(true)
                            })
                        })
                        .els()),
                ),
                (
                    "Social Buttons", "Full-width tertiary buttons behind a leading brand mark. The marks are trademarks, so this example shows the same layout with its own glyphs.",
                    col(vec![
                        gpui::div()
                            .flex()
                            .flex_col()
                            .w(px(280.))
                            .gap(px(12.))
                            .child(
                                h::Button::new("btn-soc-1")
                                    .variant(Variant::Tertiary)
                                    .full_width(true)
                                    .child(icon(h::icons::MAIL, cx))
                                    .child("Sign in with Email"),
                            )
                            .child(
                                h::Button::new("btn-soc-2")
                                    .variant(Variant::Tertiary)
                                    .full_width(true)
                                    .child(icon(h::icons::KEY, cx))
                                    .child("Sign in with a passkey"),
                            )
                            .child(
                                h::Button::new("btn-soc-3")
                                    .variant(Variant::Tertiary)
                                    .full_width(true)
                                    .child(icon(h::icons::GLOBE, cx))
                                    .child("Single sign-on"),
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "Adding custom variants", "Fix a preset in place: a constructor layers its own radius and emphasis on top of the base, and callers keep the plain `Button` API.",
                    row({
                        let custom_button = |id: &'static str, label: &'static str| {
                            h::Button::new(id)
                                .label(label)
                                .variant(Variant::Secondary)
                                .size(Size::Lg)
                                .sx(|el| el.rounded_full())
                        };
                        vec![
                            custom_button("btn-custom-primary", "Custom Button")
                                .into_any_element(),
                            custom_button("btn-custom-secondary", "Another Preset")
                                .into_any_element(),
                        ]
                    }),
                ),
                (
                    "Press handler",
                    col(vec![h::Button::new("btn-press")
                        .label(format!("Pressed {clicks} times"))
                        .on_press(cx.listener(|this, _, _, cx| {
                            this.button_clicks += 1;
                            cx.notify();
                        }))
                        .into_any_element()]),
                ),
                (
                    "Sx slot", "Every builder carries one `sx` slot for caller-owned low-level styling: the closure styles the component's root with GPUI's own methods and is applied after every variant and theme value, so it wins.",
                    row(vec![
                        h::Button::new("btn-sx-sized")
                            .label("200x56")
                            // The slot owns the footprint: an enlargement, not
                            // a shrink, so the label's own content size never
                            // overflows the overridden box.
                            .sx(|el| el.w(px(200.)).h(px(56.)))
                            .into_any_element(),
                        h::Button::new("btn-sx-tinted")
                            .label("Orange")
                            .sx(|el| {
                                el.bg(gpui::rgba(0xffa500ff))
                                    .text_color(gpui::rgba(0x000000ff))
                            })
                            .into_any_element(),
                    ]),
                ),
                (
                    "Custom hover fill", "An `sx` background alone holds through hover, because the fade would otherwise ease the variant colour back over the override. `hover_bg` names the other end instead, so a caller-owned surface keeps the `transition-colors` fade.",
                    row(vec![
                        h::Button::new("btn-hover-bg-sx")
                            .label("Tinted hover")
                            .sx(|el| {
                                el.bg(gpui::rgba(0xffa500ff))
                                    .text_color(gpui::rgba(0x000000ff))
                            })
                            .hover_bg(gpui::rgba(0xcc7000ff))
                            .into_any_element(),
                        h::Button::new("btn-hover-bg-variant")
                            .label("Variant resting fill")
                            .variant(Variant::Secondary)
                            // No `sx`: the fade still rests on the variant's
                            // own colour and only the hover end is replaced.
                            .hover_bg(gpui::rgba(0x7828c8ff))
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_button_group(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Button Group",
            crate::pages::Page::ButtonGroup.description(),
            crate::pages::Page::ButtonGroup.import_line(),
            vec![
                (
                    "Usage",
                    // v3's Button paints `text-*` on its members and svgs
                    // inherit `currentColor`, so the icon-only chevron reads
                    // the primary member's own foreground. gpui svgs never
                    // inherit, so the demo spells it out.
                    specimen_body(
                        "bgroup-usage",
                        row(vec![h::ButtonGroup::new()
                            .separators(true)
                            .button(h::Button::new("bgu-1").label("Merge pull request"))
                            .button(
                                h::Button::new("bgu-2").is_icon_only(true).child(
                                    gpui::svg()
                                        .size(px(16.))
                                        .path(h::icons::CHEVRON_DOWN)
                                        .text_color(cx.colors().accent.foreground),
                                ),
                            )
                            .into_any_element()]),
                        cx
                    ),
                ),
                (
                    "Merged",
                    specimen_body(
                        "bgroup-merged",
                        row(vec![h::ButtonGroup::new()
                            .variant(Variant::Secondary)
                            .separators(true)
                            .button(h::Button::new("bg-1").label("Day"))
                            .button(h::Button::new("bg-2").label("Week"))
                            .button(h::Button::new("bg-3").label("Month"))
                            .into_any_element()]),
                        cx
                    ),
                ),
                (
                    "Sizes",
                    col(Size::ALL
                        .iter()
                        .filter_map(|sz| {
                            let key = format!("bgroup-size-{sz:?}");
                            crate::control::specimen_wanted(&key, cx).then(|| {
                                h::ButtonGroup::new()
                                    .variant(Variant::Secondary)
                                    .size(*sz)
                                    .separators(true)
                                    .button(
                                        h::Button::new(el_id(format!("bgs-{sz:?}-1")))
                                            .label("Left"),
                                    )
                                    .button(
                                        h::Button::new(el_id(format!("bgs-{sz:?}-2")))
                                            .label("Middle"),
                                    )
                                    .button(
                                        h::Button::new(el_id(format!("bgs-{sz:?}-3")))
                                            .label("Right"),
                                    )
                            })
                        })
                        .els()),
                ),
                (
                    "With Icons",
                    specimen_body(
                        "bgroup-icons",
                        row(vec![h::ButtonGroup::new()
                            .variant(Variant::Tertiary)
                            .separators(true)
                            .button(
                                h::Button::new("bgi-1")
                                    .child(icon(h::icons::COPY, cx))
                                    .child("Fork"),
                            )
                            .button(
                                h::Button::new("bgi-2")
                                    .child(icon(h::icons::PLUS, cx))
                                    .child("Star"),
                            )
                            .button(
                                h::Button::new("bgi-3")
                                    .is_icon_only(true)
                                    .child(icon(h::icons::ELLIPSIS, cx)),
                            )
                            .into_any_element()]),
                        cx
                    ),
                ),
                (
                    "Variants",
                    col(Variant::GROUP
                        .iter()
                        .filter_map(|v| {
                            let key = format!("bgroup-v-{v:?}");
                            crate::control::specimen_wanted(&key, cx).then(|| {
                                h::ButtonGroup::new()
                                    .variant(*v)
                                    .separators(true)
                                    .button(
                                        h::Button::new(el_id(format!("bgv-{v:?}-1"))).label("One"),
                                    )
                                    .button(
                                        h::Button::new(el_id(format!("bgv-{v:?}-2"))).label("Two"),
                                    )
                            })
                        })
                        .els()),
                ),
                (
                    "Orientation",
                    row(vec![
                        specimen_body(
                            "bgroup-orientation-horizontal",
                            h::ButtonGroup::new()
                                .variant(Variant::Secondary)
                                .separators(true)
                                .button(h::Button::new("bgo-l").label("Left"))
                                .button(h::Button::new("bgo-c").label("Center"))
                                .button(h::Button::new("bgo-r").label("Right"))
                                .into_any_element(),
                            cx
                        ),
                        specimen_body(
                            "bgroup-orientation-vertical",
                            h::ButtonGroup::new()
                                .variant(Variant::Secondary)
                                .orientation(Orientation::Vertical)
                                .separators(true)
                                .button(h::Button::new("bgv-top").label("Top"))
                                .button(h::Button::new("bgv-mid").label("Middle"))
                                .button(h::Button::new("bgv-bot").label("Bottom"))
                                .into_any_element(),
                            cx
                        ),
                    ]),
                ),
                (
                    "Full Width",
                    specimen_body(
                        "bgroup-full-width",
                        col(vec![gpui::div()
                            .w_full()
                            .child(
                                h::ButtonGroup::new()
                                    .variant(Variant::Secondary)
                                    .full_width(true)
                                    .separators(true)
                                    .button(h::Button::new("bgf-1").label("Cancel"))
                                    .button(h::Button::new("bgf-2").label("Save draft"))
                                    .button(h::Button::new("bgf-3").label("Publish")),
                            )
                            .into_any_element()]),
                        cx
                    ),
                ),
                (
                    "Without Separator",
                    specimen_body(
                        "bgroup-without-separator",
                        row(vec![h::ButtonGroup::new()
                        .variant(Variant::Secondary)
                        // v3: omit the `<ButtonGroup.Separator />` child
                        // composition — the port's default draws no dividers.
                        .button(h::Button::new("bgn-1").label("One"))
                        .button(h::Button::new("bgn-2").label("Two"))
                        .button(h::Button::new("bgn-3").label("Three"))
                        .into_any_element()]),
                        cx
                    ),
                ),
                (
                    "Disabled State",
                    specimen_body(
                        "bgroup-disabled",
                        row(vec![h::ButtonGroup::new()
                            .variant(Variant::Secondary)
                            .is_disabled(true)
                            .separators(true)
                            .button(h::Button::new("bgd2-1").label("One"))
                            .button(h::Button::new("bgd2-2").label("Two"))
                            .into_any_element()]),
                        cx
                    ),
                ),
            ],
            cx,
        )
    }

    pub fn page_close_button(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let presses = self.close_button_presses;
        let accent = cx.colors().accent.color;
        component_doc_page!(
            "Close Button",
            crate::pages::Page::CloseButton.description(),
            crate::pages::Page::CloseButton.import_line(),
            vec![
                (
                    "Usage", "`radius` sets the corners; `sx` squares the top-left corner, including while pressed.",
                    specimen_body("close-usage", row(vec![h::CloseButton::new("cb-usage")
                        .radius(px(4.))
                        .sx(|el| el.rounded_tl(px(0.)))
                        .into_any_element()]), cx),
                ),
                (
                    "Interactive",
                    specimen_body("close-interactive", col(vec![
                        h::CloseButton::new("cb-press")
                            .on_press(cx.listener(|this, _, _, cx| {
                                this.close_button_presses += 1;
                                cx.notify();
                            }))
                            .into_any_element(),
                        para(&format!("Pressed {presses} times"), cx),
                    ]), cx),
                ),
                (
                    "With Custom Icon",
                    specimen_body("close-custom-icon", row(vec![spec(
                        "Custom icon",
                        h::CloseButton::new("cb-icon-1").icon(icon(h::icons::CLOSE_CIRCLE, cx)),
                        cx,
                    )]), cx),
                ),
                (
                    "Hover Colour",
                    "`hover_bg` names the hover fill; it eases from the resting `--default` (or the `sx` background when one is set), the same contract as `Button`.",
                    specimen_body("close-hover-bg", row(vec![h::CloseButton::new("cb-hover-bg")
                        .hover_bg(accent)
                        .into_any_element()]), cx),
                ),
                (
                    "Render Function", "Hover, focus, or press the button to drive the custom icon from its live render state.",
                    specimen_body("close-render-state", {
                        let muted = cx.colors().muted;
                        let foreground = cx.colors().foreground;
                        col(vec![h::CloseButton::new("cb-render-state")
                            .content(move |state| {
                                gpui::svg()
                                    .size(px(16.))
                                    .path(if state.is_pressed {
                                        h::icons::CLOSE_CIRCLE
                                    } else {
                                        h::icons::CLOSE
                                    })
                                    .text_color(if state.is_disabled {
                                        muted
                                    } else if state.is_hovered || state.is_focused {
                                        foreground
                                    } else {
                                        muted
                                    })
                                    .into_any_element()
                            })
                            .into_any_element()])
                    }, cx)
                ),
                (
                    "Disabled",
                    specimen_body("close-disabled", row(vec![h::CloseButton::new("cb-disabled")
                        .is_disabled(true)
                        .into_any_element()]), cx),
                ),
            ],
            cx,
        )
    }

    pub fn page_toggle_button(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let liked = self.toggle_like;
        let single = self.toggle_single.clone();
        let multiple = self.toggle_multiple.clone();
        let accent = cx.colors().accent.color;
        component_doc_page!(
            "Toggle Button",
            crate::pages::Page::ToggleButton.description(),
            crate::pages::Page::ToggleButton.import_line(),
            vec![
                (
                    "Usage",
                    "`radius` sets the corners; `sx` squares the top-left corner, including while pressed.",
                    specimen_body("toggle-usage", row(vec![h::ToggleButton::new("tb-usage")
                        .radius(px(8.))
                        .sx(|el| el.rounded_tl(px(0.)))
                        .label("Bold")
                        .into_any_element()]), cx),
                ),
                (
                    "Hover Colour",
                    "`hover_bg` follows the Button resting-endpoint contract: the fade runs from the resting background to the named hover fill.",
                    specimen_body("toggle-hover-bg", row(vec![h::ToggleButton::new("tb-hover-bg")
                        .label("Hover")
                        .hover_bg(accent)
                        .into_any_element()]), cx),
                ),
                (
                    "Sizes",
                    row(Size::ALL
                        .iter()
                        .filter_map(|sz| {
                            let key = format!("toggle-size-{sz:?}");
                            crate::control::specimen_wanted(&key, cx).then(|| {
                                h::ToggleButton::new(el_id(format!("tb-sz-{sz:?}")))
                                    .label(sz.label())
                                    .size(*sz)
                                    .default_selected(true)
                            })
                        })
                        .els()),
                ),
                (
                    "Icon Only",
                    specimen_body("toggle-icon-only", row(vec![
                        h::ToggleButton::new("tb-io-1")
                            .is_icon_only(true)
                            .default_selected(true)
                            .child(icon(h::icons::EYE, cx))
                            .into_any_element(),
                        h::ToggleButton::new("tb-io-2")
                            .is_icon_only(true)
                            .child(icon(h::icons::COPY, cx))
                            .into_any_element(),
                        h::ToggleButton::new("tb-io-3")
                            .is_icon_only(true)
                            .child(icon(h::icons::SEARCH, cx))
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Disabled",
                    specimen_body("toggle-disabled", row(vec![
                        h::ToggleButton::new("tb-dis-1")
                            .label("Off")
                            .is_disabled(true)
                            .into_any_element(),
                        h::ToggleButton::new("tb-dis-2")
                            .label("On")
                            .is_selected(true)
                            .is_disabled(true)
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Controlled",
                    specimen_body("toggle-controlled", col(vec![
                        h::ToggleButton::new("tb-like")
                            .label(if liked { "Liked" } else { "Like" })
                            .is_selected(liked)
                            .child(icon(
                                if liked {
                                    h::icons::HEART_FILL
                                } else {
                                    h::icons::HEART
                                },
                                cx,
                            ))
                            .on_change(cx.listener(|this, v: &bool, _, cx| {
                                this.toggle_like = *v;
                                cx.notify();
                            }))
                            .into_any_element(),
                        para(
                            if liked {
                                "Status: selected"
                            } else {
                                "Status: not selected"
                            },
                            cx,
                        ),
                    ]), cx),
                ),
                (
                    "Single selection",
                    specimen_body("toggle-single-selection", row(vec![h::ToggleButtonGroup::new("toggle-single")
                        .selection_mode(SelectionMode::Single)
                        .separators(true)
                        .selected_keys(single.into_iter().collect::<Vec<_>>())
                        .child_toggle(h::ToggleButton::new("tb-left").key("left").label("Left"))
                        .child_toggle(
                            h::ToggleButton::new("tb-center")
                                .key("center")
                                .label("Center"),
                        )
                        .child_toggle(h::ToggleButton::new("tb-right").key("right").label("Right"))
                        .on_selection_change(cx.listener(|this, keys: &[SharedString], _, cx| {
                            this.toggle_single = keys.first().cloned();
                            cx.notify();
                        }))
                        .into_any_element()]), cx),
                ),
                (
                    "Multiple selection",
                    specimen_body("toggle-multiple-selection", row(vec![h::ToggleButtonGroup::new("toggle-multiple")
                        .selection_mode(SelectionMode::Multiple)
                        .separators(true)
                        .selected_keys(multiple.iter().cloned().collect::<Vec<_>>())
                        .child_toggle(h::ToggleButton::new("tb-bold").key("bold").label("Bold"))
                        .child_toggle(
                            h::ToggleButton::new("tb-italic")
                                .key("italic")
                                .label("Italic"),
                        )
                        .child_toggle(
                            h::ToggleButton::new("tb-underline")
                                .key("underline")
                                .label("Underline"),
                        )
                        .on_change(cx.listener(|this, keys: &[SharedString], _, cx| {
                            this.toggle_multiple = keys.iter().cloned().collect();
                            cx.notify();
                        }))
                        .into_any_element()]), cx),
                ),
                (
                    "Variants",
                    specimen_body("toggle-variants", row(vec![
                        h::ToggleButton::new("tb-v-default")
                            .label("Default")
                            .default_selected(true)
                            .into_any_element(),
                        h::ToggleButton::new("tb-v-ghost")
                            .label("Ghost")
                            .variant(h::ToggleVariant::Ghost)
                            .default_selected(true)
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Orientation",
                    row(vec![specimen_body("toggle-orientation-horizontal", h::ToggleButtonGroup::new("toggle-orientation-horizontal")
                            .separators(true)
                            .child_toggle(h::ToggleButton::new("tbo-h-1").label("Day"))
                            .child_toggle(h::ToggleButton::new("tbo-h-2").label("Week"))
                            .child_toggle(h::ToggleButton::new("tbo-h-3").label("Month"))
                            .into_any_element(), cx),
                        specimen_body("toggle-orientation-vertical", h::ToggleButtonGroup::new("toggle-orientation-vertical")
                            .orientation(Orientation::Vertical)
                            .separators(true)
                            .child_toggle(h::ToggleButton::new("tbo-v-1").label("Day"))
                            .child_toggle(h::ToggleButton::new("tbo-v-2").label("Week"))
                            .child_toggle(h::ToggleButton::new("tbo-v-3").label("Month"))
                            .into_any_element(), cx),
                    ]),
                ),
                (
                    "Full Width",
                    specimen_body("toggle-full-width", col(vec![gpui::div()
                        .w_full()
                        .child(
                            h::ToggleButtonGroup::new("toggle-full-width")
                                .full_width(true)
                                .separators(true)
                                .child_toggle(h::ToggleButton::new("tbf-1").label("Left"))
                                .child_toggle(h::ToggleButton::new("tbf-2").label("Center"))
                                .child_toggle(h::ToggleButton::new("tbf-3").label("Right")),
                        )
                        .into_any_element()]), cx),
                ),
                (
                    "Without Separator",
                    specimen_body("toggle-without-separator", row(vec![h::ToggleButtonGroup::new("toggle-without-separator")
                        // v3: omit the `<ToggleButtonGroup.Separator />` child
                        // composition — the port's default draws no dividers.
                        .child_toggle(h::ToggleButton::new("tbn-1").label("One"))
                        .child_toggle(h::ToggleButton::new("tbn-2").label("Two"))
                        .child_toggle(h::ToggleButton::new("tbn-3").label("Three"))
                        .into_any_element()]), cx),
                ),
                (
                    "Selection Mode", "Single: exactly one member stays selected.",
                    specimen_body("toggle-selection-mode", col(vec![
                        h::ToggleButtonGroup::new("toggle-selection-single")
                            .selection_mode(SelectionMode::Single)
                            .separators(true)
                            .child_toggle(h::ToggleButton::new("tbsm-s-1").key("a").label("A"))
                            .child_toggle(h::ToggleButton::new("tbsm-s-2").key("b").label("B"))
                            .child_toggle(h::ToggleButton::new("tbsm-s-3").key("c").label("C"))
                            .into_any_element(),
                        para("Multiple: any number of members can be selected.", cx),
                        h::ToggleButtonGroup::new("toggle-selection-multiple")
                            .selection_mode(SelectionMode::Multiple)
                            .separators(true)
                            .child_toggle(h::ToggleButton::new("tbsm-m-1").key("a").label("A"))
                            .child_toggle(h::ToggleButton::new("tbsm-m-2").key("b").label("B"))
                            .child_toggle(h::ToggleButton::new("tbsm-m-3").key("c").label("C"))
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Default Selected Keys", "Uncontrolled: `defaultSelectedKeys` seeds the group's own selection, and the group keeps ownership from there — clicking a member still toggles it.",
                    specimen_body("toggle-default-selected-keys", col(vec![
                        h::ToggleButtonGroup::new("toggle-default-single")
                            .selection_mode(SelectionMode::Single)
                            .separators(true)
                            .default_selected_keys(["center"])
                            .child_toggle(
                                h::ToggleButton::new("tbu-s-left").key("left").label("Left")
                            )
                            .child_toggle(
                                h::ToggleButton::new("tbu-s-center")
                                    .key("center")
                                    .label("Center"),
                            )
                            .child_toggle(
                                h::ToggleButton::new("tbu-s-right")
                                    .key("right")
                                    .label("Right"),
                            )
                            .into_any_element(),
                        para("Multiple: any number of members can be selected.", cx),
                        h::ToggleButtonGroup::new("toggle-default-multiple")
                            .selection_mode(SelectionMode::Multiple)
                            .separators(true)
                            .default_selected_keys(["bold", "underline"])
                            .child_toggle(
                                h::ToggleButton::new("tbu-m-bold").key("bold").label("Bold")
                            )
                            .child_toggle(
                                h::ToggleButton::new("tbu-m-italic")
                                    .key("italic")
                                    .label("Italic"),
                            )
                            .child_toggle(
                                h::ToggleButton::new("tbu-m-underline")
                                    .key("underline")
                                    .label("Underline"),
                            )
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Vertical",
                    specimen_body("toggle-vertical", row(vec![
                        h::ToggleButtonGroup::new("toggle-vertical")
                            .orientation(Orientation::Vertical)
                            .separators(true)
                            .child_toggle(h::ToggleButton::new("tbv-1").label("Top"))
                            .child_toggle(h::ToggleButton::new("tbv-2").label("Bottom"))
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Detached", "`is_detached` separates the buttons with gaps instead of connecting them; the attached icon group above the detached one shows the default for contrast.",
                    specimen_body("toggle-detached", col(vec![
                        h::ToggleButtonGroup::new("toggle-attached")
                            .separators(true)
                            .child_toggle(
                                h::ToggleButton::new("tbatt-1")
                                    .is_icon_only(true)
                                    .child(icon(h::icons::HEART, cx)),
                            )
                            .child_toggle(
                                h::ToggleButton::new("tbatt-2")
                                    .is_icon_only(true)
                                    .child(icon(h::icons::GEAR, cx)),
                            )
                            .child_toggle(
                                h::ToggleButton::new("tbatt-3")
                                    .is_icon_only(true)
                                    .child(icon(h::icons::MAIL, cx)),
                            )
                            .into_any_element(),
                        h::ToggleButtonGroup::new("toggle-detached")
                            .is_detached(true)
                            .child_toggle(
                                h::ToggleButton::new("tbd-1")
                                    .is_icon_only(true)
                                    .child(icon(h::icons::HEART, cx)),
                            )
                            .child_toggle(
                                h::ToggleButton::new("tbd-2")
                                    .is_icon_only(true)
                                    .child(icon(h::icons::GEAR, cx)),
                            )
                            .child_toggle(
                                h::ToggleButton::new("tbd-3")
                                    .is_icon_only(true)
                                    .child(icon(h::icons::MAIL, cx)),
                            )
                            .into_any_element(),
                    ]), cx),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
}
