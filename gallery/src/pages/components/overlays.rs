//! Overlays gallery pages.
#![allow(clippy::redundant_clone)]

use super::*;

impl Gallery {
    // Overlays
    // -----------------------------------------------------------------------

    pub fn page_alert_dialog(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let is_open = self.alert_dialog_open;
        component_doc_page!(
            "Alert Dialog",
            crate::pages::Page::AlertDialog.description(),
            crate::pages::Page::AlertDialog.import_line(),
            vec![
                (
                    "Usage",
                    stretch_col(vec![{
                        overlay_min_h(
                            gpui::div()
                                .relative()
                                .flex()
                                .flex_col()
                        .items_center()
                        .w_full(),
                            is_open,
                            240.,
                        )
                        .child(
                            h::Button::new("ad-open")
                                .label("Delete project")
                                .variant(Variant::Danger)
                                .on_press(cx.listener(|this, _, _, cx| {
                                    this.alert_dialog_open = true;
                                    cx.notify();
                                })),
                        )
                        .child(
                            h::AlertDialog::new("Delete this project?").id("ad-usage")
                                .description(
                                    "This removes the project and every deployment. \
                                     This action cannot be undone.",
                                )
                                .is_open(is_open)
                                .child(h::AlertDialogCloseTrigger::new())
                                .footer_child(
                                    h::Button::new("ad-usage-cancel")
                                        .label("Cancel")
                                        .variant(Variant::Tertiary)
                                        .on_press(cx.listener(|this, _, _, cx| {
                                            this.alert_dialog_open = false;
                                            cx.notify();
                                        })),
                                )
                                .footer_child(
                                    h::Button::new("ad-usage-confirm")
                                        .label("Delete")
                                        .variant(Variant::Danger)
                                        .on_press(cx.listener(|this, _, _, cx| {
                                            this.alert_dialog_open = false;
                                            cx.notify();
                                        })),
                                )
                                .on_open_change(cx.listener(|this, v: &bool, _, cx| {
                                    this.alert_dialog_open = *v;
                                    cx.notify();
                                })),
                        )
                        .into_any_element()
                    }]),
                ),
                (
                    "Sizes",
                    stretch_col([
                        ("ad-size-xs", "Xs", h::AlertDialogSize::Xs),
                        ("ad-size-sm", "Sm", h::AlertDialogSize::Sm),
                        ("ad-size-md", "Md", h::AlertDialogSize::Md),
                        ("ad-size-lg", "Lg", h::AlertDialogSize::Lg),
                        ("ad-size-cover", "Cover", h::AlertDialogSize::Cover),
                    ]
                    .into_iter()
                    .map(|(key, label, size)| {
                        let open = self.demo_overlay(key);
                        overlay_demo(
                            open,
                            key,
                            label,
                            h::AlertDialog::new(format!("Size: {label}")).id(key)
                                .description("Every size shares one panel style.")
                                .is_open(open)
                                .size(size)
                                .child(h::AlertDialogCloseTrigger::new())
                                .on_open_change(cx.listener(
                                    move |this, v: &bool, _, cx| {
                                        this.set_demo_flag(key, *v);
                                        cx.notify();
                                    },
                                ))
                                .into_any_element(),
                            cx,
                        )
                    })
                    .collect()),
                ),
                (
                    "Statuses",
                    stretch_col([
                        ("ad-st-default", "Default", Color::Default),
                        ("ad-st-accent", "Accent", Color::Accent),
                        ("ad-st-success", "Success", Color::Success),
                        ("ad-st-warning", "Warning", Color::Warning),
                        ("ad-st-danger", "Danger", Color::Danger),
                    ]
                    .into_iter()
                    .map(|(key, label, status)| {
                        let open = self.demo_overlay(key);
                        overlay_demo(
                            open,
                            key,
                            label,
                            h::AlertDialog::new(format!("{label} status")).id(key)
                                .description("The status colours the icon above the title.")
                                .is_open(open)
                                .status(status)
                                .child(h::AlertDialogCloseTrigger::new())
                                .on_open_change(cx.listener(
                                    move |this, v: &bool, _, cx| {
                                        this.set_demo_flag(key, *v);
                                        cx.notify();
                                    },
                                ))
                                .into_any_element(),
                            cx,
                        )
                    })
                    .collect()),
                ),
                (
                    "Placements",
                    stretch_col([
                        ("ad-pl-auto", "Auto", h::ModalPlacement::Auto),
                        ("ad-pl-center", "Center", h::ModalPlacement::Center),
                        ("ad-pl-top", "Top", h::ModalPlacement::Top),
                        ("ad-pl-bottom", "Bottom", h::ModalPlacement::Bottom),
                    ]
                    .into_iter()
                    .map(|(key, label, placement)| {
                        let open = self.demo_overlay(key);
                        overlay_demo(
                            open,
                            key,
                            label,
                            h::AlertDialog::new(format!("Placement: {label}")).id(key)
                                .description("The panel keeps its own size.")
                                .is_open(open)
                                .placement(placement)
                                .child(h::AlertDialogCloseTrigger::new())
                                .on_open_change(cx.listener(
                                    move |this, v: &bool, _, cx| {
                                        this.set_demo_flag(key, *v);
                                        cx.notify();
                                    },
                                ))
                                .into_any_element(),
                            cx,
                        )
                    })
                    .collect()),
                ),
                (
                    "Backdrop Variants",
                    stretch_col(herogpui_core::Backdrop::ALL
                        .iter()
                        .map(|backdrop| {
                            let key: &'static str = match backdrop {
                                herogpui_core::Backdrop::Opaque => "ad-bd-opaque",
                                herogpui_core::Backdrop::Blur => "ad-bd-blur",
                                herogpui_core::Backdrop::Transparent => "ad-bd-transparent",
                            };
                            let open = self.demo_overlay(key);
                            overlay_demo(
                                open,
                                key,
                                backdrop.label(),
                                h::AlertDialog::new(format!("Backdrop: {}", backdrop.label())).id(key)
                                    .description("The scrim behind the panel.")
                                    .is_open(open)
                                    .backdrop(*backdrop)
                                    .child(h::AlertDialogCloseTrigger::new())
                                    .on_open_change(cx.listener(
                                        move |this, v: &bool, _, cx| {
                                            this.set_demo_flag(key, *v);
                                            cx.notify();
                                        },
                                    ))
                                    .into_any_element(),
                                cx,
                            )
                        })
                        .collect()),
                ),
                (
                    "Controlled State",
                    stretch_col(vec![overlay_demo(
                        self.demo_overlay("ad-controlled"),
                        "ad-controlled",
                        "Open (controlled)",
                        h::AlertDialog::new("Controlled").id("ad-controlled")
                            .description("The flag lives with the caller; closing reports through onOpenChange.")
                            .is_open(self.demo_overlay("ad-controlled"))
                            .child(h::AlertDialogCloseTrigger::new())
                            .on_open_change(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("ad-controlled", *v);
                                cx.notify();
                            }))
                            .into_any_element(),
                        cx,
                    )]),
                ),
                (
                    "Custom Icon",
                    stretch_col(vec![overlay_demo(
                        self.demo_overlay("ad-icon"),
                        "ad-icon",
                        "Open with a status icon",
                        h::AlertDialog::new("Heads up").id("ad-icon")
                            .description("The status picks the icon, so a warning dialog shows the warning glyph.")
                            .is_open(self.demo_overlay("ad-icon"))
                            .status(Color::Warning)
                            .child(h::AlertDialogCloseTrigger::new())
                            .on_open_change(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("ad-icon", *v);
                                cx.notify();
                            }))
                            .into_any_element(),
                        cx,
                    )]),
                ),
                (
                    "Custom Backdrop",
                    stretch_col(vec![overlay_demo(
                        self.demo_overlay("ad-custom-bd"),
                        "ad-custom-bd",
                        "Open with a blurred backdrop",
                        h::AlertDialog::new("Blurred").id("ad-custom-bd")
                            .description("The page behind the panel is blurred.")
                            .is_open(self.demo_overlay("ad-custom-bd"))
                            .backdrop(herogpui_core::Backdrop::Blur)
                            .child(h::AlertDialogCloseTrigger::new())
                            .on_open_change(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("ad-custom-bd", *v);
                                cx.notify();
                            }))
                            .into_any_element(),
                        cx,
                    )]),
                ),
                (
                    "Dismiss Behavior",
                    stretch_col(vec![overlay_demo(
                        self.demo_overlay("ad-dismiss"),
                        "ad-dismiss",
                        "Open a non-dismissable dialog",
                        h::AlertDialog::new("Confirm first").id("ad-dismiss")
                            .description("The backdrop and Escape are both inert; the composed X and the actions still close.")
                            .is_open(self.demo_overlay("ad-dismiss"))
                            .is_dismissible(false)
                            .is_keyboard_dismiss_disabled(true)
                            .child(h::AlertDialogCloseTrigger::new())
                            .on_open_change(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("ad-dismiss", *v);
                                cx.notify();
                            }))
                            .into_any_element(),
                        cx,
                    )]),
                ),
                (
                    "Close Methods",
                    stretch_col(vec![
                        overlay_demo(
                            self.demo_overlay("ad-close"),
                            "ad-close",
                            "Open (destructive confirm)",
                            h::AlertDialog::new("Delete for ever?").id("ad-close")
                                .description("A composed footer retires the built-in pair: the danger confirm and the cancel are ordinary Buttons the caller wires to close. The X is not composed here, so the corner slot is bare.")
                                .is_open(self.demo_overlay("ad-close"))
                                .footer_child(
                                    h::Button::new("ad-close-cancel")
                                        .label("Keep")
                                        .variant(Variant::Tertiary)
                                        .on_press(cx.listener(|this, _, _, cx| {
                                            this.set_demo_flag("ad-close", false);
                                            cx.notify();
                                        })),
                                )
                                .footer_child(
                                    h::Button::new("ad-close-confirm")
                                        .label("Delete")
                                        .variant(Variant::Danger)
                                        .on_press(cx.listener(|this, _, _, cx| {
                                            this.set_demo_flag("ad-close", false);
                                            cx.notify();
                                        })),
                                )
                                .on_open_change(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("ad-close", *v);
                                    cx.notify();
                                }))
                                .into_any_element(),
                            cx,
                        ),
                        overlay_demo(
                            self.demo_overlay("ad-pending"),
                            "ad-pending",
                            "Open (pending confirm)",
                            h::AlertDialog::new("Deploying").id("ad-pending")
                                .description("A composed footer Button carries the pending state: it shows a spinner and swallows the press while the action is in flight, so no close is reported; only the cancel closes.")
                                .is_open(self.demo_overlay("ad-pending"))
                                .footer_child(
                                    h::Button::new("ad-pending-cancel")
                                        .label("Cancel")
                                        .variant(Variant::Tertiary)
                                        .on_press(cx.listener(|this, _, _, cx| {
                                            this.set_demo_flag("ad-pending", false);
                                            cx.notify();
                                        })),
                                )
                                .footer_child(
                                    h::Button::new("ad-pending-confirm")
                                        .label("Deploy")
                                        .is_pending(true),
                                )
                                .on_open_change(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("ad-pending", *v);
                                    cx.notify();
                                }))
                                .into_any_element(),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Custom Animations",
                    stretch_col(vec![overlay_demo(
                        self.demo_overlay("ad-anim"),
                        "ad-anim",
                        "Open and watch the panel",
                        h::AlertDialog::new("Animated").id("ad-anim")
                            .description("The panel shrinks in from 105% over 250ms and leaves at 95% over 100ms.")
                            .is_open(self.demo_overlay("ad-anim"))
                            .child(h::AlertDialogCloseTrigger::new())
                            .on_open_change(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("ad-anim", *v);
                                cx.notify();
                            }))
                            .into_any_element(),
                        cx,
                    )]),
                ),
                (
                    "Custom Trigger",
                    stretch_col(vec![{
                        let open = self.demo_overlay("ad-custom");
                        overlay_min_h(
                            gpui::div()
                                .relative()
                                .flex()
                                .flex_col()
                        .items_center()
                        .w_full(),
                            open,
                            120.,
                        )
                        .child(
                            gpui::div()
                                .id("ad-custom-trigger")
                                .cursor_pointer()
                                .child(
                                    h::Chip::new()
                                        .color(Color::Danger)
                                        .variant(h::ChipVariant::Soft)
                                        .child(h::ChipLabel::new().child("Delete account")),
                                )
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.set_demo_flag("ad-custom", true);
                                    cx.notify();
                                })),
                        )
                        .child(
                            h::AlertDialog::new("Delete this account?").id("ad-custom")
                                .description("Any element can open an alert dialog; a composed CloseTrigger draws the corner X and a composed footer owns the danger confirm.")
                                .is_open(open)
                                .child(h::AlertDialogCloseTrigger::new())
                                .footer_child(
                                    h::Button::new("ad-custom-cancel")
                                        .label("Cancel")
                                        .variant(Variant::Tertiary)
                                        .on_press(cx.listener(|this, _, _, cx| {
                                            this.set_demo_flag("ad-custom", false);
                                            cx.notify();
                                        })),
                                )
                                .footer_child(
                                    h::Button::new("ad-custom-confirm")
                                        .label("Delete account")
                                        .variant(Variant::Danger)
                                        .on_press(cx.listener(|this, _, _, cx| {
                                            this.set_demo_flag("ad-custom", false);
                                            cx.notify();
                                        })),
                                )
                                .on_open_change(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("ad-custom", *v);
                                    cx.notify();
                                }))
                        )
                        .into_any_element()
                    }]),
                ),
            ],
            cx,
        )
    }

    pub fn page_drawer(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let is_open = self.drawer_open;
        component_doc_page!(
            "Drawer",
            crate::pages::Page::Drawer.description(),
            crate::pages::Page::Drawer.import_line(),
            vec![
                (
                    "Placement",
                    stretch_col(
                        [
                            ("dr-left", "Left", h::DrawerPlacement::Left),
                            ("dr-right", "Right", h::DrawerPlacement::Right),
                            ("dr-top", "Top", h::DrawerPlacement::Top),
                            ("dr-bottom", "Bottom", h::DrawerPlacement::Bottom),
                        ]
                        .into_iter()
                        .map(|(key, label, placement)| {
                            let open = self.demo_overlay(key);
                            overlay_demo(
                                open,
                                key,
                                label,
                                h::Drawer::new()
                                    .id(key)
                                    .is_open(open)
                                    .placement(placement)
                                    .title(format!("From the {label}"))
                                    .is_dismissible(true)
                                    .child(h::DrawerCloseTrigger::new())
                                    .child(gpui::div().child("The panel slides in along its edge."))
                                    .on_open_change(cx.listener(move |this, v: &bool, _, cx| {
                                        this.set_demo_flag(key, *v);
                                        cx.notify();
                                    }))
                                    .into_any_element(),
                                cx,
                            )
                        })
                        .collect()
                    ),
                ),
                (
                    "Non-Dismissable",
                    stretch_col(vec![overlay_demo(
                        self.demo_overlay("dr-no-dismiss"),
                        "dr-no-dismiss",
                        "Open a non-dismissable drawer",
                        h::Drawer::new()
                            .id("dr-no-dismiss")
                            .is_open(self.demo_overlay("dr-no-dismiss"))
                            .title("Finish first")
                            .is_dismissible(false)
                            .is_keyboard_dismiss_disabled(true)
                            .child(gpui::div().child("The backdrop and Escape are both inert."))
                            .footer_child(
                                h::Button::new("dr-no-dismiss-ok").label("Done").on_press(
                                    cx.listener(|this, _, _, cx| {
                                        this.set_demo_flag("dr-no-dismiss", false);
                                        cx.notify();
                                    }),
                                ),
                            )
                            .on_open_change(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("dr-no-dismiss", *v);
                                cx.notify();
                            }))
                            .into_any_element(),
                        cx,
                    )]),
                ),
                (
                    "Scrollable Content",
                    stretch_col(vec![overlay_demo(
                        self.demo_overlay("dr-scroll"),
                        "dr-scroll",
                        "Open a long drawer",
                        h::Drawer::new()
                            .id("dr-scroll")
                            .is_open(self.demo_overlay("dr-scroll"))
                            .title("Release notes")
                            .is_dismissible(true)
                            .child(h::DrawerCloseTrigger::new())
                            .child(gpui::div().flex().flex_col().gap(px(8.)).children(
                                (1..=20).map(|n| {
                                    gpui::div().child(format!("Change {n} of twenty."))
                                }),
                            ),)
                            .on_open_change(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("dr-scroll", *v);
                                cx.notify();
                            }))
                            .into_any_element(),
                        cx,
                    )]),
                ),
                (
                    "Controlled State",
                    stretch_col(vec![
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
                            self.demo_overlay("dr-controlled"),
                            "dr-controlled",
                            "Open (controlled)",
                            h::Drawer::new()
                                .id("dr-controlled")
                                .is_open(self.demo_overlay("dr-controlled"))
                                .title("Controlled")
                                .is_dismissible(true)
                                .child(h::DrawerCloseTrigger::new())
                                .child(gpui::div().child("Closing reports through onOpenChange."))
                                .on_open_change(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("dr-controlled", *v);
                                    cx.notify();
                                }))
                                .into_any_element(),
                            cx,
                        ),
                    ]),
                ),
                (
                    "With Form",
                    stretch_col(vec![overlay_demo(
                        self.demo_overlay("dr-form"),
                        "dr-form",
                        "Open a form drawer",
                        h::Drawer::new()
                            .id("dr-form")
                            .is_open(self.demo_overlay("dr-form"))
                            .title("New issue")
                            .is_dismissible(true)
                            .child(h::DrawerCloseTrigger::new())
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
                            .on_open_change(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("dr-form", *v);
                                cx.notify();
                            }))
                            .into_any_element(),
                        cx,
                    )]),
                ),
                (
                    "Navigation Drawer",
                    stretch_col(vec![overlay_demo(
                        self.demo_overlay("dr-nav"),
                        "dr-nav",
                        "Open the navigation",
                        h::Drawer::new()
                            .id("dr-nav")
                            .is_open(self.demo_overlay("dr-nav"))
                            .placement(h::DrawerPlacement::Left)
                            .title("Menu")
                            .is_dismissible(true)
                            .child(h::DrawerCloseTrigger::new())
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
                            .on_open_change(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("dr-nav", *v);
                                cx.notify();
                            }))
                            .into_any_element(),
                        cx,
                    )]),
                ),
                (
                    "Backdrop Variants",
                    stretch_col(
                        herogpui_core::Backdrop::ALL
                            .iter()
                            .map(|backdrop| {
                                let key: &'static str = match backdrop {
                                    herogpui_core::Backdrop::Opaque => "dr-bd-opaque",
                                    herogpui_core::Backdrop::Blur => "dr-bd-blur",
                                    herogpui_core::Backdrop::Transparent => "dr-bd-transparent",
                                };
                                let open = self.demo_overlay(key);
                                overlay_demo(
                                    open,
                                    key,
                                    backdrop.label(),
                                    h::Drawer::new()
                                        .id(key)
                                        .is_open(open)
                                        .backdrop(*backdrop)
                                        .title(format!("Backdrop: {}", backdrop.label()))
                                        .is_dismissible(true)
                                        .child(h::DrawerCloseTrigger::new())
                                        .child(gpui::div().child("The scrim behind the panel."))
                                        .on_open_change(cx.listener(
                                            move |this, v: &bool, _, cx| {
                                                this.set_demo_flag(key, *v);
                                                cx.notify();
                                            },
                                        ))
                                        .into_any_element(),
                                    cx,
                                )
                            })
                            .collect()
                    ),
                ),
                (
                    "Usage",
                    stretch_col(vec![overlay_min_h(
                        gpui::div()
                            .relative()
                            .flex()
                            .flex_col()
                            .items_center()
                            .w_full(),
                        is_open,
                        240.,
                    )
                    .child(
                        h::Button::new("dr-open")
                            .label("Open drawer")
                            .on_press(cx.listener(|this, _, _, cx| {
                                this.drawer_open = true;
                                cx.notify();
                            }),)
                    )
                    .child(
                        h::Drawer::new()
                            .id("dr-usage")
                            .is_open(is_open)
                            .title("Settings")
                            .placement(h::DrawerPlacement::Right)
                            .child(h::DrawerCloseTrigger::new())
                            .child(gpui::div().child("Panel content goes here."))
                            .footer_child(h::Button::new("dr-done").label("Done").on_press(
                                cx.listener(|this, _, _, cx| {
                                    this.drawer_open = false;
                                    cx.notify();
                                }),
                            ))
                            .on_close(cx.listener(|this, _, _, cx| {
                                this.drawer_open = false;
                                cx.notify();
                            })),
                    )
                    .into_any_element(),]),
                ),
            ],
            cx,
        )
    }

    pub fn page_modal(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let is_open = self.modal_open;
        let md_controlled = self.demo_overlay("md-controlled");
        let md_form = self.demo_overlay("md-form");
        let md_custom = self.demo_overlay("md-custom");
        let md_bd_custom = self.demo_overlay("md-bd-custom");
        let md_no_dismiss = self.demo_overlay("md-no-dismiss");
        let md_close = self.demo_overlay("md-close");
        let md_anim = self.demo_overlay("md-anim");
        component_doc_page!(
            "Modal",
            crate::pages::Page::Modal.description(),
            crate::pages::Page::Modal.import_line(),
            vec![
                (
                    "Sizes",
                    stretch_col([
                        ("md-size-xs", "Xs", h::ModalSize::Xs),
                        ("md-size-sm", "Sm", h::ModalSize::Sm),
                        ("md-size-md", "Md", h::ModalSize::Md),
                        ("md-size-lg", "Lg", h::ModalSize::Lg),
                        ("md-size-cover", "Cover", h::ModalSize::Cover),
                        ("md-size-full", "Full", h::ModalSize::Full),
                    ]
                    .into_iter()
                    .map(|(key, label, size)| {
                        let open = self.demo_overlay(key);
                        overlay_demo(
                            open,
                            key,
                            label,
                            h::Modal::new()
                                .id(key)
                                .is_open(open)
                                .size(size)
                                .title(format!("Size: {label}"))
                                .is_dismissible(true)
                                .child(h::ModalCloseTrigger::new())
                                .child(gpui::div().child("Every size shares one panel style."))
                                .on_open_change(cx.listener(
                                    move |this, v: &bool, _, cx| {
                                        this.set_demo_flag(key, *v);
                                        cx.notify();
                                    },
                                ))
                                .into_any_element(),
                            cx,
                        )
                    })
                    .collect()),
                ),
                (
                    "Placement",
                    stretch_col([
                        ("md-place-auto", "Auto", h::ModalPlacement::Auto),
                        ("md-place-center", "Center", h::ModalPlacement::Center),
                        ("md-place-top", "Top", h::ModalPlacement::Top),
                        ("md-place-bottom", "Bottom", h::ModalPlacement::Bottom),
                    ]
                    .into_iter()
                    .map(|(key, label, placement)| {
                        let open = self.demo_overlay(key);
                        overlay_demo(
                            open,
                            key,
                            label,
                            h::Modal::new()
                                .id(key)
                                .is_open(open)
                                .placement(placement)
                                .title(format!("Placement: {label}"))
                                .is_dismissible(true)
                                .child(h::ModalCloseTrigger::new())
                                .child(gpui::div().child("The panel keeps its own size."))
                                .on_open_change(cx.listener(
                                    move |this, v: &bool, _, cx| {
                                        this.set_demo_flag(key, *v);
                                        cx.notify();
                                    },
                                ))
                                .into_any_element(),
                            cx,
                        )
                    })
                    .collect()),
                ),
                (
                    "Scroll Behavior",
                    stretch_col([
                        ("md-scroll-inside", "Inside", h::ModalScroll::Inside),
                        ("md-scroll-outside", "Outside", h::ModalScroll::Outside),
                    ]
                    .into_iter()
                    .map(|(key, label, scroll)| {
                        let open = self.demo_overlay(key);
                        overlay_demo(
                            open,
                            key,
                            label,
                            h::Modal::new()
                                .id(key)
                                .is_open(open)
                                .scroll(scroll)
                                .title(format!("Scroll: {label}"))
                                .is_dismissible(true)
                                .child(h::ModalCloseTrigger::new())
                                .child(gpui::div().flex().flex_col().gap(px(8.)).children(
                                    (1..=12).map(|n| {
                                        gpui::div().child(format!("Paragraph {n} of twelve."))
                                    }),
                                ))
                                .on_open_change(cx.listener(
                                    move |this, v: &bool, _, cx| {
                                        this.set_demo_flag(key, *v);
                                        cx.notify();
                                    },
                                ))
                                .into_any_element(),
                            cx,
                        )
                    })
                    .collect()),
                ),
                (
                    "Controlled State",
                    stretch_col(vec![
                        para(
                            &format!(
                                "The flag lives with the caller: {}",
                                if md_controlled { "open" } else { "closed" }
                            ),
                            cx,
                        ),
                        overlay_demo(
                            self.demo_overlay("md-controlled"),
                            "md-controlled",
                            "Open (controlled)",
                            h::Modal::new()
                                .id("md-controlled")
                                .is_open(md_controlled)
                                .title("Controlled")
                                .is_dismissible(true)
                                .child(h::ModalCloseTrigger::new())
                                .child(gpui::div().child("Closing reports through onOpenChange."))
                                .on_open_change(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("md-controlled", *v);
                                    cx.notify();
                                }))
                                .into_any_element(),
                            cx,
                        ),
                    ]),
                ),
                (
                    "With Form",
                    stretch_col(vec![overlay_demo(
                        self.demo_overlay("md-form"),
                        "md-form",
                        "Open form modal",
                        h::Modal::new()
                            .id("md-form")
                            .is_open(md_form)
                            .title("Invite a teammate")
                            .is_dismissible(true)
                            .child(h::ModalCloseTrigger::new())
                            .child(
                                gpui::div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(12.))
                                    .child(
                                        h::TextField::new(self.demo_text("md-form-name", "", cx))
                                            .label("Name"),
                                    )
                                    .child(
                                        h::TextField::new(self.demo_text("md-form-email", "", cx))
                                            .label("Email")
                                            .input_type(h::InputType::Email),
                                    ),
                            )
                            .footer_child(
                                h::Button::new("md-form-send")
                                    .label("Send invite")
                                    .on_press(cx.listener(|this, _, _, cx| {
                                        this.set_demo_flag("md-form", false);
                                        cx.notify();
                                    })),
                            )
                            .on_open_change(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("md-form", *v);
                                cx.notify();
                            }))
                            .into_any_element(),
                        cx,
                    )]),
                ),
                (
                    "Custom Trigger",
                    stretch_col(vec![overlay_min_h(
                        gpui::div()
                            .relative()
                            .flex()
                            .flex_col()
                            .items_center()
                            .w_full(),
                        md_custom,
                        120.,
                    )
                        .child(
                            gpui::div()
                                .id("md-custom-trigger")
                                .cursor_pointer()
                                .child(h::Avatar::new("md-custom-avatar").name("Jane Doe"))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.set_demo_flag("md-custom", true);
                                    cx.notify();
                                })),
                        )
                        .child(
                            h::Modal::new()
                                .id("md-custom")
                                .is_open(md_custom)
                                .title("Jane Doe")
                                .is_dismissible(true)
                                .child(h::ModalCloseTrigger::new())
                                .child(gpui::div().child("Any element can open a modal."))
                                .on_open_change(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("md-custom", *v);
                                    cx.notify();
                                })),
                        )
                        .into_any_element()]),
                ),
                (
                    "Backdrop Variants",
                    stretch_col(herogpui_core::Backdrop::ALL
                        .iter()
                        .map(|backdrop| {
                            let key: &'static str = match backdrop {
                                herogpui_core::Backdrop::Opaque => "md-bd-opaque",
                                herogpui_core::Backdrop::Blur => "md-bd-blur",
                                herogpui_core::Backdrop::Transparent => "md-bd-transparent",
                            };
                            let open = self.demo_overlay(key);
                            overlay_demo(
                                open,
                                key,
                                backdrop.label(),
                                h::Modal::new()
                                    .id(key)
                                    .is_open(open)
                                    .backdrop(*backdrop)
                                    .title(format!("Backdrop: {}", backdrop.label()))
                                    .is_dismissible(true)
                                    .child(h::ModalCloseTrigger::new())
                                    .child(gpui::div().child("The scrim behind the panel."))
                                    .on_open_change(cx.listener(
                                        move |this, v: &bool, _, cx| {
                                            this.set_demo_flag(key, *v);
                                            cx.notify();
                                        },
                                    ))
                                    .into_any_element(),
                                cx,
                            )
                        })
                        .collect()),
                ),
                (
                    "Custom Backdrop", "`Backdrop::Blur` is the strongest variant the token set has; anything past it is the caller's own scrim.",
                    stretch_col(vec![
                        overlay_demo(
                            self.demo_overlay("md-bd-custom"),
                            "md-bd-custom",
                            "Open with a blurred backdrop",
                            h::Modal::new()
                                .id("md-bd-custom")
                                .is_open(md_bd_custom)
                                .backdrop(h::Backdrop::Blur)
                                .title("Blurred")
                                .is_dismissible(true)
                                .child(h::ModalCloseTrigger::new())
                                .child(gpui::div().child("The page behind is blurred."))
                                .on_open_change(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("md-bd-custom", *v);
                                    cx.notify();
                                }))
                                .into_any_element(),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Dismiss Behavior", "`isDismissible` decides whether the backdrop closes it; `isKeyboardDismissDisabled` decides whether Escape does.",
                    stretch_col(vec![
                        overlay_demo(
                            self.demo_overlay("md-no-dismiss"),
                            "md-no-dismiss",
                            "Open a non-dismissable modal",
                            h::Modal::new()
                                .id("md-no-dismiss")
                                .is_open(md_no_dismiss)
                                .title("Confirm first")
                                .is_dismissible(false)
                                .is_keyboard_dismiss_disabled(true)
                                .child(h::ModalCloseTrigger::new())
                                .child(gpui::div().child(
                                    "The backdrop and Escape are both inert; the composed X and the button still close.",
                                ))
                                .footer_child(
                                    h::Button::new("md-no-dismiss-ok").label("Got it").on_press(
                                        cx.listener(|this, _, _, cx| {
                                            this.set_demo_flag("md-no-dismiss", false);
                                            cx.notify();
                                        }),
                                    ),
                                )
                                .on_open_change(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("md-no-dismiss", *v);
                                    cx.notify();
                                }))
                                .into_any_element(),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Close Methods", "The close affordance is spelled by composition: the `Close Methods` example closes through footer buttons and composes no close trigger, so the corner slot stays bare. Every other example composes the built-in close trigger for the corner X.",
                    stretch_col(vec![
                        overlay_demo(
                            self.demo_overlay("md-close"),
                            "md-close",
                            "Open (no close trigger)",
                            h::Modal::new()
                                .id("md-close")
                                .is_open(md_close)
                                .title("Close me")
                                .is_dismissible(true)
                                .child(
                                    gpui::div().child(
                                        "The corner slot is bare; the footer button closes."
                                    )
                                )
                                .footer_child(
                                    h::Button::new("md-close-ok").label("Close").on_press(
                                        cx.listener(|this, _, _, cx| {
                                            this.set_demo_flag("md-close", false);
                                            cx.notify();
                                        }),
                                    ),
                                )
                                .on_open_change(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("md-close", *v);
                                    cx.notify();
                                }))
                                .into_any_element(),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Custom Animations", "The motion is the one the theme declares: the panel shrinks in from 105% over 250ms on `ease-out-quad` and leaves at 95% over 100ms. `Motion on` in the navbar switches it off, which is the `prefers-reduced-motion` path.",
                    stretch_col(vec![
                        overlay_demo(
                            self.demo_overlay("md-anim"),
                            "md-anim",
                            "Open and watch the panel",
                            h::Modal::new()
                                .id("md-anim")
                                .is_open(md_anim)
                                .title("Animated")
                                .is_dismissible(true)
                                .child(h::ModalCloseTrigger::new())
                                .child(gpui::div().child("Close and reopen to see it again."))
                                .on_open_change(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("md-anim", *v);
                                    cx.notify();
                                }))
                                .into_any_element(),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Usage",
                    stretch_col(vec![
                        overlay_min_h(
                            gpui::div()
                                .relative()
                                .flex()
                                .flex_col()
                        .items_center()
                        .w_full(),
                            is_open,
                            280.,
                        )
                        .child(
                            h::Button::new("md-open")
                                .label("Open modal")
                                .on_press(cx.listener(|this, _, _, cx| {
                                    this.modal_open = true;
                                    cx.notify();
                                })),
                        )
                        .child(
                            h::Modal::new()
                                .id("md-usage")
                                .is_open(is_open)
                                // `Modal.Icon` sits above the heading.
                                .icon(h::icons::MAIL)
                                .title("Create account")
                                .is_dismissible(true)
                                .child(h::ModalCloseTrigger::new())
                                .child(gpui::div().child("Sign up to get started with HeroGPUI."))
                                .footer_child(
                                    h::Button::new("md-cancel")
                                        .label("Cancel")
                                        .variant(Variant::Tertiary)
                                        .on_press(cx.listener(|this, _, _, cx| {
                                            this.modal_open = false;
                                            cx.notify();
                                        })),
                                )
                                .footer_child(h::Button::new("md-ok").label("Sign up").on_press(
                                    cx.listener(|this, _, _, cx| {
                                        this.modal_open = false;
                                        cx.notify();
                                    }),
                                ))
                                .on_close(cx.listener(|this, _, _, cx| {
                                    this.modal_open = false;
                                    cx.notify();
                                })),
                        )
                        .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_popover(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let is_open = self.popover_open;
        let po_following = self.demo_flag("po-following", false);
        let po_arrow_open = self.demo_overlay("po-arrow");
        let po_arrow_custom_open = self.demo_overlay("po-arrow-custom");
        let po_interactive_open = self.demo_overlay("po-interactive");
        let po_render_open = self.demo_overlay("po-render-function");
        let po_custom_styles_open = self.demo_overlay("po-custom-styles");
        let colors = cx.colors().clone();
        let custom_border = colors.border.alpha(0.8);
        let custom_surface = colors.surface.background.alpha(0.9);
        let custom_tint = colors
            .default
            .color
            .alpha(if cx.is_dark_theme() { 0.08 } else { 0.06 });
        let custom_shadow = cx.layout().overlay_shadow.clone();
        let usage_slot = gpui::div().relative().flex().flex_col().items_start();
        component_doc_page!(
            "Popover",
            crate::pages::Page::Popover.description(),
            crate::pages::Page::Popover.import_line(),
            vec![
                (
                    "Usage",
                    "Panel text and headings use a 20px line height independently of surrounding text.",
                    col(vec![overlay_min_h(usage_slot, is_open, 160.)
                        .child(
                            h::Popover::new(
                                gpui::div()
                                    .pr(px(96.))
                                    .child(
                                        h::Button::new("po-trigger")
                                            .label("Open popover")
                                            .variant(Variant::Secondary),
                                    ),
                            )
                            .is_open(is_open)
                            .title("Quick note")
                            .placement(h::PopoverPlacement::Bottom)
                            .padding(px(18.))
                            .child(gpui::div().child("Popovers are anchored to their trigger."))
                            .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                                set_popover_open(
                                    &mut this.popover_open,
                                    &mut this.demo_flags,
                                    "po-usage",
                                    *open,
                                );
                                cx.notify();
                            })),
                        )
                        .into_any_element()]),
                ),
                (
                    "With Arrow", "`PopoverArrow::new()` composes the arrow part: the built-in 12px curve follows the resolved side when the panel flips and preserves the configured offset. A custom child element takes the resolved position but no rotation: the placement transform is not reproducible on an arbitrary element (only `svg()` transforms at construction).",
                    col(vec![
                        gpui::div()
                            .relative()
                            .flex()
                            .flex_wrap()
                            .items_start()
                            .gap(px(24.))
                            .pl(px(48.))
                            .child(
                                overlay_min_h(
                                    gpui::div()
                                        .relative()
                                        .flex()
                                        .flex_col()
                                        .items_start(),
                                    po_arrow_open,
                                    160.,
                                )
                                .child(
                                h::Popover::new(
                                    h::Button::new("po-arrow-trigger")
                                        .label("Offset by 12px")
                                        .variant(Variant::Secondary),
                                )
                                .id("po-arrow")
                                .is_open(po_arrow_open)
                                .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                                    set_popover_open(
                                        &mut this.popover_open,
                                        &mut this.demo_flags,
                                        "po-arrow",
                                        *open,
                                    );
                                    cx.notify();
                                }))
                                .offset(px(12.))
                                .title("Anchored")
                                .child(gpui::div().child("Twelve pixels clear of the trigger."))
                                .child(h::PopoverArrow::new()),
                            )
                            )
                            .child(
                                overlay_min_h(
                                    gpui::div()
                                        .relative()
                                        .flex()
                                        .flex_col()
                                        .items_start(),
                                    po_arrow_custom_open,
                                    160.,
                                )
                                .child(
                                    h::Popover::new(
                                        h::Button::new("po-arrow-custom-trigger")
                                            .label("Custom arrow")
                                            .variant(Variant::Secondary),
                                    )
                                    .id("po-arrow-custom")
                                    .is_open(po_arrow_custom_open)
                                    .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                                        set_popover_open(
                                            &mut this.popover_open,
                                            &mut this.demo_flags,
                                            "po-arrow-custom",
                                            *open,
                                        );
                                        cx.notify();
                                    }))
                                    .offset(px(12.))
                                    .title("Custom arrow")
                                    .child(gpui::div().child("A caller-drawn element, not the curve."))
                                    .child(
                                        h::PopoverArrow::new().child(
                                            gpui::svg()
                                                .size(px(12.))
                                                .path(h::icons::TOOLTIP_ARROW)
                                                .text_color(cx.colors().accent.foreground),
                                        ),
                                    ),
                                )
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "Interactive Content",
                    col(vec![{
                        let interactive_slot = gpui::div()
                            .relative()
                            .flex()
                            .flex_col()
                            .items_start();
                        overlay_min_h(interactive_slot, po_interactive_open, 220.)
                        .child(
                            h::Popover::new(
                                gpui::div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    .pr(px(96.))
                                    .child(h::Avatar::new("po-avatar").name("Sarah Johnson").size(Size::Sm))
                                    .child(gpui::div().child("Sarah Johnson")),
                            )
                            .id("po-interactive")
                            .is_open(po_interactive_open)
                            .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                                set_popover_open(
                                    &mut this.popover_open,
                                    &mut this.demo_flags,
                                    "po-interactive",
                                    *open,
                                );
                                cx.notify();
                            }))
                            .title("Sarah Johnson")
                            .child(
                                gpui::div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(10.))
                                    .child(gpui::div().child("Design lead, Berlin"))
                                    .child(
                                        h::Button::new("po-follow")
                                            .label(if po_following {
                                                "Following"
                                            } else {
                                                "Follow"
                                            })
                                            .size(Size::Sm)
                                            .variant(if po_following {
                                                Variant::Secondary
                                            } else {
                                                Variant::Primary
                                            })
                                            .on_press(cx.listener(move |this, _, _, cx| {
                                                this.set_demo_flag("po-following", !po_following);
                                                cx.notify();
                                            })),
                                    ),
                            ),
                        )
                        .into_any_element()
                    }]),
                ),
                (
                    "Placement",
                    col(vec![gpui::div()
                            .relative()
                            .flex()
                            .flex_wrap()
                            .gap(px(24.))
                        .children(
                            [
                                ("po-pl-top", "Top", h::PopoverPlacement::Top),
                                ("po-pl-bottom", "Bottom", h::PopoverPlacement::Bottom),
                                ("po-pl-left", "Left", h::PopoverPlacement::Left),
                                ("po-pl-right", "Right", h::PopoverPlacement::Right),
                            ]
                            .into_iter()
                            .map(|(id, label, placement)| {
                                let open = self.demo_overlay(id);
                                let mut placement_slot = gpui::div()
                                    .relative()
                                    .flex()
                                    .flex_col()
                                    .items_start();
                                if open && matches!(placement, h::PopoverPlacement::Top) {
                                    placement_slot = placement_slot.pt(px(96.));
                                }
                                if open
                                    && matches!(
                                        placement,
                                        h::PopoverPlacement::Top | h::PopoverPlacement::Left
                                    )
                                {
                                    placement_slot = placement_slot.pl(px(104.));
                                }
                                overlay_min_h(placement_slot, open, 260.).child(
                                    h::Popover::new(
                                        h::Button::new(el_id(format!("{id}-trigger")))
                                            .label(label)
                                            .variant(Variant::Secondary)
                                            .size(Size::Sm),
                                    )
                                    .id(id)
                                    .is_open(open)
                                    .on_open_change(cx.listener(
                                        move |this, open: &bool, _, cx| {
                                            set_popover_open(
                                                &mut this.popover_open,
                                                &mut this.demo_flags,
                                                id,
                                                *open,
                                            );
                                            cx.notify();
                                        },
                                    ))
                                    .placement(placement)
                                    .title(label)
                                .child(gpui::div().child("Anchored to its trigger.")),
                                )
                            }),
                        )
                        .into_any_element()]),
                ),
                (
                    "Render Function", "The pinned Render Function replaces the Popover content's DOM element with a callback. This GPUI Popover has no content or state render callback, so the controlled panel records that limitation instead of faking an API.",
                    col(vec![
                        overlay_min_h(
                            gpui::div()
                                .relative()
                                .flex()
                                .flex_col()
                                .items_start(),
                            po_render_open,
                            160.,
                        )
                        .child(
                            h::Popover::new(
                                gpui::div()
                                    .pr(px(96.))
                                    .child(
                                        h::Button::new("po-render-function-trigger")
                                            .label("Click me")
                                            .variant(Variant::Secondary),
                                    ),
                            )
                            .id("po-render-function")
                            .is_open(po_render_open)
                            .on_open_change(cx.listener(
                                |this, open: &bool, _, cx| {
                                    set_popover_open(
                                        &mut this.popover_open,
                                        &mut this.demo_flags,
                                        "po-render-function",
                                        *open,
                                    );
                                    cx.notify();
                                },
                            ))
                            .child(
                                gpui::div()
                                    .child(
                                        gpui::div()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .child("Popover Title"),
                                    )
                                    .child(
                                        gpui::div()
                                            .mt(px(8.))
                                            .text_size(px(14.))
                                            .text_color(colors.muted)
                                            .child(
                                                "This is the popover content. You can put any content here.",
                                            ),
                                    ),
                            ),
                        )
                        .into_any_element(),
                    ]),
                ),
                (
                    "Custom Styles", "The pinned DOM styling is expressed here with public GPUI builders: `w`, `overflow_hidden`, `rounded`, `border_color`, `bg`, `shadow`, spacing, and `font_family`, using the active theme tokens. GPUI 0.2.2 has no DOM class, ring, gradient, or backdrop-blur hooks, so the styling belongs to the composed content element.",
                    col(vec![
                        overlay_min_h(
                            gpui::div()
                                .relative()
                                .flex()
                                .flex_col()
                                .items_start(),
                            po_custom_styles_open,
                            160.,
                        )
                        .child(
                            h::Popover::new(
                                gpui::div()
                                    .pr(px(96.))
                                    .child(
                                        h::Button::new("po-custom-styles-trigger")
                                            .label("Details")
                                            .variant(Variant::Secondary),
                                    ),
                            )
                            .id("po-custom-styles")
                            .is_open(po_custom_styles_open)
                            .on_open_change(cx.listener(
                                |this, open: &bool, _, cx| {
                                    set_popover_open(
                                        &mut this.popover_open,
                                        &mut this.demo_flags,
                                        "po-custom-styles",
                                        *open,
                                    );
                                    cx.notify();
                                },
                            ))
                            .child(
                                gpui::div()
                                    .relative()
                                    .w(px(224.))
                                    .overflow_hidden()
                                    .rounded(px(12.))
                                    .border_1()
                                    .border_color(custom_border)
                                    .bg(custom_surface)
                                    .shadow(custom_shadow)
                                    .p(px(16.))
                                    .child(
                                        gpui::div()
                                            .absolute()
                                            .top_0()
                                            .left_0()
                                            .right_0()
                                            .h(px(48.))
                                            .bg(custom_tint),
                                    )
                                    .child(
                                        gpui::div()
                                            .relative()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(colors.foreground)
                                            .child("Keyboard shortcuts"),
                                    )
                                    .child(
                                        gpui::div()
                                            .relative()
                                            .mt(px(12.))
                                            .flex()
                                            .flex_col()
                                            .gap(px(8.))
                                            .text_size(px(14.))
                                            .child(
                                                gpui::div()
                                                    .flex()
                                                    .justify_between()
                                                    .gap(px(16.))
                                                    .child(
                                                        gpui::div()
                                                            .text_color(colors.muted)
                                                            .child("Save"),
                                                    )
                                                    .child(
                                                        gpui::div()
                                                            .font_family(crate::app::MONO_FONT)
                                                            .text_color(colors.foreground)
                                                            .child("⌘ S"),
                                                    ),
                                            )
                                            .child(
                                                gpui::div()
                                                    .flex()
                                                    .justify_between()
                                                    .gap(px(16.))
                                                    .child(
                                                        gpui::div()
                                                            .text_color(colors.muted)
                                                            .child("Search"),
                                                    )
                                                    .child(
                                                        gpui::div()
                                                            .font_family(crate::app::MONO_FONT)
                                                            .text_color(colors.foreground)
                                                            .child("⌘ K"),
                                                    ),
                                            ),
                                    ),
                            ),
                        )
                        .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_toast(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let toast_closed = crate::app::toasts_closed(cx);
        component_doc_page!(
            "Toast",
            crate::pages::Page::Toast.description(),
            crate::pages::Page::Toast.import_line(),
            vec![
                (
                    "Usage",
                    row(vec![h::Button::new("toast-usage")
                        .label("Show a toast")
                        .variant(Variant::Secondary)
                        .on_press(|_, _, cx| {
                            h::Toast::new("Saved")
                                .description("Your changes are live.")
                                .closable(true)
                                .close_hover_bg(cx.colors().accent.soft())
                                .padding(px(18.))
                                .push(Some(std::time::Duration::from_secs(4)), cx);
                        })
                        .into_any_element()]),
                ),
                (
                    "Variants",
                    row(Color::ALL
                        .iter()
                        .map(|c| {
                            let color = *c;
                            h::Button::new(el_id(format!("toast-v-{c:?}")))
                                .label(c.label())
                                .variant(Variant::Secondary)
                                .size(Size::Sm)
                                .on_press(move |_, _, cx| {
                                    h::Toast::new(format!("{} toast", color.label()))
                                        .description("One variant per status colour.")
                                        .variant(color)
                                        .closable(true)
                                        .push(Some(std::time::Duration::from_secs(4)), cx);
                                })
                        })
                        .els()),
                ),
                (
                    "Placements", "The viewport decides where the stack sits. This gallery mounts one `ToastViewport` in its shell; each button moves it and pushes a toast into that corner.",
                    col(vec![
                        row([
                            ("TopStart", h::ToastPlacement::TopStart),
                            ("Top", h::ToastPlacement::Top),
                            ("TopEnd", h::ToastPlacement::TopEnd),
                            ("BottomStart", h::ToastPlacement::BottomStart),
                            ("Bottom", h::ToastPlacement::Bottom),
                            ("BottomEnd", h::ToastPlacement::BottomEnd),
                        ]
                        .into_iter()
                        .map(|(label, placement)| {
                            h::Button::new(el_id(format!("toast-pl-{label}")))
                                .label(label)
                                .variant(Variant::Tertiary)
                                .size(Size::Sm)
                                // Move the viewport, then push into it: six
                                // buttons that all pushed into the same corner
                                // showed nothing about `placement`.
                                .on_press(cx.listener(move |this, _, _, cx| {
                                    this.toast_placement = placement;
                                    h::Toast::new(label)
                                        .description("Pushed into the shell's viewport.")
                                        .closable(true)
                                        .push(Some(std::time::Duration::from_secs(3)), cx);
                                    cx.notify();
                                }))
                                .into_any_element()
                        })
                        .collect()),
                    ]),
                ),
                (
                    "Simple Toasts",
                    row(vec![h::Button::new("toast-simple")
                        .label("Title only")
                        .variant(Variant::Secondary)
                        .size(Size::Sm)
                        .on_press(|_, _, cx| {
                            h::Toast::new("Copied to the clipboard")
                                .push(Some(std::time::Duration::from_secs(3)), cx);
                        })
                        .into_any_element()]),
                ),
                (
                    "Custom Indicators", "The variant picks the glyph — success shows a tick, danger a crossed circle. `indicator` overrides it with any icon, and `indicator(None)` removes the glyph entirely.",
                    col(vec![
                        row(vec![
                            h::Button::new("toast-ind-success")
                                .label("Success")
                                .variant(Variant::Secondary)
                                .size(Size::Sm)
                                .on_press(|_, _, cx| {
                                    h::Toast::success("Deployed")
                                        .description("Build 412 is live.")
                                        .closable(true)
                                        .push(None, cx);
                                })
                                .into_any_element(),
                            h::Button::new("toast-ind-danger")
                                .label("Danger")
                                .variant(Variant::Secondary)
                                .size(Size::Sm)
                                .on_press(|_, _, cx| {
                                    h::Toast::error("Deploy failed")
                                        .description("Two tests did not pass.")
                                        .closable(true)
                                        .push(None, cx);
                                })
                                .into_any_element(),
                            h::Button::new("toast-ind-custom")
                                .label("Custom glyph")
                                .variant(Variant::Secondary)
                                .size(Size::Sm)
                                .on_press(|_, _, cx| {
                                    h::Toast::new("One new message")
                                        .description("From martha@example.com.")
                                        .indicator(SharedString::from(h::icons::MAIL))
                                        .push(None, cx);
                                })
                                .into_any_element(),
                            h::Button::new("toast-ind-none")
                                .label("No glyph")
                                .variant(Variant::Secondary)
                                .size(Size::Sm)
                                .on_press(|_, _, cx| {
                                    h::Toast::success("Saved").indicator(None).push(None, cx);
                                })
                                .into_any_element(),
                        ]),
                    ]),
                ),
                (
                    "Custom Toast Rendering", "A toast is a title, a description and a status. Anything richer is the caller's own panel: this example renders its own body inside the queue's slot.",
                    col(vec![
                        row(vec![h::Button::new("toast-custom")
                            .label("Push a two-line toast")
                            .variant(Variant::Secondary)
                            .size(Size::Sm)
                            .on_press(|_, _, cx| {
                                h::Toast::new("Jane invited you")
                                    .description("Acme workspace \u{2014} Owner")
                                    .variant(Color::Accent)
                                    .closable(true)
                                    .push(Some(std::time::Duration::from_secs(5)), cx);
                            })
                            .into_any_element()]),
                    ]),
                ),
                (
                    "Promise & Loading", "`toast.promise` shows a loading toast while the work runs, then replaces it. `Toast::loading` is the pending half: a spinner, and no timeout, so it waits to be closed.",
                    col(vec![
                        row(vec![h::Button::new("toast-promise")
                            .label("Upload a file")
                            .variant(Variant::Secondary)
                            .size(Size::Sm)
                            .on_press(|_, window, cx| {
                                let id = h::Toast::loading("Uploading\u{2026}")
                                    .description("document.pdf")
                                    .push(None, cx);
                                // The resolution replaces the pending toast,
                                // which is what v3's promise helper does.
                                window
                                    .spawn(cx, async move |cx| {
                                        cx.background_executor()
                                            .timer(std::time::Duration::from_millis(1500))
                                            .await;
                                        cx.update(|_window, cx| {
                                            h::dismiss_toast(id, cx);
                                            h::Toast::success("Uploaded")
                                                .description("document.pdf \u{2014} 1 KB")
                                                .closable(true)
                                                .action("View", |_| {})
                                                .push(None, cx);
                                        })
                                        .ok();
                                    })
                                    .detach();
                            })
                            .into_any_element()]),
                    ]),
                ),
                (
                    "Callbacks", "`onClose` runs however the toast goes -- dismissed by hand or timed out -- so the count follows the toast, not the button.",
                    col(vec![
                        para(&format!("Toasts closed so far: {toast_closed}"), cx),
                        row(vec![h::Button::new("toast-callback")
                            .label("Push a closable toast")
                            .variant(Variant::Secondary)
                            .size(Size::Sm)
                            .on_press(|_, _, cx| {
                                h::Toast::new("Dismiss me")
                                    .description("Or wait four seconds.")
                                    .closable(true)
                                    .on_close(crate::app::bump_toast_closed)
                                    .push(None, cx);
                            })
                            .into_any_element()]),
                    ]),
                ),
                (
                    "Custom Queues", "`maxVisibleToasts` caps visibility without dropping overflow: the ones past the cap wait their turn. Push four and watch one queue.",
                    col(vec![
                        row(vec![h::Button::new("toast-queue")
                            .label("Push four")
                            .variant(Variant::Secondary)
                            .size(Size::Sm)
                            .on_press(|_, _, cx| {
                                for n in 1..=4 {
                                    h::Toast::new(format!("Message {n}"))
                                        .description("Three are visible at a time.")
                                        .push(Some(std::time::Duration::from_secs(3)), cx);
                                }
                            })
                            .into_any_element()]),
                    ]),
                ),
                (
                    "Setup", "A toast needs a viewport somewhere in the tree. This gallery mounts one in its shell, which is why every page can push.",
                    col(vec![
                        crate::pages::code_block(TOAST_SETUP, cx),
                    ]),
                ),
                (
                    "Push a toast",
                    row(Color::ALL
                        .iter()
                        .map(|c| {
                            let color = *c;
                            h::Button::new(el_id(format!("toast-{c:?}")))
                                .label(c.label())
                                .variant(Variant::Secondary)
                                .size(Size::Sm)
                                .on_press(move |_, _, cx| {
                                    h::Toast::new(format!("{} toast", color.label()))
                                        .description("Pushed from the gallery.")
                                        .variant(color)
                                        .closable(true)
                                        .push(Some(std::time::Duration::from_secs(4)), cx);
                                })
                        })
                        .els()),
                ),
            ],
            cx,
        )
    }

    pub fn page_tooltip(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Tooltip",
            crate::pages::Page::Tooltip.description(),
            crate::pages::Page::Tooltip.import_line(),
            vec![
                (
                    "With Arrow",
                    row(vec![
                        h::Tooltip::new("With an arrow")
                            .show_arrow(true)
                            // `offset` is the gap between trigger and panel.
                            .offset(px(10.))
                            .child(
                                h::Button::new("tt-arrow-on")
                                    .label("Arrow")
                                    .variant(Variant::Secondary),
                            )
                            .into_any_element(),
                        h::Tooltip::new("Without one")
                            .show_arrow(false)
                            .child(
                                h::Button::new("tt-arrow-off")
                                    .label("No arrow")
                                    .variant(Variant::Secondary),
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "Custom Triggers",
                    row(vec![
                        h::Tooltip::new("Jane Doe")
                            .delay(0)
                            .show_arrow(true)
                            .child(h::Avatar::new("tt-avatar").name("Jane Doe").size(Size::Sm))
                            .into_any_element(),
                        h::Tooltip::new("Verified account")
                            .delay(0)
                            .child(
                                h::Chip::new()
                                    .color(Color::Success)
                                    .variant(h::ChipVariant::Soft)
                                    .child(h::ChipLabel::new().child("Verified")),
                            )
                            .into_any_element(),
                        h::Tooltip::new("What is this?")
                            .delay(0)
                            .child(icon(h::icons::SEARCH, cx))
                            .into_any_element(),
                        h::Tooltip::new("Tab to me")
                            .delay(0)
                            // `trigger="focus"`: the pointer does nothing and
                            // keyboard focus is what opens it.
                            .trigger(h::TooltipTrigger::Focus)
                            .child(
                                h::Button::new("tt-focus-only")
                                    .label("Focus only")
                                    .variant(Variant::Secondary),
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "Usage",
                    row(vec![
                        h::Tooltip::new("Above the trigger")
                            .placement(h::TooltipPlacement::Top)
                            .child(
                                h::Button::new("tt-top")
                                    .label("Top")
                                    .variant(Variant::Secondary),
                            )
                            .into_any_element(),
                        h::Tooltip::new("Below the trigger")
                            .placement(h::TooltipPlacement::Bottom)
                            .child(
                                h::Button::new("tt-bottom")
                                    .label("Bottom")
                                    .variant(Variant::Secondary),
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "Placement",
                    row(h::TooltipPlacement::ALL
                        .iter()
                        .map(|p| {
                            h::Tooltip::new(p.label())
                                .placement(*p)
                                .show_arrow(true)
                                .child(
                                    h::Button::new(el_id(format!("tip-{}", p.label())))
                                        .label(p.label())
                                        .variant(Variant::Secondary)
                                        .into_any_element(),
                                )
                                .into_any_element()
                        })
                        .collect()),
                ),
                (
                    "Delay",
                    row(vec![
                        h::Tooltip::new("Opens at once")
                            .delay(0)
                            .child(
                                h::Button::new("tip-instant")
                                    .label("No delay")
                                    .into_any_element(),
                            )
                            .into_any_element(),
                        h::Tooltip::new("Waits half a second")
                            .delay(500)
                            .close_delay(0)
                            .child(h::Button::new("tip-slow").label("500ms").into_any_element())
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
}
