<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="record-PlatformNotice"></a>

### `og.PlatformNotice` (event)

Unasked host input arrived: sent by og.platform at the start of the tick it was applied, read by systems the tick after; read it with og.platform.notice. Registered when a game first names it (a system's reads); without it notices are still logged and readable.

Token `og.PlatformNotice`, registered under the schema name `og.PlatformNotice`; its value type is `PlatformNotice`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `notice` | `number` | `0` | — | — | — | The notice's number, for og.platform.notice: 1 for the first, counting up. |
| `kind` | `PlatformNoticeKind` | `"purchase"` | — | — | — | What it is about. |

```ogt
event PlatformNotice {
  doc "Unasked host input arrived: sent by og.platform at the start of the tick it was applied, read by systems the tick after; read it with og.platform.notice. Registered when a game first names it (a system's reads); without it notices are still logged and readable."
  field notice u32 "The notice's number, for og.platform.notice: 1 for the first, counting up."
  field kind PlatformNoticeKind "What it is about."
}
```

<sub>`api.ogt` line 15054</sub>
