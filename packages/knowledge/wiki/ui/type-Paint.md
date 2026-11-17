<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Paint"></a>

### `Paint` (desc struct)

A colour: a theme token, a role of the style, or a Color (linear RGB, as every Color of the API), with an opacity. None of them is white.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `token` | `ThemeColor \| null` | `null` | — | — | — | A semantic colour of the theme. |
| `role` | `string \| null` | `null` | — | — | — | A colour role the style declares of its own (its `roles` block, docs/ui.md §3.1: a game's corruption or faction colour) by name, when no token is given; a name the style lacks draws magenta. |
| `color` | `Color \| null` | `null` | — | — | — | A linear RGB colour, when no token or role is given: `color.fromHex(0xff8800)` for a hex or CSS colour. The UI draws 8-bit sRGB, so each component is clamped to 0 to 1 (no HDR). |
| `opacity` | `number` | `1` | 0 … 1 | `ratio` | — | Its opacity, multiplied with the token's or the role's own. |

```ogt
struct Paint {
  doc "A colour: a theme token, a role of the style, or a Color (linear RGB, as every Color of the API), with an opacity. None of them is white."
  desc true
  field token Option.ThemeColor "A semantic colour of the theme."
  field role Option.String "A colour role the style declares of its own (its `roles` block, docs/ui.md §3.1: a game's corruption or faction colour) by name, when no token is given; a name the style lacks draws magenta."
  field color Option.Color "A linear RGB colour, when no token or role is given: `color.fromHex(0xff8800)` for a hex or CSS colour. The UI draws 8-bit sRGB, so each component is clamped to 0 to 1 (no HDR)."
  field opacity f32 "Its opacity, multiplied with the token's or the role's own." {
    default 1
    min 0
    max 1
    unit ratio
  }
}
```

<sub>Module `ui` line 118</sub>
