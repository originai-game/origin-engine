<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`atmosphere`](../atmosphere.md) · [index](../index.md)

<a id="type-WindMotorKind"></a>

### `WindMotorKind` (enum)

How an og.WindMotor pushes the wind.

A string: one of the variants below. The zero value is `"omni"`.

| Variant | Doc |
|---|---|
| `"omni"` | Outward from its centre: a blast, a landing, a horse's hooves. |
| `"directional"` | Along the entity's forward (its -z, horizontal part): a fan, a rotor's downwash leaning the grass one way. |
| `"vortex"` | Around its centre, counter-clockwise seen from above: a whirlwind, a spell. |

```ogt
enum WindMotorKind {
  doc "How an og.WindMotor pushes the wind."
  variant omni "Outward from its centre: a blast, a landing, a horse's hooves."
  variant directional "Along the entity's forward (its -z, horizontal part): a fan, a rotor's downwash leaning the grass one way."
  variant vortex "Around its centre, counter-clockwise seen from above: a whirlwind, a spell."
}
```

<sub>Module `atmosphere` line 606</sub>
