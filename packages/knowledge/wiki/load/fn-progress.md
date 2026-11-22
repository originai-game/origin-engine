<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-progress"></a>

### `og.load.progress`

How far a load has arrived: a bundle og.load.bundle opened, or any handle (a cooked load answers ready at once). Answered from what the host has received, for a loading screen; natively a read completes at once.

See `loading §12`.

```ts
og.load.progress(world: World, asset: Asset): Result<LoadProgress>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `asset` | `Asset` | no | The handle. |

**Returns** `Result<LoadProgress>`: Refused for a handle that is not alive.

Binding symbol `og_load_progress` · publish builds: kept

```ogt
fn progress {
  doc "How far a load has arrived: a bundle og.load.bundle opened, or any handle (a cooked load answers ready at once). Answered from what the host has received, for a loading screen; natively a read completes at once."
  guide "loading §12"
  param world World "The world that owns the asset."
  param asset Asset "The handle."
  returns Result.LoadProgress "Refused for a handle that is not alive."
}
```

<sub>Module `load` line 228</sub>
