//! Live `Theme.components` resolution: stock, defaults, recipes, and instance overrides.

use crate::harness;

use std::sync::{Arc, Mutex};

use gpui::{prelude::*, px, TestAppContext, VisualTestContext, Window};
use harness::{open_host, probe, settle, still};
use herogpui_components::{
    Button, Input, InputState, Menu, MenuItem, PickerItem, Popover, Select, Slider, Switch,
    TabItem, Tabs, TextArea, Tooltip, TooltipPlacement, TooltipPresentationRequest,
    TooltipPresenter, TooltipTrigger,
};
use herogpui_theme::{
    set_theme, with_unknown_recipe_diagnostic_handler, ButtonStyle, ComponentTheme,
    ComponentThemes, MenuStyle, PopoverStyle, SelectStyle, SliderStyle, SwitchStyle, TabsStyle,
    TextAreaStyle, TextFieldStyle, Theme, TooltipStyle, UnknownRecipeDiagnostic,
};

fn themed(components: ComponentThemes) -> Theme {
    Theme::builder("app", Theme::light())
        .components(components)
        .build()
}

fn apply_theme(cx: &mut VisualTestContext, theme: Theme) {
    cx.update(|window, cx| {
        set_theme(theme, cx);
        window.refresh();
    });
}

fn restore_stock(cx: &mut VisualTestContext) {
    apply_theme(cx, Theme::light());
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.05
}

fn painted_corners(quad: &gpui::Quad, scale: f32) -> [f32; 4] {
    let corners = quad.corner_radii;
    [
        corners.top_left.0 / scale,
        corners.top_right.0 / scale,
        corners.bottom_right.0 / scale,
        corners.bottom_left.0 / scale,
    ]
}

fn md_inner_mark(cx: &mut VisualTestContext) -> [f32; 4] {
    let (quads, scale) = cx.update(|window, _| (window.painted_quads(), window.scale_factor()));
    let inner = quads
        .iter()
        .find(|quad| {
            close(quad.bounds.size.width.0 / scale, 24.)
                && close(quad.bounds.size.height.0 / scale, 16.)
        })
        .expect("Md inner mark must paint");
    painted_corners(inner, scale)
}

fn assert_corners(actual: [f32; 4], expected: [f32; 4], context: &str) {
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert!(
            close(actual, expected),
            "{context}: painted corner {actual} must match {expected}"
        );
    }
}

fn slider_host<'a>(
    cx: &'a mut TestAppContext,
    recipe: Option<&'static str>,
) -> &'a mut VisualTestContext {
    open_host(cx, move || {
        let mut slider = Slider::new("theme-slider", 100.);
        if let Some(name) = recipe {
            slider = slider.recipe(name);
        }
        gpui::div().w(px(600.)).child(slider).into_any_element()
    })
}

#[gpui::test]
fn stock_slider_geometry_is_unchanged(cx: &mut TestAppContext) {
    let cx = slider_host(cx, None);
    let key_radius = cx.update(|_, cx| f32::from(herogpui_components::extend::key_radius(cx)));
    assert_corners(
        md_inner_mark(cx),
        [key_radius.min(8.); 4],
        "stock slider must keep the Md key-radius inner mark",
    );
}

#[gpui::test]
fn stock_menu_row_interval_is_unchanged(cx: &mut TestAppContext) {
    let cx = open_host(cx, move || {
        Menu::new(
            "stock-menu",
            vec![
                MenuItem::new("first", "First"),
                MenuItem::new("second", "Second"),
            ],
        )
        .animate_entry(false)
        .item_content(|key, _| {
            let key = key.clone();
            gpui::div()
                .h(px(20.))
                .w(px(40.))
                .debug_selector(move || format!("stock-{key}"))
                .into_any_element()
        })
        .into_any_element()
    });
    let first = cx.debug_bounds("stock-first").expect("first row");
    let second = cx.debug_bounds("stock-second").expect("second row");
    assert_eq!(second.origin.y - first.origin.y, px(38.));
}

#[gpui::test]
fn missing_recipe_is_a_noop(cx: &mut TestAppContext) {
    let cx = slider_host(cx, Some("missing"));
    let key_radius = cx.update(|_, cx| f32::from(herogpui_components::extend::key_radius(cx)));
    assert_corners(
        md_inner_mark(cx),
        [key_radius.min(8.); 4],
        "an unknown recipe must leave stock slider geometry",
    );
}

#[gpui::test]
fn slider_theme_defaults_recipes_and_instance_sx_follow_precedence(cx: &mut TestAppContext) {
    let cx = open_host(cx, move || {
        gpui::div()
            .w(px(600.))
            .child(Slider::new("theme-default", 100.))
            .into_any_element()
    });
    let key_radius = cx.update(|_, cx| f32::from(herogpui_components::extend::key_radius(cx)));
    assert_corners(
        md_inner_mark(cx),
        [key_radius.min(8.); 4],
        "first frame must paint stock before set_theme",
    );

    apply_theme(
        cx,
        themed(
            ComponentThemes::default().slider(
                ComponentTheme::new(SliderStyle::default().radius(px(2.)))
                    .recipe("compact", SliderStyle::default().radius(px(3.))),
            ),
        ),
    );
    assert_corners(
        md_inner_mark(cx),
        [2.; 4],
        "theme defaults must reach the inner mark",
    );

    restore_stock(cx);
    assert_corners(
        md_inner_mark(cx),
        [key_radius.min(8.); 4],
        "switching back to the stock theme must restore stock corners",
    );
}

#[gpui::test]
fn slider_named_recipes_refine_defaults_and_ignore_missing_names(cx: &mut TestAppContext) {
    let cx = open_host(cx, move || {
        gpui::div()
            .w(px(600.))
            .child(
                Slider::new("recipe-slider", 100.)
                    .recipe("compact")
                    .recipe("missing"),
            )
            .into_any_element()
    });
    apply_theme(
        cx,
        themed(
            ComponentThemes::default().slider(
                ComponentTheme::new(SliderStyle::default().radius(px(2.)))
                    .recipe("compact", SliderStyle::default().radius(px(3.))),
            ),
        ),
    );
    assert_corners(
        md_inner_mark(cx),
        [3.; 4],
        "named recipes must refine defaults; a missing name adds nothing",
    );
}

#[gpui::test]
fn slider_instance_sx_wins_named_corners_over_theme_radius(cx: &mut TestAppContext) {
    let cx = open_host(cx, move || {
        gpui::div()
            .w(px(600.))
            .child(
                Slider::new("sx-slider", 100.)
                    .recipe("compact")
                    .sx(|el| el.rounded_tl(px(6.))),
            )
            .into_any_element()
    });
    apply_theme(
        cx,
        themed(
            ComponentThemes::default().slider(
                ComponentTheme::new(SliderStyle::default())
                    .recipe("compact", SliderStyle::default().radius(px(3.))),
            ),
        ),
    );
    assert_corners(
        md_inner_mark(cx),
        [6., 3., 3., 3.],
        "instance sx must win the named corner over the theme radius",
    );
}

#[gpui::test]
fn switch_theme_radius_reaches_the_track(cx: &mut TestAppContext) {
    let cx = open_host(cx, move || Switch::new("theme-switch").into_any_element());
    apply_theme(
        cx,
        themed(
            ComponentThemes::default()
                .switch(ComponentTheme::new(SwitchStyle::default().radius(px(3.)))),
        ),
    );
    let (quads, scale) = cx.update(|window, _| (window.painted_quads(), window.scale_factor()));
    let track = quads
        .iter()
        .find(|quad| {
            close(quad.bounds.size.width.0 / scale, 40.)
                && close(quad.bounds.size.height.0 / scale, 20.)
        })
        .expect("Md switch track must paint");
    assert_corners(
        painted_corners(track, scale),
        [3.; 4],
        "switch theme radius must reach the track",
    );
}

#[gpui::test]
fn menu_theme_recipe_sets_row_interval(cx: &mut TestAppContext) {
    let cx = open_host(cx, move || {
        Menu::new(
            "theme-menu",
            vec![
                MenuItem::new("first", "First"),
                MenuItem::new("second", "Second"),
            ],
        )
        .recipe("compact")
        .animate_entry(false)
        .item_content(|key, _| {
            let key = key.clone();
            gpui::div()
                .h(px(20.))
                .w(px(40.))
                .debug_selector(move || format!("theme-{key}"))
                .into_any_element()
        })
        .into_any_element()
    });
    apply_theme(
        cx,
        themed(
            ComponentThemes::default().menu(
                ComponentTheme::new(MenuStyle::default().panel_gap(px(0.))).recipe(
                    "compact",
                    MenuStyle::default()
                        .row_height(px(28.))
                        .row_padding_y(px(2.))
                        .panel_padding(px(4.)),
                ),
            ),
        ),
    );
    let first = cx.debug_bounds("theme-first").expect("first row");
    let second = cx.debug_bounds("theme-second").expect("second row");
    assert_eq!(second.origin.y - first.origin.y, px(28.));
}

#[gpui::test]
fn menu_instance_panel_gap_wins_over_theme_defaults(cx: &mut TestAppContext) {
    let cx = open_host(cx, move || {
        Menu::new(
            "override-menu",
            vec![
                MenuItem::new("first", "First"),
                MenuItem::new("second", "Second"),
            ],
        )
        .animate_entry(false)
        .panel_gap(px(8.))
        .row_height(px(28.))
        .row_padding_y(px(2.))
        .panel_padding(px(4.))
        .item_content(|key, _| {
            let key = key.clone();
            gpui::div()
                .h(px(20.))
                .w(px(40.))
                .debug_selector(move || format!("override-{key}"))
                .into_any_element()
        })
        .into_any_element()
    });
    apply_theme(
        cx,
        themed(
            ComponentThemes::default()
                .menu(ComponentTheme::new(MenuStyle::default().panel_gap(px(0.)))),
        ),
    );
    let first = cx.debug_bounds("override-first").expect("first row");
    let second = cx.debug_bounds("override-second").expect("second row");
    assert_eq!(second.origin.y - first.origin.y, px(36.));
}

#[gpui::test]
fn button_theme_style_and_instance_sx_follow_precedence(cx: &mut TestAppContext) {
    let cx = open_host(cx, move || {
        gpui::div()
            .flex()
            .flex_col()
            .items_start()
            .child(
                gpui::div()
                    .debug_selector(|| "stock-btn".into())
                    .child(Button::new("stock").label("Go")),
            )
            .child(
                gpui::div()
                    .debug_selector(|| "recipe-btn".into())
                    .child(Button::new("recipe").label("Go").recipe("compact")),
            )
            .child(
                gpui::div().debug_selector(|| "sx-btn".into()).child(
                    Button::new("sx")
                        .label("Go")
                        .recipe("compact")
                        .sx(|el| el.h(px(40.))),
                ),
            )
            .into_any_element()
    });
    apply_theme(
        cx,
        themed(
            ComponentThemes::default().button(
                ComponentTheme::new(ButtonStyle::default())
                    .recipe("compact", ButtonStyle::default().style(|el| el.h(px(28.)))),
            ),
        ),
    );
    assert_eq!(cx.debug_bounds("stock-btn").unwrap().size.height, px(36.));
    assert_eq!(cx.debug_bounds("recipe-btn").unwrap().size.height, px(28.));
    assert_eq!(cx.debug_bounds("sx-btn").unwrap().size.height, px(40.));
}

#[gpui::test]
fn select_and_input_theme_heights_resolve_live(cx: &mut TestAppContext) {
    let state = cx.new(|cx| InputState::new(cx));
    let cx = open_host(cx, move || {
        gpui::div()
            .flex()
            .flex_col()
            .items_start()
            .child(Select::new(
                "theme-sel",
                vec![PickerItem::new("a", "Alpha")],
            ))
            .child(
                gpui::div()
                    .debug_selector(|| "theme-field".into())
                    .child(Input::new(state.clone()).recipe("compact")),
            )
            .into_any_element()
    });
    apply_theme(
        cx,
        themed(
            ComponentThemes::default()
                .select(ComponentTheme::new(SelectStyle::default().height(px(28.))))
                .text_field(
                    ComponentTheme::new(TextFieldStyle::default())
                        .recipe("compact", TextFieldStyle::default().height(px(28.))),
                ),
        ),
    );
    assert_eq!(
        cx.debug_bounds("select-trigger-Name(\"theme-sel\")")
            .unwrap()
            .size
            .height,
        px(28.)
    );
    assert_eq!(cx.debug_bounds("theme-field").unwrap().size.height, px(28.));

    restore_stock(cx);
    assert_eq!(
        cx.debug_bounds("select-trigger-Name(\"theme-sel\")")
            .unwrap()
            .size
            .height,
        px(36.)
    );
    assert_eq!(cx.debug_bounds("theme-field").unwrap().size.height, px(36.));
}

#[gpui::test]
fn tooltip_theme_defaults_recipes_and_instance_overrides_reach_presenters_and_in_window(
    cx: &mut TestAppContext,
) {
    still();
    #[derive(Clone, Debug)]
    struct Recorded {
        radius: gpui::Pixels,
        offset: gpui::Pixels,
        padding: gpui::Pixels,
        text_size: gpui::Pixels,
    }
    struct CapturePresenter(Arc<Mutex<Option<Recorded>>>);
    impl TooltipPresenter for CapturePresenter {
        fn present(
            &self,
            req: &TooltipPresentationRequest,
            _window: &mut Window,
            _cx: &mut gpui::App,
        ) {
            *self.0.lock().unwrap() = Some(Recorded {
                radius: req.radius,
                offset: req.offset,
                padding: req.padding,
                text_size: req.text_size,
            });
        }

        fn dismiss(
            &self,
            _key: &gpui::ElementId,
            _window: Option<&mut Window>,
            _cx: &mut gpui::App,
        ) {
        }
    }

    let seen = Arc::new(Mutex::new(None));
    let presenter: Arc<dyn TooltipPresenter> = Arc::new(CapturePresenter(seen.clone()));
    let vcx = open_host(cx, move || {
        gpui::div()
            .p(px(40.))
            .flex()
            .gap(px(24.))
            .child(
                Tooltip::new("Copy")
                    .id("tip-ext")
                    .recipe("compact")
                    .offset(px(14.))
                    .presenter(presenter.clone())
                    .child(Button::new("ext-btn").label("Copy")),
            )
            .child(
                Tooltip::new("Paste")
                    .id("tip-inline")
                    .trigger(TooltipTrigger::Focus)
                    .delay(0)
                    .placement(TooltipPlacement::Bottom)
                    .body(|_, _| {
                        gpui::div()
                            .debug_selector(|| "tip-inline-body".into())
                            .w(px(30.))
                            .h(px(12.))
                            .into_any_element()
                    })
                    .child(Button::new("inline-btn").label("Paste")),
            )
            .into_any_element()
    });

    apply_theme(
        vcx,
        themed(
            ComponentThemes::default().tooltip(
                ComponentTheme::new(
                    TooltipStyle::default()
                        .radius(px(5.))
                        .offset(px(9.))
                        .delay_ms(500)
                        .padding(px(10.)),
                )
                .recipe(
                    "compact",
                    TooltipStyle::default()
                        .delay_ms(0)
                        .padding(px(4.))
                        .text_size(px(11.)),
                ),
            ),
        ),
    );

    // Focus opens `tip-ext` (delay_ms = 0 from recipe, offset = 14 from instance).
    harness::press(vcx, "tab");
    settle(vcx);
    let rec = seen
        .lock()
        .unwrap()
        .clone()
        .expect("external presenter received request");
    assert_eq!(rec.radius, px(5.));
    assert_eq!(rec.offset, px(14.));
    assert_eq!(rec.padding, px(4.));
    assert_eq!(rec.text_size, px(11.));

    // Tab to the inline tooltip trigger: theme default padding(10) and radius(5) reach the surface.
    harness::press(vcx, "tab");
    settle(vcx);
    let body = vcx
        .debug_bounds("tip-inline-body")
        .expect("inline tooltip body");
    assert_eq!(body.size.width, px(30.));
    let (quads, scale) = vcx.update(|w, _| (w.painted_quads(), w.scale_factor()));
    let tip_quad = quads
        .iter()
        .find(|q| {
            close(q.bounds.size.width.0 / scale, 50.) && close(q.bounds.size.height.0 / scale, 32.)
        })
        .expect("in-window tooltip surface with 10px theme padding must paint 50x32");
    assert_corners(
        painted_corners(tip_quad, scale),
        [5.; 4],
        "tooltip theme radius",
    );
}

#[gpui::test]
fn popover_theme_defaults_recipes_and_instance_overrides(cx: &mut TestAppContext) {
    still();
    let vcx = open_host(cx, move || {
        gpui::div()
            .p(px(20.))
            .flex()
            .gap(px(40.))
            .child(
                Popover::new(
                    gpui::div()
                        .debug_selector(|| "pop-rec-trigger".into())
                        .w(px(40.))
                        .h(px(20.)),
                )
                .id("pop-recipe")
                .default_open(true)
                .recipe("flush")
                .child(probe("pop-rec-probe")),
            )
            .child(
                Popover::new(
                    gpui::div()
                        .debug_selector(|| "pop-ovr-trigger".into())
                        .w(px(40.))
                        .h(px(20.)),
                )
                .id("pop-override")
                .default_open(true)
                .recipe("flush")
                .panel_width(px(210.))
                .padding(px(12.))
                .offset(px(16.))
                .radius(px(9.))
                .child(probe("pop-ovr-probe")),
            )
            .into_any_element()
    });

    apply_theme(
        vcx,
        themed(
            ComponentThemes::default().popover(
                ComponentTheme::new(
                    PopoverStyle::default()
                        .panel_width(px(180.))
                        .padding(px(8.))
                        .offset(px(6.))
                        .radius(px(3.)),
                )
                .recipe("flush", PopoverStyle::default().padding(px(4.))),
            ),
        ),
    );
    settle(vcx);

    let (quads, scale) = vcx.update(|w, _| (w.painted_quads(), w.scale_factor()));
    let rec_trig = vcx.debug_bounds("pop-rec-trigger").unwrap();
    let rec_probe = vcx.debug_bounds("pop-rec-probe").unwrap();
    // Probe is inset by 4px (`flush` recipe); dialog width is 180px (theme default); offset is 6px.
    assert_eq!(rec_probe.origin.y - rec_trig.bottom(), px(6. + 4.));
    let rec_dialog = quads
        .iter()
        .find(|q| close(q.bounds.size.width.0 / scale, 180.))
        .expect("themed popover dialog quad");
    assert_corners(
        painted_corners(rec_dialog, scale),
        [3.; 4],
        "popover theme radius",
    );

    // Instance overrides win every field over both defaults and `flush` recipe.
    let ovr_trig = vcx.debug_bounds("pop-ovr-trigger").unwrap();
    let ovr_probe = vcx.debug_bounds("pop-ovr-probe").unwrap();
    assert_eq!(ovr_probe.origin.y - ovr_trig.bottom(), px(16. + 12.));
    let ovr_dialog = quads
        .iter()
        .find(|q| close(q.bounds.size.width.0 / scale, 210.))
        .expect("overridden popover dialog quad");
    assert_corners(
        painted_corners(ovr_dialog, scale),
        [9.; 4],
        "popover instance radius",
    );
}

#[gpui::test]
fn textarea_theme_cascades_and_keeps_rows_height(cx: &mut TestAppContext) {
    still();
    let state_recipe = cx.new(|cx| InputState::new(cx));
    let state_override = cx.new(|cx| InputState::new(cx));
    let vcx = open_host(cx, move || {
        gpui::div()
            .p(px(16.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(
                gpui::div().debug_selector(|| "ta-recipe".into()).child(
                    TextArea::new(state_recipe.clone())
                        .rows(3)
                        .placeholder("One")
                        .recipe("compact"),
                ),
            )
            .into_any_element()
    });

    apply_theme(
        vcx,
        themed(
            ComponentThemes::default()
                .text_field(ComponentTheme::new(
                    TextFieldStyle::default()
                        .height(px(90.))
                        .padding_x(px(18.))
                        .radius(px(5.)),
                ))
                .textarea(
                    ComponentTheme::new(TextAreaStyle::default().radius(px(3.)))
                        .recipe("compact", TextAreaStyle::default().padding_x(px(6.))),
                ),
        ),
    );
    settle(vcx);

    // `text_field.height(90.)` must NOT replace TextArea's `rows(3)` height (3*20 + 16 = 76).
    let ta_bounds = vcx.debug_bounds("ta-recipe").unwrap();
    assert_eq!(ta_bounds.size.height, px(76.));
    // Textarea surface quad uses `textarea`'s radius (3.) rather than `text_field`'s (5.).
    let (quads, scale) = vcx.update(|w, _| (w.painted_quads(), w.scale_factor()));
    let ta_quad = quads
        .iter()
        .find(|q| close(q.bounds.size.height.0 / scale, 76.))
        .expect("textarea surface quad");
    assert_corners(
        painted_corners(ta_quad, scale),
        [3.; 4],
        "textarea theme radius",
    );

    // `compact` recipe padding_x(6.) insets the placeholder by 6px inside the 1px border (or 6px).
    let rec_ph = vcx.debug_bounds("input-placeholder").unwrap();
    let rec_inset = f32::from(rec_ph.origin.x - ta_bounds.origin.x);
    assert!(
        (rec_inset - 6.0).abs() <= 1.5,
        "compact recipe padding_x(6.) must inset placeholder ~6px, got {rec_inset}"
    );

    let vcx_ovr = open_host(cx, move || {
        gpui::div()
            .p(px(16.))
            .child(
                gpui::div().debug_selector(|| "ta-override".into()).child(
                    TextArea::new(state_override.clone())
                        .rows(3)
                        .placeholder("Two")
                        .recipe("compact")
                        .padding_x(px(14.)),
                ),
            )
            .into_any_element()
    });
    apply_theme(
        vcx_ovr,
        themed(
            ComponentThemes::default().textarea(
                ComponentTheme::new(TextAreaStyle::default().radius(px(3.)))
                    .recipe("compact", TextAreaStyle::default().padding_x(px(6.))),
            ),
        ),
    );
    settle(vcx_ovr);
    let ovr_bounds = vcx_ovr.debug_bounds("ta-override").unwrap();
    let ovr_ph = vcx_ovr.debug_bounds("input-placeholder").unwrap();
    let ovr_inset = f32::from(ovr_ph.origin.x - ovr_bounds.origin.x);
    assert!(
        (ovr_inset - 14.0).abs() <= 1.5 && ovr_inset > rec_inset + 5.0,
        "instance padding_x(14.) must override compact recipe padding_x(6.): ovr={ovr_inset}, rec={rec_inset}"
    );
}

#[gpui::test]
fn tabs_theme_defaults_recipes_and_instance_overrides(cx: &mut TestAppContext) {
    still();
    let vcx = open_host(cx, move || {
        gpui::div()
            .p(px(16.))
            .flex()
            .flex_col()
            .gap(px(20.))
            .child(
                Tabs::new(
                    "tabs-recipe",
                    vec![TabItem::new("a", "One"), TabItem::new("b", "Two")],
                    "a",
                )
                .recipe("dense"),
            )
            .child(
                Tabs::new(
                    "tabs-override",
                    vec![TabItem::new("a", "One"), TabItem::new("b", "Two")],
                    "a",
                )
                .recipe("dense")
                .list_padding(px(10.))
                .radius(px(7.)),
            )
            .into_any_element()
    });

    apply_theme(
        vcx,
        themed(
            ComponentThemes::default().tabs(
                ComponentTheme::new(
                    TabsStyle::default()
                        .list_padding(px(8.))
                        .radius(px(3.))
                        .indicator_shadow(false)
                        .hover_fill(false)
                        .text_size(px(12.)),
                )
                .recipe("dense", TabsStyle::default().list_padding(px(2.))),
            ),
        ),
    );
    settle(vcx);

    let (quads, scale) = vcx.update(|w, _| (w.painted_quads(), w.scale_factor()));
    let recipe_ind = vcx
        .debug_bounds("Name(\"tabs-recipe\")-indicator")
        .expect("recipe indicator bounds");
    let rec_ind_quad = quads
        .iter()
        .find(|q| {
            close(q.bounds.origin.x.0 / scale, f32::from(recipe_ind.origin.x))
                && close(
                    q.bounds.size.width.0 / scale,
                    f32::from(recipe_ind.size.width),
                )
        })
        .expect("recipe indicator quad");
    assert_corners(
        painted_corners(rec_ind_quad, scale),
        [3.; 4],
        "tabs theme default radius",
    );

    let ovr_ind = vcx
        .debug_bounds("Name(\"tabs-override\")-indicator")
        .expect("override indicator bounds");
    let ovr_ind_quad = quads
        .iter()
        .find(|q| {
            close(q.bounds.origin.x.0 / scale, f32::from(ovr_ind.origin.x))
                && close(q.bounds.size.width.0 / scale, f32::from(ovr_ind.size.width))
        })
        .expect("override indicator quad");
    assert_corners(
        painted_corners(ovr_ind_quad, scale),
        [7.; 4],
        "tabs instance radius must override theme",
    );

    // Tabs list_padding: recipe sets 2px; explicit instance override sets 10px.
    let rec_list_quad = quads
        .iter()
        .find(|q| {
            q.bounds.size.height.0 / scale > f32::from(recipe_ind.size.height) + 1.0
                && close(
                    f32::from(recipe_ind.origin.y) - q.bounds.origin.y.0 / scale,
                    2.0,
                )
        })
        .expect("recipe tabs list tray quad with 2px list_padding inset");
    let ovr_list_quad = quads
        .iter()
        .find(|q| {
            q.bounds.size.height.0 / scale > f32::from(ovr_ind.size.height) + 1.0
                && close(
                    f32::from(ovr_ind.origin.y) - q.bounds.origin.y.0 / scale,
                    10.0,
                )
        })
        .expect("override tabs list tray quad with 10px list_padding inset");
    assert!(
        ovr_list_quad.bounds.size.height.0 / scale > rec_list_quad.bounds.size.height.0 / scale,
        "instance list_padding(10.) must produce a taller tray than dense recipe list_padding(2.)"
    );
}

#[gpui::test]
fn unknown_recipe_diagnostics_report_live_components_when_opted_in(cx: &mut TestAppContext) {
    still();
    let seen = Arc::new(Mutex::new(Vec::<UnknownRecipeDiagnostic>::new()));
    let sink = seen.clone();
    with_unknown_recipe_diagnostic_handler(
        Arc::new(move |diag| {
            sink.lock().unwrap().push(diag.clone());
        }),
        || {
            let ta_state = cx.new(|cx| InputState::new(cx));
            let vcx = open_host(cx, move || {
                gpui::div()
                    .flex()
                    .flex_col()
                    .child(
                        Tooltip::new("Tip")
                            .id("diag-tip")
                            .recipe("missing-tip")
                            .child(Button::new("diag-tip-btn").label("Tip")),
                    )
                    .child(
                        Popover::new(Button::new("diag-pop-btn").label("Pop"))
                            .id("diag-pop")
                            .recipe("missing-pop")
                            .child(gpui::div().child("Body")),
                    )
                    .child(TextArea::new(ta_state.clone()).recipe("missing-ta"))
                    .child(
                        Tabs::new("diag-tabs", vec![TabItem::new("a", "A")], "a")
                            .recipe("missing-tabs"),
                    )
                    .into_any_element()
            });
            settle(vcx);
        },
    );

    let events = seen.lock().unwrap().clone();
    assert!(events
        .iter()
        .any(|d| d.component == "tooltip" && d.requested == "missing-tip"));
    assert!(events
        .iter()
        .any(|d| d.component == "popover" && d.requested == "missing-pop"));
    assert!(events
        .iter()
        .any(|d| d.component == "textarea" && d.requested == "missing-ta"));
    assert!(
        !events
            .iter()
            .any(|d| d.component == "text_field" && d.requested == "missing-ta"),
        "TextArea must not emit a duplicate text_field diagnostic: {events:?}"
    );
    assert!(events
        .iter()
        .any(|d| d.component == "tabs" && d.requested == "missing-tabs"));
}

#[gpui::test]
fn select_intrinsic_panel_width_and_fit_panel_width_keep_rows_unwrapped(cx: &mut TestAppContext) {
    still();
    let long_label = "Export Annotated Video Recording With Frame Borders";
    let items = vec![
        PickerItem::new("short", "Short"),
        PickerItem::new("long", long_label),
    ];

    // Custom `item_content` with `.flex_1()` + `fit_panel_width(true)`.
    let vcx = open_host(cx, {
        let items = items.clone();
        move || {
            gpui::div()
                .p(px(16.))
                .child(
                    Select::new("fit-sel", items.clone())
                        .default_open(true)
                        .fit_panel_width(true)
                        .item_content(|key, _sel, _, _| {
                            let label = if key.as_ref() == "long" {
                                "Export Annotated Video Recording With Frame Borders"
                            } else {
                                "Short"
                            };
                            let probe_key = key.clone();
                            gpui::div()
                                .flex_1()
                                .flex()
                                .items_center()
                                .gap(px(12.))
                                .debug_selector(move || format!("fit-row-{probe_key}"))
                                .child(gpui::div().w(px(16.)).h(px(16.)))
                                .child(gpui::div().flex_1().child(label))
                                .into_any_element()
                        }),
                )
                .into_any_element()
        }
    });
    settle(vcx);

    // Single-line custom row: height remains 20px (1 line) instead of wrapping onto multiple lines.
    let long_row = vcx
        .debug_bounds("fit-row-long")
        .expect("long custom option row must be rendered");
    assert_eq!(
        long_row.size.height,
        px(20.),
        "fitted panel width must prevent long item_content row from wrapping"
    );

    // Verify `intrinsic_panel_width` directly inside a render/prepaint host (its intended lifecycle).
    let measurements = Arc::new(Mutex::new(
        None::<(gpui::Pixels, gpui::Pixels, gpui::Pixels, gpui::Pixels)>,
    ));
    let m_out = measurements.clone();
    let vcx2 = open_host(cx, move || {
        let items = items.clone();
        let m_out = m_out.clone();
        gpui::canvas(
            move |_, window, cx| {
                let custom = Select::new("fit-sel-measure", items.clone()).item_content(
                    |key, _sel, _, _| {
                        let label = if key.as_ref() == "long" {
                            "Export Annotated Video Recording With Frame Borders"
                        } else {
                            "Short"
                        };
                        gpui::div()
                            .flex_1()
                            .flex()
                            .items_center()
                            .gap(px(12.))
                            .child(gpui::div().w(px(16.)).h(px(16.)))
                            .child(gpui::div().flex_1().child(label))
                            .into_any_element()
                    },
                );
                let w_custom = custom.intrinsic_panel_width(window, cx);
                let base = Select::new("m1", items.clone()).item_leading(|_, _| {
                    Some(gpui::div().w(px(16.)).h(px(16.)).into_any_element())
                });
                let w_base = base.intrinsic_panel_width(window, cx);
                let floored = Select::new("m2", items.clone()).panel_min_width(px(800.));
                let w_floor = floored.intrinsic_panel_width(window, cx);
                let compact = Select::new("m3", items.clone())
                    .recipe("compact")
                    .item_leading(|_, _| {
                        Some(gpui::div().w(px(16.)).h(px(16.)).into_any_element())
                    });
                let w_compact = compact.intrinsic_panel_width(window, cx);
                *m_out.lock().unwrap() = Some((w_custom, w_base, w_floor, w_compact));
            },
            |_, _, _, _| {},
        )
        .size_full()
        .into_any_element()
    });
    apply_theme(
        vcx2,
        themed(
            ComponentThemes::default().select(
                ComponentTheme::new(SelectStyle::default()).recipe(
                    "compact",
                    SelectStyle::default()
                        .panel_padding(px(2.))
                        .row_padding_x(px(4.))
                        .row_text_size(px(11.)),
                ),
            ),
        ),
    );
    let (custom_w, plain_w, floored_w, compact_w) = measurements
        .lock()
        .unwrap()
        .expect("prepaint measurements recorded");
    restore_stock(vcx2);
    assert!(
        custom_w > px(260.),
        "intrinsic width must exceed default trigger width for long option, got {custom_w:?}"
    );
    assert_eq!(floored_w, px(800.));
    assert!(
        compact_w < plain_w,
        "compact recipe metrics must shrink intrinsic_panel_width ({compact_w:?} < {plain_w:?})"
    );
}

#[gpui::test]
fn bare_textarea_and_input_group_suppress_theme_background_and_popover_arrow_matches_panel_bg(
    cx: &mut TestAppContext,
) {
    still();
    let bare_state = cx.new(|cx| InputState::new(cx));
    let group_state = cx.new(|cx| InputState::new(cx));
    let bg_color = herogpui_core::oklch(0.65, 0.2, 145.0);
    let pop_bg = herogpui_core::oklch(0.45, 0.22, 260.0);

    let vcx = open_host(cx, move || {
        gpui::div()
            .p(px(16.))
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(
                gpui::div()
                    .debug_selector(|| "bare-ta".into())
                    .child(TextArea::new(bare_state.clone()).is_bare(true).rows(2)),
            )
            .child(
                gpui::div().debug_selector(|| "grouped-input".into()).child(
                    herogpui_components::InputGroup::new()
                        .is_bare(true)
                        .text_area(TextArea::new(group_state.clone())),
                ),
            )
            .child(
                Popover::new(Button::new("pop-arrow-btn").label("Open"))
                    .id("pop-arrow-themed")
                    .default_open(true)
                    .child(herogpui_components::PopoverArrow::new())
                    .child(gpui::div().child("Body")),
            )
            .into_any_element()
    });
    apply_theme(
        vcx,
        themed(
            ComponentThemes::default()
                .textarea(ComponentTheme::new(
                    TextAreaStyle::default().background(bg_color),
                ))
                .text_field(ComponentTheme::new(
                    TextFieldStyle::default().background(bg_color),
                ))
                .popover(ComponentTheme::new(
                    PopoverStyle::default().background(pop_bg),
                )),
        ),
    );
    settle(vcx);

    let (quads, scale) = vcx.update(|w, _| (w.painted_quads(), w.scale_factor()));
    let bare_bounds = vcx.debug_bounds("bare-ta").unwrap();
    let grouped_bounds = vcx.debug_bounds("grouped-input").unwrap();
    let has_bare_bg = quads.iter().any(|q| {
        q.background == gpui::Background::from(bg_color)
            && close(q.bounds.origin.y.0 / scale, f32::from(bare_bounds.origin.y))
    });
    assert!(
        !has_bare_bg,
        "TextArea::is_bare(true) must suppress TextAreaStyle::background"
    );
    let has_grouped_bg = quads.iter().any(|q| {
        q.background == gpui::Background::from(bg_color)
            && close(
                q.bounds.origin.y.0 / scale,
                f32::from(grouped_bounds.origin.y),
            )
    });
    assert!(
        !has_grouped_bg,
        "Input inside InputGroup must suppress TextFieldStyle::background"
    );

    // Popover dialog paints with `pop_bg`, and PopoverArrow receives the same resolved color.
    let has_popover_bg = quads
        .iter()
        .any(|q| q.background == gpui::Background::from(pop_bg));
    assert!(
        has_popover_bg,
        "Popover panel must paint PopoverStyle::background"
    );
    let popover_src = include_str!("../src/popover.rs");
    assert!(
        popover_src.contains("arrow.background = Some(panel_bg);")
            && popover_src.contains("background: Option<gpui::Hsla>")
            && popover_src.contains(".text_color(color)"),
        "PopoverArrow must inherit resolved panel_bg from Popover"
    );
}
