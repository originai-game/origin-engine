<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`debug`](../debug.md) · [index](../index.md)

<a id="fn-arrow"></a>

### `og.debug.arrow`

An arrow from one point to another, its head a fifth of its length (three.js ArrowHelper): a velocity, a force, a direction to a target.

```ts
og.debug.arrow(world: World, from: Vector3, to: Vector3, desc?: DebugDrawDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world drawn in. |
| `from` | `Vector3` | no | Its tail. |
| `to` | `Vector3` | no | Its tip. |
| `desc` | `DebugDrawDesc` | yes | Colour, width, duration. |

**Returns** `void`.

Binding symbol `og_debug_arrow` · publish builds: dropped

```ogt
fn arrow {
  doc "An arrow from one point to another, its head a fifth of its length (three.js ArrowHelper): a velocity, a force, a direction to a target."
  param world World "The world drawn in."
  param from Vector3 "Its tail."
  param to Vector3 "Its tip."
  param desc DebugDrawDesc "Colour, width, duration."
}
```

<sub>`api.ogt` line 25775</sub>
