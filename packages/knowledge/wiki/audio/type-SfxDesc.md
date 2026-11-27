<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

<a id="type-SfxDesc"></a>

### `SfxDesc` (desc struct)

Procedural sound effect synthesis specification: name a built-in `preset` (coin, pickup, powerup, jump, land, footstep, hit, hurt, explosion, laser, whoosh, door, death, click, select, back, error, levelup) or supply custom `layers` (or both; `layers` takes precedence when non-empty). Deterministic on every host.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `preset` | `string` | `""` | — | — | — | Built-in preset name; ignored when `layers` is non-empty. |
| `layers` | `readonly SfxLayerDesc[]` | `[]` | — | — | — | Custom oscillator layers to mix. |
| `seed` | `number` | `1` | — | — | — | Deterministic RNG seed for noise and variation. |
| `pitch` | `number` | `1` | 0.1 … 10 | — | — | Global pitch multiplier. |
| `duration` | `number` | `1` | 0.05 … 20 | — | — | Global envelope duration multiplier. |
| `volume` | `number` | `1` | 0 … 4 | — | — | Global output volume multiplier. |
| `vary` | `number` | `0` | 0 … 1 | `ratio` | — | Randomised per-seed pitch/duration jitter share. |

```ogt
struct SfxDesc {
  doc "Procedural sound effect synthesis specification: name a built-in `preset` (coin, pickup, powerup, jump, land, footstep, hit, hurt, explosion, laser, whoosh, door, death, click, select, back, error, levelup) or supply custom `layers` (or both; `layers` takes precedence when non-empty). Deterministic on every host."
  desc true
  field preset String "Built-in preset name; ignored when `layers` is non-empty."
  field layers List.SfxLayerDesc "Custom oscillator layers to mix."
  field seed u32 "Deterministic RNG seed for noise and variation." {
    default 1
  }
  field pitch f32 "Global pitch multiplier." {
    default 1
    min 0.1
    max 10
  }
  field duration f32 "Global envelope duration multiplier." {
    default 1
    min 0.05
    max 20
  }
  field volume f32 "Global output volume multiplier." {
    default 1
    min 0
    max 4
  }
  field vary f32 "Randomised per-seed pitch/duration jitter share." {
    min 0
    max 1
    unit ratio
  }
}
```

<sub>Module `audio` line 875</sub>
