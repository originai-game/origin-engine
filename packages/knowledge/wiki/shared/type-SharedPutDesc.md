<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="type-SharedPutDesc"></a>

### `SharedPutDesc` (desc struct)

Where a shared object is, and the version a change expects (og.shared.put).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Where, in world space; within this visitor's sight. |
| `yaw` | `number` | `0` | — | `rad` | — | Its rotation about y. |
| `expect` | `number \| null` | `null` | — | — | — | Apply only while the object has this version (0: only if it does not exist yet): two visitors opening one door, the second is refused. Null: whatever it has. |

```ogt
struct SharedPutDesc {
  doc "Where a shared object is, and the version a change expects (og.shared.put)."
  desc true
  field position Vector3 "Where, in world space; within this visitor's sight."
  field yaw f32 "Its rotation about y." {
    unit rad
  }
  field expect Option.u32 "Apply only while the object has this version (0: only if it does not exist yet): two visitors opening one door, the second is refused. Null: whatever it has."
}
```

<sub>Module `shared` line 142</sub>
