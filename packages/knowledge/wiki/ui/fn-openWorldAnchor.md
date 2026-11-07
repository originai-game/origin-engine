<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-openWorldAnchor"></a>

### `og.ui.openWorldAnchor`

Open an element anchored in the world, closed by close(): its children are an ordinary subtree (labels, bars, icons, buttons) laid out at its own size, then projected, scaled, depth tested, decluttered and clamped as desc says. Drawn under the screen's own elements. og.ui.setDraw after it draws its box. For thousands of tags of one kind (overhead bars on an army) use mass elements instead.

```ts
og.ui.openWorldAnchor(id: string, desc?: WorldAnchorDesc): AnchorInfo
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id; the probe path is the id alone. |
| `desc` | `WorldAnchorDesc` | yes | What it sits on and how it follows the camera. |

**Returns** `AnchorInfo`.

Binding symbol `og_ui_open_world_anchor` · publish builds: kept

```ogt
fn openWorldAnchor {
  doc "Open an element anchored in the world, closed by close(): its children are an ordinary subtree (labels, bars, icons, buttons) laid out at its own size, then projected, scaled, depth tested, decluttered and clamped as desc says. Drawn under the screen's own elements. og.ui.setDraw after it draws its box. For thousands of tags of one kind (overhead bars on an army) use mass elements instead."
  param id String "Its id; the probe path is the id alone."
  param desc WorldAnchorDesc "What it sits on and how it follows the camera."
  returns AnchorInfo
}
```

<sub>Module `ui` line 7638</sub>
