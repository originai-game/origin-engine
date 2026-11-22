<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `interaction` guide: Interaction: focus, use, hold, doors, switches and carrying

How og.systems.interact picks what the player can use, uses it on a press or a hold, opens doors with their navmesh links, flips levers and pressure plates, and picks things up to throw. Read before building an adventure's "press E", a puzzle's switches or a physics game's carrying. The candidates themselves are og.play.candidates; buffered presses are in `input`; sensors in `physics` §5; navmesh links in `og api show nav`.

The API: `og api show interact`, `og api show play`.

<a id="guide-interaction-1"></a>
### `interaction §1` Focus and use

List the systems in the game:

```ts
og.game({ systems: [...og.systems.interact, ...mySystems] });
```

- An **og.Interactable** is anything that can be used, at its entity's
  position. Its `holdTicks` makes it a hold (0 is a press), `priority`
  ranks it under the priority rule and `disabled` hides it.
- An **og.Interactor** looks from its og.Transform plus `eye`, along its
  forward (+Z, as `facing` states), `range` metres and within `cone` of
  it. With `sight`, a candidate behind a collider of `sightMask` is
  skipped (the interactor's own body and the target's are never in the
  way). It focuses one candidate by `rule`:
  - `nearest`: the nearest, equal distances in entity order;
  - `centred`: the smallest angle off its forward (what the camera points
    at), then the nearest;
  - `priority`: the highest priority (a quest item over a barrel), then
    the nearest.
- `focus` is written every tick: show the prompt
  (`world/interaction-prompt`) for it. A focus that changes cancels a hold.
- A press of `action` (og.input.consume, so a press just before something
  comes into focus waits in the action's buffer) uses a press interactable
  at once. A hold interactable is used once the action has been down on it
  for `holdTicks`, the press tick counting as the first; `progress` counts
  the ticks for a ring, and letting go or looking away sets it back to 0
  with nothing used.
- Using one sends og.Interacted; then a door, toggle or grab on it acts
  (§2). Anything else is the game's: read og.Interacted in a system.

Typical numbers: a range of 1.5 to 2.5 m for a first-person game, a cone
of 0.5 to 0.8 rad with the centred rule; 20 to 60 hold ticks at 60 Hz
for a deliberate act. These are starting points to tune by feel.

<a id="guide-interaction-2"></a>
### `interaction §2` Doors, toggles and carrying

- An **og.Door** flips `open` when used, or follows an og.Toggle's `on`
  when it names one. With `mesh` and `link` (an off-mesh link of kind
  `door`, og.nav.setLink) the link is opened and closed with it, so agents
  path through only while it is open; the first run sets the link to the
  door's state. Moving its leaf (a tween, a hinge) is the game's, read
  from `open`.
- An **og.Toggle** flips `on` when used (a lever, a button). A `plate` is on
  exactly while its entity's sensor collider holds anything
  (og.physics.sensorContents) and ignores use: a pressure plate a crate
  holds down.
- An **og.Grab** on a dynamic og.RigidBody is picked up when used: kinematic
  while carried, moved each tick to `carry` in the carrier's frame, and
  `heldBy` names the carrier (og.Interactor.holding names it back). The
  carrier focuses nothing while it carries; its next press throws it,
  dynamic again, along the carrier's forward at `throwSpeed` (0 drops it).
  Carried bodies are ordinary world state: a save made while carrying
  restores the carry.

To change a rule (a door that needs a key, a throw that arcs up), copy
`systems/interact.ts` from the engine package into the game and list the
copy in GameDesc.systems instead of og.systems.interact.
