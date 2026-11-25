<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="type-FlowJumpDesc"></a>

### `FlowJumpDesc` (desc struct)

Jump a running flow to another node, optionally restoring story variables from a FlowCheckpoint (og.flow.jump).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `node` | `string` | `""` | — | — | — | The flow node title to jump to. |
| `vars` | `unknown` | `null` | — | — | — | Optional story variables to apply before entering `node` (null/omitted keeps current variables). |

```ogt
struct FlowJumpDesc {
  doc "Jump a running flow to another node, optionally restoring story variables from a FlowCheckpoint (og.flow.jump)."
  desc true
  field node String "The flow node title to jump to."
  field vars Json "Optional story variables to apply before entering `node` (null/omitted keeps current variables)."
}
```

<sub>Module `flow` line 229</sub>
