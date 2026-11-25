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

- `style.rs` adds the `env()` variables `hud-inset-top|right|bottom|left`
  (the HUD's safe area, which og-ui-next adds at layout) and
  `Style::env_padding`, which reads the padding's `env()` terms.

- `style.rs` and `tailwind/utilities.rs` add the text outline:
  `TextStyle::stroke_width` and `stroke_color`, the properties
  `webkitTextStrokeWidth` and `webkitTextStrokeColor` (CSS's
  `-webkit-text-stroke-width` and `-color`), and the classes
  `text-stroke-<number>`, `text-stroke-[<length>]` and
  `text-stroke-<color>`.

- `tailwind/mod.rs` adds `ClassList::apply_data`, `apply` with the
  node's `data-*` attributes (the `Data` predicate `apply_with` takes), so
  og-ui-next's tree resolves `data-<name>:` and `group-data-<name>:`.

- `tailwind/utilities.rs`, `style.rs` and `css.rs` (each marked in place):
  `flex-1`, `flex: <number>` and a `flex` shorthand without a basis take
  the basis `0%`, as Tailwind's `flex: 1` and browsers' expansion of the
  shorthand do (it was 0 px, React Native's): against a container of no
  definite size it is the item's content. An explicit unitless `0` basis
  after two factors stays 0 px. `style.rs`'s `flex_shorthand` and
  `flex_and_grid_longhands_override_their_shorthands_in_any_order` expect
  the `0%`.

- `tailwind/utilities.rs` adds `grid-cols-fill-<steps>`, `grid-cols-fit-<steps>`
  and the same for `grid-rows` (`grid_template`, marked in place):
  `repeat(auto-fill | auto-fit, minmax(<steps>, 1fr))` with the length on
  the spacing scale.

- `tailwind/theme.rs` adds `Theme::rem_lengths` (and `tailwind/utilities.rs`
  makes its `rem_lengths` crate-visible): a style prop's value with its
  `rem` lengths in px at the theme's rem, which og-ui-next applies before
  `Style::set`, so a style prop and a bracketed class read the same rem.

- `style.rs` makes `apply_direction` public: og-ui-next has no NSH core to
  place a style's start and end sides (`ps-*`, `me-*`, `rounded-s-*`), so
  it calls it on each record's style with the page's direction.

- `style.rs` makes `Style::node_props` public, for og-ui-next to read what
  a style sets of a node's own props (`line-clamp-*`, `pointer-events-*`,
  `text-ellipsis`, `select-*`) without NSH's core.

- `css.rs` adds `calc()`, which parses `calc()`, `min()`, `max()` and
  `clamp()` over lengths and percentages with `+ - * /`; `style.rs`'s `Len`
  takes it (`Len::Calc`, a `rune_taffy::calc` handle), so an inline
  `left: calc(50% - 2px)` resolves at layout instead of being rejected.

- `tailwind/utilities.rs` and `tailwind/mod.rs` add CSS `overflow-wrap`
  (each marked in place): `break-normal` / `wrap-normal`, `break-words` /
  `wrap-break-word` and `wrap-anywhere` set `ClassProps::overflow_wrap`
  (0, 1, 2), which og-ui-next hands to the text's line breaking.

- `tailwind/utilities.rs` and `tailwind/mod.rs` add CSS `writing-mode`:
  `writing-vertical-rl` and `writing-horizontal-tb` set
  `ClassProps::vertical`, which og-ui-next lays out as columns.

- `Cargo.toml` allows `dead_code` and `unused_imports`: the subset kept
  here leaves items nsh-core's other modules call unused.

Its own tests (`tailwind/tests.rs`, `geometry/tests.rs`, `style.rs`) run
unchanged otherwise.
