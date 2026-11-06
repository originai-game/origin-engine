<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="type-AdDesc"></a>

### `AdDesc` (desc struct)

An ad to show.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `AdKind` | `"rewarded"` | — | — | — | Rewarded or interstitial (default rewarded). |
| `placement` | `string` | `"default"` | — | — | — | Where in the game, as the game names it (`level.end`, `continue`): 1 to 64 of letters, digits, `.`, `_` and `-` (default `default`). |

```ogt
struct AdDesc {
  doc "An ad to show."
  desc true
  field kind AdKind "Rewarded or interstitial (default rewarded)."
  field placement String "Where in the game, as the game names it (`level.end`, `continue`): 1 to 64 of letters, digits, `.`, `_` and `-` (default `default`)." {
    default "default"
  }
}
```

<sub>`api.ogt` line 15168</sub>
