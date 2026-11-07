<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-DialogDesc"></a>

### `DialogDesc` (desc struct)

A dialog: a title, a tone mark, a message, a body the game builds and a row of actions.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `title` | `string` | `""` | — | — | — | Its title. |
| `message` | `string` | `""` | — | — | — | The message, as rich text markup (og.ui.richText). |
| `tone` | `DialogTone` | `"neutral"` | — | — | — | Its tone. |
| `actions` | `readonly DialogAction[]` | `[]` | — | — | — | The buttons; ordered by weight on each breakpoint. |
| `cancelId` | `string` | `""` | — | — | — | The action that cancels (its sound is cancel's); empty is `cancel`. |
| `locked` | `boolean` | `false` | — | — | — | Back and the backdrop do not dismiss it. |
| `wide` | `boolean` | `false` | — | — | — | 640 CSS px wide instead of 460. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null is 460 (wide 640), at most the surface less its gutters. |
| `transition` | `TransitionKind \| null` | `null` | — | — | — | How it enters and leaves; null is the style's. |

```ogt
struct DialogDesc {
  doc "A dialog: a title, a tone mark, a message, a body the game builds and a row of actions."
  desc true
  field title String "Its title."
  field message String "The message, as rich text markup (og.ui.richText)."
  field tone DialogTone "Its tone."
  field actions List.DialogAction "The buttons; ordered by weight on each breakpoint."
  field cancelId String "The action that cancels (its sound is cancel's); empty is `cancel`."
  field locked bool "Back and the backdrop do not dismiss it."
  field wide bool "640 CSS px wide instead of 460."
  field width Option.Size "Its width; null is 460 (wide 640), at most the surface less its gutters."
  field transition Option.TransitionKind "How it enters and leaves; null is the style's."
}
```

<sub>Module `ui` line 2059</sub>
