<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-sheet"></a>

### `og.ui.sheet`

A sheet sliding in from the bottom (resting at detents, dragged) or a side.

```ts
og.ui.sheet(id: string, open: boolean, desc: SheetDesc, build: UiBuild): SheetResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `open` | `boolean` | no | Whether it is open. |
| `desc` | `SheetDesc` | no | Edge, detents and title. |
| `build` | `UiBuild` | no | Builds its body. |

**Returns** `SheetResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn sheet {
  doc "A sheet sliding in from the bottom (resting at detents, dragged) or a side."
  host false
  param id String "Its id."
  param open bool "Whether it is open."
  param desc SheetDesc "Edge, detents and title."
  param build UiBuild "Builds its body."
  returns SheetResult
}
```

<sub>Module `ui` line 1450</sub>
