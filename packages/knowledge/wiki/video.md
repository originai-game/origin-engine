<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `video` module

Footage in a game (docs/design.md §2.9): a `type video` asset cooked by ffmpeg, loaded with og.load.video and played by og.VideoPlayer. A clip (the default, for short loops: at most 256 frames) is cooked into a texture array of its frames; a stream (`stream true`: any length) is cooked to AV1 and decoded as it plays, natively by rav1d on a thread of its own, on the web by WebCodecs, or by rav1d in a Worker of its own where the browser has no AV1 decoder (og.video.capabilities says which). Either way the frame shown is a Texture (VideoInfo.texture, also VideoPlayer.texture) any material map, og.Sprite or UI image draws, and the sound plays on the entity's og.AudioSource in step with it.

Playback is simulation, decoding is presentation: the og.video system (after the game's systems, before og.audio) advances VideoPlayer.time by the tick's dt times `speed` (not the time scale, so the picture stays with its sound; a paused game, whose ticks do not run, pauses both), so the frame a player is at on a tick is the same on both hosts, in every run and after a snapshot is restored. With a 30 Hz tick a player started at 0 is at the source's frame 3 s in at tick 90. A stream's texture shows that frame once it is decoded: an executor (og run, og render, og test) waits for it, so every capture shows exactly its tick's frame; a game on the web keeps the last frame decoded until it is, and nothing the decoder has or has not finished reaches the world.

- **TypeScript**: functions are the namespace `og.video`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-render`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module video {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 27547</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`Video`](#type-Video) | handle | Cooked footage: its frames as a texture array, its sound (og.load.video). |
| [`Video()`](#column-Video) | column kind | A Video handle field (0 is none). |
| [`VideoInfo`](#type-VideoInfo) | struct | What og.load.video loaded. |
| [`VideoCapabilities`](#type-VideoCapabilities) | struct | How this host plays streams. Backend limits are reported here, never by missing members: every host loads and plays a stream through the same API. |
| [`og.video.capabilities`](#fn-capabilities) | function | How streams play on this host: the decoder, whether it is hardware and its largest real-time size. |
| [`og.VideoPlayer`](#record-VideoPlayer) | component | Plays a loaded video: VideoInfo.texture shows its current frame (frame n is the source at n / fps seconds; a stream's once decoded), and when the entity has an og.AudioSource the og.video system sets its sound, playing, pitch, loop and loop region (the picture's length) each tick so the sound plays in step, however long or often it loops: a seek or a restart starts the sound again at the new time on the next tick (AudioSource.startOffset). Each player has a texture of its own (written to `texture`): the first player of a video takes VideoInfo.texture, each further one a texture the og.video system makes for it over the same decoded frames (one frame of GPU memory each), so two players at different times show different frames. Show a player's `texture`, not VideoInfo.texture, once a video has more than one player. |

## Functions

<a id="fn-capabilities"></a>

### `og.video.capabilities`

How streams play on this host: the decoder, whether it is hardware and its largest real-time size.

```ts
og.video.capabilities(world: World): VideoCapabilities
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world asked. |

**Returns** `VideoCapabilities`.

Binding symbol `og_video_capabilities` · publish builds: kept

```ogt
fn capabilities {
  doc "How streams play on this host: the decoder, whether it is hardware and its largest real-time size."
  param world World "The world asked."
  returns VideoCapabilities
}
```

<sub>`api.ogt` line 27591</sub>

## Components, resources and events

<a id="record-VideoPlayer"></a>

### `og.VideoPlayer` (component)

Plays a loaded video: VideoInfo.texture shows its current frame (frame n is the source at n / fps seconds; a stream's once decoded), and when the entity has an og.AudioSource the og.video system sets its sound, playing, pitch, loop and loop region (the picture's length) each tick so the sound plays in step, however long or often it loops: a seek or a restart starts the sound again at the new time on the next tick (AudioSource.startOffset). Each player has a texture of its own (written to `texture`): the first player of a video takes VideoInfo.texture, each further one a texture the og.video system makes for it over the same decoded frames (one frame of GPU memory each), so two players at different times show different frames. Show a player's `texture`, not VideoInfo.texture, once a video has more than one player.

Token `og.VideoPlayer`, registered under the schema name `og.VideoPlayer`; its value type is `VideoPlayer`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `video` | `Video` | `0` | — | — | — | The video (og.load.video's `video`); 0 plays nothing. |
| `playing` | `boolean` | `true` | — | — | — | Whether time advances. |
| `time` | `number` | `0` | ≥ 0 | `s` | — | Seconds into the video; written every tick, set it to seek. Double precision, as the sound's playhead: summed in f32, an hour at 60 Hz would fall 2.8 s behind its sound. |
| `speed` | `number` | `1` | 0 … 4 | — | tunable | The playback rate: 1 as cooked, 0.5 half speed (the sound's pitch follows). |
| `loop` | `boolean` | `false` | — | — | — | Start again at 0 past the end; otherwise hold the last frame and set finished. |
| `frame` | `number` | `0` | — | — | read only | Written: the frame shown, floor(time * fps). |
| `finished` | `boolean` | `false` | — | — | read only | Written: a player without loop reached the end. |
| `texture` | `Texture` | `0` | — | — | read only | Written: this player's texture (VideoInfo.texture for the video's first player, a texture of its own for each further one), for a material, sprite or UI image set up from the player. It keeps it while it lives. |

```ogt
component VideoPlayer {
  doc "Plays a loaded video: VideoInfo.texture shows its current frame (frame n is the source at n / fps seconds; a stream's once decoded), and when the entity has an og.AudioSource the og.video system sets its sound, playing, pitch, loop and loop region (the picture's length) each tick so the sound plays in step, however long or often it loops: a seek or a restart starts the sound again at the new time on the next tick (AudioSource.startOffset). Each player has a texture of its own (written to `texture`): the first player of a video takes VideoInfo.texture, each further one a texture the og.video system makes for it over the same decoded frames (one frame of GPU memory each), so two players at different times show different frames. Show a player's `texture`, not VideoInfo.texture, once a video has more than one player."
  field video Video "The video (og.load.video's `video`); 0 plays nothing."
  field playing bool "Whether time advances." {
    default true
  }
  field time f64 "Seconds into the video; written every tick, set it to seek. Double precision, as the sound's playhead: summed in f32, an hour at 60 Hz would fall 2.8 s behind its sound." {
    min 0
    unit s
  }
  field speed f32 "The playback rate: 1 as cooked, 0.5 half speed (the sound's pitch follows)." {
    default 1
    min 0
    max 4
    tunable true
  }
  field loop bool "Start again at 0 past the end; otherwise hold the last frame and set finished."
  field frame u32 "Written: the frame shown, floor(time * fps)." {
    readonly true
  }
  field finished bool "Written: a player without loop reached the end." {
    readonly true
  }
  field texture Texture "Written: this player's texture (VideoInfo.texture for the video's first player, a texture of its own for each further one), for a material, sprite or UI image set up from the player. It keeps it while it lives." {
    readonly true
  }
}
```

<sub>`api.ogt` line 27597</sub>

## Types

<a id="type-Video"></a>

### `Video` (handle)

Cooked footage: its frames as a texture array, its sound (og.load.video).

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Video "Cooked footage: its frames as a texture array, its sound (og.load.video)."
```

<sub>`api.ogt` line 27552</sub>

<a id="type-VideoInfo"></a>

### `VideoInfo` (struct)

What og.load.video loaded.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `video` | `Video` | `0` | — | — | — | The video. |
| `texture` | `Texture` | `0` | — | — | — | The texture that shows the frame its og.VideoPlayer is at (frame 0 until one plays it). |
| `sound` | `Sound` | `0` | — | — | — | Its sound track (og.load.sound's kind of handle, for an og.AudioSource); 0 when it has none or was cooked with `audio false`. |
| `width` | `number` | `0` | — | — | — | Each frame's width in texels. |
| `height` | `number` | `0` | — | — | — | Each frame's height in texels. |
| `frames` | `number` | `0` | — | — | — | Frames kept (at most 256). |
| `fps` | `number` | `0` | — | — | — | Frames per second kept. |
| `seconds` | `number` | `0` | — | `s` | — | frames / fps. |
| `format` | `string` | `""` | — | — | — | The frames' encoding on the GPU (`rgba8`, `bc7`, `astc4x4`, `etc2`; a stream's texture is `rgba8`). |
| `bytes` | `number` | `0` | — | `bytes` | — | Bytes its frames take on the GPU (a stream: its texture's, one frame). |
| `streamed` | `boolean` | `false` | — | — | — | Whether it is a stream (`stream true`: decoded as it plays) rather than a clip of cooked frames. |
| `codec` | `string` | `""` | — | — | — | A stream's codec (`av1`); empty for a clip. |
| `poster` | `Texture` | `0` | — | — | — | A stream's first frame as an ordinary texture, ready as soon as the bundle is (a menu's thumbnail, the picture while the stream is fetched); for a clip, VideoInfo.texture. |

```ogt
struct VideoInfo {
  doc "What og.load.video loaded."
  field video Video "The video."
  field texture Texture "The texture that shows the frame its og.VideoPlayer is at (frame 0 until one plays it)."
  field sound Sound "Its sound track (og.load.sound's kind of handle, for an og.AudioSource); 0 when it has none or was cooked with `audio false`."
  field width u32 "Each frame's width in texels."
  field height u32 "Each frame's height in texels."
  field frames u32 "Frames kept (at most 256)."
  field fps u32 "Frames per second kept."
  field seconds f32 "frames / fps." {
    unit s
  }
  field format String "The frames' encoding on the GPU (`rgba8`, `bc7`, `astc4x4`, `etc2`; a stream's texture is `rgba8`)."
  field bytes f64 "Bytes its frames take on the GPU (a stream: its texture's, one frame)." {
    unit bytes
  }
  field streamed bool "Whether it is a stream (`stream true`: decoded as it plays) rather than a clip of cooked frames."
  field codec String "A stream's codec (`av1`); empty for a clip."
  field poster Texture "A stream's first frame as an ordinary texture, ready as soon as the bundle is (a menu's thumbnail, the picture while the stream is fetched); for a clip, VideoInfo.texture."
}
```

<sub>`api.ogt` line 27561</sub>

<a id="type-VideoCapabilities"></a>

### `VideoCapabilities` (struct)

How this host plays streams. Backend limits are reported here, never by missing members: every host loads and plays a stream through the same API.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `streamed` | `boolean` | `false` | — | — | — | Whether streams decode here: false only on a host without a renderer (a headless run without a GPU), where a stream's texture shows nothing and its sound plays as usual. |
| `decoder` | `string` | `""` | — | — | — | The AV1 decoder: `rav1d` (native, a thread per stream), `webcodecs` (the browser's VideoDecoder), `rav1d-wasm` (rav1d in a Worker of its own, where the browser has no AV1 decoder) or `none`. |
| `hardware` | `boolean` | `false` | — | — | — | Whether the browser reports its decoder as hardware-accelerated (WebCodecs' `prefer-hardware` is supported); false for rav1d. |
| `maxWidth` | `number` | `0` | — | — | — | The widest stream this decoder plays in real time at 30 fps (wider ones still play, with frames held where decoding falls behind): 7680 for rav1d natively and WebCodecs, 1280 (by 720, or 720 by 1280 for a portrait stream) for rav1d-wasm, which plays a larger stream's 720p rendition, cooked beside it, instead (measured; docs/design.md §2.9). |
| `maxHeight` | `number` | `0` | — | — | — | The tallest, as maxWidth. |

```ogt
struct VideoCapabilities {
  doc "How this host plays streams. Backend limits are reported here, never by missing members: every host loads and plays a stream through the same API."
  field streamed bool "Whether streams decode here: false only on a host without a renderer (a headless run without a GPU), where a stream's texture shows nothing and its sound plays as usual."
  field decoder String "The AV1 decoder: `rav1d` (native, a thread per stream), `webcodecs` (the browser's VideoDecoder), `rav1d-wasm` (rav1d in a Worker of its own, where the browser has no AV1 decoder) or `none`."
  field hardware bool "Whether the browser reports its decoder as hardware-accelerated (WebCodecs' `prefer-hardware` is supported); false for rav1d."
  field maxWidth u32 "The widest stream this decoder plays in real time at 30 fps (wider ones still play, with frames held where decoding falls behind): 7680 for rav1d natively and WebCodecs, 1280 (by 720, or 720 by 1280 for a portrait stream) for rav1d-wasm, which plays a larger stream's 720p rendition, cooked beside it, instead (measured; docs/design.md §2.9)."
  field maxHeight u32 "The tallest, as maxWidth."
}
```

<sub>`api.ogt` line 27582</sub>

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

<sub>`api.ogt` line 27554</sub>
