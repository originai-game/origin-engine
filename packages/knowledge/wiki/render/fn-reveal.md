<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-reveal"></a>

### `og.render.reveal`

Set the fade-in of entity's og.Model or light when streaming brings it into view. By default an entity a cell spawned, and a model or HLOD proxy hidden while its textures were prepared, is dithered in over 0.4 s; a light ramps its intensity instead. seconds is the fade's length; 0 draws it whole at once. On a cell (og.StreamCell) it sets everything the cell spawns that has no setting of its own, and the cell's proxies. Presentation only, as og.render.snap: never in the world, world_hash or a snapshot; it lasts while the entity lives.

See `world §5`.

```ts
og.render.reveal(world: World, entity: Entity, seconds: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer draws it. |
| `entity` | `Entity` | no | The entity with the og.Model or the light, or the cell. |
| `seconds` | `number` | no | The fade's length in seconds; 0 draws the model whole at once. |

**Returns** `Result<void>`: Refused for a dead entity, or a negative or non-finite length.

Binding symbol `og_render_reveal` · publish builds: kept

```ogt
fn reveal {
  doc "Set the fade-in of entity's og.Model or light when streaming brings it into view. By default an entity a cell spawned, and a model or HLOD proxy hidden while its textures were prepared, is dithered in over 0.4 s; a light ramps its intensity instead. seconds is the fade's length; 0 draws it whole at once. On a cell (og.StreamCell) it sets everything the cell spawns that has no setting of its own, and the cell's proxies. Presentation only, as og.render.snap: never in the world, world_hash or a snapshot; it lasts while the entity lives."
  guide "world §5"
  param world World "The world whose renderer draws it."
  param entity Entity "The entity with the og.Model or the light, or the cell."
  param seconds f32 "The fade's length in seconds; 0 draws the model whole at once."
  returns Result.Void "Refused for a dead entity, or a negative or non-finite length."
}
```

<sub>Module `render` line 936</sub>
