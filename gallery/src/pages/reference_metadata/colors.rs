//! colors reference metadata.

use super::*;

const COLOR_AREA_REQUIRED_PARTS: &[&str] = &["ColorArea", "ColorArea.Thumb"];

const COLOR_AREA_API: &[ApiDoc] = &[
    ApiDoc { owner: "ColorArea", prop: "value", ty: "string | Color", default: "—", description: "Controlled color; pointer and keyboard changes wait for owner acceptance.", rust_owner: "ColorArea", rust: "new(id, PickerColor)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorArea", prop: "defaultValue", ty: "string | Color", default: "—", description: "Seeds picker-owned color state once.", rust_owner: "ColorArea", rust: "default_value(PickerColor)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorArea", prop: "onChange", ty: "(color: Color) => void", default: "—", description: "Reports each changed pointer coordinate or keyboard step.", rust_owner: "ColorArea", rust: "on_change(callback)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorArea", prop: "onChangeEnd", ty: "(color: Color) => void", default: "—", description: "Reports once on pointer release and once for each completed keyboard change.", rust_owner: "ColorArea", rust: "on_change_end(callback)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorArea", prop: "xChannel", ty: "ColorChannel", default: "saturation", description: "Selects the horizontal channel and its pinned step/range.", rust_owner: "ColorArea", rust: "x_channel(ColorChannel)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorArea", prop: "yChannel", ty: "ColorChannel", default: "brightness", description: "Selects the vertical channel, increasing toward the top.", rust_owner: "ColorArea", rust: "y_channel(ColorChannel)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorArea", prop: "colorSpace", ty: "ColorSpace", default: "—", description: "Selects HSB, HSL, or RGB channel interpretation and default axes.", rust_owner: "ColorArea", rust: "color_space(ColorSpace)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorArea", prop: "isDisabled", ty: "boolean", default: "false", description: "Dims the complete area and removes focus, pointer, and keyboard interaction.", rust_owner: "ColorArea", rust: "is_disabled(bool)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorArea", prop: "showDots", ty: "boolean", default: "false", description: "Overlays the pinned eight-pixel precision grid.", rust_owner: "ColorArea", rust: "show_dots(bool)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorArea", prop: "children", ty: "ReactNode | RenderFunction", default: "ColorArea.Thumb", description: "The monolithic area always owns the thumb; its child render state is exposed through the thumb builder.", rust_owner: "ColorArea", rust: "thumb(render)", status: ImplementationStatus::Partial },
    ApiDoc { owner: "ColorArea", prop: "className / render", ty: "string / DOMRenderFunction", default: "—", description: "Browser classes and DOM root substitution are unavailable; size provides the desktop geometry customization point.", rust_owner: "ColorArea", rust: "size(width, height)", status: ImplementationStatus::Partial },
    ApiDoc { owner: "ColorArea.Thumb", prop: "children", ty: "ReactNode | ((state: ColorThumbRenderProps) => ReactNode)", default: "—", description: "Caller content receives color, dragging, hovered, focused, focus-visible, and disabled state.", rust_owner: "ColorArea", rust: "thumb(render)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorArea.Thumb", prop: "className", ty: "string | RenderFunction", default: "—", description: "Browser class customization is unavailable.", rust_owner: "ColorArea", rust: "—", status: ImplementationStatus::Unavailable },
    ApiDoc { owner: "ColorArea.Thumb", prop: "style", ty: "CSSProperties | RenderFunction", default: "—", description: "Browser inline-style render functions are unavailable.", rust_owner: "ColorArea", rust: "—", status: ImplementationStatus::Unavailable },
    ApiDoc { owner: "ColorArea.Thumb", prop: "render", ty: "DOMRenderFunction", default: "—", description: "The thumb DOM element cannot be replaced; caller content is composed inside its stable owner.", rust_owner: "ColorArea", rust: "thumb(render)", status: ImplementationStatus::Partial },
];

const COLOR_AREA_PARTS: &[PartDoc] = &[
    PartDoc { name: "ColorArea", slot: "color-area", description: "Two-dimensional gradient, value owner, focus target, and pointer/keyboard interaction surface.", rust_owner: "ColorArea", status: ImplementationStatus::Implemented },
    PartDoc { name: "ColorArea.Thumb", slot: "color-area-thumb", description: "Positioned current-color indicator with live render state and drag geometry.", rust_owner: "ColorArea", status: ImplementationStatus::Implemented },
];

const COLOR_AREA_STATES: &[StateDoc] = &[
    StateDoc { state: "Disabled", selector: ".color-area[data-disabled=true] / thumb", description: "Whole-control opacity applies and every interaction/render-state flag is masked except disabled.", rust: "is_disabled", status: ImplementationStatus::Implemented },
    StateDoc { state: "Dragging", selector: ".color-area__thumb[data-dragging=true]", description: "Pointer down grows the thumb from 16px to 20px and reaches the thumb render state even when the color is unchanged.", rust: "dragging keyed state + ColorAreaThumbState", status: ImplementationStatus::Implemented },
    StateDoc { state: "Hovered", selector: "ColorThumbRenderProps.isHovered", description: "The stable thumb owner reports hover without placing listeners above a changing animation id.", rust: "thumb_hovered keyed state", status: ImplementationStatus::Implemented },
    StateDoc { state: "Focused", selector: "ColorThumbRenderProps.isFocused", description: "Pointer and keyboard focus are reported from the area focus handle.", rust: "ColorAreaThumbState", status: ImplementationStatus::Implemented },
    StateDoc { state: "Focus visible", selector: ".color-area__thumb[data-focus-visible=true]", description: "Keyboard-origin focus rings the thumb, not the whole gradient, and reaches render state.", rust: "with_focus_ring(thumb) + focus_visible", status: ImplementationStatus::Implemented },
];

const COLOR_AREA_STYLING: &[StyleDoc] = &[
    StyleDoc { class_or_token: ".color-area", value: "relative w-full max-w-56 aspect-square shrink-0 rounded-2xl", description: "The default desktop area is the pinned 224px square with 16px corners; size can override both axes.", rust: "224x224 + size(width, height) + soft_radius", status: ImplementationStatus::Implemented },
    StyleDoc { class_or_token: ".color-area inner shadow", value: "inset 0 0 0 1px rgba(0,0,0,.1)", description: "The port uses a theme border rather than the pinned translucent inner shadow.", rust: "border(theme border)", status: ImplementationStatus::Partial },
    StyleDoc { class_or_token: ".color-area--show-dots::after", value: "1px white/.2 radial dots; background-size 8px 8px", description: "Two-pixel translucent dots are centered in an eight-pixel grid because GPUI has no repeating radial background.", rust: "show_dots 8px grid", status: ImplementationStatus::Partial },
    StyleDoc { class_or_token: ".color-area__thumb", value: "size-4 rounded-xl border-[3px] border-white", description: "Idle size, circular radius, white border, and current-color fill match.", rust: "16px + rounded(12) + border(3)", status: ImplementationStatus::Implemented },
    StyleDoc { class_or_token: ".color-area__thumb shadows", value: "outer and inset 1px rgba(0,0,0,.1)", description: "The pinned thumb shadows are not drawn.", rust: "—", status: ImplementationStatus::Unavailable },
    StyleDoc { class_or_token: ".color-area__thumb transition", value: "width/height 150ms ease-out; reduced-motion none", description: "A listener-free visual child interpolates the current size, preserves reversal position, and switches directly under reduced motion.", rust: "color_area_thumb_motion", status: ImplementationStatus::Implemented },
    StyleDoc { class_or_token: ".color-area__thumb[data-dragging=true]", value: "size-5", description: "Active drag size is 20px while the stable owner keeps hover continuity.", rust: "COLOR_AREA_THUMB_DRAGGING_PX", status: ImplementationStatus::Implemented },
    StyleDoc { class_or_token: ".color-area__thumb[data-focus-visible=true]", value: "status-focused", description: "The shared status ring is applied to the animated thumb visual.", rust: "with_focus_ring(thumb)", status: ImplementationStatus::Implemented },
];

pub(super) const COLOR_AREA: ReferenceMetadata = ReferenceMetadata {
    page: "ColorArea",
    import_line: "use herogpui::components::color_picker::ColorArea;",
    source_module: "color_picker",
    version: "3.2.5",
    docs_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/apps/docs/content/docs/en/react/components/(colors)/color-area.mdx",
    api_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/packages/react/src/components/color-area/color-area.tsx + https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/ColorArea.tsx + https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/ColorThumb.tsx",
    style_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/packages/styles/components/color-area.css",
    required_parts: COLOR_AREA_REQUIRED_PARTS,
    api: COLOR_AREA_API,
    parts: COLOR_AREA_PARTS,
    states: COLOR_AREA_STATES,
    styling: COLOR_AREA_STYLING,
};

const COLOR_SLIDER_REQUIRED_PARTS: &[&str] = &[
    "ColorSlider",
    "Label",
    "ColorSlider.Output",
    "ColorSlider.Track",
    "ColorSlider.Thumb",
];

const COLOR_SLIDER_API: &[ApiDoc] = &[
    ApiDoc { owner: "ColorSlider", prop: "channel", ty: "ColorChannel", default: "—", description: "Required channel supplied positionally and edited with its pinned range and step.", rust_owner: "ColorSlider", rust: "new(id, value, channel)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorSlider", prop: "colorSpace", ty: "ColorSpace", default: "value color space", description: "Selects HSB, HSL, or RGB interpretation; invalid channel/space pairs are accepted rather than auto-corrected with a warning.", rust_owner: "ColorSlider", rust: "color_space(ColorSpace)", status: ImplementationStatus::Partial },
    ApiDoc { owner: "ColorSlider", prop: "value", ty: "string | Color", default: "—", description: "Controlled color waits for owner acceptance after pointer and keyboard reports.", rust_owner: "ColorSlider", rust: "new(id, PickerColor, channel)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorSlider", prop: "defaultValue", ty: "string | Color", default: "—", description: "Seeds picker-owned color state once.", rust_owner: "ColorSlider", rust: "default_value(PickerColor)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorSlider", prop: "onChange", ty: "(value: Color) => void", default: "—", description: "Reports each changed pointer coordinate or keyboard step.", rust_owner: "ColorSlider", rust: "on_change(callback)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorSlider", prop: "onChangeEnd", ty: "(value: Color) => void", default: "—", description: "Reports once on release and once for each completed keyboard change.", rust_owner: "ColorSlider", rust: "on_change_end(callback)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorSlider", prop: "orientation", ty: "horizontal | vertical", default: "horizontal", description: "Switches the value axis and vertical layout direction.", rust_owner: "ColorSlider", rust: "orientation(Orientation)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorSlider", prop: "isDisabled", ty: "boolean", default: "false", description: "Removes focus, pointer, keyboard, and successful form submission while masking thumb interaction state.", rust_owner: "ColorSlider", rust: "is_disabled(bool)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorSlider", prop: "name", ty: "string", default: "—", description: "Registers the live channel number for FormData.", rust_owner: "ColorSlider", rust: "name(text) + form_field()", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorSlider", prop: "aria-label", ty: "string", default: "—", description: "GPUI exposes no accessibility-tree attribute surface.", rust_owner: "ColorSlider", rust: "—", status: ImplementationStatus::Unavailable },
    ApiDoc { owner: "ColorSlider", prop: "children", ty: "ReactNode | RenderFunction", default: "—", description: "The monolithic Rust root composes built-in label, output, track, and thumb parts; root replacement is unavailable.", rust_owner: "ColorSlider", rust: "show_label / output / thumb", status: ImplementationStatus::Partial },
    ApiDoc { owner: "ColorSlider", prop: "className / render", ty: "string / DOMRenderFunction", default: "—", description: "Browser class customization and DOM root substitution are unavailable; length provides geometry customization.", rust_owner: "ColorSlider", rust: "length(Pixels)", status: ImplementationStatus::Partial },
    ApiDoc { owner: "ColorSliderRenderProps", prop: "color / orientation / isDisabled", ty: "Color / Orientation / boolean", default: "—", description: "The values are computed by the control but a root children render function is not exposed.", rust_owner: "ColorSlider", rust: "internal state", status: ImplementationStatus::Unavailable },
    ApiDoc { owner: "ColorSlider.Output", prop: "children", ty: "ReactNode | RenderFunction", default: "formatted value", description: "Replacement output receives current color and formatted channel text.", rust_owner: "ColorSlider", rust: "output(render)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorSlider.Output", prop: "className", ty: "string", default: "—", description: "Browser classes are unavailable.", rust_owner: "ColorSlider", rust: "—", status: ImplementationStatus::Unavailable },
    ApiDoc { owner: "ColorSlider.Track", prop: "children", ty: "ReactNode | RenderFunction", default: "ColorSlider.Thumb", description: "The built-in track owns the thumb; independent track render content and hover state are unavailable.", rust_owner: "ColorSlider", rust: "built-in track", status: ImplementationStatus::Partial },
    ApiDoc { owner: "ColorSlider.Track", prop: "className / style", ty: "string / CSSProperties | RenderFunction", default: "—", description: "Browser classes and inline-style render functions are unavailable.", rust_owner: "ColorSlider", rust: "—", status: ImplementationStatus::Unavailable },
    ApiDoc { owner: "ColorSlider.Thumb", prop: "children", ty: "ReactNode | RenderFunction", default: "—", description: "Caller content receives color, dragging, hovered, focused, focus-visible, and disabled state.", rust_owner: "ColorSlider", rust: "thumb(render)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorSlider.Thumb", prop: "className / style", ty: "string / CSSProperties | RenderFunction", default: "—", description: "Browser class and inline-style render functions are unavailable.", rust_owner: "ColorSlider", rust: "—", status: ImplementationStatus::Unavailable },
];

const COLOR_SLIDER_PARTS: &[PartDoc] = &[
    PartDoc {
        name: "ColorSlider",
        slot: "color-slider",
        description: "Value, orientation, form, label/output layout, and interaction owner.",
        rust_owner: "ColorSlider",
        status: ImplementationStatus::Implemented,
    },
    PartDoc {
        name: "Label",
        slot: "label",
        description: "Built-in channel name; arbitrary label composition is unavailable.",
        rust_owner: "ColorSlider",
        status: ImplementationStatus::Partial,
    },
    PartDoc {
        name: "ColorSlider.Output",
        slot: "color-slider-output",
        description: "Formatted value or caller-rendered color/value content.",
        rust_owner: "ColorSlider",
        status: ImplementationStatus::Implemented,
    },
    PartDoc {
        name: "ColorSlider.Track",
        slot: "color-slider-track",
        description:
            "Gradient and pointer geometry owner; independent child render state is unavailable.",
        rust_owner: "ColorSlider",
        status: ImplementationStatus::Partial,
    },
    PartDoc {
        name: "ColorSlider.Thumb",
        slot: "color-slider-thumb",
        description:
            "Current-color indicator, focus ring, and complete shared ColorThumb render state.",
        rust_owner: "ColorSlider",
        status: ImplementationStatus::Implemented,
    },
];

const COLOR_SLIDER_STATES: &[StateDoc] = &[
    StateDoc { state: "Disabled", selector: ".color-slider[data-disabled=true] / thumb", description: "Track dims, thumb uses default fill, interaction is removed, FormData is omitted, and thumb state is masked.", rust: "is_disabled", status: ImplementationStatus::Implemented },
    StateDoc { state: "Dragging", selector: ".color-slider__thumb[data-dragging=true]", description: "Same-value and changed-value presses expose live dragging state until global release.", rust: "dragging keyed state + ColorSliderThumbState", status: ImplementationStatus::Implemented },
    StateDoc { state: "Hovered", selector: ".color-slider__thumb[data-hovered=true]", description: "Enabled thumb hover reaches its render function.", rust: "thumb_hovered keyed state", status: ImplementationStatus::Implemented },
    StateDoc { state: "Focused", selector: ".color-slider__thumb[data-focused=true]", description: "Pointer and keyboard focus reach thumb render state.", rust: "focus_handle + ColorSliderThumbState", status: ImplementationStatus::Implemented },
    StateDoc { state: "Focus visible", selector: ".color-slider__thumb[data-focus-visible=true]", description: "Keyboard-origin focus rings the thumb rather than the whole track.", rust: "with_focus_ring(thumb) + focus_visible", status: ImplementationStatus::Implemented },
];

const COLOR_SLIDER_STYLING: &[StyleDoc] = &[
    StyleDoc { class_or_token: ".color-slider", value: "grid w-full gap-1", description: "The four-pixel label/output-to-track gap matches; the port uses flex rows instead of CSS grid areas.", rust: "flex_col + gap(4)", status: ImplementationStatus::Partial },
    StyleDoc { class_or_token: ".color-slider label / output", value: "text-sm font-medium tabular-nums", description: "Placement, 14px size, explicit 20px line height, medium weight, and disabled label/output opacity split match; GPUI has no tabular-number switch.", rust: "14px/20px MEDIUM label/output row", status: ImplementationStatus::Partial },
    StyleDoc { class_or_token: ".color-slider__track horizontal / vertical", value: "20px cross-axis; total length minus 20px with 10px edge caps", description: "Cross-axis thickness, rounded total footprint, endpoint inset, and pointer travel match; the Rust gradient continues through the rounded caps rather than drawing separate solid endpoint fills.", rust: "track_h=20 + COLOR_SLIDER_TRACK_INSET_PX", status: ImplementationStatus::Partial },
    StyleDoc { class_or_token: ".color-slider__track shadows", value: "orientation-specific 1px inset borders", description: "A theme border substitutes for the pinned inset edge shadows.", rust: "theme border", status: ImplementationStatus::Partial },
    StyleDoc { class_or_token: ".color-slider alpha checkerboard", value: "repeating-conic 16px checkerboard under gradient", description: "The alpha gradient is present but its checkerboard is not drawn.", rust: "alpha gradient only", status: ImplementationStatus::Unavailable },
    StyleDoc { class_or_token: ".color-slider__thumb", value: "size-4 rounded-2xl border-3 border-white shadow-overlay", description: "Size, circular radius, border, current-color fill, and disabled default fill match; overlay shadow is absent.", rust: "16px + rounded(16) + border(3)", status: ImplementationStatus::Partial },
    StyleDoc { class_or_token: ".color-slider__thumb transitions", value: "transform 250ms ease-out; box-shadow 150ms ease-out; reduced-motion none", description: "Thumb position and focus shadow still change on a frame.", rust: "immediate geometry/ring", status: ImplementationStatus::Unavailable },
    StyleDoc { class_or_token: ".color-slider__thumb[data-dragging=true]", value: "cursor-grabbing", description: "Dragging state is exposed, but GPUI provides one pointer cursor for the track rather than a separate grabbing cursor.", rust: "ColorSliderThumbState", status: ImplementationStatus::Partial },
];

pub(super) const COLOR_SLIDER: ReferenceMetadata = ReferenceMetadata {
    page: "ColorSlider",
    import_line: "use herogpui::components::color_picker::{ColorChannel, ColorSlider};",
    source_module: "color_picker",
    version: "3.2.5",
    docs_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/apps/docs/content/docs/en/react/components/(colors)/color-slider.mdx",
    api_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/packages/react/src/components/color-slider/color-slider.tsx + https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/ColorSlider.tsx + https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/ColorThumb.tsx + https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/Slider.tsx",
    style_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/packages/styles/components/color-slider.css",
    required_parts: COLOR_SLIDER_REQUIRED_PARTS,
    api: COLOR_SLIDER_API,
    parts: COLOR_SLIDER_PARTS,
    states: COLOR_SLIDER_STATES,
    styling: COLOR_SLIDER_STYLING,
};

const COLOR_PICKER_REQUIRED_PARTS: &[&str] =
    &["ColorPicker", "ColorPicker.Trigger", "ColorPicker.Popover"];

const COLOR_PICKER_API: &[ApiDoc] = &[
    ApiDoc {
        owner: "ColorPicker",
        prop: "value",
        ty: "string | Color",
        default: "—",
        description: "Controlled color value; owner acceptance drives the next frame.",
        rust_owner: "ColorPicker",
        rust: "new(id, value)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "ColorPicker",
        prop: "defaultValue",
        ty: "string | Color",
        default: "—",
        description: "Seeds the picker-owned color once.",
        rust_owner: "ColorPicker",
        rust: "default_value(PickerColor)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "ColorPicker",
        prop: "onChange",
        ty: "(color: Color) => void",
        default: "—",
        description: "Reports changes from the area and each rendered slider.",
        rust_owner: "ColorPicker",
        rust: "on_change(callback)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "ColorPicker",
        prop: "children",
        ty: "ReactNode",
        default: "—",
        description: "The Rust root renders a fixed trigger, area, hue slider, optional alpha slider, and readout rather than arbitrary compound children.",
        rust_owner: "ColorPicker",
        rust: "label / show_alpha",
        status: ImplementationStatus::Partial,
    },
    ApiDoc {
        owner: "ColorPicker",
        prop: "className",
        ty: "string",
        default: "—",
        description: "Browser CSS classes are unavailable.",
        rust_owner: "ColorPicker",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
    ApiDoc {
        owner: "ColorPicker.Trigger",
        prop: "children",
        ty: "ReactNode | RenderFunction",
        default: "—",
        description: "The trigger is the fixed swatch and hexadecimal value and does not delegate Button render state.",
        rust_owner: "ColorPicker",
        rust: "built-in trigger",
        status: ImplementationStatus::Partial,
    },
    ApiDoc {
        owner: "ColorPicker.Trigger",
        prop: "isDisabled",
        ty: "boolean",
        default: "false",
        description: "Inherited Button state disables the trigger and suppresses the panel.",
        rust_owner: "ColorPicker",
        rust: "is_disabled(bool)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "ColorPicker.Trigger",
        prop: "className",
        ty: "string",
        default: "—",
        description: "Browser CSS classes are unavailable.",
        rust_owner: "ColorPicker",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
    ApiDoc {
        owner: "ColorPicker.Popover",
        prop: "placement",
        ty: "Placement",
        default: "bottom left",
        description: "Positions and flips the floating panel to the side with more room when the preferred side cannot fit through the shared placement engine, keeping a 12px cross-axis viewport inset with the scroller capped to the available height.",
        rust_owner: "ColorPicker",
        rust: "placement(Placement)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc {
        owner: "ColorPicker.Popover",
        prop: "children",
        ty: "ReactNode",
        default: "—",
        description: "The built-in area, sliders, and hexadecimal readout cannot be replaced independently.",
        rust_owner: "ColorPicker",
        rust: "built-in popover content",
        status: ImplementationStatus::Partial,
    },
    ApiDoc {
        owner: "ColorPicker.Popover",
        prop: "className",
        ty: "string",
        default: "—",
        description: "Browser CSS classes are unavailable.",
        rust_owner: "ColorPicker",
        rust: "—",
        status: ImplementationStatus::Unavailable,
    },
];

const COLOR_PICKER_PARTS: &[PartDoc] = &[
    PartDoc {
        name: "ColorPicker",
        slot: "color-picker",
        description: "Color-value and internal DialogTrigger state owner.",
        rust_owner: "ColorPicker",
        status: ImplementationStatus::Implemented,
    },
    PartDoc {
        name: "ColorPicker.Trigger",
        slot: "color-picker-trigger",
        description: "Focusable swatch and hexadecimal-value trigger with fixed content.",
        rust_owner: "ColorPicker",
        status: ImplementationStatus::Partial,
    },
    PartDoc {
        name: "ColorPicker.Popover",
        slot: "color-picker-popover",
        description: "Dismissible floating color controls with fixed built-in children.",
        rust_owner: "ColorPicker",
        status: ImplementationStatus::Partial,
    },
];

const COLOR_PICKER_STATES: &[StateDoc] = &[
    StateDoc { state: "Open", selector: "DialogTrigger open state", description: "The default trigger owns open state; Rust's composition-only is_open builder reports without mutating until its owner accepts the change.", rust: "internal keyed state / is_open", status: ImplementationStatus::Implemented },
    StateDoc { state: "Focused", selector: ".color-picker__trigger:focus", description: "The trigger participates in tab order and regains focus after dismissal.", rust: "tab_stop_handle", status: ImplementationStatus::Implemented },
    StateDoc { state: "Focus visible", selector: ".color-picker__trigger[data-focus-visible=true]", description: "Keyboard focus draws the pinned status ring.", rust: "ring_if_focused", status: ImplementationStatus::Implemented },
    StateDoc { state: "Hovered", selector: "ColorPicker.Trigger render props isHovered", description: "The fixed Rust trigger does not expose Button hover render state.", rust: "—", status: ImplementationStatus::Unavailable },
    StateDoc { state: "Pressed", selector: "ColorPicker.Trigger render props isPressed", description: "Press tracking protects outside dismissal but is not delegated to trigger content.", rust: "trigger_pressed dismissal guard", status: ImplementationStatus::Partial },
    StateDoc { state: "Disabled", selector: ".color-picker__trigger[data-disabled=true]", description: "The trigger dims, leaves pointer interaction, and suppresses its panel.", rust: "is_disabled", status: ImplementationStatus::Implemented },
    StateDoc { state: "Entering", selector: ".color-picker__popover[data-entering=true]", description: "Panel fades and grows from 95% over 150ms with ease-smooth.", rust: "Motion::LIST_IN", status: ImplementationStatus::Implemented },
    StateDoc { state: "Exiting", selector: ".color-picker__popover[data-exiting=true]", description: "Panel is retained while fading and shrinking to 95% over 100ms.", rust: "Motion::LIST_OUT", status: ImplementationStatus::Implemented },
];

const COLOR_PICKER_STYLING: &[StyleDoc] = &[
    StyleDoc { class_or_token: ".color-picker", value: "inline-flex", description: "The Rust root also establishes a relative column and optional eight-pixel label gap around its monolithic composition.", rust: "relative + flex_col + gap(8)", status: ImplementationStatus::Partial },
    StyleDoc { class_or_token: ".color-picker__trigger", value: "inline-flex items-center gap-3 rounded-sm text-sm", description: "Trigger alignment, 12px gap, 4px radius, and 14px/20px type match regardless of parent line height.", rust: "flex row + gap(12) + hairline_radius + 14px", status: ImplementationStatus::Implemented },
    StyleDoc { class_or_token: ".color-picker__trigger cursor", value: "var(--cursor-interactive)", description: "Enabled triggers use the interactive pointer cursor.", rust: "cursor_pointer", status: ImplementationStatus::Implemented },
    StyleDoc { class_or_token: ".color-picker__trigger transitions", value: "background 150ms ease-smooth; shadow 150ms ease-out", description: "Trigger state colors and rings still change on a frame rather than interpolating.", rust: "immediate trigger chrome", status: ImplementationStatus::Unavailable },
    StyleDoc { class_or_token: ".color-picker__trigger focus-visible", value: "status-focused", description: "Keyboard focus uses the theme status ring.", rust: "ring_if_focused", status: ImplementationStatus::Implemented },
    StyleDoc { class_or_token: ".color-picker__trigger disabled", value: "status-disabled", description: "Disabled opacity and pointer suppression match.", rust: "disabled_opacity + no listeners", status: ImplementationStatus::Implemented },
    StyleDoc { class_or_token: ".color-picker__popover", value: "min-w-62 px-2 pt-2 pb-3 gap-3 bg-overlay", description: "The 248px minimum, 8px horizontal/top inset, 12px bottom inset and gap, and overlay fill match.", rust: "min_w(248) + px/pt(8) + pb/gap(12)", status: ImplementationStatus::Implemented },
    StyleDoc { class_or_token: ".color-picker__popover overflow", value: "overflow-x-hidden overflow-y-auto overscroll-contain scrollbar-none", description: "The panel hides horizontal overflow and scrolls vertically within the available viewport height without shrinking its controls; Tab reveals each slider and boundary scrolling never moves the page, and scrollbars are not drawn.", rust: "scrollable_popover + max_h_full + overflow_y_scroll", status: ImplementationStatus::Implemented },
    StyleDoc { class_or_token: ".color-picker__popover radius / shadow", value: "min(32px, radius * 2.5); shadow-overlay", description: "Theme-derived 2.5x radius and overlay shadow match, including the optional dark inset hairline.", rust: "layout.capped(radius_lg * 2.5) + overlay_shadow", status: ImplementationStatus::Implemented },
    StyleDoc { class_or_token: ".color-picker__popover entering", value: "150ms ease-smooth fade-in zoom-in-95 slide 4px by placement", description: "Duration, curve, fade, and scale match; the placement-specific four-pixel translation is not animated.", rust: "anim::entering_zoom(Motion::LIST_IN)", status: ImplementationStatus::Partial },
    StyleDoc { class_or_token: ".color-picker__popover exiting", value: "100ms ease-smooth fade-out zoom-out-95", description: "Exit lifetime, curve, opacity, and scale match and reduced motion snaps them.", rust: "anim::exiting(Motion::LIST_OUT)", status: ImplementationStatus::Implemented },
];

pub(super) const COLOR_PICKER: ReferenceMetadata = ReferenceMetadata {
    page: "ColorPicker",
    import_line: "use herogpui::components::color_picker::ColorPicker;",
    source_module: "color_picker",
    version: "3.2.5",
    docs_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/apps/docs/content/docs/en/react/components/(colors)/color-picker.mdx",
    api_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/packages/react/src/components/color-picker/color-picker.tsx + https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/ColorPicker.tsx + https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/Button.tsx + https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/Popover.tsx",
    style_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/packages/styles/components/color-picker.css",
    required_parts: COLOR_PICKER_REQUIRED_PARTS,
    api: COLOR_PICKER_API,
    parts: COLOR_PICKER_PARTS,
    states: COLOR_PICKER_STATES,
    styling: COLOR_PICKER_STYLING,
};

const COLOR_FIELD_REQUIRED_PARTS: &[&str] = &[
    "ColorField",
    "Label",
    "ColorField.Group",
    "ColorField.Prefix",
    "ColorField.Input",
    "ColorField.Suffix",
    "Description",
    "FieldError",
];

const COLOR_FIELD_API: &[ApiDoc] = &[
    ApiDoc { owner: "ColorField", prop: "children", ty: "ReactNode | RenderFunction", default: "—", description: "Replacement content receives disabled, invalid, read-only, required, focused, focus-within, and focus-visible state.", rust_owner: "ColorField", rust: "content(render)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorField", prop: "className", ty: "string | RenderFunction", default: "—", description: "Browser CSS classes are unavailable.", rust_owner: "ColorField", rust: "—", status: ImplementationStatus::Unavailable },
    ApiDoc { owner: "ColorField", prop: "style", ty: "CSSProperties | RenderFunction", default: "—", description: "Browser inline styles are unavailable.", rust_owner: "ColorField", rust: "—", status: ImplementationStatus::Unavailable },
    ApiDoc { owner: "ColorField", prop: "fullWidth", ty: "boolean", default: "false", description: "Stretches the root and input group in editable and display modes.", rust_owner: "ColorField", rust: "full_width(bool)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorField", prop: "id", ty: "string", default: "—", description: "Stable GPUI element identity is required by the constructor.", rust_owner: "ColorField", rust: "new(id, value)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorField", prop: "render", ty: "DOMRenderFunction", default: "—", description: "DOM root substitution is unavailable.", rust_owner: "ColorField", rust: "—", status: ImplementationStatus::Unavailable },
    ApiDoc { owner: "ColorField", prop: "value", ty: "Color | null", default: "—", description: "Controlled concrete colors wait for owner acceptance; the port cannot represent a controlled null value.", rust_owner: "ColorField", rust: "new(id, PickerColor)", status: ImplementationStatus::Partial },
    ApiDoc { owner: "ColorField", prop: "defaultValue", ty: "Color | null", default: "—", description: "Seeds picker-owned concrete color state once; a null seed is unavailable.", rust_owner: "ColorField", rust: "default_value(PickerColor)", status: ImplementationStatus::Partial },
    ApiDoc { owner: "ColorField", prop: "onChange", ty: "(Color | null) => void", default: "—", description: "Reports a parsed color or None while text is incomplete or invalid.", rust_owner: "ColorField", rust: "on_change(callback)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorField", prop: "colorSpace", ty: "ColorSpace", default: "—", description: "Selects the channel interpretation when channel is present.", rust_owner: "ColorField", rust: "color_space(ColorSpace)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorField", prop: "channel", ty: "ColorChannel", default: "—", description: "Switches from hexadecimal editing to one numeric channel.", rust_owner: "ColorField", rust: "channel(ColorChannel)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorField", prop: "isRequired", ty: "boolean", default: "false", description: "Marks the label and form field required, but the concrete-only value model cannot express v3's empty null value.", rust_owner: "ColorField", rust: "is_required(bool)", status: ImplementationStatus::Partial },
    ApiDoc { owner: "ColorField", prop: "isInvalid", ty: "boolean", default: "—", description: "Forces invalid chrome and render state.", rust_owner: "ColorField", rust: "is_invalid(bool)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorField", prop: "validate", ty: "(Color) => ValidationError | true | null", default: "—", description: "Custom validation contributes to field chrome, form validity, error text, and render state.", rust_owner: "ColorField", rust: "validate(callback)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorField", prop: "validationBehavior", ty: "native | aria", default: "native", description: "Native blocks submission; Allow exposes invalid state without blocking.", rust_owner: "ColorField", rust: "validation_behavior(ValidationBehavior)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorField", prop: "isDisabled", ty: "boolean", default: "false", description: "Removes editing, focus, wheel changes, and successful form submission.", rust_owner: "ColorField", rust: "is_disabled(bool)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorField", prop: "isReadOnly", ty: "boolean", default: "false", description: "Keeps the editable input focusable while preventing changes.", rust_owner: "ColorField", rust: "is_read_only(bool)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorField", prop: "isWheelDisabled", ty: "boolean", default: "false", description: "Suppresses focused wheel stepping for channel fields.", rust_owner: "ColorField", rust: "is_wheel_disabled(bool)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorField", prop: "name", ty: "string", default: "—", description: "Registers the current hex text or channel number for FormData.", rust_owner: "ColorField", rust: "name(text) + form_field()", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorField", prop: "autoFocus", ty: "boolean", default: "false", description: "Focuses the editable input on its first render.", rust_owner: "ColorField", rust: "auto_focus(bool)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorField", prop: "aria-label", ty: "string", default: "—", description: "GPUI exposes no accessibility tree attribute surface.", rust_owner: "ColorField", rust: "—", status: ImplementationStatus::Unavailable },
    ApiDoc { owner: "ColorField", prop: "aria-labelledby", ty: "string", default: "—", description: "GPUI exposes no accessibility tree attribute surface.", rust_owner: "ColorField", rust: "—", status: ImplementationStatus::Unavailable },
    ApiDoc { owner: "ColorField", prop: "aria-describedby", ty: "string", default: "—", description: "GPUI exposes no accessibility tree attribute surface.", rust_owner: "ColorField", rust: "—", status: ImplementationStatus::Unavailable },
    ApiDoc { owner: "ColorField", prop: "aria-details", ty: "string", default: "—", description: "GPUI exposes no accessibility tree attribute surface.", rust_owner: "ColorField", rust: "—", status: ImplementationStatus::Unavailable },
    ApiDoc { owner: "ColorField.Group", prop: "className", ty: "string", default: "—", description: "Browser CSS classes are unavailable.", rust_owner: "ColorField", rust: "—", status: ImplementationStatus::Unavailable },
    ApiDoc { owner: "ColorField.Group", prop: "fullWidth", ty: "boolean", default: "false", description: "The monolithic group follows the root width builder.", rust_owner: "ColorField", rust: "full_width(bool)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorField.Group", prop: "variant", ty: "primary | secondary", default: "primary", description: "Selects field or default fill and primary shadow.", rust_owner: "ColorField", rust: "variant(FieldVariant)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorField.Group", prop: "render", ty: "DOMRenderFunction", default: "—", description: "The group cannot be replaced independently.", rust_owner: "ColorField", rust: "—", status: ImplementationStatus::Unavailable },
    ApiDoc { owner: "ColorField.Input", prop: "className", ty: "string", default: "—", description: "Browser CSS classes are unavailable.", rust_owner: "ColorField", rust: "—", status: ImplementationStatus::Unavailable },
    ApiDoc { owner: "ColorField.Input", prop: "placeholder", ty: "string", default: "—", description: "Shows caller text, or the resolved current color when omitted.", rust_owner: "ColorField", rust: "placeholder(text)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorField.Prefix", prop: "className", ty: "string", default: "—", description: "Browser CSS classes are unavailable.", rust_owner: "ColorField", rust: "—", status: ImplementationStatus::Unavailable },
    ApiDoc { owner: "ColorField.Prefix", prop: "children", ty: "ReactNode", default: "—", description: "The port always renders the current ColorSwatch and cannot replace the prefix independently.", rust_owner: "ColorField", rust: "built-in ColorSwatch", status: ImplementationStatus::Partial },
    ApiDoc { owner: "ColorField.Suffix", prop: "className", ty: "string", default: "—", description: "Browser CSS classes are unavailable.", rust_owner: "ColorField", rust: "—", status: ImplementationStatus::Unavailable },
    ApiDoc { owner: "ColorField.Suffix", prop: "children", ty: "ReactNode", default: "—", description: "Caller content remains present in editable and display compositions.", rust_owner: "ColorField", rust: "suffix(element)", status: ImplementationStatus::Implemented },
];

const COLOR_FIELD_PARTS: &[PartDoc] = &[
    PartDoc { name: "ColorField", slot: "color-field", description: "Value, validation, form, and complete render-state owner.", rust_owner: "ColorField", status: ImplementationStatus::Implemented },
    PartDoc { name: "Label", slot: "label", description: "Optional string label with required, disabled, and invalid state; composes field::Label.", rust_owner: "ColorField", status: ImplementationStatus::Partial },
    PartDoc { name: "ColorField.Group", slot: "color-input-group", description: "Field chrome drawn by ColorField surrounding prefix, input, and suffix; the input is a composed Input.", rust_owner: "ColorField", status: ImplementationStatus::Implemented },
    PartDoc { name: "ColorField.Prefix", slot: "color-input-group-prefix", description: "Fixed current-color swatch; arbitrary replacement is unavailable.", rust_owner: "ColorSwatch", status: ImplementationStatus::Partial },
    PartDoc { name: "ColorField.Input", slot: "color-input-group-input", description: "State-backed text input with parsing, channel keys, wheel editing, and focus; backed by a composed input::InputState.", rust_owner: "ColorField", status: ImplementationStatus::Implemented },
    PartDoc { name: "ColorField.Suffix", slot: "color-input-group-suffix", description: "Caller-provided trailing content in placeholder color.", rust_owner: "ColorField", status: ImplementationStatus::Implemented },
    PartDoc { name: "Description", slot: "description", description: "Optional helper text replaced by resolved error content when invalid; composes field::Description.", rust_owner: "ColorField", status: ImplementationStatus::Partial },
    PartDoc { name: "FieldError", slot: "field-error", description: "Resolved controlled, server, or validator message handed to the composed Input's error_message; arbitrary child composition is unavailable.", rust_owner: "ColorField", status: ImplementationStatus::Partial },
];

const COLOR_FIELD_STATES: &[StateDoc] = &[
    StateDoc { state: "Invalid", selector: ".color-field[data-invalid=true] / .color-input-group[data-invalid=true]", description: "Hides description, shows the resolved message, draws danger chrome, blocks native forms, and reaches render state.", rust: "validation::resolve + Input", status: ImplementationStatus::Implemented },
    StateDoc { state: "Required", selector: ".color-field[data-required=true]", description: "Marks the label and reaches render state; concrete-only color state cannot represent v3's empty null value.", rust: "is_required", status: ImplementationStatus::Partial },
    StateDoc { state: "Disabled", selector: ".color-input-group[data-disabled=true]", description: "Dims the group, removes input and wheel behavior, omits FormData, and reaches render state.", rust: "is_disabled", status: ImplementationStatus::Implemented },
    StateDoc { state: "Read only", selector: "ColorField render props isReadOnly", description: "Keeps the editable input focusable but blocks mutations and reaches render state.", rust: "is_read_only", status: ImplementationStatus::Implemented },
    StateDoc { state: "Focused", selector: "ColorField render props isFocused", description: "Reports focus on the state-backed input.", rust: "ColorFieldRenderState", status: ImplementationStatus::Implemented },
    StateDoc { state: "Focus within", selector: ".color-input-group[data-focus-within=true]", description: "Focus anywhere in the composed field drives focused chrome and render state.", rust: "focus_handle.contains_focused", status: ImplementationStatus::Implemented },
    StateDoc { state: "Focus visible", selector: "ColorField render props isFocusVisible", description: "Keyboard-origin focus reaches the render function and the shared field ring.", rust: "focus_visible", status: ImplementationStatus::Implemented },
    StateDoc { state: "Hovered", selector: ".color-input-group[data-hovered=true]", description: "The Rust group does not yet apply the pinned hover fill and border interpolation.", rust: "—", status: ImplementationStatus::Unavailable },
];

const COLOR_FIELD_STYLING: &[StyleDoc] = &[
    StyleDoc { class_or_token: ".color-field", value: "flex flex-col gap-1", description: "Four-pixel field-part spacing matches.", rust: "flex_col + gap(4)", status: ImplementationStatus::Implemented },
    StyleDoc { class_or_token: ".color-field invalid description", value: "hidden", description: "Resolved error content replaces the description row rather than drawing both.", rust: "Input validity branch", status: ImplementationStatus::Implemented },
    StyleDoc { class_or_token: ".color-field--full-width", value: "w-full", description: "Both the wrapper and inner group stretch, including in a non-stretching flex parent.", rust: "full_width(bool) + root/group w_full", status: ImplementationStatus::Implemented },
    StyleDoc { class_or_token: ".color-input-group", value: "inline-flex h-9 items-center overflow-hidden rounded-field border-0 bg-field text-sm shadow-field", description: "Height, alignment, field radius/fill, zero border, 14px type, clipping, and primary shadow match.", rust: "Input + FIELD_HEIGHT/FIELD_TEXT + apply_field_chrome", status: ImplementationStatus::Implemented },
    StyleDoc { class_or_token: ".color-input-group transitions", value: "background/border 150ms ease-smooth; shadow 150ms ease-out", description: "Field state colors and rings still change on a frame.", rust: "immediate apply_field_chrome", status: ImplementationStatus::Unavailable },
    StyleDoc { class_or_token: ".color-input-group hover", value: "bg-field-hover border-field-hover", description: "The pinned hover-only group state is not drawn.", rust: "—", status: ImplementationStatus::Unavailable },
    StyleDoc { class_or_token: ".color-input-group focus-within", value: "status-focused-field", description: "Focused input draws the shared two-pixel focus ring.", rust: "apply_field_chrome(focused)", status: ImplementationStatus::Implemented },
    StyleDoc { class_or_token: ".color-input-group invalid", value: "status-invalid-field bg-field-focus", description: "Danger outline/ring matches; the port keeps the variant fill rather than switching to the field-focus fill.", rust: "apply_field_chrome(invalid)", status: ImplementationStatus::Partial },
    StyleDoc { class_or_token: ".color-input-group disabled", value: "status-disabled", description: "Disabled opacity and listener suppression match.", rust: "Input::is_disabled", status: ImplementationStatus::Implemented },
    StyleDoc { class_or_token: ".color-input-group__input", value: "h-full flex-1 cursor-text px-3 py-2 text-sm bg-transparent", description: "State-backed Input supplies the cursor, horizontal inset, transparent fill, and unified 36px/14px field metrics.", rust: "Input", status: ImplementationStatus::Implemented },
    StyleDoc { class_or_token: ".color-input-group__prefix", value: "shrink-0 text-field-placeholder ms-3 me-0", description: "The fixed 16px swatch starts at the 12px field inset; text follows after the pinned eight-pixel prefix gap.", rust: "Input start_content(ColorSwatch)", status: ImplementationStatus::Implemented },
    StyleDoc { class_or_token: ".color-input-group__suffix", value: "shrink-0 text-field-placeholder me-3", description: "Editable and display suffix content stays at the trailing 12px inset and uses a 20px line height.", rust: "suffix + Input::end_content", status: ImplementationStatus::Implemented },
    StyleDoc { class_or_token: ".color-input-group--secondary", value: "shadow-none bg-default; hover default-hover", description: "Resting fill and shadow match; the absent group hover leaves the hover token undrawn.", rust: "FieldVariant::Secondary", status: ImplementationStatus::Partial },
];

pub(super) const COLOR_FIELD: ReferenceMetadata = ReferenceMetadata {
    page: "ColorField",
    import_line: "use herogpui::components::color_picker::ColorField;",
    source_module: "color_picker",
    version: "3.2.5",
    docs_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/apps/docs/content/docs/en/react/components/(colors)/color-field.mdx",
    api_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/packages/react/src/components/color-field/color-field.tsx + https://github.com/heroui-inc/heroui/blob/v3.2.5/packages/react/src/components/color-input-group/color-input-group.tsx + https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/ColorField.tsx",
    style_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/packages/styles/components/color-field.css + https://github.com/heroui-inc/heroui/blob/v3.2.5/packages/styles/components/color-input-group.css",
    required_parts: COLOR_FIELD_REQUIRED_PARTS,
    api: COLOR_FIELD_API,
    parts: COLOR_FIELD_PARTS,
    states: COLOR_FIELD_STATES,
    styling: COLOR_FIELD_STYLING,
};

const COLOR_SWATCH_PICKER_REQUIRED_PARTS: &[&str] = &[
    "ColorSwatchPicker",
    "ColorSwatchPicker.Item",
    "ColorSwatchPicker.Swatch",
    "ColorSwatchPicker.Indicator",
];

const COLOR_SWATCH_PICKER_API: &[ApiDoc] = &[
    ApiDoc { owner: "ColorSwatchPicker", prop: "value", ty: "string | Color", default: "—", description: "Controlled selection changes only when the owner supplies the reported color.", rust_owner: "ColorSwatchPicker", rust: "value(PickerColor)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorSwatchPicker", prop: "defaultValue", ty: "string | Color", default: "—", description: "Seeds picker-owned selection once.", rust_owner: "ColorSwatchPicker", rust: "default_value(PickerColor)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorSwatchPicker", prop: "onChange", ty: "(value: Color) => void", default: "—", description: "Reports pointer and keyboard selection changes.", rust_owner: "ColorSwatchPicker", rust: "on_change(callback)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorSwatchPicker", prop: "size", ty: "xs | sm | md | lg | xl", default: "md", description: "Drives item edge, border, radius, indicator, hit geometry, wrapping, and grid navigation stride.", rust_owner: "ColorSwatchPicker", rust: "size(SizeXl)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorSwatchPicker", prop: "variant", ty: "circle | square", default: "circle", description: "Selects the pinned per-size item and swatch radii.", rust_owner: "ColorSwatchPicker", rust: "shape(SwatchShape)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorSwatchPicker", prop: "layout", ty: "grid | stack", default: "grid", description: "Grid wraps and uses geometric row navigation; stack is vertical and uses Up/Down.", rust_owner: "ColorSwatchPicker", rust: "layout(SwatchLayout)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorSwatchPicker", prop: "className", ty: "string", default: "—", description: "Browser classes are unavailable.", rust_owner: "ColorSwatchPicker", rust: "—", status: ImplementationStatus::Unavailable },
    ApiDoc { owner: "ColorSwatchPicker", prop: "children", ty: "ReactNode", default: "—", description: "The Rust constructor receives a concrete palette and composes one item per color.", rust_owner: "ColorSwatchPicker", rust: "new(id, Vec<PickerColor>)", status: ImplementationStatus::Partial },
    ApiDoc { owner: "ColorSwatchPicker", prop: "render", ty: "DOMRenderFunction", default: "—", description: "The GPUI root element cannot be replaced.", rust_owner: "ColorSwatchPicker", rust: "—", status: ImplementationStatus::Unavailable },
    ApiDoc { owner: "ColorSwatchPicker.Item", prop: "color", ty: "string | Color", default: "required", description: "Each palette color is supplied by index and reaches item render state.", rust_owner: "ColorSwatchPicker", rust: "new(id, Vec<PickerColor>) / item_content", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorSwatchPicker.Item", prop: "isDisabled", ty: "boolean", default: "false", description: "Per-index disabled items dim, leave navigation, and mask interaction state.", rust_owner: "ColorSwatchPicker", rust: "disabled_keys(indices)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorSwatchPicker.Item", prop: "className", ty: "string", default: "—", description: "Browser classes are unavailable.", rust_owner: "ColorSwatchPicker", rust: "—", status: ImplementationStatus::Unavailable },
    ApiDoc { owner: "ColorSwatchPicker.Item", prop: "children", ty: "ReactNode | RenderFunction", default: "Swatch + Indicator", description: "Replacement content receives color, hover, press, selection, focus, focus-visible, and disabled state.", rust_owner: "ColorSwatchPicker", rust: "item_content(render)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorSwatchPicker.Item", prop: "render", ty: "DOMRenderFunction", default: "—", description: "The behavior-owning item cannot be replaced; caller content is composed inside it.", rust_owner: "ColorSwatchPicker", rust: "item_content(render)", status: ImplementationStatus::Partial },
    ApiDoc { owner: "ColorSwatchPicker.Swatch", prop: "className", ty: "string", default: "—", description: "Browser classes are unavailable; replacing Item children replaces the swatch visual.", rust_owner: "ColorSwatchPicker", rust: "item_content(render)", status: ImplementationStatus::Partial },
    ApiDoc { owner: "ColorSwatchPicker.Indicator", prop: "children", ty: "ReactNode | RenderFunction", default: "checkmark", description: "Replacement indicator content receives the complete item state.", rust_owner: "ColorSwatchPicker", rust: "indicator(render)", status: ImplementationStatus::Implemented },
];

const COLOR_SWATCH_PICKER_PARTS: &[PartDoc] = &[
    PartDoc {
        name: "ColorSwatchPicker",
        slot: "color-swatch-picker",
        description: "Selection owner and wrapping grid or vertical stack.",
        rust_owner: "ColorSwatchPicker",
        status: ImplementationStatus::Implemented,
    },
    PartDoc {
        name: "ColorSwatchPicker.Item",
        slot: "color-swatch-picker-item",
        description: "Stable focus, pointer, selection, and render-state owner.",
        rust_owner: "ColorSwatchPicker",
        status: ImplementationStatus::Implemented,
    },
    PartDoc {
        name: "ColorSwatchPicker.Swatch",
        slot: "color-swatch-picker-swatch",
        description: "Checkerboard-backed color visual with selected and hover geometry.",
        rust_owner: "ColorSwatchPicker",
        status: ImplementationStatus::Implemented,
    },
    PartDoc {
        name: "ColorSwatchPicker.Indicator",
        slot: "color-swatch-picker-indicator",
        description:
            "Centered selected marker with luminance-aware default color or caller content.",
        rust_owner: "ColorSwatchPicker",
        status: ImplementationStatus::Implemented,
    },
];

const COLOR_SWATCH_PICKER_STATES: &[StateDoc] = &[
    StateDoc { state: "Hovered", selector: ".color-swatch-picker__item[data-hovered=true]", description: "Enabled unselected swatches grow to 1.1 and item content receives hover.", rust: "interaction + item_content", status: ImplementationStatus::Implemented },
    StateDoc { state: "Pressed", selector: "ColorSwatchPickerItemRenderProps.isPressed", description: "Enabled item content receives live pointer and keyboard press state.", rust: "interaction + item_content", status: ImplementationStatus::Implemented },
    StateDoc { state: "Selected", selector: ".color-swatch-picker__item[data-selected=true]", description: "The color border is revealed by a 0.77 inner swatch and item/indicator state updates.", rust: "resolved value + item_content / indicator", status: ImplementationStatus::Implemented },
    StateDoc { state: "Focused", selector: "ColorSwatchPickerItemRenderProps.isFocused", description: "Exactly one enabled item owns the roving focus handle.", rust: "cursor + tab_stop_handle", status: ImplementationStatus::Implemented },
    StateDoc { state: "Focus visible", selector: ".color-swatch-picker__item[data-focus-visible=true]", description: "Keyboard-origin focus draws the shared ring and reaches item state.", rust: "with_focus_ring + item_content", status: ImplementationStatus::Implemented },
    StateDoc { state: "Disabled", selector: ".color-swatch-picker__item[data-disabled=true]", description: "Whole-picker and per-item disabling dim, suppress events, remove navigation stops, and mask state.", rust: "is_disabled / disabled_keys", status: ImplementationStatus::Implemented },
];

const COLOR_SWATCH_PICKER_STYLING: &[StyleDoc] = &[
    StyleDoc { class_or_token: ".color-swatch-picker", value: "flex flex-wrap items-center gap-2", description: "Grid alignment, eight-pixel gap, wrapping, and 280px desktop cap match.", rust: "flex + items_center + gap(8) + flex_wrap + max_w(280)", status: ImplementationStatus::Implemented },
    StyleDoc { class_or_token: ".color-swatch-picker--stack", value: "flex-col", description: "Stack layout is vertical and its keyboard direction follows the visual axis.", rust: "flex_col + Up/Down navigation", status: ImplementationStatus::Implemented },
    StyleDoc { class_or_token: ".color-swatch-picker__item sizes", value: "16/24/32/36/40px; borders 1/2/2/3/3px", description: "Every size drives paint, hit testing, indicator size, wrapping, and grid stride.", rust: "SizeXl::swatch_px + border_width", status: ImplementationStatus::Implemented },
    StyleDoc { class_or_token: ".color-swatch-picker circle / square radii", value: "per-size circle radii; square item 6/8/12/12/12px and swatch 6/8/8/8/8px", description: "Pinned item and inner-swatch radii match, including the selected small-square exception.", rust: "item_radius + inner radius", status: ImplementationStatus::Implemented },
    StyleDoc { class_or_token: ".color-swatch-picker__item transitions", value: "border/shadow 100ms ease-out; reduced-motion none", description: "Border color and focus shadow still change on a frame.", rust: "immediate border/ring", status: ImplementationStatus::Unavailable },
    StyleDoc { class_or_token: ".color-swatch-picker__swatch transforms", value: "hover scale 1.1; selected scale .77; 100ms ease-out", description: "Final geometry and selected-over-hover precedence match; interpolation is not implemented.", rust: "computed inner edge + hover size", status: ImplementationStatus::Partial },
    StyleDoc { class_or_token: ".color-swatch-picker__indicator", value: "absolute inset-0; child size one-third; light-aware black/white", description: "Position, size, and luminance contrast match; custom content uses the same centered owner.", rust: "indicator / CHECK", status: ImplementationStatus::Implemented },
    StyleDoc { class_or_token: ".color-swatch-picker__indicator transition", value: "scale 0 to 1 over 150ms ease-out; reduced-motion none", description: "Indicator visibility changes immediately rather than interpolating scale.", rust: "selected visibility", status: ImplementationStatus::Unavailable },
];

pub(super) const COLOR_SWATCH_PICKER: ReferenceMetadata = ReferenceMetadata {
    page: "ColorSwatchPicker",
    import_line: "use herogpui::components::color_picker::ColorSwatchPicker;",
    source_module: "color_picker",
    version: "3.2.5",
    docs_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/apps/docs/content/docs/en/react/components/(colors)/color-swatch-picker.mdx",
    api_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/packages/react/src/components/color-swatch-picker/color-swatch-picker.tsx + https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/ColorSwatchPicker.tsx + https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/ListBox.tsx",
    style_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/packages/styles/components/color-swatch-picker.css",
    required_parts: COLOR_SWATCH_PICKER_REQUIRED_PARTS,
    api: COLOR_SWATCH_PICKER_API,
    parts: COLOR_SWATCH_PICKER_PARTS,
    states: COLOR_SWATCH_PICKER_STATES,
    styling: COLOR_SWATCH_PICKER_STYLING,
};

const COLOR_SWATCH_REQUIRED_PARTS: &[&str] = &["ColorSwatch"];

const COLOR_SWATCH_API: &[ApiDoc] = &[
    ApiDoc { owner: "ColorSwatch", prop: "color", ty: "string | Color", default: "—", description: "Color value displayed by the swatch.", rust_owner: "ColorSwatch", rust: "new(PickerColor) / color(PickerColor)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorSwatch", prop: "id", ty: "string", default: "—", description: "Optional GPUI identity. A named swatch reports role=img named by the hex; an unnamed preview produces no AccessKit node.", rust_owner: "ColorSwatch", rust: "id(ElementId)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorSwatch", prop: "colorName", ty: "string", default: "—", description: "A named swatch uses the hex as its accessible name; a separate colorName override is not a builder.", rust_owner: "ColorSwatch", rust: "id(ElementId) names the hex", status: ImplementationStatus::Partial },
    ApiDoc { owner: "ColorSwatch", prop: "className", ty: "string", default: "—", description: "Browser CSS classes are unavailable.", rust_owner: "ColorSwatch", rust: "—", status: ImplementationStatus::Unavailable },
    ApiDoc { owner: "ColorSwatch", prop: "shape", ty: "'circle' | 'square'", default: "'circle'", description: "Selects a circular or rounded-square swatch.", rust_owner: "ColorSwatch", rust: "shape(SwatchShape)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorSwatch", prop: "size", ty: "'xs' | 'sm' | 'md' | 'lg' | 'xl'", default: "'md'", description: "Selects the 16, 24, 32, 36 or 40px swatch size.", rust_owner: "ColorSwatch", rust: "size(SizeXl)", status: ImplementationStatus::Implemented },
    ApiDoc { owner: "ColorSwatch", prop: "style", ty: "CSSProperties | render function", default: "—", description: "Browser inline styles and style render functions are unavailable.", rust_owner: "ColorSwatch", rust: "—", status: ImplementationStatus::Unavailable },
    ApiDoc { owner: "ColorSwatch", prop: "aria-label", ty: "string", default: "—", description: "A named swatch reports role=img named by the hex. There is no separate aria-label builder.", rust_owner: "ColorSwatch", rust: "id(ElementId)", status: ImplementationStatus::Partial },
    ApiDoc { owner: "ColorSwatch", prop: "render", ty: "DOMRenderFunction", default: "—", description: "DOM root substitution has no GPUI equivalent.", rust_owner: "ColorSwatch", rust: "—", status: ImplementationStatus::Unavailable },
];

const COLOR_SWATCH_PARTS: &[PartDoc] = &[PartDoc {
    name: "ColorSwatch",
    slot: "color-swatch",
    description: "Color preview with a transparency backdrop, border, size, and shape.",
    rust_owner: "ColorSwatch",
    status: ImplementationStatus::Implemented,
}];

const COLOR_SWATCH_STATES: &[StateDoc] = &[];

const COLOR_SWATCH_STYLING: &[StyleDoc] = &[
    StyleDoc {
        class_or_token: ".color-swatch",
        value: "size-8 overflow-hidden rounded-3xl border border-default",
        description: "The default swatch is a clipped 32px circle with the theme border.",
        rust: "SizeXl::Md + overflow_hidden + border(layout.border_width)",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: ".color-swatch--circle / --square",
        value: "size-specific round radius / rounded-md",
        description: "Circle radii follow each size; square uses the theme medium radius.",
        rust: "SwatchShape => edge / 2 or radius_md",
        status: ImplementationStatus::Implemented,
    },
    StyleDoc {
        class_or_token: "transparency checkerboard",
        value: "background-image checker pattern",
        description: "Translucent colors blend over the secondary surface; the port does not draw the upstream checker pattern.",
        rust: "colors.surface_secondary beneath PickerColor",
        status: ImplementationStatus::Partial,
    },
];

pub(super) const COLOR_SWATCH: ReferenceMetadata = ReferenceMetadata {
    page: "ColorSwatch",
    import_line: "use herogpui::components::color_picker::ColorSwatch;",
    source_module: "color_picker",
    version: "3.2.5",
    docs_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/apps/docs/content/docs/en/react/components/(colors)/color-swatch.mdx",
    api_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/packages/react/src/components/color-swatch/color-swatch.tsx",
    style_source: "https://github.com/heroui-inc/heroui/blob/v3.2.5/packages/styles/components/color-swatch.css",
    required_parts: COLOR_SWATCH_REQUIRED_PARTS,
    api: COLOR_SWATCH_API,
    parts: COLOR_SWATCH_PARTS,
    states: COLOR_SWATCH_STATES,
    styling: COLOR_SWATCH_STYLING,
};
