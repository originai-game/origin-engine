<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shapes`](../shapes.md) · [index](../index.md)

<a id="type-ShapeClass"></a>

### `ShapeClass` (interface)

og.Shape: `new og.Shape(points?)`.

```ogt
interface ShapeClass {
  doc "og.Shape: `new og.Shape(points?)`."
  # members follow, each quoted below
```

<sub>`api.ogt` line 7868</sub>

<a id="member-ShapeClass-new"></a>

#### `ShapeClass.new`

A shape, through the points when given (setFromPoints).

```ts
ShapeClass.new(points?: readonly Vector2[] | null): Shape
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `points` | `readonly Vector2[] \| null` | yes | Points to start from. |

**Returns** `Shape`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn new {
  doc "A shape, through the points when given (setFromPoints)."
  host false
  param points Option.List.Vector2 "Points to start from." {
    optional true
  }
  returns Shape
}
```

<sub>`api.ogt` line 7870</sub>
