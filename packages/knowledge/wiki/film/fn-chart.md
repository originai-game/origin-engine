<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`film`](../film.md) · [index](../index.md)

<a id="fn-chart"></a>

### `og.film.chart`

A prepared dialogue's film nodes and where each may lead, laid out for a flowchart.

```ts
og.film.chart(world: World, dialogue: string): Result<FilmChart>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `dialogue` | `string` | no | The dialogue (StoryAsset.name), prepared by og.film.load. |

**Returns** `Result<FilmChart>`: Refused for a dialogue og.film.load has not prepared.

Binding symbol `og_film_chart` · publish builds: kept

```ogt
fn chart {
  doc "A prepared dialogue's film nodes and where each may lead, laid out for a flowchart."
  param world World "The world."
  param dialogue String "The dialogue (StoryAsset.name), prepared by og.film.load."
  returns Result.FilmChart "Refused for a dialogue og.film.load has not prepared."
}
```

<sub>Module `film` line 250</sub>
