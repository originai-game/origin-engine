<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`debug`](../debug.md) · [index](../index.md)

<a id="fn-enabled"></a>

### `og.debug.enabled`

Whether debug draws and views show.

```ts
og.debug.enabled(world: World): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world asked. |

**Returns** `boolean`.

Binding symbol `og_debug_enabled` · publish builds: dropped

```ogt
fn enabled {
  doc "Whether debug draws and views show."
  param world World "The world asked."
  returns bool
}
```

<sub>`api.ogt` line 25697</sub>
