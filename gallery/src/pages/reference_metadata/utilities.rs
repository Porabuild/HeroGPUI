//! utilities reference metadata.

use super::*;

const SCROLL_SHADOW_REQUIRED_PARTS: &[&str] = &["ScrollShadow"];

const SCROLL_SHADOW_API: &[ApiDoc] = &[
    ApiDoc {
        owner: "ScrollShadow",
        prop: "orientation",
        ty: "'vertical' | 'horizontal'",
        default: "'vertical'",
        description: "Axis along which content scrolls and fades appear.",
        rust_owner: "ScrollShadow",
        rust: "orientation(Orientation)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "ScrollShadow",
        prop: "variant",
        ty: "'fade'",
        default: "'fade'",
        description:
            "The documented API has one visual variant, which is the port's built-in treatment.",
        rust_owner: "ScrollShadow",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
    ApiDoc {
        owner: "ScrollShadow",
        prop: "size",
        ty: "number",
        default: "40",
        description: "Depth of each fade gradient in pixels.",
        rust_owner: "ScrollShadow",
        rust: "size(Pixels)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "ScrollShadow",
        prop: "offset",
        ty: "number",
        default: "0",
        description: "Scroll distance before the corresponding fade appears.",
        rust_owner: "ScrollShadow",
        rust: "offset(Pixels)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "ScrollShadow",
        prop: "hideScrollBar",
        ty: "boolean",
        default: "false",
        description: "Omits the painted overlay thumb that stands in for the browser scrollbar.",
        rust_owner: "ScrollShadow",
        rust: "hide_scroll_bar(bool)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "ScrollShadow",
        prop: "isEnabled",
        ty: "boolean",
        default: "true",
        description: "Turns fade rendering and visibility reporting on or off.",
        rust_owner: "ScrollShadow",
        rust: "is_enabled(bool)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "ScrollShadow",
        prop: "visibility",
        ty: "'auto' | 'both' | 'top' | 'bottom' | 'left' | 'right' | 'none'",
        default: "'auto'",
        description: "Controls which edge fades are eligible to render.",
        rust_owner: "ScrollShadow",
        rust: "visibility(ScrollShadowVisibility)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "ScrollShadow",
        prop: "onVisibilityChange",
        ty: "(visibility: ScrollShadowVisibility) => void",
        default: "—",
        description: "Reports resolved visible edges when they change.",
        rust_owner: "ScrollShadow",
        rust: "on_visibility_change(callback)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "ScrollShadow",
        prop: "className",
        ty: "string",
        default: "—",
        description: "Browser CSS classes are unavailable.",
        rust_owner: "ScrollShadow",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
    ApiDoc {
        owner: "ScrollShadow",
        prop: "children",
        ty: "ReactNode",
        default: "—",
        description: "Scrollable child content.",
        rust_owner: "ScrollShadow",
        rust: "—",
        status: ImplementationStatus::Partial,
    },
];

const SCROLL_SHADOW_PARTS: &[PartDoc] = &[PartDoc {
    name: "ScrollShadow",
    slot: "scroll-shadow",
    description: "Scrollable root plus edge-fade overlays.",
    rust_owner: "ScrollShadow",
    status: ImplementationStatus::Implemented,
}];

const SCROLL_SHADOW_STATES: &[StateDoc] = &[
    StateDoc {
        state: "Leading edge",
        selector: "[data-top-scroll] / [data-left-scroll]",
        description: "Leading fade appears after content moves beyond the configured offset.",
        rust: "ScrollHandle::offset + offset",
        status: ImplementationStatus::Implemented,
    },
    StateDoc {
        state: "Trailing edge",
        selector: "[data-bottom-scroll] / [data-right-scroll]",
        description: "Trailing fade appears while more content remains after the viewport.",
        rust: "ScrollHandle::max_offset + offset",
        status: ImplementationStatus::Implemented,
    },
    StateDoc {
        state: "Both edges",
        selector: "[data-top-bottom-scroll] / [data-left-right-scroll]",
        description: "Both fades render when content can scroll in either direction.",
        rust: "ScrollShadowVisibility::Both",
        status: ImplementationStatus::Implemented,
    },
];

const SCROLL_SHADOW_STYLING: &[StyleDoc] = &[
    StyleDoc {
        class_or_token: ".scroll-shadow",
        value: "overflow auto with mask-image",
        description: "The port uses a tracked GPUI scroller with explicit gradient overlays.",
        rust: "track_scroll + overflow_x_scroll / overflow_y_scroll",
        status: ImplementationStatus::Partial,
    },
    StyleDoc {
        class_or_token: "--scroll-shadow-size",
        value: "40px",
        description: "Fade depth follows the size builder.",
        rust: "size(Pixels)",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: "[data-orientation]",
        value: "vertical | horizontal",
        description: "Orientation switches scroll axis and fade direction; wheel input from the other axis is not remapped.",
        rust: "orientation(Orientation)",
        status: ImplementationStatus::Implemented,
    },
];

pub(super) const SCROLL_SHADOW: ReferenceMetadata = ReferenceMetadata {
    page: "ScrollShadow",
    import_line: "use herogpui::components::scroll_shadow::ScrollShadow;",
    source_module: "scroll_shadow",
    version: "3.2.4",
    docs_source: "https://github.com/heroui-inc/heroui/blob/v3.2.4/apps/docs/content/docs/en/react/components/(utilities)/scroll-shadow.mdx",
    api_source: "https://github.com/heroui-inc/heroui/blob/v3.2.4/packages/react/src/components/scroll-shadow/scroll-shadow.tsx + https://github.com/heroui-inc/heroui/blob/v3.2.4/packages/react/src/components/scroll-shadow/use-scroll-shadow.ts",
    style_source: "https://github.com/heroui-inc/heroui/blob/v3.2.4/packages/styles/components/scroll-shadow.css",
    required_parts: SCROLL_SHADOW_REQUIRED_PARTS,
    api: SCROLL_SHADOW_API,
    parts: SCROLL_SHADOW_PARTS,
    states: SCROLL_SHADOW_STATES,
    styling: SCROLL_SHADOW_STYLING,
};
