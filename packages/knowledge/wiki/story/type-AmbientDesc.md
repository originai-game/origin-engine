<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="type-AmbientDesc"></a>

### `AmbientDesc` (desc struct)

An entity that barks on its own.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | Who barks. |
| `topic` | `string` | `""` | — | — | — | The topic it asks for. |
| `dialogue` | `string` | `""` | — | — | — | Only this dialogue's barks (empty: any). |
| `every` | `number` | `600` | — | — | — | Ticks between asks (0 stops it). |
| `jitter` | `number` | `0` | — | — | — | Up to this many ticks more each time, drawn on the story's stream. |

```ogt
struct AmbientDesc {
  doc "An entity that barks on its own."
  desc true
  field entity Entity "Who barks."
  field topic String "The topic it asks for."
  field dialogue String "Only this dialogue's barks (empty: any)."
  field every u32 "Ticks between asks (0 stops it)." {
    default 600
  }
  field jitter u32 "Up to this many ticks more each time, drawn on the story's stream."
}
```

<sub>Module `story` line 196</sub>
