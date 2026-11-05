# Compatibility round handoff — unfinished work

Stopped at the owner's round close. Evidence only; no proposed engine fix
in this handoff has been implemented or validated. Runner: studio-win,
RTX 5060, Pixel 10 serial 58051FDCR007QL. Browser measurements use the
existing engine build from `144f9b19c`, not a rebuild of the latest main.
Thread: https://ampcode.com/threads/T-01a1ab9d-3662-750e-b906-7429d51e0ebd

## Landed

- SSR: `8ae87efac`, browser fixture/evidence `226f11fbf`. Chrome three
  scenes pass; fresh DX12 high-preset pass; art's scoped Metal suite 17/17.
- Derivative audit: `d54163388`, evidence `3ff897b43`. Shader tests 5/5;
  Vulkan 517/526, every failure reproduced on unchanged parent. See
  `derivative-audit.md`. No Metal frame change expected.
- Vulkan water references: `b71681daa`. Normal sampler retained;
  anisotropy-clamp-1 controls isolate the backend difference. Fresh
  water/ocean tests 19/19 pass. Both frames inspected here and by art.
- Earlier Android evidence: `f4d3f97af`, `b6a082bcf`.

## Android E5 completed: 17/26, with an actionable shader failure

`bun tests/web/reference-e5.ts --android 58051FDCR007QL` completed.
Raw log, water failure JSON and black water PNG:
`tests/evidence/compat/android/e5-144f9b19c/`.

| Failed station | RMSE | Outliers % |
| --- | ---: | ---: |
| controller | .01039 | .272 |
| environment | .01948 | .645 |
| fluids3d | .03997 | 3.766 |
| geometry | .01438 | .436 |
| islands | .68045 | 99.517 |
| materials | .01142 | .124 |
| terrain | .54775 | 99.517 |
| volumetric | .02212 | .047 |
| water | .74012 | 99.517 |

Water's engine log reports Tint refusing `dpdyFine(ms)` in water's fragment
shader: `below_view`, derived from fragment input, has already returned on
one branch. The invalid water pipeline invalidates the frame command buffer.
This is **not** a legitimate backend reference difference. The same source
ordering remains on main `71a428479`: `water.wgsl` has the underwater return
around line 1115 and lake slope derivatives around line 1309. Islands was
also visually inspected and is completely black; it uses the same water
shader, but its per-station engine log was overwritten by the stock runner.
Terrain has the same near-total mismatch; its precise cause remains to verify.

Next: hoist the lake mean-slope derivatives after the final normal adjustment
but before non-uniform returns; compile the actual assembled WGSL in Chrome
and on the phone, not only naga (the existing native shader tests pass).
Then rebuild in the exclusive wasm window, rerun water/islands/terrain and
the native/Metal water controls, preserving tolerances. No shader edit was
made before the stop instruction. Other misses need desktop Chrome controls
before any per-phone reference decision. All these runs remain studio-win work.

## Other open compatibility work

- **Toon/PBR:** DX12 and Vulkan PBR 26 / toon 13. Art's fresh Metal capture
  on `b71681daa` corrects the historical 29/13 report: **Metal now 26/12**.
  PBR agrees; the remaining difference is the toon row. Art reports three
  flat bands with filtered transition pixels and edge wobble, suggesting
  an extra transition bucket on Windows, not broken quantization. Fine/plain
  controls do not change it. Metal PNGs and RGB/bucket rows are preserved at
  `/Volumes/app/origin-engine-lane-9/.amp/in/toon-{pbr,toon}.png` and
  `toon-row-{pbr,toon}.txt`, with PNG attachments in this thread. Next compare
  a fresh DX12 row y=90, x=100..219 to identify the extra bucket; no new run
  was started after the stop. Do not relax the strict `toon * 2 < pbr` check.
- **DX12 garment/impostor/post-adjust:** all reproduce on main `17edeb203`
  without the derivative change: .03373 / .01007 / .01008 RMSE. Garment
  has 1.569% outliers. Vulkan also fails garment/impostor, while its existing
  backend-specific post-adjust reference passes. The newer Metal post-adjust
  reference follows the accepted shadow-tent change (`6d1524a61`); DX12
  before/after derivative images are identical. Isolate shading/shadow
  differences rather than record unexplained mismatches.
- **Firefox tap, paused:** raw data under `web/firefox-warm-control/`.
  Original 600ms assertion retained. Original Chrome 4/4 modes pass;
  Firefox auto sync isolated/nonisolated fails around 1.5s while async
  passes around 111–126ms. Warm enabled control: isolated advances at
  1592ms, nonisolated 72ms. Disable only `warmBehindGate`: 92/73ms, both
  pass. Six pre-tap queue callbacks complete at +1542ms on the failing
  isolated run; this does not distinguish GPU work from delayed callbacks.
  Interrupted sync/prewarm comparison: sync passes 4/6 tap checks,
  prewarm 0/4. Sync isolated 1539/1177/171ms, nonisolated 103/534/81ms;
  prewarm isolated 1666/3778ms, nonisolated 1580/3747ms. First-frame boot.at
  1318–1440ms sync, 1257–1341ms prewarm, all first frames 9 sync/0 async
  pipelines. Thus **prewarm benefit is not confirmed**. No policy change,
  no cold-cache claim. Partial logs and scratch instrumentation were uploaded
  to the Safari owner's thread before local scratch cleanup.
- **Skinned-hooks:** historical DX12 .01011 / phone .01008, unchanged by
  texel-centre fix. Fresh Vulkan test passes against its selected reference.
  Isolate the non-Metal difference before accepting another reference.
- **Grass determinism:** one inherited DX12 full-run failure at tick 90
  drawn twice. Did not recur in this Vulkan run. Chase only on recurrence;
  the separate shared_frames resize issue is already filed.
- **Phone device loss / G1–4:** not run. Force device destruction in the
  Worker, verify `display.deviceLosses` increments and visible rendering
  recovers; rerun g14 auto quality on the current build after cooling.
  Old g14 tooling is on `origin/android-scratch-wip`, not for landing.
- **Phone water-wake rejection:** not reproduced here. Capture full Worker
  console/rejection reasons through the preceding gfx scenes. Distinguish
  shader rejection from resource pressure; fix the engine if pressure.
- **Android UI:** do not record style-pack/scene-HUD references. Those
  tasks were canceled. Rerun UI only after UI 1B M2's plain-ground layout
  and retirement of style sheets lands.
- **Remaining phone runs:** wire, template playtests, fx stations with
  `--time 120`, plus desktop controls for misses. Not run this round.
- **G1–7:** monitor-change DPI, fullscreen, 16:10–32:9 HUD safe zones,
  raw mouse input and physical-key bindings remain unimplemented here.
- **Showcase frame times:** reference-gfx `--perf-out` and
  `scripts/perf.sh --measure` remain open, with nothing else on the GPU.
  Art holds the three SSR PNGs at `.amp/in/compat-*.png` for the next
  showcase archive batch; this runner lacks archive credentials.

## Cleanup and execution ownership

The side worktree `E:\src\og-verify\wt-e295` was removed; its reusable target
directory remains. Requested g17/adapter/hashcorr/dissolve scratch was removed.
Abandoned diagnostics and owner-authorized desktop browser sessions were
stopped. The Firefox control was stopped on request and its GPU locks released.
The Android E5 runner exited normally with failure status after all 26 stations.
No browser/GPU run or lock remains owned by this thread; no macrunner lock
was acquired. Local scratch scripts/generated wire harnesses were removed
after preservation. Logs and review artifacts remain under `.amp/in/` and
the inherited `E:\src\og-verify\compat` directory, not queued jobs.
Leave unrelated `examples/site-hero` and `examples/site-ocean` alone.
No wasm build may overlap a browser/GPU run on studio-win.
