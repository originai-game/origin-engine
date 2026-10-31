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

`examples/<name>` in this package's skills is a game in the engine's
source repository (https://github.com/originai-game/origin-engine/tree/main/examples),
not installed with the package; read one there when a skill points to it.
What a game starts from is installed: the archetypes of
`@origingame/create-game` (`npm create @origingame/game -- --archetype <name>`,
or Origin's `engine_scaffold`).

Two games written from this package alone (`examples/lantern-vale`, a 3D
adventure, and `examples/hollowbell`, a 2D platformer) hold these rules
for a whole game, not a toy: each ends a full session run in two processes
on one world hash, a player's name and a save live in `str` fields as
world state, and a resource or column a system did not declare is refused
with the system and the record named, so the fix is one line. A third,
The Kestrel Stair (`examples/kestrel-stair`, 3D third-person), restored
`world.snapshot()` after a relaunch with `world.restore()`: 93 entities
came back with their cloth, navmesh paths and behaviour state, and a full
session in two processes ended on one hash.

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

### Saving progress

The world is one snapshot: `world.snapshot()` returns it as text
(`og-save:1:...`: every entity, component, name, resource, rng state,
pending event and module state, with each type's schema hash) in every
build, publish included, and `world.restore(save)` puts it back, so the
restored world hashes as the saved one did, on either host. The session
stays across a restore (input, the player's volumes, the platform's calls,
the language, scheduled command steps); og.Time takes the saved tick.
Inside a system the snapshot is the world in the middle of that tick, as
that system sees it (its own queued spawns and despawns are not in it),
and it names the system: the world restored from it completes the tick on
its next step (the systems after that one, then the engine's, physics
included) and then goes on tick for tick as the run that saved did, so a
replay from it matches. Make the snapshot the system's last write of the
tick. A restore asked for in a system applies when the tick ends. A save whose types' fields changed is refused naming them, or
mapped by name with `{ migrate: true }`. Only what is in the world is
saved: progress in module variables is lost. The text is a few tens of
KiB (11 KiB for a few entities, 25 KiB for a 2D level of 189, 78 KB for a
3D world of 93 with cloths, minds and skinned characters), often over the
store's 64 KiB a key: save it with `og.platform.saveSlot(world, { slot,
text, meta })`, which spreads it over as many keys as it needs and writes
the slot's manifest last, so a save cut short leaves the last one whole,
and read it back with `og.platform.loadSlot` (og-game §8, og-platform-sdk
§2).

### Recording, replaying and ghosts

The engine has no in-game recorder: determinism is the recorder. The same
world and the same inputs give the same run to the tick, so a game records
its own inputs and replays them. Three things are called "replay": the
probe's and playtests' re-runs (`og run --commands`, `--seed`: tests,
og-build §4), a **watch replay** (the player's run driven again) and a
**ghost** (a past run drawn beside the player). The last two are game code;
examples/meridian-loop does both (`src/replay.ts`, `src/tape.ts`,
`src/tapes.ts`, `src/ghost.ts`; `tests/replay.og.ts` checks the replay
crosses the line on the race's own ticks).

Watch replay: snapshot the start, tape the inputs, restore and feed the tape.

```ts
// 1. The start, taken by the FIRST system of a tick (later in a tick, the
//    restored world skips the rest of that tick and runs a different race).
r.replayMark = true;                       // saved set: the restored world knows it is a replay
const snap = world.snapshot();
r.replayMark = false;
const asked = snap.ok ? og.platform.saveSlot(world, { slot: "replay", text: snap.value }) : snap;
if (asked.ok) r.replaySave = asked.value;  // its answer comes on a later tick (og.platform.result)

// 2. Every tick: the intent drives the car only through its quantised code,
//    live and replayed alike, so the tape reproduces it exactly.
const code = r.replaying ? codeAt(tape, r.runLen) : encode({ steer, throttle, brake });
const i = decode(code);                     // steer 33 steps, throttle 17, brake 5: 12 bits
world.set(car, og.Vehicle, { steer: i.steer, throttle: i.throttle, brake: i.brake });
if (!r.replaying) putCode(tape, r.runLen, code);   // two characters a tick
r.runLen += 1;

// 3. At the finish: the tape beside the start.
og.platform.storeSet(world, { key: "replay.tape", value: { ticks: r.runLen, inputs: tapeText } });

// 4. Watch replay: loadSlot, then world.restore(text) when it answers (it
//    applies when the tick ends). The next tick sees replayMark set: clear
//    it, set replaying, read "replay.tape" back, feed it from tick 0.
```

Keep the tape and the replay's flags in components and resources (a
restore rolls back everything else). A component has no array field and a
`str` field holds at most 4,096 bytes, so a tape of a few thousand ticks
is cut into `str` chunks, one entity each, written through the column's
bytes by the system that owns them (§4). The start is the whole world:
triangle-mesh colliders make it large (a 956 m circuit's world is 400 KB),
so it goes through `saveSlot`, which spreads it over the store's keys.
Request numbers are not carried across a restore: ask the store again
after it.

A ghost: record poses, draw a model. While a lap runs, tape the car's pose
every second tick (position and yaw, pitch, roll: 15 characters); from the
next lap draw a model with no body where the best lap's car was at the
same time into its lap, between the two poses around it:

```ts
if (into % 2 === 0) putPose(cur, into / 2, { ...car.position, ...euler(car.quaternion) });
const f = into / 2, i0 = Math.floor(f), u = f - i0;
const a = poseAt(best, i0), b = poseAt(best, i0 + 1);
world.set(ghost, og.Transform, {
  position: { x: a.x + (b.x - a.x) * u, y: a.y + (b.y - a.y) * u, z: a.z + (b.z - a.z) * u },
  quaternion: quat(lerpAngle(a.yaw, b.yaw, u), lerpAngle(a.pitch, b.pitch, u), lerpAngle(a.roll, b.roll, u)),
});
```

Do not re-drive a second physics body from the tape for a ghost: it takes
part in the simulation the player drives in (a second og.Vehicle's wheel
rays land on whatever collider is under them, the player's car included),
so the player's run is no longer the one taped.

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
- A resource handle is a typed view of the resource's packed record:
  `world.resource(R)` hands back the same handle in a world every time, and
  a field read or write is a typed-array access (about 0.02 µs, whatever
  the resource's size; a 130-field run state costs what an 11-field one
  does). The record is fetched again, in one engine call answering its
  bytes (no JSON), only by the first read after another engine call
  (`world.*`, `og.*`) or a system boundary, the only times the engine can
  change it; writes are kept and written back in one call before the next
  engine call and when the system or command returns. So two handles
  always agree, a read after an engine call sees the engine's value, and
  the world hash is what writing field by field gives. The rule: keep the
  run in one resource and read and write its fields where they are used;
  no plain copies, no handle passed around to save a fetch. What costs is
  alternating an engine call with a read of a resource the system wrote
  (one write-back and one fetch each time, a few µs): in a hot loop over
  entities, read the fields into locals before the loop's engine calls.
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
  A `str` holds 1 to 4,096 bytes: a larger capacity typechecks and is
  refused when the game starts (`a string field holds 1 to 4096 bytes`).
- A field holds one value, or a fixed-capacity list of one number or
  handle kind: `list(u32(), 8)`, `list(Asset(), 4)` (1 to 1,024 elements,
  starting empty, never tunable). Reading it answers a frozen array, a
  copy: `push`, `splice` and index writes throw, so write it whole
  (`v.passes = [...v.passes, h]`, `v.passes = []`, `world.set(e, Chain,
  { passes: [3, 5] })`). One over the capacity is refused with the fix
  named (`declare the field list(u32(), 9) or larger`), never truncated.
  Its column is the element's typed array with (capacity + 1) lanes per
  entity, the count first, zeros past it, so `[]` and `[0]` are different
  worlds. The probe reads it as a JSON array (`get:`, `dump_entity`),
  `set:Chain.passes=[3,5]` (or `3,5`, nothing for `[]`) writes it whole, a
  scene or a look gives it on one line (`passes 3 5`, the key alone for
  `[]`), and snapshots and the hash carry it; a migrating restore keeps a
  list that fits a new capacity and empties one that does not.
  A list holds numbers only: a few vectors are numbered fields
  (`p0`..`p7`, as og.CameraRail's rail points); a list of things with
  fields of their own is an entity per element with its own component,
  looped over by a query (a ring of skid marks, the laps of a race); a
  long record (a lap's taped inputs) is text in `str(4096)` chunks, an
  entity each.
- A handle is a 64-bit number (an `og.load` answer, a texture, a sound,
  a geometry, a material, an entity): keep it in a field of its own kind,
  `Asset()`, `Texture()`, `Sound()`, `Font()`, `Style()`, `Look()`,
  `Geometry()`, `Material()` or `Entity()` (each handle type of the API
  has its factory, named as the type: a font is a `Font()`, never an
  `Asset()` cast), in a component or a resource. A `u32` field refuses one (`4294967298 is not
  a u32`).
- Per-entity calls (`world.get`, `world.set`) are for commands, setup and
  the few named entities a system touches (a camera, a listener, the
  player), not for hot loops. A system's `reads` and `writes` borrow
  nothing by themselves: a column is borrowed when the system takes its
  view (`a.column(C)`), and held until the system returns. So `world.get`
  and `world.set` reach any entity its declarations match until the system
  takes that archetype's column of the component; after it, they refuse
  (naming the view), and the entity is read and written through the view's
  arrays at its row (`a.entities`). Take per-entity calls first, views
  after, or keep to one of the two in a system.
- A system has one query: every component in its `reads` and `writes`
  together, so an entity matches only when it has all of them (and none of
  `without`). A component declared only to `world.get` one named entity
  (the player's car) narrows the loop to entities that also have it, often
  to none, with no error. `world.query` is refused inside a system, by
  design (a system reaches only what it declared): loop over each
  archetype in a system of its own that declares only its components, and
  pass what crosses between them through a resource.
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
- Math on those values is three.js's methods as pure functions, imported
  from `@origingame/engine`: `vec3`, `vec2`, `quat`, `mat4`, `box3`,
  `sphere`, `plane`, `frustum`, `ray`, `color` (`vec3.add(a, b)`,
  `quat.slerp(a, b, t)`, `mat4.transformPoint(m, p)`,
  `frustum.fromCamera(...)` then `frustum.intersectsBox(f, box)`,
  `ray.intersectTriangle(r, a, b, c)` answering a point or null,
  `color.fromHex(0x14161b)` answering linear RGB). They never change their
  arguments and answer new plain objects, so there are no classes to
  construct and a result is written into a column as any other value;
  they use only arithmetic and `sqrt`, so a system using them stays
  deterministic. Signatures: `og api show math`.

## 7. Where the API is described

Look one thing up rather than reading pages: `og api find <words>` lists the
entries that match (name, module, first sentence and field docs), one line
each, and `og api show <name>` prints one in full — signature, fields with
defaults, ranges and units, and its doc (`og api show og.CharacterController`,
`og api show og.physics.raycast`). Both read the wiki below, for the engine
installed.

The generated wiki (`wiki/index.md` in `@origingame/engine-knowledge`) has
one page per module with every signature quoted from the IDL, grouped by
the skill that teaches it; `og api show <module>` prints a module's
overview. This skill's are the ECS and its rules: `og api show world`
(entities, queries, resources, events), `og api show system` (systems and
their order), `og api show reflect` (components, resources and their
column factories), `og api show rng` (seeded streams) and
`og api show math` (vectors, quaternions, colours as data). When the wiki
and memory disagree, the wiki is right: it is regenerated from the same IDL the engine is built
from. The other skills, and which modules each teaches, are og-game §1.
