<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ControlSize"></a>

### `ControlSize` (enum)

Control heights from the density (h-control-sm, -md, -lg).

A string: one of the variants below. The zero value is `"medium"`.

| Variant | Doc |
|---|---|
| `"medium"` | The medium control height. |
| `"small"` | The small control height. |
| `"large"` | The large control height. |

```ogt
enum ControlSize {
  doc "Control heights from the density (h-control-sm, -md, -lg)."
  variant medium "The medium control height."
  variant small "The small control height."
  variant large "The large control height."
}
```

<sub>`api.ogt` line 15610</sub>
