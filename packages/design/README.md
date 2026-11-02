# @origingame/design

The design language Origin Engine and origin-agent share, apart from the
platform's (the owner, 2026-11-02): [`design-language.md`](design-language.md)
and its values, [`tokens.json`](tokens.json). It was seeded from origingame's
`docs/design-language.md` and `packages/tokens/tokens.json` at d25f1f15
(2026-11-02) and evolves here.

- **The engine** generates og-ui's default style from `tokens.json`
  (`crates/og-ui/scripts/gen-theme.sh` writes `crates/og-ui/src/theme.rs`;
  og-ui's `tests/theme.rs` fails on drift). The default style is for
  origin-agent's surfaces and for prototypes: every game ships its own style,
  and `og build --publish` refuses one still in the default.
- **origin-agent** pins this package by version, as it pins the engine.

```ts
import tokens from "@origingame/design"; // tokens.json
```

`bun test` holds the language's pairs: text at 4.5:1 and focus at 3:1 in
both themes, text over glass over the brightest and darkest scenes, and the
`linear` fills against their CSS values.
