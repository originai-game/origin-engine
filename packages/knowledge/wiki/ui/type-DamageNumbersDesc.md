<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-DamageNumbersDesc"></a>

### `DamageNumbersDesc` (desc struct)

Damage numbers.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `hits` | `readonly DamageHitDesc[]` | `[]` | — | — | — | The hits that landed this tick (give each hit once). |
| `compact` | `boolean` | `true` | — | — | — | 12.3K over 12,300 (the default). |
| `size` | `number` | `0` | ≥ 0 | `cssPx` | — | A normal hit's text size; 0 is 22. |
| `reference` | `number` | `10` | ≥ 0 | `m` | — | The distance at which numbers are their size; 0 keeps the screen size. |

```ogt
struct DamageNumbersDesc {
  doc "Damage numbers."
  desc true
  field hits List.DamageHitDesc "The hits that landed this tick (give each hit once)."
  field compact bool "12.3K over 12,300 (the default)." {
    default true
  }
  field size f32 "A normal hit's text size; 0 is 22." {
    min 0
    unit cssPx
  }
  field reference f32 "The distance at which numbers are their size; 0 keeps the screen size." {
    default 10
    unit m
    min 0
  }
}
```

<sub>Module `ui` line 6535</sub>
