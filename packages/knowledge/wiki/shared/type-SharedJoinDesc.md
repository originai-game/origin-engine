<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="type-SharedJoinDesc"></a>

### `SharedJoinDesc` (desc struct)

Which world to join (og.shared.join).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `world` | `string` | `"main"` | — | — | — | The world's name on the server (at most 64 of letters, digits, `.`, `_` and `-`). |

```ogt
struct SharedJoinDesc {
  doc "Which world to join (og.shared.join)."
  desc true
  field world String "The world's name on the server (at most 64 of letters, digits, `.`, `_` and `-`)." {
    default "main"
  }
}
```

<sub>Module `shared` line 119</sub>
