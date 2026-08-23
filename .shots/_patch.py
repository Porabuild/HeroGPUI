"""Drawer page: the seven v3 examples it was missing."""
import io

P = 'gallery/src/pages/components.rs'
s = io.open(P, encoding='utf-8', newline='').read()


def rep(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:70])
    s = s.replace(old, new)


rep("""            crate::pages::Page::Drawer.import_line(),
            vec![(
                "Usage",""",
    """            crate::pages::Page::Drawer.import_line(),
            vec![
                (
                    "Placement",
                    col([
                        ("dr-left", "Left", h::DrawerPlacement::Left),
                        ("dr-right", "Right", h::DrawerPlacement::Right),
                        ("dr-top", "Top", h::DrawerPlacement::Top),
                        ("dr-bottom", "Bottom", h::DrawerPlacement::Bottom),
                    ]
                    .into_iter()
                    .map(|(key, label, placement)| {
                        let open = self.demo_overlay(key);
                        overlay_demo(
                            key,
                            label,
                            open,
                            h::Drawer::new()
                                .is_open(open)
                                .placement(placement)
                                .title(format!("From the {label}"))
                                .is_dismissible(true)
                                .child(gpui::div().child("The panel slides in along its edge."))
                                .on_open_change(bool_cb(cx.listener(
                                    move |this, v: &bool, _, cx| {
                                        this.set_demo_flag(key, *v);
                                        cx.notify();
                                    },
                                )))
                                .into_any_element(),
                            cx,
                        )
                    })
                    .collect()),
                ),
                (
                    "Non-Dismissable",
                    col(vec![overlay_demo(
                        "dr-no-dismiss",
                        "Open a non-dismissable drawer",
                        self.demo_overlay("dr-no-dismiss"),
                        h::Drawer::new()
                            .is_open(self.demo_overlay("dr-no-dismiss"))
                            .title("Finish first")
                            .is_dismissible(false)
                            .is_keyboard_dismiss_disabled(true)
                            .child(gpui::div().child("The backdrop and Escape are both inert."))
                            .footer_child(h::Button::new("dr-no-dismiss-ok").label("Done").on_press(
                                cx.listener(|this, _, _, cx| {
                                    this.set_demo_flag("dr-no-dismiss", false);
                                    cx.notify();
                                }),
                            ))
                            .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("dr-no-dismiss", *v);
                                cx.notify();
                            })))
                            .into_any_element(),
                        cx,
                    )]),
                ),
                (
                    "Scrollable Content",
                    col(vec![overlay_demo(
                        "dr-scroll",
                        "Open a long drawer",
                        self.demo_overlay("dr-scroll"),
                        h::Drawer::new()
                            .is_open(self.demo_overlay("dr-scroll"))
                            .title("Release notes")
                            .is_dismissible(true)
                            .child(
                                gpui::div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(8.))
                                    .children((1..=20).map(|n| {
                                        gpui::div().child(format!("Change {n} of twenty."))
                                    })),
                            )
                            .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("dr-scroll", *v);
                                cx.notify();
                            })))
                            .into_any_element(),
                        cx,
                    )]),
                ),
                (
                    "Controlled State",
                    col(vec![
                        para(
                            &format!(
                                "The flag lives with the caller: {}",
                                if self.demo_overlay("dr-controlled") {
                                    "open"
                                } else {
                                    "closed"
                                }
                            ),
                            cx,
                        ),
                        overlay_demo(
                            "dr-controlled",
                            "Open (controlled)",
                            self.demo_overlay("dr-controlled"),
                            h::Drawer::new()
                                .is_open(self.demo_overlay("dr-controlled"))
                                .title("Controlled")
                                .is_dismissible(true)
                                .child(gpui::div().child("Closing reports through onOpenChange."))
                                .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("dr-controlled", *v);
                                    cx.notify();
                                })))
                                .into_any_element(),
                            cx,
                        ),
                    ]),
                ),
                (
                    "With Form",
                    col(vec![overlay_demo(
                        "dr-form",
                        "Open a form drawer",
                        self.demo_overlay("dr-form"),
                        h::Drawer::new()
                            .is_open(self.demo_overlay("dr-form"))
                            .title("New issue")
                            .is_dismissible(true)
                            .child(
                                gpui::div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(12.))
                                    .child(
                                        h::TextField::new(self.demo_text("dr-form-title", "", cx))
                                            .label("Title"),
                                    )
                                    .child(
                                        h::TextArea::new(self.demo_text("dr-form-body", "", cx))
                                            .label("Description")
                                            .rows(3),
                                    ),
                            )
                            .footer_child(h::Button::new("dr-form-save").label("Create").on_press(
                                cx.listener(|this, _, _, cx| {
                                    this.set_demo_flag("dr-form", false);
                                    cx.notify();
                                }),
                            ))
                            .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("dr-form", *v);
                                cx.notify();
                            })))
                            .into_any_element(),
                        cx,
                    )]),
                ),
                (
                    "Navigation Drawer",
                    col(vec![overlay_demo(
                        "dr-nav",
                        "Open the navigation",
                        self.demo_overlay("dr-nav"),
                        h::Drawer::new()
                            .is_open(self.demo_overlay("dr-nav"))
                            .placement(h::DrawerPlacement::Left)
                            .title("Menu")
                            .is_dismissible(true)
                            .child(h::ListBox::new(
                                "dr-nav-list",
                                vec![
                                    h::ListBoxItem::new("home", "Home"),
                                    h::ListBoxItem::new("projects", "Projects"),
                                    h::ListBoxItem::new("settings", "Settings"),
                                    h::ListBoxItem::separator(),
                                    h::ListBoxItem::new("logout", "Log out").danger(),
                                ],
                            ))
                            .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("dr-nav", *v);
                                cx.notify();
                            })))
                            .into_any_element(),
                        cx,
                    )]),
                ),
                (
                    "Backdrop Variants",
                    col(herogpui_core::Backdrop::ALL
                        .iter()
                        .map(|backdrop| {
                            let key: &'static str = match backdrop {
                                herogpui_core::Backdrop::Opaque => "dr-bd-opaque",
                                herogpui_core::Backdrop::Blur => "dr-bd-blur",
                                herogpui_core::Backdrop::Transparent => "dr-bd-transparent",
                            };
                            let open = self.demo_overlay(key);
                            overlay_demo(
                                key,
                                backdrop.label(),
                                open,
                                h::Drawer::new()
                                    .is_open(open)
                                    .backdrop(*backdrop)
                                    .title(format!("Backdrop: {}", backdrop.label()))
                                    .is_dismissible(true)
                                    .child(gpui::div().child("The scrim behind the panel."))
                                    .on_open_change(bool_cb(cx.listener(
                                        move |this, v: &bool, _, cx| {
                                            this.set_demo_flag(key, *v);
                                            cx.notify();
                                        },
                                    )))
                                    .into_any_element(),
                                cx,
                            )
                        })
                        .collect()),
                ),
                (
                    "Usage",""")

io.open(P, 'w', encoding='utf-8', newline='').write(s)
print('patched drawer page')
