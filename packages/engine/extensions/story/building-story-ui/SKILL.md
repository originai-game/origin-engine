---
name: building-story-ui
description: Builds story UI with the bundled kit. Use when adding or editing story screens and HUDs.
---

# Building story UI

Run `og ui list` to discover documents and `og ui add story` to install an editable copy.
Import `{ story }` from the installed `assets/ui-kits/story/index.ts`, or from
`@origingame/engine/extensions/story/index.ts` for the bundled version.
The namespace exposes `backlog`, `barks`, `cgGallery`, `routeChart`.
Read `../types.ts` for their inputs and results. The game owns state and applies results.
Use `og ui eject story/<document>` to copy a screen, then fill its slots from code.
Modify the installed components through the public engine API; new components register
with `registerUiExtension` and never add members to `og.ui`.
Keep a game's own style and test desktop, phone, keyboard, touch and increased text size.
