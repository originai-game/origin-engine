<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`inventory`](../inventory.md) · [index](../index.md)

<a id="record-InventoryEntities"></a>

### `og.InventoryEntities` (resource)

Every og.Container and og.Pickup in entity order, written by og.systems.inventory's first systems each tick for its last.

Token `og.InventoryEntities`, registered under the schema name `og.InventoryEntities`; its value type is `InventoryEntities`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `containers` | `readonly Entity[]` | `[]` | — | — | a list of at most 1024 `Entity` (`List.Entity.1024`): reads a frozen copy, writes a whole array | The containers (at most 1,024). |
| `pickups` | `readonly Entity[]` | `[]` | — | — | a list of at most 1024 `Entity` (`List.Entity.1024`): reads a frozen copy, writes a whole array | The pickups (at most 1,024). |

```ogt
resource InventoryEntities {
  doc "Every og.Container and og.Pickup in entity order, written by og.systems.inventory's first systems each tick for its last."
  field containers List.Entity.1024 "The containers (at most 1,024)."
  field pickups List.Entity.1024 "The pickups (at most 1,024)."
}
```

<sub>Module `inventory` line 49</sub>
