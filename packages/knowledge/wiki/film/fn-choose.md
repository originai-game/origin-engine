<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`film`](../film.md) · [index](../index.md)

<a id="fn-choose"></a>

### `og.film.choose`

Take a choice while the window is open: recorded now (FilmView.window.pending; a later call replaces it) and taken at the cut, so the footage plays on to it. Systems may call it; og.film applies it this tick.

```ts
og.film.choose(world: World, film: Entity, index: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `film` | `Entity` | no | The film. |
| `index` | `number` | no | ChoiceView.index. |

**Returns** `Result<void>`: Refused outside an open window, or for a choice that is hidden or disabled.

Binding symbol `og_film_choose` · publish builds: kept

```ogt
fn choose {
  doc "Take a choice while the window is open: recorded now (FilmView.window.pending; a later call replaces it) and taken at the cut, so the footage plays on to it. Systems may call it; og.film applies it this tick."
  param world World "The world."
  param film Entity "The film."
  param index u32 "ChoiceView.index."
  returns Result.Void "Refused outside an open window, or for a choice that is hidden or disabled."
}
```

<sub>Module `film` line 146</sub>
