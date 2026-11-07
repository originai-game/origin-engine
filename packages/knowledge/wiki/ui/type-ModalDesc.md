<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ModalDesc"></a>

### `ModalDesc` (desc struct)

A dialog: a scrim over everything and a centred panel. Focus moves in and stays while it is the topmost; Back and the backdrop dismiss it unless locked.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `title` | `string` | `""` | — | — | — | A title over the content, set in the style's heading type role (its step and face); empty is none. |
| `width` | `Size \| null` | `null` | — | — | — | The panel's width; null is 420 CSS px, at most the surface less its gutters. |
| `locked` | `boolean` | `false` | — | — | — | Back and the backdrop do not dismiss it (a choice must be made). |
| `ignoreBackdrop` | `boolean` | `false` | — | — | — | A press on the backdrop does not dismiss it. |
| `transition` | `TransitionKind \| null` | `null` | — | — | — | How it enters and leaves; null is scale. |
| `backdrop` | `Paint \| null` | `null` | — | — | — | The backdrop's fill; null is scrim. |
| `bare` | `boolean` | `false` | — | — | — | No panel: the content sits centred on the backdrop (full-screen menus, loading). |

```ogt
struct ModalDesc {
  doc "A dialog: a scrim over everything and a centred panel. Focus moves in and stays while it is the topmost; Back and the backdrop dismiss it unless locked."
  desc true
  field title String "A title over the content, set in the style's heading type role (its step and face); empty is none."
  field width Option.Size "The panel's width; null is 420 CSS px, at most the surface less its gutters."
  field locked bool "Back and the backdrop do not dismiss it (a choice must be made)."
  field ignoreBackdrop bool "A press on the backdrop does not dismiss it."
  field transition Option.TransitionKind "How it enters and leaves; null is scale."
  field backdrop Option.Paint "The backdrop's fill; null is scrim."
  field bare bool "No panel: the content sits centred on the backdrop (full-screen menus, loading)."
}
```

<sub>Module `ui` line 931</sub>
