---
name: building-idle-ui
description: Builds idle UI with the bundled kit. Use when adding or editing idle screens and HUDs.
---

# Building idle UI

Run `og ui list` to discover documents and `og ui add idle` to install an editable copy.
Import `{ idle }` from the installed `assets/ui-kits/idle/index.ts`, or from
`@origingame/engine/extensions/idle/index.ts` for the bundled version.
The namespace exposes `idleGenerator`, `prestigePanel`.
Read `../types.ts` for their inputs and results. The game owns state and applies results.
Use `og ui eject idle/<document>` to copy a screen, then fill its slots from code.
Modify the installed components through the public engine API; new components register
with `registerUiExtension` and never add members to `og.ui`.
Keep a game's own style and test desktop, phone, keyboard, touch and increased text size.
