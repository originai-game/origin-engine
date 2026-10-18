---
name: og-game
description: Building a game on Origin Engine — the project layout, components and systems on the ECS, the game entry, scenes as .ogt text, assets cooked from assets/ to cooked/, and the og.load loaders. Use when creating, structuring or extending a TypeScript Origin Engine game (a project with a project.ogt; not a Godot project.godot).
---

An Origin Engine project is plain files: TypeScript in `src/`, source assets
in `assets/`, text scenes, and a manifest. `npm create @origingame/game`
produces it; everything after that is code and text, never an editor.

## The project

| Path | What it is |
|---|---|
| `project.ogt` | The manifest: `name`, `id`, the pinned `engine` version, `entry`, `profile` (`2d` or `3d`), `tick`, `seed`, `scenes`, the first `scene`, `publish` settings |
| `src/main.ts` | The game entry: its default export is `og.game({...})` |
| `assets/` | Source assets: glTF models, PNG/JPEG textures, WAV sounds, TTF/OTF fonts, `.ogt` scenes, and `<file>.ogt` import settings |
| `cooked/` | `og cook` output (`.ogb` blobs and `.cook.ogt`); never committed, the only thing the runtime reads |
| `tests/*.og.ts` | Playtest scripts `og test` runs headless |
| `dist/` | `og build` output |

The engine version in `project.ogt` is a pin. Upgrading it is an explicit
change with its own run, never a side effect.

## Components, resources, systems, the game

```ts
import { og, defineComponent, defineSystem, f32, Vector3 } from "@origingame/engine";

export const Health = defineComponent("Health", { hp: f32(100, { min: 0, max: 1000, tunable: true }), max: f32(100) });
export const Velocity = defineComponent("Velocity", { v: Vector3() });

export const Move = defineSystem("Move", { reads: [Velocity], writes: [og.Transform] }, (q, time) => {
  for (const a of q.archetypes()) {
    const p = a.column(og.Transform).position;
    const v = a.column(Velocity).v;
    for (let i = 0; i < a.count * 3; i++) p[i] += v[i] * time.dt;
  }
});

export default og.game({
  tick: 60,
  scenes: { main: "scenes/main.ogt" },
  systems: [Move],
  commands: { spawn_wave: (world, args) => spawnWave(world, Number(args[0])) },
});
```

- A **component** is plain data per entity; a **resource** is a reflected
  singleton (tuning tables, game state) read with `world.resource(token)`.
  Engine-owned ones are tokens on `og`: `og.Transform`, `og.Time`, `og.Rng`,
  `og.Input`, `og.Camera`.
- A **system** is a function over its matched archetypes, once per tick
  (`og-core` has the rules). Keep each system to one job; share state through
  components and resources, never module variables.
- **Entities** are spawned with `world.spawn({ name, parent, position })` and
  addressed by name everywhere outside the hot loop: `world.find(name)`, the
  probe's ids, `teleport`, `set`, `dump_entity`. Give every entity a person
  will ask about a stable name.
- **Commands** in `GameDesc.commands` are snake_case actions the command
  channel and playtests can call (`0.5:spawn_wave:3`). Add one for each
  state a test needs to reach quickly, instead of playing there.
- **Facts** (`GameDesc.facts`) return a small JSON object for the probe's
  `state()`: score, phase, lives. Tests assert on facts and dumps.

## Scenes are text

A scene is an `.ogt` file under `assets/`, cooked like any asset:

```text
type scene
version 1
---
name "Main"
entity "ship" {
  model "models/dropship.glb"
  Transform {
    position 0 1.5 0
  }
  entity "light" {
    Transform {
      position 0 3 0
    }
  }
}
```

Each `entity` has a name, optional `model` (a glTF under `assets/` whose
default scene becomes the entity's children), component blocks keyed by the
component's reflected name with one line per field, and nested child
entities. Fields not written take their defaults. Edit scenes as text and
diff them like code; the cook checks field names and lane counts against the
registered schema and refuses a model path that does not exist.

## Assets: assets/ to cooked/

- Put sources in `assets/`; `og cook` turns them into `cooked/`: glTF into
  meshopt-compressed meshes with LODs, textures into KTX2 Basis Universal
  (ETC1S for colour, UASTC for normal and linear maps), WAV into QOA, fonts
  into MSDF atlases, scenes into component tables. Import settings live next
  to the source as `<file>.ogt` (`texture { kind normal }`,
  `font { charset latin chars "开始游戏" }`).
- Declare the characters a font must show (`chars`) rather than cooking a
  whole CJK charset: the full set is several times the size of the font.
- Reusable production assets come through the platform's asset catalog into
  `assets/`; record each file's licence next to it.

## Loading

`og.load.*` returns a handle at once and completes at a tick boundary; under
`og run` and in replays every load completes before the next tick, so the
tick it lands on is deterministic. Paths are asset ids (the path under
`assets/`). Check `og.load.status` (or the `AssetReady` / `AssetFailed`
events) before using an asset, and spawn a loaded scene or model with
`world.instantiate(asset, { name, position })`. The scene in
`GameDesc.scene` is spawned for you when the world starts. The wiki's `load`
page lists every loader and what each returns.

## Input

`world.resource(og.Input)` is the keyboard, the mouse and the first touch
(`down("KeyW")`, `pressed`, `axis`, `buttonDown`). Two more resources carry
what one pointer cannot: `og.Pointers` (pointer 0 is og.Input's, 1 to 4
further touches: `down(id)`, `pressed(id)`, `position(id)`, `start(id)`)
and `og.Gamepad` (up to four pads: `connected(pad)`, `down("a", pad)`,
`pressed`, `released`, `axis("leftX", pad)`; sticks are y down). All three
are world state, so playtests and replays reproduce them; script them with
`touch:<id>:<down|move|up>:<x,y>` and `gamepad:<a+b|->[:lx,ly][:lt,rt]`.

## Effects

Every effect is off until the game turns it on, as data:

- **Post** — the `og.PostEffects` resource: `ao`, `outline`,
  `depthOfField`, `motionBlur`, `godRays`, `glow`, `lut` (a handle from
  `og.load.lut(world, "luts/look.cube")`), `vignette`,
  `chromaticAberration`, `filmGrain`, `flash`, and camera shake through
  `shakeTrauma` (set it on a hit and let it decay). A game with fast motion
  wants `render: { antialias: "taa" }` in `og.game`.
- **Particles** — an emitter is an `.ogt` asset of `type particles`
  (rate, bursts, lifetime, speed, curves, collision; cooked like any asset)
  loaded with `og.load.emitter`; an entity with `og.ParticleSystem
  { emitter, seed, startTick }` emits it at its transform. Particles are
  drawn by the renderer from the tick, deterministic and never part of the
  simulation.
- **Trails and beams** — components `og.Trail` (follows the entity) and
  `og.Beam` (between two points); `og.render.ribbon(world, desc)` draws a
  polyline this tick.
- **Decals** — `og.Decal` projects an atlas rectangle
  (`og.RenderSettings.decalAtlas`) onto what its box covers.
- **2D lighting** — `og.Lighting2D` (ambient, the lit layer range),
  `og.Light2D` and `og.Occluder2D` components, `castShadow` on sprites;
  sprites also take `flash`, `glow` and `blend`.
- **Materials** — toon, rim, dissolve, hit flash and outline are
  `og.render.standardMaterial` fields; animate them with
  `og.render.updateMaterial(world, material, { dissolve: t })`.
- **HUD** — `og.render.sprite` / `og.render.text` with `screen: true` draw
  in CSS pixels from the top-left over the finished frame, untouched by
  post effects, with an optional `clip`; `og.ui` uses the same layer.

`templates/fx-showcase` shows every one of these on five stations.

## Where to look things up

`reference/index.md` (the generated wiki shipped with this skill) maps every
module, type and function to its page, with the IDL text, defaults, ranges
and units of every field.
