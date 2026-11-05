# WIP scratch from the Android thread (not for landing)

Copied from `.amp/in/scratch` and `.amp/in/build` on studio-win
(C:\Users\win\src\origin-engine-android). Paths inside are relative to
`.amp/in/scratch`, so copy them back there before running.

- trace-auto.ts <dist> <out.json> <tapDelayS>: one export traced on the phone (first frame, freeze after tap, boot.pipelines at ff+20/50 s, Dawn slices, Worker console).
- export-cmp.ts <auto|sync|async|prewarm> <out>: fx-showcase test-reference export from the webboot worktree with one `pipelines` value.
- g14.ts <outDir> <stressS>: G1-4, boot, memory, thermals; needs the temporary fx-showcase `display:` command (reverted on android; see git show of templates/fx-showcase/src/main.ts in the thread). G1_SCALE env sets the render scale for the cap windows.
- probe.ts: capabilities / visibilityBuffer after a tap. raf.ts, pace.ts: rAF against engine fps. pxdiff.ts, pxbox.ts, pxrow.ts: PNG diffs. wgsl-check.ts: compile post modules on the phone.
- build.sh, build2.sh, window.sh, native2.sh, native3.sh: the build-window scripts (gate wasm, OG_GATE=1 build.sh, og-tests-web, native DX12).
