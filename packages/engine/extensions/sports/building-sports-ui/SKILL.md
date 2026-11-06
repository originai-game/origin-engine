---
name: building-sports-ui
description: Builds sports UI with the bundled kit. Use when adding or editing sports screens and HUDs.
---

# Building sports UI

Run `og ui list` to discover documents and `og ui add sports` to install an editable copy.
Import `{ sports }` from the installed `assets/ui-kits/sports/index.ts`, or from
`@origingame/engine/extensions/sports/index.ts` for the bundled version.
The namespace exposes `scorebug`, `matchStats`, `goalTimeline`, `playerPlate`, `substitution`.
Read `../types.ts` for their inputs and results. The game owns state and applies results.
Use `og ui eject sports/<document>` to copy a screen, then fill its slots from code.
Modify the installed components through the public engine API; new components register
with `registerUiExtension` and never add members to `og.ui`.
Keep a game's own style and test desktop, phone, keyboard, touch and increased text size.
