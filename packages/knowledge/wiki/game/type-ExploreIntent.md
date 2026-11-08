<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`game`](../game.md) · [index](../index.md)

<a id="type-ExploreIntent"></a>

### `ExploreIntent` (desc struct)

An intent `og explore` sends (ExploreDesc.intents).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `event` | `string` | `""` | — | — | — | The event's name. |
| `values` | `readonly unknown[]` | `[]` | — | — | — | The payloads to send, as objects of its fields. Empty: drawn from its reflection (each enum variant, a ranged integer's values, both bools, a ranged float's min, middle and max; other fields at their defaults). |

```ogt
struct ExploreIntent {
  doc "An intent `og explore` sends (ExploreDesc.intents)."
  desc true
  field event String "The event's name."
  field values List.Json "The payloads to send, as objects of its fields. Empty: drawn from its reflection (each enum variant, a ranged integer's values, both bools, a ranged float's min, middle and max; other fields at their defaults)."
}
```

<sub>Module `game` line 69</sub>
