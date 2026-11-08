<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-IconMarkDesc"></a>

### `IconMarkDesc` (desc struct)

An icon's size, tone, plate and badge.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `size` | `number` | `0` | ≥ 0 | `cssPx` | — | Its size; 0 is 24. |
| `tone` | `TextTone` | `"normal"` | — | — | — | Its ink by meaning. |
| `plate` | `IconPlate` | `"none"` | — | — | — | What it sits on. |
| `badge` | `string` | `""` | — | — | — | A count or word over its top end corner (`3`, `NEW`); empty for none. Past 99 it says 99+. |
| `dot` | `boolean` | `false` | — | — | — | A small dot over its top end corner (something new), when there is no badge. |
| `label` | `string \| null` | `null` | — | — | — | What a screen reader calls it: null is the icon's name (an English word), empty is decoration. |

```ogt
struct IconMarkDesc {
  doc "An icon's size, tone, plate and badge."
  desc true
  field size f32 "Its size; 0 is 24." {
    unit cssPx
    min 0
  }
  field tone TextTone "Its ink by meaning."
  field plate IconPlate "What it sits on."
  field badge String "A count or word over its top end corner (`3`, `NEW`); empty for none. Past 99 it says 99+."
  field dot bool "A small dot over its top end corner (something new), when there is no badge."
  field label Option.String "What a screen reader calls it: null is the icon's name (an English word), empty is decoration."
}
```

<sub>Module `ui` line 3233</sub>
