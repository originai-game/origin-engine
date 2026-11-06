<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-WfcTile"></a>

### `WfcTile` (struct)

A tile for og.procgen.wfc: its weight and the socket on each edge. Two tiles may sit side by side when the sockets on their shared edge are equal (a road edge meets a road edge).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `weight` | `number` | `1` | ≥ 0 | — | — | How often it is drawn against the others that fit. |
| `up` | `number` | `0` | — | — | — | The socket on its top edge. |
| `right` | `number` | `0` | — | — | — | The socket on its right edge. |
| `down` | `number` | `0` | — | — | — | The socket on its bottom edge. |
| `left` | `number` | `0` | — | — | — | The socket on its left edge. |
| `rotations` | `boolean` | `false` | — | — | — | It also comes turned 90°, 180° and 270° clockwise (its sockets with it; turns that look the same count once). |

```ogt
struct WfcTile {
  doc "A tile for og.procgen.wfc: its weight and the socket on each edge. Two tiles may sit side by side when the sockets on their shared edge are equal (a road edge meets a road edge)."
  field weight f32 "How often it is drawn against the others that fit." {
    default 1
    min 0
  }
  field up u32 "The socket on its top edge."
  field right u32 "The socket on its right edge."
  field down u32 "The socket on its bottom edge."
  field left u32 "The socket on its left edge."
  field rotations bool "It also comes turned 90°, 180° and 270° clockwise (its sockets with it; turns that look the same count once)."
}
```

<sub>`api.ogt` line 1944</sub>
