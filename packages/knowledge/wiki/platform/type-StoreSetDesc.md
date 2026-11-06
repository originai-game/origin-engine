<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="type-StoreSetDesc"></a>

### `StoreSetDesc` (desc struct)

A write to the store.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | The key: 1 to 64 of letters, digits, `.`, `_` and `-`. |
| `value` | `unknown` | `null` | — | — | — | Any JSON value, at most 64 KiB encoded. |
| `revision` | `number` | `0` | — | — | — | Write only if the key is at this revision (a conflict otherwise, whose error carries the revision the key is at now); 0 writes whatever is there (the last write wins). |
| `create` | `boolean` | `false` | — | — | — | Write only if the key has never been written (a conflict otherwise); revision is then ignored. |

```ogt
struct StoreSetDesc {
  doc "A write to the store."
  desc true
  field key String "The key: 1 to 64 of letters, digits, `.`, `_` and `-`."
  field value Json "Any JSON value, at most 64 KiB encoded."
  field revision u32 "Write only if the key is at this revision (a conflict otherwise, whose error carries the revision the key is at now); 0 writes whatever is there (the last write wins)."
  field create bool "Write only if the key has never been written (a conflict otherwise); revision is then ignored."
}
```

<sub>`api.ogt` line 14849</sub>
