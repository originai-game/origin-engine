<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="record-PlatformAnswered"></a>

### `og.PlatformAnswered` (event)

An answer arrived: sent by og.platform at the start of the tick it was applied, read by systems the tick after (og.platform.result has it at once).

Token `og.PlatformAnswered`, registered under the schema name `og.PlatformAnswered`; its value type is `PlatformAnswered`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `request` | `number` | `0` | — | — | — | The call's request number. |
| `ok` | `boolean` | `false` | — | — | — | Whether it succeeded; og.platform.result has the data or the error. |

```ogt
event PlatformAnswered {
  doc "An answer arrived: sent by og.platform at the start of the tick it was applied, read by systems the tick after (og.platform.result has it at once)."
  field request u32 "The call's request number."
  field ok bool "Whether it succeeded; og.platform.result has the data or the error."
}
```

<sub>`api.ogt` line 14915</sub>
