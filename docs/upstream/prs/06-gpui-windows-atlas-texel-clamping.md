# 06: gpui_windows: Clamp polychrome sprite sampling to atlas tile centers

Patch: [`06-gpui-windows-atlas-texel-clamping.patch`](06-gpui-windows-atlas-texel-clamping.patch).
Base: `gpui-pre-windows 0.3.11` (`zed@f816e1f`). File: `crates/gpui_windows/src/shaders.hlsl` (+8/-1).

`git apply --check`: OK on the `gpui-pre-windows 0.3.11` source tree (`crates/gpui_windows`).
HLSL shader compilation was **not** run locally (this host is macOS; `shaders.hlsl` is compiled on Windows builds of `gpui_windows`).

Still needed at `f816e1f` (`gpui-pre 0.3.11`): yes. `polychrome_sprite_fragment` calls `t_sprite.Sample(s_sprite, input.tile_position)` directly without clamping to the tile's half-texel inset (`shaders.hlsl:1256-1260`).

---

**PR title:** `gpui_windows: Clamp polychrome sprite sampling to atlas tile centers`

**PR body:**

`polychrome_sprite_fragment` samples the Direct3D sprite atlas with `t_sprite.Sample(s_sprite, input.tile_position)` using bilinear filtering. At fractional DPI scaling or non-integer transforms, sample coordinates along a tile edge can straddle the texel boundary and bleed neighboring atlas entries (or transparent gutter pixels) into the rendered image edge.

This clamps `input.tile_position` in `polychrome_sprite_fragment` to the tile's inner texel centers:

```hlsl
float2 atlas_size;
t_sprite.GetDimensions(atlas_size.x, atlas_size.y);
float2 tile_min = (float2(sprite.tile.bounds.origin) + 0.5) / atlas_size;
float2 tile_max = max(
    tile_min,
    (float2(sprite.tile.bounds.origin + sprite.tile.bounds.size) - 0.5) / atlas_size
);
float4 sample = t_sprite.Sample(s_sprite, clamp(input.tile_position, tile_min, tile_max));
```

Taking `max(tile_min, ...)` keeps 1px tiles degenerate-safe (`tile_min == tile_max`) rather than inverting bounds.

Motivation: HeroGPUI and downstream Windows apps (including Poratake) rendering cropped/scaled raster images or multi-sprite UI at 125%/150% DPI scaling without dark/transparent seams around image edges.

Testing: on Windows with `gpui_windows`, build `cargo check -p gpui_windows` (compiles `shaders.hlsl`) and visually inspect polychrome images packed next to distinct neighboring atlas tiles at 125%/150% display scaling.

Release Notes:

- N/A

---

HeroGPUI follow-up once released: remove the corresponding local `shaders.hlsl` workaround in downstream Windows forks (`gpui-pre-windows`).
