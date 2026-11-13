# @origingame/design

The design language Origin Engine and origin-agent share, apart from the
platform's: [`design-language.md`](design-language.md) and its values,
[`tokens/`](tokens): CSS files of custom properties on `:root`, one per
surface (`color.css`, `type.css`, `space.css`, `effect.css`, `motion.css`,
`state.css`, `density.css`, `role.css`, `derived.css`, `contrast.css`). A token is
named by its path (`--motion-duration-fast`); a `:root[data-scheme=dark]`
block holds what the dark scheme changes. Edit a value there, then run
`crates/og-ui/scripts/gen-theme.sh`.

- **The engine** generates og-ui's default style from `tokens/*.css`
  (`crates/og-ui/scripts/gen-theme.sh` writes `crates/og-ui/src/theme.rs`,
  `crates/og-ui/default.css` and [`tokens.json`](tokens.json), the same
  tokens as one tree;
  og-ui's `tests/theme.rs` fails on drift). The default style is
  origin-agent's look; a game may ship it, a built-in pack or a style of
  its own.
- **origin-agent** pins this package by version, as it pins the engine.

```ts
import tokens from "@origingame/design"; // tokens.json, generated
import "@origingame/design/tokens/color.css"; // or the CSS itself
```

`bun test` holds the language's pairs: text at 4.5:1 and focus at 3:1 in
both themes, text over glass over the brightest and darkest scenes, and the
`linear` fills against their CSS values.
