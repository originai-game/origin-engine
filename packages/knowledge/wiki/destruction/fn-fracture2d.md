<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="fn-fracture2d"></a>

### `og.destruction.fracture2d`

Break an og.PixelPiece (or every cell of an og.PixelTerrain but its bedrock) into Voronoi pieces, this tick, as carve2d: each piece is an og.PixelPiece moving as what it broke from moved; the piece broken is despawned.

Also known as: shatter 2D, break sprite, voronoi 2D

```ts
og.destruction.fracture2d(world: World, entity: Entity, desc?: Fracture2dDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with og.PixelPiece or og.PixelTerrain. |
| `desc` | `Fracture2dDesc` | yes | How many pieces. |

**Returns** `Result<void>`: Refused for an entity with neither.

Binding symbol `og_destruction_fracture2d` · publish builds: kept

```ogt
fn fracture2d {
  doc "Break an og.PixelPiece (or every cell of an og.PixelTerrain but its bedrock) into Voronoi pieces, this tick, as carve2d: each piece is an og.PixelPiece moving as what it broke from moved; the piece broken is despawned."
  aliases "shatter 2D, break sprite, voronoi 2D"
  param world World "The world."
  param entity Entity "An entity with og.PixelPiece or og.PixelTerrain."
  param desc Fracture2dDesc "How many pieces."
  returns Result.Void "Refused for an entity with neither."
}
```

<sub>Module `destruction` line 501</sub>
