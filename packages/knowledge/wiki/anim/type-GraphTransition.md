<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-GraphTransition"></a>

### `GraphTransition` (struct)

A prioritized transition; requires at least one condition or exit.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `from` | `string` | `""` | — | — | optional | Source state; omitted means any state except the target. |
| `to` | `string` | `""` | — | — | — | Target state. |
| `conditions` | `readonly GraphCondition[]` | `[]` | — | — | optional | All must hold; omitted means no parameter conditions. |
| `exit` | `number` | `0` | — | — | optional | Minimum source phase in cycles, nonnegative. |
| `fade` | `number` | `0` | — | — | optional | Nonnegative cross-fade seconds; omitted is 0.2. |
| `interrupt` | `GraphInterrupt` | `"any"` | — | — | optional | Fade interruption policy; omitted is any. |

```ogt
struct GraphTransition {
  doc "A prioritized transition; requires at least one condition or exit."
  field from String "Source state; omitted means any state except the target." {
    optional true
  }
  field to String "Target state."
  field conditions List.GraphCondition "All must hold; omitted means no parameter conditions." {
    optional true
  }
  field exit f32 "Minimum source phase in cycles, nonnegative." {
    optional true
  }
  field fade f32 "Nonnegative cross-fade seconds; omitted is 0.2." {
    optional true
  }
  field interrupt GraphInterrupt "Fade interruption policy; omitted is any." {
    optional true
  }
}
```

<sub>Module `anim` line 750</sub>
