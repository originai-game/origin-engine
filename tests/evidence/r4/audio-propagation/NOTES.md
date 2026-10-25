# r4/audio-propagation

Closes `crates/og-audio/README.md` §"What phase 2 owes" as engine features
in og-audio, with the API in `crates/og-api/api.ogt` (module `audio`) and
the generated `.d.ts` and wiki from `og api gen` (`og api check` clean).
Base: `42206f80` (`origin/r4/candidate`). One commit per feature group.

Measurements here are the dev profile (og-audio at opt-level 0) on this
orb (x86_64, 16 vCPU), as the working rules ask: no release or wasm
build. They are ratios against the same scene with the feature off more
than real-time costs; the release numbers are the wave's end's.

## 1. Propagation: Doppler, cones, HRTF

What changed:

- `og.AudioSource` gains `dopplerFactor` (1), `coneInnerAngle` (360),
  `coneOuterAngle` (360), `coneOuterGain` (0); `og.AudioListener` gains
  `dopplerFactor` (1) and `panningModel` (new enum `PanningModel`:
  `equalpower` default, `hrtf`); `og.Audio` gains `speedOfSound` (343.3).
  Engine-owned `og.AudioPlayback` gains `position` and `direction`,
  `og.AudioStream` gains `listener` and `listenerAt`: the state Doppler
  and the HRTF read is world state, in snapshots and the world hash.
- Doppler (OpenAL 1.1's formula) and the cone gain (Web Audio's
  PannerNode) are computed tick-side into the pitch and gain the stream
  already carries; the angle uses og-core's deterministic `atan2`.
  Velocities are tick velocities of world positions (down `og.Parent`,
  row 86's `Places::world`); a voice's first tick has none; a jump at or
  past the speed of sound in a tick is a teleport (kestrel-stair's `warp`
  commands must not shriek); `DF·v` is held within ±c/2.
- The HRTF is Brown and Duda's spherical-head model (ITD by Woodworth,
  the head-shadow one-pole/one-zero filter by the bilinear transform;
  `dsp.rs` `Hrtf`), per voice in the mixer, steered by a new command op 8
  `hrtf` (direction in the listener's frame in `gain`/`pitch`/`pan`; zero
  returns to the panner). The command record's layout is unchanged.

Tests (all fail to compile on the base, which lacks the members; each
was also run against the change with its feature's line reverted, and
failed as listed):

| test | reverted | failure |
|---|---|---|
| `propagation.rs` `doppler_shifts_by_the_motion_along_the_line_between_source_and_listener` | Doppler ratio not applied | `tick 10 (z -36.67): pitch 1, OpenAL's model gives 1.0617645549892292` |
| `propagation.rs` `a_cone_turns_a_source_down_away_from_its_front` | cone gain not applied | `45°: gain 1, want 0.8` |
| `propagation.rs` `the_hrtf_delays_and_shadows_the_far_ear` | mixer skips the HRTF | `the left ear lags by 0 frames (Woodworth: 31.5)` |
| `propagation.rs` `a_walk_with_doppler_cones_and_the_hrtf_is_the_same_twice_and_across_a_restore` | the listener's last position kept in a thread-local instead of `og.AudioStream` | `restored at tick 117, the walk goes on as it did` |
| `parented.rs` `a_source_on_a_child_follows_its_moving_parent` (extended: pitch bits, and the shift) | Doppler ratio not applied | `Doppler down og.Parent: [1.0, 1.0, …]` (the old code's pitch is 1 every tick) |

Byte-for-byte: a scratch scene (six static sources, four spatial,
assorted pitches, a turned listener, `equalpower`) and a second
unpitched one rendered the same PCM hash on the base and on this commit
(`2abf84ecfcd5ac01`, `025b6b818e77631c`); the scratch file was deleted.

Cost (`cargo run -p og-audio --example feature_costs`, 24 voices, 16 of
them spatial and circling the listener; mixer ms per second of audio):

| variant | mixer ms/s | over base |
|---|---:|---:|
| base (every feature off) | 73.6 | 1.000 |
| Doppler on all 16 | 76.4 | 1.04 (noise: the tick sets pitch, the mixer's work is the same) |
| cones on all 16 | 73.8 | 1.00 (tick side only) |
| HRTF listener (16 voices on it) | 131.2 | 1.78 (≈3.6 ms/s per HRTF voice at opt-level 0) |

The tick-side cost of Doppler and cones did not show above the noise of
the whole world step (288–305 µs a tick in every variant). Off costs
nothing: `one_second_of_a_busy_mix_renders_in_bounded_time` (32 voices)
took 288 and 295 ms after the change against 292 ms on the base.

## 2. Windowed sinc, virtual voices, retrigger

What changed:

- `og.Audio.resampler` (new enum `AudioResampler`: `linear`, `sinc`;
  default `sinc`) reaches the mixer as an fx command (`fx::RESAMPLER`,
  22) with the other fx state, so it is in the stream, the world hash and
  every resync. `resample.rs`: a Kaiser-windowed sinc, β 7, 8 zero
  crossings (16 taps) at pitch ≤ 1 from 2048 normalised phases; widened
  by the step (rounded up to an eighth, ≤ 4×, ≤ 64 taps, 512-phase tables
  built on first use) when pitched up; a per-voice 128-frame history
  (mirrored, so the taps read one slice) filled across the loop seam as
  the linear path reads it; on an exact source frame the frame itself.
  `linear` is the old code path, untouched.
- `og.AudioSource.virtualize`: past the cap the voice is stopped in the
  mixer but keeps its `og.AudioPlayback` (new `virtual` flag) and its
  cursor runs on; back within the cap it is played at its cursor (a
  resumed play, fading in). A virtual one-shot ends on its own tick.
- `og.AudioSource.retrigger`: one-tick restart from the top, playing or
  not; the old voice ends with the new `SOUND_RETRIGGERED` (5); the
  engine clears the flag and sets `playing`.

Tests (`tests/voices.rs`; all fail to compile on the base; each also run
with its feature's line reverted):

| test | reverted | failure |
|---|---|---|
| `the_windowed_sinc_resamples_without_images_and_linear_is_kept` | mixer never uses the sinc | `sinc keeps the tone: -3.44 dB` |
| same | `resampler` not sent in the stream (so `linear` is not honoured) | `linear is kept as it was: tone 0.00 dB, image -74.0 dB` |
| `a_virtual_voice_resumes_where_it_would_have_been` | the resumed play starts at 0 | `B is played once, at tick 30, from where it would be` |
| same | `virtualize` ignored (culled) | `B is never culled: SoundEnded { …, reason: 4 }` |
| `retrigger_restarts_a_sound_without_a_level_change` | `retrigger` ignored | the old voice's `SoundEnded`/`SOUND_RETRIGGERED` assertion |
| `a_run_with_sinc_virtual_voices_and_retriggers_is_the_same_twice_and_across_a_restore` | the virtual flag kept in a thread-local, not `og.AudioPlayback` | `the same samples and stream twice` |

Measured in the test: 15 kHz from 44.1 kHz, linear −3.44 dB on the tone
and its image at −11.5 dB, sinc 0.00 dB and −74.0 dB; 6 kHz pitched
2.5×, the folded image −28.1 dB linear and −89.7 dB sinc; the resumed
voice's level 0.17343 against the swell's 0.17348 at its cursor.

`linear` keeps the old output bit for bit: a scratch scene (seven looping
voices, pitches 0.7–2.08, sounds at 48 kHz, stereo and 22.05 kHz, some
spatial, startAt inside ticks, a loop start) hashed `c92e89684e332016` on
the base and with `resampler: linear` on this commit; `f7f349331a1b7a61`
with the sinc. The scratch file was deleted.

Cost (`feature_costs`, same scene as §1, dev profile, mixer ms per second):

| variant | mixer ms/s | over base |
|---|---:|---:|
| base (linear, everything off) | 76.8 | 1.00 |
| sinc (16 voices pitched by Doppler, 8 on their own frames) | 229.2 | 2.98 |
| virtual (cap 12, every source `virtualize`: 12 heard) | 40.5 | 0.53 (the 12 virtual voices cost the mixer nothing) |
| everything on (Doppler, cones, HRTF, sinc) | 285.3 | 3.72 |

Tick side: 311–352 µs a world step in every variant (the virtual run's
ranking every tick is the top of that range). The 32-voice busy mix
(`one_second_of_a_busy_mix_renders_in_bounded_time`, every voice pitched,
so every one resamples) is 650–675 ms in debug with the sinc default
against 292 ms linear (bound 1000 ms; the release bound of 50 ms against
17.5 ms measured on an M4 Pro at linear is for the wave's end to
re-measure). Retrigger and `virtualize` off cost nothing in the mixer;
with `linear` the mixer runs the old loop plus one branch per frame.

## 3. Decoding off the engine Worker: left owed

It does not fit without changing the mixer's observable output, so it
stays in the README's §"What phase 2 owes" with the reason. The engine's
wasm is `wasm32-unknown-unknown` without atomics, so nothing shares the
bank across threads. A driven mixer (the Worker, null, capture) renders
each tick's PCM as a function of the stream alone, so a voice's QOA frame
must be decoded at the sample the voice reaches it. A decode on another
Worker that lands late would play silence there, as a missing chunk does,
and the PCM would depend on timing. A deterministic version needs a
decode-ahead the tick can bound (the playheads' next frames, as
`chunk_distances` bounds chunks), a synchronous fallback on a miss, and a
threaded wasm build. No code changed for it and no CHANGELOG line.

## For the coordinator

- References that move: no committed audio reference (WAV hash) exists
  outside `tests/evidence`, whose numbers re-record at the wave's end. Any
  run with pitched voices, or sounds not at the output's rate, renders
  other samples with the sinc default (`og run --audio` SHA-256s; the
  bun tests compare two runs, so they still pass). Set
  `og.Audio.resampler` to `linear` for the old samples. Unpitched 48 kHz
  sounds with static spatial sources and `equalpower` are bit for bit
  unchanged.
- World hashes move for every game with audio: the records' layouts grow
  (`og.AudioSource`, `og.AudioListener`, `og.Audio`, `og.AudioPlayback`,
  `og.AudioStream`), and every stream sends one more fx command
  (`RESAMPLER`) in its first tick, so `AudioStream.hash` changes. Replay
  tests compare two runs and are unaffected. A save taken before this
  restores with `migrate`. Moving spatial sources now get Doppler by
  default (`dopplerFactor` 1), so kestrel-stair's playtest of
  `og.AudioPlayback.pitch`, if any, would see it; its gains, pans and
  low-passes do not change.
- No reference frames: nothing here renders.
- Not run, per the rules: `scripts/check.sh`, `og test`, the bun suites,
  web-reference, wasm and release builds. Run here: `cargo test -p
  og-audio` (all 39 tests, 8 of them new or extended), `cargo test -p og-api --test modules`,
  `cargo check -p og-runtime`, `cargo fmt`, `cargo clippy -p og-audio
  --all-targets -D warnings`, `og api gen` then `og api check` (clean),
  the knowledge `validate` script.
