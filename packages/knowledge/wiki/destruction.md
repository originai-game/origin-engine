<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `destruction` module

Props that break (walls, crates, pillars, glass) and 2D ground that carves. A model cooked with a `fracture { }` block is cut by the cook into Voronoi chunks bonded where they touch; hits, og.destruction.damage and explode break an og.Destructible's bonds, each unanchored island falls as one body, and a stress solver (NVIDIA Blast's) breaks what can no longer carry its own weight. og.PixelTerrain is 2D ground of cells.

It runs in the tick after the game's systems, before og.physics, in fixed f32 arithmetic; bonds, islands, chunks and cells are world state on both hosts.

See `physics §9`.

- **TypeScript**: functions are the namespace `og.destruction`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Core** (what most games use): `MATERIAL_BRICK`, `MATERIAL_CONCRETE`, `MATERIAL_GLASS`, `og.Destructible`, `og.Chunk`, `og.Fractured`, `DamageDesc`, `ExplosionDesc`, `DestructibleInfo`, `og.destruction.damage`, `og.destruction.explode`, `og.destruction.info`, `og.destruction.material`, `og.PixelTerrain`, `og.PixelDetached`, `Fill2dDesc`, `Explosion2dDesc`, `og.destruction.fill2d`, `og.destruction.explode2d`; 20 more are advanced (`og api show destruction --all`).
- **Index**: [every module and guide](index.md).

```ogt
module destruction {
  doc "Props that break (walls, crates, pillars, glass) and 2D ground that carves. A model cooked with a `fracture { }` block is cut by the cook into Voronoi chunks bonded where they touch; hits, og.destruction.damage and explode break an og.Destructible's bonds, each unanchored island falls as one body, and a stress solver (NVIDIA Blast's) breaks what can no longer carry its own weight. og.PixelTerrain is 2D ground of cells."
  doc "It runs in the tick after the game's systems, before og.physics, in fixed f32 arithmetic; bonds, islands, chunks and cells are world state on both hosts."
  # declarations follow, each on its own page
```

<sub>Module `destruction` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`CAUSE_IMPACT`](#const-CAUSE_IMPACT) | constant |  | Fractured.cause: a collision's impulse (og.CollisionStarted) past the prop's impactThreshold. |
| [`CAUSE_DAMAGE`](#const-CAUSE_DAMAGE) | constant |  | Fractured.cause: og.destruction.damage. |
| [`CAUSE_EXPLOSION`](#const-CAUSE_EXPLOSION) | constant |  | Fractured.cause: og.destruction.explode; PixelDetached.cause: og.destruction.explode2d. |
| [`CAUSE_STRESS`](#const-CAUSE_STRESS) | constant |  | Fractured.cause: the stress solver: a part that could no longer hold its own weight. |
| [`CAUSE_SHATTER`](#const-CAUSE_SHATTER) | constant |  | Fractured.cause: a chunk with children (the fracture block's `children`) took three times the damage that breaks a bond of its cross-section, and broke into them: they fly free, each its own body, le… |
| [`MATERIAL_BRICK`](#const-MATERIAL_BRICK) | constant |  | og.destruction.material: brickwork in mortar (no-tension masonry: stands by its weight, steps back over its piers). |
| [`MATERIAL_CONCRETE`](#const-MATERIAL_CONCRETE) | constant |  | og.destruction.material: plain concrete (strong, breaks in chunky lumps, holds what is left). |
| [`MATERIAL_WOOD`](#const-MATERIAL_WOOD) | constant |  | og.destruction.material: timber boards (light and weak across the grain: splinters where it lands or is struck). |
| [`MATERIAL_GLASS`](#const-MATERIAL_GLASS) | constant |  | og.destruction.material: a glass pane (shatters at a touch, no stress solver). |
| [`MATERIAL_STONE`](#const-MATERIAL_STONE) | constant |  | og.destruction.material: dry-laid stone (masonry with no mortar: no tension, heavier and tougher than brick). |
| [`CAUSE_FRACTURE`](#const-CAUSE_FRACTURE) | constant |  | Fractured.cause: og.destruction.fracture; PixelDetached.cause: og.destruction.fracture2d. |
| [`CAUSE_CARVE`](#const-CAUSE_CARVE) | constant |  | PixelDetached.cause: og.destruction.carve2d cut it loose. |
| [`og.Destructible`](destruction/record-Destructible.md) | component |  | Breaks the entity's og.Model, cooked with a `fracture` block, into its chunks (3D profile). |
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
| [`og.PixelTerrain`](destruction/record-PixelTerrain.md) | component |  | Destructible 2D ground of square cells: ground, bedrock or empty. |
| [`og.PixelPiece`](destruction/record-PixelPiece.md) | component |  | A piece of an og.PixelTerrain cut loose (an island carve2d cut free, a piece explode2d threw or fracture2d broke off), written by og.destruction2d: a dynamic og.RigidBody whose og.Collider is the con… |
| [`og.PixelDetached`](destruction/record-PixelDetached.md) | event |  | Cells of an og.PixelTerrain (or of an og.PixelPiece fracture2d broke) broke loose during the tick, read the tick after: one for each piece, in the order they were cut, for its sound, its dust, its sc… |
| [`Carve2dDesc`](destruction/type-Carve2dDesc.md) | desc struct |  | A round hole for og.destruction.carve2d. |
| [`Fill2dDesc`](destruction/type-Fill2dDesc.md) | desc struct |  | A round patch of ground for og.destruction.fill2d. |
| [`Explosion2dDesc`](destruction/type-Explosion2dDesc.md) | desc struct |  | A blast for og.destruction.explode2d. |
| [`Fracture2dDesc`](destruction/type-Fracture2dDesc.md) | desc struct |  | How og.destruction.fracture2d breaks a piece. |
| [`og.destruction.carve2d`](destruction/fn-carve2d.md) | function | `(world: World, entity: Entity, desc?: Carve2dDesc): Result<void>` | Empty a round hole in an og.PixelTerrain, this tick (applied by og.destruction2d after the game's systems, in the order called): the chunks it touches collide as their new outline before this tick's … |
| [`og.destruction.fill2d`](destruction/fn-fill2d.md) | function | `(world: World, entity: Entity, desc?: Fill2dDesc): Result<void>` | Make a round patch of an og.PixelTerrain solid (ground, or bedrock), this tick, as carve2d. |
| [`og.destruction.explode2d`](destruction/fn-explode2d.md) | function | `(world: World, desc?: Explosion2dDesc): Result<void>` | A blast on every og.PixelTerrain in reach, this tick, as carve2d: a hole of `radius`, the ring of ground `debris` wide round it cut into Voronoi pieces and thrown outwards, the islands it cuts loose … |
| [`og.destruction.fracture2d`](destruction/fn-fracture2d.md) | function | `(world: World, entity: Entity, desc?: Fracture2dDesc): Result<void>` | Break an og.PixelPiece (or every cell of an og.PixelTerrain but its bedrock) into Voronoi pieces, this tick, as carve2d: each piece is an og.PixelPiece moving as what it broke from moved; the piece b… |
| [`og.destruction.cell2d`](destruction/fn-cell2d.md) | function | `(world: World, entity: Entity, point: Vector3): Result<number>` | The cell of an og.PixelTerrain under a world point, as the last tick left it: 0 empty, 255 bedrock, any other ground (fill2d writes 1). |

## Constants

| Constant | Type | Value | Doc |
|---|---|---|---|
| <a id="const-CAUSE_IMPACT"></a>`CAUSE_IMPACT` | `number` | `0` | Fractured.cause: a collision's impulse (og.CollisionStarted) past the prop's impactThreshold. |
| <a id="const-CAUSE_DAMAGE"></a>`CAUSE_DAMAGE` | `number` | `1` | Fractured.cause: og.destruction.damage. |
| <a id="const-CAUSE_EXPLOSION"></a>`CAUSE_EXPLOSION` | `number` | `2` | Fractured.cause: og.destruction.explode; PixelDetached.cause: og.destruction.explode2d. |
| <a id="const-CAUSE_STRESS"></a>`CAUSE_STRESS` | `number` | `3` | Fractured.cause: the stress solver: a part that could no longer hold its own weight. |
| <a id="const-CAUSE_SHATTER"></a>`CAUSE_SHATTER` | `number` | `4` | Fractured.cause: a chunk with children (the fracture block's `children`) took three times the damage that breaks a bond of its cross-section, and broke into them: they fly free, each its own body, leaving a hole. |
| <a id="const-MATERIAL_BRICK"></a>`MATERIAL_BRICK` | `number` | `0` | og.destruction.material: brickwork in mortar (no-tension masonry: stands by its weight, steps back over its piers). |
| <a id="const-MATERIAL_CONCRETE"></a>`MATERIAL_CONCRETE` | `number` | `1` | og.destruction.material: plain concrete (strong, breaks in chunky lumps, holds what is left). |
| <a id="const-MATERIAL_WOOD"></a>`MATERIAL_WOOD` | `number` | `2` | og.destruction.material: timber boards (light and weak across the grain: splinters where it lands or is struck). |
| <a id="const-MATERIAL_GLASS"></a>`MATERIAL_GLASS` | `number` | `3` | og.destruction.material: a glass pane (shatters at a touch, no stress solver). |
| <a id="const-MATERIAL_STONE"></a>`MATERIAL_STONE` | `number` | `4` | og.destruction.material: dry-laid stone (masonry with no mortar: no tension, heavier and tougher than brick). |
| <a id="const-CAUSE_FRACTURE"></a>`CAUSE_FRACTURE` | `number` | `5` | Fractured.cause: og.destruction.fracture; PixelDetached.cause: og.destruction.fracture2d. |
| <a id="const-CAUSE_CARVE"></a>`CAUSE_CARVE` | `number` | `6` | PixelDetached.cause: og.destruction.carve2d cut it loose. |

```ogt
const CAUSE_IMPACT u32 0 "Fractured.cause: a collision's impulse (og.CollisionStarted) past the prop's impactThreshold."
```

<sub>Module `destruction` line 7</sub>

```ogt
const CAUSE_DAMAGE u32 1 "Fractured.cause: og.destruction.damage."
```

<sub>Module `destruction` line 8</sub>

```ogt
const CAUSE_EXPLOSION u32 2 "Fractured.cause: og.destruction.explode; PixelDetached.cause: og.destruction.explode2d."
```

<sub>Module `destruction` line 9</sub>

```ogt
const CAUSE_STRESS u32 3 "Fractured.cause: the stress solver: a part that could no longer hold its own weight."
```

<sub>Module `destruction` line 10</sub>

```ogt
const CAUSE_SHATTER u32 4 "Fractured.cause: a chunk with children (the fracture block's `children`) took three times the damage that breaks a bond of its cross-section, and broke into them: they fly free, each its own body, leaving a hole."
```

<sub>Module `destruction` line 11</sub>

```ogt
const MATERIAL_BRICK u32 0 "og.destruction.material: brickwork in mortar (no-tension masonry: stands by its weight, steps back over its piers)."
```

<sub>Module `destruction` line 12</sub>

```ogt
const MATERIAL_CONCRETE u32 1 "og.destruction.material: plain concrete (strong, breaks in chunky lumps, holds what is left)."
```

<sub>Module `destruction` line 13</sub>

```ogt
const MATERIAL_WOOD u32 2 "og.destruction.material: timber boards (light and weak across the grain: splinters where it lands or is struck)."
```

<sub>Module `destruction` line 14</sub>

```ogt
const MATERIAL_GLASS u32 3 "og.destruction.material: a glass pane (shatters at a touch, no stress solver)."
```

<sub>Module `destruction` line 15</sub>

```ogt
const MATERIAL_STONE u32 4 "og.destruction.material: dry-laid stone (masonry with no mortar: no tension, heavier and tougher than brick)."
```

<sub>Module `destruction` line 16</sub>

```ogt
const CAUSE_FRACTURE u32 5 "Fractured.cause: og.destruction.fracture; PixelDetached.cause: og.destruction.fracture2d."
```

<sub>Module `destruction` line 17</sub>

```ogt
const CAUSE_CARVE u32 6 "PixelDetached.cause: og.destruction.carve2d cut it loose."
```

<sub>Module `destruction` line 18</sub>
