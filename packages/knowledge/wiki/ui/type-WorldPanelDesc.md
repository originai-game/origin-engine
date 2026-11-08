<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-WorldPanelDesc"></a>

### `WorldPanelDesc` (desc struct)

A world panel (docs/ui.md §1): a UI tree drawn into a render target that a mesh shows (a terminal, a cockpit screen, a shop sign), reached by the pointer through a ray onto its quad.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `target` | `Texture \| null` | `null` | — | — | — | The colour of an og.render.renderTarget made with format rgba16float (RenderTargetInfo.texture); give the mesh's material it as map (lit) or emissiveMap with colour black (unlit). It shows the frame after the UI drew it. Null: laid out and hit, drawn nowhere. |
| `width` | `number` | `480` | ≥ 1 | `cssPx` | — | Its layout width; the target's pixels spread over it. |
| `height` | `number` | `320` | ≥ 1 | `cssPx` | — | Its layout height. |
| `quad` | `WorldTarget \| null` | `null` | — | — | — | Where its quad is: centred on the target's pose, facing its +Z (an og.render.planeGeometry of quadWidth by quadHeight on that entity). Null: no ray reaches it. |
| `rotation` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | The quad's rotation after its target's. |
| `quadWidth` | `number` | `1` | ≥ 0 | `m` | — | The quad's width along its +X. |
| `quadHeight` | `number` | `1` | ≥ 0 | `m` | — | The quad's height along its +Y. |
| `direction` | `Direction` | `"row"` | — | — | — | Which way it lays out its children; the default is a column. |
| `gap` | `number` | `0` | — | `cssPx` | — | Space between children. |
| `padding` | `Insets` | `{ left: 0, top: 0, right: 0, bottom: 0 }` | — | — | — | Space inside its edges. |
| `background` | `Paint \| null` | `null` | — | — | — | Its fill; null draws none (the target clears to transparent). |
| `projected` | `boolean` | `false` | — | — | — | Drawn by the UI itself onto where its quad is seen (a ground decal under a unit, a sign over a door) in a band under the screen's elements, instead of into target: no render target or mesh. Inert: no ray reaches it and its controls take no focus. Not drawn while a corner of its quad is behind the camera. |

```ogt
struct WorldPanelDesc {
  doc "A world panel (docs/ui.md §1): a UI tree drawn into a render target that a mesh shows (a terminal, a cockpit screen, a shop sign), reached by the pointer through a ray onto its quad."
  desc true
  field target Option.Texture "The colour of an og.render.renderTarget made with format rgba16float (RenderTargetInfo.texture); give the mesh's material it as map (lit) or emissiveMap with colour black (unlit). It shows the frame after the UI drew it. Null: laid out and hit, drawn nowhere."
  field width f32 "Its layout width; the target's pixels spread over it." {
    default 480
    unit cssPx
    min 1
  }
  field height f32 "Its layout height." {
    default 320
    unit cssPx
    min 1
  }
  field quad Option.WorldTarget "Where its quad is: centred on the target's pose, facing its +Z (an og.render.planeGeometry of quadWidth by quadHeight on that entity). Null: no ray reaches it."
  field rotation Quaternion "The quad's rotation after its target's."
  field quadWidth f32 "The quad's width along its +X." {
    default 1
    unit m
    min 0
  }
  field quadHeight f32 "The quad's height along its +Y." {
    default 1
    unit m
    min 0
  }
  field direction Direction "Which way it lays out its children; the default is a column."
  field gap f32 "Space between children." {
    unit cssPx
  }
  field padding Insets "Space inside its edges."
  field background Option.Paint "Its fill; null draws none (the target clears to transparent)."
  field projected bool "Drawn by the UI itself onto where its quad is seen (a ground decal under a unit, a sign over a door) in a band under the screen's elements, instead of into target: no render target or mesh. Inert: no ray reaches it and its controls take no focus. Not drawn while a corner of its quad is behind the camera."
}
```

<sub>Module `ui` line 7431</sub>
