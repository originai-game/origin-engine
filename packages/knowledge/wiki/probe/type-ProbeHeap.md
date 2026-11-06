<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-ProbeHeap"></a>

### `ProbeHeap` (struct)

The engine's allocator, counted (both hosts install a counting global allocator).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `live` | `number` | `0` | — | `bytes` | — | Bytes allocated and not freed. |
| `peak` | `number` | `0` | — | `bytes` | — | The most bytes live at once since load. |
| `allocations` | `number` | `0` | — | — | — | Allocations made since load. |

```ogt
struct ProbeHeap {
  doc "The engine's allocator, counted (both hosts install a counting global allocator)."
  field live f64 "Bytes allocated and not freed." {
    unit bytes
  }
  field peak f64 "The most bytes live at once since load." {
    unit bytes
  }
  field allocations f64 "Allocations made since load."
}
```

<sub>`api.ogt` line 24115</sub>
