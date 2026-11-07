<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `vfx` module

Composed effects: one effect (a fireball's impact, a heal, a level-up) as a timeline of elements in an `.ogt` of `type vfx`, Effekseer's and Niagara's systems as text the agent reads and diffs. A file has `duration <seconds>`, `loop true|false`, `event <seconds> "<name>"` and `impact <seconds>` (`event <seconds> "impact"`, at most one; the same four lines mean the same in a `type vfx2d` pixel effect), and element blocks, each with `at <seconds>` from the effect's start, `offset <x> <y> <z>` and `rotate <x> <y> <z>` (degrees) in the effect's space, `move <x> <y> <z>` (metres per second) and `accelerate <x> <y> <z>` (per second squared) carrying it from its offset from its start (an emitter spawns along the path), and `life <seconds>` or `until <seconds>` for those that last: `emitter "<id>"` (an og.load.emitter asset; `until` stops its emission and its particles live out; `seed`), `mesh sphere | quad | ring | box | cylinder | "<model id>"` (drawn with `effect "<id>"`, an effect material: `scale`, a value over its life, `spin` degrees per second, `color`, `blend additive | alpha`; a `ring` takes its own shape with any of `inner <radius>` (outer 1), `band <0..1>` (its middle row), `height <outer> <inner>` (its edges raised along its axis: a cone, a tube), `arc <start> <length>` (degrees from +x), `segments <n>` and `colors <outer rgba> <band rgba> <inner rgba>`, its u then along the arc and v from the outer edge to the inner, built at load), `trail` (`effect`, `width <head> <tail>`, `lifetime`, `color`, `orbit <radius> <turns per second>` and `rise <metres per second>` about and up the element's y), `beam` (`to <x> <y> <z>` from the element, `width`, `effect`, `segments`, `jitter`, `color`), `light` (a point light: `color`, `intensity` a value over its life, `distance`), `decal` (projected down the element's -y: `uv <u0> <v0> <u1> <v1>`, its region of og.RenderSettings.decalAtlas, `size <x> <y> <z>` with y the depth it reaches, `fade <seconds>` at its end, `color`), `flash` (a soft glow facing the eye: `size` a value over its life, `color`), `shake` (`trauma`, og.camera.shake), `flipbook "<id>"` (a pixel effect, an `.ogt` of `type vfx2d`, or any og.load.flipbook sheet: a camera-facing og.Sprite with og.Flipbook playing its first clip, `size` its height with the width by the frame's shape, `color`, `blend additive | alpha`, for `life` or its clip's length; its og.FlipbookEvent events, the impact frame among them, reach the game as this effect's og.VfxEvent of the same name) and `sound "<id>"` (`volume`, `pitch <min> <max>` picked by the seed; a child entity with og.AudioSource at the element, cut when the effect despawns). A value is a number, `random <min> <max>` (fixed per instance) or `curve <t> <v> ...` over the element's life. `og cook` checks the file and that what it names is in the project.

An og.Vfx plays one from its startTick at its entity's MatrixWorld, so an effect follows the entity it is spawned on (og.vfx.spawn with a parent) or stays where it was spawned. Its visuals are drawn from (asset, seed, startTick, the tick) alone: the renderer expands its elements every frame and nothing of them is world state, so a replay, a restore or a seek draws the same effect. Its gameplay side runs in the tick, in the engine's og.vfx system (scheduled when a game first names og.Vfx or og.VfxEvent, after the systems defined so far): each `event` line sends an og.VfxEvent on the tick its time lands on (startTick plus its time rounded to ticks; an effect started after og.vfx ran that tick catches up on the next), each `shake` adds its trauma and each `sound` plays, then an effect that has ended and does not loop despawns its entity and its sounds (unless `despawn` is false). Element times run on the tick.

- **TypeScript**: functions are the namespace `og.vfx`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module vfx {
  doc "Composed effects: one effect (a fireball's impact, a heal, a level-up) as a timeline of elements in an `.ogt` of `type vfx`, Effekseer's and Niagara's systems as text the agent reads and diffs. A file has `duration <seconds>`, `loop true|false`, `event <seconds> \"<name>\"` and `impact <seconds>` (`event <seconds> \"impact\"`, at most one; the same four lines mean the same in a `type vfx2d` pixel effect), and element blocks, each with `at <seconds>` from the effect's start, `offset <x> <y> <z>` and `rotate <x> <y> <z>` (degrees) in the effect's space, `move <x> <y> <z>` (metres per second) and `accelerate <x> <y> <z>` (per second squared) carrying it from its offset from its start (an emitter spawns along the path), and `life <seconds>` or `until <seconds>` for those that last: `emitter \"<id>\"` (an og.load.emitter asset; `until` stops its emission and its particles live out; `seed`), `mesh sphere | quad | ring | box | cylinder | \"<model id>\"` (drawn with `effect \"<id>\"`, an effect material: `scale`, a value over its life, `spin` degrees per second, `color`, `blend additive | alpha`; a `ring` takes its own shape with any of `inner <radius>` (outer 1), `band <0..1>` (its middle row), `height <outer> <inner>` (its edges raised along its axis: a cone, a tube), `arc <start> <length>` (degrees from +x), `segments <n>` and `colors <outer rgba> <band rgba> <inner rgba>`, its u then along the arc and v from the outer edge to the inner, built at load), `trail` (`effect`, `width <head> <tail>`, `lifetime`, `color`, `orbit <radius> <turns per second>` and `rise <metres per second>` about and up the element's y), `beam` (`to <x> <y> <z>` from the element, `width`, `effect`, `segments`, `jitter`, `color`), `light` (a point light: `color`, `intensity` a value over its life, `distance`), `decal` (projected down the element's -y: `uv <u0> <v0> <u1> <v1>`, its region of og.RenderSettings.decalAtlas, `size <x> <y> <z>` with y the depth it reaches, `fade <seconds>` at its end, `color`), `flash` (a soft glow facing the eye: `size` a value over its life, `color`), `shake` (`trauma`, og.camera.shake), `flipbook \"<id>\"` (a pixel effect, an `.ogt` of `type vfx2d`, or any og.load.flipbook sheet: a camera-facing og.Sprite with og.Flipbook playing its first clip, `size` its height with the width by the frame's shape, `color`, `blend additive | alpha`, for `life` or its clip's length; its og.FlipbookEvent events, the impact frame among them, reach the game as this effect's og.VfxEvent of the same name) and `sound \"<id>\"` (`volume`, `pitch <min> <max>` picked by the seed; a child entity with og.AudioSource at the element, cut when the effect despawns). A value is a number, `random <min> <max>` (fixed per instance) or `curve <t> <v> ...` over the element's life. `og cook` checks the file and that what it names is in the project."
  doc "An og.Vfx plays one from its startTick at its entity's MatrixWorld, so an effect follows the entity it is spawned on (og.vfx.spawn with a parent) or stays where it was spawned. Its visuals are drawn from (asset, seed, startTick, the tick) alone: the renderer expands its elements every frame and nothing of them is world state, so a replay, a restore or a seek draws the same effect. Its gameplay side runs in the tick, in the engine's og.vfx system (scheduled when a game first names og.Vfx or og.VfxEvent, after the systems defined so far): each `event` line sends an og.VfxEvent on the tick its time lands on (startTick plus its time rounded to ticks; an effect started after og.vfx ran that tick catches up on the next), each `shake` adds its trauma and each `sound` plays, then an effect that has ended and does not loop despawns its entity and its sounds (unless `despawn` is false). Element times run on the tick."
  # declarations follow, each on its own page
```

<sub>Module `vfx` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`VfxAsset`](vfx/type-VfxAsset.md) | handle |  | A loaded composed effect (og.load.vfx): its elements and the emitters, effect materials, meshes, textures and sounds they name. |
| [`VfxAsset()`](#column-VfxAsset) | column kind |  | A VfxAsset handle field (0 is none). |
| [`og.Vfx`](vfx/record-Vfx.md) | component |  | Plays a composed effect at the entity's MatrixWorld from startTick (og.vfx.spawn makes one in a call). |
| [`og.VfxEvent`](vfx/record-VfxEvent.md) | event |  | A composed effect reached one of its `event` lines (an `impact` line included, named `impact`): the frame a fireball lands, a slash connects, a spell resolves. |
| [`VfxSpawnDesc`](vfx/type-VfxSpawnDesc.md) | desc struct |  | Where and how og.vfx.spawn plays an effect. |
| [`VfxInfo`](vfx/type-VfxInfo.md) | struct |  | A loaded composed effect. |
| [`og.vfx.spawn`](vfx/fn-spawn.md) | function | `(world: World, desc?: VfxSpawnDesc): Result<Entity>` | Play a composed effect now: a new entity with og.Transform (at the desc's position and rotation, a child of its parent when it has one) and og.Vfx started at this tick (world.tick). |
| [`og.vfx.info`](vfx/fn-info.md) | function | `(world: World, asset: VfxAsset): Result<VfxInfo>` | What a loaded composed effect holds. |

## Column kinds

Each is a factory, `VfxAsset(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-VfxAsset"></a>`VfxAsset` | `VfxAsset` | `Uint32Array` | 2 | A VfxAsset handle field (0 is none). |

```ogt
column VfxAsset {
  doc "A VfxAsset handle field (0 is none)."
  array Uint32Array
  stride 2
  value VfxAsset
}
```

<sub>Module `vfx` line 7</sub>
