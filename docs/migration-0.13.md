# Migrating from 0.12 to 0.13

This guide covers only the changes that can break a 0.12 build or change
what a callback receives. The full list, including additions, is in
[`CHANGELOG.md`](../CHANGELOG.md#unreleased).

Update the dependency first:

```toml
herogpui = "0.13"
```

## `VirtualListScroll` is `#[non_exhaustive]`

It gained `Center` (GPUI's `ScrollStrategy::Center`, which the fixed-row
collections use for their keyboard cursor), and further strategies may
follow, so an exhaustive `match` outside HeroGPUI no longer compiles.

```rust
// 0.12
match strategy {
    VirtualListScroll::Reveal => { /* ... */ }
    VirtualListScroll::Top => { /* ... */ }
}

// 0.13
match strategy {
    VirtualListScroll::Reveal => { /* ... */ }
    VirtualListScroll::Top => { /* ... */ }
    _ => { /* Center, and anything added later */ }
}
```
