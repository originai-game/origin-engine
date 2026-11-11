<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="type-LeaderboardSubmitDesc"></a>

### `LeaderboardSubmitDesc` (desc struct)

A score.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `board` | `string` | `"main"` | — | — | — | The board (default `main`). |
| `score` | `number \| null` | `null` | — | — | — | The score, or a time in seconds for an ascending board. Required: a desc without one is refused, never submitted as 0. |
| `order` | `PlatformOrder` | `"desc"` | — | — | — | How the board sorts, fixed by its first submission. |

```ogt
struct LeaderboardSubmitDesc {
  doc "A score."
  desc true
  field board String "The board (default `main`)." {
    default "main"
  }
  field score Option.f64 "The score, or a time in seconds for an ascending board. Required: a desc without one is refused, never submitted as 0."
  field order PlatformOrder "How the board sorts, fixed by its first submission."
}
```

<sub>Module `platform` line 30</sub>
