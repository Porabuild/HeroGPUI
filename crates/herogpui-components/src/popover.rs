//! Popover — port of `@heroui/popover`.

use gpui::{
    point, prelude::*, px, AnyElement, App, Bounds, ClickEvent, Display, Element, GlobalElementId,
    InspectorElementId, IntoElement, LayoutId, ParentElement, Pixels, Position, RenderOnce,
    SharedString, Size, StatefulInteractiveElement, Style, Styled, Window,
};
use herogpui_theme::ActiveTheme;

/// `placement` on `Popover.Content`.
///
/// Shares the one placement vocabulary with the pickers and dropdown.
pub use herogpui_core::Placement as PopoverPlacement;

type OnOpenChange = std::sync::Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>;

#[derive(Clone, Copy)]
enum PopoverSide {
    Top,
    Bottom,
    Left,
    Right,
}

impl PopoverSide {
    const ALL: [Self; 4] = [Self::Top, Self::Bottom, Self::Left, Self::Right];

    fn opposite(self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Top => 0,
            Self::Bottom => 1,
            Self::Left => 2,
            Self::Right => 3,
        }
    }

    fn arrow_rotation(self) -> f32 {
        match self {
            Self::Top => 0.,
            Self::Bottom => std::f32::consts::PI,
            Self::Left => -std::f32::consts::FRAC_PI_2,
            Self::Right => std::f32::consts::FRAC_PI_2,
        }
    }
}

struct PopoverPositioner {
    trigger: std::rc::Rc<std::cell::Cell<Option<Bounds<Pixels>>>>,
    resolved: std::rc::Rc<std::cell::Cell<Option<PopoverResolved>>>,
    placement: PopoverPlacement,
    offset: Pixels,
    should_flip: bool,
    has_arrow: bool,
    constrain_height: bool,
    /// Size the panel from the trigger width instead of `MaxContent`.
    ///
    /// Field panels (Select) are `w_full`: against a `MaxContent` root width
    /// that resolves incorrectly, so both measurements use the measured
    /// trigger width. With the widths equal, start/center/end alignment
    /// coincide and only the flipped side differs.
    match_trigger_width: bool,
    children: Vec<AnyElement>,
}

#[derive(Clone, Copy)]
struct PopoverResolved {
    trigger: Bounds<Pixels>,
    panel: Bounds<Pixels>,
    side: PopoverSide,
}

/// The v3 `Popover.Arrow` part.
///
/// Upstream v3 draws no arrow unless the part is composed into the panel's
/// children (`<Popover.Arrow />`). Without `.child(..)` the part renders v3's
/// built-in 12px curved arrow, rotated for the resolved side. With children,
/// upstream stamps the single composed element with
/// `data-slot="popover-overlay-arrow"`, and the `.popover` placement CSS
/// (`data-placement="bottom|left|right"`) rotates *that* element — so a custom
/// child is rotated too, not just the built-in curve.
///
/// This port is Partial there, deliberately: GPUI 0.2.2 transforms only `svg()`
/// elements (`with_transformation`), not arbitrary divs, and the resolved side
/// is known only at prepaint while an `Svg`'s transformation can only be set at
/// construction — so a caller-provided element (SVG included) takes the built-in
/// arrow's resolved position but no rotation. The port renders the first
/// composed child; additional children are ignored, where upstream composes
/// exactly one element and falls back to the default curve otherwise.
///
/// The part reads its placement from the popover that composes it, which is
/// discovered among the panel's direct children: a `PopoverArrow` nested inside
/// another element is never resolved and paints nothing, like one rendered
/// outside any popover.
pub struct PopoverArrow {
    resolved: std::rc::Rc<std::cell::Cell<Option<PopoverResolved>>>,
    children: Vec<AnyElement>,
}

impl Default for PopoverArrow {
    fn default() -> Self {
        Self::new()
    }
}

impl PopoverArrow {
    pub fn new() -> Self {
        Self {
            resolved: std::rc::Rc::new(std::cell::Cell::new(None)),
            children: Vec::new(),
        }
    }
}

impl ParentElement for PopoverArrow {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// Layout state of a composed `Popover.Arrow`. Public only because the
/// `Element` associated type leaks; nothing here is a public contract.
pub struct PopoverArrowState {
    leaves: Vec<AnyElement>,
    layouts: Vec<LayoutId>,
    custom: bool,
}

impl Element for PopoverArrow {
    type RequestLayoutState = PopoverArrowState;
    type PrepaintState = Option<usize>;

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let custom = !self.children.is_empty();
        let mut leaves = if custom {
            std::mem::take(&mut self.children)
        } else {
            let color = cx.colors().overlay.background;
            PopoverSide::ALL
                .map(|side| {
                    gpui::div()
                        .absolute()
                        .size(px(12.))
                        .debug_selector(|| "popover-arrow".to_owned())
                        .child(
                            gpui::svg()
                                .size(px(12.))
                                .path(crate::icons::TOOLTIP_ARROW)
                                .text_color(color)
                                .with_transformation(gpui::Transformation::rotate(gpui::radians(
                                    side.arrow_rotation(),
                                ))),
                        )
                        .into_any_element()
                })
                .into_iter()
                .collect()
        };
        let layouts = leaves
            .iter_mut()
            .map(|leaf| leaf.request_layout(window, cx))
            .collect::<Vec<_>>();
        let layout = window.request_layout(
            Style {
                position: Position::Absolute,
                display: Display::Flex,
                ..Style::default()
            },
            layouts.iter().copied(),
            cx,
        );
        (
            layout,
            PopoverArrowState {
                leaves,
                layouts,
                custom,
            },
        )
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        state: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let resolved = self.resolved.get()?;
        let index = if state.custom {
            0
        } else {
            resolved.side.index()
        };
        let bounds = window.layout_bounds(state.layouts[index]);
        let offset =
            PopoverPositioner::arrow_origin(resolved.side, resolved.trigger, resolved.panel)
                - bounds.origin;
        let offset = point(offset.x.round(), offset.y.round());
        window.with_element_offset(offset, |window| {
            state.leaves[index].prepaint(window, cx);
        });
        Some(index)
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        state: &mut Self::RequestLayoutState,
        selected: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        if let Some(index) = *selected {
            state.leaves[index].paint(window, cx);
        }
    }
}

impl IntoElement for PopoverArrow {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

pub(crate) struct PopoverTriggerMeasure {
    child: AnyElement,
    bounds: std::rc::Rc<std::cell::Cell<Option<Bounds<Pixels>>>>,
}

impl PopoverTriggerMeasure {
    pub(crate) fn new(
        child: impl IntoElement,
        bounds: std::rc::Rc<std::cell::Cell<Option<Bounds<Pixels>>>>,
    ) -> Self {
        Self {
            child: child.into_any_element(),
            bounds,
        }
    }
}

impl Element for PopoverTriggerMeasure {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        (self.child.request_layout(window, cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.bounds.set(Some(bounds));
        self.child.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        _: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.paint(window, cx);
    }
}

impl IntoElement for PopoverTriggerMeasure {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl PopoverPositioner {
    fn new(
        trigger: std::rc::Rc<std::cell::Cell<Option<Bounds<Pixels>>>>,
        resolved: std::rc::Rc<std::cell::Cell<Option<PopoverResolved>>>,
        placement: PopoverPlacement,
        offset: Pixels,
        should_flip: bool,
        has_arrow: bool,
    ) -> Self {
        Self {
            trigger,
            resolved,
            placement,
            offset,
            should_flip,
            has_arrow,
            constrain_height: false,
            match_trigger_width: false,
            children: Vec::new(),
        }
    }

    fn preferred_side(&self) -> PopoverSide {
        match self.placement {
            PopoverPlacement::Left => PopoverSide::Left,
            PopoverPlacement::Right => PopoverSide::Right,
            placement if placement.is_above() => PopoverSide::Top,
            _ => PopoverSide::Bottom,
        }
    }

    fn available(
        &self,
        side: PopoverSide,
        trigger: Bounds<Pixels>,
        viewport: Size<Pixels>,
    ) -> Pixels {
        let available = match side {
            PopoverSide::Top => trigger.top(),
            PopoverSide::Bottom => viewport.height - trigger.bottom(),
            PopoverSide::Left => trigger.left(),
            PopoverSide::Right => viewport.width - trigger.right(),
        };
        available
            - if self.constrain_height {
                px(12.)
            } else {
                px(0.)
            }
    }

    fn resolved_side(
        &self,
        trigger: Bounds<Pixels>,
        popup: Size<Pixels>,
        viewport: Size<Pixels>,
    ) -> PopoverSide {
        let preferred = self.preferred_side();
        if !self.should_flip {
            return preferred;
        }
        let opposite = preferred.opposite();
        let arrow = if self.has_arrow { px(12.) } else { px(0.) };
        let extent = match preferred {
            PopoverSide::Top | PopoverSide::Bottom => popup.height + self.offset,
            PopoverSide::Left | PopoverSide::Right => popup.width + self.offset,
        } + arrow;
        if extent <= self.available(preferred, trigger, viewport) {
            preferred
        } else if extent <= self.available(opposite, trigger, viewport) {
            opposite
        } else if self.available(preferred, trigger, viewport)
            >= self.available(opposite, trigger, viewport)
        {
            preferred
        } else {
            opposite
        }
    }

    fn origin(
        &self,
        side: PopoverSide,
        trigger: Bounds<Pixels>,
        popup: Size<Pixels>,
        viewport: Size<Pixels>,
    ) -> gpui::Point<Pixels> {
        use herogpui_core::PlacementAlign;

        let gap = self.offset + if self.has_arrow { px(12.) } else { px(0.) };
        let align = self.placement.align();
        let aligned_x = match align {
            PlacementAlign::Start => trigger.left(),
            PlacementAlign::Center => trigger.center().x - px(f32::from(popup.width) / 2.0),
            PlacementAlign::End => trigger.right() - popup.width,
        };
        let aligned_y = match align {
            PlacementAlign::Start => trigger.top(),
            PlacementAlign::Center => trigger.center().y - px(f32::from(popup.height) / 2.0),
            PlacementAlign::End => trigger.bottom() - popup.height,
        };
        let mut origin = match side {
            PopoverSide::Top => point(aligned_x, trigger.top() - popup.height - gap),
            PopoverSide::Bottom => point(aligned_x, trigger.bottom() + gap),
            PopoverSide::Left => point(trigger.left() - popup.width - gap, aligned_y),
            PopoverSide::Right => point(trigger.right() + gap, aligned_y),
        };

        let inset = if self.constrain_height {
            px(12.)
        } else {
            px(0.)
        };
        let max_x = (viewport.width - popup.width - inset).max(inset);
        let max_y = (viewport.height - popup.height - inset).max(inset);
        if matches!(side, PopoverSide::Top | PopoverSide::Bottom) {
            origin.x = origin.x.max(inset).min(max_x);
            if self.should_flip && !self.constrain_height {
                origin.y = origin.y.max(inset).min(max_y);
            }
        } else {
            origin.y = origin.y.max(inset).min(max_y);
            if self.should_flip && !self.constrain_height {
                origin.x = origin.x.max(inset).min(max_x);
            }
        }
        origin
    }

    fn arrow_origin(
        side: PopoverSide,
        trigger: Bounds<Pixels>,
        panel: Bounds<Pixels>,
    ) -> gpui::Point<Pixels> {
        let size = px(12.);
        let max_x = (panel.right() - size).max(panel.left());
        let max_y = (panel.bottom() - size).max(panel.top());
        match side {
            PopoverSide::Top => point(
                (trigger.center().x - size / 2.)
                    .max(panel.left())
                    .min(max_x),
                panel.bottom(),
            ),
            PopoverSide::Bottom => point(
                (trigger.center().x - size / 2.)
                    .max(panel.left())
                    .min(max_x),
                panel.top() - size,
            ),
            PopoverSide::Left => point(
                panel.right(),
                (trigger.center().y - size / 2.).max(panel.top()).min(max_y),
            ),
            PopoverSide::Right => point(
                panel.left() - size,
                (trigger.center().y - size / 2.).max(panel.top()).min(max_y),
            ),
        }
    }
}

impl ParentElement for PopoverPositioner {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

struct PopoverPositionerState {
    children: Vec<LayoutId>,
}

impl Element for PopoverPositioner {
    type RequestLayoutState = PopoverPositionerState;
    type PrepaintState = bool;

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let children = self
            .children
            .iter_mut()
            .map(|child| child.request_layout(window, cx))
            .collect::<Vec<_>>();
        let layout = window.request_layout(
            Style {
                position: Position::Absolute,
                display: Display::Flex,
                ..Style::default()
            },
            children.iter().copied(),
            cx,
        );
        (layout, PopoverPositionerState { children })
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        state: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        if state.children.is_empty() {
            return false;
        }
        let Some(trigger) = self.trigger.get() else {
            return false;
        };
        let viewport = window.viewport_size();
        // A trigger-width panel resolves `w_full` against this width, not
        // against `MaxContent`.
        let width_space = if self.match_trigger_width {
            gpui::AvailableSpace::Definite(trigger.size.width)
        } else {
            gpui::AvailableSpace::MaxContent
        };
        let mut popup = window.layout_bounds(state.children[0]).size;
        if self.constrain_height {
            popup = self.children[0].layout_as_root(
                gpui::size(width_space, gpui::AvailableSpace::MaxContent),
                window,
                cx,
            );
        }
        let side = self.resolved_side(trigger, popup, viewport);
        if self.constrain_height {
            let max_height = match side {
                PopoverSide::Top | PopoverSide::Bottom => {
                    self.available(side, trigger, viewport) - self.offset
                }
                PopoverSide::Left | PopoverSide::Right => {
                    viewport.height - self.origin(side, trigger, popup, viewport).y - px(12.)
                }
            }
            .max(px(0.));
            popup = self.children[0].layout_as_root(
                gpui::size(width_space, gpui::AvailableSpace::Definite(max_height)),
                window,
                cx,
            );
        }
        let origin = self.origin(side, trigger, popup, viewport);
        self.resolved.set(Some(PopoverResolved {
            trigger,
            panel: Bounds {
                origin,
                size: popup,
            },
            side,
        }));
        let layout_origin = if self.constrain_height {
            window.layout_bounds(state.children[0]).origin
        } else {
            bounds.origin
        };
        let offset = origin - layout_origin;
        let offset = point(offset.x.round(), offset.y.round());
        window.with_element_offset(offset, |window| {
            self.children[0].prepaint(window, cx);
        });

        true
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        prepainted: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        if !*prepainted {
            return;
        }
        self.children[0].paint(window, cx);
    }
}

impl IntoElement for PopoverPositioner {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

/// The panel must use `max_h_full()` and own its vertical scroll container.
pub(crate) fn scrollable_popover(
    trigger: std::rc::Rc<std::cell::Cell<Option<Bounds<Pixels>>>>,
    placement: PopoverPlacement,
    panel: impl IntoElement,
) -> impl IntoElement {
    let mut positioner = PopoverPositioner::new(
        trigger,
        std::rc::Rc::new(std::cell::Cell::new(None)),
        placement,
        px(8.),
        true,
        false,
    );
    positioner.constrain_height = true;
    positioner.child(panel)
}

/// A trigger-width variant of [`scrollable_popover`] for field panels.
///
/// `Select.Popover` is `min-w-(--trigger-width)`: the panel matches the
/// trigger width, so the positioner measures with that width on both passes
/// instead of `MaxContent`. The panel must still use `max_h_full()`; a plain
/// list owns the panel's vertical scroll container, while a virtual list
/// sizes itself from its rows (`Infer`) so it caps together with the panel
/// instead of nesting a fixed viewport inside an outer scroller.
pub(crate) fn scrollable_field_popover(
    trigger: std::rc::Rc<std::cell::Cell<Option<Bounds<Pixels>>>>,
    placement: PopoverPlacement,
    panel: impl IntoElement,
) -> impl IntoElement {
    let mut positioner = PopoverPositioner::new(
        trigger,
        std::rc::Rc::new(std::cell::Cell::new(None)),
        placement,
        px(8.),
        true,
        false,
    );
    positioner.constrain_height = true;
    positioner.match_trigger_width = true;
    positioner.child(panel)
}

/// HeroUI Popover (controlled).
#[derive(IntoElement)]
pub struct Popover {
    /// Distinguishes this popover's uncontrolled state from its neighbours'.
    id: gpui::ElementId,
    trigger: AnyElement,
    /// `isOpen` — `None` leaves the component holding the state, seeded
    /// from `defaultOpen`.
    is_open: Option<bool>,
    default_open: bool,
    placement: PopoverPlacement,
    title: Option<SharedString>,
    show_close_button: bool,
    offset: Pixels,
    should_flip: bool,
    on_open_change: Option<OnOpenChange>,
    children: Vec<AnyElement>,
}

impl Popover {
    pub fn new(trigger: impl IntoElement) -> Self {
        Self {
            id: gpui::ElementId::Name("popover".into()),
            trigger: trigger.into_any_element(),
            is_open: None,
            default_open: false,
            placement: PopoverPlacement::Bottom,
            offset: px(8.),
            should_flip: true,
            title: None,
            show_close_button: false,
            on_open_change: None,
            children: Vec::new(),
        }
    }

    /// Distinguishes this popover from its neighbours.
    ///
    /// Only matters in the uncontrolled mode, where the open flag lives in
    /// element state: two popovers sharing a key would open together.
    pub fn id(mut self, id: impl Into<gpui::ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn is_open(mut self, v: bool) -> Self {
        self.is_open = Some(v);
        self
    }

    /// `defaultOpen` — the uncontrolled initial state.
    ///
    /// Only consulted when `is_open` is not supplied; the component then owns
    /// the flag and the trigger toggles it.
    pub fn default_open(mut self, v: bool) -> Self {
        self.default_open = v;
        self
    }

    /// `offset` — distance from the trigger, 8px in v3.
    pub fn offset(mut self, offset: impl Into<Pixels>) -> Self {
        self.offset = offset.into();
        self
    }

    /// `shouldFlip` — lets the panel reposition to stay inside the window.
    pub fn should_flip(mut self, v: bool) -> Self {
        self.should_flip = v;
        self
    }

    pub fn placement(mut self, p: PopoverPlacement) -> Self {
        self.placement = p;
        self
    }

    /// Optional bold header inside the panel.
    pub fn title(mut self, t: impl Into<SharedString>) -> Self {
        self.title = Some(t.into());
        self
    }

    pub fn show_close_button(mut self, v: bool) -> Self {
        self.show_close_button = v;
        self
    }

    /// Toggle handler wired to the trigger click.
    pub fn on_open_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_open_change = Some(std::sync::Arc::new(f));
        self
    }
}

impl ParentElement for Popover {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Popover {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        let (is_open, open_own) = crate::util::controlled(
            window,
            cx,
            gpui::ElementId::Name(format!("{:?}-open", self.id).into()),
            self.is_open,
            self.default_open,
        );
        // v3 keeps a closing panel on screen for its `[data-exiting]` run.
        // `overlay_phase` takes `cx` mutably too, so it goes here.
        let phase_key = gpui::ElementId::Name(format!("{:?}-popover-phase", self.id).into());
        let (phase, dismissal_token) =
            crate::util::overlay_scope(window, cx, phase_key, is_open, true);
        let exiting = phase == crate::util::OverlayPhase::Exiting;
        // Escape is read on the root, and a key event only reaches an element
        // that is on the focused element's path -- so an open panel needs
        // *something* inside this root to hold the focus. A click on the
        // trigger does that by itself, which is why the keyboard appeared to
        // work; a caller that drives `isOpen` focuses nothing, and Escape went
        // to the app root instead. The root handle is how the component asks
        // whether anything inside it already has the focus, and it is
        // deliberately not a tab stop: the popover adds no stop of its own.
        // Both `use_keyed_state` calls take `cx` mutably, so they precede the
        // theme tokens.
        let base = format!("{:?}", self.id);
        let anchor_bounds = std::rc::Rc::new(std::cell::Cell::new(None::<Bounds<Pixels>>));
        let resolved = std::rc::Rc::new(std::cell::Cell::new(None::<PopoverResolved>));
        let root_focus = window
            .use_keyed_state(
                gpui::ElementId::Name(format!("{base}-root-focus").into()),
                cx,
                |_, cx| cx.focus_handle(),
            )
            .read(cx)
            .clone();
        // A v3 popover is a dialog focus scope. It claims focus on every open
        // transition, contains Tab inside the panel, and restores the handle
        // that opened it when it closes.
        let claim = is_open;
        let trigger_focus = crate::util::panel_restore_focus(window, cx, &base);
        let panel_focus = crate::util::panel_focus(window, cx, &base, claim);
        // The panel's outside-press capture runs on mouse-down, before the
        // trigger's click listener runs on mouse-up. Keep this latch for one
        // dispatch so an open trigger is owned by its own toggle, not by both
        // dismissal paths.
        let trigger_pressed = std::rc::Rc::new(std::cell::Cell::new(false));
        let colors = cx.colors();
        let layout = cx.layout();

        let mut trigger_wrap = gpui::div()
            .id(gpui::ElementId::Name(
                format!("{:?}-trigger", self.id).into(),
            ))
            .flex()
            .track_focus(&trigger_focus)
            .cursor_pointer();
        if self.on_open_change.is_some() || open_own.is_some() {
            let on_open_change = self.on_open_change.clone();
            let own = open_own.clone();
            let open = is_open;
            let capture_pressed = trigger_pressed.clone();
            let click_pressed = trigger_pressed.clone();
            let toggle = crate::util::shared(move |window: &mut Window, cx: &mut App| {
                if let Some(held) = &own {
                    held.update(cx, |value, cx| {
                        *value = !open;
                        cx.notify();
                    });
                }
                if let Some(cb) = &on_open_change {
                    cb(!open, window, cx);
                }
            });
            trigger_wrap = trigger_wrap
                .capture_any_mouse_down(move |_, _, cx| {
                    capture_pressed.set(true);
                    let clear = capture_pressed.clone();
                    cx.defer(move |_| clear.set(false));
                })
                .on_key_down({
                    let toggle = toggle.clone();
                    move |event, window, cx| {
                        if matches!(event.keystroke.key.as_str(), "enter" | "space")
                            && trigger_focus.contains_focused(window, cx)
                        {
                            // Moving focus between key-down and key-up cancels
                            // GPUI's keyboard click. Activate without transferring it.
                            if !event.is_held {
                                toggle(window, cx);
                            }
                            window.prevent_default();
                            cx.stop_propagation();
                        }
                    }
                })
                .on_click(move |_: &ClickEvent, window, cx| {
                    toggle(window, cx);
                    click_pressed.set(false);
                });
        }

        let trigger =
            PopoverTriggerMeasure::new(trigger_wrap.child(self.trigger), anchor_bounds.clone());
        let mut root = gpui::div()
            .track_focus(&root_focus)
            .relative()
            .flex()
            .flex_col()
            .items_start()
            .child(trigger);

        if phase == crate::util::OverlayPhase::Closed {
            return root;
        }

        let close = crate::util::shared({
            let own = open_own;
            let cb = self.on_open_change.clone();
            move |window: &mut Window, cx: &mut App| -> crate::util::DismissResult {
                if let Some(held) = &own {
                    held.update(cx, |v, cx| {
                        *v = false;
                        cx.notify();
                    });
                }
                if let Some(cb) = &cb {
                    cb(false, window, cx);
                }
                crate::util::DismissResult::Handled
            }
        });

        // Panel
        // `.popover__heading` is the title beside the close button.
        let mut header_row = gpui::div().flex().items_center().justify_between();
        if let Some(title) = &self.title {
            header_row = header_row.child(
                gpui::div()
                    .text_size(px(14.))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(colors.foreground)
                    .child(title.to_string()),
            );
        } else {
            header_row = header_row.child("");
        }
        if self.show_close_button {
            let close_button = close.clone();
            header_row = header_row.child(
                crate::close_button::CloseButton::new(gpui::ElementId::Name(
                    format!("{base}-close").into(),
                ))
                .on_press(move |_, window, cx| {
                    close_button(window, cx);
                }),
            );
        }

        let mut panel = gpui::div()
            .w(px(260.))
            .flex()
            .flex_col()
            .gap(px(8.))
            .px(px(16.))
            .py(px(16.))
            .bg(colors.overlay.background)
            .text_color(colors.surface.foreground)
            // `.popover` is `text-sm`.
            .text_size(px(14.))
            .line_height(px(20.))
            .rounded(crate::util::container_radius(cx))
            // v3 gives a floating panel no border: `.popover` and friends are
            // `bg-overlay shadow-overlay` and a radius, and dark mode's
            // inset hairline is what separates the panel from the page.
            .when_some(layout.overlay_hairline, |el, hairline| {
            el.border(layout.border_width).border_color(hairline)
            })
            .shadow(layout.overlay_shadow.clone());

        if self.title.is_some() || self.show_close_button {
            panel = panel.child(header_row);
        }
        // v3 composes the arrow as a part inside the panel's children. Hand
        // every composed `Popover.Arrow` the resolved placement it needs to
        // draw, and let the positioner reserve the arrow's 12px of gap.
        let mut has_arrow = false;
        for child in &mut self.children {
            if let Some(arrow) = child.downcast_mut::<PopoverArrow>() {
                arrow.resolved = resolved.clone();
                has_arrow = true;
            }
        }
        panel = panel.children(self.children);

        // The panel owns the dialog scope. Its handle is not a tab stop, but
        // the close affordance and child controls are, and Tab wraps among
        // those descendants without reaching the trigger or the page behind.
        let panel = crate::util::trap_tab(panel.track_focus(&panel_focus), &panel_focus);

        // React Aria dismisses a popover on Escape and on a press outside it.
        //
        let outside_close = close.clone();
        let trigger_pressed_for_dismissal = trigger_pressed;
        let panel = crate::util::dismiss_on_press_outside_with_token(
            panel,
            dismissal_token.clone(),
            move |window, cx| {
                if trigger_pressed_for_dismissal.get() {
                    return crate::util::DismissResult::Declined;
                }
                outside_close(window, cx);
                crate::util::DismissResult::Handled
            },
        );
        root =
            crate::util::dismiss_on_escape_with_token(root, dismissal_token, move |window, cx| {
                close(window, cx);
                crate::util::DismissResult::Handled
            });

        // v3 fades the panel in on `[data-entering]`.
        let zoom = crate::anim::ZoomBox::panel(px(12.), crate::util::container_radius(cx))
            .padding_x(px(14.))
            .sized(px(260.));
        let panel = if exiting {
            crate::anim::exiting(
                panel,
                "popover-panel-out",
                zoom,
                crate::anim::Motion::LIST_OUT,
                cx,
            )
        } else {
            crate::anim::entering_zoom(
                panel,
                "popover-panel",
                zoom,
                crate::anim::Motion::POPOVER_IN,
                cx,
            )
        };

        let positioner = PopoverPositioner::new(
            anchor_bounds,
            resolved,
            self.placement,
            self.offset,
            self.should_flip,
            has_arrow,
        )
        .child(panel);
        root = root.child(positioner);
        root
    }
}
