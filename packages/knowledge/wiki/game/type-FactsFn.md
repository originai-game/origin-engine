<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`game`](../game.md) · [index](../index.md)

<a id="type-FactsFn"></a>

### `FactsFn` (callback)

The game's own facts for the probe's state(): a small JSON object, computed from the world without changing it.

Game code supplies this function.

```ts
type FactsFn = (world: World) => unknown
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world, read only. |

```ogt
callback FactsFn {
  doc "The game's own facts for the probe's state(): a small JSON object, computed from the world without changing it."
  param world World "The world, read only."
  returns Json
}
```

<sub>Module `game` line 6</sub>
