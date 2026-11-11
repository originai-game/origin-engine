<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="type-LoadBudget"></a>

### `LoadBudget` (desc struct)

The limits an uncooked load (og.load.imageData, og.load.gltfData, og.load.environmentData) is held to: content a player or a generator made is untrusted, so each limit is checked from the file's headers before anything is decoded, and a load over one is refused naming the limit, the value and the fix.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `bytes` | `number` | `33554432` | ≥ 0 | `bytes` | — | The most bytes the source may have. |
| `triangles` | `number` | `500000` | — | — | — | The most triangles a glTF's meshes may have, every primitive of every mesh. |
| `textureSize` | `number` | `4096` | — | — | — | The widest or tallest an image may be, in texels (the device's own limit lowers it further). |
| `frameMs` | `number` | `2` | ≥ 0 | `ms` | — | How long a frame may spend on the engine's thread putting finished loads in place, in milliseconds; at least one load is put in place a frame, and decoding never runs on the engine's thread on the web. |

```ogt
struct LoadBudget {
  doc "The limits an uncooked load (og.load.imageData, og.load.gltfData, og.load.environmentData) is held to: content a player or a generator made is untrusted, so each limit is checked from the file's headers before anything is decoded, and a load over one is refused naming the limit, the value and the fix."
  desc true
  field bytes f64 "The most bytes the source may have." {
    default 33554432
    min 0
    unit bytes
  }
  field triangles u32 "The most triangles a glTF's meshes may have, every primitive of every mesh." {
    default 500000
  }
  field textureSize u32 "The widest or tallest an image may be, in texels (the device's own limit lowers it further)." {
    default 4096
  }
  field frameMs f32 "How long a frame may spend on the engine's thread putting finished loads in place, in milliseconds; at least one load is put in place a frame, and decoding never runs on the engine's thread on the web." {
    default 2
    min 0
    unit ms
  }
}
```

<sub>Module `load` line 472</sub>
