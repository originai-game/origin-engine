<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`art`](../art.md) · [index](../index.md)

<a id="record-ArtMotion"></a>

### `og.ArtMotion` (resource)

The style's motion: animation held on steps as hand animation is, how many frames a clip may hold, and how much squash and stretch its movement takes. Every field is no constraint at its zero value.

Token `og.ArtMotion`, registered under the schema name `og.ArtMotion`; its value type is `ArtMotion`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `stepFps` | `number` | `0` | 0 … 120 | `hz` | — | Clips and flipbooks advance this many times a second whatever the tick, holding each pose between (12 is on twos at 24 fps, 8 on threes); 0 is smooth, no constraint. |
| `maxFrames` | `number` | `0` | ≥ 0 | — | — | The most frames one clip or flipbook may hold; 0 is no limit. |
| `squash` | `number` | `0` | 0 … 1 | `ratio` | — | The default squash and stretch of the style's movement: og.tween.squash's impact where a call leaves it out, 0.5 the preset's own (1.25, 0.8), 1 as rubber hose (1.5, 0.6); 0 is no constraint (the preset's own). |

```ogt
resource ArtMotion {
  doc "The style's motion: animation held on steps as hand animation is, how many frames a clip may hold, and how much squash and stretch its movement takes. Every field is no constraint at its zero value."
  field stepFps f32 "Clips and flipbooks advance this many times a second whatever the tick, holding each pose between (12 is on twos at 24 fps, 8 on threes); 0 is smooth, no constraint." {
    min 0
    max 120
    unit hz
  }
  field maxFrames u32 "The most frames one clip or flipbook may hold; 0 is no limit." {
    min 0
  }
  field squash f32 "The default squash and stretch of the style's movement: og.tween.squash's impact where a call leaves it out, 0.5 the preset's own (1.25, 0.8), 1 as rubber hose (1.5, 0.6); 0 is no constraint (the preset's own)." {
    min 0
    max 1
    unit ratio
  }
}
```

<sub>`api.ogt` line 26718</sub>
