<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`debug`](../debug.md) · [index](../index.md)

<a id="fn-box"></a>

### `og.debug.box`

A box's twelve edges, as three.js Box3Helper (turned by desc.rotation).

```ts
og.debug.box(world: World, center: Vector3, size: Vector3, desc?: DebugDrawDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world drawn in. |
| `center` | `Vector3` | no | Its centre. |
| `size` | `Vector3` | no | Its extent along each axis. |
| `desc` | `DebugDrawDesc` | yes | Colour, width, duration, rotation. |

**Returns** `void`.

Binding symbol `og_debug_box` · publish builds: dropped

```ogt
fn box {
  doc "A box's twelve edges, as three.js Box3Helper (turned by desc.rotation)."
  param world World "The world drawn in."
  param center Vector3 "Its centre."
  param size Vector3 "Its extent along each axis."
  param desc DebugDrawDesc "Colour, width, duration, rotation."
}
```

<sub>Module `debug` line 30</sub>
