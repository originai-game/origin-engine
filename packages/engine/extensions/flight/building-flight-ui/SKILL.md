---
name: building-flight-ui
description: Builds flight UI with the bundled kit. Use when adding or editing flight screens and HUDs.
---

# Building flight UI

Run `og ui list` to discover documents and `og ui add flight` to install an editable copy.
Import `{ flight }` from the installed `assets/ui-kits/flight/index.ts`, or from
`@origingame/engine/extensions/flight/index.ts` for the bundled version.
The namespace exposes `flightHud`.
Read `../types.ts` for their inputs and results. The game owns state and applies results.
Use `og ui eject flight/<document>` to copy a screen, then fill its slots from code.
Modify the installed components through the public engine API; new components register
with `registerUiExtension` and never add members to `og.ui`.
Keep a game's own style and test desktop, phone, keyboard, touch and increased text size.
