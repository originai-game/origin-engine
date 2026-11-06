<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="type-LoadSlotDesc"></a>

### `LoadSlotDesc` (desc struct)

Which slot to read, and how much of it.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `slot` | `string` | `""` | — | — | — | The slot's name, as saveSlot wrote it. |
| `metaOnly` | `boolean` | `false` | — | — | — | Read the manifest alone: the answer's text stays null (a title's Continue label, before the player asks for the world). |

```ogt
struct LoadSlotDesc {
  doc "Which slot to read, and how much of it."
  desc true
  field slot String "The slot's name, as saveSlot wrote it."
  field metaOnly bool "Read the manifest alone: the answer's text stays null (a title's Continue label, before the player asks for the world)."
}
```

<sub>Module `platform` line 80</sub>
