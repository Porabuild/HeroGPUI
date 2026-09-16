# Retired GPUI deviations (not wired into any build)

These five patches record what HeroGPUI used to carry as `[patch.crates-io]`
path overrides against `gpui-pre` 0.3.3 (`zed@5b055fa`), before the workspace
moved back to vanilla registry GPUI so that `cargo add herogpui` just works:

- `gpui-pre-0.3.3.patch` — rounded `ClipRegion`/`RoundedClip` scene nodes plus
  the `aria_current` accessibility builder.
- `gpui-pre-wgpu/apple/windows-0.3.3.patch` — the clip-node carry-through for
  each native renderer.
- `gpui-pre-web-0.3.3.patch` — shift+wheel horizontal scroll, IME mirror
  resync after paste, and the `default = []` feature deviation.

Verified still missing from vanilla `gpui-pre` 0.3.5 (`zed@d89e9c2`), so a
version bump alone did not obsolete them. They are kept here for one purpose:
as source material for the upstream PRs to `zed-industries/zed`
(`crates/gpui*` / `crates/gpui_web`). The component-level replacements that
let the workspace drop the forks live in `crates/herogpui-components/src/`:

- `a11y.rs` — `aria-current="page"` is a documented omission on vanilla.
- `util.rs` (`shift_wheel_as_horizontal`, `inner_fill_radius`) — web wheel
  handling and rounded-overflow mitigation without renderer support.

Do not point `[patch.crates-io]` back at these: the 0.3.3 hunks are stale
against 0.3.5, and re-wiring them would reintroduce the materialization step
for every downstream user. If an upstream PR lands, delete the corresponding
deviation here; when all land, delete this directory.
