# world: built-in components

The labels a game anchors over its scene (nameplates, overhead bars, loot,
interaction prompts, speech, waypoints) and the marks it lays on its ground
(telegraphs, selection rings, placement ghosts, paths), and the compass and
minimaps. They draw what the game tells them: every figure, state, timing
and decision (reaction, health, cast progress, what is near or far, when a
bubble times out, what can be built) is the game's.

They are built into the engine: a game's page uses one by name
(`<Nameplate entity={e.bits} name={$name} ... />`) and loading the page
loads it. `og ui eject world/nameplate` copies one into `assets/ui/world/`.

## Structure and appearance

Labels are `<anchor>`s, ground marks `<ground>`s and overhead bars GPU
`mass` bars (docs/ui.md §2.1). A ground's box takes its content's size
(the world `size` maps that box), and its badge (a placement's cost, a
path's length) stands upright in an `<anchor>` beside it, so it reads at
any tilt. Each member's `<style>` holds layout only:
the default style's smoked glass and lit edges come from `--hud-*` and
`--world-*` role tokens (reaction inks and fills, the nameplate's ghost,
rims and bar sizes, cast states, loot tiers, the prompt's hold ring,
speech, waypoint pins, telegraph, ring, placement and path strokes).
Text over the scene takes one stroke under the fill and one soft shadow,
never stacked offset shadows. Type is in font steps, so it follows the
player's text scale; bars, pins, rings and footprints are art and keep
their size. Short words and figures (a verb, a level, a distance, a
percent) fit whole; names, titles, spells and objects stay on one line
and scroll (`-og-scroll`) when they overflow. Every loop and shake has a
still state under reduced motion.

Placement uses only what `<anchor>` offers: `priority` (a targeted plate
1100, a waypoint 500, a boss or casting plate 400, a prompt 300, speech
200, a plate 100, loot 10), `declutter`, `clamp="edge"` (waypoints),
`max-distance`. Hooks for a pack: `data-reaction`, `data-elite`,
`data-boss`, `data-targeted`, `data-low`, `data-dead`, `data-compact` on
the plate and `data-state` (casting, locked, interrupted) on its cast;
`data-side` on bars and rings; `data-rarity` and `data-hovered` on loot;
`data-pressed`, `data-done`, `data-disabled`, `data-pad` on a prompt;
`data-reaction` and `data-shout` on speech; `data-kind`, `data-range` and
`data-way` on a waypoint; `data-shape`, `data-friendly`, `data-hit` on a
telegraph; `data-valid` and a cell's `data-blocked` on a ghost;
`data-invalid` on a path.

## Members

| Document | Inputs (props) | Intents | Parts |
| --- | --- | --- | --- |
| `Nameplate` (`nameplate.svelte`) | `entity`, `at`, `height`, `name`, `title`, `level`, `reaction` (neutral, friendly, hostile, player), `health` (negative: none), `shield`, `castName`, `castProgress`, `castTime`, `castState` (casting, locked, interrupted), `icon`, `elite`, `boss`, `targeted`, `dead`, `lootable`, `compact`, `maxDistance` | none | `frame`, `level`, `icon`, `name`, `subtitle`, `state`, `health`, `ghost`, `fill`, `shield`, `percent`, `wing`, `bracket`, `cast`, `spell`, `time`, `castbar` |
| `OverheadBars` (`overhead-bars.svelte`) | `bars` keyed `{ id, entity, at, value, max, side (own, ally, enemy, neutral), resource, resourceMax }`, `height` | none | `frame`, `bar`, `resource` |
| `LootLabel` (`loot-label.svelte`) | `entity`, `at`, `height`, `name`, `count`, `rarity`, `hovered`, `action`, `more`, `tiers` keyed `{ id, rarity }` | `pick` | `frame`, `glow`, `chip`, `strip`, `name`, `count`, `key`, `tier`, `beam` |
| `InteractionPrompt` (`interaction-prompt.svelte`) | `entity`, `at`, `height`, `action`, `key`, `pad`, `verb`, `name`, `pressed`, `hold` (negative: none), `disabled`, `reason`, `maxDistance` | none | `frame`, `cap`, `key`, `ring`, `plate`, `verb`, `object`, `reason` |
| `SpeechBubble` (`speech-bubble.svelte`) | `entity`, `at`, `height`, `speaker`, `reaction`, `text`, `shout`, `left` (negative: no timer) | none | `frame`, `box`, `speaker`, `line`, `timer`, `tail` |
| `Telegraph` (`telegraph.svelte`) | `entity`, `at`, `shape` (circle, cone, line, ring), `radius`, `inner`, `angle`, `width`, `direction`, `progress`, `friendly`, `hit` | none | `area`, `wash`, `sweep`, `rim` |
| `SelectionRing` (`selection-ring.svelte`) | `entity`, `at`, `radius`, `side`, `leader`, `target`, `hover` | none | `ring`, `glow`, `line`, `tick` |
| `PlacementGhost` (`placement-ghost.svelte`) | `at`, `width`, `depth`, `rotation`, `columns`, `cells` keyed `{ id, blocked }`, `valid`, `label`, `range` | none | `reach`, `footprint`, `cell`, `badge` |
| `Waypoint` (`waypoint.svelte`) | `entity`, `at`, `height`, `kind` (objective, ping, danger, ally, loot), `label`, `distance`, `units`, `range` (near, mid, far), `edge` (left, right) | none | `frame`, `pin`, `name`, `distance`, `chevron` |
| `PathLine` (`path-line.svelte`) | `at`, `size`, `path`, `reach`, `stops` keyed `{ id, x, y }`, `endX`, `endY`, `label`, `invalid` | none | `line`, `under`, `walk`, `past`, `stop`, `end`, `badge` |

`Compass`, `Minimap` and `MinimapView` keep their own documents.

Every member carries a `fixture:default` and its states: nameplate
friendly, neutral, hostile, player, boss, targeted, low, casting, locked,
interrupted, dead, compact, overflow; bars sides, low, empty; loot common,
stack, uncommon, rare, legendary, hovered, more, overflow; prompt pressed,
hold, done, disabled, gamepad, action, overflow; speech short, shout,
nameless, overflow; telegraph cone, line, ring, impact, ally; ring leader,
ally, target, neutral, hover; ghost blocked, partial; waypoint near, far,
ping, danger, ally, loot, offscreen, offscreenRight, overflow; path
invalid.

## Waiting on the placement

The anchor does not yet tell a document its distance scale, its clamp
side or angle or its LOD, keep a label inside the screen (`contain`, a
clamp margin), reserve rects or danger areas, bound a loot stack, lay mass
bars apart from the labels or carry a shield track on a mass bar (UI Sol's
placement items). Until it does the game passes `range`, `edge`,
`compact` and `more`; a clamped bubble's tail points down; a label whose
point lies near the screen's edge can run past it; a prompt keeps off a
plate by priority but not off a telegraph; loot folds into a `+n` chip
the game counts; overhead bars draw no shield track and can cross a label.

## Review sheet and tests

`og ui sheet world` renders `packages/engine/sheets/world/`: `scene` (the
third-person mockup scene), `topdown` (the strategy scene), `states`
(the labels in every fixture state) and `marks` (the ground marks,
bubbles and bars in every state); on a phone `states` and `marks` show a
choice of states that fits the screen.
`bun test tests/ui-world.test.ts` mounts every member from every fixture.
