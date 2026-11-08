//! Frame snapshots, structural diffs and point compositing for the
//! interaction sweep.
//!
//! `Window::painted_quads()` / `painted_underlines()` expose the last
//! rendered frame's primitives in *scaled* pixels, sorted by paint `order`.
//! Glyphs, SVG icons and shadows are not observable headlessly (the test
//! text system rasterizes nothing and there is no asset source), so
//! underlines are the only proxy for where text paints and in what order.
//!
//! Frames are compared by *compositing* them at sample points rather than
//! by primitive identity: `hover_fade` legitimately moves the fill between
//! the element itself and an absolutely positioned child across the first
//! hover, so structurally different scenes can be pixel-identical.

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;

use gpui::{Bounds, Corners, Hsla, Quad, Rgba, ScaledPixels, Underline};

/// One rendered frame's observable primitives, in paint order.
#[derive(Clone, Debug, Default)]
pub(crate) struct Snapshot {
    pub quads: Vec<Quad>,
    pub underlines: Vec<Underline>,
}

/// Side length of the preview stage's dot grid marks, in logical px —
/// `stage_dot_grid`'s `dot_size`. They are the only primitives dropped:
/// every other quad, including small indicators and step dots, stays
/// observable.
const STAGE_DOT: f32 = 2.5;
/// Match tolerance on [`STAGE_DOT`], in logical px.
const STAGE_DOT_TOLERANCE: f32 = 0.02;

impl Snapshot {
    /// Captures the last rendered frame, dropping only the stage dot grid's
    /// 2.5px marks so they cannot drown every diff.
    pub(crate) fn capture(window: &gpui::Window, scale: f32) -> Self {
        let mut snapshot = Self {
            quads: window.painted_quads(),
            underlines: window.painted_underlines(),
        };
        snapshot.quads.retain(|quad| {
            let width = quad.bounds.size.width.as_f32() / scale;
            let height = quad.bounds.size.height.as_f32() / scale;
            !((width - STAGE_DOT).abs() <= STAGE_DOT_TOLERANCE
                && (height - STAGE_DOT).abs() <= STAGE_DOT_TOLERANCE)
        });
        snapshot
    }
}

/// A quad's identity for structural comparison: every painted field except
/// `order` (paint order is compared through list position instead).
fn quad_key(quad: &Quad) -> String {
    let mut key = String::with_capacity(224);
    write!(
        key,
        "{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}",
        quad.border_style,
        quad.bounds,
        quad.content_mask,
        quad.background,
        quad.border_color,
        quad.corner_radii,
        quad.border_widths
    )
    .unwrap();
    key
}

/// A quad's *geometry* identity for the monotonic-leave check: bounds,
/// content mask, corner radii and border widths — everything but colour and
/// paint order. Two quads sharing this key are the same painted shape; a
/// hover layer that snaps off and reveals what is underneath has no match,
/// while a fill whose colour reverses keeps its geometry.
fn geometry_key(quad: &Quad) -> String {
    let mut key = String::with_capacity(160);
    write!(
        key,
        "{:?}|{:?}|{:?}|{:?}",
        quad.bounds, quad.content_mask, quad.corner_radii, quad.border_widths
    )
    .unwrap();
    key
}

fn underline_key(underline: &Underline) -> String {
    format!(
        "{:?}|{:?}|{:?}|{:?}|{:?}",
        underline.bounds,
        underline.content_mask,
        underline.color,
        underline.thickness,
        underline.wavy
    )
}

/// The multiset symmetric difference between two snapshots: primitives
/// present in one but not the other.
#[derive(Default)]
pub(crate) struct SceneDiff {
    /// Quads only in the first snapshot.
    pub quads_a: Vec<Quad>,
    /// Quads only in the second snapshot.
    pub quads_b: Vec<Quad>,
    /// Underlines only in the first snapshot.
    pub underlines_a: Vec<Underline>,
    /// Underlines only in the second snapshot.
    pub underlines_b: Vec<Underline>,
}

impl SceneDiff {
    pub(crate) fn is_empty(&self) -> bool {
        self.quads_a.is_empty()
            && self.quads_b.is_empty()
            && self.underlines_a.is_empty()
            && self.underlines_b.is_empty()
    }
}

fn diff_quads(a: &[Quad], b: &[Quad]) -> (Vec<Quad>, Vec<Quad>) {
    diff_by_key(a, b, quad_key)
}

fn diff_underlines(a: &[Underline], b: &[Underline]) -> (Vec<Underline>, Vec<Underline>) {
    diff_by_key(a, b, underline_key)
}

/// `a` vs `b`: primitives unique to each side.
pub(crate) fn diff(a: &Snapshot, b: &Snapshot) -> SceneDiff {
    diff_with(a, b, false)
}

/// `a` vs `b` with paint `order` included in primitive identity. Discovery
/// uses this: `hover_fade`'s element→fill move can produce two quads that
/// differ only in paint order, which the visual [`diff`] cancels — and a
/// target that never diffs is never swept.
pub(crate) fn diff_ordered(a: &Snapshot, b: &Snapshot) -> SceneDiff {
    diff_with(a, b, true)
}

fn diff_with(a: &Snapshot, b: &Snapshot, include_order: bool) -> SceneDiff {
    let (quads_a, quads_b) = if include_order {
        diff_quads_ordered(&a.quads, &b.quads)
    } else {
        diff_quads(&a.quads, &b.quads)
    };
    let (underlines_a, underlines_b) = if include_order {
        diff_underlines_ordered(&a.underlines, &b.underlines)
    } else {
        diff_underlines(&a.underlines, &b.underlines)
    };
    SceneDiff {
        quads_a,
        quads_b,
        underlines_a,
        underlines_b,
    }
}

fn quad_key_ordered(quad: &Quad) -> String {
    let mut key = quad_key(quad);
    write!(key, "|{}", quad.order).unwrap();
    key
}

fn underline_key_ordered(underline: &Underline) -> String {
    let mut key = underline_key(underline);
    write!(key, "|{}", underline.order).unwrap();
    key
}

fn diff_quads_ordered(a: &[Quad], b: &[Quad]) -> (Vec<Quad>, Vec<Quad>) {
    diff_by_key(a, b, quad_key_ordered)
}

fn diff_underlines_ordered(a: &[Underline], b: &[Underline]) -> (Vec<Underline>, Vec<Underline>) {
    diff_by_key(a, b, underline_key_ordered)
}

fn diff_by_key<T: Copy>(a: &[T], b: &[T], key: fn(&T) -> String) -> (Vec<T>, Vec<T>) {
    let mut unmatched: HashMap<String, Vec<T>> = HashMap::new();
    for item in a {
        unmatched.entry(key(item)).or_default().push(*item);
    }
    let mut only_b = Vec::new();
    for item in b {
        let item_key = key(item);
        match unmatched.get_mut(&item_key) {
            Some(bucket) if !bucket.is_empty() => {
                bucket.pop();
            }
            _ => only_b.push(*item),
        }
    }
    let mut only_a = Vec::new();
    for bucket in unmatched.into_values() {
        only_a.extend(bucket);
    }
    (only_a, only_b)
}

/// One point where two frames composite differently.
pub(crate) struct Mismatch {
    /// Logical-pixel coordinates, for reporting.
    pub point: (f32, f32),
    /// `self`'s composite: `rgba(r,g,b,a)` or `structural[…]`.
    pub a: String,
    /// `other`'s composite.
    pub b: String,
}

/// The result of [`equivalent`].
pub(crate) enum Equivalence {
    Equal,
    Different { points: Vec<Mismatch> },
}

impl Equivalence {
    pub(crate) fn is_equal(&self) -> bool {
        matches!(self, Equivalence::Equal)
    }
}

/// Maximum per-channel difference tolerated before two composites count as
/// different: 2/255 in 0–1 channel units.
const CHANNEL_TOLERANCE: f32 = 2. / 255.;

/// Sample points (scaled px) for one quad: a 3×3 grid at 0.2/0.5/0.8 when
/// both sides are at least 6 logical px, else the centre; plus the midpoint
/// of each bordered side inset by half the border. Points outside the
/// content mask are dropped.
pub(crate) fn sample_points(quad: &Quad, scale: f32) -> Vec<(f32, f32)> {
    let left = quad.bounds.left().as_f32();
    let top = quad.bounds.top().as_f32();
    let right = quad.bounds.right().as_f32();
    let bottom = quad.bounds.bottom().as_f32();
    let width = right - left;
    let height = bottom - top;
    let mut points = Vec::with_capacity(13);
    if width / scale >= 6. && height / scale >= 6. {
        for fy in [0.2, 0.5, 0.8] {
            for fx in [0.2, 0.5, 0.8] {
                points.push((left + fx * width, top + fy * height));
            }
        }
    } else {
        points.push(((left + right) / 2., (top + bottom) / 2.));
    }
    let cx = (left + right) / 2.;
    let cy = (top + bottom) / 2.;
    let borders = &quad.border_widths;
    if borders.top.as_f32() > 0. {
        points.push((cx, top + borders.top.as_f32() / 2.));
    }
    if borders.bottom.as_f32() > 0. {
        points.push((cx, bottom - borders.bottom.as_f32() / 2.));
    }
    if borders.left.as_f32() > 0. {
        points.push((left + borders.left.as_f32() / 2., cy));
    }
    if borders.right.as_f32() > 0. {
        points.push((right - borders.right.as_f32() / 2., cy));
    }
    let mask = &quad.content_mask.bounds;
    points.retain(|point| point_in_bounds(*point, mask));
    points
}

fn point_in_bounds(point: (f32, f32), bounds: &Bounds<ScaledPixels>) -> bool {
    point.0 >= bounds.left().as_f32()
        && point.0 <= bounds.right().as_f32()
        && point.1 >= bounds.top().as_f32()
        && point.1 <= bounds.bottom().as_f32()
}

/// Whether `point` lies inside the quad's rounded outer shape (and its
/// content mask). Mirrors the `quad_sdf` test in GPUI's quad shader.
pub(crate) fn covers_point(quad: &Quad, point: (f32, f32)) -> bool {
    if !point_in_bounds(point, &quad.content_mask.bounds) {
        return false;
    }
    inside_rounded(&quad.bounds, &quad.corner_radii, point)
}

/// Signed-distance inside test for a rounded rect, mirroring `quad_sdf` in
/// `shaders.wgsl`: negative inside, positive outside.
fn inside_rounded(
    bounds: &Bounds<ScaledPixels>,
    radii: &Corners<ScaledPixels>,
    point: (f32, f32),
) -> bool {
    let left = bounds.left().as_f32();
    let top = bounds.top().as_f32();
    let right = bounds.right().as_f32();
    let bottom = bounds.bottom().as_f32();
    let center_x = (left + right) / 2.;
    let center_y = (top + bottom) / 2.;
    let half_w = (right - left) / 2.;
    let half_h = (bottom - top) / 2.;
    let radius = if point.0 < center_x {
        if point.1 < center_y {
            radii.top_left
        } else {
            radii.bottom_left
        }
    } else if point.1 < center_y {
        radii.top_right
    } else {
        radii.bottom_right
    }
    .as_f32();
    // corner_center_to_point = |point - center| - half_size + radius.
    let cx = (point.0 - center_x).abs() - half_w + radius;
    let cy = (point.1 - center_y).abs() - half_h + radius;
    let sdf = if radius == 0. {
        cx.max(cy)
    } else {
        cx.max(0.).hypot(cy.max(0.)) + cx.max(cy).min(0.) - radius
    };
    sdf <= 0.
}

fn rgba(hsla: Hsla) -> [f32; 4] {
    let color: Rgba = hsla.into();
    [color.r, color.g, color.b, color.a]
}

/// Straight-alpha `over`, matching the shader's border-over-background blend.
fn over(below: [f32; 4], above: [f32; 4]) -> [f32; 4] {
    let alpha = above[3] + below[3] * (1. - above[3]);
    if alpha <= 0. {
        return [0.; 4];
    }
    [
        (above[0] * above[3] + below[0] * below[3] * (1. - above[3])) / alpha,
        (above[1] * above[3] + below[1] * below[3] * (1. - above[3])) / alpha,
        (above[2] * above[3] + below[2] * below[3] * (1. - above[3])) / alpha,
        alpha,
    ]
}

/// The inner (background) shape of a quad: bounds inset by the border widths,
/// corner radii shrunk by the adjacent border. `None` when the border
/// consumes the whole quad.
fn inner_shape(quad: &Quad) -> Option<(Bounds<ScaledPixels>, Corners<ScaledPixels>)> {
    let widths = &quad.border_widths;
    if widths.top.as_f32() <= 0.
        && widths.bottom.as_f32() <= 0.
        && widths.left.as_f32() <= 0.
        && widths.right.as_f32() <= 0.
    {
        return Some((quad.bounds, quad.corner_radii));
    }
    let left = quad.bounds.left().as_f32() + widths.left.as_f32();
    let top = quad.bounds.top().as_f32() + widths.top.as_f32();
    let right = quad.bounds.right().as_f32() - widths.right.as_f32();
    let bottom = quad.bounds.bottom().as_f32() - widths.bottom.as_f32();
    if right <= left || bottom <= top {
        return None;
    }
    let radius = |outer: ScaledPixels, horizontal: ScaledPixels, vertical: ScaledPixels| {
        ScaledPixels((outer.as_f32() - horizontal.as_f32().max(vertical.as_f32())).max(0.))
    };
    let radii = Corners {
        top_left: radius(quad.corner_radii.top_left, widths.left, widths.top),
        top_right: radius(quad.corner_radii.top_right, widths.right, widths.top),
        bottom_right: radius(quad.corner_radii.bottom_right, widths.right, widths.bottom),
        bottom_left: radius(quad.corner_radii.bottom_left, widths.left, widths.bottom),
    };
    Some((
        Bounds::new(
            gpui::point(ScaledPixels(left), ScaledPixels(top)),
            gpui::size(ScaledPixels(right - left), ScaledPixels(bottom - top)),
        ),
        radii,
    ))
}

/// The colour a quad paints at `point` (scaled px), assuming a solid
/// background: the background inside the inner shape, the border colour
/// composited over it in the border band.
fn quad_color_at(quad: &Quad, point: (f32, f32)) -> [f32; 4] {
    let background = rgba(match quad.background.as_solid() {
        Some(hsla) => hsla,
        // Non-solid backgrounds are handled by the structural fallback
        // before colours are computed; treat as transparent defensively.
        None => return [0.; 4],
    });
    let in_inner = inner_shape(quad).is_some_and(|(bounds, radii)| {
        point_in_bounds(point, &bounds) && inside_rounded(&bounds, &radii, point)
    });
    if in_inner {
        return background;
    }
    // Border band: inside the outer shape but outside the inner one.
    let border = rgba(quad.border_color);
    if border[3] <= 0. {
        return background;
    }
    over(background, border)
}

/// What a snapshot composites to at one point.
pub(crate) enum Composite {
    /// Straight-alpha RGBA in sRGB floats, composited bottom-to-top.
    Color([f32; 4]),
    /// A covering quad has a gradient/pattern background: the ordered keys
    /// of every quad covering the point, for structural comparison.
    Structural(Vec<String>),
}

impl Composite {
    fn describe(&self) -> String {
        match self {
            Composite::Color(color) => format!(
                "rgba({:.3},{:.3},{:.3},{:.3})",
                color[0], color[1], color[2], color[3]
            ),
            Composite::Structural(keys) => {
                let mut out = String::from("structural[");
                for (index, key) in keys.iter().enumerate() {
                    if index > 0 {
                        out.push_str(", ");
                    }
                    // Keys are long; the bounds field identifies the quad.
                    let bounds = key.split('|').nth(1).unwrap_or(key);
                    write!(out, "{bounds}").unwrap();
                }
                out.push(']');
                out
            }
        }
    }
}

/// Composites `snapshot` at `point` (scaled px) in paint order.
pub(crate) fn composite(snapshot: &Snapshot, point: (f32, f32)) -> Composite {
    let mut covering = Vec::new();
    let mut non_solid = false;
    for quad in &snapshot.quads {
        if covers_point(quad, point) {
            covering.push(quad);
            if quad.background.as_solid().is_none() {
                non_solid = true;
            }
        }
    }
    if non_solid {
        return Composite::Structural(covering.iter().map(|quad| quad_key(quad)).collect());
    }
    let mut color = [0.; 4];
    for quad in covering {
        color = over(color, quad_color_at(quad, point));
    }
    Composite::Color(color)
}

/// Channel distance between two composites, 0–1. Structural composites
/// compare by list equality: identical lists are distance 0, anything else
/// counts as maximally different.
pub(crate) fn distance(a: &Composite, b: &Composite) -> f32 {
    match (a, b) {
        (Composite::Color(a), Composite::Color(b)) => (0..4)
            .map(|channel| (a[channel] - b[channel]).abs())
            .fold(0., f32::max),
        (Composite::Structural(a), Composite::Structural(b)) if a == b => 0.,
        _ => 1.,
    }
}

/// The union of [`sample_points`] across every quad in `diffs`, deduplicated.
pub(crate) fn union_sample_points<'a>(
    diffs: impl IntoIterator<Item = &'a SceneDiff>,
    scale: f32,
) -> Vec<(f32, f32)> {
    let mut points = Vec::new();
    let mut seen = HashSet::new();
    for difference in diffs {
        for quad in difference.quads_a.iter().chain(difference.quads_b.iter()) {
            for point in sample_points(quad, scale) {
                let key = (
                    (point.0 * 64.).round() as i64,
                    (point.1 * 64.).round() as i64,
                );
                if seen.insert(key) {
                    points.push(point);
                }
            }
        }
    }
    points
}

/// Whether `a` and `b` composite identically at every sample point of their
/// structural difference. Mismatches (at most five) are reported with
/// logical-pixel coordinates and both composites.
pub(crate) fn equivalent(a: &Snapshot, b: &Snapshot, scale: f32) -> Equivalence {
    let difference = diff(a, b);
    if difference.is_empty() {
        return Equivalence::Equal;
    }
    let points = union_sample_points([&difference], scale);
    let mut mismatches = Vec::new();
    for point in points {
        let ca = composite(a, point);
        let cb = composite(b, point);
        if distance(&ca, &cb) > CHANNEL_TOLERANCE {
            mismatches.push(Mismatch {
                point: (point.0 / scale, point.1 / scale),
                a: ca.describe(),
                b: cb.describe(),
            });
            if mismatches.len() >= 5 {
                break;
            }
        }
    }
    if mismatches.is_empty() {
        Equivalence::Equal
    } else {
        Equivalence::Different { points: mismatches }
    }
}

/// One sample point where leaving moved the composite *away* from rest.
pub(crate) struct LeaveViolation {
    /// Scaled-pixel coordinates.
    pub point: (f32, f32),
    pub before: f32,
    pub after: f32,
}

/// Check (2) core: on the sample points of the rest↔`prev` and rest↔`next`
/// diffs that belong to the hovered control — only quads whose bounds contain
/// `target` are sampled, so an overlay elsewhere (a tooltip panel) is not
/// judged against the control — leaving must never increase a point's
/// distance from the rest composite.
///
/// A sample point is skipped when the topmost quad covering it in `prev` has
/// no *geometry* match in `next`: an instantly removed hover layer reveals
/// what is fading underneath rather than reversing a colour, so there is no
/// reversal to judge there. A fill whose colour snaps keeps its geometry, so
/// the snap-on-leave regression still fails.
pub(crate) fn monotonic_leave_violations(
    rest: &Snapshot,
    prev: &Snapshot,
    next: &Snapshot,
    target: (f32, f32),
    scale: f32,
) -> Vec<LeaveViolation> {
    let target_scaled = (target.0 * scale, target.1 * scale);
    let mut quads: Vec<Quad> = Vec::new();
    for difference in [diff(rest, prev), diff(rest, next)] {
        for quad in difference.quads_a.iter().chain(difference.quads_b.iter()) {
            if point_in_bounds(target_scaled, &quad.bounds) {
                quads.push(*quad);
            }
        }
    }
    let mut points = Vec::new();
    let mut seen = HashSet::new();
    for quad in &quads {
        for point in sample_points(quad, scale) {
            let key = (
                (point.0 * 64.).round() as i64,
                (point.1 * 64.).round() as i64,
            );
            if seen.insert(key) {
                points.push(point);
            }
        }
    }
    let next_geometry: HashSet<String> = next.quads.iter().map(geometry_key).collect();
    let mut violations = Vec::new();
    for sample in points {
        let mut topmost = None;
        for quad in &prev.quads {
            if covers_point(quad, sample) {
                topmost = Some(quad);
            }
        }
        if let Some(top) = topmost {
            if !next_geometry.contains(&geometry_key(top)) {
                continue;
            }
        }
        let rest_color = composite(rest, sample);
        let before = distance(&composite(prev, sample), &rest_color);
        let after = distance(&composite(next, sample), &rest_color);
        if after > before + CHANNEL_TOLERANCE {
            violations.push(LeaveViolation {
                point: sample,
                before,
                after,
            });
        }
    }
    violations
}

/// The key set of a snapshot's quads, for "structurally present" tests.
pub(crate) fn quad_keys(snapshot: &Snapshot) -> HashSet<String> {
    snapshot.quads.iter().map(quad_key).collect()
}

/// Whether `quad`'s key appears in `keys`.
pub(crate) fn key_present(keys: &HashSet<String>, quad: &Quad) -> bool {
    keys.contains(&quad_key(quad))
}

/// Bounds of `quad` in logical pixels, for reporting.
pub(crate) fn logical_bounds(quad: &Quad, scale: f32) -> (f32, f32, f32, f32) {
    (
        quad.bounds.left().as_f32() / scale,
        quad.bounds.top().as_f32() / scale,
        quad.bounds.size.width.as_f32() / scale,
        quad.bounds.size.height.as_f32() / scale,
    )
}

/// Check (3): in frame `frame`, each offending quad paired with the underlines
/// it covers. A quad `Q` and underline `U` offend when the quad paints after
/// the underline (`Q.order > U.order`), is new versus the rest frame, has a
/// solid background with alpha ≥ 0.1, its masked bounds contain the sweep
/// `point`, and its masked rounded shape contains the underline's centre.
/// One entry per offending quad keeps the report readable.
pub(crate) fn fill_over_text(
    frame: &Snapshot,
    rest_keys: &HashSet<String>,
    point: (f32, f32),
    scale: f32,
) -> Vec<(Quad, Vec<Underline>)> {
    let point_scaled = (point.0 * scale, point.1 * scale);
    // Quads that could offend at all: new versus rest, solid enough to read,
    // and covering the sweep point.
    let suspects: Vec<&Quad> = frame
        .quads
        .iter()
        .filter(|quad| {
            quad.background.as_solid().is_some_and(|hsla| hsla.a >= 0.1)
                && !key_present(rest_keys, quad)
                && point_in_bounds(point_scaled, &quad.bounds)
                && point_in_bounds(point_scaled, &quad.content_mask.bounds)
        })
        .collect();
    let mut offenders = Vec::new();
    for quad in suspects {
        let mut covered = Vec::new();
        for underline in &frame.underlines {
            let center = underline.bounds.center();
            let center_scaled = (center.x.as_f32(), center.y.as_f32());
            if !point_in_bounds(center_scaled, &underline.content_mask.bounds) {
                continue;
            }
            if quad.order <= underline.order {
                continue;
            }
            if !covers_point(quad, center_scaled) {
                continue;
            }
            covered.push(*underline);
        }
        if !covered.is_empty() {
            offenders.push((*quad, covered));
        }
    }
    offenders
}

/// Any primitive's bounds in logical pixels, for reporting.
pub(crate) fn logical_bounds_from_bounds(
    bounds: &Bounds<ScaledPixels>,
    scale: f32,
) -> (f32, f32, f32, f32) {
    (
        bounds.left().as_f32() / scale,
        bounds.top().as_f32() / scale,
        bounds.size.width.as_f32() / scale,
        bounds.size.height.as_f32() / scale,
    )
}
