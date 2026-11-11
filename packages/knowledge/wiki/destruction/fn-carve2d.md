<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="fn-carve2d"></a>

### `og.destruction.carve2d`

Empty a round hole in an og.PixelTerrain, this tick (applied by og.destruction2d after the game's systems, in the order called): the chunks it touches collide as their new outline before this tick's physics step, and what it cuts loose falls.

Also known as: dig, carve terrain, crater, destroy ground

```ts
og.destruction.carve2d(world: World, entity: Entity, desc?: Carve2dDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with og.PixelTerrain. |
| `desc` | `Carve2dDesc` | yes | Where and how big. |

**Returns** `Result<void>`: Refused for an entity without og.PixelTerrain, or a point or radius that is not finite.

Binding symbol `og_destruction_carve2d` · publish builds: kept

```ogt
fn carve2d {
  doc "Empty a round hole in an og.PixelTerrain, this tick (applied by og.destruction2d after the game's systems, in the order called): the chunks it touches collide as their new outline before this tick's physics step, and what it cuts loose falls."
  aliases "dig, carve terrain, crater, destroy ground"
  param world World "The world."
  param entity Entity "An entity with og.PixelTerrain."
  param desc Carve2dDesc "Where and how big."
  returns Result.Void "Refused for an entity without og.PixelTerrain, or a point or radius that is not finite."
}
```

<sub>Module `destruction` line 483</sub>
