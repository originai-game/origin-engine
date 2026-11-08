<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-sceneViewport"></a>

### `og.ui.sceneViewport`

A viewport: a camera's render target shown in the layout, its drag, wheel and pointer ray handed back so a menu can orbit, pick and rotate.

```ts
og.ui.sceneViewport(id: string, desc?: SceneViewportDesc): SceneViewportInfo
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `SceneViewportDesc` | yes | What it shows and its camera. |

**Returns** `SceneViewportInfo`.

Binding symbol `og_ui_scene_viewport` · publish builds: kept

```ogt
fn sceneViewport {
  doc "A viewport: a camera's render target shown in the layout, its drag, wheel and pointer ray handed back so a menu can orbit, pick and rotate."
  param id String "Its id."
  param desc SceneViewportDesc "What it shows and its camera."
  returns SceneViewportInfo
}
```

<sub>Module `ui` line 7793</sub>
