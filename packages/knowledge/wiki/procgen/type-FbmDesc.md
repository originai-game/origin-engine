<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-FbmDesc"></a>

### `FbmDesc` (desc struct)

Fractal noise (FBM): `octaves` layers, each `lacunarity` times the frequency and `gain` times the amplitude of the last, divided by the amplitudes' sum, so the value stays about -1 to 1. One octave is plain noise.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `seed` | `number` | `0` | — | — | — | The stream's seed: `world.resource(og.Rng)`'s value `.fork("terrain").seed`. Each octave reads its own table derived from it. |
| `kind` | `NoiseKind` | `"perlin"` | — | — | — | Perlin or simplex. |
| `octaves` | `number` | `1` | 1 … 16 | — | — | Layers summed. |
| `frequency` | `number` | `1` | ≥ 0 | — | — | The first octave's frequency: features about 1 / frequency apart. |
| `lacunarity` | `number` | `2` | ≥ 1 | — | — | Each octave's frequency over the last's. |
| `gain` | `number` | `0.5` | 0 … 1 | — | — | Each octave's amplitude over the last's (persistence). |
| `threeD` | `boolean` | `false` | — | — | — | Sample 3D noise at z (a slice that changes smoothly with z: animate it by moving z); 2D ignores z. |

```ogt
struct FbmDesc {
  doc "Fractal noise (FBM): `octaves` layers, each `lacunarity` times the frequency and `gain` times the amplitude of the last, divided by the amplitudes' sum, so the value stays about -1 to 1. One octave is plain noise."
  desc true
  field seed u32 "The stream's seed: `world.resource(og.Rng)`'s value `.fork(\"terrain\").seed`. Each octave reads its own table derived from it."
  field kind NoiseKind "Perlin or simplex."
  field octaves u32 "Layers summed." {
    default 1
    min 1
    max 16
  }
  field frequency f32 "The first octave's frequency: features about 1 / frequency apart." {
    default 1
    min 0
  }
  field lacunarity f32 "Each octave's frequency over the last's." {
    default 2
    min 1
  }
  field gain f32 "Each octave's amplitude over the last's (persistence)." {
    default 0.5
    min 0
    max 1
  }
  field threeD bool "Sample 3D noise at z (a slice that changes smoothly with z: animate it by moving z); 2D ignores z."
}
```

<sub>`api.ogt` line 1789</sub>
