<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`debug`](../debug.md) · [index](../index.md)

<a id="fn-text"></a>

### `og.debug.text`

A label anchored at a world point, drawn in screen space at a constant size in the UI's body font (a world with no UI font draws none), centred above the point: an entity's state, a counter, a name.

```ts
og.debug.text(world: World, position: Vector3, text: string, desc?: DebugDrawDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world drawn in. |
| `position` | `Vector3` | no | The point the label sits above. |
| `text` | `string` | no | The label. |
| `desc` | `DebugDrawDesc` | yes | Colour and duration (width is the text's size in CSS pixels, 14 where it is 2 or less). |

**Returns** `void`.

Binding symbol `og_debug_text` · publish builds: dropped

```ogt
fn text {
  doc "A label anchored at a world point, drawn in screen space at a constant size in the UI's body font (a world with no UI font draws none), centred above the point: an entity's state, a counter, a name."
  param world World "The world drawn in."
  param position Vector3 "The point the label sits above."
  param text String "The label."
  param desc DebugDrawDesc "Colour and duration (width is the text's size in CSS pixels, 14 where it is 2 or less)."
}
```

<sub>`api.ogt` line 25747</sub>
