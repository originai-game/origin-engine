<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-BodyWoke"></a>

### `og.BodyWoke` (event)

A sleeping body woke during the tick: something touched, pushed or moved it, game code wrote to it, or sleep was turned off; sent in entity order.

Token `og.BodyWoke`, registered under the schema name `og.BodyWoke`; its value type is `BodyWoke`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The body's entity. |

```ogt
event BodyWoke {
  doc "A sleeping body woke during the tick: something touched, pushed or moved it, game code wrote to it, or sleep was turned off; sent in entity order."
  field entity Entity "The body's entity."
}
```

<sub>`api.ogt` line 8780</sub>
