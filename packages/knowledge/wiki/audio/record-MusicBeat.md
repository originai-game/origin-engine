<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

<a id="record-MusicBeat"></a>

### `og.MusicBeat` (event)

A beat of a Music clock, sent in the tick whose interval holds it (systems read it the tick after).

Token `og.MusicBeat`, registered under the schema name `og.MusicBeat`; its value type is `MusicBeat`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity with the Music. |
| `bar` | `number` | `0` | — | — | — | The bar, from 1. |
| `beat` | `number` | `0` | — | — | — | The beat in the bar, from 1 (1 is the bar line). |
| `time` | `number` | `0` | — | — | — | Its world time. |

```ogt
event MusicBeat {
  doc "A beat of a Music clock, sent in the tick whose interval holds it (systems read it the tick after)."
  field entity Entity "The entity with the Music."
  field bar u32 "The bar, from 1."
  field beat u32 "The beat in the bar, from 1 (1 is the bar line)."
  field time f64 "Its world time."
}
```

<sub>`api.ogt` line 13496</sub>
