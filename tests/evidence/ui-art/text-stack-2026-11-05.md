# B7 text stack verification

Branch `ui-text-stack`, code through `512c2a321`, macrunner, 2026-11-05.
This is a verified migration milestone, not final B7 acceptance. Styled
discretionary hyphens and long-line scaling evidence remain open.

## Logic and cooking

`cargo test -p og-ui --test tests -- --skip gfx:: --test-threads=1`
passed 298 tests, with one performance test ignored. The linebreak tests cover
tabs, NBSP, ZWSP, hyphens, slashes, grapheme-safe emergency breaks, Thai, Lao,
Khmer, Myanmar, and cooked-font discretionary hyphens. Existing RTL, CJK,
rich-text, editing, fallback, and accessibility tests passed.

The tab, ZWSP, hyphen, slash, combining-grapheme, and four dictionary assertions
were also run against the B8 baseline `898c46f0d`: all nine failed. NBSP and
soft-hyphen tests passed on the new code but were not run against that baseline.

`cargo clippy -p og-ui -p og-cook --all-targets -- -D warnings` passed.
The cook's feature tests and `latin_default_contextual_glyphs_are_cooked`
passed. The latter checks Inter's contextual numeric hyphen. The cook now
includes default GSUB closure and seeds it with digits as well as letters.

## Metal references

Under a GPU slot, the four text references were recorded, inspected, then
compared again without recording:

```sh
. scripts/lock.sh
og_gpu_acquire ui-text-verify
cargo test -p og-tests-cooked --test tests \
  text_gfx::text_frames_match_their_references -- --nocapture --test-threads=1
```

All four comparisons passed with RMSE 0 and 0% outliers. Tolerances did not
change. The renderer's missing-glyph assertion passed.

- `text-bidi`: Arabic joins, mixed directions, and digits are legible.
- `text-kerning`: kerned Latin pairs, ligatures, wrapping, and ellipses are legible.
- `text-edit`: selection, caret, accented text, and multiline text remain legible.
- `text-mixed-runs`: Latin digits, Arabic joins, Indic marks, CJK, and rich spans
  remain visible without clipping.

References are in `crates/og-ui/references/metal/`. Captures are in
`target/tmp/og-ui/`. No other references were recorded.

## Native and scalar wasm agree

Both debug builds returned `text_layout_hash 16183596539576476136`.
The test uses real Latin, Arabic, and Indic font bytes, asymmetric line widths,
glyph placement, and a rich-text keycap. The scalar wasm needs wasm-bindgen
glue because Fontique imports it even with system font discovery disabled.

Commands ran under the machine gate lock and the lane target lock:

```sh
cargo build --manifest-path tools/wasm-determinism/Cargo.toml --target-dir target --bin wasm-determinism
target/debug/wasm-determinism --text-only
cargo build --manifest-path tools/wasm-determinism/Cargo.toml --target-dir target --lib --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir target/evidence/ui-text-determinism --out-name determinism target/wasm32-unknown-unknown/debug/wasm_determinism.wasm
node tools/wasm-determinism/load.mjs target/evidence/ui-text-determinism/determinism_bg.wasm target/evidence/ui-text-determinism/determinism.js --text-only
```

Release, SIMD, and lanes cross-host comparisons were not run in this stream.

## 3D publish wasm stays under the gate

The same build, bindgen, optimizer, and compression recipe ran at the B8
baseline and at B7. Builds ran serially under the machine gate and target locks.

| Revision | Optimized wasm bytes | Brotli bytes |
|---|---:|---:|
| B8 baseline `898c46f0d` | 17,749,400 | 4,740,481 |
| B7 code through `512c2a321` | 21,198,489 | 6,819,069 |
| Change | +3,449,089 | +2,078,588 |

The Brotli budget is 16,777,216 bytes. The measured 3D publish module passes.
This is not a measurement of the 2D or lanes modules or the packaged release.

```sh
cargo build -p og-host-web --lib --profile wasm-gate --target wasm32-unknown-unknown --no-default-features --features publish,physics3d,shaping
wasm-bindgen --target web --out-dir target/evidence/ui-text-size --out-name origin-engine target/wasm32-unknown-unknown/wasm-gate/og_host_web.wasm
scripts/wasm-opt.sh target/evidence/ui-text-size/origin-engine_bg.wasm target/evidence/ui-text-size/og_host_web-publish.wasm
scripts/size-gate.sh target/evidence/ui-text-size/og_host_web-publish.wasm
```

The baseline ran last. The lane's current publish wasm and
`target/evidence/size-gate.json` therefore describe the baseline, not B7.

## Remaining acceptance work

- Parley 0.12 hides SHY without drawing a discretionary hyphen. Plain cooked
  labels reserve and draw it at taken breaks; rich styled spans still need
  the equivalent behavior, including inline boxes and RTL.
- The growing-prefix measurement loop is removed. A controlled scaling run
  and a regression check for its cost have not been completed.
- Run the remaining determinism variants at wave end.
- B8's engine mapping and action transport are on this branch. Origin Desktop
  still needs the OS adapter described in platform ask 25. The engine never
  owns or attaches to the Desktop window.
