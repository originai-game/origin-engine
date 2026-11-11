<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`world`](../world.md) · [index](../index.md)

<a id="type-TransformDesc"></a>

### `TransformDesc` (desc struct)

How SaveEdit.transform writes what it returns.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `into` | `string` | `""` | — | — | — | The type to write the results as, one this build declares; empty: the type itself. |

```ogt
struct TransformDesc {
  doc "How SaveEdit.transform writes what it returns."
  desc true
  field into String "The type to write the results as, one this build declares; empty: the type itself."
}
```

<sub>Module `world` line 74</sub>
