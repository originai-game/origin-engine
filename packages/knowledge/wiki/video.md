<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `video` module

Footage in a game (docs/design.md §2.9): a `type video` asset cooked by ffmpeg, loaded with og.load.video and played by og.VideoPlayer. A clip (the default, for short loops: at most 256 frames) is cooked into a texture array of its frames; a stream (`stream true`: any length) is cooked to AV1 and decoded as it plays, natively by rav1d on a thread of its own, on the web by WebCodecs, or by rav1d in a Worker of its own where the browser has no AV1 decoder (og.video.capabilities says which). Either way the frame shown is a Texture (VideoInfo.texture, also VideoPlayer.texture) any material map, og.Sprite or UI image draws, and the sound plays on the entity's og.AudioSource in step with it.

Playback is simulation, decoding is presentation: the og.video system (after the game's systems, before og.audio) advances VideoPlayer.time by the tick's dt times `speed` (not the time scale, so the picture stays with its sound; a paused game, whose ticks do not run, pauses both), so the frame a player is at on a tick is the same on both hosts, in every run and after a snapshot is restored. With a 30 Hz tick a player started at 0 is at the source's frame 3 s in at tick 90. A stream's texture shows that frame once it is decoded: an executor (og run, og render, og test) waits for it, so every capture shows exactly its tick's frame; a game on the web keeps the last frame decoded until it is, and nothing the decoder has or has not finished reaches the world.

- **TypeScript**: functions are the namespace `og.video`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module video {
  doc "Footage in a game (docs/design.md §2.9): a `type video` asset cooked by ffmpeg, loaded with og.load.video and played by og.VideoPlayer. A clip (the default, for short loops: at most 256 frames) is cooked into a texture array of its frames; a stream (`stream true`: any length) is cooked to AV1 and decoded as it plays, natively by rav1d on a thread of its own, on the web by WebCodecs, or by rav1d in a Worker of its own where the browser has no AV1 decoder (og.video.capabilities says which). Either way the frame shown is a Texture (VideoInfo.texture, also VideoPlayer.texture) any material map, og.Sprite or UI image draws, and the sound plays on the entity's og.AudioSource in step with it."
  doc "Playback is simulation, decoding is presentation: the og.video system (after the game's systems, before og.audio) advances VideoPlayer.time by the tick's dt times `speed` (not the time scale, so the picture stays with its sound; a paused game, whose ticks do not run, pauses both), so the frame a player is at on a tick is the same on both hosts, in every run and after a snapshot is restored. With a 30 Hz tick a player started at 0 is at the source's frame 3 s in at tick 90. A stream's texture shows that frame once it is decoded: an executor (og run, og render, og test) waits for it, so every capture shows exactly its tick's frame; a game on the web keeps the last frame decoded until it is, and nothing the decoder has or has not finished reaches the world."
  # declarations follow, each on its own page
```

<sub>`api.ogt` line 25634</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`Video`](video/type-Video.md) | handle |  | Cooked footage: its frames as a texture array, its sound (og.load.video). |
| [`Video()`](#column-Video) | column kind |  | A Video handle field (0 is none). |
| [`VideoInfo`](video/type-VideoInfo.md) | struct |  | What og.load.video loaded. |
| [`VideoCapabilities`](video/type-VideoCapabilities.md) | struct |  | How this host plays streams. |
| [`og.video.capabilities`](video/fn-capabilities.md) | function | `(world: World): VideoCapabilities` | How streams play on this host: the decoder, whether it is hardware and its largest real-time size. |
| [`og.VideoPlayer`](video/record-VideoPlayer.md) | component |  | Plays a loaded video: VideoInfo.texture shows its current frame (frame n is the source at n / fps seconds; a stream's once decoded), and when the entity has an og.AudioSource the og.video system sets… |

## Column kinds

Each is a factory, `Video(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-Video"></a>`Video` | `Video` | `Uint32Array` | 2 | A Video handle field (0 is none). |

```ogt
column Video {
  doc "A Video handle field (0 is none)."
  array Uint32Array
  stride 2
  value Video
}
```

<sub>`api.ogt` line 25640</sub>
