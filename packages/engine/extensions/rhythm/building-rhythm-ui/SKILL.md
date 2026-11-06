---
name: building-rhythm-ui
description: Builds rhythm UI with the bundled kit. Use when adding or editing rhythm screens and HUDs.
---

# Building rhythm UI

Run `og ui list` to discover documents and `og ui add rhythm` to install an editable copy.
Import `{ rhythm }` from the installed `assets/ui-kits/rhythm/index.ts`, or from
`@origingame/engine/extensions/rhythm/index.ts` for the bundled version.
The namespace exposes `songSelect`, `laneKeys`, `comboCounter`, `judgement`.
Read `../types.ts` for their inputs and results. The game owns state and applies results.
Use `og ui eject rhythm/<document>` to copy a screen, then fill its slots from code.
Modify the installed components through the public engine API; new components register
with `registerUiExtension` and never add members to `og.ui`.
Keep a game's own style and test desktop, phone, keyboard, touch and increased text size.
