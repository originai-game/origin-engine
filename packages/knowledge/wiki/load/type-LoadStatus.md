<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="type-LoadStatus"></a>

### `LoadStatus` (enum)

Where a load is.

A string: one of the variants below. The zero value is `"pending"`.

| Variant | Doc |
|---|---|
| `"pending"` | Still loading. |
| `"ready"` | Loaded; it can be instantiated. |
| `"failed"` | Failed; og.load.error gives the reason. |

```ogt
enum LoadStatus {
  doc "Where a load is."
  variant pending "Still loading."
  variant ready "Loaded; it can be instantiated."
  variant failed "Failed; og.load.error gives the reason."
}
```

<sub>Module `load` line 48</sub>
