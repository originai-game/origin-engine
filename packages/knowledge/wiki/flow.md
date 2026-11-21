<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `flow` module

Interactive film (FMV): a Yarn dialogue paced by its footage. A node with an `og-flow: <timeline>` header plays that segment (a timeline with a video track) on a deck; its choices open at the segment's `flow.open` marker, lock at `flow.lock` and are taken at its cut (`flow.cut`, else its end), the default (`#default`) when none was chosen.

The story stays og.story's, the footage og.video's, the timing og.timeline's: og.flow moves the conversation between segments and warms those a choice may lead to on its other decks. All of it is world state.

See `flow`.

- **TypeScript**: functions are the namespace `og.flow`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module flow {
  doc "Interactive film (FMV): a Yarn dialogue paced by its footage. A node with an `og-flow: <timeline>` header plays that segment (a timeline with a video track) on a deck; its choices open at the segment's `flow.open` marker, lock at `flow.lock` and are taken at its cut (`flow.cut`, else its end), the default (`#default`) when none was chosen."
  doc "The story stays og.story's, the footage og.video's, the timing og.timeline's: og.flow moves the conversation between segments and warms those a choice may lead to on its other decks. All of it is world state."
  # declarations follow, each on its own page
```

<sub>Module `flow` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`FlowStartDesc`](flow/type-FlowStartDesc.md) | desc struct |  | A flow to start. |
| [`FlowInfo`](flow/type-FlowInfo.md) | struct |  | What og.flow.load prepared. |
| [`og.Flow`](flow/record-Flow.md) | component |  | A flow og.flow.start made: the conversation it paces and its decks. |
| [`og.FlowDeck`](flow/record-FlowDeck.md) | component |  | An entity og.flow plays segments on, with an og.Timeline, an og.VideoPlayer (its `wait` set) and an og.AudioSource. |
| [`WindowView`](flow/type-WindowView.md) | struct |  | A segment's choice window. |
| [`FlowHotspot`](flow/type-FlowHotspot.md) | struct |  | A place on the picture that takes a choice (`#hotspot:<id>` on the choice, a `track hotspot` key in its segment), while the window is open. |
| [`QteView`](flow/type-QteView.md) | struct |  | A choice taken by a quick-time event (`#qte:press:<action>`, `#qte:hold:<action>:<ms>`, `#qte:mash:<action>:<presses>`): the og.input action's presses or hold while the window is open. |
| [`FlowView`](flow/type-FlowView.md) | struct |  | What a flow shows now. |
| [`FlowChartNode`](flow/type-FlowChartNode.md) | struct |  | A flow node on og.flow.chart's layout. |
| [`FlowChartLink`](flow/type-FlowChartLink.md) | struct |  | Where a flow node may lead, through logic nodes. |
| [`FlowChart`](flow/type-FlowChart.md) | struct |  | A dialogue's flow nodes laid out as a flowchart, columns by depth from the start: what a flowchart screen draws (the built-in NodeGraph), the game marking what the player has seen. |
| [`og.FlowSegment`](flow/record-FlowSegment.md) | event |  | A flow began a node's segment. |
| [`og.FlowChoice`](flow/record-FlowChoice.md) | event |  | A flow took a choice at its cut (before og.story ran it). |
| [`og.FlowChapter`](flow/record-FlowChapter.md) | event |  | A flow entered a chapter's node (`og-flow-chapter`): a place to save. |
| [`og.FlowEnded`](flow/record-FlowEnded.md) | event |  | A flow's last segment reached its cut with the conversation ended. |
| [`og.flow.load`](flow/fn-load.md) | function | `(world: World, dialogue: string): Result<FlowInfo>` | Prepare a loaded dialogue's flow: every segment's timeline (and through it every video, which fetches nothing until it plays) and og.flow's records. |
| [`og.flow.start`](flow/fn-start.md) | function | `(world: World, desc?: FlowStartDesc): Result<Entity>` | Start a flow: its decks, and its conversation held at the first flow node, whose segment plays from the next tick. |
| [`og.flow.view`](flow/fn-view.md) | function | `(world: World, flow: Entity): FlowView \| null` | What a flow shows now: its picture, segment time, choices and window. |
| [`og.flow.choose`](flow/fn-choose.md) | function | `(world: World, flow: Entity, index: number): Result<void>` | Take a choice while the window is open: recorded now (FlowView.window.pending; a later call replaces it) and taken at the cut, so the footage plays on to it. |
| [`og.flow.play`](flow/fn-play.md) | function | `(world: World, flow: Entity): Result<void>` | Play a paused flow from where it is. |
| [`og.flow.pause`](flow/fn-pause.md) | function | `(world: World, flow: Entity): Result<void>` | Hold a flow on its frame (a pause menu); its window's time holds too. |
| [`og.flow.skip`](flow/fn-skip.md) | function | `(world: World, flow: Entity): Result<void>` | Skip ahead in the segment playing: to its window's open (flow.open) when that is ahead, else to its cut, which takes the pending choice, else the default. |
| [`og.flow.chart`](flow/fn-chart.md) | function | `(world: World, dialogue: string): Result<FlowChart>` | A prepared dialogue's flow nodes and where each may lead, laid out for a flowchart. |
| [`og.flow.stop`](flow/fn-stop.md) | function | `(world: World, flow: Entity): Result<void>` | End a flow: its conversation stops and its decks and entity are despawned this tick. |
