# B7 text stack verification

Branch `ui-text-stack`, macrunner, 2026-11-05–06. Initial migration through
`512c2a321`; follow-up `c48e22e14` (rebased as `6f28eec50` onto main
`3a273183c`) fixes rich discretionary hyphens and the cook/runtime contracts
found during landing. AccessKit remains platform ask 25.

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

The follow-up ran `cargo nextest run -p og-ui -p og-cook -p og-runtime
-E 'not test(gfx::)'`: **931 passed, 21 skipped** (GPU and ignored performance
tests). `cargo fmt --all --check` and clippy with `-D warnings` for og-ui,
og-cook, og-runtime and og-tests-cooked passed.

After rebasing onto `3a273183c`, the same scoped logic command passed
**947 tests, 21 skipped**. The four Metal comparisons passed again without
recording, each at RMSE 0 and 0% outliers. Scoped clippy and fmt passed again.
The rebase preserved both streams' append-only archive records and lessons;
no source conflict needed resolution. This is not a whole-workspace quick
or functional gate run.

All five landing regressions were reproduced before the fix and pass after it:

- `strings::an_arabic_font_carries_its_joined_forms_and_its_file`: layout-only
  SFNT tables are separate from runtime outlines. The test now checks that
  shaping data cannot enable runtime rasterization. Default GSUB glyph aliases
  are intentional; an assertion forbidding all private glyph keys was obsolete.
- `missing_glyphs::a_missing_glyph_is_named_once_on_the_tick_it_is_first_drawn`
  and `missing_glyphs::ui_text_warns_per_face_with_no_renderer`: retaining
  layout tables no longer expands the declared charset or suppresses warnings.
- `text_facing::a_floating_text_mixing_a_companion_draws_each_run_in_its_font`:
  font selection, ordering and facing pass. Expected companion glyph keys are
  derived independently from the source cmap, not from the runtime mapper.
- `template::cooking_twice_gives_identical_bytes`: cooked output is
  2,590,986 / 4,905,765 bytes (**52.82%**, formerly 107.1%). The 60% budget is
  unchanged. write-fonts emits the layout-only SFNT; no outlines or hints ship
  for fonts that did not already request runtime/shaped-script outlines.

`rich_soft_hyphens_reserve_the_styled_glyph_at_a_break` failed before the rich
SHY fix and passes after: narrow, wide and insufficient-hyphen-room widths,
colored spans, one intact keycap, and bounds. A second test checks Arabic
direction, Latin hyphen fallback, inherited ink and trailing empty paragraphs.

## Long-line measurement is not quadratic

`long_kerned_line_measurement_is_not_quadratic` measures the full `Ui::end`
path for 2,000 and 8,000 CJK characters. Negative kerning makes the whole line
fit even though isolated advances exceed the box: this exercises the old
growing-prefix measurement failure. Each sample checks one complete drawn
line and its bounds; no OpenType cache can make this atlas-only case a no-op.

Seven alternating samples per length, after warmup, dev profile:

| Characters | Median ms | Range ms |
|---|---:|---:|
| 2,000 | 65.584583 | 46.770084–75.085042 |
| 8,000 | 240.765125 | 230.952625–328.767583 |

Four times the input took **3.67106 times** the time, not quadratic 16× growth.
The machine was busy (12 cores, load average 16.78), so this is a scaling
regression check, not a release latency claim or a measured old/new speedup.
The test is ignored in routine runs; run it alone:

```sh
cargo test -p og-ui --test tests long_kerned_line_measurement_is_not_quadratic -- --ignored --nocapture --test-threads=1
```

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

The follow-up changes only `text-mixed-runs`: its lower-right examples show
a colored `extra-` at the narrow break and unbroken `extraordinary` at the
wide width, each with an E keycap. The actual Metal capture was inspected:
both are legible, nothing overlaps, and the multilingual rows and Name field
remain unobstructed. The other three reference files are byte-identical to
the previously approved B7 frames. No tolerance changed.

## Native, scalar, SIMD and lanes text hashes agree

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

The frozen follow-up source, and then the rebased `6f28eec50` source, returned
the same exact u64 in every release configuration: **16183596539576476136**.

| Configuration | Result |
|---|---|
| Native release, `OG_LANES=off` | 16183596539576476136 |
| Native release, `OG_LANES=4` | 16183596539576476136 |
| Scalar wasm release | 16183596539576476136 |
| SIMD128 wasm release | 16183596539576476136 |
| Lanes wasm release, physics3d | 16183596539576476136 |
| Lanes wasm release, physics2d | 16183596539576476136 |

Release builds use `--manifest-path tools/wasm-determinism/Cargo.toml` and
the lane's `target/`. Scalar uses empty `RUSTFLAGS`, SIMD uses
`-C target-feature=+simd128`. Lanes builds use `scripts/wasm-lanes.sh --locked
--manifest-path tools/wasm-determinism/Cargo.toml --release --lib
--no-default-features --features physics3d` (then physics2d), not private
atomics flags. All wasm outputs run through wasm-bindgen and `load.mjs
<wasm> <glue> --text-only`. The Node loader instantiates the real shared-memory
lanes module but does not start a browser Worker pool. Text layout itself is
single-threaded. These are text-layout comparisons, not whole-world or browser
worker determinism claims.

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
The owner accepted this increase and explicitly dropped size trimming.
ICU4X segmentation dictionaries/data contribute: a discarded word-dictionary
feature experiment measured 5,490,837 Brotli bytes (1,328,232 fewer than B7).
That is not a complete attribution of the growth. No trimming patch or
vendored Parley fork ships; all complex-script line dictionaries remain.

```sh
cargo build -p og-host-web --lib --profile wasm-gate --target wasm32-unknown-unknown --no-default-features --features publish,physics3d,shaping
wasm-bindgen --target web --out-dir target/evidence/ui-text-size --out-name origin-engine target/wasm32-unknown-unknown/wasm-gate/og_host_web.wasm
scripts/wasm-opt.sh target/evidence/ui-text-size/origin-engine_bg.wasm target/evidence/ui-text-size/og_host_web-publish.wasm
scripts/size-gate.sh target/evidence/ui-text-size/og_host_web-publish.wasm
```

The follow-up before rebase measured 21,213,164 optimized wasm bytes and
6,836,126 Brotli bytes, still within the gate. Relative to the same baseline,
that is +2,095,645 Brotli bytes. The owner-approved initial +2,078,588-byte
measurement above is retained rather than silently replaced.

The final rebased `6f28eec50` build measured **21,466,252 optimized wasm
bytes and 6,870,070 Brotli bytes**, passing the same 16,777,216-byte gate.
This combined build includes the other work already on main `3a273183c`;
its difference from the original baseline is not solely attributable to B7.
`target/evidence/size-gate.json` describes this final build. No size reduction
or budget change was applied.

## Platform boundary

B8 landed independently as `abf7df43c`, with ask numbering in `3a273183c`.
Origin Desktop still needs the OS adapter described in platform ask 25.
The engine never owns or attaches to the Desktop window. B7 does not add
an OS adapter or change the web shell's DOM mirror.
