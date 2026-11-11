<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`game`](../game.md) · [index](../index.md)

<a id="type-UiSetupFn"></a>

### `UiSetupFn` (callback)

Register named presentation stores. Called after start and after hot-reload restore, never per tick. World reads may choose initial values; world writes and gameplay side effects are refused. Existing IDs keep values, kept callbacks are replaced, and removed IDs are disposed. Document mounts await the Svelte parser IR.

Game code supplies this function.

```ts
type UiSetupFn = (world: World) => void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world, read only during registration. |

```ogt
callback UiSetupFn {
  doc "Register named presentation stores. Called after start and after hot-reload restore, never per tick. World reads may choose initial values; world writes and gameplay side effects are refused. Existing IDs keep values, kept callbacks are replaced, and removed IDs are disposed. Document mounts await the Svelte parser IR."
  param world World "The world, read only during registration."
}
```

<sub>Module `game` line 17</sub>
