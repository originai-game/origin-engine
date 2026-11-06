<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-DeadzoneDesc"></a>

### `DeadzoneDesc` (desc struct)

A pad's dead zones: what its sticks and triggers read as rest. The player's to set (a settings slider); og.input.saveBindings keeps them.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `pad` | `number` | `-1` | -1 … 7 | — | — | The pad, 0 to 7; -1 every pad. |
| `stick` | `number` | `0.15` | 0 … 0.95 | — | — | The sticks' (radial for a stick's two axes). |
| `trigger` | `number` | `0.05` | 0 … 0.95 | — | — | The triggers'. |

```ogt
struct DeadzoneDesc {
  doc "A pad's dead zones: what its sticks and triggers read as rest. The player's to set (a settings slider); og.input.saveBindings keeps them."
  desc true
  field pad i32 "The pad, 0 to 7; -1 every pad." {
    default -1
    min -1
    max 7
  }
  field stick f32 "The sticks' (radial for a stick's two axes)." {
    default 0.15
    min 0
    max 0.95
  }
  field trigger f32 "The triggers'." {
    default 0.05
    min 0
    max 0.95
  }
}
```

<sub>`api.ogt` line 2255</sub>
