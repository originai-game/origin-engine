# E2 vulkan-cpu runs (E2B origin-runner-probe, Mesa lavapipe/llvmpipe 15.0.6)

## og-tests: capture-vulkan-cpu.ts (cross-built into target/ with cargo zigbuild, 315 s sandbox)

```
effects.txt:test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 83.97s
tests.txt:$ cd /tmp/og && ./gfx-test --test-threads=2 --nocapture 2>&1 | grep -E '^(gfx |test |pixels|shadowed|partially|auto:|manual:|bgra|[0-9.]+% of|[0-9]+ pixels)|panicked|test result' 
tests.txt:test result: ok. 39 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 97.97s
capture.txt:overlay: 1280x720 tier=vulkan-cpu adapter="llvmpipe (LLVM 15.0.6, 256 bits)" aa=Taa draws=318 batches=43 instances=92 culled=0 triangles=95045 shadow_passes=10 shadow_casters=293 lights=4 sprites=52 glyphs=57 sprite_draws=7 setup+first=7820.3ms
```

Only references/vulkan-cpu/overlay.png was new; every other vulkan-cpu reference recaptured byte-identical.

## fx-showcase: packed engine (build.sh --linux-x64, pack.sh), OG_FX_CAPTURE=1 og test . then og test .

```
exit 0, 0.3 s
exit 0, 1.0 s
ok       renderer      vulkan-cpu (a 64x64 frame headless in 3637 ms: og run needs no sandbox here)
exit 0, 5.4 s
og cook: 16 assets, 21 outputs (21 cooked, 0 up to date), 1671264 bytes in cooked/, 7310 ms
PASS frames (19 checks, 104650 ms): every station matches its reference frame
exit 0, 112.4 s
  post.png: 0.000 % outliers, RMSE 0.00000 against /tmp/og/game/references/vulkan-cpu/post.png
  lens.png: 0.000 % outliers, RMSE 0.00000 against /tmp/og/game/references/vulkan-cpu/lens.png
  fx.png: 0.000 % outliers, RMSE 0.00000 against /tmp/og/game/references/vulkan-cpu/fx.png
  2d.png: 0.000 % outliers, RMSE 0.00000 against /tmp/og/game/references/vulkan-cpu/2d.png
  stylised.png: 0.000 % outliers, RMSE 0.00000 against /tmp/og/game/references/vulkan-cpu/stylised.png
PASS frames (19 checks, 103424 ms): every station matches its reference frame
exit 0, 103.7 s
exit 0, 0.3 s
```
