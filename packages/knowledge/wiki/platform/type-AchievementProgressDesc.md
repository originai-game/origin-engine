<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="type-AchievementProgressDesc"></a>

### `AchievementProgressDesc` (desc struct)

Progress towards an achievement.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | The achievement's id (1 to 64 of letters, digits, `.`, `_` and `-`). |
| `percent` | `number` | `0` | 0 … 100 | — | — | How far, 0 to 100: 100 unlocks it. Progress never goes back (a lower value keeps the higher one). |

```ogt
struct AchievementProgressDesc {
  doc "Progress towards an achievement."
  desc true
  field id String "The achievement's id (1 to 64 of letters, digits, `.`, `_` and `-`)."
  field percent f32 "How far, 0 to 100: 100 unlocks it. Progress never goes back (a lower value keeps the higher one)." {
    min 0
    max 100
  }
}
```

<sub>`api.ogt` line 15185</sub>
