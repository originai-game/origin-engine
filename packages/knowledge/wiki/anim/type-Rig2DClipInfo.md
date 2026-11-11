<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-Rig2DClipInfo"></a>

### `Rig2DClipInfo` (struct)

One clip of a loaded 2D rig.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `clip` | `number` | `0` | — | — | — | Its index, what Skeleton2D.clip takes. |
| `name` | `string` | `""` | — | — | — | Its name in the rig. |
| `duration` | `number` | `0` | — | `s` | — | Its length. |
| `loop` | `boolean` | `false` | — | — | — | Whether the rig's author meant it to loop (its `loop` line): copy it into Skeleton2D.loop. |
| `events` | `readonly string[]` | `[]` | — | — | — | Its events' names, in time order. |

```ogt
struct Rig2DClipInfo {
  doc "One clip of a loaded 2D rig."
  field clip u32 "Its index, what Skeleton2D.clip takes."
  field name String "Its name in the rig."
  field duration f32 "Its length." {
    unit s
  }
  field loop bool "Whether the rig's author meant it to loop (its `loop` line): copy it into Skeleton2D.loop."
  field events List.String "Its events' names, in time order."
}
```

<sub>Module `anim` line 858</sub>
