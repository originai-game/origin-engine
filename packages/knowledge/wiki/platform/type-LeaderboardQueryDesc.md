<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="type-LeaderboardQueryDesc"></a>

### `LeaderboardQueryDesc` (desc struct)

Which entries.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `board` | `string` | `"main"` | — | — | — | The board (default `main`). |
| `count` | `number` | `10` | 1 … 100 | — | — | How many entries: the top `count`, or `count` around the player. |

```ogt
struct LeaderboardQueryDesc {
  doc "Which entries."
  desc true
  field board String "The board (default `main`)." {
    default "main"
  }
  field count u32 "How many entries: the top `count`, or `count` around the player." {
    default 10
    min 1
    max 100
  }
}
```

<sub>`api.ogt` line 14947</sub>
