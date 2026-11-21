<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="type-FlowStartDesc"></a>

### `FlowStartDesc` (desc struct)

A flow to start.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `dialogue` | `string` | `""` | — | — | — | The dialogue (StoryAsset.name), loaded with og.story.load and og.flow.load. |
| `node` | `string` | `""` | — | — | — | The node to start at (empty: the dialogue's start). |
| `decks` | `number` | `0` | ≤ 3 | — | — | Decks to play on, 1 to 3 (0: 3). One plays; the others warm what the flow may cut to. 1 warms nothing (one decoder: WeChat on iOS). |
| `paused` | `boolean` | `false` | — | — | — | Start holding the first segment's first frame, warmed, until og.flow.play (a title screen before the first gesture). |

```ogt
struct FlowStartDesc {
  doc "A flow to start."
  desc true
  field dialogue String "The dialogue (StoryAsset.name), loaded with og.story.load and og.flow.load."
  field node String "The node to start at (empty: the dialogue's start)."
  field decks u32 "Decks to play on, 1 to 3 (0: 3). One plays; the others warm what the flow may cut to. 1 warms nothing (one decoder: WeChat on iOS)." {
    max 3
  }
  field paused bool "Start holding the first segment's first frame, warmed, until og.flow.play (a title screen before the first gesture)."
}
```

<sub>Module `flow` line 7</sub>
