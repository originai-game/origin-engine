<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-clear"></a>

### `og.ui.clear`

Empty the layer stack.

```ts
og.ui.clear(world: World): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `void`.

Binding symbol `og_ui_clear` · publish builds: kept

```ogt
fn clear {
  doc "Empty the layer stack."
  param world World "The world."
  returns Void
}
```

<sub>Module `ui` line 4262</sub>
