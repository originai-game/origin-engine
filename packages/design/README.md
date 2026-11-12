# @origingame/design

The design language Origin Engine and origin-agent share, apart from the
platform's: [`design-language.md`](design-language.md) and its values,
[`tokens.json`](tokens.json).

- **The engine** generates og-ui's default style from `tokens.json`
  (`crates/og-ui/scripts/gen-theme.sh` writes `crates/og-ui-core/src/theme.rs`;
  og-ui's `tests/theme.rs` fails on drift). The default style is
  origin-agent's look; a game may ship it, a built-in pack or a style of
  its own.
- **origin-agent** pins this package by version, as it pins the engine.

```ts
import tokens from "@origingame/design"; // tokens.json
```

`bun test` holds the language's pairs: text at 4.5:1 and focus at 3:1 in
both themes, text over glass over the brightest and darkest scenes, and the
`linear` fills against their CSS values.
