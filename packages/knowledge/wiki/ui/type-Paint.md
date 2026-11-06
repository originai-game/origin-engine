<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Paint"></a>

### `Paint` (desc struct)

A colour: a theme token, or an sRGB colour as authored (components 0 to 1), with an alpha. Neither is white.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `token` | `ThemeColor \| null` | `null` | — | — | — | A semantic colour of the theme. |
| `role` | `string \| null` | `null` | — | — | — | A colour role the style declares of its own (its `roles` block, docs/ui.md §12.2: a game's corruption or faction colour) by name, when no token is given; a name the style lacks draws magenta. |
| `color` | `Color \| null` | `null` | — | — | — | An sRGB colour as authored, when no token or role is given. |
| `alpha` | `number` | `1` | 0 … 1 | — | — | The opacity. |

```ogt
struct Paint {
  doc "A colour: a theme token, or an sRGB colour as authored (components 0 to 1), with an alpha. Neither is white."
  desc true
  field token Option.ThemeColor "A semantic colour of the theme."
  field role Option.String "A colour role the style declares of its own (its `roles` block, docs/ui.md §12.2: a game's corruption or faction colour) by name, when no token is given; a name the style lacks draws magenta."
  field color Option.Color "An sRGB colour as authored, when no token or role is given."
  field alpha f32 "The opacity." {
    default 1
    min 0
    max 1
  }
}
```

<sub>`api.ogt` line 15819</sub>
