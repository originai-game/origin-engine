<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="type-PointDesc"></a>

### `PointDesc` (desc struct)

Origin Point for something the player did.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `reason` | `string` | `""` | — | — | — | Why, as the game names it: `level.completed`, `daily.streak` (1 to 64 of letters, digits, `.`, `_` and `-`). |
| `amount` | `number` | `1` | 1 … 1000 | — | — | How many points the game asks for; the platform decides what it grants. |

```ogt
struct PointDesc {
  doc "Origin Point for something the player did."
  desc true
  field reason String "Why, as the game names it: `level.completed`, `daily.streak` (1 to 64 of letters, digits, `.`, `_` and `-`)."
  field amount u32 "How many points the game asks for; the platform decides what it grants." {
    default 1
    min 1
    max 1000
  }
}
```

<sub>Module `platform` line 49</sub>
