<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-sound"></a>

### `og.load.sound`

Load a cooked sound for the audio module. Sounds are `.wav`, `.mp3`, `.ogg` (Vorbis) and `.flac` files under assets/ (and procedural `.ogt` presets): og cook decodes each to 16-bit PCM and encodes it to QOA, so every format plays, streams and loops the same. The asset id keeps the source's extension (`audio/theme.mp3`).

A long sound (music, ambience) can be cooked in chunks of whole QOA frames: `sound { stream true chunk 4 }` in its import settings, `<file>.ogt` (`theme.mp3.ogt`; chunk in seconds, 1 to 60, default 4), and every sound whose QOA file would pass 1 MiB unless `stream false`. Its first chunk loads with it and the others follow the playheads (SoundInfo.chunks), so music is never resident whole; it plays sample for sample as the whole file would, loops and crossfades included. A sound cooked in chunks cannot join a streaming group.

```ts
og.load.sound(world: World, path: string): Result<Sound>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the sound. |
| `path` | `string` | no | The sound's asset id, such as `audio/blip.wav`. |

**Returns** `Result<Sound>`.

Binding symbol `og_load_sound` · publish builds: kept

```ogt
fn sound {
  doc "Load a cooked sound for the audio module. Sounds are `.wav`, `.mp3`, `.ogg` (Vorbis) and `.flac` files under assets/ (and procedural `.ogt` presets): og cook decodes each to 16-bit PCM and encodes it to QOA, so every format plays, streams and loops the same. The asset id keeps the source's extension (`audio/theme.mp3`)."
  doc "A long sound (music, ambience) can be cooked in chunks of whole QOA frames: `sound { stream true chunk 4 }` in its import settings, `<file>.ogt` (`theme.mp3.ogt`; chunk in seconds, 1 to 60, default 4), and every sound whose QOA file would pass 1 MiB unless `stream false`. Its first chunk loads with it and the others follow the playheads (SoundInfo.chunks), so music is never resident whole; it plays sample for sample as the whole file would, loops and crossfades included. A sound cooked in chunks cannot join a streaming group."
  param world World "The world that owns the sound."
  param path String "The sound's asset id, such as `audio/blip.wav`."
  returns Result.Sound
}
```

<sub>`api.ogt` line 8128</sub>
