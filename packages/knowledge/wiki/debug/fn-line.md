<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`debug`](../debug.md) · [index](../index.md)

<a id="fn-line"></a>

### `og.debug.line`

A line from one point to another.

```ts
og.debug.line(world: World, from: Vector3, to: Vector3, desc?: DebugDrawDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world drawn in. |
| `from` | `Vector3` | no | Where it starts. |
| `to` | `Vector3` | no | Where it ends. |
| `desc` | `DebugDrawDesc` | yes | Colour, width, duration. |

**Returns** `void`.

Binding symbol `og_debug_line` · publish builds: dropped

```ogt
fn line {
  doc "A line from one point to another."
  param world World "The world drawn in."
  param from Vector3 "Where it starts."
  param to Vector3 "Where it ends."
  param desc DebugDrawDesc "Colour, width, duration."
}
```

<sub>`api.ogt` line 25751</sub>
