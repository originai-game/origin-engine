<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-PlantDesc"></a>

### `PlantDesc` (desc struct)

Deterministic procedural tree, shrub or flower mesh specification (og-shape's growth engine). Omitting `levelsSpec` uses a default ~6 m broadleaf tree.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `seed` | `number` | `1` | — | — | — | Deterministic seed. |
| `growth` | `PlantGrowth` | `"deciduous"` | — | — | — | Branching habit (deciduous or evergreen). |
| `levels` | `number` | `2` | ≤ 3 | — | — | Branching levels beyond the trunk (0 to 3). |
| `levelsSpec` | `readonly PlantLevelDesc[]` | `[]` | — | — | — | Per-level branch descriptors (up to 4 entries, index 0 = trunk; omitted levels take defaults). |
| `path` | `readonly Vector3[]` | `[]` | — | — | — | Optional 2..1025 authored trunk ring centres in metres; replaces trunk wandering and section count. Branches and foliage grow from these exact centres with the existing rules. Consecutive centres must differ. |
| `branches` | `readonly PlantBranchDesc[]` | `[]` | — | — | — | Optional complete parent-first skeleton; exclusive with path. Leaves and flowers grow on terminal limbs with the existing rules. At most 20000 limbs; no procedural continuation is added. |
| `bounds` | `PlantBoundsDesc \| null` | `null` | — | — | — | Optional centreline box. Procedural limbs stop before their first outside ring, becoming terminal; fewer than two inside rings discard the limb. Explicit skeleton rings must all lie inside. Leaf/wood radii can extend outside; this is not triangle clipping. |
| `leafMesh` | `BufferGeometryDesc \| null` | `null` | — | — | — | Optional unit leaf geometry, replacing the built-in blade/card/cross. x is width, y is length, z faces out; scaled by each leaf's declared size and width. Buffer geometry validation, colours, UVs and generated wind apply. |
| `flowerMesh` | `BufferGeometryDesc \| null` | `null` | — | — | — | Optional unit flower geometry replacing generated petals and centre; same axes and attributes as leafMesh, scaled by flower size and width. |
| `forceDirection` | `Vector3` | `{ x: 0, y: 1, z: 0 }` | — | — | — | Tropism / gravity bend direction. |
| `forceStrength` | `number` | `0.01` | -2 … 2 | — | — | Tropism bend strength (negative droops boughs downward). |
| `leaves` | `PlantLeavesDesc \| null` | `null` | — | — | — | Optional foliage leaves (pass {} for standard broadleaf leaves; omit or null for bare winter wood). |
| `flowers` | `PlantFlowersDesc \| null` | `null` | — | — | — | Optional flowers on terminal shoots. |

```ogt
struct PlantDesc {
  doc "Deterministic procedural tree, shrub or flower mesh specification (og-shape's growth engine). Omitting `levelsSpec` uses a default ~6 m broadleaf tree."
  desc true
  field seed u32 "Deterministic seed." {
    default 1
  }
  field growth PlantGrowth "Branching habit (deciduous or evergreen)."
  field levels u32 "Branching levels beyond the trunk (0 to 3)." {
    default 2
    max 3
  }
  field levelsSpec List.PlantLevelDesc "Per-level branch descriptors (up to 4 entries, index 0 = trunk; omitted levels take defaults)."
  field path List.Vector3 "Optional 2..1025 authored trunk ring centres in metres; replaces trunk wandering and section count. Branches and foliage grow from these exact centres with the existing rules. Consecutive centres must differ."
  field branches List.PlantBranchDesc "Optional complete parent-first skeleton; exclusive with path. Leaves and flowers grow on terminal limbs with the existing rules. At most 20000 limbs; no procedural continuation is added."
  field bounds Option.PlantBoundsDesc "Optional centreline box. Procedural limbs stop before their first outside ring, becoming terminal; fewer than two inside rings discard the limb. Explicit skeleton rings must all lie inside. Leaf/wood radii can extend outside; this is not triangle clipping."
  field leafMesh Option.BufferGeometryDesc "Optional unit leaf geometry, replacing the built-in blade/card/cross. x is width, y is length, z faces out; scaled by each leaf's declared size and width. Buffer geometry validation, colours, UVs and generated wind apply."
  field flowerMesh Option.BufferGeometryDesc "Optional unit flower geometry replacing generated petals and centre; same axes and attributes as leafMesh, scaled by flower size and width."
  field forceDirection Vector3 "Tropism / gravity bend direction." {
    default 0 1 0
  }
  field forceStrength f32 "Tropism bend strength (negative droops boughs downward)." {
    default 0.01
    min -2
    max 2
  }
  field leaves Option.PlantLeavesDesc "Optional foliage leaves (pass {} for standard broadleaf leaves; omit or null for bare winter wood)."
  field flowers Option.PlantFlowersDesc "Optional flowers on terminal shoots."
}
```

<sub>Module `procgen` line 812</sub>
