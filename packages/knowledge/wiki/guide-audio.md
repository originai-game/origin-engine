<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `audio` guide: Audio

How a game makes sound: sources on entities, buses and their effects, spatial sound, music on a beat clock, mixer snapshots and reverb zones, the web's first-input unlock, and the voice budget. Sound is decided in the tick and rendered outside it: the engine's og.audio system turns og.AudioSource changes into a deterministic, time-stamped command stream that snapshots, restores and the world hash cover, and the host's mixer renders that stream to PCM, which never reaches the world.

The API: `og api show audio`; sounds load with og.load.sound.

<a id="guide-audio-1"></a>
### `audio §1` Sources, buses and the mix

A source is level-triggered: inserting it plays it, `playing: false` stops
it, and a one-shot that reaches its end sets `playing` back to false and
sends og.SoundEnded.

```ts
const hit = og.load.sound(world, "audio/hit.ogg").value!;
const e = world.spawn({ name: `hit-${world.tick}` }).value!;
world.insert(e, og.AudioSource, { sound: hit, bus: "sfx", volume: 0.8 });

// variations with jitter, a cooldown and a voice cap
world.insert(feet, og.AudioEvent, { sounds: steps, order: "shuffle", pitchJitter: 1, volumeJitter: 2, maxVoices: 2 });
world.set(feet, og.AudioEvent, { trigger: true });   // the engine resets it
```

- Buses: `master`, `music`, `sfx` and `ui`; every bus passes through
  master. og.Audio holds their gains, and a gain change ramps over 256
  frames in the mixer. Give the player a slider per bus.
- og.AudioBusFx puts insert effects on one bus: filters, a compressor,
  ducking by another bus (music under dialogue or effects), the shared
  reverb's send, an equaliser, a delay and a distortion. Without one a bus
  passes straight through.
- User buses 4 to 15 (og.AudioBusNode) build a graph under the built-in
  ones: a footsteps bus under sfx is turned down with sfx, because each
  bus's gain multiplies down the chain to master.
- og.AudioEvent is the way to play footsteps, impacts and gunshots: it
  picks among up to eight variations from the engine's seeded stream, so a
  replay plays the same sounds at the same pitches. Each play spawns a
  voice entity parented to the event's entity.
- og.audio.playFeedback is for presentation-only UI clicks: no entity, no
  event, no hash change, not replayed. Gameplay sound is an og.AudioSource.
- og.audio.createSound makes a sound from PCM the game computed. Make them
  in the same order on every launch (in GameDesc.start): the handle is the
  next the bank gives, and saves name handles.
- Systems read og.SoundStarted, og.SoundEnded and og.MusicBeat the tick
  after they are sent.

<a id="guide-audio-2"></a>
### `audio §2` Spatial audio

- Spatial gain and pan are computed in the tick from the source's world
  transform relative to the first og.AudioListener's, in three.js's
  convention: the listener looks down -Z, +X is right. With no listener it
  is the origin looking down -Z.
- A source, a listener or a zone on a child entity is placed at its
  og.Transform composed down og.Parent, as og.MatrixWorld is: a sound on a
  child is heard where its parent carries it, a listener under a camera rig
  hears from the rig. Put the listener on the camera rig in third person
  and on the head in first person.
- Distance follows Web Audio's inverse model (`refDistance`,
  `maxDistance`, `rolloffFactor`); `distanceCutoff` adds air absorption,
  and `occlude` muffles a source behind a collider (a physics ray to the
  listener, at og.Audio's occlusion gain and cutoff).
- Propagation is all in the tick, and so in the stream: a moving source or
  listener shifts the pitch (Doppler, from their tick velocities), a source
  with a cone is turned down away from its +Z (as three.js's
  setDirectionalCone), and a listener whose `panningModel` is `hrtf` places
  sources for headphones (interaural delay and head shadow) instead of the
  stereo panner. Offer hrtf as a headphones setting; it does not tell front
  from back.
- Doppler treats a move at or past the speed of sound within one tick as a
  teleport and shifts nothing, and holds the shift within a factor of 3.

| AudioSource | Value | Source |
|---|---|---|
| `refDistance` | 1 m | engine default (Web Audio PannerNode's too) |
| `maxDistance` | 100 m | engine default |
| `rolloffFactor` | 1 | engine default (Web Audio PannerNode's too) |
| `dopplerFactor` | 1: physical; 0 off | engine default |

<a id="guide-audio-3"></a>
### `audio §3` Music and beats

- A Music component is a beat clock: tempo, metre and where bar 1 starts.
  The music system writes the bar, the beat and the next beat and bar line every
  tick, and sends og.MusicBeat for every beat.
- Start stems and stingers on the clock: an AudioSource whose `startAt` is
  the Music's `nextBar` starts on the bar line to the sample, so stems
  started together stay in step and a stinger lands on the downbeat.
- A crossfade is one source stopped and another started with the same
  `fade`. A resumed phrase sets `startOffset` to continue instead of
  restarting.
- A rhythm game hears a note at its time plus og.audio.outputLatency:
  judge a press against the note's time plus the latency, then a
  calibrated rest (the display, the hand) the player sets in a calibration
  screen. On the web the latency is the AudioContext's baseLatency and
  outputLatency plus the worklet ring's buffer; natively the device's
  latency (cpal's playback less callback instant) plus the mixer's buffer;
  0 headless, where the null and capture sinks are driven to the tick.

<a id="guide-audio-4"></a>
### `audio §4` Snapshots and reverb zones

- og.AudioSnapshot is a mixer state to blend towards: combat (music down,
  sfx up), underwater (`{ lowpass: [0, 0, 600] }` muffles sfx), paused
  (`{ volume: [1, 0.3] }` turns the music down). Set `active`; the blend
  takes `fade` seconds each way and is a straight line to the sample.
  Several snapshots stack.
- Snapshots never write og.Audio or og.AudioBusFx: what the game set stays,
  and the snapshot multiplies it. Turn a snapshot off to go back.
- og.ReverbZone makes the shared reverb sound like a room while the
  listener is inside the og.Volume on its entity. Past the volume's surface
  it fades out over the volume's `fade`, so walking in through a door the
  room grows around the listener. A zone without an og.Volume has no
  bounds: it is heard everywhere, which is rarely what a room wants. A zone on
  a child of a moving room moves with it. Outside every zone the reverb is
  og.Audio's `reverbRoom` and `reverbDamping` and the bus's send.

| ReverbZone | `room` | `damping` | `send` |
|---|---|---|---|
| engine default | 0.85 | 0.3 | 0.5 |

<a id="guide-audio-5"></a>
### `audio §5` The web's first-input unlock

Browsers start no audio before the player interacts. The web shell creates
and resumes its AudioContext once, on the page's first pointer, key or
touch gesture, then opens the engine's mixer at the context's rate. Until
then the world runs and its command stream is world state as ever, but
nothing is heard.

- Open with a title screen or a "click to start" press, so the first sound
  that matters plays after the gesture.
- Never wait for the unlock in game logic: nothing in the world tells it,
  and a headless run never has one. A test hears what the capture sink
  renders.

<a id="guide-audio-6"></a>
### `audio §6` Voice budget

og.Audio's `maxVoices` (32 by default, at most 256) caps the sources that
play at once. Past it the lowest `priority` stops first, then the quietest,
then the oldest, with og.SoundEnded reason SOUND_CULLED; a source with
`virtualize` goes silent instead and is heard again from where it would be
once it stands within the cap.

| Field | Value | Source |
|---|---|---|
| og.Audio `maxVoices` | 32, at most 256 | engine default and limit |
| AudioSource `priority` | 128; higher is kept longer | engine default |

Raise `priority` on what must never drop (dialogue, the player's own
actions), give loops that should resume in place `virtualize`, and cap
rapid one-shots (footsteps, debris) with an og.AudioEvent's `maxVoices`
and `cooldown`.

- The mixer's sinc resampler (the default) costs more per voice than
  `linear`; keep it unless a profile shows the mixer, and switch to linear
  only for a game that relies on the old mix's exact output.
- An og.AudioEvent's `maxVoices` with `steal` keeps a fast trigger
  (machine-gun fire) from eating the global budget.

<a id="guide-audio-7"></a>
### `audio §7` Pitfalls

- Reading a voice's state for gameplay: only an og.AudioEvent's `played`,
  `last`, `index`, `bag` and `voices` and og.SoundStarted and og.SoundEnded feed back.
  Time gameplay by ticks, never by when a sound ends in the speaker.
- Starting the same one-shot again by toggling `playing` in one tick does
  nothing (it is level-triggered): set `retrigger`.
- Playing sounds from a system during a rollback re-simulation: the engine
  already plays each sound once (`networking §4`); a game's own
  presentation sounds check og.net.resimulating.
- A source on a despawned entity is silenced without an event.

<a id="guide-audio-8"></a>
### `audio §8` Video and its sound

Footage is a `type video` asset cooked by ffmpeg, loaded with
og.load.video and played by og.VideoPlayer; its sound plays on the entity's
og.AudioSource in step with the picture.

| Kind | Cooked to | Use |
|---|---|---|
| clip (the default) | a texture array of its frames, at most 256 | short loops: a screen in the world, an animated sign |
| stream (`stream true`) | AV1, decoded as it plays: rav1d on its own thread natively, WebCodecs on the web, rav1d in a Worker of its own where the browser has no AV1 decoder | any length: intros, cutscenes |

- The frame shown is a Texture any material map, og.Sprite or UI image
  draws. With several players of one video, show each player's own
  `texture`: the first player takes VideoInfo.texture, each further one a
  texture the og.video system makes for it over the same decoded frames
  (one frame of GPU memory each), so two players at different times show
  different frames.
- Playback is simulation, decoding is presentation. og.video (after the
  game's systems, before og.audio) advances `time` by the tick's dt times
  `speed`, not the time scale, so the picture stays with its sound; a
  paused game, whose ticks do not run, pauses both. With a 30 Hz tick a
  player started at 0 is at the source's frame 3 s in at tick 90, on both
  hosts, in every run and after a restore.
- With an og.AudioSource on the entity, og.video sets its sound, playing,
  pitch, loop and loop region (the picture's length) each tick, however
  long or often it loops; a seek or restart starts the sound again at the
  new time on the next tick (AudioSource.startOffset).
- A stream's texture shows its tick's frame once decoded. An executor
  (`og run`, `og render`, `og test`) waits for it, so every capture shows
  exactly its tick's frame; a game on the web keeps the last decoded frame
  until then, and nothing the decoder has or has not finished reaches the
  world.

| Decoder (og.video.capabilities) | Real-time size at 30 fps |
|---|---|
| rav1d natively, WebCodecs | 7680 wide |
| rav1d-wasm | 1280 by 720 (720 by 1280 portrait); it plays a larger stream's 720p rendition, cooked beside it |

  Wider streams still play, with frames held where decoding falls behind
  (measured; docs/design.md §2.9).
