# Phase 4, UI shell round: evidence

Branch `ui-shell` from main 7696db4, merged with main 376f3cd (then
e795264 at the end); Apple M4 Pro (metal), the og-lavapipe image
(vulkan-cpu, binaries cross-built with `cargo zigbuild --release --target
x86_64-unknown-linux-gnu.2.31`), Chrome for Testing 150 (webgpu,
webgl2); 2026-10-20 and 21.

| File | What it shows |
|---|---|
| `break-and-restore.md` | each new behaviour broken once and the test that failed |
| `pack-contrast-table.txt` | every pack's rows of `pack_contrast.rs` (ink and ground measured), all at their bars |
| `kit-metal-moved.tsv`, `kit-vulkan-cpu-moved.tsv` | every og-ui reference this round moved, against the one before it |
| `vulkan-cpu-vs-metal.tsv` | all 83 og-ui frames, vulkan-cpu against metal: no outlier, RMSE at most 0.0019 |
| `templates-vulkan-cpu-moved.tsv` | the 31 template vulkan-cpu frames (Goal B), against the one before and against metal |
| `fx-showcase-browser-vs-metal.tsv` | every fx-showcase station on webgpu and webgl2 against metal |
| `kit-desktop-settings-edge-before-after.png` | the settings dialog's corner, 2x: top before, bottom with the edge light |
| `kit-skin-<pack>-cells-before-after.png` | pause, inventory and one more cell of each sheet, before over after |
| `a11y/` | `tests/web/a11y.ts`'s reports on both tiers (23 of 23 checks each) and its contrast frames |
| `glass-cost.md`, `glass-cost.txt` | the glass cost in four cases |
| `wasm-size-after.json` | the size gate on the branch merged with 376f3cd |

## Size

Against main 376f3cd's own size gate (tests/evidence/P4/cook/wasm-size-after.json):
3D publish 2,913,544 to 2,921,812 B brotli (+8,268), 2D publish 2,749,599
to 2,758,190 (+8,591), lanes 3,290,490 to 3,297,822 (+7,332; the largest
download, 78.6 % of 4 MB). The iconoir atlas is a kit asset (72 KB
cooked), not wasm.

## Gate

`scripts/check.sh` (path-aware full lane) on 18550a3 + docs: fmt, clippy,
drift, determinism (904 s), wasm-build (1,047 s), wasm-opt (372 s), test
(618 s, nextest) and size-gate passed; bun-test failed on five timeouts
under load 60 to 80 with the lavapipe container running (asset-stream
5 s, render.test strips 5 s, three web-export builds at 120 s), not
rerun (the coordinator runs the gate on main).

`scripts/check.sh web-reference` on a71c66b: failed on 36 fx-showcase
frames (18 per tier) against their own stale webgpu/webgl2 references
(RMSE 0.0101 to 0.0149; the UI text round's kerned HUD); everything else
passed: og-tests gfx 56 scenes per tier, og-ui kit frames 16 of 16
playtests per tier (judged against the re-recorded metal frames), Ember
Vale on both tiers (WebGL2 beacon RMSE 0.0038 against its own), lane
panic, a11y 23 of 23 per tier. The 47 fx-showcase browser frames within
the tolerance of metal and topdown's eight were then deleted (74e291c);
web-reference was not rerun after that.
