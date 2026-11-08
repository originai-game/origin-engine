<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-shapeBox"></a>

### `og.ui.shapeBox`

A decorative shape drawn by the UI pass (docs/ui.md §3.3) in the style's colours: a pip, a gem, a star, a ring, a backdrop blob. It takes no input and is decoration to a screen reader. Part: `Shape.<kind>` (a style dresses `Shape.star` with art). Probe: `<id>`.

```ts
og.ui.shapeBox(id: string, desc?: ShapeBoxDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ShapeBoxDesc` | yes | Its outline and paints. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn shapeBox {
  doc "A decorative shape drawn by the UI pass (docs/ui.md §3.3) in the style's colours: a pip, a gem, a star, a ring, a backdrop blob. It takes no input and is decoration to a screen reader. Part: `Shape.<kind>` (a style dresses `Shape.star` with art). Probe: `<id>`."
  host false
  param id String "Its id."
  param desc ShapeBoxDesc "Its outline and paints."
  returns Void
}
```

<sub>Module `ui` line 3355</sub>
