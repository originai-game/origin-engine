<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="type-PlatformResult"></a>

### `PlatformResult` (struct)

The answer to one call.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `request` | `number` | `0` | — | — | — | The call's request number. |
| `method` | `string` | `""` | — | — | — | Its method, such as `store.get`. |
| `ok` | `boolean` | `false` | — | — | — | Whether it succeeded. |
| `data` | `unknown` | `null` | — | — | — | The answer (see each function); null on failure. |
| `error` | `PlatformError` | `{ code: "", message: "", retryable: false, revision: 0 }` | — | — | optional | Why it failed. |
| `tick` | `number` | `0` | — | — | — | The tick it was answered at. |
| `time` | `number` | `0` | — | `ms` | — | The host's clock when it answered, UTC milliseconds since 1970 (PlatformInfo.clock says whose); 0 when the host gave none. A slot call (saveSlot, loadSlot) answers the time of the store answer that ended it. |

```ogt
struct PlatformResult {
  doc "The answer to one call."
  field request u32 "The call's request number."
  field method String "Its method, such as `store.get`."
  field ok bool "Whether it succeeded."
  field data Json "The answer (see each function); null on failure."
  field error PlatformError "Why it failed." {
    optional true
  }
  field tick u32 "The tick it was answered at."
  field time f64 "The host's clock when it answered, UTC milliseconds since 1970 (PlatformInfo.clock says whose); 0 when the host gave none. A slot call (saveSlot, loadSlot) answers the time of the store answer that ended it." {
    unit ms
  }
}
```

<sub>Module `platform` line 91</sub>
