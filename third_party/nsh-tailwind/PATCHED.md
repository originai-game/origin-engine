# nsh-tailwind

From NSH (https://github.com/longbridge/nsh), commit
`6fa6368f46e3b204100b37beefce5985886515db`, Apache-2.0 (`LICENSE-APACHE`).
`src/` holds nsh-core's `tailwind/`, `css.rs`, `css_named.rs`, `grid.rs`,
`color.rs`, `effects.rs`, `geometry/`, `transform.rs`, `style.rs`,
`element/tokens.rs` and `animation/ops.rs`: the class resolver and the value
types it writes. The first commit adding this directory has them unchanged;
the changes after it are only these:

- `src/lib.rs` (new): declares the modules at nsh-core's paths and
  re-exports at the crate root what nsh-core's `lib.rs` does for them.
- `src/shim.rs` (new): nsh-core definitions the modules name from outside
  this set, copied unchanged with their source paths: `Spring` (with its
  impls and the `SETTLE`/`CRITICAL` constants), `ResizeMode`,
  `AdaptiveColor` (with its impls), `AnimationValue`, `Reactive`,
  `IntoReactive` and `PointerEvents`.
- `#[derive(PropEnum)]` and its `#[prop(...)]` attributes are removed
  (`effects.rs`, `geometry/shapes.rs`, `style.rs`, `shim.rs`): the derive is
  nsh-macros' JavaScript binding, which nothing here calls.
- `geometry/unit_scan.rs` and `geometry/unit_types.rs` are left out with
  their `mod` lines: NSH's lint over its own source tree (it needs `syn`).
- `style.rs`'s font-family test drops its `CanvasFont` round trip (canvas is
  not part of this set).

- `tailwind/mod.rs` adds Origin Engine's variants, each marked in place:
  `narrow:` (the UI's box under 600 wide), `short:` (under 500 tall) and
  `compact:` (either), read from `Context.window`, and the states
  `starting:` and `exit:` (bits `STARTING` and `EXIT`) that the motion
  system sets.
- Motion (`effects.rs`, `tailwind/utilities.rs`, each marked in place):
  `TransitionProps::LAYOUT` and the class `transition-layout`, which adds
  to the group named before or after it and is not in `ALL`; the keyframes
  `Pop`, `Stamp`, `Beat`, `Float` and `Shimmer` with their classes
  `animate-pop` … `animate-shimmer` and their timings in `preset`.

Its own tests (`tailwind/tests.rs`, `geometry/tests.rs`, `style.rs`) run
unchanged otherwise.
