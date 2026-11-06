<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-GraphDesc"></a>

### `GraphDesc` (desc struct)

A blend-tree graph to register.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `text` | `string` | `""` | — | — | — | The graph's `.ogt` text (`type animgraph`, `version 1`). |
| `clips` | `readonly ClipInfo[]` | `[]` | — | — | — | The clips a `clip <motion> "<name>"` line may name, by ClipInfo.name: usually og.load.animation's `clips`. |
| `sprite` | `boolean` | `false` | — | — | — | A graph for og.Flipbook and og.Skeleton2D: its `clip` lines name clips of the sheet or rig of the entity it runs on, looked up by name as it plays (a clip the sheet lacks plays nothing), and `clips` is not read. |

```ogt
struct GraphDesc {
  doc "A blend-tree graph to register."
  desc true
  field text String "The graph's `.ogt` text (`type animgraph`, `version 1`)."
  field clips List.ClipInfo "The clips a `clip <motion> \"<name>\"` line may name, by ClipInfo.name: usually og.load.animation's `clips`."
  field sprite bool "A graph for og.Flipbook and og.Skeleton2D: its `clip` lines name clips of the sheet or rig of the entity it runs on, looked up by name as it plays (a clip the sheet lacks plays nothing), and `clips` is not read."
}
```

<sub>Module `anim` line 569</sub>
