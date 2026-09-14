//! Rounded clipping keeps each ancestor's geometry separate from culling bounds.

use crate::{px, Bounds, ContentMask, Corners, Pixels, Point, ScaledPixels};
use smallvec::SmallVec;
use std::fmt::Debug;

/// One immutable rounded rectangle in a scene's clip chain.
/// Separate horizontal and vertical radii preserve corners inset by unequal borders.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct RoundedClip<P: Clone + Debug + Default + PartialEq> {
    /// Original geometry, never replaced by an intersection's bounding box.
    pub bounds: Bounds<P>,
    /// Horizontal corner radii.
    pub radii_x: Corners<P>,
    /// Vertical corner radii.
    pub radii_y: Corners<P>,
    /// One-based index of the next clip in the scene; zero ends the chain.
    pub parent: u32,
    /// Explicit GPU record padding.
    pub padding: u32,
}

impl RoundedClip<Pixels> {
    /// Convert logical clip geometry to device coordinates.
    pub fn scale(&self, scale: f32) -> RoundedClip<ScaledPixels> {
        RoundedClip {
            bounds: self.bounds.scale(scale),
            radii_x: self.radii_x.scale(scale),
            radii_y: self.radii_y.scale(scale),
            parent: 0,
            padding: 0,
        }
    }

    /// Exact point membership in the rounded rectangle.
    pub fn contains(&self, point: Point<Pixels>) -> bool {
        if self.bounds.is_empty() || !self.bounds.contains(&point) {
            return false;
        }
        let x = f32::from(point.x - self.bounds.left());
        let y = f32::from(point.y - self.bounds.top());
        let right = f32::from(self.bounds.right() - point.x);
        let bottom = f32::from(self.bounds.bottom() - point.y);
        [
            (x, y, self.radii_x.top_left, self.radii_y.top_left),
            (right, y, self.radii_x.top_right, self.radii_y.top_right),
            (
                right,
                bottom,
                self.radii_x.bottom_right,
                self.radii_y.bottom_right,
            ),
            (
                x,
                bottom,
                self.radii_x.bottom_left,
                self.radii_y.bottom_left,
            ),
        ]
        .into_iter()
        .all(|(x, y, rx, ry)| {
            let (rx, ry) = (f32::from(rx), f32::from(ry));
            if rx <= 0.0 || ry <= 0.0 || x >= rx || y >= ry {
                return true;
            }
            let dx = (x - rx) / rx;
            let dy = (y - ry) / ry;
            dx * dx + dy * dy <= 1.0
        })
    }
}

/// The exact intersection of rectangular and rounded ancestor clips.
///
/// `bounds` is only a culling rectangle. Rounded curves keep the bounds and
/// radii of their owning element, even when a narrow descendant intersects them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ClipRegion {
    /// Rectangular intersection used for culling and layout visibility.
    pub bounds: Bounds<Pixels>,
    /// Rounded constraints still relevant within `bounds`.
    pub rounded_clips: SmallVec<[RoundedClip<Pixels>; 1]>,
}

impl From<ContentMask<Pixels>> for ClipRegion {
    fn from(mask: ContentMask<Pixels>) -> Self {
        let radii = mask
            .corner_radii
            .clamp_radii_for_quad_size(mask.bounds.size);
        Self::rounded(mask.bounds, radii, radii)
    }
}

impl ClipRegion {
    /// Construct a clip with elliptical corners. The caller normalizes radii
    /// against the original element before deriving any border inset.
    pub fn rounded(
        bounds: Bounds<Pixels>,
        radii_x: Corners<Pixels>,
        radii_y: Corners<Pixels>,
    ) -> Self {
        let mut region = Self {
            bounds,
            rounded_clips: SmallVec::new(),
        };
        if radii_x.max() > px(0.) && radii_y.max() > px(0.) && !bounds.is_empty() {
            region.rounded_clips.push(RoundedClip {
                bounds,
                radii_x,
                radii_y,
                parent: 0,
                padding: 0,
            });
        }
        region
    }

    /// Intersect without approximating or relocating either region's curves.
    pub fn intersect(&self, other: &Self) -> Self {
        let bounds = self.bounds.intersect(&other.bounds);
        let mut result = Self {
            bounds,
            rounded_clips: SmallVec::new(),
        };
        if bounds.is_empty() {
            return result;
        }
        for clip in self.rounded_clips.iter().chain(&other.rounded_clips) {
            // Logical containment cannot prune a curve: the GPU's conservative
            // culling rectangle can expand across its antialiased edge.
            if result.rounded_clips.contains(clip) {
                continue;
            }
            result.rounded_clips.push(*clip);
        }
        result
    }

    /// Exact logical point membership, independent of rasterization scale.
    pub fn contains(&self, point: Point<Pixels>) -> bool {
        !self.bounds.is_empty()
            && self.bounds.contains(&point)
            && self.rounded_clips.iter().all(|clip| clip.contains(point))
    }
}

/// Concrete clip record exported to the Metal shader bindings.
#[allow(non_camel_case_types)]
pub type RoundedClip_ScaledPixels = RoundedClip<ScaledPixels>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::point;

    #[test]
    fn nested_clips_preserve_both_original_shapes() {
        let outer: ClipRegion = ContentMask {
            bounds: Bounds::from_corners(point(px(0.), px(0.)), point(px(100.), px(100.))),
            corner_radii: Corners::all(px(16.)),
            ..Default::default()
        }
        .into();
        for (left, top, right, bottom) in
            [(4., 4., 96., 96.), (0., 0., 10., 100.), (7., 1., 94., 80.)]
        {
            let inner: ClipRegion = ContentMask {
                bounds: Bounds::from_corners(
                    point(px(left), px(top)),
                    point(px(right), px(bottom)),
                ),
                ..Default::default()
            }
            .into();
            let intersection = outer.intersect(&inner);
            for x in 0..100 {
                for y in 0..100 {
                    let point = point(px(x as f32 + 0.5), px(y as f32 + 0.5));
                    assert_eq!(
                        intersection.contains(point),
                        outer.contains(point) && inner.contains(point)
                    );
                }
            }
        }
    }
}
