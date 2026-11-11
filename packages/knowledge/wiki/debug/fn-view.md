<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`debug`](../debug.md) · [index](../index.md)

<a id="fn-view"></a>

### `og.debug.view`

The debug view shown.

```ts
og.debug.view(world: World): DebugView
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world asked. |

**Returns** `DebugView`.

Binding symbol `og_debug_view` · publish builds: dropped

```ogt
fn view {
  doc "The debug view shown."
  param world World "The world asked."
  returns DebugView
}
```

<sub>Module `debug` line 82</sub>
