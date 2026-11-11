<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `vfx` module

Composed effects: one effect (a fireball's impact, a heal, a level-up) as a timeline of elements in an `.ogt` of `type vfx`: emitters, meshes, trails, beams, lights, decals, flashes, flipbooks, shakes and sounds.

An og.Vfx plays one from its startTick at its entity's MatrixWorld. Its visuals come from (asset, seed, startTick, the tick) alone, never world state, so a replay, restore or seek draws the same. In the tick the og.vfx system sends an og.VfxEvent for each `event` line, plays each `shake` and `sound`, and despawns an ended non-looping effect with its sounds (unless `despawn` is false).

See `loading §11`.

- **TypeScript**: functions are the namespace `og.vfx`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Core** (what most games use): `og.VfxEvent`, `VfxSpawnDesc`, `og.vfx.spawn`; 5 more are advanced (`og api show vfx --all`).
- **Index**: [every module and guide](index.md).

```ogt
module vfx {
  doc "Composed effects: one effect (a fireball's impact, a heal, a level-up) as a timeline of elements in an `.ogt` of `type vfx`: emitters, meshes, trails, beams, lights, decals, flashes, flipbooks, shakes and sounds."
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

<sub>Module `vfx` line 9</sub>
