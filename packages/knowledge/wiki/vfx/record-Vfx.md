<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`vfx`](../vfx.md) · [index](../index.md)

<a id="record-Vfx"></a>

### `og.Vfx` (component)

Plays a composed effect at the entity's MatrixWorld from startTick (og.vfx.spawn makes one in a call).

Token `og.Vfx`, registered under the schema name `og.Vfx`; its value type is `Vfx`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `asset` | `VfxAsset` | `0` | — | — | — | The effect (og.load.vfx); 0 plays nothing. |
| `seed` | `number` | `0` | — | — | — | Its random seed: what `random` values, emitters and sound pitches pick (instances with different seeds differ). |
| `startTick` | `number` | `4294967295` | — | — | — | The tick it started: write the tick (`world.tick`) to play it, or to play it again. Its default, never written, waits: an effect spawned ahead of its use plays nothing until a tick is written. |
| `scale` | `number` | `1` | ≥ 0 | — | — | Its size: offsets, meshes, trails, beams, lights' reach, decals and flashes scale with it; its emitters are placed and shaped by the scaled transform. |
| `tint` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | Multiplies the colour of everything it draws (linear; above 1 brightens). |
| `despawn` | `boolean` | `true` | — | — | — | Despawn the entity on the tick after a non-looping effect ends. |
| `ran` | `number` | `4294967295` | — | — | read only | Written by og.vfx: the last tick it ran this effect through (its events, shakes and sounds up to there are sent). Writing startTick again plays it from the start. |

```ogt
component Vfx {
  doc "Plays a composed effect at the entity's MatrixWorld from startTick (og.vfx.spawn makes one in a call)."
  field asset VfxAsset "The effect (og.load.vfx); 0 plays nothing."
  field seed u32 "Its random seed: what `random` values, emitters and sound pitches pick (instances with different seeds differ)."
  field startTick u32 "The tick it started: write the tick (`world.tick`) to play it, or to play it again. Its default, never written, waits: an effect spawned ahead of its use plays nothing until a tick is written." {
    default 4294967295
  }
  field scale f32 "Its size: offsets, meshes, trails, beams, lights' reach, decals and flashes scale with it; its emitters are placed and shaped by the scaled transform." {
    default 1
    min 0
  }
  field tint Color "Multiplies the colour of everything it draws (linear; above 1 brightens)." {
    default 1 1 1
  }
  field despawn bool "Despawn the entity on the tick after a non-looping effect ends." {
    default true
  }
  field ran u32 "Written by og.vfx: the last tick it ran this effect through (its events, shakes and sounds up to there are sent). Writing startTick again plays it from the start." {
    default 4294967295
    readonly true
  }
}
```

<sub>`api.ogt` line 27415</sub>
