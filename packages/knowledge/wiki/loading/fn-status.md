<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`loading`](../loading.md) · [index](../index.md)

<a id="fn-status"></a>

### `og.loading.status`

The screen, the last stage reported and whether loading is done.

```ts
og.loading.status(world: World): LoadingStatus
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `LoadingStatus`.

Binding symbol `og_loading_status` · publish builds: kept

```ogt
fn status {
  doc "The screen, the last stage reported and whether loading is done."
  param world World "The world."
  returns LoadingStatus
}
```

<sub>Module `loading` line 54</sub>
