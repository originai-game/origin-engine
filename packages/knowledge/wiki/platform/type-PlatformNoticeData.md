<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="type-PlatformNoticeData"></a>

### `PlatformNoticeData` (struct)

A notice og.PlatformNotice announced.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `notice` | `number` | `0` | — | — | — | Its number. |
| `kind` | `string` | `""` | — | — | — | `purchase`, `entitlements` or `invite`. |
| `data` | `unknown` | `null` | — | — | — | What it carries (see og.PlatformNoticeKind). |
| `tick` | `number` | `0` | — | — | — | The tick it was applied at. |
| `time` | `number` | `0` | — | `ms` | — | The host's clock when it sent it, UTC milliseconds; 0 when it gave none. |

```ogt
struct PlatformNoticeData {
  doc "A notice og.PlatformNotice announced."
  field notice u32 "Its number."
  field kind String "`purchase`, `entitlements` or `invite`."
  field data Json "What it carries (see og.PlatformNoticeKind)."
  field tick u32 "The tick it was applied at."
  field time f64 "The host's clock when it sent it, UTC milliseconds; 0 when it gave none." {
    unit ms
  }
}
```

<sub>`api.ogt` line 15148</sub>
