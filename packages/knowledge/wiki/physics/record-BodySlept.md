<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-BodySlept"></a>

### `og.BodySlept` (event)

A body fell asleep during the tick (Physics.sleepTicks), with every dynamic body it touches; sent in entity order. Stop a rolling sound, settle a pile's debris.

Token `og.BodySlept`, registered under the schema name `og.BodySlept`; its value type is `BodySlept`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The body's entity. |

```ogt
event BodySlept {
  doc "A body fell asleep during the tick (Physics.sleepTicks), with every dynamic body it touches; sent in entity order. Stop a rolling sound, settle a pile's debris."
  field entity Entity "The body's entity."
}
```

<sub>Module `physics` line 282</sub>
