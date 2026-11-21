# world: built-in components

The labels a game anchors over its scene (nameplates, overhead bars, loot,
interaction prompts, speech, waypoints), the compass and the minimaps. A
label sits on an entity or a world point through the `anchor` prop. They
draw what the game tells them: every figure, state, timing and decision
(reaction, health, cast progress, what is near or far, when a bubble times
out) is the game's. Marks on the ground (selection rings, telegraphs,
placement ghosts, paths) are not UI: they are og.Decal and og.render.lines
(the effects guide).

A game imports one by name from `@origingame/engine/ui/components`, with
its props as typed in its `.tsx` (`<Name>Props`, each prop documented
there); `fixtures` in the same file are its cases. How components are built
and dressed is `../README.md`.

## Members

| Component | File | What it is |
| --- | --- | --- |
| `Compass` | `world/compass` | A strip of the horizon centred on the heading. |
| `InteractionPrompt` | `world/interaction-prompt` | What the player can do with what is in front of them. |
| `LootLabel` | `world/loot-label` | An item on the ground. |
| `MinimapView` | `world/minimap-view` | A minimap. |
| `Minimap` | `world/minimap` | The map of the place, its markers on it and a zone name under. |
| `Nameplate` | `world/nameplate` | A unit's plate over its head. |
| `OverheadBars` | `world/overhead-bars` | A health bar over every unit in the side's colour, a thin resource line under it. |
| `SpeechBubble` | `world/speech-bubble` | A line spoken over a head. |
| `Waypoint` | `world/waypoint` | A place to go. |

## Waiting on the placement

The anchor clamps to the screen's edge (`clamp`, marking the side as
`data-edge-*`), stacks or hides overlapping anchors (`declutter`,
`priority`) and fades or hides them behind drawn geometry (`occlusion`).
It does not yet tell a component its distance scale, its clamp angle or
its LOD, keep an on-screen label inside the screen (`contain`), reserve
rects or danger areas, bound a loot stack, lay mass
bars apart from the labels or carry a shield track on a mass bar (UI Sol's
placement items). Until it does the game passes `range`, `edge`,
`compact` and `more`; a clamped bubble's tail points down; a label whose
point lies near the screen's edge can run past it; a prompt keeps off a
plate by priority but not off a telegraph; loot folds into a `+n` chip
the game counts; overhead bars draw no shield track and can cross a label.

## Gallery

`bun packages/engine/src/node/gallery.ts --only world/` draws every member's fixtures under every pack.
