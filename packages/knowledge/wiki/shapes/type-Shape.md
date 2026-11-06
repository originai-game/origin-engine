<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shapes`](../shapes.md) · [index](../index.md)

<a id="type-Shape"></a>

### `Shape` (interface)

An outline with holes, as three.js Shape.

Also known as: three.js Shape, 2D path, polygon with holes, outline, Godot Path2D, vector shape

Extends [`Path2`](../shapes/type-Path2.md#type-Path2): every member of it is a member here.

```ogt
interface Shape {
  doc "An outline with holes, as three.js Shape."
  # members follow, each quoted below
```

<sub>Module `shapes` line 143</sub>

<a id="member-Shape-holes"></a>

#### `Shape.holes` (property)

The holes: `shape.holes.push(hole)`.

```ts
readonly holes: PathList
```

```ogt
prop holes PathList "The holes: `shape.holes.push(hole)`." {
  host false
}
```

<sub>Module `shapes` line 147</sub>

<a id="member-Shape-extractPoints"></a>

#### `Shape.extractPoints`

The outline's and the holes' points, as three.js Shape.extractPoints.

```ts
Shape.extractPoints(divisions?: number): ShapePoints
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `divisions` | `number` | yes | Divisions per curve. Default `12`. |

**Returns** `ShapePoints`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn extractPoints {
  doc "The outline's and the holes' points, as three.js Shape.extractPoints."
  host false
  param divisions u32 "Divisions per curve." {
    default 12
  }
  returns ShapePoints
}
```

<sub>Module `shapes` line 150</sub>
