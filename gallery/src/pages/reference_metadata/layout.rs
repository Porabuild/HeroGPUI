//! layout reference metadata.

use super::*;

const SEPARATOR_REQUIRED_PARTS: &[&str] = &[
    // The root is the component's only declared part.
    "Separator",
];

const SEPARATOR_API: &[ApiDoc] = &[
    ApiDoc {
        owner: "Separator",
        prop: "id",
        ty: "string",
        default: "—",
        description: "Optional GPUI identity. AccessKit 0.24 has no Role::Separator, so a named instance still produces no node.",
        rust_owner: "Separator",
        rust: "id(ElementId)",
        status: ImplementationStatus::Partial,
    },
    ApiDoc {
        owner: "Separator",
        prop: "orientation",
        ty: "'horizontal' | 'vertical'",
        default: "'horizontal'",
        description: "The orientation of the separator.",
        rust_owner: "Separator",
        rust: "orientation(Orientation)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "Separator",
        prop: "variant",
        ty: "'default' | 'secondary' | 'tertiary'",
        default: "'default'",
        description: "The visual variant of the separator.",
        rust_owner: "Separator",
        rust: "variant(SeparatorVariant)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "Separator",
        prop: "className",
        ty: "string",
        default: "—",
        description: "Additional CSS classes; gpui has no class-name surface.",
        rust_owner: "Separator",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
    ApiDoc {
        owner: "Separator",
        prop: "render",
        ty: "DOMRenderFunction<keyof React.JSX.IntrinsicElements, undefined>",
        default: "—",
        description: "Replaces the DOM element; gpui does not substitute DOM nodes.",
        rust_owner: "Separator",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
];

const SEPARATOR_PARTS: &[PartDoc] = &[
    // v3's root owns data-slot="separator"; there are no separately exported
    // child parts.
    PartDoc {
        name: "Separator",
        slot: "separator",
        description: "Root separator primitive and data-slot owner.",
        rust_owner: "Separator",
        status: ImplementationStatus::Implemented,
    },
];

const SEPARATOR_STATES: &[StateDoc] = &[
    // v3.2.5 documents no interactive states and its stylesheet declares no
    // state selectors for Separator.
];

const SEPARATOR_STYLING: &[StyleDoc] = &[
    StyleDoc {
        class_or_token: ".separator",
        value: "shrink-0 rounded-sm border-t-0 border-b-0 bg-separator h-px w-full",
        description: "Base line geometry, color and default horizontal orientation.",
        rust: "flex_shrink_0 + hairline_radius + colors.separator + w_full/h(border_width)",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".separator--horizontal",
        value: "h-px w-full",
        description: "Horizontal separator dimensions.",
        rust: "Orientation::Horizontal => w_full().h(weight)",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".separator--vertical",
        value: "h-auto min-h-2 w-px self-stretch",
        description: "Vertical separator dimensions; the port uses full parent height instead of h-auto/self-stretch.",
        rust: "Orientation::Vertical => h_full().min_h(px(8.)).w(weight)",
        status: ImplementationStatus::Partial,
    },
    StyleDoc {
        class_or_token: ".separator--default",
        value: "bg-separator",
        description: "Default separator semantic color.",
        rust: "SeparatorVariant::Default => colors.separator",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".separator--secondary",
        value: "bg-separator-secondary",
        description: "Secondary separator semantic color.",
        rust: "SeparatorVariant::Secondary => colors.separator_secondary()",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".separator--tertiary",
        value: "bg-separator-tertiary",
        description: "Tertiary separator semantic color.",
        rust: "SeparatorVariant::Tertiary => colors.separator_tertiary()",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".separator__container",
        value: "flex items-center gap-3",
        description: "Container used when content divides two line segments.",
        rust: "content branch flex + items_center + gap(px(12.))",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".separator__container--horizontal",
        value: "w-full flex-row",
        description: "Horizontal content-container axis.",
        rust: "horizontal content branch w_full().flex_row()",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".separator__container--vertical",
        value: "h-full flex-col justify-center",
        description: "Vertical content-container axis.",
        rust: "vertical content branch h_full().flex_col().justify_center()",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".separator__line",
        value: "shrink-0 grow",
        description: "Each line beside composed content grows into remaining space.",
        rust: "line flex_shrink_0 + flex_grow()",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".separator__content",
        value: "inline-flex items-center justify-center text-center whitespace-nowrap text-muted",
        description: "Centered, non-wrapping composed content.",
        rust: "content div flex + items_center + justify_center + text_center + whitespace_nowrap + colors.muted",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".separator__content--horizontal / --vertical",
        value: "text-center",
        description: "Both orientation-specific content modifiers keep centered text.",
        rust: "content div text_center()",
        status: ImplementationStatus::Implemented,
    },
];

pub(super) const SEPARATOR: ReferenceMetadata = ReferenceMetadata {
    page: "Separator",
    import_line: "use herogpui::components::separator::Separator;",
    source_module: "separator",
    version: "3.2.5",
    docs_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/apps/docs/content/docs/en/react/components/(layout)/separator.mdx",
    api_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/packages/react/src/components/separator/separator.tsx + https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/Separator.tsx",
    style_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/packages/styles/components/separator.css",
    required_parts: SEPARATOR_REQUIRED_PARTS,
    api: SEPARATOR_API,
    parts: SEPARATOR_PARTS,
    states: SEPARATOR_STATES,
    styling: SEPARATOR_STYLING,
};

const TOOLBAR_REQUIRED_PARTS: &[&str] = &[
    // The root is the component's only declared part; the v3 page documents
    // no `Toolbar.*` composition parts.
    "Toolbar",
];

const TOOLBAR_API: &[ApiDoc] = &[
    ApiDoc {
        owner: "Toolbar",
        prop: "isAttached",
        ty: "boolean",
        default: "false",
        description: "Whether the toolbar has a surface background with full rounding.",
        rust_owner: "Toolbar",
        rust: "is_attached(bool)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "Toolbar",
        prop: "orientation",
        ty: "\"horizontal\" | \"vertical\"",
        default: "\"horizontal\"",
        description: "The orientation of the toolbar; the axis picks which arrows walk it.",
        rust_owner: "Toolbar",
        rust: "orientation(Orientation)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "Toolbar",
        prop: "aria-label",
        ty: "string",
        default: "—",
        description: "Accessible name; gpui has no accessibility tree to name it in.",
        rust_owner: "Toolbar",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
    ApiDoc {
        owner: "Toolbar",
        prop: "aria-labelledby",
        ty: "string",
        default: "—",
        description: "Accessible name by element id; gpui has no accessibility tree.",
        rust_owner: "Toolbar",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
    ApiDoc {
        owner: "Toolbar",
        prop: "children",
        ty: "React.ReactNode | (values: ToolbarRenderProps) => React.ReactNode",
        default: "—",
        description: "Content, or a render function receiving the orientation. The port takes plain child elements; the function form is not ported.",
        rust_owner: "Toolbar",
        rust: "—",
        status: ImplementationStatus::Partial,
    },
    ApiDoc {
        owner: "Toolbar",
        prop: "className",
        ty: "string | (values: ToolbarRenderProps) => string",
        default: "—",
        description: "Additional CSS classes; gpui has no class-name surface.",
        rust_owner: "Toolbar",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
    ApiDoc {
        owner: "ToolbarRenderProps",
        prop: "orientation",
        ty: "\"horizontal\" | \"vertical\"",
        default: "—",
        description: "Handed to children-as-function; the port's children are plain elements and the orientation is a builder value, so nothing receives it.",
        rust_owner: "Toolbar",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
];

const TOOLBAR_PARTS: &[PartDoc] = &[
    // The root owns `.toolbar` and its `--horizontal` / `--vertical` /
    // `--attached` modifiers; there are no separately exported child parts.
    PartDoc {
        name: "Toolbar",
        slot: "toolbar",
        description: "Root container; set its orientation before separator() to append a centred divider half the bar's cross size.",
        rust_owner: "Toolbar",
        status: ImplementationStatus::Implemented,
    },
];

const TOOLBAR_STATES: &[StateDoc] = &[
    // The v3 page has no Accessibility prose; these are the states the pinned
    // React Aria `useToolbar` (react-aria 3.52.0) defines on the children.
    StateDoc { state: "Focused child", selector: ".toolbar :focus-visible", description: "Focus lands on the child controls, never the container; each child draws its own ring and the orientation-axis arrows walk the children.", rust: "child tab stops + child control focus ring", status: ImplementationStatus::Implemented },
    StateDoc { state: "Disabled child", selector: ".toolbar [aria-disabled=true]", description: "A disabled child is no tab stop: the arrows skip it in both directions and Tab walks past a toolbar with none enabled.", rust: "child control is_disabled", status: ImplementationStatus::Implemented },
    StateDoc { state: "Last focused child", selector: ".toolbar lastFocused", description: "The pinned hook records the child the focus left from and restores it when the focus re-enters from outside; the port keeps the record in keyed state and skips a recorded child whose element has left the frame, exactly as pinned's focus-capture does.", rust: "ToolbarFocusEdge.last_focused", status: ImplementationStatus::Implemented },
    StateDoc { state: "Nested toolbar as group", selector: ".toolbar [role=group]", description: "Pinned useToolbar detects a toolbar inside another with parentElement.closest('[role=toolbar]') and the nested one binds no keyboard or focus management of its own — the enclosing toolbar's arrows walk straight across its children and one Tab leaves the whole subtree. The port asks the same question of the last rendered frame's dispatch tree: a weak per-window registry of toolbar scopes plus FocusHandle::contains, re-checked at event time against the frame each key dispatched against.", rust: "ToolbarScopes registry + FocusHandle::contains", status: ImplementationStatus::Implemented },
];

const TOOLBAR_STYLING: &[StyleDoc] = &[
    StyleDoc {
        class_or_token: ".toolbar",
        value: "grid w-fit grid-flow-col items-center gap-2",
        description: "One row (or column) hugging its controls with the 8px rhythm; a flex row or column stands in for the grid flow.",
        rust: "flex_row/flex_col + items_center + gap(px(8.))",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".toolbar--horizontal",
        value: "(no declarations; the base flow)",
        description: "Default orientation keeps the base row flow and centered cross axis.",
        rust: "Orientation::Horizontal => flex_row + items_center",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".toolbar--vertical",
        value: "grid-flow-row items-start justify-start",
        description: "Column flow whose controls hug the start edge; the base rule's centered cross axis stays with the horizontal toolbar.",
        rust: "Orientation::Vertical => flex_col + items_start + justify_start",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".toolbar--vertical .button-group",
        value: "justify-start",
        description: "v3 also re-justifies a ButtonGroup nested in a vertical toolbar to the start edge. gpui has no parent-to-child style propagation: the toolbar's children are opaque elements, so the parent cannot re-justify a ButtonGroup it contains and the group keeps its own alignment.",
        rust: "—",
        status: ImplementationStatus::Partial,
    },
    StyleDoc {
        class_or_token: ".toolbar--attached",
        value: "rounded-3xl bg-surface p-1 shadow-overlay; no border",
        description: "The attached strip hugs its controls with surface fill, full rounding, 4px padding and the overlay shadow; v3 gives a floating surface no border.",
        rust: "p(px(4.)) + util::control_radius + colors.surface.background + overlay_shadow; no border",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".toolbar .separator--vertical / .separator--horizontal",
        value: "h-1/2 self-center / w-1/2 justify-center justify-self-center",
        description: "Contained separators shrink to half the toolbar; the port's Separator keeps its own thickness and centers itself.",
        rust: "Separator orientation",
        status: ImplementationStatus::Partial,
    },
];

pub(super) const TOOLBAR: ReferenceMetadata = ReferenceMetadata {
    page: "Toolbar",
    import_line: "use herogpui::components::toolbar::Toolbar;",
    source_module: "toolbar",
    version: "3.2.5",
    docs_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/apps/docs/content/docs/en/react/components/(layout)/toolbar.mdx",
    api_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/packages/react/src/components/toolbar/toolbar.tsx + https://github.com/adobe/react-spectrum/blob/react-aria@3.52.0/packages/react-aria/src/toolbar/useToolbar.ts",
    style_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/packages/styles/components/toolbar.css",
    required_parts: TOOLBAR_REQUIRED_PARTS,
    api: TOOLBAR_API,
    parts: TOOLBAR_PARTS,
    states: TOOLBAR_STATES,
    styling: TOOLBAR_STYLING,
};

const CARD_REQUIRED_PARTS: &[&str] = &[
    "Card",
    "Card.Header",
    "Card.Title",
    "Card.Description",
    "Card.Content",
    "Card.Footer",
];

const CARD_API: &[ApiDoc] = &[
    ApiDoc {
        owner: "Card",
        prop: "variant",
        ty: "\"transparent\" | \"default\" | \"secondary\" | \"tertiary\"",
        default: "\"default\"",
        description: "Semantic variant indicating prominence level. Transparent paints nothing — no border, no background, no shadow, the full content box — while every other level fills and carries the surface shadow.",
        rust_owner: "Card",
        rust: "variant(CardVariant)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "Card",
        prop: "className",
        ty: "string",
        default: "—",
        description: "Additional CSS classes.",
        rust_owner: "Card",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
    ApiDoc {
        owner: "Card",
        prop: "children",
        ty: "React.ReactNode",
        default: "—",
        description: "Card content; the parts compose through ParentElement::extend and the root wraps them in the padded column with the capped container radius.",
        rust_owner: "Card",
        rust: "ParentElement::extend",
        status: ImplementationStatus::Partial,
    },
    ApiDoc {
        owner: "Card.Header",
        prop: "className",
        ty: "string",
        default: "—",
        description: "Additional CSS classes for the header.",
        rust_owner: "CardHeader",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
    ApiDoc {
        owner: "Card.Header",
        prop: "children",
        ty: "React.ReactNode",
        default: "—",
        description: "Header content; a plain flex column, so Card.Title and Card.Description carry their own text styles.",
        rust_owner: "CardHeader",
        rust: "ParentElement::extend",
        status: ImplementationStatus::Partial,
    },
    ApiDoc {
        owner: "Card.Title",
        prop: "className",
        ty: "string",
        default: "—",
        description: "Additional CSS classes for the title.",
        rust_owner: "CardTitle",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
    ApiDoc {
        owner: "Card.Title",
        prop: "children",
        ty: "React.ReactNode",
        default: "—",
        description: "Title content (v3 renders an h3). The port draws the title typography on a plain div; GPUI has no heading semantics to render it as.",
        rust_owner: "CardTitle",
        rust: "ParentElement::extend",
        status: ImplementationStatus::Partial,
    },
    ApiDoc {
        owner: "Card.Description",
        prop: "className",
        ty: "string",
        default: "—",
        description: "Additional CSS classes for the description.",
        rust_owner: "CardDescription",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
    ApiDoc {
        owner: "Card.Description",
        prop: "children",
        ty: "React.ReactNode",
        default: "—",
        description: "Description content (v3 renders a p). The port draws the muted description typography on a plain div.",
        rust_owner: "CardDescription",
        rust: "ParentElement::extend",
        status: ImplementationStatus::Partial,
    },
    ApiDoc {
        owner: "Card.Content",
        prop: "className",
        ty: "string",
        default: "—",
        description: "Additional CSS classes for the content.",
        rust_owner: "CardContent",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
    ApiDoc {
        owner: "Card.Content",
        prop: "children",
        ty: "React.ReactNode",
        default: "—",
        description: "Main content; the port keeps the column and its four-pixel gap but drops the upstream flex-1: the pinned-geometry test in tests/card_deep.rs measures the card as an auto-height column hugging its parts, which flex-1 regresses.",
        rust_owner: "CardContent",
        rust: "ParentElement::extend",
        status: ImplementationStatus::Partial,
    },
    ApiDoc {
        owner: "Card.Footer",
        prop: "className",
        ty: "string",
        default: "—",
        description: "Additional CSS classes for the footer.",
        rust_owner: "CardFooter",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
    ApiDoc {
        owner: "Card.Footer",
        prop: "children",
        ty: "React.ReactNode",
        default: "—",
        description: "Footer content; a plain flex row with centered items — no padding, gap or text size of its own; the card's gap separates the parts.",
        rust_owner: "CardFooter",
        rust: "ParentElement::extend",
        status: ImplementationStatus::Partial,
    },
];

const CARD_PARTS: &[PartDoc] = &[
    PartDoc {
        name: "Card",
        slot: "card",
        description: "Composition root and data-slot owner; the padded column at a 12px part gap with the capped container radius. `overflow-visible` upstream, so the port adds no clipping; the variant chooses the fill and whether the surface shadow applies.",
        rust_owner: "Card",
        status: ImplementationStatus::Implemented,
    },
    PartDoc {
        name: "Card.Header",
        slot: "card-header",
        description: "Header section container; a plain flex column, with no text style of its own.",
        rust_owner: "CardHeader",
        status: ImplementationStatus::Implemented,
    },
    PartDoc {
        name: "Card.Title",
        slot: "card-title",
        description: "Title part: 14px medium text on the foreground colour, 24px leading. v3 renders an h3; the port renders the same typography on a plain div.",
        rust_owner: "CardTitle",
        status: ImplementationStatus::Implemented,
    },
    PartDoc {
        name: "Card.Description",
        slot: "card-description",
        description: "Muted description part: 14px text at 20px leading. v3 renders a p; the port renders the same typography on a plain div.",
        rust_owner: "CardDescription",
        status: ImplementationStatus::Implemented,
    },
    PartDoc {
        name: "Card.Content",
        slot: "card-content",
        description: "Content column at a four-pixel gap. The upstream flex-1 is dropped: the pinned-geometry test in tests/card_deep.rs measures the card as an auto-height column hugging its parts, which flex-1 regresses.",
        rust_owner: "CardContent",
        status: ImplementationStatus::Partial,
    },
    PartDoc {
        name: "Card.Footer",
        slot: "card-footer",
        description: "Footer row with centered items; the card's gap separates it from the other parts and the caller composes the row's contents.",
        rust_owner: "CardFooter",
        status: ImplementationStatus::Implemented,
    },
];

// `.card` declares no state utilities and the docs name no interactive states:
// v3's Card is presentational, and an interactive card composes a Button.
const CARD_STATES: &[StateDoc] = &[];

const CARD_STYLING: &[StyleDoc] = &[
    StyleDoc {
        class_or_token: ".card",
        value: "relative flex flex-col gap-3 overflow-visible p-4 shadow-surface; border-radius: min(32px, var(--radius-3xl))",
        description: "Padded column root: 12px part gap, 16px padding, the capped container radius, no clipping, and the surface shadow carried by every non-transparent variant.",
        rust: "flex + flex_col + gap(px(12.)) + p(px(16.)) + container_radius + surface_shadow",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".card__header",
        value: "flex flex-col",
        description: "The header stacks its parts and carries nothing else; the title's text style belongs to Card.Title and the description's to Card.Description.",
        rust: "CardHeader flex + flex_col",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".card__title",
        value: "text-sm leading-6 font-medium text-foreground",
        description: "Title typography.",
        rust: "CardTitle text_size(px(14.)) + line_height(px(24.)) + FontWeight::MEDIUM + colors.foreground",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".card__description",
        value: "text-sm leading-5 text-muted",
        description: "Muted description typography.",
        rust: "CardDescription text_size(px(14.)) + line_height(px(20.)) + colors.muted",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".card__content",
        value: "flex flex-1 flex-col gap-1",
        description: "Content column at a four-pixel gap; the upstream flex-1 is dropped (tests/card_deep.rs pins the card as an auto-height column hugging its parts, which flex-1 regresses).",
        rust: "CardContent flex + flex_col + gap(px(4.))",
        status: ImplementationStatus::Partial,
    },
    StyleDoc {
        class_or_token: ".card__footer",
        value: "flex flex-row items-center",
        description: "Footer row with centered items and no chrome of its own.",
        rust: "CardFooter flex + items_center",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".card--transparent",
        value: "border-none bg-transparent shadow-none",
        description: "The transparent variant paints nothing: no border, no background, no shadow. GPUI's default div is already transparent, so the variant branch adds nothing and the shadow is gated on the variant.",
        rust: "CardVariant::Transparent => el (no bg, no border, no shadow)",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".card--default",
        value: "bg-surface",
        description: "Standard card fill.",
        rust: "CardVariant::Default => colors.surface.background",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".card--secondary",
        value: "bg-surface-secondary",
        description: "Medium-prominence fill.",
        rust: "CardVariant::Secondary => colors.surface_secondary",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".card--tertiary",
        value: "bg-surface-tertiary",
        description: "Higher-prominence fill.",
        rust: "CardVariant::Tertiary => colors.surface_tertiary",
        status: ImplementationStatus::Implemented,
    },
];

pub(super) const CARD: ReferenceMetadata = ReferenceMetadata {
    page: "Card",
    import_line: "use herogpui::components::card::{Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle};",
    source_module: "card",
    version: "3.2.5",
    docs_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/apps/docs/content/docs/en/react/components/(layout)/card.mdx",
    api_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/packages/react/src/components/card/card.tsx",
    style_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/packages/styles/components/card.css",
    required_parts: CARD_REQUIRED_PARTS,
    api: CARD_API,
    parts: CARD_PARTS,
    states: CARD_STATES,
    styling: CARD_STYLING,
};

const SURFACE_REQUIRED_PARTS: &[&str] = &[
    // The root is the component's only declared part.
    "Surface",
];

const SURFACE_API: &[ApiDoc] = &[
    ApiDoc {
        owner: "Surface",
        prop: "variant",
        ty: "\"transparent\" | \"default\" | \"secondary\" | \"tertiary\"",
        default: "\"default\"",
        description: "The visual variant of the surface. The strict root is only `relative text-foreground` plus each variant's fill and foreground pair — no padding, gap, radius or border of its own.",
        rust_owner: "Surface",
        rust: "variant(SurfaceVariant)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "Surface",
        prop: "className",
        ty: "string",
        default: "—",
        description: "Additional CSS classes.",
        rust_owner: "Surface",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
    ApiDoc {
        owner: "Surface",
        prop: "children",
        ty: "ReactNode",
        default: "—",
        description: "Surface content, composed through ParentElement::extend. The port's `.padding`/`.gap` builders are repository conveniences standing in for the className skeleton every docs example adds (`flex flex-col gap-3 rounded-3xl p-6`) — not upstream props, and they default to zero because the component itself ships none.",
        rust_owner: "Surface",
        rust: "ParentElement::extend",
        status: ImplementationStatus::Partial,
    },
    ApiDoc {
        owner: "SurfaceContext",
        prop: "variant",
        ty: "\"transparent\" | \"default\" | \"secondary\" | \"tertiary\" | undefined",
        default: "—",
        description: "Context that lets child components read the surrounding surface variant; GPUI has no ancestor context propagation, so nothing downstream reads the surface variant in this port.",
        rust_owner: "Surface",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
];

const SURFACE_PARTS: &[PartDoc] = &[
    // v3's root owns data-slot="surface"; there are no separately exported
    // child parts.
    PartDoc {
        name: "Surface",
        slot: "surface",
        description: "Strict component root: `relative` plus the variant's background and foreground pair. Zero default padding and gap, no radius and no border; the port's `.padding`/`.gap` builders are port conveniences, not v3 props.",
        rust_owner: "Surface",
        status: ImplementationStatus::Implemented,
    },
];

// `.surface` declares no state utilities and the docs name no interactive states.
const SURFACE_STATES: &[StateDoc] = &[];

const SURFACE_STYLING: &[StyleDoc] = &[
    StyleDoc {
        class_or_token: ".surface",
        value: "relative text-foreground",
        description: "The strict component root: positioning and a foreground default only. The port adds a minimal flex column so its `.padding`/`.gap` conveniences work — layout the upstream stylesheet does not declare.",
        rust: "flex + flex_col + gap(self.gap) + p(self.padding) + text_color(colors.foreground)",
        status: ImplementationStatus::Partial,
    },
    StyleDoc {
        class_or_token: ".surface--transparent",
        value: "bg-transparent",
        description: "The transparent variant paints nothing extra: GPUI's default div background is already transparent, and no outline is added — the v3 docs example draws its border through className.",
        rust: "SurfaceVariant::Transparent => el (no bg, no border)",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".surface--default",
        value: "bg-surface text-surface-foreground",
        description: "Standard surface fill with its own foreground.",
        rust: "SurfaceVariant::Default => colors.surface.background + colors.surface.foreground",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".surface--secondary",
        value: "bg-surface-secondary text-surface-secondary-foreground",
        description: "Medium-prominence fill with its own foreground.",
        rust: "SurfaceVariant::Secondary => colors.surface_secondary + colors.surface_secondary_foreground()",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".surface--tertiary",
        value: "bg-surface-tertiary text-surface-tertiary-foreground",
        description: "Higher-prominence fill with its own foreground.",
        rust: "SurfaceVariant::Tertiary => colors.surface_tertiary + colors.surface_tertiary_foreground()",
        status: ImplementationStatus::Implemented,
    },
];

pub(super) const SURFACE: ReferenceMetadata = ReferenceMetadata {
    page: "Surface",
    import_line: "use herogpui::components::surface::Surface;",
    source_module: "surface",
    version: "3.2.5",
    docs_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/apps/docs/content/docs/en/react/components/(layout)/surface.mdx",
    api_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/packages/react/src/components/surface/surface.tsx",
    style_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/packages/styles/components/surface.css",
    required_parts: SURFACE_REQUIRED_PARTS,
    api: SURFACE_API,
    parts: SURFACE_PARTS,
    states: SURFACE_STATES,
    styling: SURFACE_STYLING,
};
