<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `destruction` module

Props that break: walls, crates, pillars, glass. A model cooked with a `fracture { }` block (at the top of a `type model`, or in a glTF's import settings) is cut by the cook into Voronoi chunks bonded where they touch, and the chunks resting on its floor are anchored to the world. Give the entity that draws it (its og.Model) an og.Destructible: hits, og.destruction.damage and og.destruction.explode wear the bonds down, a broken bond splits the prop into islands, an island anchored to the world stays put and every other island falls as one rigid body, and a stress solver (NVIDIA Blast's) breaks what can no longer carry its own weight, so a wall shot out at its foot comes down.

**Deterministic.** Damage, bonds, islands and the stress solver are fixed f32 arithmetic in the tick (og.destruction runs after the game's systems and before og.physics), the chunks are world entities and the bonds' health is in a state block: snapshots, restores, replays and the world hash cover the whole fracture, on both hosts.

**What a game sees.** og.Fractured, the tick after, for every island that breaks loose (play a sound by its volume, shake the camera); each loose chunk is an entity with og.Chunk (its body is a dynamic og.RigidBody: query it, push it, despawn it). Its look: the og.Destructible's `breakFx` emitter bursts where each island breaks loose and `hitFx` at each damaging hit, each the material's own (dust for stone, splinters for wood, shards and glints for glass, a few sparks only for metal). Budgets (og.Destruction): a loose island (a body and the chunks it carries) smaller than `debrisVolume` despawns after `debrisLifetime`, and past `maxChunks` loose chunks the oldest islands go first.

3D profile only. The chunks of a dynamic prop (a crate with an og.RigidBody) fly with its velocity; a prop without a body, or with a static one, stands on its anchors.

- **TypeScript**: functions are the namespace `og.destruction`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module destruction {
  doc "Props that break: walls, crates, pillars, glass. A model cooked with a `fracture { }` block (at the top of a `type model`, or in a glTF's import settings) is cut by the cook into Voronoi chunks bonded where they touch, and the chunks resting on its floor are anchored to the world. Give the entity that draws it (its og.Model) an og.Destructible: hits, og.destruction.damage and og.destruction.explode wear the bonds down, a broken bond splits the prop into islands, an island anchored to the world stays put and every other island falls as one rigid body, and a stress solver (NVIDIA Blast's) breaks what can no longer carry its own weight, so a wall shot out at its foot comes down."
  doc "**Deterministic.** Damage, bonds, islands and the stress solver are fixed f32 arithmetic in the tick (og.destruction runs after the game's systems and before og.physics), the chunks are world entities and the bonds' health is in a state block: snapshots, restores, replays and the world hash cover the whole fracture, on both hosts."
  doc "**What a game sees.** og.Fractured, the tick after, for every island that breaks loose (play a sound by its volume, shake the camera); each loose chunk is an entity with og.Chunk (its body is a dynamic og.RigidBody: query it, push it, despawn it). Its look: the og.Destructible's `breakFx` emitter bursts where each island breaks loose and `hitFx` at each damaging hit, each the material's own (dust for stone, splinters for wood, shards and glints for glass, a few sparks only for metal). Budgets (og.Destruction): a loose island (a body and the chunks it carries) smaller than `debrisVolume` despawns after `debrisLifetime`, and past `maxChunks` loose chunks the oldest islands go first."
  doc "3D profile only. The chunks of a dynamic prop (a crate with an og.RigidBody) fly with its velocity; a prop without a body, or with a static one, stands on its anchors."
  # declarations follow, each on its own page
```

<sub>`api.ogt` line 27098</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`CAUSE_IMPACT`](#const-CAUSE_IMPACT) | constant |  | Fractured.cause: a collision's impulse (og.CollisionStarted) past the prop's impactThreshold. |
| [`CAUSE_DAMAGE`](#const-CAUSE_DAMAGE) | constant |  | Fractured.cause: og.destruction.damage. |
| [`CAUSE_EXPLOSION`](#const-CAUSE_EXPLOSION) | constant |  | Fractured.cause: og.destruction.explode. |
| [`CAUSE_STRESS`](#const-CAUSE_STRESS) | constant |  | Fractured.cause: the stress solver: a part that could no longer hold its own weight. |
| [`CAUSE_SHATTER`](#const-CAUSE_SHATTER) | constant |  | Fractured.cause: a chunk with children (the fracture block's `children`) took three times the damage that breaks a bond of its cross-section, and broke into them: they fly free, each its own body, le… |
| [`MATERIAL_BRICK`](#const-MATERIAL_BRICK) | constant |  | og.destruction.material: brickwork in mortar (no-tension masonry: stands by its weight, steps back over its piers). |
| [`MATERIAL_CONCRETE`](#const-MATERIAL_CONCRETE) | constant |  | og.destruction.material: plain concrete (strong, breaks in chunky lumps, holds what is left). |
| [`MATERIAL_WOOD`](#const-MATERIAL_WOOD) | constant |  | og.destruction.material: timber boards (light and weak across the grain: splinters where it lands or is struck). |
| [`MATERIAL_GLASS`](#const-MATERIAL_GLASS) | constant |  | og.destruction.material: a glass pane (shatters at a touch, no stress solver). |
| [`MATERIAL_STONE`](#const-MATERIAL_STONE) | constant |  | og.destruction.material: dry-laid stone (masonry with no mortar: no tension, heavier and tougher than brick). |
| [`CAUSE_FRACTURE`](#const-CAUSE_FRACTURE) | constant |  | Fractured.cause: og.destruction.fracture. |
| [`og.Destructible`](destruction/record-Destructible.md) | component |  | Makes the entity's og.Model, a model cooked with a `fracture` block, break into its chunks. |
| [`og.Chunk`](destruction/record-Chunk.md) | component |  | One chunk of an og.Destructible, written by og.destruction when the prop fractures. |
| [`og.Fractured`](destruction/record-Fractured.md) | event |  | An island of chunks broke loose from a destructible during the tick (or the destructible shattered): read the tick after, for its sound, its camera shake, its score. |
| [`og.Destruction`](destruction/record-Destruction.md) | resource |  | The world's destruction budgets. |
| [`DamageDesc`](destruction/type-DamageDesc.md) | desc struct |  | A radial damage for og.destruction.damage. |
| [`ExplosionDesc`](destruction/type-ExplosionDesc.md) | desc struct |  | An explosion for og.destruction.explode. |
| [`DestructibleInfo`](destruction/type-DestructibleInfo.md) | struct |  | What a destructible holds and how much of it stands (og.destruction.info). |
| [`DestructibleMaterial`](destruction/type-DestructibleMaterial.md) | struct |  | A material's og.Destructible fields (og.destruction.material): spread it into the component and add the material's fx, `{ ...og.destruction.material(world, MATERIAL_BRICK), breakFx }`, so a prop is t… |
| [`DestructionCapabilities`](destruction/type-DestructionCapabilities.md) | struct |  | What destruction does on this device. |
| [`og.destruction.damage`](destruction/fn-damage.md) | function | `(world: World, entity: Entity, desc?: DamageDesc): Result<void>` | Damage a destructible at a point: every bond, anchor and chunk within reach takes the amount, all of it within innerRadius and falling to nothing at radius. |
| [`og.destruction.explode`](destruction/fn-explode.md) | function | `(world: World, desc?: ExplosionDesc): Result<void>` | An explosion: damage to every destructible in reach (as og.destruction.damage), then a push outwards to their loose chunks. |
| [`og.destruction.fracture`](destruction/fn-fracture.md) | function | `(world: World, entity: Entity): Result<void>` | Break every bond of a destructible at once (this tick, as damage): its anchored chunks stay, the rest fall. |
| [`og.destruction.info`](destruction/fn-info.md) | function | `(world: World, entity: Entity): Result<DestructibleInfo>` | How much of a destructible stands. |
| [`og.destruction.material`](destruction/fn-material.md) | function | `(world: World, kind: number): Result<DestructibleMaterial>` | A material's og.Destructible tuning (MATERIAL_*): toughness, strength, friction, tension, density and how hard a hit must be to hurt it, measured against the props of the destruction-yard test fixtur… |
| [`og.destruction.capabilities`](destruction/fn-capabilities.md) | function | `(world: World): DestructionCapabilities` | What destruction does on this device. |

## Constants

| Constant | Type | Value | Doc |
|---|---|---|---|
| <a id="const-CAUSE_IMPACT"></a>`CAUSE_IMPACT` | `number` | `0` | Fractured.cause: a collision's impulse (og.CollisionStarted) past the prop's impactThreshold. |
| <a id="const-CAUSE_DAMAGE"></a>`CAUSE_DAMAGE` | `number` | `1` | Fractured.cause: og.destruction.damage. |
| <a id="const-CAUSE_EXPLOSION"></a>`CAUSE_EXPLOSION` | `number` | `2` | Fractured.cause: og.destruction.explode. |
| <a id="const-CAUSE_STRESS"></a>`CAUSE_STRESS` | `number` | `3` | Fractured.cause: the stress solver: a part that could no longer hold its own weight. |
| <a id="const-CAUSE_SHATTER"></a>`CAUSE_SHATTER` | `number` | `4` | Fractured.cause: a chunk with children (the fracture block's `children`) took three times the damage that breaks a bond of its cross-section, and broke into them: they fly free, each its own body, leaving a hole. |
| <a id="const-MATERIAL_BRICK"></a>`MATERIAL_BRICK` | `number` | `0` | og.destruction.material: brickwork in mortar (no-tension masonry: stands by its weight, steps back over its piers). |
| <a id="const-MATERIAL_CONCRETE"></a>`MATERIAL_CONCRETE` | `number` | `1` | og.destruction.material: plain concrete (strong, breaks in chunky lumps, holds what is left). |
| <a id="const-MATERIAL_WOOD"></a>`MATERIAL_WOOD` | `number` | `2` | og.destruction.material: timber boards (light and weak across the grain: splinters where it lands or is struck). |
| <a id="const-MATERIAL_GLASS"></a>`MATERIAL_GLASS` | `number` | `3` | og.destruction.material: a glass pane (shatters at a touch, no stress solver). |
| <a id="const-MATERIAL_STONE"></a>`MATERIAL_STONE` | `number` | `4` | og.destruction.material: dry-laid stone (masonry with no mortar: no tension, heavier and tougher than brick). |
| <a id="const-CAUSE_FRACTURE"></a>`CAUSE_FRACTURE` | `number` | `5` | Fractured.cause: og.destruction.fracture. |

```ogt
const CAUSE_IMPACT u32 0 "Fractured.cause: a collision's impulse (og.CollisionStarted) past the prop's impactThreshold."
```

<sub>`api.ogt` line 27104</sub>

```ogt
const CAUSE_DAMAGE u32 1 "Fractured.cause: og.destruction.damage."
```

<sub>`api.ogt` line 27105</sub>

```ogt
const CAUSE_EXPLOSION u32 2 "Fractured.cause: og.destruction.explode."
```

<sub>`api.ogt` line 27106</sub>

```ogt
const CAUSE_STRESS u32 3 "Fractured.cause: the stress solver: a part that could no longer hold its own weight."
```

<sub>`api.ogt` line 27107</sub>

```ogt
const CAUSE_SHATTER u32 4 "Fractured.cause: a chunk with children (the fracture block's `children`) took three times the damage that breaks a bond of its cross-section, and broke into them: they fly free, each its own body, leaving a hole."
```

<sub>`api.ogt` line 27108</sub>

```ogt
const MATERIAL_BRICK u32 0 "og.destruction.material: brickwork in mortar (no-tension masonry: stands by its weight, steps back over its piers)."
```

<sub>`api.ogt` line 27109</sub>

```ogt
const MATERIAL_CONCRETE u32 1 "og.destruction.material: plain concrete (strong, breaks in chunky lumps, holds what is left)."
```

<sub>`api.ogt` line 27110</sub>

```ogt
const MATERIAL_WOOD u32 2 "og.destruction.material: timber boards (light and weak across the grain: splinters where it lands or is struck)."
```

<sub>`api.ogt` line 27111</sub>

```ogt
const MATERIAL_GLASS u32 3 "og.destruction.material: a glass pane (shatters at a touch, no stress solver)."
```

<sub>`api.ogt` line 27112</sub>

```ogt
const MATERIAL_STONE u32 4 "og.destruction.material: dry-laid stone (masonry with no mortar: no tension, heavier and tougher than brick)."
```

<sub>`api.ogt` line 27113</sub>

```ogt
const CAUSE_FRACTURE u32 5 "Fractured.cause: og.destruction.fracture."
```

<sub>`api.ogt` line 27114</sub>
