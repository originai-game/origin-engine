---
name: building-photo-ui
description: Builds photo UI with the bundled kit. Use when adding or editing photo screens and HUDs.
---

# Building photo UI

Run `og ui list` to discover documents and `og ui add photo` to install an editable copy.
Import `{ photo }` from the installed `assets/ui-kits/photo/index.ts`, or from
`@origingame/engine/extensions/photo/index.ts` for the bundled version.
The namespace exposes `photoMode`.
Read `../types.ts` for their inputs and results. The game owns state and applies results.
Use `og ui eject photo/<document>` to copy a screen, then fill its slots from code.
Modify the installed components through the public engine API; new components register
with `registerUiExtension` and never add members to `og.ui`.
Keep a game's own style and test desktop, phone, keyboard, touch and increased text size.
