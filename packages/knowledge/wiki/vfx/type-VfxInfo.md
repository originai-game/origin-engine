<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`vfx`](../vfx.md) · [index](../index.md)

<a id="type-VfxInfo"></a>

### `VfxInfo` (struct)

A loaded composed effect.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `asset` | `VfxAsset` | `0` | — | — | — | Its handle. |
| `duration` | `number` | `0` | — | `s` | — | Its length. |
| `loop` | `boolean` | `false` | — | — | — | Whether it starts over at its end. |
| `events` | `readonly string[]` | `[]` | — | — | — | Its events' names in time order, `impact` included. |
| `impact` | `number` | `0` | — | `s` | — | Its impact's time, or -1 without one. |
| `elements` | `readonly string[]` | `[]` | — | — | — | Its elements' kinds, in file order (emitter, mesh, trail, beam, light, decal, flash, flipbook, shake, sound). |

```ogt
struct VfxInfo {
  doc "A loaded composed effect."
  field asset VfxAsset "Its handle."
  field duration f32 "Its length." {
    unit s
  }
  field loop bool "Whether it starts over at its end."
  field events List.String "Its events' names in time order, `impact` included."
  field impact f32 "Its impact's time, or -1 without one." {
    unit s
  }
  field elements List.String "Its elements' kinds, in file order (emitter, mesh, trail, beam, light, decal, flash, flipbook, shake, sound)."
}
```

<sub>Module `vfx` line 68</sub>
