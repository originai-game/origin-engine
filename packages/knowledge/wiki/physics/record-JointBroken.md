<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-JointBroken"></a>

### `og.JointBroken` (event)

A joint broke during the tick: the solver's force or torque on it passed its breakForce or breakTorque. Sent in joint entity order; the joint's `broken` is set, and its bodies part.

Token `og.JointBroken`, registered under the schema name `og.JointBroken`; its value type is `JointBroken`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `joint` | `Entity` | `0` | — | — | — | The og.Joint's entity. |
| `body` | `Entity` | `0` | — | — | — | Its first body. |
| `other` | `Entity` | `0` | — | — | — | Its second body; 0 for the world. |
| `force` | `number` | `0` | — | `newton` | — | The force it carried when it broke. |
| `torque` | `number` | `0` | — | `newtonMetre` | — | The torque about its anchor it carried when it broke. |

```ogt
event JointBroken {
  doc "A joint broke during the tick: the solver's force or torque on it passed its breakForce or breakTorque. Sent in joint entity order; the joint's `broken` is set, and its bodies part."
  field joint Entity "The og.Joint's entity."
  field body Entity "Its first body."
  field other Entity "Its second body; 0 for the world."
  field force f32 "The force it carried when it broke." {
    unit newton
  }
  field torque f32 "The torque about its anchor it carried when it broke." {
    unit newtonMetre
  }
}
```

<sub>`api.ogt` line 8739</sub>
