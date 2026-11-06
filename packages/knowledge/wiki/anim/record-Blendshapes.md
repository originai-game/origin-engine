<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="record-Blendshapes"></a>

### `og.Blendshapes` (component)

Morph-target weights of the skeleton entity's mesh, in the order of the glTF mesh's targets, up to 64 targets (w0...w7, then `influences`). With `source` clips the playing clips' weight channels write them; with `component` game code does. Under TAA a morphing mesh has motion vectors from its previous weights, so a talking face does not ghost.

Also known as: morph targets, shape keys, facial expression, blend shapes, three.js morphTargetInfluences

Token `og.Blendshapes`, registered under the schema name `og.Blendshapes`; its value type is `Blendshapes`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `source` | `BlendshapeSource` | `"component"` | — | — | — | Who decides the weights: component (game code writes w0...w7) or clips (the animation writes them). |
| `mesh` | `number` | `0` | ≥ 0 | — | — | The glTF mesh index whose targets the weights drive (0 is the first mesh). |
| `w0` | `number` | `0` | -1 … 2 | — | tunable | Weight of target 0. |
| `w1` | `number` | `0` | -1 … 2 | — | tunable | Weight of target 1. |
| `w2` | `number` | `0` | -1 … 2 | — | tunable | Weight of target 2. |
| `w3` | `number` | `0` | -1 … 2 | — | tunable | Weight of target 3. |
| `w4` | `number` | `0` | -1 … 2 | — | tunable | Weight of target 4. |
| `w5` | `number` | `0` | -1 … 2 | — | tunable | Weight of target 5. |
| `w6` | `number` | `0` | -1 … 2 | — | tunable | Weight of target 6. |
| `w7` | `number` | `0` | -1 … 2 | — | tunable | Weight of target 7. |
| `influences` | `readonly number[]` | `[]` | — | — | a list of at most 56 `f32` (`List.f32.56`): reads a frozen copy, writes a whole array | Weights of targets 8 to 63, as three.js morphTargetInfluences past the first eight (w0...w7): influences[k] drives target 8 + k; past the list's length 0. A face rig's 52 ARKit shapes fit. Clips write it with `source` clips, as they write w0...w7. |

```ogt
component Blendshapes {
  doc "Morph-target weights of the skeleton entity's mesh, in the order of the glTF mesh's targets, up to 64 targets (w0...w7, then `influences`). With `source` clips the playing clips' weight channels write them; with `component` game code does. Under TAA a morphing mesh has motion vectors from its previous weights, so a talking face does not ghost."
  aliases "morph targets, shape keys, facial expression, blend shapes, three.js morphTargetInfluences"
  field source BlendshapeSource "Who decides the weights: component (game code writes w0...w7) or clips (the animation writes them)."
  field mesh u32 "The glTF mesh index whose targets the weights drive (0 is the first mesh)." {
    min 0
  }
  field w0 f32 "Weight of target 0." {
    min -1
    max 2
    tunable true
  }
  field w1 f32 "Weight of target 1." {
    min -1
    max 2
    tunable true
  }
  field w2 f32 "Weight of target 2." {
    min -1
    max 2
    tunable true
  }
  field w3 f32 "Weight of target 3." {
    min -1
    max 2
    tunable true
  }
  field w4 f32 "Weight of target 4." {
    min -1
    max 2
    tunable true
  }
  field w5 f32 "Weight of target 5." {
    min -1
    max 2
    tunable true
  }
  field w6 f32 "Weight of target 6." {
    min -1
    max 2
    tunable true
  }
  field w7 f32 "Weight of target 7." {
    min -1
    max 2
    tunable true
  }
  field influences List.f32.56 "Weights of targets 8 to 63, as three.js morphTargetInfluences past the first eight (w0...w7): influences[k] drives target 8 + k; past the list's length 0. A face rig's 52 ARKit shapes fit. Clips write it with `source` clips, as they write w0...w7."
}
```

<sub>`api.ogt` line 11681</sub>
