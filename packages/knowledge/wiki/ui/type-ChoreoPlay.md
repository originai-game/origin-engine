<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ChoreoPlay"></a>

### `ChoreoPlay` (struct)

A timeline on the play's element or a descendant.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `timeline` | `string` | `""` | — | — | — | The timeline, by name. |
| `target` | `string` | `""` | — | — | — | A path below the play's element (empty: the element itself). |
| `delayMs` | `number` | `0` | — | `ms` | — | Wait this long first. |

```ogt
struct ChoreoPlay {
  doc "A timeline on the play's element or a descendant."
  field timeline String "The timeline, by name."
  field target String "A path below the play's element (empty: the element itself)."
  field delayMs f32 "Wait this long first." {
    unit ms
  }
}
```

<sub>Module `ui` line 2569</sub>
