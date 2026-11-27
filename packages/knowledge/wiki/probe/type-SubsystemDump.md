<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-SubsystemDump"></a>

### `SubsystemDump` (struct)

A dump a subsystem registered (`physics`, `render`, `audio`, ...), as its `dump_<name>` command writes it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | The dump's name, as DumpDesc.what named it. |
| `data` | `unknown` | `null` | — | — | — | What the subsystem reported; its shape is the subsystem's. |

```ogt
struct SubsystemDump {
  doc "A dump a subsystem registered (`physics`, `render`, `audio`, ...), as its `dump_<name>` command writes it."
  field name String "The dump's name, as DumpDesc.what named it."
  field data Json "What the subsystem reported; its shape is the subsystem's."
}
```

<sub>Module `probe` line 609</sub>
