<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`game`](../game.md) · [index](../index.md)

<a id="type-StartFn"></a>

### `StartFn` (callback)

The game's setup: called once before the first tick, after GameDesc.scene has spawned. Spawn, insert, load and set resources here; systems then only read and write their views.

Game code supplies this function.

```ts
type StartFn = (world: World) => void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

```ogt
callback StartFn {
  doc "The game's setup: called once before the first tick, after GameDesc.scene has spawned. Spawn, insert, load and set resources here; systems then only read and write their views."
  param world World "The world."
}
```

<sub>Module `game` line 11</sub>
