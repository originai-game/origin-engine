<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-Buoyancy"></a>

### `og.Buoyancy` (component)

Floats a dynamic rigid body (3D profile) on the water under it: an og.WaterSurface (at its rest level), an og.Water (a flat lake, or a sea with a wind: its waves as drawn, og.physics.waterHeight's) or both on one entity. Its hull is its colliders' box cut into `samples` cells a side, kept where a cell's centre lies in a solid collider; each cell under the surface pushes up by the water it displaces (Archimedes: `waterDensity` × its share of the colliders' volume × gravity plus the water's own acceleration, so it sinks with a crest that falls away rather than being flung clear) at that cell, so a hull rights itself, pitches and rolls with the waves, and is slowed by `drag` and `angularDrag` against the moving water as far as it is under. Computed in the tick, the same on every run and after a restore; the sea's waves are the absolute world's, so a body moved by og.WorldOrigin rides the same water. A body with it takes no og.WaterSurface `buoyancy` (only its splash). Insert it beside og.RigidBody and og.Collider: `{}` floats a body of the default density half under.

Token `og.Buoyancy`, registered under the schema name `og.Buoyancy`; its value type is `Buoyancy`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `waterDensity` | `number` | `2` | ≥ 0 | `kgPerM3` | tunable | The water's density, in og.Collider.density's units: the default 2 floats a body of the colliders' default density 1 half under, as wood floats; at 1 the same body hangs just under the surface, under it sinks. |
| `samples` | `number` | `4` | 1 … 12 | — | — | Cells along each side of its colliders' box (1: one point at its centre; up to 12³ points): more follow its shape and the waves more closely, at more cost. |
| `drag` | `number` | `4` | 0 … 100 | `perSecond` | tunable | How much of its speed through the water it loses each second when fully under (in part when in part); off-centre cells slow its turning too, so it settles instead of bobbing. |
| `angularDrag` | `number` | `3` | 0 … 100 | `perSecond` | tunable | How much of its spin it loses each second when fully under (in part when in part). |
| `submerged` | `number` | `0` | 0 … 1 | — | read only | Written: the fraction of its hull under water after the last step (0 above it or out of water, 1 sunk): for wakes, splashes and a sinking ship. |

```ogt
component Buoyancy {
  doc "Floats a dynamic rigid body (3D profile) on the water under it: an og.WaterSurface (at its rest level), an og.Water (a flat lake, or a sea with a wind: its waves as drawn, og.physics.waterHeight's) or both on one entity. Its hull is its colliders' box cut into `samples` cells a side, kept where a cell's centre lies in a solid collider; each cell under the surface pushes up by the water it displaces (Archimedes: `waterDensity` × its share of the colliders' volume × gravity plus the water's own acceleration, so it sinks with a crest that falls away rather than being flung clear) at that cell, so a hull rights itself, pitches and rolls with the waves, and is slowed by `drag` and `angularDrag` against the moving water as far as it is under. Computed in the tick, the same on every run and after a restore; the sea's waves are the absolute world's, so a body moved by og.WorldOrigin rides the same water. A body with it takes no og.WaterSurface `buoyancy` (only its splash). Insert it beside og.RigidBody and og.Collider: `{}` floats a body of the default density half under."
  field waterDensity f32 "The water's density, in og.Collider.density's units: the default 2 floats a body of the colliders' default density 1 half under, as wood floats; at 1 the same body hangs just under the surface, under it sinks." {
    default 2
    min 0
    unit kgPerM3
    tunable true
  }
  field samples u32 "Cells along each side of its colliders' box (1: one point at its centre; up to 12³ points): more follow its shape and the waves more closely, at more cost." {
    default 4
    min 1
    max 12
  }
  field drag f32 "How much of its speed through the water it loses each second when fully under (in part when in part); off-centre cells slow its turning too, so it settles instead of bobbing." {
    default 4
    min 0
    max 100
    unit perSecond
    tunable true
  }
  field angularDrag f32 "How much of its spin it loses each second when fully under (in part when in part)." {
    default 3
    min 0
    max 100
    unit perSecond
    tunable true
  }
  field submerged f32 "Written: the fraction of its hull under water after the last step (0 above it or out of water, 1 sunk): for wakes, splashes and a sinking ship." {
    min 0
    max 1
    readonly true
  }
}
```

<sub>`api.ogt` line 10082</sub>
