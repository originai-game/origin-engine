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

## Placement and current limits

The anchor clamps to the screen's edge (`clamp`, marking the side as
`data-edge-*`), stacks or hides overlapping anchors (`declutter`,
`priority`) and fades or hides them behind drawn geometry (`occlusion`).
Host props `scaleAt`, `scaleMin`, `scaleMax` opt into distance scale;
`maxDistance` limits range and `lod={[near, far]}` sets
`data-lod-near|mid|far`. These capabilities do not automatically change
every component: use the props each component actually forwards, or
compose an anchored `View` directly.

The current host model has no `contain` clamp, clamp-angle output,
HUD/player/danger reservations, bounded loot-stack solver or mass-bar
template. Where a component accepts `range`, `edge`, `compact` or `more`,
the game still supplies that presentation data. A clamped bubble's tail
is not automatically aimed at the target; an unclamped label can extend
past an edge; a prompt's priority does not reserve a telegraph. Loot's
overflow count is game-provided. Overhead bars are TSX rows, not the old
GPU mass-anchor markup. Do not copy document-era anchor attributes or CSS
custom properties into these props. Ground effects remain decals/lines.

## Gallery

`bun packages/engine/src/node/gallery.ts --only world/` draws every member's fixtures under every pack.
