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

/// A two-column row: what v3 does, and what this port does instead.
fn mapping(v3: &str, ours: &str, cx: &App) -> gpui::AnyElement {
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
                .child(v3.to_owned()),
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
                .child(ours.to_owned()),
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
            "v3 styles a component three ways: props for the documented variants, a `className` for \
             anything else, and render props for state. There are no CSS classes in gpui, so the \
             middle one has no direct analogue — this page is where each of v3's styling routes \
             comes out.",
            "",
            vec![
                (
                    "Variants carry the intent",
                    stack(vec![
                        para(
                            "Reach for the documented prop first. Every variant, size and colour \
                             in v3 is a prop here too, with the same name and the same spelling, so \
                             a v3 snippet ports across without deciding anything.",
                            cx,
                        ),
                        hierarchy,
                        code_block(STYLING_VARIANTS, cx),
                    ]),
                ),
                (
                    "Where `className` goes",
                    stack(vec![
                        para(
                            "A v3 `className` does one of four things, and each has its own route \
                             here. Nothing is styled by string.",
                            cx,
                        ),
                        mapping(
                            "className=\"w-full\"",
                            "Layout: wrap the control in a styled div, or use the prop the \
                             component documents for it (`full_width`).",
                            cx,
                        ),
                        mapping(
                            "className=\"bg-accent\"",
                            "Colour: read the token — `cx.colors()`, `cx.role(Color::Accent)` — \
                             so the value follows the active theme instead of pinning a shade.",
                            cx,
                        ),
                        mapping(
                            "className=\"rounded-2xl\"",
                            "Radius: `util::soft_radius(cx)` and its siblings, one per v3 step. \
                             v3 has no single control radius; each component names its own.",
                            cx,
                        ),
                        mapping(
                            "className=\"px-3 text-sm\"",
                            "Spacing and type: gpui's `Styled` methods on the element you own. \
                             Inside a component, they are the component's business.",
                            cx,
                        ),
                    ]),
                ),
                (
                    "State-based styling",
                    stack(vec![
                        para(
                            "v3 styles states with `data-hovered`, `data-pressed` and \
                             `data-disabled` selectors. gpui has the states themselves: `.hover()` \
                             takes a closure, a press is an animation, and `is_disabled` is a prop. \
                             Use the controls below — a screenshot of this page cannot show them.",
                            cx,
                        ),
                        states,
                        code_block(STYLING_STATES, cx),
                    ]),
                ),
                (
                    "Render props",
                    stack(vec![
                        para(
                            "v3's render props hand you the state and let you draw the part \
                             yourself. Ported as closures: the component computes the value it \
                             already knows and passes it in, so a caller never re-derives it.",
                            cx,
                        ),
                        code_block(STYLING_RENDER, cx),
                    ]),
                ),
                (
                    "Wrapper components",
                    stack(vec![
                        para(
                            "v3 suggests wrapping a component to fix a set of props in place. The \
                             builders are plain Rust values, so a wrapper is a function that \
                             returns one — no new type, and every remaining builder still \
                             available to the caller.",
                            cx,
                        ),
                        code_block(STYLING_WRAPPER, cx),
                    ]),
                ),
                (
                    "The class reference, translated",
                    stack(vec![
                        para(
                            "v3 ends this page with its BEM class list (`.button`, \
                             `.button--primary`, `.card__header`). The equivalent map here is one \
                             module per `@heroui/*` package, one struct per component, and a \
                             builder per documented prop — `herogpui::components::button::Button`, \
                             `Button::variant`, `Card::header`. The parity audits in `.shots/` are \
                             what keep that mapping honest.",
                            cx,
                        ),
                        code_block(STYLING_CLASSES, cx),
                    ]),
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
            "v3 is built on ten principles. Nine of them port directly, because they are about API \
             shape rather than about the web; the tenth is where a gpui port has to be honest about \
             what it cannot do.",
            "",
            vec![
                (
                    "1. Semantic intent over visual style",
                    stack(vec![
                        para(
                            "Variants are named for what they mean, not for how they look: \
                             primary is the one action that moves forward, secondary is an \
                             alternative, tertiary is dismissive, danger is destructive. v3 dropped \
                             `solid`, `flat` and `bordered` for exactly this reason, and so did \
                             this port.",
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
                            "This is the principle that does not survive the platform. v3 sits on \
                             React Aria, so every component ships ARIA roles, labels and a screen \
                             reader story. gpui has no accessibility tree to write into, so those \
                             props are recorded as omissions with a reason rather than accepted \
                             and ignored — an `aria_label` builder that went nowhere would be a \
                             promise the library cannot keep.",
                            cx,
                        ),
                        para(
                            "What does port is the part that is behaviour rather than annotation: \
                             focus handling, keyboard navigation, `Escape` to dismiss, arrow keys \
                             through a menu, typing into a date field segment.",
                            cx,
                        ),
                    ]),
                ),
                (
                    "3. Composition over configuration",
                    stack(vec![
                        para(
                            "v3 composes parts: `Modal.Close`, `Card.Header`, \
                             `InputGroup.Prefix`. gpui has no JSX and no way for a child to reach \
                             its parent, so a named part becomes a named slot on the builder — the \
                             same composition, spelled as a method. Where the part carries \
                             behaviour, the slot takes the typed component rather than an element, \
                             so the parent can still configure it.",
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
                             other prop is optional. The three buttons below are v3's own three \
                             levels of the same control.",
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
                            "The same prop means the same thing everywhere: `size` is `sm`/`md`/`lg`, \
                             `is_disabled` reads the same on every control, a callback is always \
                             `Fn(&T, &mut Window, &mut App)`. Two audits enforce it — one checks \
                             every documented prop is implemented, the other checks we expose \
                             nothing v3 does not document.",
                            cx,
                        ),
                        code_block(DP_PREDICTABLE, cx),
                    ]),
                ),
                (
                    "6. Type safety first",
                    stack(vec![
                        para(
                            "v3 asks TypeScript for compile-time errors and autocomplete. Rust \
                             gives more of it: a variant is an enum rather than a string union, so \
                             a typo is a compile error and not a silently unstyled control, and an \
                             exhaustive `match` over `Variant` cannot miss a case when v3 adds one.",
                            cx,
                        ),
                        code_block(DP_TYPES, cx),
                    ]),
                ),
                (
                    "7. Separation of styles and logic",
                    stack(vec![
                        para(
                            "v3 ships `@heroui/styles` apart from `@heroui/react` so the styles \
                             work without the framework. The same split is three crates: \
                             `herogpui-core` for the prop vocabularies and colour maths, \
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
                        "Every builder carries the v3 prop name it ports in its doc comment, so \
                         `rustdoc` reads as a translation table. This gallery is the Storybook \
                         equivalent: one page per component, every documented example on it.",
                        cx,
                    )]),
                ),
                (
                    "9. Complete customization",
                    stack(vec![
                        para(
                            "Defaults are the v3 defaults, transcribed rather than tuned — the \
                             OKLCH values are the same numbers. A theme overrides a base token and \
                             every derived value follows, because the derivations are the same \
                             `color-mix` expressions v3 uses.",
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
                         `util::field_radius(cx)` for its corners and animate on `Motion::LIST_IN`, \
                         and it will match everything shipped here.",
                        cx,
                    )]),
                ),
                (
                    "What v3 dropped from v2",
                    stack(vec![
                        para(
                            "The principles above are why v3 removed a long list of v2 concepts, \
                             and this port removed them too: the numbered 50–900 colour scales, \
                             `content1`–`content4`, the per-component `radius` prop, `color` on \
                             anything that is not a status, `size` on a form field, `isBordered`, \
                             `isBlurred`, `isStriped`. When one reappears, `extra_audit.py` is what \
                             catches it.",
                            cx,
                        ),
                        code_block(DP_V2, cx),
                    ]),
                ),
            ],
            cx,
        )
    }
}

const STYLING_VARIANTS: &str = r#"// v3
<Button variant="secondary" size="lg">Edit</Button>

// HeroGPUI — same prop names, checked at compile time
Button::new("edit")
    .label("Edit")
    .variant(Variant::Secondary)
    .size(Size::Lg)"#;

const STYLING_STATES: &str = r#"// v3: a selector per state
<Button className="data-[hovered]:bg-accent-hover" />

// HeroGPUI: the state itself
div()
    .id("row")
    .bg(colors.surface.background)
    .hover(move |s| s.bg(colors.default.soft()))

// Components already do this internally: `anim::hover_fade` is the
// `transition-colors` equivalent, and a press is `anim::pressed`."#;

const STYLING_RENDER: &str = r#"// v3
<Slider>{({ index }) => <Slider.Thumb key={index} />}</Slider>

// HeroGPUI — the closure is handed the value the component computed
Slider::new("volume")
    .thumb(|index, _window, _cx| {
        div().child(format!("thumb {index}")).into_any_element()
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

const STYLING_CLASSES: &str = r#"// v3 class            -> HeroGPUI
// .button              -> components::button::Button
// .button--secondary   -> Button::variant(Variant::Secondary)
// .button--lg          -> Button::size(Size::Lg)
// .card__header        -> Card::header(..)
// .input--secondary    -> Input::variant(FieldVariant::Secondary)
// --field-radius       -> util::field_radius(cx)
// --ease-out-fluid     -> anim::Curve::OutFluid"#;

const DP_SEMANTIC: &str = r#"// Hierarchy, not appearance.
Button::new("save").label("Save")                              // primary
Button::new("edit").label("Edit").variant(Variant::Secondary)
Button::new("cancel").label("Cancel").variant(Variant::Tertiary)
Button::new("del").label("Delete").variant(Variant::Danger)"#;

const DP_COMPOSITION: &str = r#"// v3
<InputGroup>
  <InputGroup.Prefix>$</InputGroup.Prefix>
  <InputGroup.Input placeholder="0.00" />
  <InputGroup.Suffix>USD</InputGroup.Suffix>
</InputGroup>

// HeroGPUI — the same parts, as slots. `input` takes an `Input`, not an
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
Avatar::new().size(SizeXl::Lg)

// And one callback shape everywhere.
.on_change(|value: &str, _window, _cx| { /* ... */ })"#;

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
// expressions v3 uses, so they move with it."#;

const DP_V2: &str = r#"// Gone in v3, and gone here:
//   content1..content4   -> surface / surface_secondary / surface_tertiary / overlay
//   default-50..900      -> RoleColor::hover() / soft() / soft_hover()
//   radius="lg"          -> theme radius tokens
//   color on a Button    -> variant
//   size on an Input     -> util::FIELD_HEIGHT (v3 gives fields one height)
//   isLoading            -> is_pending
//   Divider / Progress   -> Separator / ProgressBar"#;
