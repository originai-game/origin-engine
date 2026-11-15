<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`film`](../film.md) · [index](../index.md)

<a id="fn-pause"></a>

### `og.film.pause`

Hold a film on its frame (a pause menu); its window's time holds too. Systems may call it.

```ts
og.film.pause(world: World, film: Entity): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `film` | `Entity` | no | The film. |

**Returns** `Result<void>`: Refused for an entity that is no film.

Binding symbol `og_film_pause` · publish builds: kept

```ogt
fn pause {
  doc "Hold a film on its frame (a pause menu); its window's time holds too. Systems may call it."
  param world World "The world."
  param film Entity "The film."
  returns Result.Void "Refused for an entity that is no film."
}
```

<sub>Module `film` line 235</sub>
