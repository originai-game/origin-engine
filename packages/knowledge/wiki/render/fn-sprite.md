<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-sprite"></a>

### `og.render.sprite`

Draw a sprite this frame. Systems and commands may call it; the list is cleared at every tick, so submit it every tick the sprite should show.

Also known as: draw a 2D image, immediate sprite, blit

```ts
og.render.sprite(world: World, desc?: SpriteDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer draws it. |
| `desc` | `SpriteDesc` | yes | The texture, placement, region and colour. |

**Returns** `Result<void>`: Refused for a dead texture.

Binding symbol `og_render_sprite` · publish builds: kept

```ogt
fn sprite {
  doc "Draw a sprite this frame. Systems and commands may call it; the list is cleared at every tick, so submit it every tick the sprite should show."
  aliases "draw a 2D image, immediate sprite, blit"
  param world World "The world whose renderer draws it."
  param desc SpriteDesc "The texture, placement, region and colour."
  returns Result.Void "Refused for a dead texture."
}
```

<sub>Module `render` line 2282</sub>
