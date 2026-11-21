# nsh-jsx

From NSH (https://github.com/longbridge/nsh), commit
`6fa6368f46e3b204100b37beefce5985886515db`, Apache-2.0 (`LICENSE-APACHE`).
`src/` holds nsh-bundler's `jsx.rs`, `compile.rs`, `props.rs`, `hot.rs` and
`classes.rs`: the JSX compiler (TSX to Solid-style renderer calls, one oxc
pass) and the build-time class check. The first commit adding this
directory has them unchanged; the changes after it are only these:

- `src/lib.rs` (new): `compile` (a module to JS and its source map) and
  `unknown_classes` (the class check against a theme).
- oxc 0.153 to 0.157 (the workspace's pin): a string literal's value is a
  `JSStr`, read with `display()`/`as_str()`; `oxc_sourcemap` 9.
- The primitives are Origin Engine's (`crates/og-ui-next/CONVENTIONS.md`
  §Components): `View`, `Text`, `Image`, `Icon`, `Scroll`, `Input`, `Svg`
  host elements and the control flow `Show` (`when`, `fallback`, `keyed`,
  compiled to `cond`) and `For` (`each`, `key`, compiled to `each`), all
  imported from `@origingame/engine/ui`. NSH's global `If`/`Each`, its
  `List`, `Canvas`, `Dialog` and the other platform tags are gone, and
  compiled code imports the renderer from `@origingame/engine/ui/rt`.
- `compile.rs`'s `NativeModules` keeps only `exports` and `components`,
  empty: NSH filled them from its own module registries; Bun reports a
  missing export.
- The tests are NSH's, with these imports and tags; the ones for NSH-only
  primitives (`every_host_element_has_one_home`, `Canvas`, `WebView`,
  `nsh/base`) are replaced by the same checks over the engine's, and the
  class check's theme test reads a pack theme instead of `nsh_base`'s.
