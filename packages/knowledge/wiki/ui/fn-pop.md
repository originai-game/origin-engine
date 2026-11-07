<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-pop"></a>

### `og.ui.pop`

Pop the active layer; the next active one takes its remembered focus. Answers the popped screen's name (empty: the stack was empty).

```ts
og.ui.pop(world: World): string
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `string`.

Binding symbol `og_ui_pop` · publish builds: kept

```ogt
fn pop {
  doc "Pop the active layer; the next active one takes its remembered focus. Answers the popped screen's name (empty: the stack was empty)."
  param world World "The world."
  returns String
}
```

<sub>Module `ui` line 8115</sub>
