<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Cursor"></a>

### `Cursor` (enum)

The pointer's shape over an element (CSS cursor's keywords, CSS UI 4 §5.1, docs/ui.md §2), inherited by its subtree.

A string: one of the variants below. The zero value is `"auto"`.

| Variant | Doc |
|---|---|
| `"auto"` | Its parent's; with none set above it, a text control's I-beam, a draggable element's open hand (closed while dragged), a link's hand, else the arrow. |
| `"default"` | The arrow. |
| `"pointer"` | The hand (a link). |
| `"text"` | The I-beam. |
| `"grab"` | An open hand. |
| `"grabbing"` | A closed hand. |
| `"move"` | The move arrows. |
| `"notAllowed"` | The slashed circle. |
| `"crosshair"` | The crosshair. |
| `"help"` | The question mark. |
| `"wait"` | Busy. |
| `"progress"` | Busy, still taking input. |
| `"resizeEw"` | Left and right. |
| `"resizeNs"` | Up and down. |
| `"resizeNesw"` | Up right and down left. |
| `"resizeNwse"` | Up left and down right. |
| `"zoomIn"` | The magnifier with a plus. |
| `"zoomOut"` | The magnifier with a minus. |
| `"none"` | Hidden (a game drawing its own). |
| `"contextMenu"` | A menu is available. |
| `"cell"` | A cell or cells can be selected. |
| `"verticalText"` | The I-beam of vertical text. |
| `"alias"` | An alias or shortcut is made. |
| `"copy"` | A copy is made. |
| `"noDrop"` | It cannot be dropped here. |
| `"allScroll"` | Scrolls in any direction. |
| `"resizeCol"` | A column resizes left and right. |
| `"resizeRow"` | A row resizes up and down. |
| `"resizeN"` | Its top edge moves. |
| `"resizeE"` | Its right edge moves. |
| `"resizeS"` | Its bottom edge moves. |
| `"resizeW"` | Its left edge moves. |
| `"resizeNe"` | Its top right corner moves. |
| `"resizeNw"` | Its top left corner moves. |
| `"resizeSe"` | Its bottom right corner moves. |
| `"resizeSw"` | Its bottom left corner moves. |

```ogt
enum Cursor {
  doc "The pointer's shape over an element (CSS cursor's keywords, CSS UI 4 §5.1, docs/ui.md §2), inherited by its subtree."
  variant auto "Its parent's; with none set above it, a text control's I-beam, a draggable element's open hand (closed while dragged), a link's hand, else the arrow."
  variant default "The arrow."
  variant pointer "The hand (a link)."
  variant text "The I-beam."
  variant grab "An open hand."
  variant grabbing "A closed hand."
  variant move "The move arrows."
  variant notAllowed "The slashed circle."
  variant crosshair "The crosshair."
  variant help "The question mark."
  variant wait "Busy."
  variant progress "Busy, still taking input."
  variant resizeEw "Left and right."
  variant resizeNs "Up and down."
  variant resizeNesw "Up right and down left."
  variant resizeNwse "Up left and down right."
  variant zoomIn "The magnifier with a plus."
  variant zoomOut "The magnifier with a minus."
  variant none "Hidden (a game drawing its own)."
  variant contextMenu "A menu is available."
  variant cell "A cell or cells can be selected."
  variant verticalText "The I-beam of vertical text."
  variant alias "An alias or shortcut is made."
  variant copy "A copy is made."
  variant noDrop "It cannot be dropped here."
  variant allScroll "Scrolls in any direction."
  variant resizeCol "A column resizes left and right."
  variant resizeRow "A row resizes up and down."
  variant resizeN "Its top edge moves."
  variant resizeE "Its right edge moves."
  variant resizeS "Its bottom edge moves."
  variant resizeW "Its left edge moves."
  variant resizeNe "Its top right corner moves."
  variant resizeNw "Its top left corner moves."
  variant resizeSe "Its bottom right corner moves."
  variant resizeSw "Its bottom left corner moves."
}
```

<sub>Module `ui` line 876</sub>
