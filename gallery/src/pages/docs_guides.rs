//! The two getting-started guides that describe *how* to use the library:
//! v3's Styling page and its Design Principles page.
//!
//! Both are ported rather than paraphrased. Where a principle cannot hold in a
//! gpui port — there is no accessibility tree, so React Aria's ARIA layer has
//! no analogue — the page says so instead of claiming it.

use gpui::{prelude::*, px, App, Context};
use herogpui_components as h;
use herogpui_core::{Size, Variant};
use herogpui_theme::ActiveTheme;

use crate::app::Gallery;
use crate::pages::{code_block, doc_page, para};

/// A two-column row: the styling need, and the route that answers it.
fn mapping(need: &str, route: &str, cx: &App) -> gpui::AnyElement {
    let colors = cx.colors();
    gpui::div()
        .flex()
        .items_start()
        .gap(px(12.))
        .child(
            gpui::div()
                .w(px(230.))
                .flex_shrink_0()
                .text_size(px(13.5))
                .line_height(px(22.))
                .font_family(crate::app::MONO_FONT)
                .text_color(colors.accent.color)
                .child(need.to_owned()),
        )
        .child(
            gpui::div()
                .flex_1()
                // Without `min_w_0` a flex child does not shrink below its
                // longest line, so the text is clipped instead of wrapped.
                .min_w_0()
                .text_size(px(14.))
                .line_height(px(23.))
                .text_color(colors.foreground)
                .child(route.to_owned()),
        )
        .into_any_element()
}

fn stack(children: Vec<gpui::AnyElement>) -> gpui::AnyElement {
    gpui::div()
        .flex()
        .flex_col()
        .gap(px(12.))
        .children(children)
        .into_any_element()
}

impl Gallery {
    pub fn page_styling(&mut self, cx: &mut Context<'_, Self>) -> gpui::AnyElement {
        let hierarchy = gpui::div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(12.))
            .child(h::Button::new("sty-primary").label("Save"))
            .child(
                h::Button::new("sty-secondary")
                    .label("Edit")
                    .variant(Variant::Secondary),
            )
            .child(
                h::Button::new("sty-tertiary")
                    .label("Cancel")
                    .variant(Variant::Tertiary),
            )
            .child(
                h::Button::new("sty-danger")
                    .label("Delete")
                    .variant(Variant::Danger),
            )
            .into_any_element();

        // The "state-based styling" demo: hover and press are component states
        // here, not selectors, so the only way to show them is to let the reader
        // use the control.
        let states = gpui::div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(12.))
            .child(h::Button::new("sty-hover").label("Hover me"))
            .child(h::Button::new("sty-press").label("Press me").size(Size::Lg))
            .child(
                h::Button::new("sty-disabled")
                    .label("Disabled")
                    .is_disabled(true),
            )
            .into_any_element();

        doc_page(
            "Styling",
            "HeroGPUI styles a component three ways: typed props for the documented variants, \
             theme tokens for shared values, and render closures for state. There are no CSS \
             classes in gpui — this page is where each styling route comes out.",
            "",
            vec![
                (
                    "Variants carry the intent",
                    stack(vec![
                        para(
                            "Reach for the documented builder first. Every variant, size and \
                             colour is a typed builder method, so the available choices are \
                             explicit at the call site and checked at compile time. The \
                             hierarchy below is the meaning of each button variant.",
                            cx,
                        ),
                        hierarchy,
                        code_block(STYLING_VARIANTS, cx),
                    ]),
                ),
                (
                    "How to style",
                    stack(vec![
                        para(
                            "Most styling needs do one of four things, and each has its own \
                             route here. Nothing is styled by string.",
                            cx,
                        ),
                        mapping(
                            "Shared appearance",
                            "Theme recipes: put height, padding, radius and semantic colours on \
                             `ThemeBuilder::components` / `.recipe(\"name\")`. Instance `sx` is \
                             for placement, width and flex.",
                            cx,
                        ),
                        mapping(
                            "Full width",
                            "Layout: wrap the control in a styled div, or use the prop the \
                             component documents for it (`full_width`).",
                            cx,
                        ),
                        mapping(
                            "A themed colour",
                            "Colour: read the token — `cx.colors()`, `cx.role(Color::Accent)` — \
                             so the value follows the active theme instead of pinning a shade.",
                            cx,
                        ),
                        mapping(
                            "A bigger corner",
                            "Radius: `util::soft_radius(cx)` and its siblings, one per radius \
                             step. There is no single control radius; each component names its \
                             own.",
                            cx,
                        ),
                        mapping(
                            "Spacing and type",
                            "Spacing and type: gpui's `Styled` methods on the element you own, \
                             or the component's `sx` slot for an override that wins. Inside a \
                             component, they are the component's business.",
                            cx,
                        ),
                    ]),
                ),
                (
                    "State-based styling",
                    stack(vec![
                        para(
                            "States are part of the Rust API: `.hover()` takes a closure, a \
                             press is an animation, and `is_disabled` is a prop. Use the \
                             controls below — a screenshot of this page cannot show them.",
                            cx,
                        ),
                        states,
                        code_block(STYLING_STATES, cx),
                    ]),
                ),
                (
                    "Render closures",
                    stack(vec![
                        para(
                            "Render closures hand you the state and let you draw the part \
                              yourself. The component computes the value it already knows and \
                              passes it in, so a caller never re-derives it.",
                            cx,
                        ),
                        code_block(STYLING_RENDER, cx),
                    ]),
                ),
                (
                    "Wrapper components",
                    stack(vec![
                        para(
                            "Fix a set of props in place by returning a configured builder from \
                             a function. Builders are plain Rust values — no new type, and \
                             every remaining builder still available to the caller.",
                            cx,
                        ),
                        code_block(STYLING_WRAPPER, cx),
                    ]),
                ),
                (
                    "Style through the Rust API",
                    stack(vec![para(
                        "The `Button` struct and its `variant` method select the look, part \
                             builders such as `CardHeader` place the pieces, and theme tokens \
                             supply the values. The map is one module per component, one struct \
                             per component, and a builder per documented prop — \
                             `herogpui::components::button::Button`, `Button::variant`, \
                             `Card::header`.",
                        cx,
                    )]),
                ),
            ],
            cx,
        )
    }

    pub fn page_design_principles(&mut self, cx: &mut Context<'_, Self>) -> gpui::AnyElement {
        let semantic = gpui::div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(12.))
            .child(h::Button::new("dp-primary").label("Save"))
            .child(
                h::Button::new("dp-secondary")
                    .label("Edit")
                    .variant(Variant::Secondary),
            )
            .child(
                h::Button::new("dp-tertiary")
                    .label("Cancel")
                    .variant(Variant::Tertiary),
            )
            .into_any_element();

        let disclosure = gpui::div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(12.))
            .child(h::Button::new("dp-l1").label("Click me"))
            .child(
                h::Button::new("dp-l2")
                    .size(Size::Lg)
                    .child(
                        gpui::svg()
                            .size(px(16.))
                            .path(h::icons::CHECK)
                            .text_color(cx.colors().accent.foreground),
                    )
                    .child("Submit"),
            )
            .child(h::Button::new("dp-l3").label("Submitting").is_pending(true))
            .into_any_element();

        doc_page(
            "Design Principles",
            "Use these ten principles when choosing components, structuring state and shaping an \
             interface. They cover the decisions that keep a HeroGPUI application clear as it \
             grows.",
            "",
            vec![
                (
                    "1. Semantic intent over visual style",
                    stack(vec![
                        para(
                            "Variants are named for what they mean, not for how they look: \
                              primary is the one action that moves forward, secondary is an \
                              alternative, tertiary is dismissive, danger is destructive. The \
                              names communicate hierarchy without relying on colour alone.",
                            cx,
                        ),
                        semantic,
                        code_block(DP_SEMANTIC, cx),
                    ]),
                ),
                (
                    "2. Accessibility as foundation",
                    stack(vec![
                        para(
                            "Design keyboard and focus behaviour into every interactive flow. A \
                              desktop GPUI surface has no screen-reader tree to write into, and \
                              the library records that gap honestly instead of shipping an \
                              `aria_label` builder that would go nowhere — a promise the library \
                              cannot keep is worse than no promise.",
                            cx,
                        ),
                        para(
                            "What every component does guarantee is the part that is behaviour: \
                              focus handling, keyboard navigation, `Escape` to dismiss, arrow \
                              keys through a menu, typing into a date field segment.",
                            cx,
                        ),
                    ]),
                ),
                (
                    "3. Composition over configuration",
                    stack(vec![
                        para(
                            "Compose parts through named builder slots: `ModalCloseTrigger`, \
                              `CardHeader`, `InputGroup::prefix` attach behaviour to the parent. \
                              Where the part carries behaviour, the slot takes the typed \
                              component rather than an element, so the parent can still \
                              configure it.",
                            cx,
                        ),
                        code_block(DP_COMPOSITION, cx),
                    ]),
                ),
                (
                    "4. Progressive disclosure",
                    stack(vec![
                        para(
                            "A component is useful with nothing but its constructor, and every \
                              other prop is optional. The three buttons below show increasing \
                              levels of configuration.",
                            cx,
                        ),
                        disclosure,
                        code_block(DP_DISCLOSURE, cx),
                    ]),
                ),
                (
                    "5. Predictable behaviour",
                    stack(vec![
                        para(
                            "Keep shared props consistent across the application: `size` is \
                              `sm`/`md`/`lg`, `is_disabled` reads the same on every control, and \
                              a callback is always `Fn(&T, &mut Window, &mut App)`.",
                            cx,
                        ),
                        code_block(DP_PREDICTABLE, cx),
                    ]),
                ),
                (
                    "6. Type safety first",
                    stack(vec![
                        para(
                            "Prefer the Rust types over stringly-typed configuration. A variant \
                              is an enum, so a typo is a compile error and not a silently \
                              unstyled control, and an exhaustive `match` over `Variant` cannot \
                              miss a case when a new one is added.",
                            cx,
                        ),
                        code_block(DP_TYPES, cx),
                    ]),
                ),
                (
                    "7. Separation of styles and logic",
                    stack(vec![
                        para(
                            "Keep shared vocabulary, theme tokens and component implementations \
                              apart: `herogpui-core` for the prop vocabularies and colour maths, \
                              `herogpui-theme` for the tokens, `herogpui-components` for the \
                              components. The theme crate has no component code in it, so a \
                              different widget set can read the same tokens.",
                            cx,
                        ),
                        code_block(DP_SEPARATION, cx),
                    ]),
                ),
                (
                    "8. Developer experience",
                    stack(vec![para(
                        "Use `rustdoc` for builder-level API details and this gallery for \
                          runnable examples — one page per component, every documented example \
                          on it.",
                        cx,
                    )]),
                ),
                (
                    "9. Complete customization",
                    stack(vec![
                        para(
                            "Start from a built-in theme and override a base token when your \
                              application needs a different value. Derived colours follow \
                              through the same `color-mix` rules the theme itself uses. Shared \
                              component metrics belong on `ThemeBuilder::components` and named \
                              recipes, not on every call site.",
                            cx,
                        ),
                        code_block(DP_CUSTOM, cx),
                    ]),
                ),
                (
                    "10. Open and extensible",
                    stack(vec![para(
                        "The tokens, the colour maths and the motion curves are public. A \
                          component built outside this crate can read `cx.colors()`, ask \
                          `util::field_radius(cx)` for its corners and animate on \
                          `Motion::LIST_IN`, and it will match everything shipped here.",
                        cx,
                    )]),
                ),
                (
                    "Use the supported vocabulary",
                    stack(vec![para(
                        "The vocabulary is deliberately small: semantic roles instead of \
                              numbered colour scales, surfaces instead of stacked content \
                              shades, one radius helper per step instead of a single global \
                              radius, and `color` reserved for status. When a control has a \
                              documented state, it is a real prop — `is_pending`, \
                              `is_disabled` — and the component reference is the source of \
                              truth for every available option.",
                        cx,
                    )]),
                ),
            ],
            cx,
        )
    }
}

const STYLING_VARIANTS: &str = r#"// Same prop names, checked at compile time.
Button::new("edit")
    .label("Edit")
    .variant(Variant::Secondary)
    .size(Size::Lg)"#;

const STYLING_STATES: &str = r#"div()
    .id("row")
    .bg(colors.surface.background)
    .hover(move |s| s.bg(colors.default.soft()))

// Components do this internally: `anim::hover_fade` fades the resting
// surface, and a press is `anim::pressed`."#;

const STYLING_RENDER: &str = r#"// The closure is handed the value the component computed.
Slider::new("volume", 50.)
    .thumb(|index, value| {
        div().child(format!("thumb {index}: {value}")).into_any_element()
    })"#;

const STYLING_WRAPPER: &str = r#"/// A save button, everywhere the same.
fn save_button(id: impl Into<ElementId>) -> Button {
    Button::new(id)
        .variant(Variant::Primary)
        .child(icon(icons::CHECK))
        .child("Save")
}

// Still a `Button`, so the caller keeps every other prop.
save_button("save").is_pending(saving).full_width()"#;

const DP_SEMANTIC: &str = r#"// Hierarchy, not appearance.
Button::new("save").label("Save")                              // primary
Button::new("edit").label("Edit").variant(Variant::Secondary)
Button::new("cancel").label("Cancel").variant(Variant::Tertiary)
Button::new("del").label("Delete").variant(Variant::Danger)"#;

const DP_COMPOSITION: &str = r#"// The same parts, as slots. `input` takes an `Input`, not an
// element, so the group can strip the field's own chrome.
InputGroup::new()
    .prefix(InputAddon::new("$"))
    .input(Input::new(amount).placeholder("0.00"))
    .suffix(InputAddon::new("USD"))"#;

const DP_DISCLOSURE: &str = r#"// Level 1
Button::new("go").label("Click me")

// Level 2
Button::new("go").size(Size::Lg).child(check).child("Submit")

// Level 3
Button::new("go").label("Submitting").is_pending(true)"#;

const DP_PREDICTABLE: &str = r#"// The same three props, on three different components.
Button::new("b").size(Size::Lg).is_disabled(true)
Chip::new().size(Size::Lg).child(ChipLabel::new().child("c"))
Avatar::new("a").size(Size::Lg)

// And one callback shape everywhere.
Switch::new("predictable")
    .size(Size::Lg)
    .on_change(|value: bool, _window, _cx| { /* feed value back */ })"#;

const DP_TYPES: &str = r#"// A variant is an enum, so this does not compile:
//     Button::new("b").variant(Variant::Solid)
//                                      ^^^^^ no variant named `Solid`
//
// and an exhaustive match cannot miss one:
match variant {
    Variant::Primary => ..,
    Variant::Secondary => ..,
    Variant::Tertiary => ..,
    Variant::Outline => ..,
    Variant::Ghost => ..,
    Variant::Danger => ..,
    Variant::DangerSoft => ..,
}"#;

const DP_SEPARATION: &str = r#"herogpui-core        // Color, Variant, Size, oklch(), mix_oklab()
herogpui-theme       // the tokens + ThemeProvider (no component code)
herogpui-components  // the components
herogpui             // umbrella re-export

// Read a token without touching a component:
let accent = cx.role(Color::Accent).color;
let radius = herogpui::components::util::field_radius(cx);"#;

const DP_CUSTOM: &str = r#"// Override one base token; every derived value follows.
let violet = Theme::builder("violet", Theme::light())
    .accent(oklch(0.55, 0.23, 295.0))
    .build();

// `accent.hover()` and `accent.soft()` are the same color-mix
// expressions, so they move with the base color."#;

#[cfg(test)]
pub(super) fn doc_code_blocks() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Styling/variants", STYLING_VARIANTS),
        ("Styling/states", STYLING_STATES),
        ("Styling/render", STYLING_RENDER),
        ("Styling/wrapper", STYLING_WRAPPER),
        ("Design Principles/semantic", DP_SEMANTIC),
        ("Design Principles/composition", DP_COMPOSITION),
        ("Design Principles/disclosure", DP_DISCLOSURE),
        ("Design Principles/predictable", DP_PREDICTABLE),
        ("Design Principles/types", DP_TYPES),
        ("Design Principles/separation", DP_SEPARATION),
        ("Design Principles/custom", DP_CUSTOM),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The render-prop teaching snippet must match `Slider::new(id, value)`
    /// and the `thumb(|index, value| el)` closure shape the component crate
    /// actually exposes — no stale no-arg constructor or `window`/`cx`
    /// parameters.
    #[test]
    fn render_prop_example_matches_slider_api() {
        assert!(
            STYLING_RENDER.contains("Slider::new(\"volume\", 50.)"),
            "Slider example lost its initial value: {STYLING_RENDER}"
        );
        assert!(
            STYLING_RENDER.contains(".thumb(|index, value|"),
            "thumb closure shape changed: {STYLING_RENDER}"
        );
        assert!(!STYLING_RENDER.contains("Slider::new(\"volume\")"));
        assert!(!STYLING_RENDER.contains("_window"));
        assert!(!STYLING_RENDER.contains("_cx"));
    }

    /// Avatar takes a constructor id and the shared `Size` enum; the
    /// predictability example must keep teaching that exact shape.
    #[test]
    fn predictable_example_uses_avatar_id_and_core_size() {
        assert!(
            DP_PREDICTABLE.contains("Avatar::new(\"a\").size(Size::Lg)"),
            "Avatar example shape changed: {DP_PREDICTABLE}"
        );
        assert!(!DP_PREDICTABLE.contains("Avatar::new()"));
        assert!(!DP_PREDICTABLE.contains("SizeXl"));
    }

    #[test]
    fn predictable_example_attaches_change_to_a_real_control() {
        assert!(
            DP_PREDICTABLE.contains(
                "Switch::new(\"predictable\")\n    .size(Size::Lg)\n    .on_change(|value: bool, _window, _cx|"
            ),
            "callback must be attached to a component with the real on_change API: {DP_PREDICTABLE}"
        );
        assert!(!DP_PREDICTABLE.contains(
            "Avatar::new(\"a\").size(Size::Lg)\n\n// And one callback shape everywhere.\n.on_change("
        ));
    }
}
