<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-HudEditorDesc"></a>

### `HudEditorDesc` (desc struct)

The HUD edit mode's panel; empty labels take English ones (pass the locale's).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `title` | `string` | `""` | — | — | — | Its title; empty is Edit HUD. |
| `sizeLabel` | `string` | `""` | — | — | — | The size steps' label; empty is Size. |
| `opacityLabel` | `string` | `""` | — | — | — | The opacity steps' label; empty is Opacity. |
| `nextLabel` | `string` | `""` | — | — | — | Next's label; empty is Next. |
| `resetLabel` | `string` | `""` | — | — | — | Reset's label; empty is Reset. |
| `doneLabel` | `string` | `""` | — | — | — | Done's label; empty is Done. |

```ogt
struct HudEditorDesc {
  doc "The HUD edit mode's panel; empty labels take English ones (pass the locale's)."
  desc true
  field title String "Its title; empty is Edit HUD."
  field sizeLabel String "The size steps' label; empty is Size."
  field opacityLabel String "The opacity steps' label; empty is Opacity."
  field nextLabel String "Next's label; empty is Next."
  field resetLabel String "Reset's label; empty is Reset."
  field doneLabel String "Done's label; empty is Done."
}
```

<sub>`api.ogt` line 17462</sub>
