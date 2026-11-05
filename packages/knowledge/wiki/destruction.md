<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `destruction` module

Props that break: walls, crates, pillars, glass. A model cooked with a `fracture { }` block (at the top of a `type model`, or in a glTF's import settings) is cut by the cook into Voronoi chunks bonded where they touch, and the chunks resting on its floor are anchored to the world. Give the entity that draws it (its og.Model) an og.Destructible: hits, og.destruction.damage and og.destruction.explode wear the bonds down, a broken bond splits the prop into islands, an island anchored to the world stays put and every other island falls as one rigid body, and a stress solver (NVIDIA Blast's) breaks what can no longer carry its own weight, so a wall shot out at its foot comes down.

**Deterministic.** Damage, bonds, islands and the stress solver are fixed f32 arithmetic in the tick (og.destruction runs after the game's systems and before og.physics), the chunks are world entities and the bonds' health is in a state block: snapshots, restores, replays and the world hash cover the whole fracture, on both hosts.

**What a game sees.** og.Fractured, the tick after, for every island that breaks loose (play a sound by its volume, shake the camera); each loose chunk is an entity with og.Chunk (its body is a dynamic og.RigidBody: query it, push it, despawn it). Its look: the og.Destructible's `breakFx` emitter bursts where each island breaks loose and `hitFx` at each damaging hit, each the material's own (dust for stone, splinters for wood, shards and glints for glass, a few sparks only for metal). Budgets (og.Destruction): a loose island (a body and the chunks it carries) smaller than `debrisVolume` despawns after `debrisLifetime`, and past `maxChunks` loose chunks the oldest islands go first.

3D profile only. The chunks of a dynamic prop (a crate with an og.RigidBody) fly with its velocity; a prop without a body, or with a static one, stands on its anchors.

- **TypeScript**: functions are the namespace `og.destruction`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-destruction`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module destruction {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 28578</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`CAUSE_IMPACT`](#const-CAUSE_IMPACT) | constant | Fractured.cause: a collision's impulse (og.CollisionStarted) past the prop's impactThreshold. |
| [`CAUSE_DAMAGE`](#const-CAUSE_DAMAGE) | constant | Fractured.cause: og.destruction.damage. |
| [`CAUSE_EXPLOSION`](#const-CAUSE_EXPLOSION) | constant | Fractured.cause: og.destruction.explode. |
| [`CAUSE_STRESS`](#const-CAUSE_STRESS) | constant | Fractured.cause: the stress solver: a part that could no longer hold its own weight. |
| [`CAUSE_SHATTER`](#const-CAUSE_SHATTER) | constant | Fractured.cause: a chunk with children (the fracture block's `children`) took three times the damage that breaks a bond of its cross-section, and broke into them: they fly free, each its own body, leaving a hole. |
| [`MATERIAL_BRICK`](#const-MATERIAL_BRICK) | constant | og.destruction.material: brickwork in mortar (no-tension masonry: stands by its weight, steps back over its piers). |
| [`MATERIAL_CONCRETE`](#const-MATERIAL_CONCRETE) | constant | og.destruction.material: plain concrete (strong, breaks in chunky lumps, holds what is left). |
| [`MATERIAL_WOOD`](#const-MATERIAL_WOOD) | constant | og.destruction.material: timber boards (light and weak across the grain: splinters where it lands or is struck). |
| [`MATERIAL_GLASS`](#const-MATERIAL_GLASS) | constant | og.destruction.material: a glass pane (shatters at a touch, no stress solver). |
| [`MATERIAL_STONE`](#const-MATERIAL_STONE) | constant | og.destruction.material: dry-laid stone (masonry with no mortar: no tension, heavier and tougher than brick). |
| [`CAUSE_FRACTURE`](#const-CAUSE_FRACTURE) | constant | Fractured.cause: og.destruction.fracture. |
| [`og.Destructible`](#record-Destructible) | component | Makes the entity's og.Model, a model cooked with a `fracture` block, break into its chunks. A prop that stands (no og.RigidBody, or a static one) is its chunks from the first tick: each a static og.Collider (the hull of its piece) on an entity with og.Chunk, and the entity's own og.Collider and og.RigidBody are removed, so hits land on the chunks. A prop that moves (a dynamic or kinematic og.RigidBody) collides as itself until its first break, when its chunks take its place, moving as it moved, and it loses its og.Collider, og.RigidBody and og.Velocity. Until the first break the model is drawn whole; from it the model is hidden and each chunk draws its piece of it (with og.Model's tint, variant and shadows). Anchored islands stay static, the rest fall. |
| [`og.Chunk`](#record-Chunk) | component | One chunk of an og.Destructible, written by og.destruction when the prop fractures. A loose chunk is a dynamic body (with og.RigidBody and og.Velocity, or a child collider of its island's body); an anchored one a static collider. Drawn with its part of the destructible's model. Despawn it to clear it away. |
| [`og.Fractured`](#record-Fractured) | event | An island of chunks broke loose from a destructible during the tick (or the destructible shattered): read the tick after, for its sound, its camera shake, its score. Sent in the order the islands broke. |
| [`og.Destruction`](#record-Destruction) | resource | The world's destruction budgets. |
| [`DamageDesc`](#type-DamageDesc) | desc struct | A radial damage for og.destruction.damage. |
| [`ExplosionDesc`](#type-ExplosionDesc) | desc struct | An explosion for og.destruction.explode. |
| [`DestructibleInfo`](#type-DestructibleInfo) | struct | What a destructible holds and how much of it stands (og.destruction.info). |
| [`DestructibleMaterial`](#type-DestructibleMaterial) | struct | A material's og.Destructible fields (og.destruction.material): spread it into the component and add the material's fx, `{ ...og.destruction.material(world, MATERIAL_BRICK), breakFx }`, so a prop is tuned by what it is made of rather than by numbers in the scene. Its fields are og.Destructible's, for a prop of about a metre; override any one after the spread. |
| [`DestructionCapabilities`](#type-DestructionCapabilities) | struct | What destruction does on this device. |
| [`og.destruction.damage`](#fn-damage) | function | Damage a destructible at a point: every bond, anchor and chunk within reach takes the amount, all of it within innerRadius and falling to nothing at radius. Applied by og.destruction after the game's systems this tick (in the order called), so a bond broken now splits the prop before this tick's physics step. |
| [`og.destruction.explode`](#fn-explode) | function | An explosion: damage to every destructible in reach (as og.destruction.damage), then a push outwards to their loose chunks. Applied by og.destruction this tick, as damage. Push other bodies with og.physics.applyImpulse. |
| [`og.destruction.fracture`](#fn-fracture) | function | Break every bond of a destructible at once (this tick, as damage): its anchored chunks stay, the rest fall. |
| [`og.destruction.info`](#fn-info) | function | How much of a destructible stands. |
| [`og.destruction.material`](#fn-material) | function | A material's og.Destructible tuning (MATERIAL_*): toughness, strength, friction, tension, density and how hard a hit must be to hurt it, measured against the props of the demolition-yard example (a brick wall, a concrete pillar, a crate, a pane). Pair it with a model whose fracture block has the same `style` (brick, concrete, wood, glass). |
| [`og.destruction.capabilities`](#fn-capabilities) | function | What destruction does on this device. |

## Functions

<a id="fn-damage"></a>

### `og.destruction.damage`

Damage a destructible at a point: every bond, anchor and chunk within reach takes the amount, all of it within innerRadius and falling to nothing at radius. Applied by og.destruction after the game's systems this tick (in the order called), so a bond broken now splits the prop before this tick's physics step.

```ts
og.destruction.damage(world: World, entity: Entity, desc?: DamageDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with og.Destructible, or one of its chunks. |
| `desc` | `DamageDesc` | yes | Where, how far and how much. |

**Returns** `Result<void>`: Refused for an entity that is neither, or a point that is not finite.

Binding symbol `og_destruction_damage` · publish builds: kept

```ogt
fn damage {
  doc "Damage a destructible at a point: every bond, anchor and chunk within reach takes the amount, all of it within innerRadius and falling to nothing at radius. Applied by og.destruction after the game's systems this tick (in the order called), so a bond broken now splits the prop before this tick's physics step."
  param world World "The world."
  param entity Entity "An entity with og.Destructible, or one of its chunks."
  param desc DamageDesc "Where, how far and how much."
  returns Result.Void "Refused for an entity that is neither, or a point that is not finite."
}
```

<sub>`api.ogt` line 28831</sub>

<a id="fn-explode"></a>

### `og.destruction.explode`

An explosion: damage to every destructible in reach (as og.destruction.damage), then a push outwards to their loose chunks. Applied by og.destruction this tick, as damage. Push other bodies with og.physics.applyImpulse.

```ts
og.destruction.explode(world: World, desc?: ExplosionDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `ExplosionDesc` | yes | Its centre, reach, damage and push. |

**Returns** `Result<void>`: Refused for a point that is not finite.

Binding symbol `og_destruction_explode` · publish builds: kept

```ogt
fn explode {
  doc "An explosion: damage to every destructible in reach (as og.destruction.damage), then a push outwards to their loose chunks. Applied by og.destruction this tick, as damage. Push other bodies with og.physics.applyImpulse."
  param world World "The world."
  param desc ExplosionDesc "Its centre, reach, damage and push."
  returns Result.Void "Refused for a point that is not finite."
}
```

<sub>`api.ogt` line 28839</sub>

<a id="fn-fracture"></a>

### `og.destruction.fracture`

Break every bond of a destructible at once (this tick, as damage): its anchored chunks stay, the rest fall.

```ts
og.destruction.fracture(world: World, entity: Entity): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with og.Destructible. |

**Returns** `Result<void>`: Refused for an entity without one.

Binding symbol `og_destruction_fracture` · publish builds: kept

```ogt
fn fracture {
  doc "Break every bond of a destructible at once (this tick, as damage): its anchored chunks stay, the rest fall."
  param world World "The world."
  param entity Entity "An entity with og.Destructible."
  returns Result.Void "Refused for an entity without one."
}
```

<sub>`api.ogt` line 28846</sub>

<a id="fn-info"></a>

### `og.destruction.info`

How much of a destructible stands.

```ts
og.destruction.info(world: World, entity: Entity): Result<DestructibleInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with og.Destructible. |

**Returns** `Result<DestructibleInfo>`: Refused for an entity without one, or one whose model is not loaded or has no fracture.

Binding symbol `og_destruction_info` · publish builds: kept

```ogt
fn info {
  doc "How much of a destructible stands."
  param world World "The world."
  param entity Entity "An entity with og.Destructible."
  returns Result.DestructibleInfo "Refused for an entity without one, or one whose model is not loaded or has no fracture."
}
```

<sub>`api.ogt` line 28853</sub>

<a id="fn-material"></a>

### `og.destruction.material`

A material's og.Destructible tuning (MATERIAL_*): toughness, strength, friction, tension, density and how hard a hit must be to hurt it, measured against the props of the demolition-yard example (a brick wall, a concrete pillar, a crate, a pane). Pair it with a model whose fracture block has the same `style` (brick, concrete, wood, glass).

```ts
og.destruction.material(world: World, kind: number): Result<DestructibleMaterial>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `kind` | `number` | no | MATERIAL_*. |

**Returns** `Result<DestructibleMaterial>`: Refused for a kind that is none of MATERIAL_*.

Binding symbol `og_destruction_material` · publish builds: kept

```ogt
fn material {
  doc "A material's og.Destructible tuning (MATERIAL_*): toughness, strength, friction, tension, density and how hard a hit must be to hurt it, measured against the props of the demolition-yard example (a brick wall, a concrete pillar, a crate, a pane). Pair it with a model whose fracture block has the same `style` (brick, concrete, wood, glass)."
  param world World "The world."
  param kind u32 "MATERIAL_*."
  returns Result.DestructibleMaterial "Refused for a kind that is none of MATERIAL_*."
}
```

<sub>`api.ogt` line 28860</sub>

<a id="fn-capabilities"></a>

### `og.destruction.capabilities`

What destruction does on this device.

```ts
og.destruction.capabilities(world: World): DestructionCapabilities
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `DestructionCapabilities`.

Binding symbol `og_destruction_capabilities` · publish builds: kept

```ogt
fn capabilities {
  doc "What destruction does on this device."
  param world World "The world."
  returns DestructionCapabilities
}
```

<sub>`api.ogt` line 28867</sub>

## Components, resources and events

<a id="record-Destructible"></a>

### `og.Destructible` (component)

Makes the entity's og.Model, a model cooked with a `fracture` block, break into its chunks. A prop that stands (no og.RigidBody, or a static one) is its chunks from the first tick: each a static og.Collider (the hull of its piece) on an entity with og.Chunk, and the entity's own og.Collider and og.RigidBody are removed, so hits land on the chunks. A prop that moves (a dynamic or kinematic og.RigidBody) collides as itself until its first break, when its chunks take its place, moving as it moved, and it loses its og.Collider, og.RigidBody and og.Velocity. Until the first break the model is drawn whole; from it the model is hidden and each chunk draws its piece of it (with og.Model's tint, variant and shadows). Anchored islands stay static, the rest fall.

Scale it uniformly: a chunk's collider and mass follow og.Transform.scale's mean.

Token `og.Destructible`, registered under the schema name `og.Destructible`; its value type is `Destructible`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `toughness` | `number` | `800` | ≥ 0 | `newtonSecondPerM2` | tunable | How much damage a square metre of bond takes before it breaks: an impulse of that many newton-seconds through 1 m² (a hit spreads over impactRadius). Glass about 50, wood 300, brick 800, concrete 2000. |
| `strength` | `number` | `30000` | ≥ 0 | `pascal` | tunable | The stress its bonds hold in tension and shear before they weaken (compression holds thirty times as much, shear more for the compression across it (friction), and a bond breaks outright at twice the stress): low, an overhang falls and a wall shot out at its foot comes down; high, it holds whatever is left. 0 turns the stress solver off. |
| `friction` | `number` | `0.7` | 0 … 4 | — | tunable | How much more shear a bond holds for the compression across it, per pascal (Mohr–Coulomb, the tangent of the friction angle): a joint pressed by what stands on it holds where an unloaded one slips, so a wall shot out at its foot keeps its loaded piers and drops what hung over the hole, cracking in steps along its courses. 0 is shear at strength alone. |
| `tension` | `number` | `1` | 0 … 1 | — | tunable | The stress its bonds hold pulled apart, as a fraction of strength (shear and compression keep strength's). Masonry and dry stone hold almost none: mortar cracks long before brick does, so brickwork stands by its weight and friction (Heyman's no-tension masonry, the premise of masonry limit analysis), and at about 0.1 no block is left hanging from the course above over air, and a wall shot out at its foot steps back course by course over its piers. At 0.1 or below (with a strength above 0) a block of what stands also needs its weight within the middle third of what bears it from below, the blocks under it (a cracked bed joint bears as well as a sound one) or its foundation: one held only from its side, or by a tie-in into the wall beside it, comes loose whole, and the blocks it bore are judged again. 1 is tension at strength. |
| `density` | `number` | `1000` | ≥ 1 | `kgPerM3` | tunable | The chunks' density: their mass as bodies and their weight in the stress solver. |
| `impactThreshold` | `number` | `30` | ≥ 0 | `newtonSecond` | tunable | A collision's impulse (og.CollisionStarted.impulse) that does no damage: what is above it is the damage of the hit, and, on a prop that stood still, the push the pieces it breaks loose take along the striker's path (shared by falloff, none faster than the striker closed), so a ball through a pane carries its shards on; a moving prop's pieces keep the velocity its hit left it. |
| `impactRadius` | `number` | `0.4` | ≥ 0.01 | `m` | tunable | How far a hit's damage reaches round its point (all of it within a third of this, falling to nothing at it). |
| `anchored` | `boolean` | `true` | — | — | — | Whether its anchors (the chunks on its floor) hold it to the world; false, every island falls. |
| `breakFx` | `ParticleEmitter` | `0` | — | — | — | A one-shot emitter (og.load.emitter, `looping false`) burst where each island breaks loose, at its broken faces: the material's own, a lit dust puff for stone and brick, splinters and a little dust for wood, shards and glints for glass. 0 none. |
| `hitFx` | `ParticleEmitter` | `0` | — | — | — | A one-shot emitter burst at each hit that damages it: the material's own, grit and a puff off stone, splinters off wood, glints off glass; sparks only off metal, and a few. 0 none. |
| `debris` | `ParticleEmitter` | `0` | — | — | — | Fine debris, as visuals only: where a chunk shatters, its pieces smaller than og.Destruction.debrisVolume never become bodies. They fly on the GPU instead, each its own shape and materials, from where and how fast it broke off (the blast's push included), each throwing four to six thick rubble chips of uneven size in the prop's cut-face material, tumbling and landing on the colliders that are boxes, spheres and terrains, and this mesh-particle emitter (`render mesh`, best with `blend alpha`, `collision` and `lit true`) bursts its finer grains there too. They never touch the simulation, so the world is the same on every device, and a device without the GPU layer draws none of them (og.destruction.capabilities' visualDebris). They last og.Destruction.debrisLifetime. 0: every piece is a body. |
| `fractured` | `boolean` | `false` | — | — | read only | Written: true from its first break (the model hidden, its chunks drawn). |
| `broken` | `number` | `0` | — | — | read only | Written: its bonds broken so far. |
| `loose` | `number` | `0` | — | — | read only | Written: its chunks that have broken loose and still exist. |

```ogt
component Destructible {
  doc "Makes the entity's og.Model, a model cooked with a `fracture` block, break into its chunks. A prop that stands (no og.RigidBody, or a static one) is its chunks from the first tick: each a static og.Collider (the hull of its piece) on an entity with og.Chunk, and the entity's own og.Collider and og.RigidBody are removed, so hits land on the chunks. A prop that moves (a dynamic or kinematic og.RigidBody) collides as itself until its first break, when its chunks take its place, moving as it moved, and it loses its og.Collider, og.RigidBody and og.Velocity. Until the first break the model is drawn whole; from it the model is hidden and each chunk draws its piece of it (with og.Model's tint, variant and shadows). Anchored islands stay static, the rest fall."
  doc "Scale it uniformly: a chunk's collider and mass follow og.Transform.scale's mean."
  field toughness f32 "How much damage a square metre of bond takes before it breaks: an impulse of that many newton-seconds through 1 m² (a hit spreads over impactRadius). Glass about 50, wood 300, brick 800, concrete 2000." {
    default 800
    min 0
    unit newtonSecondPerM2
    tunable true
  }
  field strength f32 "The stress its bonds hold in tension and shear before they weaken (compression holds thirty times as much, shear more for the compression across it (friction), and a bond breaks outright at twice the stress): low, an overhang falls and a wall shot out at its foot comes down; high, it holds whatever is left. 0 turns the stress solver off." {
    default 30000
    min 0
    unit pascal
    tunable true
  }
  field friction f32 "How much more shear a bond holds for the compression across it, per pascal (Mohr–Coulomb, the tangent of the friction angle): a joint pressed by what stands on it holds where an unloaded one slips, so a wall shot out at its foot keeps its loaded piers and drops what hung over the hole, cracking in steps along its courses. 0 is shear at strength alone." {
    default 0.7
    min 0
    max 4
    tunable true
  }
  field tension f32 "The stress its bonds hold pulled apart, as a fraction of strength (shear and compression keep strength's). Masonry and dry stone hold almost none: mortar cracks long before brick does, so brickwork stands by its weight and friction (Heyman's no-tension masonry, the premise of masonry limit analysis), and at about 0.1 no block is left hanging from the course above over air, and a wall shot out at its foot steps back course by course over its piers. At 0.1 or below (with a strength above 0) a block of what stands also needs its weight within the middle third of what bears it from below, the blocks under it (a cracked bed joint bears as well as a sound one) or its foundation: one held only from its side, or by a tie-in into the wall beside it, comes loose whole, and the blocks it bore are judged again. 1 is tension at strength." {
    default 1
    min 0
    max 1
    tunable true
  }
  field density f32 "The chunks' density: their mass as bodies and their weight in the stress solver." {
    default 1000
    min 1
    unit kgPerM3
    tunable true
  }
  field impactThreshold f32 "A collision's impulse (og.CollisionStarted.impulse) that does no damage: what is above it is the damage of the hit, and, on a prop that stood still, the push the pieces it breaks loose take along the striker's path (shared by falloff, none faster than the striker closed), so a ball through a pane carries its shards on; a moving prop's pieces keep the velocity its hit left it." {
    default 30
    min 0
    unit newtonSecond
    tunable true
  }
  field impactRadius f32 "How far a hit's damage reaches round its point (all of it within a third of this, falling to nothing at it)." {
    default 0.4
    min 0.01
    unit m
    tunable true
  }
  field anchored bool "Whether its anchors (the chunks on its floor) hold it to the world; false, every island falls." {
    default true
  }
  field breakFx ParticleEmitter "A one-shot emitter (og.load.emitter, `looping false`) burst where each island breaks loose, at its broken faces: the material's own, a lit dust puff for stone and brick, splinters and a little dust for wood, shards and glints for glass. 0 none."
  field hitFx ParticleEmitter "A one-shot emitter burst at each hit that damages it: the material's own, grit and a puff off stone, splinters off wood, glints off glass; sparks only off metal, and a few. 0 none."
  field debris ParticleEmitter "Fine debris, as visuals only: where a chunk shatters, its pieces smaller than og.Destruction.debrisVolume never become bodies. They fly on the GPU instead, each its own shape and materials, from where and how fast it broke off (the blast's push included), each throwing four to six thick rubble chips of uneven size in the prop's cut-face material, tumbling and landing on the colliders that are boxes, spheres and terrains, and this mesh-particle emitter (`render mesh`, best with `blend alpha`, `collision` and `lit true`) bursts its finer grains there too. They never touch the simulation, so the world is the same on every device, and a device without the GPU layer draws none of them (og.destruction.capabilities' visualDebris). They last og.Destruction.debrisLifetime. 0: every piece is a body."
  field fractured bool "Written: true from its first break (the model hidden, its chunks drawn)." {
    readonly true
  }
  field broken u32 "Written: its bonds broken so far." {
    readonly true
  }
  field loose u32 "Written: its chunks that have broken loose and still exist." {
    readonly true
  }
}
```

<sub>`api.ogt` line 28597</sub>

<a id="record-Chunk"></a>

### `og.Chunk` (component)

One chunk of an og.Destructible, written by og.destruction when the prop fractures. A loose chunk is a dynamic body (with og.RigidBody and og.Velocity, or a child collider of its island's body); an anchored one a static collider. Drawn with its part of the destructible's model. Despawn it to clear it away.

Token `og.Chunk`, registered under the schema name `og.Chunk`; its value type is `Chunk`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `destructible` | `Entity` | `0` | — | — | read only | The og.Destructible it belongs to. |
| `chunk` | `number` | `0` | — | — | read only | Its index in the fracture asset. |
| `level` | `number` | `0` | — | — | read only | 1 a support chunk, 2 a chunk's child. |
| `body` | `Entity` | `0` | — | — | read only | The entity whose og.RigidBody moves it: its own, its island's body, or 0 while anchored. |
| `looseTick` | `number` | `4294967295` | — | — | read only | The tick it broke loose; 4294967295 while anchored. |

```ogt
component Chunk {
  doc "One chunk of an og.Destructible, written by og.destruction when the prop fractures. A loose chunk is a dynamic body (with og.RigidBody and og.Velocity, or a child collider of its island's body); an anchored one a static collider. Drawn with its part of the destructible's model. Despawn it to clear it away."
  field destructible Entity "The og.Destructible it belongs to." {
    readonly true
  }
  field chunk u32 "Its index in the fracture asset." {
    readonly true
  }
  field level u32 "1 a support chunk, 2 a chunk's child." {
    readonly true
  }
  field body Entity "The entity whose og.RigidBody moves it: its own, its island's body, or 0 while anchored." {
    readonly true
  }
  field looseTick u32 "The tick it broke loose; 4294967295 while anchored." {
    default 4294967295
    readonly true
  }
}
```

<sub>`api.ogt` line 28659</sub>

<a id="record-Fractured"></a>

### `og.Fractured` (event)

An island of chunks broke loose from a destructible during the tick (or the destructible shattered): read the tick after, for its sound, its camera shake, its score. Sent in the order the islands broke.

Token `og.Fractured`, registered under the schema name `og.Fractured`; its value type is `Fractured`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The og.Destructible. |
| `body` | `Entity` | `0` | — | — | — | The island's body: the entity with its og.RigidBody. |
| `chunks` | `number` | `0` | — | — | — | Its chunks. |
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `world` | Where it broke: the centre of its broken bonds. |
| `volume` | `number` | `0` | — | `m3` | — | Its volume. |
| `mass` | `number` | `0` | — | `kg` | — | Its mass. |
| `cause` | `number` | `0` | — | — | — | What broke it: one of the CAUSE_* codes. |

```ogt
event Fractured {
  doc "An island of chunks broke loose from a destructible during the tick (or the destructible shattered): read the tick after, for its sound, its camera shake, its score. Sent in the order the islands broke."
  field entity Entity "The og.Destructible."
  field body Entity "The island's body: the entity with its og.RigidBody."
  field chunks u32 "Its chunks."
  field point Vector3 "Where it broke: the centre of its broken bonds." {
    unit m
    worldspace world
  }
  field volume f32 "Its volume." {
    unit m3
  }
  field mass f32 "Its mass." {
    unit kg
  }
  field cause u32 "What broke it: one of the CAUSE_* codes."
}
```

<sub>`api.ogt` line 28679</sub>

<a id="record-Destruction"></a>

### `og.Destruction` (resource)

The world's destruction budgets.

Token `og.Destruction`, registered under the schema name `og.Destruction`; its value type is `Destruction`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `maxChunks` | `number` | `256` | 1 … 4096 | — | tunable | The most loose chunks alive at once, over every destructible: past it the oldest loose islands (the smallest first among those that broke in the same tick) despawn, each with every chunk it carries. |
| `debrisVolume` | `number` | `0.004` | ≥ 0 | `m3` | tunable | A loose island smaller than this is debris: it despawns debrisLifetime after it broke loose. |
| `debrisLifetime` | `number` | `8` | ≥ 0 | `s` | tunable | How long debris lasts (0: until maxChunks takes it). |
| `fxLifetime` | `number` | `3` | ≥ 0 | `s` | — | How long a breakFx or hitFx burst's entity lasts before og.destruction despawns it. |
| `stressIterations` | `number` | `0` | 0 … 1000 | — | — | The stress solver's iterations when a destructible's support changes (Blast's maxIter). 0, the default, solves until it converges (at most six times its bonds), so a large structure's weight reaches every bond it rests on; a cap is cheaper on a slow device, but an unconverged solve loads distant bonds too little, and what should fall can stand. |
| `stressBudget` | `number` | `200000` | ≥ 0 | — | tunable | The stress solver's work in one tick, across every destructible, in bond-iterations (one solver iteration over one bond). A solve that would start past it waits for the next tick, so a tick's worst case is this plus one solve (six iterations a bond) whatever comes down at once; the order is the destructibles' entity order, so a replay waits the same way. The 200-chunk brick wall's largest solve takes about 41000. 0 sets no limit. |

```ogt
resource Destruction {
  doc "The world's destruction budgets."
  field maxChunks u32 "The most loose chunks alive at once, over every destructible: past it the oldest loose islands (the smallest first among those that broke in the same tick) despawn, each with every chunk it carries." {
    default 256
    min 1
    max 4096
    tunable true
  }
  field debrisVolume f32 "A loose island smaller than this is debris: it despawns debrisLifetime after it broke loose." {
    default 0.004
    min 0
    unit m3
    tunable true
  }
  field debrisLifetime f32 "How long debris lasts (0: until maxChunks takes it)." {
    default 8
    min 0
    unit s
    tunable true
  }
  field fxLifetime f32 "How long a breakFx or hitFx burst's entity lasts before og.destruction despawns it." {
    default 3
    min 0
    unit s
  }
  field stressIterations u32 "The stress solver's iterations when a destructible's support changes (Blast's maxIter). 0, the default, solves until it converges (at most six times its bonds), so a large structure's weight reaches every bond it rests on; a cap is cheaper on a slow device, but an unconverged solve loads distant bonds too little, and what should fall can stand." {
    default 0
    min 0
    max 1000
  }
  field stressBudget u32 "The stress solver's work in one tick, across every destructible, in bond-iterations (one solver iteration over one bond). A solve that would start past it waits for the next tick, so a tick's worst case is this plus one solve (six iterations a bond) whatever comes down at once; the order is the destructibles' entity order, so a replay waits the same way. The 200-chunk brick wall's largest solve takes about 41000. 0 sets no limit." {
    default 200000
    min 0
    tunable true
  }
}
```

<sub>`api.ogt` line 28697</sub>

## Types

<a id="type-DamageDesc"></a>

### `DamageDesc` (desc struct)

A radial damage for og.destruction.damage.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where it lands, in world space. |
| `radius` | `number` | `0.4` | ≥ 0 | `m` | — | How far it reaches; it falls linearly to nothing here from innerRadius. |
| `innerRadius` | `number` | `0.1` | ≥ 0 | `m` | — | Within this the bonds take the whole amount. |
| `amount` | `number` | `500` | ≥ 0 | `newtonSecond` | — | The damage, as an impulse through a bond (Destructible.toughness says how much a square metre takes). |
| `impulse` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `newtonSecond` | — | The momentum the blow carries, along its direction (a hammer, a wrecking ball, a shot): the pieces it breaks loose share it by its falloff and fly on, as a hit's do. Zero, the default, crushes without pushing, and what it breaks out of a standing prop can stay standing on what holds it. |

```ogt
struct DamageDesc {
  doc "A radial damage for og.destruction.damage."
  desc true
  field point Vector3 "Where it lands, in world space." {
    unit m
  }
  field radius f32 "How far it reaches; it falls linearly to nothing here from innerRadius." {
    default 0.4
    min 0
    unit m
  }
  field innerRadius f32 "Within this the bonds take the whole amount." {
    default 0.1
    min 0
    unit m
  }
  field amount f32 "The damage, as an impulse through a bond (Destructible.toughness says how much a square metre takes)." {
    default 500
    min 0
    unit newtonSecond
  }
  field impulse Vector3 "The momentum the blow carries, along its direction (a hammer, a wrecking ball, a shot): the pieces it breaks loose share it by its falloff and fly on, as a hit's do. Zero, the default, crushes without pushing, and what it breaks out of a standing prop can stay standing on what holds it." {
    unit newtonSecond
  }
}
```

<sub>`api.ogt` line 28734</sub>

<a id="type-ExplosionDesc"></a>

### `ExplosionDesc` (desc struct)

An explosion for og.destruction.explode.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Its centre, in world space. |
| `radius` | `number` | `3` | ≥ 0 | `m` | — | How far it reaches. |
| `innerRadius` | `number` | `0.5` | ≥ 0 | `m` | — | Within this everything takes the whole damage and push. |
| `damage` | `number` | `3000` | ≥ 0 | `newtonSecond` | — | The damage to every destructible's bonds in reach, as og.destruction.damage's amount. |
| `impulse` | `number` | `2000` | ≥ 0 | `newtonSecondPerM2` | — | The push outwards on every loose piece in reach, per square metre of the face it shows the blast (the two-thirds power of its volume), at its centre, falling off as the damage does: a blast pushes on surfaces, so a lone brick flies faster than a run of bricks, but not by their count. The pieces it drives apart pull on the bonds between them, so a run of bricks by the charge breaks up rather than flying off whole. Brick by a demolition charge about 2500. |

```ogt
struct ExplosionDesc {
  doc "An explosion for og.destruction.explode."
  desc true
  field point Vector3 "Its centre, in world space." {
    unit m
  }
  field radius f32 "How far it reaches." {
    default 3
    min 0
    unit m
  }
  field innerRadius f32 "Within this everything takes the whole damage and push." {
    default 0.5
    min 0
    unit m
  }
  field damage f32 "The damage to every destructible's bonds in reach, as og.destruction.damage's amount." {
    default 3000
    min 0
    unit newtonSecond
  }
  field impulse f32 "The push outwards on every loose piece in reach, per square metre of the face it shows the blast (the two-thirds power of its volume), at its centre, falling off as the damage does: a blast pushes on surfaces, so a lone brick flies faster than a run of bricks, but not by their count. The pieces it drives apart pull on the bonds between them, so a run of bricks by the charge breaks up rather than flying off whole. Brick by a demolition charge about 2500." {
    default 2000
    min 0
    unit newtonSecondPerM2
  }
}
```

<sub>`api.ogt` line 28760</sub>

<a id="type-DestructibleInfo"></a>

### `DestructibleInfo` (struct)

What a destructible holds and how much of it stands (og.destruction.info).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `chunks` | `number` | `0` | — | — | — | Its support chunks. |
| `children` | `number` | `0` | — | — | — | Their children, all told. |
| `bonds` | `number` | `0` | — | — | — | The bonds of its support graph. |
| `intact` | `number` | `0` | — | — | — | The bonds not yet broken. |
| `islands` | `number` | `0` | — | — | — | Its islands now. |
| `anchored` | `number` | `0` | — | — | — | Its support chunks still held to the world. |
| `health` | `number` | `0` | 0 … 1 | — | — | Its bonds' remaining health over their total, 1 intact to 0 rubble. |
| `debris` | `number` | `0` | — | — | — | Its fine pieces on the GPU now (og.Destructible.debris): broken off within og.Destruction.debrisLifetime, whether or not this device draws them. |

```ogt
struct DestructibleInfo {
  doc "What a destructible holds and how much of it stands (og.destruction.info)."
  field chunks u32 "Its support chunks."
  field children u32 "Their children, all told."
  field bonds u32 "The bonds of its support graph."
  field intact u32 "The bonds not yet broken."
  field islands u32 "Its islands now."
  field anchored u32 "Its support chunks still held to the world."
  field health f32 "Its bonds' remaining health over their total, 1 intact to 0 rubble." {
    min 0
    max 1
  }
  field debris u32 "Its fine pieces on the GPU now (og.Destructible.debris): broken off within og.Destruction.debrisLifetime, whether or not this device draws them."
}
```

<sub>`api.ogt` line 28788</sub>

<a id="type-DestructibleMaterial"></a>

### `DestructibleMaterial` (struct)

A material's og.Destructible fields (og.destruction.material): spread it into the component and add the material's fx, `{ ...og.destruction.material(world, MATERIAL_BRICK), breakFx }`, so a prop is tuned by what it is made of rather than by numbers in the scene. Its fields are og.Destructible's, for a prop of about a metre; override any one after the spread.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `toughness` | `number` | `0` | — | `newtonSecondPerM2` | — | og.Destructible.toughness. |
| `strength` | `number` | `0` | — | `pascal` | — | og.Destructible.strength. |
| `friction` | `number` | `0` | — | — | — | og.Destructible.friction. |
| `tension` | `number` | `0` | — | — | — | og.Destructible.tension. |
| `density` | `number` | `0` | — | `kgPerM3` | — | og.Destructible.density. |
| `impactThreshold` | `number` | `0` | — | `newtonSecond` | — | og.Destructible.impactThreshold. |
| `impactRadius` | `number` | `0` | — | `m` | — | og.Destructible.impactRadius. |

```ogt
struct DestructibleMaterial {
  doc "A material's og.Destructible fields (og.destruction.material): spread it into the component and add the material's fx, `{ ...og.destruction.material(world, MATERIAL_BRICK), breakFx }`, so a prop is tuned by what it is made of rather than by numbers in the scene. Its fields are og.Destructible's, for a prop of about a metre; override any one after the spread."
  field toughness f32 "og.Destructible.toughness." {
    unit newtonSecondPerM2
  }
  field strength f32 "og.Destructible.strength." {
    unit pascal
  }
  field friction f32 "og.Destructible.friction."
  field tension f32 "og.Destructible.tension."
  field density f32 "og.Destructible.density." {
    unit kgPerM3
  }
  field impactThreshold f32 "og.Destructible.impactThreshold." {
    unit newtonSecond
  }
  field impactRadius f32 "og.Destructible.impactRadius." {
    unit m
  }
}
```

<sub>`api.ogt` line 28803</sub>

<a id="type-DestructionCapabilities"></a>

### `DestructionCapabilities` (struct)

What destruction does on this device.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `visualDebris` | `boolean` | `false` | — | — | — | Whether og.Destructible.debris pieces and bursts are simulated and drawn on the GPU here (a renderer with compute passes: og.render.capabilities' compute); false draws none of them. The bodies are the same either way: pieces given to `debris` are never bodies, on any device, so a game never depends on it. |
| `maxVisualDebris` | `number` | `0` | — | — | — | The most fine pieces the GPU layer holds at once: past it the oldest go first (og.Destruction.debrisLifetime takes them sooner). |
| `visualDebrisContacts` | `string` | `""` | — | — | — | What the GPU pieces collide with: `world` (the colliders that are boxes, spheres and terrains); they pass through each other and through the chunks that are bodies. |

```ogt
struct DestructionCapabilities {
  doc "What destruction does on this device."
  field visualDebris bool "Whether og.Destructible.debris pieces and bursts are simulated and drawn on the GPU here (a renderer with compute passes: og.render.capabilities' compute); false draws none of them. The bodies are the same either way: pieces given to `debris` are never bodies, on any device, so a game never depends on it."
  field maxVisualDebris u32 "The most fine pieces the GPU layer holds at once: past it the oldest go first (og.Destruction.debrisLifetime takes them sooner)."
  field visualDebrisContacts String "What the GPU pieces collide with: `world` (the colliders that are boxes, spheres and terrains); they pass through each other and through the chunks that are bodies."
}
```

<sub>`api.ogt` line 28824</sub>

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

<sub>`api.ogt` line 28585</sub>

```ogt
const CAUSE_DAMAGE u32 1 "Fractured.cause: og.destruction.damage."
```

<sub>`api.ogt` line 28586</sub>

```ogt
const CAUSE_EXPLOSION u32 2 "Fractured.cause: og.destruction.explode."
```

<sub>`api.ogt` line 28587</sub>

```ogt
const CAUSE_STRESS u32 3 "Fractured.cause: the stress solver: a part that could no longer hold its own weight."
```

<sub>`api.ogt` line 28588</sub>

```ogt
const CAUSE_SHATTER u32 4 "Fractured.cause: a chunk with children (the fracture block's `children`) took three times the damage that breaks a bond of its cross-section, and broke into them: they fly free, each its own body, leaving a hole."
```

<sub>`api.ogt` line 28589</sub>

```ogt
const MATERIAL_BRICK u32 0 "og.destruction.material: brickwork in mortar (no-tension masonry: stands by its weight, steps back over its piers)."
```

<sub>`api.ogt` line 28590</sub>

```ogt
const MATERIAL_CONCRETE u32 1 "og.destruction.material: plain concrete (strong, breaks in chunky lumps, holds what is left)."
```

<sub>`api.ogt` line 28591</sub>

```ogt
const MATERIAL_WOOD u32 2 "og.destruction.material: timber boards (light and weak across the grain: splinters where it lands or is struck)."
```

<sub>`api.ogt` line 28592</sub>

```ogt
const MATERIAL_GLASS u32 3 "og.destruction.material: a glass pane (shatters at a touch, no stress solver)."
```

<sub>`api.ogt` line 28593</sub>

```ogt
const MATERIAL_STONE u32 4 "og.destruction.material: dry-laid stone (masonry with no mortar: no tension, heavier and tougher than brick)."
```

<sub>`api.ogt` line 28594</sub>

```ogt
const CAUSE_FRACTURE u32 5 "Fractured.cause: og.destruction.fracture."
```

<sub>`api.ogt` line 28595</sub>
