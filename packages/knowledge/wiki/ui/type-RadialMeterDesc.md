<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-RadialMeterDesc"></a>

### `RadialMeterDesc` (desc struct)

A radial stamina wheel.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `value` | `number` | `0` | — | — | — | The value, in wheels: 1 is one full wheel. |
| `wheels` | `number` | `0` | — | — | — | Wheels (an extended stamina's second and third ring); 0 is 1. |
| `exhausted` | `boolean` | `false` | — | — | — | Ran out: red, refilling slowly, until full (Breath of the Wild). |
| `size` | `number` | `0` | ≥ 0 | `cssPx` | — | Its diameter; 0 is 56. |
| `hideWhenFull` | `boolean` | `true` | — | — | — | Fades out a second after it is full (the default). |
| `color` | `Paint \| null` | `null` | — | — | — | Its fill; null is success. |
| `anchor` | `Anchor \| null` | `null` | — | — | — | Opens its own anchored group there; null builds it in place. |
| `name` | `string` | `"Stamina"` | — | — | — | What a screen reader calls it. |

```ogt
struct RadialMeterDesc {
  doc "A radial stamina wheel."
  desc true
  field value f32 "The value, in wheels: 1 is one full wheel."
  field wheels u32 "Wheels (an extended stamina's second and third ring); 0 is 1."
  field exhausted bool "Ran out: red, refilling slowly, until full (Breath of the Wild)."
  field size f32 "Its diameter; 0 is 56." {
    min 0
    unit cssPx
  }
  field hideWhenFull bool "Fades out a second after it is full (the default)." {
    default true
  }
  field color Option.Paint "Its fill; null is success."
  field anchor Option.Anchor "Opens its own anchored group there; null builds it in place."
  field name String "What a screen reader calls it." {
    default "Stamina"
  }
}
```

<sub>Module `ui` line 5987</sub>
