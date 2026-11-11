<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="record-Destructible"></a>

### `og.Destructible` (component)

Breaks the entity's og.Model, cooked with a `fracture` block, into its chunks (3D profile). A standing prop (no og.RigidBody, or a static one) is its chunks from the first tick: each a static og.Collider (its piece's hull) on an og.Chunk, replacing the entity's own Collider and RigidBody. A moving prop (a dynamic or kinematic body) collides as itself until its first break, when its chunks replace its Collider, RigidBody and Velocity, moving as it moved. Then each chunk draws its piece of the hidden model (its tint, variant, shadows).

Chunks' colliders and mass follow og.Transform.scale's mean.

See `physics §9`.

Also known as: breakable, destructable, destructible prop, fracture, Unreal Geometry Collection

Token `og.Destructible`, registered under the schema name `og.Destructible`; its value type is `Destructible`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `toughness` | `number` | `800` | ≥ 0 | `newtonSecondPerM2` | tunable | How much damage a square metre of bond takes before it breaks: an impulse of that many newton-seconds through 1 m² (a hit spreads over impactRadius). Glass about 50, wood 300, brick 800, concrete 2000. |
| `strength` | `number` | `30000` | ≥ 0 | `pascal` | tunable | The stress its bonds hold in tension and shear before they weaken; compression holds thirty times as much, shear more by `friction`, and a bond breaks outright at twice the stress. 0 turns the stress solver off. See `physics §9`. |
| `friction` | `number` | `0.7` | 0 … 4 | — | tunable | How much more shear a bond holds per pascal of compression across it (Mohr–Coulomb: the tangent of the friction angle). 0 is shear at strength alone. See `physics §9`. |
| `tension` | `number` | `1` | 0 … 1 | — | tunable | The stress its bonds hold pulled apart, as a fraction of strength (shear and compression keep strength's); 1 is tension at strength. At 0.1 or below, with strength above 0, a standing block also needs its weight within the middle third of what bears it from below, or it comes loose whole. See `physics §9`. |
| `density` | `number` | `1000` | ≥ 1 | `kgPerM3` | tunable | The chunks' density: their mass as bodies and their weight in the stress solver. |
| `impactThreshold` | `number` | `30` | ≥ 0 | `newtonSecond` | tunable | A collision's impulse (og.CollisionStarted.impulse) that does no damage: the excess is the hit's damage and, on a prop that stood still, the push its broken-loose pieces take along the striker's path (shared by falloff, none faster than the striker closed); a moving prop's keep their velocity. See `physics §9`. |
| `impactRadius` | `number` | `0.4` | ≥ 0.01 | `m` | tunable | How far a hit's damage reaches round its point (all of it within a third of this, falling to nothing at it). |
| `anchored` | `boolean` | `true` | — | — | — | Whether its anchors (the chunks on its floor) hold it to the world; false, every island falls. |
| `breakFx` | `ParticleEmitter` | `0` | — | — | — | A one-shot emitter (og.load.emitter, `looping false`) burst where each island breaks loose, at its broken faces: the material's own, a lit dust puff for stone and brick, splinters and a little dust for wood, shards and glints for glass. 0 none. |
| `hitFx` | `ParticleEmitter` | `0` | — | — | — | A one-shot emitter burst at each hit that damages it: the material's own, grit and a puff off stone, splinters off wood, glints off glass; sparks only off metal, and a few. 0 none. |
| `debris` | `ParticleEmitter` | `0` | — | — | — | GPU-only debris: a shattering chunk's pieces under og.Destruction.debrisVolume never become bodies but fly on the GPU for debrisLifetime, never touching the simulation (none drawn without visualDebris); this mesh-particle emitter bursts finer grains where they land. 0: every piece is a body. See `physics §9`. |
| `fractured` | `boolean` | `false` | — | — | read only | Written: true from its first break (the model hidden, its chunks drawn). |
| `broken` | `number` | `0` | — | — | read only | Written: its bonds broken so far. |
| `loose` | `number` | `0` | — | — | read only | Written: its chunks that have broken loose and still exist. |

```ogt
component Destructible {
  doc "Breaks the entity's og.Model, cooked with a `fracture` block, into its chunks (3D profile). A standing prop (no og.RigidBody, or a static one) is its chunks from the first tick: each a static og.Collider (its piece's hull) on an og.Chunk, replacing the entity's own Collider and RigidBody. A moving prop (a dynamic or kinematic body) collides as itself until its first break, when its chunks replace its Collider, RigidBody and Velocity, moving as it moved. Then each chunk draws its piece of the hidden model (its tint, variant, shadows)."
  doc "Chunks' colliders and mass follow og.Transform.scale's mean."
  guide "physics §9"
  aliases "breakable, destructable, destructible prop, fracture, Unreal Geometry Collection"
  field toughness f32 "How much damage a square metre of bond takes before it breaks: an impulse of that many newton-seconds through 1 m² (a hit spreads over impactRadius). Glass about 50, wood 300, brick 800, concrete 2000." {
    default 800
    min 0
    unit newtonSecondPerM2
    tunable true
  }
  field strength f32 "The stress its bonds hold in tension and shear before they weaken; compression holds thirty times as much, shear more by `friction`, and a bond breaks outright at twice the stress. 0 turns the stress solver off." {
    guide "physics §9"
    default 30000
    min 0
    unit pascal
    tunable true
  }
  field friction f32 "How much more shear a bond holds per pascal of compression across it (Mohr–Coulomb: the tangent of the friction angle). 0 is shear at strength alone." {
    guide "physics §9"
    default 0.7
    min 0
    max 4
    tunable true
  }
  field tension f32 "The stress its bonds hold pulled apart, as a fraction of strength (shear and compression keep strength's); 1 is tension at strength. At 0.1 or below, with strength above 0, a standing block also needs its weight within the middle third of what bears it from below, or it comes loose whole." {
    guide "physics §9"
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
  field impactThreshold f32 "A collision's impulse (og.CollisionStarted.impulse) that does no damage: the excess is the hit's damage and, on a prop that stood still, the push its broken-loose pieces take along the striker's path (shared by falloff, none faster than the striker closed); a moving prop's keep their velocity." {
    guide "physics §9"
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
  field debris ParticleEmitter "GPU-only debris: a shattering chunk's pieces under og.Destruction.debrisVolume never become bodies but fly on the GPU for debrisLifetime, never touching the simulation (none drawn without visualDebris); this mesh-particle emitter bursts finer grains where they land. 0: every piece is a body." {
    guide "physics §9"
  }
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

<sub>Module `destruction` line 19</sub>
