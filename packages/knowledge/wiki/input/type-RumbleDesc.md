<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-RumbleDesc"></a>

### `RumbleDesc` (desc struct)

A rumble of a pad's motors: output for the host, never world state (a replay does not repeat it).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `pad` | `number` | `-1` | -1 … 7 | — | — | The pad, 0 to 7; -1 every connected pad. |
| `low` | `number` | `0` | 0 … 1 | `ratio` | — | The heavy (low-frequency) motor. |
| `high` | `number` | `0` | 0 … 1 | `ratio` | — | The light (high-frequency) motor. |
| `ms` | `number` | `200` | ≥ 0 | `ms` | — | How long; 0 stops the pad's rumble. |

```ogt
struct RumbleDesc {
  doc "A rumble of a pad's motors: output for the host, never world state (a replay does not repeat it)."
  desc true
  field pad i32 "The pad, 0 to 7; -1 every connected pad." {
    default -1
    min -1
    max 7
  }
  field low f32 "The heavy (low-frequency) motor." {
    min 0
    max 1
    unit ratio
  }
  field high f32 "The light (high-frequency) motor." {
    min 0
    max 1
    unit ratio
  }
  field ms f32 "How long; 0 stops the pad's rumble." {
    default 200
    min 0
    unit ms
  }
}
```

<sub>`api.ogt` line 2275</sub>
