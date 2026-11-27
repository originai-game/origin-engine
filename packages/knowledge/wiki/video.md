<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `video` module

Footage authored with file.video in TypeScript, cooked by ffmpeg, loaded with og.load.video and played by og.VideoPlayer. A clip (the default: at most 256 frames) is cooked into a texture array; a stream (`stream: true`: any length) is cooked to AV1 and decoded as it plays. The frame is a Texture; the sound plays on the entity's og.AudioSource in step with it.

Playback is simulation, decoding is presentation: og.video advances VideoPlayer.time each tick, so a player's frame on a tick is the same on both hosts and after a restore; nothing the decoder has finished reaches the world.

See `audio §8`.

- **TypeScript**: functions are the namespace `og.video`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module video {
  doc "Footage authored with file.video in TypeScript, cooked by ffmpeg, loaded with og.load.video and played by og.VideoPlayer. A clip (the default: at most 256 frames) is cooked into a texture array; a stream (`stream: true`: any length) is cooked to AV1 and decoded as it plays. The frame is a Texture; the sound plays on the entity's og.AudioSource in step with it."
  doc "Playback is simulation, decoding is presentation: og.video advances VideoPlayer.time each tick, so a player's frame on a tick is the same on both hosts and after a restore; nothing the decoder has finished reaches the world."
  # declarations follow, each on its own page
```

<sub>Module `video` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`Video`](video/type-Video.md) | handle |  | Cooked footage: its frames as a texture array, its sound (og.load.video). |
| [`Video()`](#column-Video) | column kind |  | A Video handle field (0 is none). |
| [`VideoInfo`](video/type-VideoInfo.md) | struct |  | What og.load.video loaded. |
| [`VideoQuality`](video/type-VideoQuality.md) | enum |  | The quality tier for streamed video playback (presentation-only: never changes simulation). |
| [`VideoBuffering`](video/type-VideoBuffering.md) | struct |  | Presentation-only buffering status of active video streams. |
| [`VideoCapabilities`](video/type-VideoCapabilities.md) | struct |  | How this host plays streams. |
| [`og.video.capabilities`](video/fn-capabilities.md) | function | `(world: World): VideoCapabilities` | How streams play on this host: the decoder, whether it is hardware and its largest real-time size. |
| [`og.video.setQuality`](video/fn-setQuality.md) | function | `(world: World, quality: VideoQuality): Result<void>` | Presentation-only: choose the quality tier (`auto`, `high`, `medium`, `low`) for streamed videos. |
| [`og.video.buffering`](video/fn-buffering.md) | function | `(world: World): VideoBuffering` | Presentation-only: whether any player with `wait` is currently buffering, its download progress, and the active quality tier. |
| [`og.MediaIntent`](video/record-MediaIntent.md) | event |  | A request for a player to change at the next tick boundary, sent through the recorded intent path, so playback stays world state. |
| [`og.VideoPlayer`](video/record-VideoPlayer.md) | component |  | Plays a loaded video: its texture shows its current frame (frame n is the source at n / fps seconds; a stream's once decoded), and when the entity has an og.AudioSource the og.video system sets its s… |

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

<sub>Module `video` line 9</sub>
