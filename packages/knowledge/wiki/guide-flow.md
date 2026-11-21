<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `flow` guide: Make an interactive film

An interactive film is a Yarn dialogue whose nodes play footage. The story
stays in Yarn (`yarn`), each segment is a timeline with a video track
(`animation §13`), and the footage is a streamed video (`audio §8`).
og.flow moves the conversation from segment to segment and warms the
segments a choice may lead to. Nothing else is new: captions, markers,
transitions and sound are the timeline's, saves are the world's.

<a id="guide-flow-1"></a>
### `flow §1` Lay out the files

```text
assets/story/film.yarn        # the story: nodes, choices, variables, endings
assets/segments/kitchen.ogt   # a segment: type timeline with one video track
assets/clips/kitchen.ogt      # footage: type video, stream true
```

Load the dialogue, then the flow, in GameDesc.start, in every process that
may restore a flow:

```ts
og.story.load(world, "story/film.yarn");
og.flow.load(world, "film");
const flow = og.flow.start(world, { dialogue: "film", paused: true });
```

`og.flow.load` loads every segment and through it every video. A stream
fetches nothing until it plays, so a flow of hundreds of segments loads
them all at start.

<a id="guide-flow-2"></a>
### `flow §2` Write a flow node

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

<a id="guide-flow-3"></a>
### `flow §3` Write a segment

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
  og.Timeline and an og.VideoPlayer. A segment has exactly one such track.
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

<a id="guide-flow-4"></a>
### `flow §4` Show the flow

Each tick, read `og.flow.view`: put its `texture` on a full-screen sprite
or UI image, and while `window` is set offer `choices` with the time left
(`ticksLeft / ticksTotal`). Call `og.flow.choose` when one is picked; the
footage plays on to the cut, and a later pick replaces it until the window
locks.

The built-in `FlowChoices` draws the window and `Subtitles` the segment's
captions (`@origingame/engine/ui/components`). Both take plain values:
keep a signal per world, set it each tick from `og.flow.view` and
`og.timeline.captions`, and pass it to the mounts. `FlowChoices` derives
the choice rows, the pending one, each QTE's progress, the hotspots, the
time left and the lock from the view itself, and calls `onChoose` with the
ChoiceView.index of a row or hotspot pressed (nothing after the lock):

```ts
import { createSignal, mount } from "@origingame/engine/ui";
import { FlowChoices, Subtitles } from "@origingame/engine/ui/components";

const view = createSignal<FlowView | null>(null);        // one per world
const captions = createSignal<readonly TimelineCaption[]>([]);

// A system, each tick:
view.set(og.flow.view(world, flow));
captions.set(og.timeline.captions(world));

// GameDesc.ui:
mount(world, FilmView, { get picture() { return view()?.texture ?? 0; }, ... });
mount(world, Subtitles, { get captions() { return captions(); } });
mount(world, FlowChoices, {
  get view() { return view(); },
  onChoose: (e, index) => void og.flow.choose(e.world, flow, index),
});
```

Mount `Subtitles` before `FlowChoices`, so the choices lie over the
captions. `og new --form film` writes this whole presentation
(`src/main.ts`, `assets/ui/film-view.tsx`). A test takes a choice without
a press through a command (`flow:choose:<index>`) or a playtest.

The screen's hotspot layer fills the surface, so a hotspot's `rect`
(fractions of the picture) lands on the picture only when the game draws
`texture` to fill the same surface (the usual FMV presentation). A game that
letterboxes the video places its own hotspots over where it drew it.

A choice is also taken by its QTE (`#qte` on it) or its hotspot (`#hotspot`):
og.flow records the QTE itself from the actions the tags name, and
`FlowChoices` shows its progress from the view. `og.flow.skip` jumps a seen segment
to its window, then to its cut.

On the web, audio starts only after a gesture: start the flow paused
behind a title screen and call `og.flow.play` from its Start button.

<a id="guide-flow-5"></a>
### `flow §5` Saves, chapters and seen segments

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

<a id="guide-flow-6"></a>
### `flow §6` Decks and devices

`FlowStartDesc.decks` is 3 by default: one plays, two hold the segments the
current node may lead to, paused on their first frame, the chosen one
first, then the default. A cut then shows a frame already decoded.

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

<a id="guide-flow-7"></a>
### `flow §7` Make the footage (AI models)

When a generative video model makes the clips, two cook settings keep them
consistent and one tool continues them:

- `loudness -16` in a `type video` normalises its sound to −16 LUFS, so
  clips a model made at different levels play the same.
- `aiGenerated true` marks the footage; VideoInfo.aiGenerated carries it, so
  a game or store labels AI content.
- `og video edges <clip>` writes the first and last frame of a clip as PNG.
  Feed the last frame to an image-to-video model as the seed for the next
  clip, so the cut joins; the first frame checks it against the clip before.

<a id="guide-flow-8"></a>
### `flow §8` Test every route

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
