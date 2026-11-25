<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `flow` module

A game's flow as a Yarn dialogue of segments: film, cutscenes, levels and minigames. A node with an `og-flow: <timeline>` header plays that segment on a deck: footage, a live scene (a `scene` track) or both. Its choices open at `flow.open`, lock at `flow.lock` and are taken at the cut (`flow.cut`, else the end), else the `#default`. A playable segment stops at `flow.hold` until og.flow.advance takes a hidden `#outcome:<name>` choice (guide flow).

og.flow moves the conversation between segments and prepares those it may lead to on its other decks (og.Timeline.preroll). All of it is world state.

See `flow`.

- **TypeScript**: functions are the namespace `og.flow`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module flow {
  doc "A game's flow as a Yarn dialogue of segments: film, cutscenes, levels and minigames. A node with an `og-flow: <timeline>` header plays that segment on a deck: footage, a live scene (a `scene` track) or both. Its choices open at `flow.open`, lock at `flow.lock` and are taken at the cut (`flow.cut`, else the end), else the `#default`. A playable segment stops at `flow.hold` until og.flow.advance takes a hidden `#outcome:<name>` choice (guide flow)."
  doc "og.flow moves the conversation between segments and prepares those it may lead to on its other decks (og.Timeline.preroll). All of it is world state."
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
| [`FlowHotspot`](flow/type-FlowHotspot.md) | struct |  | A place on the picture: either a choice (`#hotspot:<id>` on the choice, a `track hotspot` key in its segment) while the window is open, or a non-cutting `inspect` hotspot (triggered with og.flow.insp… |
| [`QteView`](flow/type-QteView.md) | struct |  | A choice taken by a quick-time event (`#qte:press:<action>`, `#qte:hold:<action>:<ms>`, `#qte:mash:<action>:<presses>`, `#qte:timing:<action>:<start>-<end>`, `#qte:sequence:<a1>+<a2>+...`, `#qte:swip… |
| [`FlowView`](flow/type-FlowView.md) | struct |  | What a flow shows now. |
| [`FlowTranscriptLine`](flow/type-FlowTranscriptLine.md) | struct |  | One entry in a flow's transcript backlog (og.flow.history): a caption shown or a choice taken. |
| [`FlowGestureDesc`](flow/type-FlowGestureDesc.md) | desc struct |  | A deterministic touch or pointer gesture fed into active swipe/drag QTEs (og.flow.gesture). |
| [`FlowMusicDesc`](flow/type-FlowMusicDesc.md) | desc struct |  | Persistent cross-segment background music (og.flow.setMusic or `og-flow-music` header). |
| [`FlowCheckpoint`](flow/type-FlowCheckpoint.md) | struct |  | A lightweight branch-rewind checkpoint: the dialogue, node and story variables (og.flow.checkpoint / og.flow.jump). |
| [`FlowJumpDesc`](flow/type-FlowJumpDesc.md) | desc struct |  | Jump a running flow to another node, optionally restoring story variables from a FlowCheckpoint (og.flow.jump). |
| [`FlowChartNode`](flow/type-FlowChartNode.md) | struct |  | A flow node on og.flow.chart's layout. |
| [`FlowChartLink`](flow/type-FlowChartLink.md) | struct |  | Where a flow node may lead, through logic nodes. |
| [`FlowChart`](flow/type-FlowChart.md) | struct |  | A dialogue's flow nodes laid out as a flowchart, columns by depth from the start with barycenter crossing minimisation: what a flowchart screen draws (the built-in NodeGraph), the game marking what t… |
| [`og.FlowSegment`](flow/record-FlowSegment.md) | event |  | A flow began a node's segment. |
| [`og.FlowChoice`](flow/record-FlowChoice.md) | event |  | A flow took a choice at its cut (before og.story ran it). |
| [`og.FlowChapter`](flow/record-FlowChapter.md) | event |  | A flow entered a chapter's node (`og-flow-chapter`): a place to save. |
| [`og.FlowEnded`](flow/record-FlowEnded.md) | event |  | A flow's last segment reached its cut with the conversation ended. |
| [`og.FlowInspected`](flow/record-FlowInspected.md) | event |  | An `inspect` hotspot was triggered without cutting (og.flow.inspect). |
| [`og.FlowQteResult`](flow/record-FlowQteResult.md) | event |  | A quick-time event resolved with a grade (`perfect`, `good`, or `miss`). |
| [`og.flow.load`](flow/fn-load.md) | function | `(world: World, dialogue: string): Result<FlowInfo>` | Prepare a loaded dialogue's flow: every segment's timeline (and through it every video, which fetches nothing until it plays) and og.flow's records. |
| [`og.flow.start`](flow/fn-start.md) | function | `(world: World, desc?: FlowStartDesc): Result<Entity>` | Start a flow: its decks, and its conversation held at the first flow node, whose segment plays from the next tick. |
| [`og.flow.current`](flow/fn-current.md) | function | `(world: World): Entity \| null` | The flow project.ogt's `flow` line started before GameDesc.start (the engine loads its dialogue, prepares it and starts it, and its FlowScreen draws it unless the line's block says `screen game`): wh… |
| [`og.flow.view`](flow/fn-view.md) | function | `(world: World, flow: Entity): FlowView \| null` | What a flow shows now: its picture, segment time, choices and window. |
| [`og.flow.choose`](flow/fn-choose.md) | function | `(world: World, flow: Entity, index: number): Result<void>` | Take a choice while the window is open: recorded now (FlowView.window.pending; a later call replaces it) and taken at the cut (or immediately when tagged `#immediate`, held, or in a choice loop). |
| [`og.flow.advance`](flow/fn-advance.md) | function | `(world: World, flow: Entity, outcome?: string): Result<void>` | End the segment playing now, as its cut would: with `outcome`, the node's `#outcome:<name>` choice; without, the choice taken in the window, else the default (or its own way on, for a node without ch… |
| [`og.flow.play`](flow/fn-play.md) | function | `(world: World, flow: Entity): Result<void>` | Play a paused flow from where it is. |
| [`og.flow.pause`](flow/fn-pause.md) | function | `(world: World, flow: Entity): Result<void>` | Hold a flow on its frame (a pause menu); its window's time holds too. |
| [`og.flow.setSpeed`](flow/fn-setSpeed.md) | function | `(world: World, flow: Entity, speed: number): Result<void>` | Set playback speed (0.25 to 4.0) for the flow's active segment (fast-forward / slow-motion). |
| [`og.flow.seek`](flow/fn-seek.md) | function | `(world: World, flow: Entity, time: number): Result<void>` | Seek within the current segment (0 to its cut time, e.g., -5s rewind). |
| [`og.flow.inspect`](flow/fn-inspect.md) | function | `(world: World, flow: Entity, hotspot: string): Result<void>` | Trigger an active `inspect` hotspot by id without cutting: marks it inspected and sends og.FlowInspected. |
| [`og.flow.gesture`](flow/fn-gesture.md) | function | `(world: World, flow: Entity, desc?: FlowGestureDesc): Result<void>` | Feed a deterministic swipe or drag gesture into active QTEs (mobile touch support). |
| [`og.flow.setMusic`](flow/fn-setMusic.md) | function | `(world: World, flow: Entity, desc?: FlowMusicDesc): Result<void>` | Start, crossfade, or stop persistent cross-segment background music with automatic ducking over segment audio. |
| [`og.flow.history`](flow/fn-history.md) | function | `(world: World, flow: Entity): readonly FlowTranscriptLine[]` | Recent captions and player choices recorded by this flow (backlog). |
| [`og.flow.checkpoint`](flow/fn-checkpoint.md) | function | `(world: World, flow: Entity): Result<FlowCheckpoint>` | Capture a lightweight branch-rewind checkpoint: the current dialogue, flow node, and story variables. |
| [`og.flow.jump`](flow/fn-jump.md) | function | `(world: World, flow: Entity, desc?: FlowJumpDesc): Result<void>` | Jump a running flow directly to another flow node, optionally applying story variables from a FlowCheckpoint. |
| [`og.flow.skip`](flow/fn-skip.md) | function | `(world: World, flow: Entity): Result<void>` | Skip ahead in the segment playing: to its window's open (flow.open) when that is ahead, else to its cut, which takes the pending choice, else the default. |
| [`og.flow.chart`](flow/fn-chart.md) | function | `(world: World, dialogue: string): Result<FlowChart>` | A prepared dialogue's flow nodes and where each may lead, laid out for a flowchart. |
| [`og.flow.stop`](flow/fn-stop.md) | function | `(world: World, flow: Entity): Result<void>` | End a flow: its conversation stops and its decks and entity are despawned this tick. |
