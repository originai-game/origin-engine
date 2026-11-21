<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`film`](../film.md) · [index](../index.md)

<a id="record-FilmChoice"></a>

### `og.FilmChoice` (event)

A film took a choice at its cut (before og.story ran it).

Token `og.FilmChoice`, registered under the schema name `og.FilmChoice`; its value type is `FilmChoice`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `film` | `Entity` | `0` | — | — | — | The film. |
| `node` | `string` | `""` | — | — | — | The node (at most 63 bytes). |
| `index` | `number` | `0` | — | — | — | The ChoiceView.index taken. |
| `timedOut` | `boolean` | `false` | — | — | — | Nothing was chosen in the window: the default was taken. |

```ogt
event FilmChoice {
  doc "A film took a choice at its cut (before og.story ran it)."
  field film Entity "The film."
  field node String "The node (at most 63 bytes)."
  field index u32 "The ChoiceView.index taken."
  field timedOut bool "Nothing was chosen in the window: the default was taken."
}
```

<sub>Module `film` line 178</sub>
