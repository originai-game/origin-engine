<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`video`](../video.md) · [index](../index.md)

<a id="record-VideoPlayer"></a>

### `og.VideoPlayer` (component)

Plays a loaded video: VideoInfo.texture shows its current frame (frame n is the source at n / fps seconds; a stream's once decoded), and when the entity has an og.AudioSource the og.video system sets its sound, playing, pitch, loop and loop region (the picture's length) each tick so the sound plays in step, however long or often it loops: a seek or a restart starts the sound again at the new time on the next tick (AudioSource.startOffset). Each player has a texture of its own (written to `texture`): the first player of a video takes VideoInfo.texture, each further one a texture the og.video system makes for it over the same decoded frames (one frame of GPU memory each), so two players at different times show different frames. Show a player's `texture`, not VideoInfo.texture, once a video has more than one player.

Also known as: movie, video, cinematic, FMV, intro video, Unity VideoPlayer, Godot VideoStreamPlayer

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
  aliases "movie, video, cinematic, FMV, intro video, Unity VideoPlayer, Godot VideoStreamPlayer"
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

<sub>`api.ogt` line 25654</sub>
