<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="record-JointLimit"></a>

### `og.JointLimit` (component)

How far a joint may turn from its rest rotation when og.anim's solvers (og.IkChain, og.LookAt, og.FootLock) turn it: a swing of `axis` within a cone and a twist about it within a range. A hinge (a knee, an elbow) is `axis` along the hinge, `swing` 0 (or a little play) and the bend range as twistMin...twistMax. Put it on its own entity, one per joint; the animated pose itself is never clamped.

Token `og.JointLimit`, registered under the schema name `og.JointLimit`; its value type is `JointLimit`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `skeleton` | `Entity` | `0` | — | — | — | The entity with the og.Skeleton whose joint this limits. |
| `joint` | `number` | `0` | ≥ 0 | — | — | The limited joint (an index into the skin's joints). |
| `axis` | `Vector3` | `{ x: 0, y: 1, z: 0 }` | — | — | — | The twist axis in the joint's rest frame: along the bone for a ball joint, along the hinge for a hinge. |
| `swing` | `number` | `3.1416` | 0 … 3.1416 | `rad` | tunable | The cone half-angle the axis may swing away from its rest direction (0 locks it: a hinge; pi is free). |
| `twistMin` | `number` | `-3.1416` | -3.1416 … 3.1416 | `rad` | tunable | The least twist about `axis` from rest (a hinge's bend one way). |
| `twistMax` | `number` | `3.1416` | -3.1416 … 3.1416 | `rad` | tunable | The most twist about `axis` from rest (a hinge's bend the other way). |

```ogt
component JointLimit {
  doc "How far a joint may turn from its rest rotation when og.anim's solvers (og.IkChain, og.LookAt, og.FootLock) turn it: a swing of `axis` within a cone and a twist about it within a range. A hinge (a knee, an elbow) is `axis` along the hinge, `swing` 0 (or a little play) and the bend range as twistMin...twistMax. Put it on its own entity, one per joint; the animated pose itself is never clamped."
  field skeleton Entity "The entity with the og.Skeleton whose joint this limits."
  field joint u32 "The limited joint (an index into the skin's joints)." {
    min 0
  }
  field axis Vector3 "The twist axis in the joint's rest frame: along the bone for a ball joint, along the hinge for a hinge." {
    default 0 1 0
  }
  field swing f32 "The cone half-angle the axis may swing away from its rest direction (0 locks it: a hinge; pi is free)." {
    default 3.1416
    min 0
    max 3.1416
    unit rad
    tunable true
  }
  field twistMin f32 "The least twist about `axis` from rest (a hinge's bend one way)." {
    default -3.1416
    min -3.1416
    max 3.1416
    unit rad
    tunable true
  }
  field twistMax f32 "The most twist about `axis` from rest (a hinge's bend the other way)." {
    default 3.1416
    min -3.1416
    max 3.1416
    unit rad
    tunable true
  }
}
```

<sub>`api.ogt` line 11829</sub>
