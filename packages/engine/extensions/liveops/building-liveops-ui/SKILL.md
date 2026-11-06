---
name: building-liveops-ui
description: Builds liveops UI with the bundled kit. Use when adding or editing liveops screens and HUDs.
---

# Building liveops UI

Run `og ui list` to discover documents and `og ui add liveops` to install an editable copy.
Import `{ liveops }` from the installed `assets/ui-kits/liveops/index.ts`, or from
`@origingame/engine/extensions/liveops/index.ts` for the bundled version.
The namespace exposes `storeFront`, `battlePass`.
Read `../types.ts` for their inputs and results. The game owns state and applies results.
Use `og ui eject liveops/<document>` to copy a screen, then fill its slots from code.
Modify the installed components through the public engine API; new components register
with `registerUiExtension` and never add members to `og.ui`.
Keep a game's own style and test desktop, phone, keyboard, touch and increased text size.
