<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-light2dAt"></a>

### `og.render.light2dAt`

The 2D light reaching a point, for game rules: the sum of every og.Light2D whose layers share a bit with `layers`, each its color times intensity times its falloff (and a spot's cone), blocked when castShadows and an og.Occluder2D on its layers stands between it and the point (not one holding the point or the light). Ambient light is not included. Read at world transforms as of the call.

The same bits on every host (og-core's deterministic math, never the GPU), so a system may store it. Inside a system, declare reads of og.Light2D, og.Occluder2D, og.Transform and og.Parent.

Also known as: is the player lit, standing in the dark, side-view stealth game, light level at a point, 2D light probe

```ts
og.render.light2dAt(world: World, point: Vector2, layers?: number): Result<Color>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world asked. |
| `point` | `Vector2` | no | The point in the 2D plane, in world units. |
| `layers` | `number` | yes | The light layers the point is on, a bit per layer as og.LightMask2D.layers. Default `1`. |

**Returns** `Result<Color>`: Refused for a non-finite point, or inside a system that did not declare the reads.

Binding symbol `og_render_light2d_at` · publish builds: kept

```ogt
fn light2dAt {
  aliases "is the player lit, standing in the dark, side-view stealth game, light level at a point, 2D light probe"
  doc "The 2D light reaching a point, for game rules: the sum of every og.Light2D whose layers share a bit with `layers`, each its color times intensity times its falloff (and a spot's cone), blocked when castShadows and an og.Occluder2D on its layers stands between it and the point (not one holding the point or the light). Ambient light is not included. Read at world transforms as of the call."
  doc "The same bits on every host (og-core's deterministic math, never the GPU), so a system may store it. Inside a system, declare reads of og.Light2D, og.Occluder2D, og.Transform and og.Parent."
  param world World "The world asked."
  param point Vector2 "The point in the 2D plane, in world units."
  param layers u32 "The light layers the point is on, a bit per layer as og.LightMask2D.layers." {
    default 1
  }
  returns Result.Color "Refused for a non-finite point, or inside a system that did not declare the reads."
}
```

<sub>Module `render` line 3923</sub>
