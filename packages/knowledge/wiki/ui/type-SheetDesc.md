<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SheetDesc"></a>

### `SheetDesc` (desc struct)

A sheet: a surface that slides in from an edge over a scrim; a bottom sheet rests at detents and a drag moves it.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `edge` | `SheetEdge` | `"bottom"` | — | — | — | The edge it comes from. |
| `detents` | `readonly number[]` | `[]` | — | — | — | A bottom sheet's rest heights, fractions of the room; empty is 0.5 and 0.92. |
| `detent` | `number` | `0` | — | — | — | The detent it rests at (keep SheetResult.detent). |
| `size` | `number` | `0` | ≥ 0 | `cssPx` | — | A side sheet's width; 0 is 400. |
| `title` | `string` | `""` | — | — | — | A title at its top; empty is none. |
| `dismissible` | `boolean` | `true` | — | — | — | A drag past the lowest detent, a swipe outwards, Back or the scrim close it. |
| `scrim` | `boolean` | `true` | — | — | — | Dim what is behind it. |
| `handle` | `boolean` | `true` | — | — | — | A bottom sheet's grab handle. |
| `closeButton` | `boolean \| null` | `null` | — | — | — | A close button by the title; null is a side sheet's only. |
| `closeLabel` | `string` | `""` | — | — | — | The close button's accessible name; empty is Close. |

```ogt
struct SheetDesc {
  doc "A sheet: a surface that slides in from an edge over a scrim; a bottom sheet rests at detents and a drag moves it."
  desc true
  field edge SheetEdge "The edge it comes from."
  field detents List.f32 "A bottom sheet's rest heights, fractions of the room; empty is 0.5 and 0.92."
  field detent u32 "The detent it rests at (keep SheetResult.detent)."
  field size f32 "A side sheet's width; 0 is 400." {
    unit cssPx
    min 0
  }
  field title String "A title at its top; empty is none."
  field dismissible bool "A drag past the lowest detent, a swipe outwards, Back or the scrim close it." {
    default true
  }
  field scrim bool "Dim what is behind it." {
    default true
  }
  field handle bool "A bottom sheet's grab handle." {
    default true
  }
  field closeButton Option.bool "A close button by the title; null is a side sheet's only."
  field closeLabel String "The close button's accessible name; empty is Close."
}
```

<sub>Module `ui` line 2226</sub>
