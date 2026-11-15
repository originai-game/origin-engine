<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`film`](../film.md) · [index](../index.md)

<a id="fn-view"></a>

### `og.film.view`

What a film shows now: its picture, segment time, choices and window.

```ts
og.film.view(world: World, film: Entity): FilmView | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `film` | `Entity` | no | The film. |

**Returns** `FilmView | null`: Null for an entity that is no film.

Binding symbol `og_film_view` · publish builds: kept

```ogt
fn view {
  doc "What a film shows now: its picture, segment time, choices and window."
  param world World "The world."
  param film Entity "The film."
  returns Option.FilmView "Null for an entity that is no film."
}
```

<sub>Module `film` line 139</sub>
