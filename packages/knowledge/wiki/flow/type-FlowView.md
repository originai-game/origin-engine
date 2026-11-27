<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="type-FlowView"></a>

### `FlowView` (struct)

What a flow shows now.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `flow` | `Entity` | `0` | — | — | — | The flow. |
| `node` | `string` | `""` | — | — | — | The flow node playing. |
| `segment` | `string` | `""` | — | — | — | Its segment's asset id. |
| `texture` | `Texture` | `0` | — | — | — | The current deck's picture (its og.VideoPlayer.texture): put it on a full-screen sprite or image. 0 while its footage plays under the scene (a video track's `layer under`), which og.Camera.backgroundTexture shows. |
| `time` | `number` | `0` | — | `s` | — | Seconds into the segment. |
| `cut` | `number` | `0` | — | `s` | — | When it cuts to the next: its flow.cut marker, else its duration. |
| `speed` | `number` | `0` | — | — | — | Current playback speed (og.flow.setSpeed). |
| `videoWidth` | `number` | `0` | — | — | — | Current segment's video frame width in texels (0 for a live-only segment), for aspect-exact hotspot mapping. |
| `videoHeight` | `number` | `0` | — | — | — | Current segment's video frame height in texels (0 for a live-only segment). |
| `choices` | `readonly ChoiceView[]` | `[]` | — | — | — | The node's choices while its window is open or locked (og.story's), without its `#outcome` choices, which the game takes. |
| `window` | `WindowView \| null` | `null` | — | — | — | The window, while it is open or locked. |
| `hotspots` | `readonly FlowHotspot[]` | `[]` | — | — | — | The hotspots active now: enabled choices' hotspots while the window is open, plus active `inspect` hotspots. |
| `qtes` | `readonly QteView[]` | `[]` | — | — | — | The quick-time events of its enabled choices while the window is open or locked. |
| `previous` | `Texture` | `0` | — | — | — | The picture dissolving out (`og-flow-dissolve`); 0: none. Draw it under `texture`, which shows at `mix`. |
| `mix` | `number` | `0` | — | `ratio` | — | How far the dissolve is: `texture`'s opacity over `previous`, 1 without one. |
| `paused` | `boolean` | `false` | — | — | — | Held by og.flow.pause. |
| `held` | `boolean` | `false` | — | — | — | Stopped at its flow.hold, waiting for og.flow.advance (og.Flow.held). |
| `looping` | `boolean` | `false` | — | — | — | Looping inside the segment's `flow.loopStart`..`flow.loopEnd` idle loop. |
| `buffering` | `boolean` | `false` | — | — | — | Presentation-only: whether the current deck is waiting on stream download or decode. |
| `bufferProgress` | `number` | `0` | — | `ratio` | — | Presentation-only: 0 to 1 download progress for the current stream's next needed chunk. |
| `chapter` | `string` | `""` | — | — | — | The last chapter entered. |
| `ending` | `string` | `""` | — | — | — | The ending reached, once ended. |
| `ended` | `boolean` | `false` | — | — | — | It played to its end. |

```ogt
struct FlowView {
  doc "What a flow shows now."
  field flow Entity "The flow."
  field node String "The flow node playing."
  field segment String "Its segment's asset id."
  field texture Texture "The current deck's picture (its og.VideoPlayer.texture): put it on a full-screen sprite or image. 0 while its footage plays under the scene (a video track's `layer under`), which og.Camera.backgroundTexture shows."
  field time f32 "Seconds into the segment." {
    unit s
  }
  field cut f32 "When it cuts to the next: its flow.cut marker, else its duration." {
    unit s
  }
  field speed f32 "Current playback speed (og.flow.setSpeed)."
  field videoWidth u32 "Current segment's video frame width in texels (0 for a live-only segment), for aspect-exact hotspot mapping."
  field videoHeight u32 "Current segment's video frame height in texels (0 for a live-only segment)."
  field choices List.ChoiceView "The node's choices while its window is open or locked (og.story's), without its `#outcome` choices, which the game takes."
  field window Option.WindowView "The window, while it is open or locked."
  field hotspots List.FlowHotspot "The hotspots active now: enabled choices' hotspots while the window is open, plus active `inspect` hotspots."
  field qtes List.QteView "The quick-time events of its enabled choices while the window is open or locked."
  field previous Texture "The picture dissolving out (`og-flow-dissolve`); 0: none. Draw it under `texture`, which shows at `mix`."
  field mix f32 "How far the dissolve is: `texture`'s opacity over `previous`, 1 without one." {
    unit ratio
  }
  field paused bool "Held by og.flow.pause."
  field held bool "Stopped at its flow.hold, waiting for og.flow.advance (og.Flow.held)."
  field looping bool "Looping inside the segment's `flow.loopStart`..`flow.loopEnd` idle loop."
  field buffering bool "Presentation-only: whether the current deck is waiting on stream download or decode."
  field bufferProgress f32 "Presentation-only: 0 to 1 download progress for the current stream's next needed chunk." {
    unit ratio
  }
  field chapter String "The last chapter entered."
  field ending String "The ending reached, once ended."
  field ended bool "It played to its end."
}
```

<sub>Module `flow` line 145</sub>
