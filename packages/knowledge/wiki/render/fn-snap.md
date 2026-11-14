<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-snap"></a>

### `og.render.snap`

Draw entity, and everything under it through og.Parent, at its tick transform on the frames until the next tick instead of blending from where it was (RenderSettings.interpolate): call it on the tick a game teleports, respawns or resets an entity, so it does not glide across the screen. A move past RenderSettings.snapDistance snaps without it. Systems and commands may call it; it is presentation only and never reaches the world, world_hash or a snapshot.

```ts
og.render.snap(world: World, entity: Entity): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer draws it. |
| `entity` | `Entity` | no | The entity that was moved. |

**Returns** `Result<void>`: Refused for a dead entity.

Binding symbol `og_render_snap` · publish builds: kept

```ogt
fn snap {
  doc "Draw entity, and everything under it through og.Parent, at its tick transform on the frames until the next tick instead of blending from where it was (RenderSettings.interpolate): call it on the tick a game teleports, respawns or resets an entity, so it does not glide across the screen. A move past RenderSettings.snapDistance snaps without it. Systems and commands may call it; it is presentation only and never reaches the world, world_hash or a snapshot."
  param world World "The world whose renderer draws it."
  param entity Entity "The entity that was moved."
  returns Result.Void "Refused for a dead entity."
}
```

<sub>Module `render` line 811</sub>
