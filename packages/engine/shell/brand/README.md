# Vendored brand (do not edit)

The platform's brand package (`origingame/origingame`, `packages/brand`) is
the source of every file here; the engine introduces no second mark, palette
or motion (design §9, AGENTS.md "Brand is one ink"). Copied at platform
revision `ef1555b83`:

| File | Source |
|---|---|
| `origin-mark.svg` | `packages/brand/motion/origin-mark.svg` (generated there from `source/origin-ai.svg`) |
| `origin-motion.css` | `packages/brand/motion/origin-motion.css` |
| `mark-on-paper.svg` | `packages/brand/app-icon/mark-on-paper.svg` (the app-icon tile) |
| `icon-192.png`, `icon-512.png` | `mark-on-paper.svg` rendered with `rsvg-convert -w N -h N` (the PWA's fallback icons) |
| `tokens.css` | the dark-theme values the shell reads, from `packages/tokens/dist/{tokens,theme-dark}.css` |

To refresh after a brand change, copy the files again from the platform
checkout, re-render the two PNGs, update the revision above and rebuild.
`og build` derives the shell's marks from `origin-mark.svg` (`genesis` for
the boot splash, `progress` with the real load value, `drift` for the error
card; the `engine` product mark for "Made with Origin Engine" is the
symbol's inner arms and core ring, as the platform's `product-marks.ts`
composes it) and inlines only the rules of those motions
(`shell/brand.ts`).
