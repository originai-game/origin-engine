<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`debug`](../debug.md) · [index](../index.md)

<a id="fn-axes"></a>

### `og.debug.axes`

A frame's three axes, as three.js AxesHelper: +x red, +y green, +z blue (desc.color is not read).

```ts
og.debug.axes(world: World, position: Vector3, size: number, desc?: DebugDrawDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world drawn in. |
| `position` | `Vector3` | no | The frame's origin. |
| `size` | `number` | no | Each axis's length. |
| `desc` | `DebugDrawDesc` | yes | Width, duration and the frame's rotation. |

**Returns** `void`.

Binding symbol `og_debug_axes` · publish builds: dropped

```ogt
fn axes {
  doc "A frame's three axes, as three.js AxesHelper: +x red, +y green, +z blue (desc.color is not read)."
  param world World "The world drawn in."
  param position Vector3 "The frame's origin."
  param size f32 "Each axis's length."
  param desc DebugDrawDesc "Width, duration and the frame's rotation."
}
```

<sub>`api.ogt` line 25782</sub>
