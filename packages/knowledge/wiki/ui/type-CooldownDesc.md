<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-CooldownDesc"></a>

### `CooldownDesc` (desc struct)

A cooldown ring's size and centre.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `size` | `number` | `0` | ≥ 0 | `cssPx` | — | Its diameter; 0 is 56 (never under 44). |
| `label` | `string` | `""` | — | — | — | Text in the middle (a key, seconds left); empty is none. |
| `icon` | `string` | `""` | — | — | — | An icon of the atlas in the middle, by name; empty is none. |

```ogt
struct CooldownDesc {
  doc "A cooldown ring's size and centre."
  desc true
  field size f32 "Its diameter; 0 is 56 (never under 44)." {
    unit cssPx
    min 0
  }
  field label String "Text in the middle (a key, seconds left); empty is none."
  field icon String "An icon of the atlas in the middle, by name; empty is none."
}
```

<sub>Module `ui` line 1047</sub>
