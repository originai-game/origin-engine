<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-telegraph"></a>

### `og.ui.telegraph`

An attack telegraph to the bar of Final Fantasy XIV and WildStar: the area on the ground (disc, ring, cone or line; with desc.ground over slopes and hills, every vertex on the surface) tinted in danger with a bright edge, a fill growing from its origin to the edge as the wind-up runs so the moment of impact reads without a number, and a flash as it lands.

Parts: Telegraph.area[<shape>][hostile|friendly], Telegraph.edge, Telegraph.fill, Telegraph.flash. States: group windup (charging, impact). Accessibility: none (the game announces what is dangerous). Probe ids: <id>, <id>/Box/Area, Fill, Edge#i.

```ts
og.ui.telegraph(world: World, id: string, desc?: TelegraphDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world: its og.Camera projects the ground, so the calling system declares og.Camera among its reads. |
| `id` | `string` | no | Its id. |
| `desc` | `TelegraphDesc` | yes | The area and the wind-up. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn telegraph {
  doc "An attack telegraph to the bar of Final Fantasy XIV and WildStar: the area on the ground (disc, ring, cone or line; with desc.ground over slopes and hills, every vertex on the surface) tinted in danger with a bright edge, a fill growing from its origin to the edge as the wind-up runs so the moment of impact reads without a number, and a flash as it lands."
  doc "Parts: Telegraph.area[<shape>][hostile|friendly], Telegraph.edge, Telegraph.fill, Telegraph.flash. States: group windup (charging, impact). Accessibility: none (the game announces what is dangerous). Probe ids: <id>, <id>/Box/Area, Fill, Edge#i."
  host false
  param world World "The world: its og.Camera projects the ground, so the calling system declares og.Camera among its reads."
  param id String "Its id."
  param desc TelegraphDesc "The area and the wind-up."
  returns Void
}
```

<sub>`api.ogt` line 22410</sub>
