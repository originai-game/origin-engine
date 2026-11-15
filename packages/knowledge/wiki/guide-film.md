<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `film` guide: Make an interactive film

An interactive film is a Yarn dialogue whose nodes play footage. The story
stays in Yarn (`yarn`), each segment is a timeline with a video track
(`animation §13`), and the footage is a streamed video (`audio §8`).
og.film moves the conversation from segment to segment and warms the
segments a choice may lead to. Nothing else is new: captions, markers,
transitions and sound are the timeline's, saves are the world's.

<a id="guide-film-1"></a>
### `film §1` Lay out the files

```text
assets/story/film.yarn        # the story: nodes, choices, variables, endings
assets/segments/kitchen.ogt   # a segment: type timeline with one video track
assets/clips/kitchen.ogt      # footage: type video, stream true
```

Load the dialogue, then the film, in GameDesc.start, in every process that
may restore a film:

```ts
og.story.load(world, "story/film.yarn");
og.film.load(world, "film");
const film = og.film.start(world, { dialogue: "film", paused: true });
```

`og.film.load` loads every segment and through it every video. A stream
fetches nothing until it plays, so a film of hundreds of segments loads
them all at start.

<a id="guide-film-2"></a>
### `film §2` Write a film node

```yarn
title: kitchen
og-film: segments/kitchen.ogt
og-film-chapter: chapter.kitchen
---
-> Take the knife #line:kitchen.knife #hotspot:knife
    <<set $brave = true>>
    <<jump hall>>
-> Open the door #line:kitchen.door #default
    <<jump door>>
===
```

| Header or tag | Meaning |
|---|---|
| `og-film: <timeline>` | The segment this node plays. A node without it is a logic node: it runs through at once (branch on variables there) |
| `og-film-chapter: <key>` | Entering it sends og.FilmChapter: a chapter to list and to save at |
| `og-film-ending: <id>` | Its segment's cut with the conversation over sends og.FilmEnded with this id |
| `#default` on a choice | Taken when the window locks with nothing chosen. Put it on exactly one choice of each film node with choices; without one the first enabled choice is taken |

A film node holds no Yarn lines: what is said is the segment's caption
track, so there is one source of subtitles. Choice text is the Yarn
option's, translated as any line is.

<a id="guide-film-3"></a>
### `film §3` Write a segment

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
    name "film.open"
  }
  key 17 {
    name "film.lock"
  }
}
```

- The video track names no entity: it plays on the og.Timeline's own
  player, and og.film plays each segment on a deck, an entity with an
  og.Timeline and an og.VideoPlayer. A segment has exactly one such track.
- `film.open` shows the choices, `film.lock` ends choosing, and the cut is
  `film.cut` if the segment has one, else its end. Keep
  `film.open < film.lock ≤ cut`.
- A node without choices plays to its cut and goes on.
- `from` lets several segments share one long file, which the one-decoder
  mode needs (§6).

Thresholds that read well:

| | |
|---|---|
| Window (open to lock) | 4 to 10 s; under 3 s only for a QTE-style beat |
| Lock to cut | 0.5 to 1.5 s: the time the next segment has to start from a warm frame |
| Key frame interval | at most 1 s at the cut (`keyframe`), so a cut never waits on a seek |
| Chunk | the default 6 s; 2 to 4 s for films that cut every few seconds |

<a id="guide-film-4"></a>
### `film §4` Show the film

Each tick, read `og.film.view`: put its `texture` on a full-screen sprite
or UI image, and while `window` is set offer `choices` with the time left
(`ticksLeft / ticksTotal`). Call `og.film.choose` when one is picked; the
footage plays on to the cut, and a later pick replaces it until the window
locks.

The built-in `film-choices` screen draws the window
(`og.load.document(world, "@ui/screens/film-choices.svelte")`, then
`og.ui.push(world, "film-choices")`). Write its stores from the view each
tick: `open` (the window is set), `choices` (each ChoiceView as
`{ id: String(index), text, enabled, chosen: index === window.pending }`),
`left` (`ticksLeft / ticksTotal`) and `locked`. Its `choose` action reports
the row's key, the index to pass to `og.film.choose`. Put the subtitles
screen under it for the segment's captions.

On the web, audio starts only after a gesture: start the film paused
behind a title screen and call `og.film.play` from its Start button.

<a id="guide-film-5"></a>
### `film §5` Saves, chapters and seen segments

A film is world state: a snapshot taken mid-segment restores to the same
frame, window and pending choice. On og.FilmChapter, take a snapshot into
a save slot with the chapter's name and the segment's poster
(VideoInfo.poster) as its thumbnail.

What lasts across playthroughs (segments seen, endings reached) is the
game's, kept outside the world. Count it from og.FilmSegment and
og.FilmEnded under these keys, so tools agree:

| Key | Value |
|---|---|
| `film.<dialogue>.seen` | node titles whose segment has played |
| `film.<dialogue>.endings` | ending ids reached |

Telemetry, when a game sends it: `film_choice {node, index, timedOut}`,
`film_ending {ending}`, `film_quit {node}`.

<a id="guide-film-6"></a>
### `film §6` Decks and devices

`FilmDesc.decks` is 3 by default: one plays, two hold the segments the
current node may lead to, paused on their first frame, the chosen one
first, then the default. A cut then shows a frame already decoded.

| Device | Decks |
|---|---|
| Desktop and Android browsers (WebCodecs) | 3 |
| Safari playing the HEVC rendition | 3; measure on the oldest iPhone you ship to |
| `og.video.capabilities().decoder` is `rav1d-wasm` | 2 |
| One decoder at a time (WeChat on iOS) | 1: no warming; cut inside one long file with `from` |

<a id="guide-film-7"></a>
### `film §7` Test every route

The world is deterministic, so a test walks `og.story.graph` from the
start, chooses each choice of each film node in turn with `film:choose:<i>`
and asserts every ending is reached and every segment played. Check also
that a snapshot taken inside a window and restored reaches the same world
hash as the run that was never interrupted.
