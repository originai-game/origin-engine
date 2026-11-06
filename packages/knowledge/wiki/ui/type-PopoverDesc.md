<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-PopoverDesc"></a>

### `PopoverDesc` (desc struct)

A popover beside an element, flipped where there is no room and kept on screen, with an arrow at the element.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `anchor` | `string` | `""` | — | — | — | The path of the element it belongs to (og.ui.lastPath right after building it); empty centres it. |
| `side` | `FloatSide` | `"below"` | — | — | — | The side it prefers. |
| `align` | `FloatAlign` | `"center"` | — | — | — | How it lines up with the element. |
| `arrow` | `boolean` | `true` | — | — | — | Draw an arrow at the element. |
| `offset` | `number` | `0` | — | `cssPx` | — | Extra distance from the element. |
| `title` | `string` | `""` | — | — | — | A title over the content; empty is none. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits, at most 360. |
| `padding` | `number \| null` | `null` | — | `cssPx` | — | Space inside; null is space.md. |

```ogt
struct PopoverDesc {
  doc "A popover beside an element, flipped where there is no room and kept on screen, with an arrow at the element."
  desc true
  field anchor String "The path of the element it belongs to (og.ui.lastPath right after building it); empty centres it."
  field side FloatSide "The side it prefers."
  field align FloatAlign "How it lines up with the element." {
    default center
  }
  field arrow bool "Draw an arrow at the element." {
    default true
  }
  field offset f32 "Extra distance from the element." {
    unit cssPx
  }
  field title String "A title over the content; empty is none."
  field width Option.Size "Its width; null fits, at most 360."
  field padding Option.f32 "Space inside; null is space.md." {
    unit cssPx
  }
}
```

<sub>`api.ogt` line 17898</sub>
