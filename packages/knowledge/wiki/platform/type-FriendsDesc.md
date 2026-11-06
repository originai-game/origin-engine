<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="type-FriendsDesc"></a>

### `FriendsDesc` (desc struct)

How many friends.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `count` | `number` | `50` | 1 … 200 | — | — | At most this many, those playing this game first. |

```ogt
struct FriendsDesc {
  doc "How many friends."
  desc true
  field count u32 "At most this many, those playing this game first." {
    default 50
    min 1
    max 200
  }
}
```

<sub>Module `platform` line 288</sub>
