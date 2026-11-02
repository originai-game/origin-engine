# Concepts

Targets for the art bar (docs/ui.md §15.5), made 2026-11-02 by the art
review with Painter, each from the current frame as its content reference.
Directions: A dark fantasy ARPG, B clean sci-fi, C bright stylised anime.
Screens: hud, menu, gacha, shop, inventory, settings, dialogue.

## Approved (owner, 2026-11-02)

| Direction | Becomes | Approved for |
| --- | --- | --- |
| B, clean sci-fi | the engine's default style and the UI kit's own frames | every screen: `B-hud`, `B-menu`, `B-gacha`, `B-shop`, `B-inventory`, `B-settings`, `B-dialogue` |
| A, dark fantasy | the fantasy seed style (`templates/styles/fantasy`) | every screen: `A-hud` … `A-dialogue` |
| C, bright stylised anime | the anime seed style | every screen: `C-hud` … `C-dialogue` |

- The kit's art stays painted (`templates/ui-kit/assets/textures/art`,
  docs/ui.md §15.5.3), never line icons or flat vector stand-ins.
- The showcase frames (the gallery, the store and docs images) wear the
  seed styles, with the default style as one column beside them.
- B sets the default's hierarchy, material, frames and art; its colour
  values are the design language's (`packages/design/tokens.json`), so its
  accent is the tokens' primary, not the concept's cyan.

A frame is read side by side with the approved concept of its style's
direction and screen (docs/ui.md §15.5). A screen with no concept of its
own is read against the nearest one; the genre sheets below extend the
set.
