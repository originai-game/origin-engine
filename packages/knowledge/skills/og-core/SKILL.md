---
name: og-core
description: The rules every Origin Engine game obeys — TypeScript only, a fixed deterministic tick, Results instead of throws, typed views per archetype, no wall clock, and reflection as the one schema. Read before writing or reviewing any code in a project with a project.ogt.
---

Origin Engine is a Rust engine with a TypeScript API, generated from one IDL
(`crates/og-api/api.ogt` in the engine repository). The same game bundle runs
in the browser (wasm in a Worker) and headless on the native host under
`og run`. These rules are what makes that true; breaking one produces a game
that works on one host, one seed or one machine and not the next.

## What works

Two games written from this package alone (`examples/lantern-vale`, a 3D
adventure, and `examples/hollowbell`, a 2D platformer) hold these rules
for a whole game, not a toy: each ends a full session run in two processes
on one world hash, a player's name and a save live in `str` fields as
world state, and a resource or column a system did not declare is refused
with the system and the record named, so the fix is one line.

## 1. Game code is TypeScript, and only TypeScript

A game is `src/*.ts` against `@origingame/engine`. Never add Rust, a second
engine, a WebGL context, DOM access or a `<canvas>` of your own: the engine
renders, the shell owns the page. Anything that would need engine-side code
is an engine feature request, not something to work around in the game.

## 2. The tick is fixed and the world is deterministic

- Simulation runs in fixed ticks (`GameDesc.tick`, 60 Hz by default). A
  system sees `time.dt`, `time.tick` and `time.elapsed` from the `Time`
  resource; these are functions of the tick count.
- Never read `Date`, `performance.now()` or any other clock inside a system,
  a command or a facts function. Two runs with the same seed and the same
  inputs must produce the same world hash; a wall-clock read breaks that.
- Randomness comes from the `Rng` resource (`world.resource(og.Rng)`) or a
  stream forked from it with a label (`rng.fork("spawner")`). Never
  `Math.random()`. The project seed lives in `project.ogt` and `GameDesc.seed`;
  `og run --seed` overrides it for a replay. A draw (`next`, `range`,
  `integer`, `chance`) moves the stream, so it needs `og.Rng` in `writes`;
  `fork` reads the seed alone and moves nothing, so a system that only
  forks declares `og.Rng` in `reads` (its forks are the same on every host
  and in every system).
- The wall clock reaches the game only as host input, between ticks:
  `og.platform.now(world)` (UTC ms, the host's clock when the platform
  attached or last answered, advanced by the ticks since) and the `time`
  on every og.platform answer. A replay records them like a key press.
- Rendering never feeds back into simulation: do not read camera or screen
  state to decide game logic.
- The engine may spread its own work over the cores (physics, skinning,
  culling: the lanes), and it gives the same result to the bit as one
  thread. Game systems always run on the engine thread, one after another,
  so nothing in game code needs a lock; it needs only the rules above.

## 3. Every failure is a value

Engine calls that can fail return `Result<T>`: `{ ok: true, value }` or
`{ ok: false, reason }`. Check `ok` before reading `value`; the `reason` is a
sentence that names the fix. A dead handle is a refusal, never a crash. Do not
wrap engine calls in `try`/`catch` to hide refusals; handle the `reason`.

## 4. Systems work on archetypes, never on single entities

A system declares what it reads and writes and receives the matching
archetypes once per tick:

```ts
export const Move = defineSystem("Move", { reads: [Velocity], writes: [og.Transform] }, (q, time) => {
  for (const a of q.archetypes()) {
    const p = a.column(og.Transform).position; // Float32Array, stride 3
    const v = a.column(Velocity).v;
    for (let i = 0; i < a.count; i++) {
      p[i * 3] += v[i * 3] * time.dt;
      p[i * 3 + 1] += v[i * 3 + 1] * time.dt;
      p[i * 3 + 2] += v[i * 3 + 2] * time.dt;
    }
  }
});
```

- Columns are typed arrays over engine memory. Write only components listed
  in `writes`; reading needs `reads` or `writes`. An undeclared column throws
  inside the system and fails the tick with the system's name.
- The same holds for resources and events: `world.resource`, an event read
  (`world.events`) and an `og.Rng` fork need the record among `reads` or
  `writes`; a resource write, an `og.Rng` draw and `world.emit` need it in
  `writes`. Undeclared, it throws in `og run` and `og test` (dev and
  release builds) with the system and the record; a publish build refuses
  it, logs it once per system and access, and the game runs on.
- Views belong to one system call. Never keep an `ArchetypeView` or a column
  array across ticks or in module state: a stale view throws with the tick
  and the system it belonged to.
- Text is world state like a number: a `str(capacity)` field holds UTF-8 of
  at most `capacity` bytes inline, zero-padded (`name: str(48)` takes any 12
  characters, four bytes covering every script). `world.get`, `world.set`
  and a resource's fields move it as a JS string; text over the capacity is
  refused with the fix named (an error value from `world.insert`/`world.set`,
  a throw from a resource's setter), never truncated. Its column is the raw
  bytes (`Uint8Array`, stride the capacity). The probe reads it (`get:`,
  `dump_entity`), `set:Prefs.name=Ada Lovelace` writes it (a JSON literal for
  escapes: `set:Prefs.name=""`), a scene gives it one quoted string, and
  snapshots and the world hash carry its bytes, so a name never lives in a
  module variable or as numbers of code points.
- Per-entity calls (`world.get`, `world.set`) are for commands and setup, not
  for hot loops. Inside a system a component it writes is borrowed by its
  views: `world.get` of it refuses, so read and write it through the
  archetype's columns.
- Order systems with `after` and `before` by name; declaration order is the
  default.

## 5. Reflection is the one schema

A component or resource is declared once with `defineComponent` /
`defineResource` and its column factories (`f32(100, { min: 0, max: 1000,
tunable: true })`, `Vector3()`). That one layout is what scenes, dumps, the
probe's `bounds` and `patch`, Tune and the wiki read. Never describe the same
data a second time (a JSON config mirroring a component, a hand-written
schema for the inspector). Put ranges, units and `tunable` on the field where
it is declared.

## 6. Desc objects and names

- Desc objects (`SpawnDesc`, `SystemDesc`, `GameDesc`) may omit any field;
  the zero value or the documented default applies. Pass `{}` rather than
  inventing values the engine already defaults.
- Names follow the API: components, resources and systems PascalCase;
  command actions snake_case; entity names are stable strings, because the
  probe and the command channel address entities by them.
- Vector, quaternion, matrix, colour, transform and material vocabulary is
  three.js's (`position`, `quaternion`, `scale`, `metalness`, `roughness`).

## 7. Where the API is described

The generated wiki (`reference/index.md` next to the `og-game` skill, or
`wiki/index.md` in `@origingame/engine-knowledge`) has one page per module
with every signature quoted from the IDL, grouped by the skill that teaches
it. This skill's are the ECS and its rules: `reference/world.md` (entities,
queries, resources, events), `reference/system.md` (systems and their
order), `reference/reflect.md` (components, resources and their column
factories), `reference/rng.md` (seeded streams) and `reference/math.md`
(vectors, quaternions, colours as data). When the wiki and memory disagree,
the wiki is right: it is regenerated from the same IDL the engine is built
from. The other skills, and which modules each teaches, are og-game §1.
