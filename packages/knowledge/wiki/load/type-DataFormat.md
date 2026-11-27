<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="type-DataFormat"></a>

### `DataFormat` (enum)

How og.load.data reads a file.

A string: one of the variants below. The zero value is `"json"`.

| Variant | Doc |
|---|---|
| `"json"` | JSON: og.load.dataJson gives the value. |
| `"bytes"` | Bytes as stored: og.load.dataBytes gives them. |

```ogt
enum DataFormat {
  doc "How og.load.data reads a file."
  variant json "JSON: og.load.dataJson gives the value."
  variant bytes "Bytes as stored: og.load.dataBytes gives them."
}
```

<sub>Module `load` line 594</sub>
