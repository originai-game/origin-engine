<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="type-PlatformError"></a>

### `PlatformError` (struct)

Why a call failed.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `code` | `string` | `""` | — | — | — | offline, timeout, invalid, unauthorized, not_found, conflict, too_large, rate_limited, unavailable, failed, cancelled or no_fill. |
| `message` | `string` | `""` | — | — | — | What happened, in words. |
| `retryable` | `boolean` | `false` | — | — | — | Whether the same call may succeed later. |
| `revision` | `number` | `0` | — | — | — | For a conflict of a conditional write: the revision the key is at now (read it again, merge, write at it); 0 otherwise. |

```ogt
struct PlatformError {
  doc "Why a call failed."
  field code String "offline, timeout, invalid, unauthorized, not_found, conflict, too_large, rate_limited, unavailable, failed, cancelled or no_fill."
  field message String "What happened, in words."
  field retryable bool "Whether the same call may succeed later."
  field revision u32 "For a conflict of a conditional write: the revision the key is at now (read it again, merge, write at it); 0 otherwise."
}
```

<sub>`api.ogt` line 14915</sub>
