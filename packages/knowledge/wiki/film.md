<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `film` module

Interactive film (FMV): a Yarn dialogue paced by its footage. A node with an `og-film: <timeline>` header plays that segment (a timeline with a video track) on a deck; its choices open at the segment's `film.open` marker, lock at `film.lock` and are taken at its cut (`film.cut`, else its end), the default (`#default`) when none was chosen.

The story stays og.story's, the footage og.video's, the timing og.timeline's: og.film moves the conversation between segments and warms those a choice may lead to on its other decks. All of it is world state.

See `film`.

- **TypeScript**: functions are the namespace `og.film`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module film {
  doc "Interactive film (FMV): a Yarn dialogue paced by its footage. A node with an `og-film: <timeline>` header plays that segment (a timeline with a video track) on a deck; its choices open at the segment's `film.open` marker, lock at `film.lock` and are taken at its cut (`film.cut`, else its end), the default (`#default`) when none was chosen."
  doc "The story stays og.story's, the footage og.video's, the timing og.timeline's: og.film moves the conversation between segments and warms those a choice may lead to on its other decks. All of it is world state."
  # declarations follow, each on its own page
```

<sub>Module `film` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`FilmDesc`](film/type-FilmDesc.md) | desc struct |  | A film to start. |
| [`FilmInfo`](film/type-FilmInfo.md) | struct |  | What og.film.load prepared. |
| [`og.Film`](film/record-Film.md) | component |  | A film og.film.start made: the conversation it paces and its decks. |
| [`og.FilmDeck`](film/record-FilmDeck.md) | component |  | An entity og.film plays segments on, with an og.Timeline, an og.VideoPlayer (its `wait` set) and an og.AudioSource. |
| [`WindowView`](film/type-WindowView.md) | struct |  | A segment's choice window. |
| [`FilmView`](film/type-FilmView.md) | struct |  | What a film shows now. |
| [`og.FilmSegment`](film/record-FilmSegment.md) | event |  | A film began a node's segment. |
| [`og.FilmChoice`](film/record-FilmChoice.md) | event |  | A film took a choice at its cut (before og.story ran it). |
| [`og.FilmChapter`](film/record-FilmChapter.md) | event |  | A film entered a chapter's node (`og-film-chapter`): a place to save. |
| [`og.FilmEnded`](film/record-FilmEnded.md) | event |  | A film's last segment reached its cut with the conversation ended. |
| [`og.film.load`](film/fn-load.md) | function | `(world: World, dialogue: string): Result<FilmInfo>` | Prepare a loaded dialogue's film: every segment's timeline (and through it every video, which fetches nothing until it plays) and og.film's records. |
| [`og.film.start`](film/fn-start.md) | function | `(world: World, desc?: FilmDesc): Result<Entity>` | Start a film: its decks, and its conversation held at the first film node, whose segment plays from the next tick. |
| [`og.film.view`](film/fn-view.md) | function | `(world: World, film: Entity): FilmView \| null` | What a film shows now: its picture, segment time, choices and window. |
| [`og.film.choose`](film/fn-choose.md) | function | `(world: World, film: Entity, index: number): Result<void>` | Take a choice while the window is open: recorded now (FilmView.window.pending; a later call replaces it) and taken at the cut, so the footage plays on to it. |
| [`og.film.play`](film/fn-play.md) | function | `(world: World, film: Entity): Result<void>` | Play a paused film from where it is. |
| [`og.film.pause`](film/fn-pause.md) | function | `(world: World, film: Entity): Result<void>` | Hold a film on its frame (a pause menu); its window's time holds too. |
| [`og.film.stop`](film/fn-stop.md) | function | `(world: World, film: Entity): Result<void>` | End a film: its conversation stops and its decks and entity are despawned this tick. |
