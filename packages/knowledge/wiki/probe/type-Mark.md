<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-Mark"></a>

### `Mark` (struct)

One numbered box of a marked capture (Set-of-Mark): what it is and where a person would change it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `n` | `number` | `0` | — | — | — | Its number on the image, from 1. |
| `kind` | `string` | `""` | — | — | — | `ui` for a UI element, `entity` for a scene entity. |
| `id` | `string` | `""` | — | — | — | The element's id, or the entity's probe id. |
| `path` | `string` | `""` | — | — | — | The element's full path (as dump_ui, pickUi and touch_move_to name it), or the entity's probe id. |
| `what` | `string` | `""` | — | — | — | The element's kind (`button`, `label`, a kit's own), or what draws the entity (describe's labels, comma-separated). |
| `text` | `string` | `""` | — | — | — | The text the element shows, or the entity's name; empty for none. |
| `rect` | `MarkRect` | `{ x: 0, y: 0, width: 0, height: 0 }` | — | — | — | Its box on the image. |
| `point` | `ScreenPoint` | `{ x: 0, y: 0 }` | — | — | — | A point of the play surface on it, for pick, pickUi or touch_move_to: the element's centre, or a visible point of the entity (describe's point). |
| `source` | `string \| null` | `null` | — | — | — | Where a person would change it: the document line that built the element (`ui/shop.ogt:42`), the scene line that declares the entity (`scenes/main.ogt:12`), or the TypeScript line that spawned it (SpawnDesc.source); null for code-built UI, and for spawned entities in a publish build. |
| `part` | `string \| null` | `null` | — | — | — | The style part the element is drawn as (`Button.base[primary]`); null for an entity or an element no style dresses. |
| `rule` | `UiPickRule \| null` | `null` | — | — | — | The most specific style rule that dresses the element; null when none does. |
| `depth` | `number \| null` | `null` | — | `m` | — | The entity's distance from the camera to its nearest visible point; null for an element. |

```ogt
struct Mark {
  doc "One numbered box of a marked capture (Set-of-Mark): what it is and where a person would change it."
  field n u32 "Its number on the image, from 1."
  field kind String "`ui` for a UI element, `entity` for a scene entity."
  field id String "The element's id, or the entity's probe id."
  field path String "The element's full path (as dump_ui, pickUi and touch_move_to name it), or the entity's probe id."
  field what String "The element's kind (`button`, `label`, a kit's own), or what draws the entity (describe's labels, comma-separated)."
  field text String "The text the element shows, or the entity's name; empty for none."
  field rect MarkRect "Its box on the image."
  field point ScreenPoint "A point of the play surface on it, for pick, pickUi or touch_move_to: the element's centre, or a visible point of the entity (describe's point)."
  field source Option.String "Where a person would change it: the document line that built the element (`ui/shop.ogt:42`), the scene line that declares the entity (`scenes/main.ogt:12`), or the TypeScript line that spawned it (SpawnDesc.source); null for code-built UI, and for spawned entities in a publish build."
  field part Option.String "The style part the element is drawn as (`Button.base[primary]`); null for an entity or an element no style dresses."
  field rule Option.UiPickRule "The most specific style rule that dresses the element; null when none does."
  field depth Option.f32 "The entity's distance from the camera to its nearest visible point; null for an element." {
    unit m
  }
}
```

<sub>Module `probe` line 203</sub>
