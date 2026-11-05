# SSR ordering fix: Windows and browser validation

Renderer revision: `144f9b19cc8c5bf46084ba3aa3d44e33739abbe3` (`sssr`).
Runner: studio-win, Windows, NVIDIA RTX 5060. Chrome 156.
Thread: https://ampcode.com/threads/T-01a1ab9d-3662-750e-b906-7429d51e0ebd

The half-resolution upsample now writes a separate texture, then copies
into the lit pass's read-only storage texture. This makes the DX12
COPY_DST/UAV transition order the write and read. No shader, tolerance,
or reference changed in this validation follow-up.

## Browser coverage repaired

`ssr-wet-floor` previously appeared only in og-tests-cooked's native
registry, and its builder and procedural scene module were excluded from
wasm. The requested browser run would reject the scene name. It now lives
in the common registry, using the exact same scene and 16 frames as the
native test. The cooked registry no longer lists it a second time.

## Fresh validation

The normal Chrome and Firefox sessions were closed with the owner's
permission, as was Edge's background browser process. Two abandoned
Android diagnostic processes were stopped. No browser tier or GPU test
ran alongside the wasm build. GPU runs acquired the machine's exclusive
GPU lock from `scripts/lock.sh`, once per run.

- `scripts/check.sh --only wasm-build wasm-opt`: passed, 2/2 steps;
  wasm-build 536 s, wasm-opt 696 s, total 1235 s.
- Native og/host addon rebuilt and the engine package rebuilt with
  `OG_GATE=1` and the pinned motion model.
- `cargo build --profile wasm --target wasm32-unknown-unknown
  --target-dir target/wasm-gfx -p og-tests-web --lib`: passed with the
  shared SSR fixture.
- Chrome `reference-gfx.ts --scene
  ssr-wet-floor,weather-street-wet,weather-puddles`: 3/3 pass against Metal.
  Full results: `chrome.json`.

| Scene | Frames | Outliers | RGB RMSE |
| --- | ---: | ---: | ---: |
| ssr-wet-floor | 16 | 0.000% | 0.00044 |
| weather-street-wet | 16 | 0.001% | 0.00151 |
| weather-puddles | 16 | 0.000% | 0.00342 |

All three captures were visually inspected. The floor reflects the red
sphere and pale panel without torn blocks; the wet street's lamps and
authored red lacquered box remain intact; the rocks and puddle reflections
are intact. These stills do not establish temporal determinism. The JSON's
render times include initial compilation and are not steady-state timings.

- DX12 nextest filter
  `test(the_high_preset_refines_ao_reflections_and_fog_and_medium_moves_nothing)`:
  1/1 passes in 44.056 s (`dx12-quality.log`).
- `rustfmt --edition 2024 --check --config skip_children=true` on the
  three changed registry/module files, and `git diff --check`: pass.

## Inherited native evidence, not rerun here

`inherited-native-reruns.log` is preserved from the previous Windows
compatibility thread's `/e/src/og-verify/compat/ssr-fix.log`, at the same
renderer revision. It reports five byte-identical reruns each for half
and full resolution on DX12 and Vulkan, and 3/3 high-preset passes on each
backend. Before the fix, the previous thread reported roughly 8,000
different pixels between repeated DX12 half-resolution frames.

## Archive limitation

The showcase upload was attempted but this runner has neither
`OG_SHOWCASE_S3_ACCESS_KEY` nor `OG_SHOWCASE_S3_SECRET_KEY`. The inspected
PNGs remain under `.amp/in/artifacts/ssr-web/webgpu/`; no showcase upload
or manifest entry is claimed.
