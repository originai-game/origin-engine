<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`world`](../world.md) · [index](../index.md)

<a id="type-SetRecordsDesc"></a>

### `SetRecordsDesc` (desc struct)

How SaveEdit.setRecords writes a type.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `into` | `string` | `""` | — | — | — | The type to write them as, one this build declares; empty: the type itself (as this build declares it). |
| `records` | `readonly SaveRecord[]` | `[]` | — | — | — | The new values; a field a record leaves out takes its default, a key the type lacks is refused. |

```ogt
struct SetRecordsDesc {
  doc "How SaveEdit.setRecords writes a type."
  desc true
  field into String "The type to write them as, one this build declares; empty: the type itself (as this build declares it)."
  field records List.SaveRecord "The new values; a field a record leaves out takes its default, a key the type lacks is refused."
}
```

<sub>Module `world` line 67</sub>
