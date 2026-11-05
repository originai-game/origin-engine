<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `destruction` module

Props that break: walls, crates, pillars, glass. A model cooked with a `fracture { }` block (at the top of a `type model`, or in a glTF's import settings) is cut by the cook into Voronoi chunks bonded where they touch, and the chunks resting on its floor are anchored to the world. Give the entity that draws it (its og.Model) an og.Destructible: hits, og.destruction.damage and og.destruction.explode wear the bonds down, a broken bond splits the prop into islands, an island anchored to the world stays put and every other island falls as one rigid body, and a stress solver (NVIDIA Blast's) breaks what can no longer carry its own weight, so a wall shot out at its foot comes down.

**Deterministic.** Damage, bonds, islands and the stress solver are fixed f32 arithmetic in the tick (og.destruction runs after the game's systems and before og.physics), the chunks are world entities and the bonds' health is in a state block: snapshots, restores, replays and the world hash cover the whole fracture, on both hosts.

**What a game sees.** og.Fractured, the tick after, for every island that breaks loose (play a sound by its volume, shake the camera); each loose chunk is an entity with og.Chunk (its body is a dynamic og.RigidBody: query it, push it, despawn it). Dust and sparks: the og.Destructible's `dust` and `sparks` emitters burst at each break. Budgets (og.Destruction): a loose island (a body and the chunks it carries) smaller than `debrisVolume` despawns after `debrisLifetime`, and past `maxChunks` loose chunks the oldest islands go first.

3D profile only. The chunks of a dynamic prop (a crate with an og.RigidBody) fly with its velocity; a prop without a body, or with a static one, stands on its anchors.

- **TypeScript**: functions are the namespace `og.destruction`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-destruction`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module destruction {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 28501</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`CAUSE_IMPACT`](#const-CAUSE_IMPACT) | constant | Fractured.cause: a collision's impulse (og.CollisionStarted) past the prop's impactThreshold. |
| [`CAUSE_DAMAGE`](#const-CAUSE_DAMAGE) | constant | Fractured.cause: og.destruction.damage. |
| [`CAUSE_EXPLOSION`](#const-CAUSE_EXPLOSION) | constant | Fractured.cause: og.destruction.explode. |
| [`CAUSE_STRESS`](#const-CAUSE_STRESS) | constant | Fractured.cause: the stress solver: a part that could no longer hold its own weight. |
| [`CAUSE_SHATTER`](#const-CAUSE_SHATTER) | constant | Fractured.cause: a chunk with children (the fracture block's `children`) took three times the damage that breaks a bond of its cross-section, and broke into them: they fly free, each its own body, leaving a hole. |
| [`CAUSE_FRACTURE`](#const-CAUSE_FRACTURE) | constant | Fractured.cause: og.destruction.fracture. |
| [`og.Destructible`](#record-Destructible) | component | Makes the entity's og.Model, a model cooked with a `fracture` block, break into its chunks. A prop that stands (no og.RigidBody, or a static one) is its chunks from the first tick: each a static og.Collider (the hull of its piece) on an entity with og.Chunk, and the entity's own og.Collider and og.RigidBody are removed, so hits land on the chunks. A prop that moves (a dynamic or kinematic og.RigidBody) collides as itself until its first break, when its chunks take its place, moving as it moved, and it loses its og.Collider, og.RigidBody and og.Velocity. Until the first break the model is drawn whole; from it the model is hidden and each chunk draws its piece of it (with og.Model's tint, variant and shadows). Anchored islands stay static, the rest fall. |
| [`og.Chunk`](#record-Chunk) | component | One chunk of an og.Destructible, written by og.destruction when the prop fractures. A loose chunk is a dynamic body (with og.RigidBody and og.Velocity, or a child collider of its island's body); an anchored one a static collider. Drawn with its part of the destructible's model. Despawn it to clear it away. |
| [`og.Fractured`](#record-Fractured) | event | An island of chunks broke loose from a destructible during the tick (or the destructible shattered): read the tick after, for its sound, its camera shake, its score. Sent in the order the islands broke. |
| [`og.Destruction`](#record-Destruction) | resource | The world's destruction budgets. |
| [`DamageDesc`](#type-DamageDesc) | desc struct | A radial damage for og.destruction.damage. |
| [`ExplosionDesc`](#type-ExplosionDesc) | desc struct | An explosion for og.destruction.explode. |
| [`DestructibleInfo`](#type-DestructibleInfo) | struct | What a destructible holds and how much of it stands (og.destruction.info). |
| [`DestructionCapabilities`](#type-DestructionCapabilities) | struct | What destruction does on this device. |
| [`og.destruction.damage`](#fn-damage) | function | Damage a destructible at a point: every bond, anchor and chunk within reach takes the amount, all of it within innerRadius and falling to nothing at radius. Applied by og.destruction after the game's systems this tick (in the order called), so a bond broken now splits the prop before this tick's physics step. |
| [`og.destruction.explode`](#fn-explode) | function | An explosion: damage to every destructible in reach (as og.destruction.damage), then a push outwards to their loose chunks. Applied by og.destruction this tick, as damage. Push other bodies with og.physics.applyImpulse. |
| [`og.destruction.fracture`](#fn-fracture) | function | Break every bond of a destructible at once (this tick, as damage): its anchored chunks stay, the rest fall. |
| [`og.destruction.info`](#fn-info) | function | How much of a destructible stands. |
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

<sub>`api.ogt` line 28711</sub>

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

<sub>`api.ogt` line 28719</sub>

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

<sub>`api.ogt` line 28726</sub>

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

<sub>`api.ogt` line 28733</sub>

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

<sub>`api.ogt` line 28740</sub>

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
| `density` | `number` | `1000` | ≥ 1 | `kgPerM3` | tunable | The chunks' density: their mass as bodies and their weight in the stress solver. |
| `impactThreshold` | `number` | `30` | ≥ 0 | `newtonSecond` | tunable | A collision's impulse (og.CollisionStarted.impulse) that does no damage: what is above it is the damage of the hit. |
| `impactRadius` | `number` | `0.4` | ≥ 0.01 | `m` | tunable | How far a hit's damage reaches round its point (all of it within a third of this, falling to nothing at it). |
| `anchored` | `boolean` | `true` | — | — | — | Whether its anchors (the chunks on its floor) hold it to the world; false, every island falls. |
| `dust` | `ParticleEmitter` | `0` | — | — | — | An emitter (og.load.emitter, `looping false`) burst where each island breaks loose, at its broken faces: dust, splinters, glass grit. 0 none. |
| `sparks` | `ParticleEmitter` | `0` | — | — | — | An emitter burst at each hit that damages it (sparks off metal, chips off stone). 0 none. |
| `debris` | `ParticleEmitter` | `0` | — | — | — | Fine debris, as visuals only: a mesh-particle emitter (`render mesh`, best with `collision` and `lit true`) burst where a chunk shatters, in place of its pieces smaller than og.Destruction.debrisVolume, which then never become bodies. It is simulated and drawn on the GPU (og.destruction.capabilities' visualDebris) and never touches the simulation, so the world is the same on every device. 0: every piece is a body. |
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
  field density f32 "The chunks' density: their mass as bodies and their weight in the stress solver." {
    default 1000
    min 1
    unit kgPerM3
    tunable true
  }
  field impactThreshold f32 "A collision's impulse (og.CollisionStarted.impulse) that does no damage: what is above it is the damage of the hit." {
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
  field dust ParticleEmitter "An emitter (og.load.emitter, `looping false`) burst where each island breaks loose, at its broken faces: dust, splinters, glass grit. 0 none."
  field sparks ParticleEmitter "An emitter burst at each hit that damages it (sparks off metal, chips off stone). 0 none."
  field debris ParticleEmitter "Fine debris, as visuals only: a mesh-particle emitter (`render mesh`, best with `collision` and `lit true`) burst where a chunk shatters, in place of its pieces smaller than og.Destruction.debrisVolume, which then never become bodies. It is simulated and drawn on the GPU (og.destruction.capabilities' visualDebris) and never touches the simulation, so the world is the same on every device. 0: every piece is a body."
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

<sub>`api.ogt` line 28515</sub>

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

<sub>`api.ogt` line 28571</sub>

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

<sub>`api.ogt` line 28591</sub>

<a id="record-Destruction"></a>

### `og.Destruction` (resource)

The world's destruction budgets.

Token `og.Destruction`, registered under the schema name `og.Destruction`; its value type is `Destruction`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `maxChunks` | `number` | `256` | 1 … 4096 | — | tunable | The most loose chunks alive at once, over every destructible: past it the oldest loose islands (the smallest first among those that broke in the same tick) despawn, each with every chunk it carries. |
| `debrisVolume` | `number` | `0.004` | ≥ 0 | `m3` | tunable | A loose island smaller than this is debris: it despawns debrisLifetime after it broke loose. |
| `debrisLifetime` | `number` | `8` | ≥ 0 | `s` | tunable | How long debris lasts (0: until maxChunks takes it). |
| `fxLifetime` | `number` | `3` | ≥ 0 | `s` | — | How long a dust or sparks burst's entity lasts before og.destruction despawns it. |
| `stressIterations` | `number` | `48` | 1 … 1000 | — | — | The stress solver's iterations a destructible a tick (Blast's maxIter): more settles larger structures in one tick. |

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
  field fxLifetime f32 "How long a dust or sparks burst's entity lasts before og.destruction despawns it." {
    default 3
    min 0
    unit s
  }
  field stressIterations u32 "The stress solver's iterations a destructible a tick (Blast's maxIter): more settles larger structures in one tick." {
    default 48
    min 1
    max 1000
  }
}
```

<sub>`api.ogt` line 28609</sub>

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
}
```

<sub>`api.ogt` line 28641</sub>

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
| `impulse` | `number` | `200` | ≥ 0 | `newtonSecond` | — | The push outwards given to every loose chunk in reach, as an impulse at its centre, falling off as the damage does. |

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
  field impulse f32 "The push outwards given to every loose chunk in reach, as an impulse at its centre, falling off as the damage does." {
    default 200
    min 0
    unit newtonSecond
  }
}
```

<sub>`api.ogt` line 28664</sub>

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
}
```

<sub>`api.ogt` line 28692</sub>

<a id="type-DestructionCapabilities"></a>

### `DestructionCapabilities` (struct)

What destruction does on this device.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `visualDebris` | `boolean` | `false` | — | — | — | Whether og.Destructible.debris bursts are simulated and drawn on the GPU here (a renderer with compute passes: og.render.capabilities' compute); false draws none of them. The bodies are the same either way: pieces given to `debris` are never bodies, on any device, so a game never depends on it. |

```ogt
struct DestructionCapabilities {
  doc "What destruction does on this device."
  field visualDebris bool "Whether og.Destructible.debris bursts are simulated and drawn on the GPU here (a renderer with compute passes: og.render.capabilities' compute); false draws none of them. The bodies are the same either way: pieces given to `debris` are never bodies, on any device, so a game never depends on it."
}
```

<sub>`api.ogt` line 28706</sub>

## Constants

| Constant | Type | Value | Doc |
|---|---|---|---|
| <a id="const-CAUSE_IMPACT"></a>`CAUSE_IMPACT` | `number` | `0` | Fractured.cause: a collision's impulse (og.CollisionStarted) past the prop's impactThreshold. |
| <a id="const-CAUSE_DAMAGE"></a>`CAUSE_DAMAGE` | `number` | `1` | Fractured.cause: og.destruction.damage. |
| <a id="const-CAUSE_EXPLOSION"></a>`CAUSE_EXPLOSION` | `number` | `2` | Fractured.cause: og.destruction.explode. |
| <a id="const-CAUSE_STRESS"></a>`CAUSE_STRESS` | `number` | `3` | Fractured.cause: the stress solver: a part that could no longer hold its own weight. |
| <a id="const-CAUSE_SHATTER"></a>`CAUSE_SHATTER` | `number` | `4` | Fractured.cause: a chunk with children (the fracture block's `children`) took three times the damage that breaks a bond of its cross-section, and broke into them: they fly free, each its own body, leaving a hole. |
| <a id="const-CAUSE_FRACTURE"></a>`CAUSE_FRACTURE` | `number` | `5` | Fractured.cause: og.destruction.fracture. |

```ogt
const CAUSE_IMPACT u32 0 "Fractured.cause: a collision's impulse (og.CollisionStarted) past the prop's impactThreshold."
```

<sub>`api.ogt` line 28508</sub>

```ogt
const CAUSE_DAMAGE u32 1 "Fractured.cause: og.destruction.damage."
```

<sub>`api.ogt` line 28509</sub>

```ogt
const CAUSE_EXPLOSION u32 2 "Fractured.cause: og.destruction.explode."
```

<sub>`api.ogt` line 28510</sub>

```ogt
const CAUSE_STRESS u32 3 "Fractured.cause: the stress solver: a part that could no longer hold its own weight."
```

<sub>`api.ogt` line 28511</sub>

```ogt
const CAUSE_SHATTER u32 4 "Fractured.cause: a chunk with children (the fracture block's `children`) took three times the damage that breaks a bond of its cross-section, and broke into them: they fly free, each its own body, leaving a hole."
```

<sub>`api.ogt` line 28512</sub>

```ogt
const CAUSE_FRACTURE u32 5 "Fractured.cause: og.destruction.fracture."
```

<sub>`api.ogt` line 28513</sub>
