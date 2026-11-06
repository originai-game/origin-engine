<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shapes`](../shapes.md) · [index](../index.md)

<a id="type-PathList"></a>

### `PathList` (interface)

A shape's holes, as three.js Shape.holes: an array; push a path to add a hole.

```ogt
interface PathList {
  doc "A shape's holes, as three.js Shape.holes: an array; push a path to add a hole."
  # members follow, each quoted below
```

<sub>`api.ogt` line 7730</sub>

<a id="member-PathList-length"></a>

#### `PathList.length` (property)

How many holes.

```ts
readonly length: number
```

```ogt
prop length u32 "How many holes." {
  host false
}
```

<sub>`api.ogt` line 7732</sub>

<a id="member-PathList-push"></a>

#### `PathList.push`

Add a hole (Array.push).

```ts
PathList.push(path: Path2): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `path` | `Path2` | no | The hole's outline, either winding. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn push {
  doc "Add a hole (Array.push)."
  host false
  param path Path2 "The hole's outline, either winding."
  returns u32
}
```

<sub>`api.ogt` line 7735</sub>
