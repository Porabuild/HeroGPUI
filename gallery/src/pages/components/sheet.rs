//! Sheet examples for HeroUI v3.2.7.
use super::*;

impl Gallery {
    pub fn page_sheet(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let profession_query = self.demo_text("sheet-profession-search", "", cx);
        let profession_filter = profession_query.read(cx).value().to_lowercase();
        let professions: Vec<h::ListBoxItem> = [
            "Engineer",
            "Designer",
            "Teacher",
            "Doctor",
            "Architect",
            "Scientist",
            "Writer",
            "Musician",
        ]
        .into_iter()
        .filter(|name| name.to_lowercase().contains(&profession_filter))
        .map(|name| h::ListBoxItem::new(name, name))
        .collect();
        let selected_profession = self.demo_text_value("sheet-profession");
        let controlled_open = self.demo_overlay("sheet-controlled");
        component_doc_page!(
            "Sheet",
            crate::pages::Page::Sheet.description(),
            crate::pages::Page::Sheet.import_line(),
            vec![
                // v3's Usage: secondary "Open Sheet" trigger, "Sheet
                // Title", the muted drag hint, and Cancel/Confirm footer
                // buttons that both close.
                (
                    "Usage",
                    col(vec![h::Sheet::new("sheet-basic")
                        .title("Sheet Title")
                        .default_open(self.demo_overlay("sheet-basic"))
                        .trigger(h::Button::new("sheet-basic-trigger")
                            .label("Open Sheet")
                            .variant(Variant::Secondary))
                        .close_trigger(
                            h::CloseButton::new("sheet-basic-close").aria_label("Dismiss details")
                        )
                        .child(gpui::div()
                            .text_size(px(14.))
                            .text_color(cx.colors().muted)
                            .child("This is a bottom sheet built with smooth drag-to-dismiss animations. Try dragging the handle down to close, or use the close button."))
                        .close_child(h::Button::new("sheet-basic-cancel")
                            .label("Cancel")
                            .variant(Variant::Secondary))
                        .close_child(h::Button::new("sheet-basic-confirm")
                            .label("Confirm")
                            .variant(Variant::Secondary))
                        .into_any_element()])
                ),
                (
                    "Placements",
                    col([
                        (
                            "sheet-bottom",
                            "sheet-bottom-open",
                            "sheet-bottom-close",
                            h::DrawerPlacement::Bottom
                        ),
                        (
                            "sheet-top",
                            "sheet-top-open",
                            "sheet-top-close",
                            h::DrawerPlacement::Top
                        ),
                        (
                            "sheet-left",
                            "sheet-left-open",
                            "sheet-left-close",
                            h::DrawerPlacement::Left
                        ),
                        (
                            "sheet-right",
                            "sheet-right-open",
                            "sheet-right-close",
                            h::DrawerPlacement::Right
                        ),
                    ]
                    .into_iter()
                    .map(|(key, trigger, close, placement)| h::Sheet::new(key)
                        .title(format!("{placement:?} sheet"))
                        .placement(placement)
                        .default_open(self.demo_overlay(key))
                        .trigger(h::Button::new(trigger).label(format!("Open {placement:?}")))
                        .close_trigger(h::CloseButton::new(close))
                        .child("Drag the handle or use the close control.")
                        .into_any_element())
                    .collect())
                ),
                (
                    "Snap Points",
                    col(vec![h::Sheet::new("sheet-snap")
                        .title("Snap Points")
                        .default_open(self.demo_overlay("sheet-snap"))
                        .trigger(h::Button::new("sheet-snap-trigger").label("Open sheet"))
                        .close_trigger(h::CloseButton::new("sheet-snap-close"))
                        .child("Drag the handle to expand, change snap position, or dismiss.")
                        .close_child(h::Button::new("sheet-snap-done").label("Done"))
                        .snap_points([160., 320.])
                        .into_any_element()])
                ),
                (
                    "Snap Points (Sequential)",
                    col(vec![h::Sheet::new("sheet-sequential")
                        .title("Snap Points (Sequential)")
                        .default_open(self.demo_overlay("sheet-sequential"))
                        .trigger(h::Button::new("sheet-sequential-trigger").label("Open sheet"))
                        .close_trigger(h::CloseButton::new("sheet-sequential-close"))
                        .child("Drag the handle to expand, change snap position, or dismiss.")
                        .close_child(h::Button::new("sheet-sequential-done").label("Done"))
                        .snap_points([140., 280., 420.])
                        .into_any_element()])
                ),
                (
                    "Snap Points (Custom Fade)",
                    col(vec![h::Sheet::new("sheet-fade")
                        .title("Snap Points (Custom Fade)")
                        .default_open(self.demo_overlay("sheet-fade"))
                        .trigger(h::Button::new("sheet-fade-trigger").label("Open sheet"))
                        .close_trigger(h::CloseButton::new("sheet-fade-close"))
                        .child("Drag the handle to expand, change snap position, or dismiss.")
                        .close_child(h::Button::new("sheet-fade-done").label("Done"))
                        .snap_points([
                            h::SheetSnapPoint::Fraction(0.25),
                            h::SheetSnapPoint::Fraction(0.5)
                        ])
                        .into_any_element()])
                ),
                (
                    "Backdrop Variants",
                    col([
                        (
                            "sheet-opaque",
                            "sheet-opaque-open",
                            "sheet-opaque-close",
                            h::Backdrop::Opaque
                        ),
                        (
                            "sheet-blur",
                            "sheet-blur-open",
                            "sheet-blur-close",
                            h::Backdrop::Blur
                        ),
                        (
                            "sheet-transparent",
                            "sheet-transparent-open",
                            "sheet-transparent-close",
                            h::Backdrop::Transparent
                        ),
                    ]
                    .into_iter()
                    .map(|(key, trigger, close, backdrop)| h::Sheet::new(key)
                        .title(format!("{backdrop:?} backdrop"))
                        .backdrop(backdrop)
                        .default_open(self.demo_overlay(key))
                        .trigger(h::Button::new(trigger).label(format!("Open {backdrop:?}")))
                        .close_trigger(h::CloseButton::new(close))
                        .child("Backdrop appearance.")
                        .into_any_element())
                    .collect())
                ),
                (
                    "Non-Dismissable",
                    col(vec![h::Sheet::new("sheet-non-dismissable")
                        .title("Non-Dismissable")
                        .default_open(self.demo_overlay("sheet-non-dismissable"))
                        .trigger(
                            h::Button::new("sheet-non-dismissable-trigger").label("Open sheet")
                        )
                        .close_trigger(h::CloseButton::new("sheet-non-dismissable-close"))
                        .child("Drag the handle to expand, change snap position, or dismiss.")
                        .close_child(h::Button::new("sheet-non-dismissable-done").label("Done"))
                        .is_dismissible(false)
                        .into_any_element()])
                ),
                (
                    "Scrollable Content",
                    col(vec![h::Sheet::new("sheet-scroll")
                        .title("Scrollable Content")
                        .default_open(self.demo_overlay("sheet-scroll"))
                        .trigger(h::Button::new("sheet-scroll-trigger").label("Open sheet"))
                        .close_trigger(h::CloseButton::new("sheet-scroll-close"))
                        .child("Drag the handle to expand, change snap position, or dismiss.")
                        .close_child(h::Button::new("sheet-scroll-done").label("Done"))
                        .snap_points([200.])
                        .child(
                            gpui::div().flex().flex_col().children(
                                (0..40).map(|index| gpui::div()
                                    .h(px(32.))
                                    .child(format!("Row {index}")))
                            )
                        )
                        .into_any_element()])
                ),
                (
                    "With Form",
                    col(vec![h::Sheet::new("sheet-form")
                        .title("With Form")
                        .default_open(self.demo_overlay("sheet-form"))
                        .trigger(h::Button::new("sheet-form-trigger").label("Open sheet"))
                        .close_trigger(h::CloseButton::new("sheet-form-close"))
                        .child("Drag the handle to expand, change snap position, or dismiss.")
                        .close_child(h::Button::new("sheet-form-done").label("Done"))
                        .child(
                            h::TextField::new(self.demo_text("sheet-name", "", cx))
                                .label("Name")
                                .variant(FieldVariant::Secondary)
                        )
                        .into_any_element()])
                ),
                (
                    "Detached",
                    col(vec![h::Sheet::new("sheet-detached")
                        .title("Detached")
                        .default_open(self.demo_overlay("sheet-detached"))
                        .trigger(h::Button::new("sheet-detached-trigger").label("Open sheet"))
                        .close_trigger(h::CloseButton::new("sheet-detached-close"))
                        .child("Drag the handle to expand, change snap position, or dismiss.")
                        .close_child(h::Button::new("sheet-detached-done").label("Done"))
                        .is_detached(true)
                        .into_any_element()])
                ),
                (
                    "Controlled State",
                    col(vec![h::Sheet::new("sheet-controlled")
                        .title("Controlled State")
                        .default_open(self.demo_overlay("sheet-controlled"))
                        .trigger(h::Button::new("sheet-controlled-trigger").label("Open sheet"))
                        .close_trigger(h::CloseButton::new("sheet-controlled-close"))
                        .child("Drag the handle to expand, change snap position, or dismiss.")
                        .close_child(h::Button::new("sheet-controlled-done").label("Done"))
                        .is_open(controlled_open)
                        .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                            this.set_demo_flag("sheet-controlled", *open);
                            cx.notify();
                        }))
                        .into_any_element()])
                ),
                (
                    "Handle Only",
                    col(vec![h::Sheet::new("sheet-handle")
                        .title("Handle Only")
                        .default_open(self.demo_overlay("sheet-handle"))
                        .trigger(h::Button::new("sheet-handle-trigger").label("Open sheet"))
                        .close_trigger(h::CloseButton::new("sheet-handle-close"))
                        .child("Drag the handle to expand, change snap position, or dismiss.")
                        .close_child(h::Button::new("sheet-handle-done").label("Done"))
                        .snap_points([180.])
                        .handle(
                            gpui::div()
                                .h(px(4.8))
                                .w(px(32.))
                                .rounded_full()
                                .bg(cx.colors().muted)
                        )
                        .into_any_element()])
                ),
                (
                    "Nested Sheets",
                    col(vec![h::Sheet::new("sheet-nested")
                        .title("Nested Sheets")
                        .default_open(self.demo_overlay("sheet-nested"))
                        .trigger(h::Button::new("sheet-nested-trigger").label("Open sheet"))
                        .close_trigger(h::CloseButton::new("sheet-nested-close"))
                        .child("Drag the handle to expand, change snap position, or dismiss.")
                        .close_child(h::Button::new("sheet-nested-done").label("Done"))
                        .nested_child(
                            h::Sheet::new("sheet-nested-child")
                                .title("Nested sheet")
                                .trigger(
                                    h::Button::new("sheet-child-trigger")
                                        .label("Open nested sheet")
                                )
                                .close_trigger(h::CloseButton::new("sheet-child-close"))
                                .child("Focus returns to the parent when this sheet closes.")
                        )
                        .into_any_element()])
                ),
                (
                    "Professions Picker",
                    col(vec![h::Sheet::new("sheet-professions")
                        .title("Professions Picker")
                        .default_open(self.demo_overlay("sheet-professions"))
                        .trigger(h::Button::new("sheet-professions-trigger").label(
                            if selected_profession.is_empty() {
                                "Choose profession".to_owned()
                            } else {
                                selected_profession.clone()
                            }
                        ))
                        .is_open(self.demo_overlay("sheet-professions"))
                        .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                            this.set_demo_flag("sheet-professions", *open);
                            cx.notify();
                        }))
                        .close_trigger(h::CloseButton::new("sheet-professions-close"))
                        .child("Drag the handle to expand, change snap position, or dismiss.")
                        .close_child(h::Button::new("sheet-professions-done").label("Done"))
                        .snap_points([355.])
                        .child(
                            h::SearchField::new(profession_query.clone())
                                .placeholder("Search professions")
                        )
                        .child(
                            h::ListBox::new("sheet-profession-list", professions.clone())
                                .row_height(px(36.))
                                .on_action(cx.listener(|this, key: &SharedString, _, cx| {
                                    this.set_demo_text_value("sheet-profession", key.to_string());
                                    this.set_demo_flag("sheet-professions", false);
                                    cx.notify();
                                }))
                        )
                        .into_any_element()])
                ),
            ],
            cx
        )
    }
}
