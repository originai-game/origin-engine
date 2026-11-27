<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`inventory`](../inventory.md) · [index](../index.md)

<a id="record-Collector"></a>

### `og.Collector` (component)

What takes an og.Pickup whose sensor it is inside (its entity's collider): into `container`.

Also known as: picker, auto loot, magnet, player pickup, vacuum items

Token `og.Collector`, registered under the schema name `og.Collector`; its value type is `Collector`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `container` | `Entity` | `0` | — | — | — | The og.Container what it picks up goes into; 0 picks up nothing. |

```ogt
component Collector {
  aliases "picker, auto loot, magnet, player pickup, vacuum items"
  doc "What takes an og.Pickup whose sensor it is inside (its entity's collider): into `container`."
  field container Entity "The og.Container what it picks up goes into; 0 picks up nothing."
}
```

<sub>Module `inventory` line 34</sub>
