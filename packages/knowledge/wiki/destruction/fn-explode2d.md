<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="fn-explode2d"></a>

### `og.destruction.explode2d`

A blast on every og.PixelTerrain in reach, this tick, as carve2d: a hole of `radius`, the ring of ground `debris` wide round it cut into Voronoi pieces and thrown outwards, the islands it cuts loose and the pieces already in reach pushed outwards. Push other bodies with og.physics.applyImpulse.

Also known as: 2D explosion, worms explosion, blast terrain

```ts
og.destruction.explode2d(world: World, desc?: Explosion2dDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `Explosion2dDesc` | yes | Its centre, reach and push. |

**Returns** `Result<void>`: Refused for a point or a size that is not finite.

Binding symbol `og_destruction_explode2d` · publish builds: kept

```ogt
fn explode2d {
  doc "A blast on every og.PixelTerrain in reach, this tick, as carve2d: a hole of `radius`, the ring of ground `debris` wide round it cut into Voronoi pieces and thrown outwards, the islands it cuts loose and the pieces already in reach pushed outwards. Push other bodies with og.physics.applyImpulse."
  aliases "2D explosion, worms explosion, blast terrain"
  param world World "The world."
  param desc Explosion2dDesc "Its centre, reach and push."
  returns Result.Void "Refused for a point or a size that is not finite."
}
```

<sub>Module `destruction` line 493</sub>
