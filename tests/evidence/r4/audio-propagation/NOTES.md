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
