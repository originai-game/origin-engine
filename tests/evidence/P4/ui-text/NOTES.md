# Phase 4, UI text round: evidence

Branch `ui-text` from main b62dd65, merged with main at 9b7f45c, 5dc3aa0
and 20a6d47; Apple M4 Pro, metal; vulkan-cpu in the og-lavapipe image
(linux/amd64, lavapipe) with og, the addon and og-ui's test binaries
cross-built by `cargo zigbuild --release --target
x86_64-unknown-linux-gnu.2.31`; 2026-10-20.

| File | What it shows |
|---|---|
| `break-and-restore.md` | each new behaviour broken once, the test that failed, restored |
| `kit-metal-moved.tsv`, `kit-vulkan-cpu-moved.tsv` | every kit frame against the frame before it: changed pixels, outliers (> 32/255), RMSE |
| `templates-metal-moved.tsv` | every template frame this round moved, captured with main's code and with the branch on this machine (two captures of one revision are byte-identical); the outliers' box locates the text |
| `kit-desktop-mainMenu-before-after.png` | top: before; bottom: the greeting and the title in the display face |
| `kit-desktop-racing-before-after.png` | the place numeral and lap count in the display face |
| `third-person-title-before-after.png` | "EMBER VALE" and the lines under it, kerned |
| `fx-showcase-hud-before-after.png` | the station HUD, kerned |
| `glyph-costs.json` | run-time glyph rendering, release: 16 CJK characters at 32 px/em, range 4 |

## Size

`scripts/check.sh`'s size gate (brotli -q 11 after wasm-opt):

| Module | Before (b62dd65) | After this round, before the merges |
|---|---:|---:|
| `og_host_web-publish.wasm` | 11,850,273 B, 3,205,718 B brotli | 11,996,014 B, 3,243,486 B brotli (+37,768, +1.2 %; 77.3 % of 4 MB) |
| `og_host_web-lanes.wasm` | 3,209,121 B brotli | 3,246,936 B brotli |
| the publish module built without `shaping` (`--no-default-features --features publish`, a one-off build in `target/wasm-noshaping`) | | 11,552,017 B, 3,113,221 B brotli: rustybuzz costs 130,265 B brotli |

After the merges the publish variants are per profile (main's
`publish,physics3d` and `publish,physics2d`); both keep `shaping`
(scripts/wasm-crates.txt). The gate summary after the last merge is in
the round's report.

## Frames

- og-ui: text-bidi, text-kerning, text-edit (new, metal and vulkan-cpu);
  sample-screen moved by its caret's blink (metal and vulkan-cpu).
- The UI kit: all 70 frames re-recorded on metal and vulkan-cpu, three new
  text frames; the fantasy sheets again after main re-recorded them.
  vulkan-cpu against metal: no outlier on any of the 73, RMSE at most
  0.0019.
- Templates (metal): the 49 frames listed in `templates-metal-moved.tsv`.
  Their vulkan-cpu (fx-showcase, match3, runner) and browser (fx-showcase,
  topdown, third-person's WebGL2 beacon) references were not re-recorded
  nor rerun: on metal the moves are at most 0.545 % outliers (third-person's
  title) and RMSE 0.036, so those tiers' own frames may now sit near or past
  the tolerance until they are recorded again.
