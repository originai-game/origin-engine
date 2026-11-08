<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ActionHintsDesc"></a>

### `ActionHintsDesc` (desc struct)

The action bar (docs/ui.md §2.1): the actions the top layer accepts, each its glyph and its hint, at an edge.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `hints` | `readonly ActionHint[]` | `[]` | — | — | — | The hints, as og.ui.actionBar answers them. |
| `anchor` | `Anchor` | `"bottomRight"` | — | — | — | The edge it sits on. |
| `background` | `boolean` | `true` | — | — | — | A HUD backing behind it. |
| `label` | `string` | `""` | — | — | — | What a screen reader calls it; empty is Actions. |

```ogt
struct ActionHintsDesc {
  doc "The action bar (docs/ui.md §2.1): the actions the top layer accepts, each its glyph and its hint, at an edge."
  desc true
  field hints List.ActionHint "The hints, as og.ui.actionBar answers them."
  field anchor Anchor "The edge it sits on." {
    default bottomRight
  }
  field background bool "A HUD backing behind it." {
    default true
  }
  field label String "What a screen reader calls it; empty is Actions."
}
```

<sub>Module `ui` line 2289</sub>
