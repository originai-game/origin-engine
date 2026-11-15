<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`film`](../film.md) · [index](../index.md)

<a id="fn-play"></a>

### `og.film.play`

Play a paused film from where it is. Systems may call it.

```ts
og.film.play(world: World, film: Entity): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `film` | `Entity` | no | The film. |

**Returns** `Result<void>`: Refused for an entity that is no film.

Binding symbol `og_film_play` · publish builds: kept

```ogt
fn play {
  doc "Play a paused film from where it is. Systems may call it."
  param world World "The world."
  param film Entity "The film."
  returns Result.Void "Refused for an entity that is no film."
}
```

<sub>Module `film` line 228</sub>
