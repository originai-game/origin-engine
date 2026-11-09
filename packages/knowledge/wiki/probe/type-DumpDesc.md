<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-DumpDesc"></a>

### `DumpDesc` (desc struct)

What to dump.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `what` | `DumpKind` | `"world"` | — | — | — | The kind of dump. |
| `id` | `string` | `""` | — | — | — | The probe id, for an entity dump. |

```ogt
struct DumpDesc {
  doc "What to dump."
  desc true
  field what DumpKind "The kind of dump."
  field id String "The probe id, for an entity dump."
}
```

<sub>Module `probe` line 380</sub>
