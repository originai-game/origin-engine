<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`debug`](../debug.md) · [index](../index.md)

<a id="fn-setView"></a>

### `og.debug.setView`

Show a debug view besides og.debug's draws (the command channel's `debug:<view>`); none shows the draws alone.

```ts
og.debug.setView(world: World, view: DebugView): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world drawn in. |
| `view` | `DebugView` | no | The view. |

**Returns** `void`.

Binding symbol `og_debug_set_view` · publish builds: dropped

```ogt
fn setView {
  doc "Show a debug view besides og.debug's draws (the command channel's `debug:<view>`); none shows the draws alone."
  param world World "The world drawn in."
  param view DebugView "The view."
}
```

<sub>`api.ogt` line 25804</sub>
