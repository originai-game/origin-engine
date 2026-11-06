<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-bounds"></a>

### `og.ui.bounds`

Where an element was in the last frame, by id or `Parent/Child` path (topmost first), as touch_move_to resolves it.

```ts
og.ui.bounds(path: string): UiRect | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `path` | `string` | no | The id or path. |

**Returns** `UiRect | null`.

Binding symbol `og_ui_bounds` · publish builds: kept

```ogt
fn bounds {
  doc "Where an element was in the last frame, by id or `Parent/Child` path (topmost first), as touch_move_to resolves it."
  param path String "The id or path."
  returns Option.UiRect
}
```

<sub>`api.ogt` line 17062</sub>
