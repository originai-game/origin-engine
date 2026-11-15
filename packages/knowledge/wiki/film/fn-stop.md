<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`film`](../film.md) · [index](../index.md)

<a id="fn-stop"></a>

### `og.film.stop`

End a film: its conversation stops and its decks and entity are despawned this tick. Systems may call it.

```ts
og.film.stop(world: World, film: Entity): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `film` | `Entity` | no | The film. |

**Returns** `Result<void>`: Refused for an entity that is no film.

Binding symbol `og_film_stop` · publish builds: kept

```ogt
fn stop {
  doc "End a film: its conversation stops and its decks and entity are despawned this tick. Systems may call it."
  param world World "The world."
  param film Entity "The film."
  returns Result.Void "Refused for an entity that is no film."
}
```

<sub>Module `film` line 256</sub>
