<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tween`](../tween.md) · [index](../index.md)

<a id="type-FlashDesc"></a>

### `FlashDesc` (desc struct)

A hit flash: the entity's og.Sprite (else og.Mesh, else og.Model) flashColor set and its flash faded from amount to 0.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The flash colour (white for a hit, red for damage taken). |
| `amount` | `number` | `1` | 0 … 1 | — | — | How far it starts towards the colour. |
| `duration` | `number` | `0.12` | ≥ 0 | `s` | — | Seconds to fade. |
| `ease` | `Ease` | `"quadIn"` | — | — | — | The fade's curve; quadIn holds the flash, then drops it. |

```ogt
struct FlashDesc {
  doc "A hit flash: the entity's og.Sprite (else og.Mesh, else og.Model) flashColor set and its flash faded from amount to 0."
  desc true
  field entity Entity "The entity."
  field color Color "The flash colour (white for a hit, red for damage taken)." {
    default 1 1 1
  }
  field amount f32 "How far it starts towards the colour." {
    default 1
    min 0
    max 1
  }
  field duration f32 "Seconds to fade." {
    default 0.12
    min 0
    unit s
  }
  field ease Ease "The fade's curve; quadIn holds the flash, then drops it." {
    default quadIn
  }
}
```

<sub>`api.ogt` line 12414</sub>
