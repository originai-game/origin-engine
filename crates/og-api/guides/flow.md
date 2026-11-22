# Make an interactive film, or any game told in segments

A flow is a Yarn dialogue whose nodes play segments: footage, a live
scene, or both, and levels or minigames that wait for the game. The story
stays in Yarn (`yarn`), each segment is a timeline (`animation §13`), and
footage is a streamed video (`audio §8`).
og.flow moves the conversation from segment to segment and warms the
segments a choice may lead to. Nothing else is new: captions, markers,
transitions and sound are the timeline's, saves are the world's.

## 1. Lay out the files

```text
project.ogt                   # flow "story/film.yarn"
assets/story/film.yarn        # the story: nodes, choices, variables, endings
assets/segments/kitchen.ogt   # a segment: type timeline with one video track
assets/clips/kitchen.ogt      # footage: type video, stream true
assets/segments/live.ogt      # a live segment: a scene track and a camera track
assets/scenes/ending.ogt      # the scene it places
```

`og create --genre interactive-film` writes this skeleton: an opening on
placeholder footage (its clip says `placeholder true` until the game's own
footage replaces it), and two endings, one on footage and one live, a set
of built-in geometry the camera track pushes in on.

One line in project.ogt makes the dialogue the game's flow:

```text
flow "story/film.yarn"
```

The engine loads the dialogue, prepares it (og.flow.load) and starts it
(og.flow.start) before GameDesc.start, in every process, and the built-in
`FlowScreen` draws it (§4). `og.flow.current(world)` is its entity: what a
playable segment passes to `og.flow.advance`, and what a title screen
pauses in start and plays from its Start button. The cook counts the
flow's segments, footage and scenes among the bundle's first modules, so
nothing waits on the first cut.

`og.flow.load` loads every segment and through it every video. A stream
fetches nothing until it plays, so a flow of hundreds of segments loads
them all at start. A game with several flows, or one that starts its flow
later, calls the three itself:

```ts
og.story.load(world, "story/side.yarn");
og.flow.load(world, "side");
const flow = og.flow.start(world, { dialogue: "side", paused: true });
```

## 2. Write a flow node

```yarn
title: kitchen
og-flow: segments/kitchen.ogt
og-flow-chapter: chapter.kitchen
---
-> Take the knife #line:kitchen.knife #hotspot:knife
    <<set $brave = true>>
    <<jump hall>>
-> Dodge #line:kitchen.dodge #qte:hold:dodge:400
    <<jump hall>>
-> Open the door #line:kitchen.door #default
    <<jump door>>
===
```

| Header or tag | Meaning |
|---|---|
| `og-flow: <timeline>` | The segment this node plays. A node without it is a logic node: it runs through at once (branch on variables there) |
| `og-flow-chapter: <key>` | Entering it sends og.FlowChapter: a chapter to list and to save at |
| `og-flow-ending: <id>` | Its segment's cut with the conversation over sends og.FlowEnded with this id |
| `og-flow-dissolve: <seconds>` | Its segment dissolves in over the one before (0 to 5 s), which plays on under it; without it the cut is hard |
| `#default` on a choice | Taken when the window locks with nothing chosen. Put it on exactly one choice of each flow node with choices; without one the first enabled choice is taken |
| `#hotspot:<id>` on a choice | Also taken by clicking the hotspot `id` on the picture (a `track hotspot` key of its segment). FlowView.hotspots gives it while the window is open |
| `#qte:press:<action>`, `#qte:hold:<action>:<ms>`, `#qte:mash:<action>:<presses>` on a choice | Taken by an og.input action in the window: one press, held that long, or pressed that many times. Only the node's first 8 choices may be a QTE; FlowView.qtes gives their progress |

A flow node holds no Yarn lines: what is said is the segment's caption
track, so there is one source of subtitles. Choice text is the Yarn
option's, translated as any line is.

## 3. Write a segment

```text
type timeline
version 1
---
duration 18
track video {
  key 0 {
    video "clips/kitchen.ogt"
  }
}
track caption {
  captions "kitchen.srt"
}
track marker {
  key 12 {
    name "flow.open"
  }
  key 17 {
    name "flow.lock"
  }
}
track hotspot {
  key 12 {
    id "knife"
    rect 0.61 0.42 0.12 0.2
  }
  key 16 {
    id "knife"
    rect 0.58 0.45 0.12 0.2
  }
}
```

- The video track names no entity: it plays on the og.Timeline's own
  player, and og.flow plays each segment on a deck, an entity with an
  og.Timeline and an og.VideoPlayer. A segment has at most one such track.
- A `scene` track makes a live segment (a cutscene in the engine, a
  level): its scene is in the world from its key, prepared ahead on a warm
  deck so the cut never waits. Give it a `camera` or `shot` track to frame
  it. With a video track too, the footage plays over the scene. A segment
  has footage, a scene, or both.
- `flow.open` shows the choices, `flow.lock` ends choosing, and the cut is
  `flow.cut` if the segment has one, else its end. Keep
  `flow.open < flow.lock ≤ cut`.
- A node without choices plays to its cut and goes on.
- `from` lets several segments share one long file, which the one-decoder
  mode needs (§6).
- A `hotspot` key gives `id` and `rect x y width height` (fractions of the
  video frame from its top left), or `off true` to end it; it moves linearly
  to the same id's next key. A choice tagged `#hotspot:<id>` is taken by
  clicking it. The id must match a `#hotspot` tag the cook checks.

Thresholds that read well:

| | |
|---|---|
| Window (open to lock) | 4 to 10 s; under 3 s only for a QTE-style beat |
| Lock to cut | 0.5 to 1.5 s: the time the next segment has to start from a warm frame |
| Key frame interval | at most 1 s at the cut (`keyframe`), so a cut never waits on a seek |
| Chunk | the default 6 s; 2 to 4 s for flows that cut every few seconds |

### Playable segments: `flow.hold` and outcomes

A level or a minigame is a segment that stops at its `flow.hold` marker
(it keeps its scene, footage frame and camera) until the game says how it
went. Its results are the node's `#outcome:<name>` choices, which the
player never sees:

```yarn
title: level1
og-flow: segments/level1.ogt
---
-> Retry #line:retry
    <<jump level1>>
-> won #outcome:won
    <<jump bridge>>
-> lost #outcome:lost
    <<jump level1>>
===
```

```ts
if (bossDown) og.flow.advance(world, flow, "won");
```

- The game calls `og.flow.advance(world, flow, "<outcome>")`; a node
  without outcomes takes `og.flow.advance(world, flow)`. It cuts that tick,
  from a system too.
- A held segment has no `flow.lock` or `flow.cut`. Its window opens at
  `flow.open` and stays open while it holds; a choice the player takes
  then (Retry above) cuts at once.
- Put the result in an outcome, never in an `<<if>>` in the node: the node
  runs when its segment starts, before the level is played, and an outcome
  shows on the chart, is prepared ahead and is walked by the route tests.
- Jumping back to the same node plays the level again from its start.
- Every cut snaps the camera (og.render.snapCamera). To ease from the
  game's camera into a segment's and back, give its `camera` track
  `blendIn` and `blendOut` seconds, or use a `shot` track's `blend`.

## 4. Show the flow

Each tick, read `og.flow.view`: put its `texture` on a full-screen sprite
or UI image, and while `window` is set offer `choices` with the time left
(`ticksLeft / ticksTotal`). Call `og.flow.choose` when one is picked; the
footage plays on to the cut, and a later pick replaces it until the window
locks.

The engine mounts the built-in `FlowScreen` for project.ogt's `flow`,
under the game's own UI: the picture filling the surface with its
dissolve (a segment with no footage leaves the live scene showing), the
captions its timelines say (`Subtitles`), and the window (`FlowChoices`:
the rows, the pending one, each QTE's progress, the hotspots, the time
left and the lock), each press taken by og.flow.choose. It reads
og.flow.view each tick; nothing of it is world state.

A game that draws the flow itself says so, and mounts what it wants from
`@origingame/engine/ui/components`: `FlowScreen` for another flow
(`{ flow }`), or its parts, `FlowPicture` (the picture and dissolve, with a
Skip button when given `onSkip`, for a cutscene alone), `Subtitles` and
`FlowChoices`, which take plain values from og.flow.view:

```text
flow "story/film.yarn" {
  screen game
}
```

A test takes a choice without a press through a command
(`flow:choose:<index>`) or a playtest.

The screen's hotspot layer fills the surface, so a hotspot's `rect`
(fractions of the picture) lands on the picture only when the game draws
`texture` to fill the same surface (the usual FMV presentation). A game that
letterboxes the video places its own hotspots over where it drew it.

A choice is also taken by its QTE (`#qte` on it) or its hotspot (`#hotspot`):
og.flow records the QTE itself from the actions the tags name, and
`FlowChoices` shows its progress from the view. `og.flow.skip` jumps a seen segment
to its window, then to its cut.

On the web, audio starts only after a gesture, and the page's play gate
takes it before the first tick. A game with its own title screen pauses
`og.flow.current(world)` in start and calls `og.flow.play` from its Start
button.

## 5. Saves, chapters and seen segments

A flow is world state: a snapshot taken mid-segment restores to the same
frame, window and pending choice. On og.FlowChapter, take a snapshot into
a save slot with the chapter's name and the segment's poster
(VideoInfo.poster) as its thumbnail.

The built-in `flow-chapters` screen lists og.flow.load's `chapters`, each
`{ id, title, thumb, locked }` from a chapter's save; `select` plays from
it. `flow-gallery` lists its `endings`, locked until reached.
`FlowChartScreen` draws `og.flow.chart` (a dialogue's flow nodes by
depth, through logic nodes) through the built-in NodeGraph: pass its nodes
as `{ id, x, y, label, icon, state, shape, team }`, marking `done` the
segments the player has seen and `current` the one playing, its links with
their `points`, and `view` as `[0, 0, chart.width, chart.height]`. It
reports `onPick(e, id)` for a node, `onPlay` for Play from here (only while
`canPlay`), and `onZoom` and `onPan` for the zoom and pan the game keeps.

What lasts across playthroughs (segments seen, endings reached) is the
game's, kept outside the world. Count it from og.FlowSegment and
og.FlowEnded under these keys, so tools agree:

| Key | Value |
|---|---|
| `flow.<dialogue>.seen` | node titles whose segment has played |
| `flow.<dialogue>.endings` | ending ids reached |

Telemetry, when a game sends it: `flow_choice {node, index, timedOut}`,
`flow_ending {ending}`, `flow_quit {node}`.

## 6. Decks and devices

`FlowStartDesc.decks` is 3 by default: one plays, two hold the segments the
current node may lead to, paused on their first frame, the chosen one
first, then the default. A cut then shows a frame already decoded. A deck
that holds a segment is its timeline in pre-roll (og.Timeline.preroll):
nothing of the segment is seen or heard until the cut plays it.

| Device | Decks |
|---|---|
| Desktop and Android browsers (WebCodecs) | 3 |
| Safari playing the HEVC rendition | 3; measure on the oldest iPhone you ship to |
| `og.video.capabilities().decoder` is `rav1d-wasm` | 2 |
| One decoder at a time (WeChat on iOS) | 1: no warming; cut inside one long file with `from` |

On one decoder (`decks 1`) a cut reloads the deck, so make it cheap: keep
the segments in one long file (a segment's `from` is where it starts), and
align each segment's start to a multiple of the clip's `keyframe` seconds
(the default 1 s). The encoder puts a key frame every `keyframe` seconds, so
a cut that lands on one decodes a single frame, not a run. A small
`keyframe` makes cuts snappier at some size cost.

## 7. Make the footage (AI models)

When a generative video model makes the clips, two cook settings keep them
consistent and one tool continues them:

- `loudness -16` in a `type video` normalises its sound to −16 LUFS, so
  clips a model made at different levels play the same.
- `aiGenerated true` marks the footage; VideoInfo.aiGenerated carries it, so
  a game or store labels AI content.
- `og video edges <clip>` writes the first and last frame of a clip as PNG.
  Feed the last frame to an image-to-video model as the seed for the next
  clip, so the cut joins; the first frame checks it against the clip before.

## 8. Test every route

`og flow list --json` is the flow as the cook reads it, without running
it: per flow dialogue (the project.ogt one first) each node with its
segment (footage, live or both, its length, window and hold), its
footage (missing, streamed, placeholder), its scenes, choices, outcomes,
hotspots, the nodes each leads to and the `problems` found. `og flow
routes --json` walks every route from the start to an ending without
passing a node twice: each step is the node, what takes it (`choose`,
`advance` with the outcome, or `cut`) and the node it reaches, marked
`condition` where an `<<if>>` decides between several. Its `unreached`
names the endings no route reaches. Each route's steps are the commands a
test sends: `flow:choose:<i>` in the window, `flow:advance:<outcome>` for a
held segment.

The probe's `dump_flow` is the running side: per flow its node, paused,
held and ended, and per deck the node it holds (current, previous, warm or
free), its segment's time and pre-roll, whether each scene its scene track
places is prepared (and why not) and the engine modules the segment
needs. Take it just before a cut to see that the segment cut to is warm.

The world is deterministic, so a test walks `og.story.graph` from the
start, chooses each choice of each flow node in turn with `flow:choose:<i>`
and asserts every ending is reached and every segment played. Check also
that a snapshot taken inside a window and restored reaches the same world
hash as the run that was never interrupted. A command runs between ticks
and its restore applies before the next tick moves anything, so one
command that snapshots and restores leaves the run on the same ticks:

```ts
// GameDesc.commands
qa_roundtrip: (world) => {
  const save = world.snapshot();
  if (!save.ok) throw new Error(save.reason);
  const r = world.restore(save.value);
  if (!r.ok) throw new Error(r.reason);
},
```

```ts
const choose = "10.6:flow:choose:0";
const plain = await runPlaytest({ commands: `${choose};19.6:quit` });
const restored = await runPlaytest({ commands: `${choose};11.1:qa_roundtrip;19.6:quit` });
expect(restored.tick).toBe(plain.tick);
expect(restored.hash).toBe(plain.hash);
```

## 9. Cutscenes and minigames in any game

A flow is not only a film. Any game plays its cutscenes and CG through
one, and any flow can stop for a game.

- **A cutscene in a game.** Give the cutscene its own dialogue, one node a
  segment (footage, a live scene with a `camera` track, or both), and
  start it when the story reaches it: `og.story.load`, `og.flow.load` in
  setup, then `og.flow.start(world, { dialogue: "boss-intro" })`. Draw it
  with `FlowPicture` and `Subtitles`, or `FlowScreen { flow }`, and give
  `FlowPicture` an `onSkip` that calls `og.flow.skip`. Its view's `ended`
  says when to `og.flow.stop` it and give the player back the game.
- **A minigame between segments.** Make it a playable segment (§3): the
  node's segment holds at `flow.hold`, the game runs its own systems over
  it, 2D or 3D, and ends it with
  `og.flow.advance(world, og.flow.current(world), "won")`. The outcome
  names the branch, so the chart and `og flow routes` show it.
- **A live scene between clips.** A segment with a `scene` track and no
  video shows the real-time world; with both, the footage plays over
  it. The decks warm
  its scene before the cut as they warm footage, so it never hitches.
  Give a live scene its own light: a `resource Sky { ... }` and its lights
  in the scene file. Without a Sky it is lit and reflected by the look's
  daylight sky even where the camera's dark background shows, so a night
  set comes out flat and pale.
