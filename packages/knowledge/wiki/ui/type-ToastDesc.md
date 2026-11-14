<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ToastDesc"></a>

### `ToastDesc` (desc struct)

A toast's tone and time.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tone` | `Tone` | `"neutral"` | — | — | — | Its tone: a dot of the tone's colour before the text. |
| `durationMs` | `number` | `0` | ≥ 0 | `ms` | — | How long it stays; 0 is 3200 ms (6000 for danger). |

```ogt
struct ToastDesc {
  doc "A toast's tone and time."
  desc true
  field tone Tone "Its tone: a dot of the tone's colour before the text."
  field durationMs f32 "How long it stays; 0 is 3200 ms (6000 for danger)." {
    unit ms
    min 0
  }
}
```

<sub>Module `ui` line 435</sub>
