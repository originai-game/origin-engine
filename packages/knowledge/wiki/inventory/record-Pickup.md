<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`inventory`](../inventory.md) · [index](../index.md)

<a id="record-Pickup"></a>

### `og.Pickup` (component)

An og.ItemStack lying in the world, with a sensor og.Collider on its entity: og.systems.inventory moves it into the og.Container of an og.Collector inside the sensor, in entity order, and despawns it once it is empty.

See `items §2`.

Also known as: pickup, loot drop, collectible, coin, item on the ground, auto pickup

Token `og.Pickup`, registered under the schema name `og.Pickup`; its value type is `Pickup`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `partial` | `boolean` | `false` | — | — | — | Take what fits and leave the rest lying; otherwise it is taken whole or not at all. |

```ogt
component Pickup {
  aliases "pickup, loot drop, collectible, coin, item on the ground, auto pickup"
  doc "An og.ItemStack lying in the world, with a sensor og.Collider on its entity: og.systems.inventory moves it into the og.Container of an og.Collector inside the sensor, in entity order, and despawns it once it is empty."
  guide "items §2"
  field partial bool "Take what fits and leave the rest lying; otherwise it is taken whole or not at all."
}
```

<sub>Module `inventory` line 31</sub>
