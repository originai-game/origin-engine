# Debt round 3: audio decoding off the engine Worker

Branch `debt/audio-decode`, from `58f5609e` (origin/main). Closes the
debt recorded in `crates/og-audio/README.md` §"What phase 2 owes" and
`tests/evidence/r4/audio-propagation/NOTES.md` §3. Worked in an orb
(Linux x64, no GPU): nothing here ran `scripts/check.sh`, `og test`,
web-reference, a reference capture, a wasm build or a release build.

## What was wrong

- A sound up to 6 s was decoded whole when it entered the bank
  (`SoundBank::insert_qoa`, called from `og.load.sound` on the engine
  thread, the engine Worker on the web).
- A longer sound was decoded QOA frame by QOA frame by each voice as the
  mixer reached it, in the pump after the frame: on the web the engine
  Worker, natively the engine thread for the null and capture sinks.

Round 4 left it because a decode on another Worker that lands late would
play silence and make the PCM depend on timing.

## What changed

```
            tick (engine thread / Worker)                   helper
  ┌──────────────────────────────────────────┐    ┌──────────────────────────┐
  │ og.load.sound → insert_qoa: index only,  │    │ native: the bank's       │
  │   queue the pins (short sound: all its   │───▶│   decode thread          │
  │   frames; long: its first)               │    │ web: audio-decoder.ts,   │
  │ og.audio, each tick (sink attached):     │    │   a blob: Worker fed     │
  │   plan the decode-ahead from             │───▶│   after each pump        │
  │   og.AudioPlayback (≤ 5 frames a voice,  │    └────────────┬─────────────┘
  │   0.25 s, across the wrap), drop the     │                 │ samples
  │   frames it no longer wants              │                 ▼
  │ pump → mixer → Reader: cache hit, or a   │◀──── per-sound frame cache
  │   miss decoded here (same samples)       │
  └──────────────────────────────────────────┘
```

- `og-audio` (`src/decode.rs`, `src/sound.rs`, `src/system.rs`): sounds
  hold their QOA bytes and a frame cache. The `og.audio` system plans
  the decode-ahead at the start of every tick from the world's playheads.
  The plan is a function of the world and writes nothing in it. A miss
  decodes the frame where it is read, with the same code as before, so
  the PCM, the command stream and `world_hash` never depend on the helper.
  New API (Rust only): `SoundBank::{start_decode_thread,
  take_decode_jobs, put_decoded, decode_queued, decode_stats}`,
  `DecodeJob`, `DecodeStats`, `DECODE_AHEAD_SECONDS`,
  `DECODE_AHEAD_FRAMES`.
- `og-runtime` (`modules.rs`): natively starts the bank's decode thread
  for every engine (once per bank: a reload keeps it).
- `og-host-web` (`audio.rs`): `og_web_audio_decode_jobs`,
  `og_web_audio_put_decoded`; `og_web_audio_stats` adds `decodedAhead`,
  `decodeMisses`, `decodeQueued`, `decodeInFlight`.
- `og-host-node` (`audio.rs`): the sink stats add `decodedAhead`,
  `decodeMisses`, `decodeQueued` (typed in `node/audio.ts`).
- `packages/engine/src/web/audio-decoder.ts` (new): `decodeQoa`, a
  TypeScript port of `qoa.rs`'s decoder in integer arithmetic, and
  `AudioDecoder`, a Worker started from a `blob:` URL of that function's
  own source, with at most 32 jobs in flight. `WorkerAudio` starts it on
  attach and feeds it after every pump. If the Worker cannot start, or
  fails (a CSP without `worker-src blob:` makes Chrome fire an error
  event), nothing is fed to it and the engine decodes as it mixes, as
  before.
- No game API change: `og api check` is clean and no IDL or generated
  file moved. `ProbeMemory.audio` (the bank's resident bytes) now counts
  a QOA sound's file plus its decoded frames.

Why not threaded wasm: the decoder Worker shares no memory with the
engine, so the same path works on isolated pages (the lanes wasm) and
non-isolated ones (the single-threaded wasm). Neither the build nor
og-lanes changed. The cost is a second QOA decoder in TypeScript, pinned
to the Rust one by a shared fixture (below).

Why not "a late decode delays a start by a known rule": nothing about a
start depends on the data. The tick times a sound by its rate and length
alone, as it does for chunks, and a frame the helper has not answered is
decoded by the mixer, so the samples are the same on every machine. Only
where the work ran differs, and the stats count it.

## Evidence

All in the orb, dev profile, one cargo command at a time, into the
checkout's `target/`.

**og-audio, every test** (`cargo test -p og-audio`): audio 9, chunks 5,
decode 5 (new), mix 13, parented 3, propagation 4, voices 4, doc 1, all
pass. Among them, `tests/chunks.rs` still
mixes the chunked 190 s track sample-identically to the whole file and
stays within its resident bound.

**New tests, each broken once** (`cargo test -p og-audio --test decode`):

| break | fails |
|---|---|
| a miss plays silence (a naive asynchronous decoder) | `the_mix_is_the_same_whatever_the_helper_did`, `answers_of_another_length_or_key_are_refused` |
| the plan stops at a loop's end | `the_decode_ahead_is_bounded_and_crosses_the_wrap`, `the_mix_is_the_same…` (misses > 1 with a helper that keeps up) |
| an answer of another length is taken | `answers_of_another_length_or_key_are_refused` |
| insert decodes its pins on the inserting thread (the old behaviour) | `a_sound_is_indexed_at_insert_and_decoded_by_the_helper`, `the_mix_is_the_same…`, `answers_…` |

On the base commit the file does not compile: the API it drives does not
exist there.

**Bit for bit the base's mix.** A scratch test (deleted) rendered 15 s
at 48 kHz of a short sound retriggered at seven pitches, a 5.5 s 44.1 kHz
stereo loop at 0.77, a 14 s 22.05 kHz loop at 1.13 with a second voice
started 6 s into it, and a 30 s chunked sound fed chunk by chunk:
`PCM 1440000 samples, hash 28d3f934b504d4fc` with `crates/og-audio/src`
at `58f5609e`, the same on this branch, and the same three times with
the decode thread racing.

**The cost moved off the engine thread.** Scratch measurement (deleted),
dev profile, so the absolute numbers are a debug build's:

| | before (no helper, the old path) | with the decode thread |
|---|---:|---:|
| inserting 6 s of 48 kHz stereo | 63.5 ms (a whole `qoa::decode`) | 0.022 ms |
| 10 s of 16 streamed stereo voices (pitched 0.8–1.55) plus blips: the pumps' time on the engine thread | 4,251 ms, worst pump 55.3 ms | 2,393 ms, worst 10.9 ms |
| frames decoded on the engine thread / by the helper | 1,665 / 0 | 0 / 1,702 |

(The scene slept 2 ms between each tick and its pump, standing in for a
frame's render.)

**Hosts and packages.**

- `cargo check -p og-host-web --target wasm32-unknown-unknown`,
  `cargo check --target wasm32-unknown-unknown -p og-audio`,
  `cargo check -p og-host-node`.
- `cargo clippy -p og-audio -p og-host-node -p og-runtime --all-targets
  -- -D warnings` and `cargo clippy -p og-host-web --target
  wasm32-unknown-unknown -- -D warnings`: clean.
- `cargo fmt --check` on the four crates: clean.
- `cargo test -p og-runtime`: 78 passed, 0 failed.
- `cargo build -p og-cli`, then `target/debug/og api check`: "the
  generated files match crates/og-api/api.ogt."
- `bun x --package typescript@7.0.2 tsc -p
  packages/engine/tsconfig.runtime.json`: no diagnostics (both new files
  listed).
- `cd packages/engine && bun test tests/audio-decoder.test.ts`: 3 pass.
  The TS decoder gives the fixture's hash `080bf5513188bc60`, as og-audio's
  `the_shared_fixture_decodes_to_its_hash_frame_by_frame` asserts. It does
  so whole, as the jobs' one-frame files, and after the export's
  minifying `Bun.build`. `AudioDecoder` feeds a real Worker and answers a
  bank. Two breaks of the TS decoder each failed all three: the residual
  that straddles the slice's two words read from one word only, and `>>>`
  for `>>` on the prediction.
- Chrome for Testing 156 (agent-browser, scratch page, deleted): a module
  Worker standing in for the engine Worker loaded the minified bundle and
  started the decoder as a nested `blob:` Worker. It reported `alive=true
  posted=3 decoded=3 inFlight=0 hash=080bf5513188bc60`. Served with
  `Content-Security-Policy: worker-src 'self'`, it reported `alive=false
  posted=3 decoded=0 inFlight=3 fedAgain=0`: the decoder marks itself
  dead and nothing is fed again.

## Not verified here

- The engine's wasm with the new exports in a browser, both variants
  (single-threaded and lanes). No wasm was built in the orb, so the
  decoder Worker was checked against a stand-in glue only.
- The bun suites that need the native `og` and addon (`audio.test.ts`,
  `sound-chunks.test.ts`, `ui-sound.test.ts`, `og-run.test.ts`): their
  WAVs are compared run against run, and the PCM is bit for bit the
  base's, so they should pass unchanged.
- Release numbers, the wasm size (decode.rs and two exports; the
  Worker's bundle grows by the decoder, about 3 KB minified) and frame
  times.
- The `device` sink (cpal, `native-audio`): it reads the same cache from
  its callback; not built here.

## For the Mac lane

```sh
scripts/check.sh functional
cd packages/engine && bun test tests/audio-decoder.test.ts tests/audio.test.ts \
  tests/sound-chunks.test.ts tests/ui-sound.test.ts && cd ../..
cargo test --release -p og-audio --test audio one_second   # the 50 ms bound
scripts/wave-end.sh                                        # size gate, determinism, browser tiers
```

Then, in Chrome, open an exported game with music (the idle archetype)
on an isolated page and on a non-isolated one. After the first gesture,
DevTools' Threads list shows a `blob:` Worker under the engine Worker,
and a Performance recording shows `decodeQoa` running there and no QOA
decode inside the engine Worker's frames. `og_web_audio_stats(<engine>)`,
evaluated in the engine Worker, reports `decodedAhead` growing and
`decodeMisses` at a handful (resynced and offset starts). The music
should sound the same as on main.

## References that move

None. The PCM is bit for bit the base's, the command stream and world
hashes are unchanged (the plan reads the world and writes nothing), and
nothing renders differently.
