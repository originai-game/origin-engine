<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="record-Destructible"></a>

### `og.Destructible` (component)

Makes the entity's og.Model, a model cooked with a `fracture` block, break into its chunks. A prop that stands (no og.RigidBody, or a static one) is its chunks from the first tick: each a static og.Collider (the hull of its piece) on an entity with og.Chunk, and the entity's own og.Collider and og.RigidBody are removed, so hits land on the chunks. A prop that moves (a dynamic or kinematic og.RigidBody) collides as itself until its first break, when its chunks take its place, moving as it moved, and it loses its og.Collider, og.RigidBody and og.Velocity. Until the first break the model is drawn whole; from it the model is hidden and each chunk draws its piece of it (with og.Model's tint, variant and shadows). Anchored islands stay static, the rest fall.

Scale it uniformly: a chunk's collider and mass follow og.Transform.scale's mean.

Also known as: breakable, destructable, destructible prop, fracture, Unreal Geometry Collection

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
| `debris` | `ParticleEmitter` | `0` | — | — | — | Fine debris, as visuals only: where a chunk shatters, its pieces smaller than og.Destruction.debrisVolume never become bodies. They fly on the GPU instead, each its own shape and materials, from where and how fast it broke off (the blast's push included), each throwing four to six thick rubble chips of uneven size in the prop's cut-face material, and every joint that breaks sheds three or four small chips of its cut faces and of its mortar (`fracture.mortar`), all tumbling and landing on the colliders that are boxes, spheres and terrains, and this mesh-particle emitter (`render mesh`, best with `blend alpha`, `collision` and `lit true`) bursts its finer grains there too. They never touch the simulation, so the world is the same on every device, and a device without the GPU layer draws none of them (og.destruction.capabilities' visualDebris). They last og.Destruction.debrisLifetime. 0: every piece is a body. |
| `fractured` | `boolean` | `false` | — | — | read only | Written: true from its first break (the model hidden, its chunks drawn). |
| `broken` | `number` | `0` | — | — | read only | Written: its bonds broken so far. |
| `loose` | `number` | `0` | — | — | read only | Written: its chunks that have broken loose and still exist. |

```ogt
component Destructible {
  doc "Makes the entity's og.Model, a model cooked with a `fracture` block, break into its chunks. A prop that stands (no og.RigidBody, or a static one) is its chunks from the first tick: each a static og.Collider (the hull of its piece) on an entity with og.Chunk, and the entity's own og.Collider and og.RigidBody are removed, so hits land on the chunks. A prop that moves (a dynamic or kinematic og.RigidBody) collides as itself until its first break, when its chunks take its place, moving as it moved, and it loses its og.Collider, og.RigidBody and og.Velocity. Until the first break the model is drawn whole; from it the model is hidden and each chunk draws its piece of it (with og.Model's tint, variant and shadows). Anchored islands stay static, the rest fall."
  doc "Scale it uniformly: a chunk's collider and mass follow og.Transform.scale's mean."
  aliases "breakable, destructable, destructible prop, fracture, Unreal Geometry Collection"
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
  field debris ParticleEmitter "Fine debris, as visuals only: where a chunk shatters, its pieces smaller than og.Destruction.debrisVolume never become bodies. They fly on the GPU instead, each its own shape and materials, from where and how fast it broke off (the blast's push included), each throwing four to six thick rubble chips of uneven size in the prop's cut-face material, and every joint that breaks sheds three or four small chips of its cut faces and of its mortar (`fracture.mortar`), all tumbling and landing on the colliders that are boxes, spheres and terrains, and this mesh-particle emitter (`render mesh`, best with `blend alpha`, `collision` and `lit true`) bursts its finer grains there too. They never touch the simulation, so the world is the same on every device, and a device without the GPU layer draws none of them (og.destruction.capabilities' visualDebris). They last og.Destruction.debrisLifetime. 0: every piece is a body."
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

<sub>`api.ogt` line 27116</sub>
