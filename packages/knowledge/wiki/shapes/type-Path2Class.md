<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shapes`](../shapes.md) · [index](../index.md)

<a id="type-Path2Class"></a>

### `Path2Class` (interface)

og.Path2: `new og.Path2(points?)`.

```ogt
interface Path2Class {
  doc "og.Path2: `new og.Path2(points?)`."
  # members follow, each quoted below
```

<sub>`api.ogt` line 7883</sub>

<a id="member-Path2Class-new"></a>

#### `Path2Class.new`

A path, through the points when given (setFromPoints).

```ts
Path2Class.new(points?: readonly Vector2[] | null): Path2
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `points` | `readonly Vector2[] \| null` | yes | Points to start from. |

**Returns** `Path2`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn new {
  doc "A path, through the points when given (setFromPoints)."
  host false
  param points Option.List.Vector2 "Points to start from." {
    optional true
  }
  returns Path2
}
```

<sub>`api.ogt` line 7885</sub>
