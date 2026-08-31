# Audit and gallery-tool instructions

These rules apply under `.shots` in addition to the root guide.

Read [parity and audits](../docs/agents/parity.md) before changing an audit, and
[gallery and visual verification](../docs/agents/gallery.md) before changing or
running a driver.

- Audit readers fail loudly when expected input cannot be found. Missing input
  is not an empty passing result.
- Anchor extraction to real structural boundaries, preserve row/part ownership,
  and report unreadable inputs separately from legitimate absence.
- When changing a parser or evidence map, demonstrate a known-negative failure
  as well as the passing repository result.
- Keep tagged inputs pinned to HeroUI v3.2.4 and inherited behavior evidence
  pinned to the versions in the root parity guide.
- Drivers must capture the GPUI window with `PrintWindow`, never the user's
  screen. Prefer off-screen posted input; foreground capture is opt-in because it
  interrupts the user.
- Do not copy route, prop, metric, or coverage totals into documentation. Let the
  current scripts report them.

After an audit change, run the changed script first, then every `*audit.py` when
shared parsing or a broad parity claim is affected. After a driver change, test
one focused page and verify the resulting image or state manually.
