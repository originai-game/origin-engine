---
name: og-audio
description: Sound on Origin Engine — one-shot voices for game events, the procedural sound presets og cook synthesises (coin, jump, hit, click and more), UI sounds, the four buses and their effects (filters, reverb send, compressor or limiter, ducking), music with stems, bar-aligned starts, crossfades and stingers on og.Music's clock, spatial sources with occlusion, distance dulling, Doppler, cones and HRTF for headphones, voice priority and the cap, and proving it all headless with listen and og run --audio. Use when a game is silent, when adding any sound or music, or when the mix is muddy, clipping or wrong.
---

Audio on Origin Engine is world state: an entity with `og.AudioSource`
plays while `playing` is true, the tick sends commands to the mixer, and
the same script renders the same samples on every host. The API is in
`reference/audio.md`; this skill is how much of each to use.

## What works

Two games written from this package alone (`examples/lantern-vale`,
`examples/hollowbell`) put this skill to work: Lantern Vale's four-minute
track, cooked in 4-second chunks, streams from the title screen under the
effects' duck, Hollowbell's music streams the same way, flipbook frame
events put Hollowbell's footsteps, strikes and bell on their exact frames,
and `og polish` heard a sound on every damage moment. In The Kestrel Stair
(`examples/kestrel-stair`, a 3D third-person game) spatial sources (the
braziers, the imps' chitter, the crypt's drip, the bell) were heard from a
listener on the camera with distance, pan, distance dulling and occlusion
behind walls, each read back exactly through `og.AudioPlayback` with
`get:`; its 2:40 track streamed in 4-second
chunks on the music bus. Its crypt is an `og.ReverbZone` (a box with a
1.5 m `fade`, the sfx bus sending 0.55 to a room of 0.92 at damping 0.25;
§4, §6): a playtest read the zone's `weight` 0 from the door and 1 inside,
while og.Audio and the bus kept the open air's room 0.5 and send 0.12.

## 1. Every event sounds

A game with no sound feels broken, whatever it shows. Before anything else,
give each of these a sound: the player's actions (jump, attack, dash), hits
given and taken, pickups, a door, a death, a win and a loss, every UI press.

- One voice entity per sound, spawned in `start` with `playing: false`, and
  a `Voices` system that plays each requested one: systems set bits of a
  game resource (`Sounds.sfx |= Sound.Jump`), Voices sets `retrigger: true`
  on the matching voice (it starts again from the top in that tick,
  playing or not, and the engine clears it; archetypes written before
  `retrigger` set `playing`, stopping a playing voice one tick and
  starting it the next). Every archetype in `templates/archetypes` does
  this; copy a `Voices`.
- Vary repeated sounds: `pitch` 0.92–1.08 from the world's `og.Rng` for
  footsteps, hits and coins, so a stream of them does not machine-gun.
- A hit's sound starts on the hit's tick, with its hit stop and flash
  (og-feel §3): the three together sell the contact.

## 2. Presets: sounds without files

`og sfx` lists the procedural presets (`coin`, `pickup`, `powerup`, `jump`,
`land`, `footstep`, `hit`, `hurt`, `explosion`, `laser`, `whoosh`, `door`,
`death`, `click`, `select`, `back`, `error`, `levelup`). `og sfx jump --out
assets/audio/jump.ogt` writes an asset `og cook` synthesises into a normal
sound; `og.load.sound(world, "audio/jump.ogt")` plays it like a WAV. Tune
`seed` (another take), `pitch`, `length` and `volume` in the file; `og sfx
jump --out jump.wav` renders it to listen to. `og sfx library <dir>` writes
all of them. Replace a preset with a recorded or generated sound when the
game's style needs it; presets are a floor, not a style.

## 3. UI sounds

`og.ui.configure(world, { sounds: { hover, press, open, close, success, error } })`
(or the style's `sound` lines) plays them on the `ui` bus: `click` for
press, `select` for success, `back` for close, `error` for a refused press
(and for `og.ui.shake({ sound: true })`, og-ui §6). Each starts in the
tick of its input. `hover` also answers keyboard and gamepad focus moving
to a control, and `press` a press by Enter, Space or the pad's A, so a
menu walked with a pad is heard step by step; keep `hover` short and quiet
(volume ≤ 0.3). `configure` changes only the fields it is given: setting
the sounds after a style (or a look that brings one) keeps the style and its
fonts, and `sounds: null` silences them again.

## 4. Buses and the mix

Four buses: `master`, `music`, `sfx`, `ui`. Set levels with the sources'
`volume` and each bus's `og.AudioBusFx` (one entity per bus):

| Need | AudioBusFx |
|---|---|
| Music under effects | on music: `duck: true`, `duckBus` sfx, `duckDepth` 8–12 (dB), `duckRelease` 0.3–0.6 (s) |
| Dialogue or a stinger above everything | duck music and sfx by the ui or a dialogue bus |
| Muffled (underwater, pause menu) | `lowpass` 600–1200 Hz on sfx and music |
| Thin radio or phone | `highpass` 400 Hz plus `lowpass` 3 kHz |
| Space and room | `reverb` 0.1–0.3 on sfx (og.Audio `reverbRoom`, `reverbDamping`); keep music dry |
| A room that sounds like a room (a crypt, a cave, a hall) | an `og.ReverbZone` on an entity at the room's centre (§6), not code that eases the bus |
| Even out loud and quiet effects | compressor: `threshold` −18 (dB), `ratio` 3–4, `attack` 0.005 and `release` 0.15 (s), `makeup` 3 (dB) |
| Never clip | a limiter on master: `threshold` −1, `ratio` 20, `attack` 0.001 |

`og polish` reports clipped samples in the run's mix (og-build §5); a peak
at 1.0 is a mix bug, not loudness.

## 5. Music

- Loop a track with `AudioSource.loop` (`loop: true`, with `loopStart` and
  `loopEnd` for an intro that plays once) and `bus: "music"`.
- Stems: start every layer at the same `startAt` (world seconds) and bring
  layers in and out with `volume` and `fade` (0.5–2 s), never by starting
  them late: they stay sample-aligned.
- `og.Music` (`bpm`, `beatsPerBar`, `start`) is the clock: `nextBar` and
  `nextBeat` are world times, so `startAt: music.nextBar` starts a stinger
  or the next section on the bar line, sample-accurately, and `og.MusicBeat`
  events drive beat-synced visuals.
- Bar-length stems started bar by bar: on the last beat of each bar
  `og.MusicBeat` brings, start the next bar's stems at that event's `time`
  plus a beat. og.audio runs on the world's ticks, which keep going at
  time scale 0, so a pause stops the stems and a resume starts them again.
  `templates/archetypes/rhythm` (its `Band` system, and
  `tests/band.og.ts`, which asserts every `startAt`) does this.
- Part-way starts: `startOffset` (seconds into the clip) with `startAt`
  starts a source that far in, to the sample, so a resume carries on from
  the beat it paused on: each stem sounding then starts at the count-in's
  end with `startOffset` how far into it that beat is (the rhythm's
  `tests/pause.og.ts` asserts both).
- What the player hears lags the mix by the device's output latency:
  `og.audio.outputLatency(world)` (seconds; the Web Audio context's
  `outputLatency` or `baseLatency` plus the engine's buffer on the web, the
  device's callback-to-playback time natively, 0 headless; the system
  declares `og.Audio` among its reads). A rhythm game judges a press at
  its tick's time plus its event's `at` (og-game, input) less this and a
  calibrated offset for the rest of the chain (the rhythm's `Play` system,
  `tests/timing.og.ts`).
- A crossfade: the new track `startAt: nextBar` with `fade: 2`, the old one
  `playing: false` with the same `fade`.
- Music and sounds can stay as the generator returned them: `og cook`
  reads `.wav`, `.mp3`, `.ogg` (Vorbis; not Opus) and `.flac` and decodes
  each to QOA itself, with no ffmpeg or other converter. The asset id keeps
  the extension: `og.load.sound(world, "music/theme.mp3")`.
- Long tracks stream in chunks: a sound whose cooked file passes 1 MiB
  (about 27 s of 48 kHz stereo) is cooked in chunks of whole frames, and
  the engine keeps only from half a second behind each playhead to four
  seconds ahead (on the web each chunk is fetched as it nears). Set it in
  the track's import settings:

  ```text
  # assets/music/theme.mp3.ogt
  type import
  version 1
  ---
  sound {
    stream true   # true, or false to keep a long track whole
    chunk 4       # seconds per chunk, 1 to 60
  }
  ```

  Nothing changes in code: loops, `startOffset`, crossfades and stems play
  sample-identical to the whole file, and the world hash is the same.
  `og.load.soundInfo(world, sound)` answers `chunks`, how many (0: whole).
  A chunk that has not arrived plays silence, so start a long track a beat
  before it must be heard rather than on the first frame of a scene.
  What a page downloads: a chunked sound in the first group costs the boot
  its first chunk only (its other chunks are fetched ahead of the
  playheads, none before it plays). In a streaming group (og-game §6) it
  costs the boot nothing: its first chunk comes in the group's pack, it
  plays silence of its own length until the group is resident, and only
  then are its other chunks read ahead of the playheads, each a file of
  its own, never in the pack; when the group unloads it is silent again
  and nothing more is read. Put a level's or a region's music in that
  level's group; keep the title's music in the first group, since a group
  wanted at boot loads right after it anyway.

## 6. Space, occlusion, priority

- `spatial: true` sources pan and attenuate by their world transform
  against the `og.AudioListener` (on the camera or the player);
  `refDistance` 1–3, `maxDistance` 20–50. A parented source follows its
  parent: put the footstep or the engine hum on a child entity
  (`SpawnDesc.parent`) and it is heard where the parent carries it; a
  listener on a child of the camera rig hears from the rig.
- `distanceCutoff` (e.g. 2000 Hz) dulls far sounds; `occlude: true` plays a
  source behind a collider at og.Audio's `occlusionGain` through
  `occlusionCutoff`: enemies behind walls sound behind walls.
- A room of its own: `og.ReverbZone` (a box `size` along the entity's
  axes, or `radius`) eases the shared reverb towards its `room`,
  `damping` and its `bus`'s `send` while the listener is inside, fading
  over `fade` metres past the walls (2–4 m reads as walking through a
  door). A stone crypt: `room` 0.9, `damping` 0.25, `send` 0.5 on sfx;
  a small wooden room: `room` 0.4, `damping` 0.6. og.Audio and the
  bus's og.AudioBusFx stay the open air's; read `weight` to know where
  the listener is. It follows the listener's world transform, so put the
  og.AudioListener where the ears are (the camera, or the player); a zone
  on a child of a moving room moves with it.
- Motion is heard: a spatial source or listener that moves shifts the
  pitch (Doppler, from tick velocities down `og.Parent`, so a car on a
  rig or a listener on a moving camera just works). `dopplerFactor` 1 is
  physical; 0.3–0.6 on a racing game's engines keeps a fly-by readable;
  0 on a source that must keep its pitch (a voice line, a musical
  instrument in the world) or on the `og.AudioListener` to turn it off
  everywhere. A warp or respawn (a jump faster than sound in one tick)
  shifts nothing. `og.Audio.speedOfSound` (343.3) is the scale.
- Directional sources: `coneInnerAngle`, `coneOuterAngle` (full angles,
  degrees, about the entity's +Z, as three.js's `setDirectionalCone`)
  and `coneOuterGain`: a loudspeaker or an NPC talking forward `90`,
  `240`, `0.3`; a siren on a vehicle `120`, `300`, `0.5`. Rotate the
  entity to aim it; the default 360/360 is omnidirectional.
- Headphones: `og.AudioListener.panningModel: "hrtf"` places spatial
  sources by interaural delay and head shadow (a spherical-head model),
  clearer left and right than the stereo panner, without front/back or
  height cues; offer it as a setting ("Headphones"), keep `equalpower`
  for speakers (the default). Non-spatial sources keep their `pan`.
- `og.Audio.maxVoices` (32) caps voices; past it the lowest `priority`,
  then the quietest, then the oldest is culled (`SOUND_CULLED`). Give the
  player's own sounds and the music a high priority (200+), ambient props a
  low one. Ambient loops and music stems that should keep their place
  when culled get `virtualize: true`: past the cap they go silent with
  their playhead running and come back where they would be, not from the
  top (`og.AudioPlayback.virtual` says which).
- `og.Audio.resampler` is `sinc` (clean highs at any pitch); `linear` is
  the older, cheaper mix, kept for a game that compares its exact output
  (a WAV hash in a test). Unpitched 48 kHz sounds play the same either way.

## 7. Prove it

- `listen[:file][:seconds]` in a command script writes the last seconds of
  the mix as a WAV; a playtest reads it with `run.wav(file)` (`peak`,
  `rms`): silence where it should be silent, sound after the event.
- `og run --audio out.wav --json` captures the whole run and prints its
  SHA-256: two runs of the same script give the same hash.
- `dump_entity:<voice>` shows `og.AudioPlayback` while it plays; a counter
  on the voice (`Sfx.plays`) proves a short sound played at all.
- `og polish` scores inputs answered by a sound, UI presses with a ui-bus
  sound and hits with a sound (og-build §5).
