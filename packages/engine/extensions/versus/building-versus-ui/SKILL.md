---
name: building-versus-ui
description: Builds versus UI with the bundled kit. Use when adding or editing versus screens and HUDs.
---

# Building versus UI

Run `og ui list` to discover documents and `og ui add versus` to install an editable copy.
Import `{ versus }` from the installed `assets/ui-kits/versus/index.ts`, or from
`@origingame/engine/extensions/versus/index.ts` for the bundled version.
The namespace exposes `lobby`, `characterSelect`, `versus`, `draft`, `chat`.
Read `../types.ts` for their inputs and results. The game owns state and applies results.
Use `og ui eject versus/versus/<document>` to copy a screen, then fill its slots from code.
Modify the installed components through the public engine API; new components register
with `registerUiExtension` and never add members to `og.ui`.
Keep a game's own style and test desktop, phone, keyboard, touch and increased text size.
