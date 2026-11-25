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
| `kind: "drawn"` | [`DrawnDump`](../probe/type-DrawnDump.md#type-DrawnDump) | The cull record. |
| `kind: "layers"` | [`LayersDump`](../probe/type-LayersDump.md#type-LayersDump) | The UI layer stack (`ui:layers`). |
| `kind: "motion"` | [`MotionDump`](../probe/type-MotionDump.md#type-MotionDump) | The UI motion (`ui:motion`). |
| `kind: "subsystem"` | [`SubsystemDump`](../probe/type-SubsystemDump.md#type-SubsystemDump) | A registered subsystem dump, by its name. |

```ogt
union DumpResult {
  doc "A dump, tagged by its kind."
  tag kind
  case world WorldDump "The world dump."
  case entity EntityDump "An entity dump."
  case ui UiDump "The UI dump."
  case desync DesyncDump "The desync dump."
  case drawn DrawnDump "The cull record."
  case layers LayersDump "The UI layer stack (`ui:layers`)."
  case motion MotionDump "The UI motion (`ui:motion`)."
  case subsystem SubsystemDump "A registered subsystem dump, by its name."
}
```

<sub>Module `probe` line 616</sub>
