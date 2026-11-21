<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="fn-load"></a>

### `og.flow.load`

Prepare a loaded dialogue's flow: every segment's timeline (and through it every video, which fetches nothing until it plays) and og.flow's records. Call it in setup, after og.story.load, in every process that restores a flow.

```ts
og.flow.load(world: World, dialogue: string): Result<FlowInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `dialogue` | `string` | no | The dialogue (StoryAsset.name). |

**Returns** `Result<FlowInfo>`: Refused for a dialogue not loaded, or a segment that is not a cooked timeline with a video track.

Binding symbol `og_flow_load` · publish builds: kept

```ogt
fn load {
  doc "Prepare a loaded dialogue's flow: every segment's timeline (and through it every video, which fetches nothing until it plays) and og.flow's records. Call it in setup, after og.story.load, in every process that restores a flow."
  param world World "The world."
  param dialogue String "The dialogue (StoryAsset.name)."
  returns Result.FlowInfo "Refused for a dialogue not loaded, or a segment that is not a cooked timeline with a video track."
}
```

<sub>Module `flow` line 200</sub>
