<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`inventory`](../inventory.md) · [index](../index.md)

<a id="record-PickedUp"></a>

### `og.PickedUp` (event)

An og.Collector took some of an og.Pickup: sent by og.systems.inventory the tick it was moved.

Also known as: on pickup, item collected, loot taken

Token `og.PickedUp`, registered under the schema name `og.PickedUp`; its value type is `PickedUp`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `collector` | `Entity` | `0` | — | — | — | The og.Collector. |
| `pickup` | `Entity` | `0` | — | — | — | The og.Pickup; despawned when og.inventory returns once it is empty. |
| `item` | `TableRow` | `0` | — | — | — | The row taken. |
| `count` | `number` | `0` | — | — | — | How many were moved. |

```ogt
event PickedUp {
  aliases "on pickup, item collected, loot taken"
  doc "An og.Collector took some of an og.Pickup: sent by og.systems.inventory the tick it was moved."
  field collector Entity "The og.Collector."
  field pickup Entity "The og.Pickup; despawned when og.inventory returns once it is empty."
  field item TableRow "The row taken."
  field count u32 "How many were moved."
}
```

<sub>Module `inventory` line 40</sub>
