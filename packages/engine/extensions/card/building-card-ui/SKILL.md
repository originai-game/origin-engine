---
name: building-card-ui
description: Builds card UI with the bundled kit. Use when adding or editing card screens and HUDs.
---

# Building card UI

Run `og ui list` to discover documents and `og ui add card` to install an editable copy.
Import `{ card }` from the installed `assets/ui-kits/card/index.ts`, or from
`@origingame/engine/extensions/card/index.ts` for the bundled version.
The namespace exposes `cardHand`, `rewardChoice`, `deckBuilder`.
Read `../types.ts` for their inputs and results. The game owns state and applies results.
Use `og ui eject card/<document>` to copy a screen, then fill its slots from code.
Modify the installed components through the public engine API; new components register
with `registerUiExtension` and never add members to `og.ui`.
Keep a game's own style and test desktop, phone, keyboard, touch and increased text size.
