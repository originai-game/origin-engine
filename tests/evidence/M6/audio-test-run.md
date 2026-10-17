# og-audio M6 test run (55ac8a2 content, at 49ed935)

`CARGO_TARGET_DIR=target/agent-qa cargo test -p og-audio -- --nocapture` then `cargo test --release -p og-audio --test audio -- --nocapture` on Apple M4 Pro, 2026-10-18 (og-audio sources unmodified at the time of the run). Both profiles: 9/9 tests in `tests/audio.rs` pass, plus the README doctest in debug.

```
test a_one_shot_ends_on_its_tick ... ok
test pitch_two_halves_the_duration ... ok
test a_late_sink_hears_the_playing_loop_and_a_despawn_silences_it ... ok
test a_spatial_source_moving_left_to_right_pans_left_to_right ... ok
test a_loop_crosses_its_seam_without_a_discontinuity ... ok
test the_limiter_keeps_summed_peaks_at_or_below_one ... ok
test the_scripted_scene_renders_to_a_wav ... ok
test command_stream_is_deterministic_and_in_the_world_hash ... ok
test one_second_of_a_busy_mix_renders_in_bounded_time ... ok
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.88s
test crates/og-audio/src/lib.rs - (line 25) ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
```

## Determinism (audio-determinism.json)

The scripted scene (music loop, a spatial drone crossing, Rng-timed blips despawned on `SoundEnded`), 600 ticks: stream hash `c46c267313eac16f`, world hash `aa9e90881eab846d`; the second run, the run without a sink and the run restored from the tick-300 snapshot all end at `aa9e90881eab846d`; changing the music volume at tick 300 gives `a179877149a70194`. The filed JSON is the release run's; the debug run (re-run into a separate $OG_EVIDENCE_DIR) wrote the same stream, world and received hashes.

## Mix timing

32 voices (16 streamed stereo QOA), 1 s at 48 kHz in 128-frame quanta: **15.4 ms release** (1.5 % of real time, bound 50 ms) and 272.8 ms debug (bound 1000 ms); the README quotes 17.5 ms / 336 ms. Load average during the run was 15–25.

## The scene WAV

`the_scripted_scene_renders_to_a_wav` writes `$OG_EVIDENCE_DIR/M6/audio-scene.wav`: 2.5 s, 2 channels, 16-bit, 48 kHz, 120,000 frames, 480,044 bytes, sha256 `d5b493d96ee3484142318027a61f3e3d06806e4fda6f8623ca92897859bba38f`. It is not filed here because scripts/evidence.sh accepts only .png, .json and .md; re-running the test reproduces it (the command stream is deterministic, so the bytes should match the hash above on the same build).

`audio-scene-waveform.png` (read): left channel above, right below, min/max per column over 2.5 s. Both lanes carry the music's regular pulses from the first column; the drone crossing makes the left lane louder over the first half (peaking around 55 %) and the right lane louder over the second half, which is the left-to-right pan the test asserts; nothing reaches the lane edges (no clipping).
