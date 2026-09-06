// Tighten a catalog tile around the component it shows.
//
// capture-tiles.ps1 saves the whole gallery client area, so a small control
// sits in a wide field of window background. On the components index that
// field paints as an opaque box inside the stage and the component reads
// tiny. This crops each tile to its content bounding box plus a margin,
// widens the box back to the stage aspect, and clamps the zoom so a tile is
// never magnified past MAX_ZOOM. The window background colour comes from the
// corner pixel; the pixel data is otherwise untouched.

import { PNG } from "pngjs";

/** Stage aspect on the components index (`aspect-[16/10]`). */
export const TILE_ASPECT = 16 / 10;
/** Breathing room kept around the content, in source pixels. */
export const TILE_MARGIN = 28;
/** Upper bound on magnification versus the source tile. */
export const MAX_ZOOM = 2.2;
/** Per-channel difference from the background that counts as content. */
const THRESHOLD = 12;

export function contentBounds(png) {
  const { width, height, data } = png;
  const bg = [data[0], data[1], data[2]];
  let minX = width,
    minY = height,
    maxX = -1,
    maxY = -1;
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const i = (y * width + x) * 4;
      if (
        Math.abs(data[i] - bg[0]) > THRESHOLD ||
        Math.abs(data[i + 1] - bg[1]) > THRESHOLD ||
        Math.abs(data[i + 2] - bg[2]) > THRESHOLD
      ) {
        if (x < minX) minX = x;
        if (x > maxX) maxX = x;
        if (y < minY) minY = y;
        if (y > maxY) maxY = y;
      }
    }
  }
  if (maxX < 0) return null;
  return { minX, minY, maxX, maxY };
}

/**
 * Compute the crop rectangle for a tile of `width`×`height` whose content
 * bounds are `bounds`. Returns integer {x, y, w, h}, always inside the tile.
 */
export function cropRect(width, height, bounds) {
  const full = { x: 0, y: 0, w: width, h: height };
  if (!bounds) return full;
  let w = bounds.maxX - bounds.minX + 1 + TILE_MARGIN * 2;
  let h = bounds.maxY - bounds.minY + 1 + TILE_MARGIN * 2;
  // Widen the shorter side to the stage aspect.
  if (w / h < TILE_ASPECT) w = h * TILE_ASPECT;
  else h = w / TILE_ASPECT;
  // Never magnify beyond MAX_ZOOM, never exceed the source.
  const minW = width / MAX_ZOOM;
  if (w < minW) {
    w = minW;
    h = w / TILE_ASPECT;
  }
  if (w > width || h > height) return full;
  const cx = (bounds.minX + bounds.maxX + 1) / 2;
  const cy = (bounds.minY + bounds.maxY + 1) / 2;
  let x = Math.round(cx - w / 2);
  let y = Math.round(cy - h / 2);
  w = Math.round(w);
  h = Math.round(h);
  x = Math.max(0, Math.min(x, width - w));
  y = Math.max(0, Math.min(y, height - h));
  return { x, y, w, h };
}

/** Crop a PNG buffer; returns the encoded PNG (unchanged bytes if no crop). */
export function cropTile(buffer) {
  const png = PNG.sync.read(buffer);
  const rect = cropRect(png.width, png.height, contentBounds(png));
  if (rect.w === png.width && rect.h === png.height) return buffer;
  const out = new PNG({ width: rect.w, height: rect.h });
  for (let row = 0; row < rect.h; row++) {
    const src = ((rect.y + row) * png.width + rect.x) * 4;
    png.data.copy(out.data, row * rect.w * 4, src, src + rect.w * 4);
  }
  return PNG.sync.write(out);
}
