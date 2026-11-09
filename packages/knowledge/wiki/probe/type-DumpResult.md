<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-DumpResult"></a>

### `DumpResult` (union)

A dump, tagged by its kind.

Tagged by `kind`.

| Case | Payload | Doc |
|---|---|---|
| `kind: "world"` | [`WorldDump`](../probe/type-WorldDump.md#type-WorldDump) | The world dump. |
| `kind: "entity"` | [`EntityDump`](../probe/type-EntityDump.md#type-EntityDump) | An entity dump. |
| `kind: "ui"` | [`UiDump`](../probe/type-UiDump.md#type-UiDump) | The UI dump. |
| `kind: "desync"` | [`DesyncDump`](../probe/type-DesyncDump.md#type-DesyncDump) | The desync dump. |

```ogt
union DumpResult {
  doc "A dump, tagged by its kind."
  tag kind
  case world WorldDump "The world dump."
  case entity EntityDump "An entity dump."
  case ui UiDump "The UI dump."
  case desync DesyncDump "The desync dump."
}
```

<sub>Module `probe` line 494</sub>
