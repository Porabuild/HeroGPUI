//! feedback reference metadata.

use super::*;

const PROGRESS_BAR_REQUIRED_PARTS: &[&str] = &[
    // Keep the compound anatomy visible to reference_audit.py.
    "ProgressBar",
    "ProgressBar.Output",
    "ProgressBar.Track",
    "ProgressBar.Fill",
];

const PROGRESS_BAR_API: &[ApiDoc] = &[
    ApiDoc {
        owner: "ProgressBar",
        prop: "value",
        ty: "number",
        default: "0",
        description: "Current value, clamped to the configured range.",
        rust_owner: "ProgressBar",
        rust: "value(f32)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "ProgressBar",
        prop: "minValue",
        ty: "number",
        default: "0",
        description: "Minimum value used to normalize the fill.",
        rust_owner: "ProgressBar",
        rust: "min_value(f32)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "ProgressBar",
        prop: "maxValue",
        ty: "number",
        default: "100",
        description: "Maximum value used to normalize the fill.",
        rust_owner: "ProgressBar",
        rust: "max_value(f32)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "ProgressBar",
        prop: "isIndeterminate",
        ty: "boolean",
        default: "false",
        description: "Shows the pinned unbounded sweep without reporting a value label.",
        rust_owner: "ProgressBar",
        rust: "is_indeterminate(bool)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "ProgressBar",
        prop: "size",
        ty: "\"sm\" | \"md\" | \"lg\"",
        default: "\"md\"",
        description: "Selects the 4, 8 or 12px track height and matching radius.",
        rust_owner: "ProgressBar",
        rust: "size(Size)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "ProgressBar",
        prop: "color",
        ty: "\"default\" | \"accent\" | \"success\" | \"warning\" | \"danger\"",
        default: "\"accent\"",
        description: "Semantic fill color; default uses the contrasting default foreground.",
        rust_owner: "ProgressBar",
        rust: "color(Color)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "ProgressBar",
        prop: "formatOptions",
        ty: "Intl.NumberFormatOptions",
        default: "{style: \"percent\"}",
        description: "Formats valueText; the local formatter covers common numeric styles without locale or CLDR data.",
        rust_owner: "ProgressBar",
        rust: "format_options(NumberFormat)",
        status: ImplementationStatus::Partial,
    },
    ApiDoc {
        owner: "ProgressBar",
        prop: "valueLabel",
        ty: "ReactNode",
        default: "—",
        description: "Replaces the generated formatted value text; the port accepts text rather than arbitrary content.",
        rust_owner: "ProgressBar",
        rust: "value_label(text)",
        status: ImplementationStatus::Partial,
    },
    ApiDoc {
        owner: "ProgressBar",
        prop: "children",
        ty: "ReactNode | (values: ProgressBarRenderProps) => ReactNode",
        default: "—",
        description: "The local ValueLabel closure receives all documented values, while the root anatomy is composed through dedicated builders.",
        rust_owner: "ProgressBar",
        rust: "label(text) + value_content(render) + show_value_label(true)",
        status: ImplementationStatus::Partial,
    },
    ApiDoc {
        owner: "ProgressBarRenderProps",
        prop: "percentage",
        ty: "number",
        default: "—",
        description: "Normalized percentage; the port uses 0 as the indeterminate stand-in for React Aria's undefined value.",
        rust_owner: "ProgressBar",
        rust: "value_content(render)",
        status: ImplementationStatus::Partial,
    },
    ApiDoc {
        owner: "ProgressBarRenderProps",
        prop: "valueText",
        ty: "string",
        default: "—",
        description: "Formatted text, or an empty string while indeterminate.",
        rust_owner: "ProgressBar",
        rust: "value_content(render)",
        status: ImplementationStatus::Partial,
    },
    ApiDoc {
        owner: "ProgressBarRenderProps",
        prop: "isIndeterminate",
        ty: "boolean",
        default: "—",
        description: "Current indeterminate state handed to the value renderer.",
        rust_owner: "ProgressBar",
        rust: "value_content(render)",
        status: ImplementationStatus::Implemented,
    },
];

const PROGRESS_BAR_PARTS: &[PartDoc] = &[
    PartDoc {
        name: "ProgressBar",
        slot: "progress-bar",
        description: "Root layout and progress state owner.",
        rust_owner: "ProgressBar",
        status: ImplementationStatus::Implemented,
    },
    PartDoc {
        name: "ProgressBar.Output",
        slot: "progress-bar-output",
        description:
            "Formatted output beside the label; customized through the value-content closure.",
        rust_owner: "ProgressBar",
        status: ImplementationStatus::Partial,
    },
    PartDoc {
        name: "ProgressBar.Track",
        slot: "progress-bar-track",
        description: "Default-color clipped track; it is not separately composable.",
        rust_owner: "ProgressBar",
        status: ImplementationStatus::Partial,
    },
    PartDoc {
        name: "ProgressBar.Fill",
        slot: "progress-bar-fill",
        description:
            "Semantic determinate fill or indeterminate sweep; it is not separately composable.",
        rust_owner: "ProgressBar",
        status: ImplementationStatus::Partial,
    },
];

const PROGRESS_BAR_STATES: &[StateDoc] = &[
    StateDoc {
        state: "Determinate",
        selector: "[aria-valuenow]",
        description: "Fill width and output follow the normalized value.",
        rust: "fraction_of(value, min_value, max_value)",
        status: ImplementationStatus::Implemented,
    },
    StateDoc {
        state: "Indeterminate",
        selector: ":not([aria-valuenow])",
        description: "A 40% fill sweeps with the pinned 1.5s curve; reduced motion leaves it static.",
        rust: "is_indeterminate + progress-bar-indeterminate + reduce_motion",
        status: ImplementationStatus::Implemented,
    },
    StateDoc {
        state: "Disabled",
        selector: ":disabled / [data-disabled=\"true\"] / [aria-disabled=\"true\"]",
        description: "The stylesheet declares disabled treatment, but the documented API has no disabled prop and the port does not invent one.",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
];

const PROGRESS_BAR_STYLING: &[StyleDoc] = &[
    StyleDoc {
        class_or_token: ".progress-bar",
        value: "grid w-full gap-1; label/output above track",
        description: "The port uses an equivalent full-width column with a justified label/output row.",
        rust: "flex_col + gap(px(4.)) + justify_between",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: "[data-slot=label] / .progress-bar__output",
        value: "text-sm font-medium; output tabular-nums",
        description: "Text size, 20px line height and weight match; GPUI does not request tabular numeral font features on the output alone.",
        rust: "text_size(px(14.)) + line_height(px(20.)) + FontWeight::MEDIUM",
        status: ImplementationStatus::Partial,
    },
    StyleDoc {
        class_or_token: ".progress-bar__track",
        value: "h-2 rounded-sm bg-default; sm h-1 rounded-xs; lg h-3 rounded-md",
        description: "Track heights, radii, clipping and semantic background follow the pinned size variants.",
        rust: "Size => (4/8/12px, micro/hairline/mark radius) + colors.default.color",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".progress-bar__fill",
        value: "absolute start-0 top-0 h-full; matching size radius",
        description: "The local fill occupies the track height and uses its radius.",
        rust: "relative fill + h_full + rounded(radius)",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".progress-bar--default",
        value: "--progress-bar-fill: var(--default-foreground)",
        description: "Default uses the contrasting foreground; other variants use their semantic base color.",
        rust: "progress_fill_color",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".progress-bar__fill",
        value: "width 300ms ease-out; motion-reduce transition-none",
        description: "Determinate changes interpolate from the currently rendered width and preserve position during reversal.",
        rust: "PROGRESS_BAR_FILL_MS + Curve::Out + progress_bar_motion",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: "@keyframes progress-bar-indeterminate",
        value: "40% width; translateX(-100% to 350%); 1.5s cubic-bezier(0.65,0,0.35,1) infinite",
        description: "The listener-free fill reproduces the pinned unbounded sweep.",
        rust: "PROGRESS_BAR_INDETERMINATE_MS + progress_bar_indeterminate_ease + relative(delta * 1.8 - 0.4)",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: "motion-reduce:animate-none",
        value: "static 40% fill",
        description: "Reduced motion keeps the indeterminate state legible without sweeping.",
        rust: "reduce_motion + relative(0.4)",
        status: ImplementationStatus::Implemented,
    },
];

pub(super) const PROGRESS_BAR: ReferenceMetadata = ReferenceMetadata {
    page: "ProgressBar",
    import_line: "use herogpui::components::progress::ProgressBar;",
    source_module: "progress",
    version: "3.2.4",
    docs_source: "https://github.com/heroui-inc/heroui/blob/v3.2.4/apps/docs/content/docs/en/react/components/(feedback)/progress-bar.mdx",
    api_source: "https://github.com/heroui-inc/heroui/blob/v3.2.4/packages/react/src/components/progress-bar/progress-bar.tsx + https://github.com/adobe/react-spectrum/blob/react-aria-components@1.20.0/packages/react-aria-components/src/ProgressBar.tsx + https://github.com/adobe/react-spectrum/blob/react-aria@3.51.0/packages/react-aria/src/progress/useProgressBar.ts",
    style_source: "https://github.com/heroui-inc/heroui/blob/v3.2.4/packages/styles/components/progress-bar.css",
    required_parts: PROGRESS_BAR_REQUIRED_PARTS,
    api: PROGRESS_BAR_API,
    parts: PROGRESS_BAR_PARTS,
    states: PROGRESS_BAR_STATES,
    styling: PROGRESS_BAR_STYLING,
};

const PROGRESS_CIRCLE_REQUIRED_PARTS: &[&str] = &[
    // Keep the compound anatomy visible to reference_audit.py.
    "ProgressCircle",
    "ProgressCircle.Track",
    "ProgressCircle.TrackCircle",
    "ProgressCircle.FillCircle",
];

const PROGRESS_CIRCLE_API: &[ApiDoc] = &[
    ApiDoc {
        owner: "ProgressCircle",
        prop: "id",
        ty: "string",
        default: "—",
        description: "Optional GPUI identity. A named ring reports role=progressbar with the same value range as ProgressBar.",
        rust_owner: "ProgressCircle",
        rust: "id(ElementId)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "ProgressCircle",
        prop: "value",
        ty: "number",
        default: "0",
        description: "Current progress value, clamped to the configured range.",
        rust_owner: "ProgressCircle",
        rust: "value(f32)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "ProgressCircle",
        prop: "minValue",
        ty: "number",
        default: "0",
        description: "Minimum value used to normalize progress.",
        rust_owner: "ProgressCircle",
        rust: "min_value(f32)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "ProgressCircle",
        prop: "maxValue",
        ty: "number",
        default: "100",
        description: "Maximum value used to normalize progress.",
        rust_owner: "ProgressCircle",
        rust: "max_value(f32)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "ProgressCircle",
        prop: "isIndeterminate",
        ty: "boolean",
        default: "false",
        description: "Shows a spinning quarter arc without exposing a value label.",
        rust_owner: "ProgressCircle",
        rust: "is_indeterminate(bool)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "ProgressCircle",
        prop: "size",
        ty: "\"sm\" | \"md\" | \"lg\"",
        default: "\"md\"",
        description: "Selects the pinned 20, 28 or 36px track diameter.",
        rust_owner: "ProgressCircle",
        rust: "size(Size)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "ProgressCircle",
        prop: "color",
        ty: "\"default\" | \"accent\" | \"success\" | \"warning\" | \"danger\"",
        default: "\"accent\"",
        description: "Semantic fill-circle stroke color.",
        rust_owner: "ProgressCircle",
        rust: "color(Color)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "ProgressCircle",
        prop: "formatOptions",
        ty: "Intl.NumberFormatOptions",
        default: "{style: \"percent\"}",
        description: "Formats valueText; the local formatter covers common numeric styles without locale or CLDR data.",
        rust_owner: "ProgressCircle",
        rust: "format_options(NumberFormat)",
        status: ImplementationStatus::Partial,
    },
    ApiDoc {
        owner: "ProgressCircle",
        prop: "children",
        ty: "ReactNode | (values: ProgressCircleRenderProps) => ReactNode",
        default: "—",
        description: "The local ValueLabel closure receives all three documented values, but arbitrary root children are not composed.",
        rust_owner: "ProgressCircle",
        rust: "value_content(render) + show_value_label(true)",
        status: ImplementationStatus::Partial,
    },
    ApiDoc {
        owner: "ProgressCircleRenderProps",
        prop: "percentage",
        ty: "number",
        default: "—",
        description: "Normalized percentage handed to the value renderer.",
        rust_owner: "ProgressCircle",
        rust: "value_content(render)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "ProgressCircleRenderProps",
        prop: "valueText",
        ty: "string",
        default: "—",
        description: "Formatted value text handed to the value renderer.",
        rust_owner: "ProgressCircle",
        rust: "value_content(render)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "ProgressCircleRenderProps",
        prop: "isIndeterminate",
        ty: "boolean",
        default: "—",
        description: "Current indeterminate state handed to the value renderer.",
        rust_owner: "ProgressCircle",
        rust: "value_content(render)",
        status: ImplementationStatus::Implemented,
    },
];

const PROGRESS_CIRCLE_PARTS: &[PartDoc] = &[
    PartDoc {
        name: "ProgressCircle",
        slot: "progress-circle",
        description: "Root progress indicator and fixed-size layout box.",
        rust_owner: "ProgressCircle",
        status: ImplementationStatus::Implemented,
    },
    PartDoc {
        name: "ProgressCircle.Track",
        slot: "progress-circle-track",
        description: "SVG track counterpart represented by a border ring and canvas arc; it is not separately composable.",
        rust_owner: "ProgressCircle",
        status: ImplementationStatus::Partial,
    },
    PartDoc {
        name: "ProgressCircle.TrackCircle",
        slot: "progress-circle-track-circle",
        description: "Full default-color ring beneath the progress arc.",
        rust_owner: "ProgressCircle",
        status: ImplementationStatus::Partial,
    },
    PartDoc {
        name: "ProgressCircle.FillCircle",
        slot: "progress-circle-fill-circle",
        description: "Semantic-color canvas arc with normalized sweep; custom SVG attributes are unavailable.",
        rust_owner: "ProgressCircle",
        status: ImplementationStatus::Partial,
    },
];

const PROGRESS_CIRCLE_STATES: &[StateDoc] = &[
    StateDoc {
        state: "Determinate",
        selector: "[aria-valuenow]",
        description: "Arc sweep and value renderer follow the normalized value.",
        rust: "fraction_of(value, min_value, max_value)",
        status: ImplementationStatus::Implemented,
    },
    StateDoc {
        state: "Indeterminate",
        selector: ":not([aria-valuenow])",
        description: "A quarter arc completes one linear turn per second; reduced motion leaves the same arc static.",
        rust: "is_indeterminate + progress-circle-spin + reduce_motion",
        status: ImplementationStatus::Implemented,
    },
    StateDoc {
        state: "Disabled",
        selector: ":disabled / [data-disabled=\"true\"] / [aria-disabled=\"true\"]",
        description: "The stylesheet has a disabled selector, but the documented component has no disabled prop and the port does not invent one.",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
];

const PROGRESS_CIRCLE_STYLING: &[StyleDoc] = &[
    StyleDoc {
        class_or_token: ".progress-circle",
        value: "inline-flex items-center justify-center",
        description: "Centered fixed-size root.",
        rust: "flex + items_center + justify_center",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".progress-circle__track",
        value: "size-7; sm size-5; lg size-9",
        description: "Pinned desktop diameters are 28px by default, 20px small and 36px large.",
        rust: "size(Size) => 20 / 28 / 36px",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".progress-circle__track-circle",
        value: "stroke: var(--default)",
        description: "Full-opacity semantic default track stroke.",
        rust: "border_color(colors.default.color)",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".progress-circle__fill-circle",
        value: "stroke-width 4 in a 36-unit viewBox",
        description: "Stroke scales to exactly one ninth of the rendered diameter.",
        rust: "stroke_w = size_px / 9",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".progress-circle__fill-circle",
        value: "stroke-linecap: round",
        description: "Filled endpoint discs reproduce the SVG arc's round line caps.",
        rust: "PathBuilder::stroke + filled endpoint discs",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".progress-circle__fill-circle",
        value: "stroke-dashoffset 300ms ease-out; motion-reduce none",
        description: "Value changes redraw immediately because the local canvas has no retained dash-offset transition.",
        rust: "direct fraction repaint",
        status: ImplementationStatus::Partial,
    },
    StyleDoc {
        class_or_token: ".progress-circle--default",
        value: "--progress-circle-stroke: var(--default-foreground)",
        description: "Default uses the contrasting foreground while other variants use their semantic base color.",
        rust: "Color::Default => colors.default.foreground",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: "@keyframes progress-circle-spin",
        value: "1s linear infinite; motion-reduce animate-none",
        description: "Indeterminate track rotation and its reduced-motion override.",
        rust: "PROGRESS_CIRCLE_SPIN_MS + progress_circle_spin_turn + reduce_motion",
        status: ImplementationStatus::Implemented,
    },
];

pub(super) const PROGRESS_CIRCLE: ReferenceMetadata = ReferenceMetadata {
    page: "ProgressCircle",
    import_line: "use herogpui::components::progress::ProgressCircle;",
    source_module: "progress",
    version: "3.2.4",
    docs_source: "https://github.com/heroui-inc/heroui/blob/v3.2.4/apps/docs/content/docs/en/react/components/(feedback)/progress-circle.mdx",
    api_source: "https://github.com/heroui-inc/heroui/blob/v3.2.4/packages/react/src/components/progress-circle/progress-circle.tsx + https://github.com/adobe/react-spectrum/blob/react-aria-components@1.20.0/packages/react-aria-components/src/ProgressBar.tsx",
    style_source: "https://github.com/heroui-inc/heroui/blob/v3.2.4/packages/styles/components/progress-circle.css",
    required_parts: PROGRESS_CIRCLE_REQUIRED_PARTS,
    api: PROGRESS_CIRCLE_API,
    parts: PROGRESS_CIRCLE_PARTS,
    states: PROGRESS_CIRCLE_STATES,
    styling: PROGRESS_CIRCLE_STYLING,
};

const ALERT_REQUIRED_PARTS: &[&str] = &[
    "Alert",
    "Alert.Indicator",
    "Alert.Content",
    "Alert.Title",
    "Alert.Description",
];

const ALERT_API: &[ApiDoc] = &[
    ApiDoc {
        owner: "Alert",
        prop: "status",
        ty: "\"default\" | \"accent\" | \"success\" | \"warning\" | \"danger\"",
        default: "\"default\"",
        description: "The visual status of the alert. It recolours the indicator glyph and the title text only — the container stays `bg-surface` for every status.",
        rust_owner: "Alert",
        rust: "status(Color)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "Alert",
        prop: "children",
        ty: "ReactNode",
        default: "—",
        description: "The alert content, typically Title and Description plus additional elements like the composed `CloseButton` the pinned examples use (v3 removed `isClosable`/`onClose`). The port takes the title and description as builders and appends the rest as composed children after the content column.",
        rust_owner: "Alert",
        rust: "new(title) / description(text) / ParentElement::extend",
        status: ImplementationStatus::Partial,
    },
    ApiDoc {
        owner: "Alert",
        prop: "className",
        ty: "string",
        default: "—",
        description: "Additional CSS classes.",
        rust_owner: "Alert",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
    ApiDoc {
        owner: "Alert.Indicator",
        prop: "children",
        ty: "ReactNode",
        default: "—",
        description: "Custom indicator icon, defaulting to the status glyph. The port draws only the pinned per-status glyph (Info for default and accent, check for success, triangle for warning, circle-exclamation for danger); there is no builder to replace it.",
        rust_owner: "Alert",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
    ApiDoc {
        owner: "Alert.Indicator",
        prop: "className",
        ty: "string",
        default: "—",
        description: "Additional CSS classes.",
        rust_owner: "Alert",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
    ApiDoc {
        owner: "Alert.Content",
        prop: "children",
        ty: "ReactNode",
        default: "—",
        description: "The content column beside the indicator. The port folds the column into the root and exposes its typical content as the title and description builders; composed children land after it instead of inside it.",
        rust_owner: "Alert",
        rust: "new(title) / description(text)",
        status: ImplementationStatus::Partial,
    },
    ApiDoc {
        owner: "Alert.Content",
        prop: "className",
        ty: "string",
        default: "—",
        description: "Additional CSS classes.",
        rust_owner: "Alert",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
    ApiDoc {
        owner: "Alert.Title",
        prop: "children",
        ty: "ReactNode",
        default: "—",
        description: "The alert title text. The port takes plain title text; arbitrary elements have no title builder.",
        rust_owner: "Alert",
        rust: "new(title)",
        status: ImplementationStatus::Partial,
    },
    ApiDoc {
        owner: "Alert.Title",
        prop: "className",
        ty: "string",
        default: "—",
        description: "Additional CSS classes.",
        rust_owner: "Alert",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
    ApiDoc {
        owner: "Alert.Description",
        prop: "children",
        ty: "ReactNode",
        default: "—",
        description: "The alert description text. The port takes plain description text; omitting it renders no description row at all.",
        rust_owner: "Alert",
        rust: "description(text)",
        status: ImplementationStatus::Partial,
    },
    ApiDoc {
        owner: "Alert.Description",
        prop: "className",
        ty: "string",
        default: "—",
        description: "Additional CSS classes.",
        rust_owner: "Alert",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
];

const ALERT_PARTS: &[PartDoc] = &[
    PartDoc {
        name: "Alert",
        slot: "alert",
        description: "Root card on the `.alert` BEM class: `flex w-full flex-row items-start justify-start gap-4 bg-surface px-4 py-3 shadow-surface` at `border-radius: min(32px, var(--radius-3xl))`. The status never paints the container; dark mode drops the surface shadow.",
        rust_owner: "Alert",
        status: ImplementationStatus::Implemented,
    },
    PartDoc {
        name: "Alert.Indicator",
        slot: "alert__indicator",
        description: "`flex items-center justify-center p-1 select-none` box around the `box-content size-4` status glyph, painted with the status soft-foreground (`text-foreground` on default). The port draws the pinned glyph but offers no custom indicator children.",
        rust_owner: "Alert",
        status: ImplementationStatus::Partial,
    },
    PartDoc {
        name: "Alert.Content",
        slot: "alert__content",
        description: "`flex h-full grow flex-col items-start` column beside the indicator, with no gap of its own. The port folds it into the root's layout and draws the title and description directly inside it.",
        rust_owner: "Alert",
        status: ImplementationStatus::Partial,
    },
    PartDoc {
        name: "Alert.Title",
        slot: "alert__title",
        description: "`text-sm leading-6 font-medium`, painted the status soft-foreground (`text-foreground` on default) like the indicator.",
        rust_owner: "Alert",
        status: ImplementationStatus::Implemented,
    },
    PartDoc {
        name: "Alert.Description",
        slot: "alert__description",
        description: "`text-sm text-muted` — 14px text at the `text-sm` leading (20px), directly beneath the title.",
        rust_owner: "Alert",
        status: ImplementationStatus::Implemented,
    },
];

// The v3 docs state the alert "is primarily informational and doesn't have
// interactive states on the base component" and `.alert` declares no state
// utilities, so there are no states to record.
const ALERT_STATES: &[StateDoc] = &[];

const ALERT_STYLING: &[StyleDoc] = &[
    StyleDoc {
        class_or_token: ".alert",
        value: "flex w-full flex-row items-start justify-start gap-4 bg-surface px-4 py-3 shadow-surface; border-radius: min(32px, var(--radius-3xl))",
        description: "68px card for title plus description: 12 + 24 + 20 + 12 of pure leading, the 16px gap-4 between the 24px indicator box and the content column.",
        rust: "flex + items_start + justify_start + gap(px(16.)) + w_full + px(px(16.)) + py(px(12.)) + control_radius + colors.surface.background + surface_shadow",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".alert__content",
        value: "flex h-full grow flex-col items-start",
        description: "The content column grows beside the indicator and stacks the title and description with no gap utility of its own; `h-full` is a no-op in the port's auto-height card.",
        rust: "flex + flex_col + items_start + flex_1",
        status: ImplementationStatus::Partial,
    },
    StyleDoc {
        class_or_token: ".alert__indicator",
        value: "flex items-center justify-center p-1 select-none; [data-slot=\"alert-default-icon\"] box-content size-4",
        description: "A 24px box (4 + 16 + 4) around the 16px glyph. `select-none` has no GPUI analogue, and the box is `flex_shrink_0` so the card cannot compress it.",
        rust: "p(px(4.)) + items_center + justify_center + flex_shrink_0 + svg size(px(16.))",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".alert__title",
        value: "text-sm leading-6 font-medium",
        description: "14px title over a 24px line box at medium weight — never semibold.",
        rust: "text_size(px(14.)) + line_height(px(24.)) + FontWeight::MEDIUM",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".alert__description",
        value: "text-sm text-muted",
        description: "14px description at the `text-sm` leading (20px), muted rather than the inherited foreground.",
        rust: "text_size(px(14.)) + line_height(px(20.)) + colors.muted",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".alert--{status} .alert__title / .alert__indicator",
        value: "text-foreground on default; text-{role}-soft-foreground otherwise",
        description: "The five status classes only recolour the title and the indicator; `.alert--default` restores the plain foreground for both.",
        rust: "role_fg = colors.foreground on Default, sem.soft_foreground(colors.foreground) otherwise",
        status: ImplementationStatus::Implemented,
    },
];

pub(super) const ALERT: ReferenceMetadata = ReferenceMetadata {
    page: "Alert",
    import_line: "use herogpui::components::alert::Alert;",
    source_module: "alert",
    version: "3.2.4",
    docs_source: "https://github.com/heroui-inc/heroui/blob/v3.2.4/apps/docs/content/docs/en/react/components/(feedback)/alert.mdx",
    api_source: "https://github.com/heroui-inc/heroui/blob/v3.2.4/packages/react/src/components/alert/alert.tsx",
    style_source: "https://github.com/heroui-inc/heroui/blob/v3.2.4/packages/styles/components/alert.css",
    required_parts: ALERT_REQUIRED_PARTS,
    api: ALERT_API,
    parts: ALERT_PARTS,
    states: ALERT_STATES,
    styling: ALERT_STYLING,
};

const METER_REQUIRED_PARTS: &[&str] = &["Meter", "Meter.Output", "Meter.Track", "Meter.Fill"];

const METER_API: &[ApiDoc] = &[
    ApiDoc { owner: "Meter", prop: "value", ty: "number", default: "0", description: "Current value within the configured range.", rust_owner: "Meter", rust: "new(id, value) / value(f32)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "Meter", prop: "minValue", ty: "number", default: "0", description: "Minimum value used to normalize the fill.", rust_owner: "Meter", rust: "min_value(f32)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "Meter", prop: "maxValue", ty: "number", default: "100", description: "Maximum value used to normalize the fill.", rust_owner: "Meter", rust: "max_value(f32)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "Meter", prop: "size", ty: "'sm' | 'md' | 'lg'", default: "'md'", description: "Selects the meter track thickness.", rust_owner: "Meter", rust: "size(Size)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "Meter", prop: "color", ty: "'default' | 'accent' | 'success' | 'warning' | 'danger'", default: "'accent'", description: "Semantic color of the fill bar.", rust_owner: "Meter", rust: "color(Color)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "Meter", prop: "formatOptions", ty: "Intl.NumberFormatOptions", default: "{style: 'percent'}", description: "Formats the value label; the port covers common numeric styles without caller-selected locale data.", rust_owner: "Meter", rust: "format_options(NumberFormat)", status: ImplementationStatus::Partial },
    ApiDoc { owner: "Meter", prop: "valueLabel", ty: "ReactNode", default: "—", description: "Replaces the generated value text; the port accepts text or a value render closure.", rust_owner: "Meter", rust: "value_label(text) / value_content(render)", status: ImplementationStatus::Partial },
    ApiDoc { owner: "Meter", prop: "children", ty: "ReactNode | (values: MeterRenderProps) => ReactNode", default: "—", description: "The port composes a label and output renderer while track and fill remain built in.", rust_owner: "Meter", rust: "label(text) + value_content(render)", status: ImplementationStatus::Partial },
    ApiDoc { owner: "MeterRenderProps", prop: "percentage", ty: "number", default: "—", description: "Normalized percentage handed to the output renderer.", rust_owner: "Meter", rust: "value_content(render)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "MeterRenderProps", prop: "valueText", ty: "string", default: "—", description: "Formatted value text handed to the output renderer.", rust_owner: "Meter", rust: "value_content(render)", status: ImplementationStatus::Implemented },
];

const METER_PARTS: &[PartDoc] = &[
    PartDoc {
        name: "Meter",
        slot: "meter",
        description: "Root meter state and layout owner.",
        rust_owner: "Meter",
        status: ImplementationStatus::Implemented,
    },
    PartDoc {
        name: "Meter.Output",
        slot: "meter-output",
        description: "Formatted output beside the label, customizable through the value closure.",
        rust_owner: "Meter",
        status: ImplementationStatus::Partial,
    },
    PartDoc {
        name: "Meter.Track",
        slot: "meter-track",
        description: "Clipped default-color track delegated to ProgressBar.",
        rust_owner: "Meter",
        status: ImplementationStatus::Partial,
    },
    PartDoc {
        name: "Meter.Fill",
        slot: "meter-fill",
        description: "Semantic fill proportional to the normalized value.",
        rust_owner: "Meter",
        status: ImplementationStatus::Partial,
    },
];

const METER_STATES: &[StateDoc] = &[StateDoc {
    state: "Determinate",
    selector: "[aria-valuenow]",
    description: "Fill width and output follow the normalized value.",
    rust: "ProgressBar::value + min_value + max_value",
    status: ImplementationStatus::Implemented,
}];

const METER_STYLING: &[StyleDoc] = &[
    StyleDoc {
        class_or_token: ".meter",
        value: "grid w-full gap-1",
        description:
            "Full-width label/output row above the track; 14px text uses a 20px line height.",
        rust: "ProgressBar root layout",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".meter__track",
        value: "h-2 rounded-sm; sm h-1 rounded-xs; lg h-3 rounded-md",
        description: "Track thickness follows the three documented sizes.",
        rust: "ProgressBar::size(Size)",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".meter__fill",
        value: "matching track radius; semantic fill",
        description: "Fill color follows the selected semantic role.",
        rust: "ProgressBar::color(Color)",
        status: ImplementationStatus::Implemented,
    },
];

pub(super) const METER: ReferenceMetadata = ReferenceMetadata {
    page: "Meter",
    import_line: "use herogpui::components::meter::Meter;",
    source_module: "meter",
    version: "3.2.4",
    docs_source: "https://github.com/heroui-inc/heroui/blob/v3.2.4/apps/docs/content/docs/en/react/components/(feedback)/meter.mdx",
    api_source: "https://github.com/heroui-inc/heroui/blob/v3.2.4/packages/react/src/components/meter/meter.tsx + https://github.com/adobe/react-spectrum/blob/react-aria-components@1.20.0/packages/react-aria-components/src/Meter.tsx",
    style_source: "https://github.com/heroui-inc/heroui/blob/v3.2.4/packages/styles/components/meter.css",
    required_parts: METER_REQUIRED_PARTS,
    api: METER_API,
    parts: METER_PARTS,
    states: METER_STATES,
    styling: METER_STYLING,
};

const SKELETON_REQUIRED_PARTS: &[&str] = &["Skeleton"];

const SKELETON_API: &[ApiDoc] = &[
    ApiDoc {
        owner: "Skeleton",
        prop: "animationType",
        ty: "'shimmer' | 'pulse' | 'none'",
        default: "theme",
        description:
            "Selects shimmer, pulse, or no animation; the theme token supplies the default.",
        rust_owner: "Skeleton",
        rust: "animation_type(SkeletonAnimation)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "Skeleton",
        prop: "className",
        ty: "string",
        default: "—",
        description: "Browser CSS classes are unavailable.",
        rust_owner: "Skeleton",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
];

const SKELETON_PARTS: &[PartDoc] = &[PartDoc {
    name: "Skeleton",
    slot: "skeleton",
    description: "Clipped placeholder surface with optional hidden layout content.",
    rust_owner: "Skeleton",
    status: ImplementationStatus::Implemented,
}];

const SKELETON_STATES: &[StateDoc] = &[
    StateDoc {
        state: "Shimmer",
        selector: ".skeleton--shimmer",
        description: "A highlight band sweeps across the placeholder.",
        rust: "SkeletonAnimation::Shimmer",
        status: ImplementationStatus::Implemented,
    },
    StateDoc {
        state: "Pulse",
        selector: ".skeleton--pulse",
        description: "The placeholder opacity pulses.",
        rust: "SkeletonAnimation::Pulse",
        status: ImplementationStatus::Implemented,
    },
    StateDoc {
        state: "No animation",
        selector: ".skeleton--none",
        description:
            "A static placeholder is rendered, including when reduced motion is requested.",
        rust: "SkeletonAnimation::None / reduce_motion",
        status: ImplementationStatus::Implemented,
    },
];

const SKELETON_STYLING: &[StyleDoc] = &[
    StyleDoc {
        class_or_token: ".skeleton",
        value: "relative overflow-hidden rounded-lg bg-surface-secondary/50",
        description: "Theme secondary-surface placeholder clipped to the shared hairline radius.",
        rust: "surface_tertiary + hairline_radius + overflow_hidden",
        status: ImplementationStatus::Partial,
    },
    StyleDoc {
        class_or_token: ".skeleton--shimmer",
        value: "1.4s highlight sweep",
        description: "A translucent background-colored band sweeps left to right.",
        rust: "SkeletonAnimation::Shimmer + 1400ms repeated animation",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".skeleton--pulse",
        value: "animate-pulse",
        description: "Opacity oscillates while motion is enabled.",
        rust: "SkeletonAnimation::Pulse + 1600ms repeated animation",
        status: ImplementationStatus::Implemented,
    },
];

pub(super) const SKELETON: ReferenceMetadata = ReferenceMetadata {
    page: "Skeleton",
    import_line: "use herogpui::components::skeleton::Skeleton;",
    source_module: "skeleton",
    version: "3.2.4",
    docs_source: "https://github.com/heroui-inc/heroui/blob/v3.2.4/apps/docs/content/docs/en/react/components/(feedback)/skeleton.mdx",
    api_source: "https://github.com/heroui-inc/heroui/blob/v3.2.4/packages/react/src/components/skeleton/skeleton.tsx",
    style_source: "https://github.com/heroui-inc/heroui/blob/v3.2.4/packages/styles/components/skeleton.css",
    required_parts: SKELETON_REQUIRED_PARTS,
    api: SKELETON_API,
    parts: SKELETON_PARTS,
    states: SKELETON_STATES,
    styling: SKELETON_STYLING,
};

const SPINNER_REQUIRED_PARTS: &[&str] = &["Spinner"];

const SPINNER_API: &[ApiDoc] = &[
    ApiDoc {
        owner: "Spinner",
        prop: "size",
        ty: "'sm' | 'md' | 'lg' | 'xl'",
        default: "'md'",
        description: "Selects the 16, 24, 32 or 40px indicator size.",
        rust_owner: "Spinner",
        rust: "size(SpinnerSize)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "Spinner",
        prop: "color",
        ty: "'current' | 'accent' | 'success' | 'warning' | 'danger'",
        default: "'accent'",
        description: "Selects a semantic color or a caller-resolved current text color.",
        rust_owner: "Spinner",
        rust: "color(Color) / current_color(Hsla)",
        status: ImplementationStatus::Partial,
    },
    ApiDoc {
        owner: "Spinner",
        prop: "className",
        ty: "string",
        default: "—",
        description: "Browser CSS classes and animation utilities are unavailable.",
        rust_owner: "Spinner",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
];

const SPINNER_PARTS: &[PartDoc] = &[PartDoc {
    name: "Spinner",
    slot: "spinner",
    description: "Rotating arc indicator with semantic color and fixed size.",
    rust_owner: "Spinner",
    status: ImplementationStatus::Implemented,
}];

const SPINNER_STATES: &[StateDoc] = &[StateDoc {
    state: "Spinning",
    selector: ".spinner",
    description: "The arc rotates continuously at the configured duration.",
    rust: "Animation::repeat + Transformation::rotate",
    status: ImplementationStatus::Implemented,
}];

const SPINNER_STYLING: &[StyleDoc] = &[
    StyleDoc { class_or_token: ".spinner", value: "size-6 animate-spin", description: "The default indicator keeps its 24px diameter in flex layouts and rotates while motion is enabled; duration_ms provides the gallery's speed customization point.", rust: "SpinnerSize::Md + duration_ms(u64) + repeated rotation", status: ImplementationStatus::Implemented },
    StyleDoc { class_or_token: ".spinner--sm / --md / --lg / --xl", value: "16px / 24px / 32px / 40px", description: "All four documented diameters map directly.", rust: "SpinnerSize::px", status: ImplementationStatus::Implemented },
    StyleDoc { class_or_token: ".spinner--current / semantic colors", value: "currentColor or semantic role", description: "GPUI SVGs require current text color to be resolved by the caller; semantic roles resolve from the active theme.", rust: "current_color(Hsla) / color(Color)", status: ImplementationStatus::Partial },
];

pub(super) const SPINNER: ReferenceMetadata = ReferenceMetadata {
    page: "Spinner",
    import_line: "use herogpui::components::spinner::Spinner;",
    source_module: "spinner",
    version: "3.2.4",
    docs_source: "https://github.com/heroui-inc/heroui/blob/v3.2.4/apps/docs/content/docs/en/react/components/(feedback)/spinner.mdx",
    api_source: "https://github.com/heroui-inc/heroui/blob/v3.2.4/packages/react/src/components/spinner/spinner.tsx",
    style_source: "https://github.com/heroui-inc/heroui/blob/v3.2.4/packages/styles/components/spinner.css",
    required_parts: SPINNER_REQUIRED_PARTS,
    api: SPINNER_API,
    parts: SPINNER_PARTS,
    states: SPINNER_STATES,
    styling: SPINNER_STYLING,
};
