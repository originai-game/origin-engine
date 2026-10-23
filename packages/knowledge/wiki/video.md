<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `video` module

Footage in a game (docs/design.md §2.9): a `type video` asset cooked by ffmpeg into a texture array of its frames and its sound, loaded with og.load.video and played by og.VideoPlayer. The frame shown is a Texture (VideoInfo.texture, also VideoPlayer.texture) any material map, og.Sprite or UI image draws, and the sound plays on the entity's og.AudioSource in step with it.

Playback is simulation: the og.video system (after the game's systems, before og.audio) advances VideoPlayer.time by the tick's dt times `speed` (not the time scale, so the picture stays with its sound), so the frame at a tick is the same on both hosts, in every run and after a snapshot is restored. With a 30 Hz tick a player started at 0 shows the source's frame 3 s in at tick 90.

- **TypeScript**: functions are none at module level (its members live on the types below); components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-game`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module video {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 13360</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`Video`](#type-Video) | handle | Cooked footage: its frames as a texture array, its sound (og.load.video). |
| [`Video()`](#column-Video) | column kind | A Video handle field (0 is none). |
| [`VideoInfo`](#type-VideoInfo) | struct | What og.load.video loaded. |
| [`og.VideoPlayer`](#record-VideoPlayer) | component | Plays a loaded video: VideoInfo.texture shows its current frame (frame n is the source at n / fps seconds), and when the entity has an og.AudioSource the og.video system sets its sound, playing, pitch and loop each tick so the sound plays in step: a seek or a restart starts the sound again at the new time on the next tick (AudioSource.startOffset). Each player has a texture of its own (written to `texture`): the first player of a video takes VideoInfo.texture, each further one a texture the og.video system makes for it over the same decoded frames (one frame of GPU memory each), so two players at different times show different frames. Show a player's `texture`, not VideoInfo.texture, once a video has more than one player. |

## Components, resources and events

<a id="record-VideoPlayer"></a>

### `og.VideoPlayer` (component)

Plays a loaded video: VideoInfo.texture shows its current frame (frame n is the source at n / fps seconds), and when the entity has an og.AudioSource the og.video system sets its sound, playing, pitch and loop each tick so the sound plays in step: a seek or a restart starts the sound again at the new time on the next tick (AudioSource.startOffset). Each player has a texture of its own (written to `texture`): the first player of a video takes VideoInfo.texture, each further one a texture the og.video system makes for it over the same decoded frames (one frame of GPU memory each), so two players at different times show different frames. Show a player's `texture`, not VideoInfo.texture, once a video has more than one player.

Token `og.VideoPlayer`, registered under the schema name `og.VideoPlayer`; its value type is `VideoPlayer`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `video` | `Video` | `0` | — | — | — | The video (og.load.video's `video`); 0 plays nothing. |
| `playing` | `boolean` | `true` | — | — | — | Whether time advances. |
| `time` | `number` | `0` | ≥ 0 | `s` | — | Seconds into the video; written every tick, set it to seek. |
| `speed` | `number` | `1` | 0 … 4 | — | tunable | The playback rate: 1 as cooked, 0.5 half speed (the sound's pitch follows). |
| `loop` | `boolean` | `false` | — | — | — | Start again at 0 past the end; otherwise hold the last frame and set finished. |
| `frame` | `number` | `0` | — | — | read only | Written: the frame shown, floor(time * fps). |
| `finished` | `boolean` | `false` | — | — | read only | Written: a player without loop reached the end. |
| `texture` | `Texture` | `0` | — | — | read only | Written: this player's texture (VideoInfo.texture for the video's first player, a texture of its own for each further one), for a material, sprite or UI image set up from the player. It keeps it while it lives. |

```ogt
component VideoPlayer {
  doc "Plays a loaded video: VideoInfo.texture shows its current frame (frame n is the source at n / fps seconds), and when the entity has an og.AudioSource the og.video system sets its sound, playing, pitch and loop each tick so the sound plays in step: a seek or a restart starts the sound again at the new time on the next tick (AudioSource.startOffset). Each player has a texture of its own (written to `texture`): the first player of a video takes VideoInfo.texture, each further one a texture the og.video system makes for it over the same decoded frames (one frame of GPU memory each), so two players at different times show different frames. Show a player's `texture`, not VideoInfo.texture, once a video has more than one player."
  field video Video "The video (og.load.video's `video`); 0 plays nothing."
  field playing bool "Whether time advances." {
    default true
  }
  field time f32 "Seconds into the video; written every tick, set it to seek." {
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

<sub>`api.ogt` line 13392</sub>

## Types

<a id="type-Video"></a>

### `Video` (handle)

Cooked footage: its frames as a texture array, its sound (og.load.video).

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Video "Cooked footage: its frames as a texture array, its sound (og.load.video)."
```

<sub>`api.ogt` line 13365</sub>

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
| `format` | `string` | `""` | — | — | — | The frames' encoding on the GPU (`rgba8`, `bc7`, `astc4x4`, `etc2`). |
| `bytes` | `number` | `0` | — | `bytes` | — | Bytes its frames take on the GPU. |

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
  field format String "The frames' encoding on the GPU (`rgba8`, `bc7`, `astc4x4`, `etc2`)."
  field bytes f64 "Bytes its frames take on the GPU." {
    unit bytes
  }
}
```

<sub>`api.ogt` line 13374</sub>

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

<sub>`api.ogt` line 13367</sub>
