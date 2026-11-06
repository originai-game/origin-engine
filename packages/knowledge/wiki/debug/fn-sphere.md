<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`debug`](../debug.md) · [index](../index.md)

<a id="fn-sphere"></a>

### `og.debug.sphere`

A sphere as its three great circles.

```ts
og.debug.sphere(world: World, center: Vector3, radius: number, desc?: DebugDrawDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world drawn in. |
| `center` | `Vector3` | no | Its centre. |
| `radius` | `number` | no | Its radius. |
| `desc` | `DebugDrawDesc` | yes | Colour, width, duration, rotation. |

**Returns** `void`.

Binding symbol `og_debug_sphere` · publish builds: dropped

```ogt
fn sphere {
  doc "A sphere as its three great circles."
  param world World "The world drawn in."
  param center Vector3 "Its centre."
  param radius f32 "Its radius."
  param desc DebugDrawDesc "Colour, width, duration, rotation."
}
```

<sub>`api.ogt` line 25752</sub>
