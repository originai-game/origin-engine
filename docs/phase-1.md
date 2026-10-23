# Phase 1 — execution plan

Scope: `docs/design.md` §11 phase 1 ("Core"), 13 weeks from 2026-10-20.
Output: a game author can `npm create @origingame/game`, write TypeScript
against `@origingame/engine`, run it in a browser and headless with `og`, and
the platform can run it in an `engine`-class sandbox. Spike findings
(`docs/spike-results.md`) are the baseline; the spike crate is the seed
of `og-render` and is deleted once its code has moved.

## Public packages first, the local loop, the development sandbox on InstaBox, E2B last (2026-10-22)

The owner's decision, after the two below: the engine is consumed as
public npm packages pinned by version, every version is developed and
tested locally against them (the install test on this machine and in a
linux-x64 container, then the platform's local stack and the M7
acceptance), then in the development sandbox, and only then does an E2B
image install it, once, as the last consumer. The layout the platform has
built from a pinned git revision of this repository (`origin-engine.ts`:
`git archive`, cargo, zigbuild, the lanes wasm, a read token) and the E2B
rebuild as the first place a change is tried are what this replaces. The
same day the owner made explicit that **the development stage's sandbox
is InstaBox** (dashboard.instabox.sh), not E2B: the development threads
run in InstaBox machines the platform makes from per-class snapshots of
the pinned version, E2B is production's provider only, and the install
test runs inside an InstaBox machine too (`tests/install/instabox.ts`;
0.1.1 passing in `tests/evidence/release/instabox-0.1.1.md`; the
engine's last script that reached E2B for development,
`packages/engine/scripts/e2b-addon.ts`, is gone). The decision, the
packages, the version rule, `scripts/release.sh` and the complete plan
of what remains to launch (stages R, L, D, F, T, E, P) are
`docs/release.md`; the platform's side is decision E10 of
`origingame/goals/origin-engine/plan.md` and `docs/platform-asks.md`
ask 15. In this table: rows 91 to 98 (the release pipeline, stage R),
each closed with its commit; the first published version is `0.1.0`,
the pinned one `0.1.1`. **The verification stage's machines (the owner,
later the same day):** the InstaBox account stays at one machine (the
free plan, no upgrade) while the features are not all working and the
tests not fully implemented; the owner's own runners carry what a
GPU-less orb cannot: `macrunner` (`/Volumes/app/origin-engine`) runs the
functional lane on metal, records every reference frame and the wave's
end, and `studio-win` (Windows x64) builds and tests the engine natively
for F7 (`docs/release.md` §4 T, the Runners row; §6 item 6).

## Features before optimisation (2026-10-22)

The owner's decision, the same day: the build-out distinguishes feature
work from optimisation, finishes the features first, and takes
optimisation afterwards as the product needs it. What changed: the
perf-2 stream (lane-4: sprite extraction by offset, `og.ui.end`'s text
measurement cache, Lantern Vale's step spikes, an adaptive lanes grain,
the terrain streaming test) was stopped with its three commits and its
in-progress work parked on branch `perf-2` (4c7ac856); web-boot-2 was
cut to Kestrel's menu fade alone (the pipeline warm-up after the tap
dropped); the owed rows that are optimisation or its measurement, 14,
44, 51, 52, 56, 57, 88, 89 and 90, are deferred until the features are
complete; rows 51 and 52's Chrome measurements leave the wave-end list.
What stays at a wave's end is correctness: the functional lane, the
wasm build (the web must run), determinism and web-reference (both
browser tiers' frames); the size gate stays in `--all` as the product's
4 MiB download budget, a number to read, never a reason to hold a
merge. The waves after this one carry feature streams and dogfood
rounds only: the feature rows of the owed table (55, 58, 59, 60, 61, 63)
and the plan's phase 2 and 3 items still missing, checked against the
waves that already landed them.

## The CPU sandbox requirement set aside (2026-10-22)

The owner's decision, after the question of what the engine needs from a
GPU: the CPU-only sandbox requirement, a playtest's frames rendered on
Mesa lavapipe in a sandbox with no GPU, is set aside to put features
first; the GPU side returns later. What the engine needs: every render
goes through wgpu (Metal, Vulkan, WebGPU or WebGL2), there is no CPU
rasteriser of the engine's own, and the simulation, the probe,
determinism, replays and UI dumps need no GPU (tier `none`). What the
requirement cost: lavapipe in the `engine` and `probe` sandbox images,
a cross-build and a docker run of every playtest at each wave's end
(8,964 s for the playtests alone in wave 11's run, holding the gate
locks), the vulkan-cpu reference copies and their table, and rows 54,
70, 74, 81, 82 of owed work. What changed (this commit):
docs/design.md §5 (the sandbox classes carry no renderer; screenshot
checks there report "no renderer, frame unchecked" and pass; frames are
verified on the author's machine, in `browser-check` or on an executor
with a GPU) and §8 (native Linux: full with a Vulkan device, else
none); `scripts/wave-end.sh` runs `gate` and `web` by default and keeps
the lavapipe stages for `--stages`; docs/ci.md §The wave-end run and
§Browser reference tiers; AGENTS.md; the last 25 `vulkan-cpu` copies
(og-tests 7, Hollowbell 2, fx-showcase 16) deleted, every tier with no
directory judged against `metal` as before. The engine change the
contract needs, `og test` and `og run` without a renderer never failing
on a frame and saying so, is wave 13's no-renderer stream (lane-1). Rows
70 and 81 are set aside with the requirement; Kestrel Stair's first
lavapipe run and the moved Lantern Vale frames on lavapipe (the wave 13
gate section) are no longer owed.

## Depth wave 1: geometry, timeline, video (2026-10-23)

The depth plan (`docs/depth.md`, nine streams against three.js and the
other engines, the AI-native baseline kept) runs in three waves after
the owner's correction of 2026-10-23 that nine orbs at once was too
many: wave 1 is S1 `geometry`, S2 `timeline`, S3 `video`; wave 2 S5
`probe-depth`, S9 `two-d-math`, S4 `rooms`; wave 3 S6 `render-lod`, S8
`render-targets`, S7 `render-light`. Each stream is a claude-code orb
(Opus 5.5, effort high, `a1.xxlarge`) on its `depth/<stream>` branch,
under §0's rule that no stream runs the gate; the streams of waves 2 and
3 started with the first batch, pushed what they had and paused on
their branches until their wave. The coordinator merged wave 1 in §10's
order on this machine, S1 (4b8e161f), S3 (01408642) and S2 (40c3b73b):
api.ogt's appended modules (video, debug, path, timeline) and the three
`load` functions were kept together, og-cook declares both the video
and the timeline cook, og-runtime installs video then timeline after
feel's front, and the generated outputs were regenerated (`og api gen
--wiki` writes only the wiki; the plain `og api gen` must follow it,
then `api check`). What each stream landed, its frames and its numbers
are in `tests/evidence/depth/{geometry,timeline,video}/NOTES.md`; the
IDL changes are the CHANGELOG's `## Unreleased` depth wave 1 entry.

- **Metal references** (707d7f7d): og-tests' `geometry-procedural`,
  `lines`, `points`, `wireframe`, `debug-overlay`; fx-showcase's
  `geometry.png`; the game template's `intro-4.png` and `intro-8.png`;
  the video fixture's `video-90.png`. Every frame read, and the three
  compared side by side with the streams' lavapipe frames: the same
  composition, only shading and anti-aliasing differ. Read: the
  geometry scene's height field occludes the lower part of the
  cylinder, cone and capsule (the stream's framing, kept); the
  `wireframe` scene's solid sphere is the test's control; the
  `debug-overlay` text label is not visible in either tier (owed: a
  look at `og.debug.text` in that scene); the game template's menu
  panel stays over the intro's frames, since the playtest never presses
  Start (the template's choice, visible in both tiers); the video frame
  reads `00:00:03.000` and `90` over the test pattern, magnified with
  nearest filtering as the fixture asks.
- **Integration the streams could not see** (their rule: no gate).
  `cargo fmt` after the merges (6066cb6d). og-idl's `load` function
  list gains `video`, `videoInfo`, `timeline` (in the merge). The
  create package's tests scaffolded from a stale untracked
  `packages/create/template/` left by an earlier prepack, which shadowed
  the live `templates/game`: in the repository the live template now
  wins, a published package has no `templates/` beside it (78b3654f).
  `api-surface.test.ts` found `Path`, `PathKind` and `TimelineAsset`
  without column factories, and then the runtime refused a game field
  of `PathKind`, an enum column no engine record uses, since
  og-runtime's codec learned an enum's variants only from `RECORDS`'
  fields: `og api gen` now emits `COLUMNS` (every `column X` with its
  variants) and the codec reads them there, so any declared column kind
  is a game's to use (c51d4c36). The template and no-renderer tests
  count the game template's third suite, `intro`, and its two frames.
- **Owed to the wave's end** (`scripts/wave-end.sh` after wave 3):
  S1's raycast inside a system refreshes the transform hierarchy every
  tick (its cost is a wave-end measurement); `og render` is native only
  and the web host has no `Capture::frame`; several players of one
  video share the first player's texture; loaded glTF geometries get
  their BVH at the first raycast, not at load; the intro's audio is
  unheard (headless), and hot reload during a timeline's playback is
  untested; the wasm build, its size and the browser tiers run at the
  wave's end, not here.

## Depth wave 2: probe-depth, two-d-math, rooms (2026-10-23)

The three streams resumed on main 24116264 the moment wave 1 was pushed
and reported the same day, S4 first, then S9, then S5. The coordinator
merged them in §10's order on this machine, S5 (c447ff93), S9 (01b0f526)
and S4 (409662ad). S5 merged clean. S9 conflicted only in generated
outputs (bindings.json and the wiki), api.ogt itself merged; the outputs
were regenerated. S4 conflicted with S5 in og-cli's `run.rs` (`--trace`
beside `--room`, in the doc comment, the usage line and the flag match),
in `run.ts` (both flags; the exit code stays 1 on a failed expectation,
and a room closes before it) and in `playtest-run.ts` (one `Summary`
with S4's `netHash` and `lockstep` and S5's `expectations`; a run whose
only failure is its expectations is still a check, not a failed run).
S4's bbce63bc (the IDL, "WIP (stream deferred)") builds only with the
commit after it; the branch is merged as pushed, its tip builds. What
each stream landed, its frames and its numbers are in
`tests/evidence/depth/{probe-depth,two-d-math,rooms}/NOTES.md`; the IDL
changes are the CHANGELOG's `## Unreleased` depth wave 2 entry.

- **Metal reference** (f1d1090f): og-tests' `skeleton-2d.png`, the one
  gfx frame the wave added. Read beside S9's lavapipe frame: the same
  three keepers, on a flat plank facing right, on a rising plank facing
  right, and on a falling plank facing left with the cape on the right;
  the lanterns lit, the capes behind the coats. S4's arena frame (peer 1
  at tick 360, "Red 2 / Blue 3", "You are Blue", one coin above the two
  avatars) and S5's `first-frame`, `smoke-tick120` and `broken-tick120`
  are evidence, not references; the game-2d keeper playtest checks its
  frame's size and the rig's dumps, not a reference.
- **Integration the streams could not see.** `packages/engine`'s
  typecheck failed after S5: `run.ts` cast the probe's `expectations()`
  to a hand-written copy of `ExpectReport` whose `results` was mutable
  where the generated one is `readonly`; the generated type is the one
  used now (8b404e6b). The og-game skill had two `## 21.`: S4's §20 and
  S5's §21 both landed before "Where to look things up", which §1's
  references expect last; it is §22 now, and the knowledge validate
  passes. The wiki, bindings and schema were regenerated after each
  merge (`og api gen`, `--wiki`, `api check`). The gate then found what
  the merged tests expected of each other: og-idl's member lists lacked
  S9's `rig2d` load and `anim.rig2dInfo`, `packages/engine` exported no
  `Rig2D` column factory, and the template and no-renderer tests counted
  the smoke playtest before S5 rewrote it as `expect:` steps. That last
  one exposed a contract break: a `region` or `frame` expectation is
  unchecked without a renderer but was not a frame check with one, so
  "checks minus frame checks is the same with or without a renderer" no
  longer held; `playtest-run.ts` counts them as frame checks now, and
  the smoke playtest reports 9 checks with a renderer and 8 checks, 2
  frames unchecked, without.
- **Owed to the wave's end**, beyond wave 1's list: the platform side of
  rooms (platform-asks #16), a browser two-page room run, the `?room=`
  Play link and rollback (S4); the trace is native-only, lays systems and
  passes end to end from their lengths, and its pass times are CPU encode
  time, not GPU time; write-back covers the boot scene and skips tuned
  resources; occlusion in `describe` ignores UI overlays (S5); Spine
  import, DragonBones skew, colour offsets, zOrder, IK timelines and
  nested armatures, Bézier curves played as linear, IK of more than two
  bones, and a walk cycle for the keeper (S9).

## UI styles: og-ui decoupled from the platform's design language (2026-10-23)

The stream `ui/style` (an orb, Linux, so every Metal reference was
owed) landed between depth waves 2 and 3: the vendored platform
`tokens.json` and its generator gone, og-ui's own palette of 33 roles,
the theme and the skin one **style** asset a look's `ui` line names,
`prototype` as the placeholder a game without a style draws in, and
square action and pause buttons. Its seven commits fast-forwarded onto
main c5aa7ff1 at 07fab232; CHANGELOG's Unreleased entry lists the API
changes.

- **Integration fixes** on main: the arena archetype's HUD still named
  `fgMuted` (6dfacd51, `inkMuted`); the knowledge scaffold's distribution
  digests follow templates/game's intro references (5dce0c28); the
  examples' generated style binaries (gitignored, `tools/make-assets.ts`)
  moved with their `.ogt` from `assets/skins/` to `assets/styles/`
  (25da2bde), without which brasswick, meridian-loop and rumpus-isle did
  not cook.
- **One defect found by a frame.** Kestrel Stair's intro moved 16.5 % of
  its pixels, far past a restyle: the kit's cutscene letterbox bars took
  the `backdrop` role (black in the platform theme, cream `#efe6d2` in
  kestrel's paper style) and drew as two textured panels with rounded
  corners and outlines. Cinematic bars are black in every style
  (9771f711); the intro then matched its reference at 0.015 % outliers,
  so it was not re-recorded.
- **References re-recorded on Metal**, each pair read before recording:
  og-ui's sample-screen, widget-set, materials and six text sheets
  (9e697f6b; prototype greys and cyan, square or modest corners; the
  clipped "Item 0" row and the blocky octagons in materials are in the old
  references too); templates/game intro-4 and intro-8 (the caption panel
  charcoal, was slate); all 75 kit sheets (95a6a459, `OG_UI_CAPTURE=1 og
  test templates/ui-kit`, 34 suites); and the 86 playtest frames the
  styles moved (0ce8c7be): fx-showcase's 34 looks frames, idle, rhythm,
  roguelike, third-person and tower-defense, and every example but none
  of match3, topdown, runner or the fx-showcase stations, which passed
  unchanged. Read: hollowbell's talk (the line cut at "bell-keepe" is the
  typing effect at the same point as before), tower-defense's wave and
  rumpus-isle's podium (their rounded capsules, pills and offset text
  shadows are the games' own `pathguard` and `rumpus` styles, radius
  10/18, unchanged from the old frames), the kit's cutscene (black bars,
  charcoal box, a quiet Skip).
- **Not UI, noted as owed.** lantern-vale's physics playtest fails on
  this machine: rock-4 moves 0.085 m against the 0.2 m check (rockMoved
  0.52 passes). It fails the same way on the measure lane at da1d3ea3,
  before 0.2.0 and every depth merge, so it predates this round; nothing
  was re-recorded for it.
- **Owed**, from the stream's list: internal Rust names (`skin.rs`,
  `Theme::skin`, `SkinFile`), ui-kit's `skin:` command and the
  `kit-skin-*` file names, rumpus-isle's and pathguard's own copies of
  the old cartoon art, the browser web-reference tiers of every frame
  above (the wave's end), `og doctor` tests failing in the orb (looks
  environmental).

## Depth wave 3: render-lod, render-targets, render-light (2026-10-23)

The three rendering streams resumed on main c5aa7ff1 and reported the
same day, S6 first, then S7, then S8. The coordinator merges them in
§10's order, S6, S8, S7, because each one changes og-render's frame
graph, `frame.rs`, `scene.rs` and the lit WGSL. What each stream
landed, its frames and its numbers are in
`tests/evidence/depth/{render-lod,render-targets,render-light}/NOTES.md`;
the IDL changes are the CHANGELOG's `## Unreleased` depth wave 3 entry.

- **S6 render-lod** (a14ba640, merged as pushed at 9430bea2) conflicted
  only in the generated wiki, regenerated after the merge (`og api gen`,
  `--wiki`, `api check`). The api-surface test then refused
  `ReflectionProbeMode` as a column kind: the addon in
  `packages/engine/native` predated the merge, and rebuilt (`OG_GATE=1
  sh scripts/build.sh`) it takes the column. Three **Metal references**
  recorded (`OG_GFX_CAPTURE=lod-levels,probe-interior,ssr-wet-floor`),
  each read beside S6's lavapipe frame: `lod-levels` (the near tree
  green with the full canopy, the middle one yellow and simpler, the far
  one orange and smallest, shadows on the ground); `probe-interior` (the
  chrome sphere shows the blue left wall, the red back wall, the pale
  ceiling, the grey floor and the green block; nothing of the sky, the
  one difference to lavapipe a slightly darker lower half);
  `ssr-wet-floor` (the red sphere and the white panel mirrored in the
  black floor, the panel's reflection grainy at its foot on both tiers;
  the dark wedge left of the panel is its shadow, present on both tiers
  too). The `lod` and `reflections` stations have no playtest, so no
  station frame is a reference. The gate then found two tests the
  stream could not run: `no-renderer.test.ts` compares every dump with
  and without a renderer, and an entity's new `render.lod` exists only
  with one, so the comparison leaves the `render` block out (it is the
  renderer's observation, not the game's); og-probe's conformance test
  spelled `state.timings` without `triangles` and now sets and checks
  it. One flake beside them: og-cli's `dev_reports_the_check_of_each_save`
  once saw the watcher reload twice (`n: 2`) under the gate's load and
  passes alone three times running; it is not S6's.
- **Owed to the wave's end**, S6: probe captures skip skinned, terrain,
  scatter and transparent meshes; one probe per batch, not per instance;
  SSR reads material factors, not maps; no expect-step station playtest;
  WebGL2 not run in a browser.
- **S8 render-targets** (adfd6d64, merged as pushed at 148b0617, rebased
  on c5aa7ff1 so every conflict was with S6): og-render `lib.rs` (the
  prepass now runs for custom posts, AO or SSR; at the frame's end the
  probe captures resolve, then the viewport views; the views' `draw_list`
  takes S6's probe argument), og-runtime `engine.rs`/`extract.rs`/`api.rs`
  (the probe and views layouts side by side, the capability JSON with
  both streams' fields), og-cook `cook/mod.rs` and `runtime.rs` (S8's
  `post` beside main's `style`, S8's stale `skin` names dropped),
  `api.ogt` (`PostEffects` ssr then `custom1..4`, the capabilities, `og.Model`
  `lodBias` then `layers`), fx-showcase `main.ts` (the lod, reflections
  and views stations all label as depth stations), design §2.6 (S6's
  paragraphs then S8's) and the og-game skill (S8's section is §23,
  "Where to look things up" §24). Generated outputs regenerated; `api
  check`, `cargo check --workspace --all-targets`, og-idl, og-probe, the
  knowledge tests and api-surface (after the addon rebuild) pass. Five
  **Metal references** recorded (`OG_GFX_CAPTURE=views-monitor,views-mirror,
  views-split,shader-full-dissolve,post-crt`, og-tests `targets`), each
  read: `views-monitor` (the inset on the back wall shows the second
  camera with the magenta cube, which the main view lacks); `views-mirror`
  (the glass in the blue wall reflects the green pillar, the red box and
  the blue sphere); `views-split` (left the front camera, right the
  top-down view with the yellow cube); `shader-full-dissolve` (the torus
  and the eroded slab with pale glowing rims, the shadow cut by the same
  holes, as on lavapipe); `post-crt` (barrel distortion, black corners,
  scanlines, the room still readable). Station references `views.png`
  (new: the mirror wall, the back-wall monitor and the bottom-right
  top-down inset) and `shaders.png` (the cyan panel dissolving beside the
  disintegrating sphere and the grass, now through the CRT pass; the old
  frame had no distortion) recorded with `OG_FX_CAPTURE=1`.
- **Owed to the wave's end**, S8: views draw opaque meshes, terrain and
  sky only (no transparents, sprites, particles, lines or post); a frame
  with views culls on the CPU; target depth is full tier only;
  `PostEffects.custom` is four fixed slots, not a list.
- **S7 render-light** (f55588a8, merged as pushed at fe2dd662, rebased
  on c5aa7ff1, so its conflicts were with S6 and S8): og-render `lib.rs`
  (the prepass runs for custom posts, AO, SSR or volumetric fog) and
  `post.rs` (the main pass reads both S8's `views` and S7's `froxels`);
  `pbr_fs.wgsl` and `physical.wgsl`, where the ambient term is
  `ambient_irradiance(ambient_light(n, p), p, n)`: a lightmap or light
  volume wins, else S6's probe-blended environment; og-runtime
  `extract.rs` (the lightmapped tuple keeps S6's `bias`), `api.rs` (the
  no-renderer capability JSON carries `volumetricFog: "raymarch"`),
  og-tests `NATIVE_SCENES` (+ `baked-cornell`); `api.ogt` (`RenderSettings`
  `lodBias` then `bakedLighting`; `og.Model` `lodBias`, `layers`, then
  `static`); `index.ts` (`render` spreads S1's, S8's and S7's modules,
  `load` S8's); fx-showcase `main.ts` (`...VW_STATIONS, ...LIGHT_STATIONS`,
  usage `<name|1-32>`, the `baked` command and the `LightShow` system);
  og-cli `main.rs` (`bake probes` dispatched before `bake`); design §2.6
  (S6, S8, then S7) and the og-game skill (S7's section §24, "Where to
  look things up" §25). Generated outputs regenerated; `api check`, `cargo
  check --workspace --all-targets`, og-idl, og-probe, og-render, the
  knowledge tests and api-surface pass. Four **Metal references**
  recorded (`OG_GFX_CAPTURE=rect-lights,light-cookies,volumetric-fog,
  baked-cornell`, og-tests `light`) and read against S7's lavapipe
  frames: `rect-lights`, `light-cookies` and `volumetric-fog` are the
  same picture on both (the cookie blur too, and the vertical froxel
  streaks of `volumetric-fog`, which S7 traced to the 160-column grid);
  `baked-cornell` shows the red and green bleed, a bounce-lit ceiling,
  the bake's noise and its jagged shadow edges. Station references
  `baked.png` and `volumetric.png` recorded with `OG_FX_CAPTURE=1`;
  `volumetric` shows the window shafts, the floor patch, the lantern's
  dots and the cyan panel. **The `baked` playtest failed on the first
  capture**: the station drew unbaked (block side 94,102,119, ceiling
  luma 46, S7's unbaked numbers). `og.render.bakedLighting` gave the
  reason, "its bake is stale: scenes/cornell.ogt changed since og bake
  lightmaps": the scene source is untouched, but the merged cook (the
  schema S6 and S8 added to) gives the scene a new output hash, and the
  bake is keyed on that hash. The fix is the legitimate one, a rebake
  with S7's options (`og bake lightmaps templates/fx-showcase --scene
  scenes/cornell.ogt --samples 512 --texels-per-metre 32`): the new
  `.lightmap.ogt` differs in `scene-hash` and the bake's seconds only,
  the pages are re-traced, and the playtest passes (7 checks). The
  lesson for the wave's end: a bake goes stale with every engine change
  that moves a scene's cooked bytes, so every project's bakes are
  re-baked after a merge that touches the scene schema, and `og doctor
  --project` is the check. **The functional gate then failed six og-tests**
  (`a_reflection_probe_puts_the_room_in_the_chrome_sphere`,
  `probe_interior`, `views-monitor`, `views-mirror`, `views-split` and
  `a_layer_the_main_camera_lacks_shows_only_on_the_monitor`) with a wgpu
  validation error, "10 bindings against a layout of 12": S7 added the
  light-volume probes and the LTC table to the frame layout, and S6's
  reflection-probe frame groups (`lib.rs`) and S8's view frame groups
  (`views.rs`), written against the old layout, were not in any of the
  three branches' conflicts. Both groups now carry the two buffers; the
  six tests pass against their Metal references unchanged (no probe or
  view scene has a lightmap or a rect light), and rustfmt's order of
  og-cli's `mod bake_*` lines, which the gate's fmt step caught first.
- **Owed to the wave's end**, S7: one scene's lightmap drawn at a time;
  the physical, texture-array, material-shader and compact batches use
  the probes, not the lightmap; rect lights cast no shadows and terrain,
  scatter and fluids light them as a point; cookies are capped at 24
  layers; WebGL2's volumetric fog is lit by the sun alone; render
  snapshot/restore keeps the baked data by design but is untested; the
  froxel grid's vertical streaks in `volumetric-fog` (on Metal too).

**Wave 3 end (2026-10-23, `scripts/wave-end.sh` on main at c5274b67):**
the first run failed at `wasm-build`: og-cook's new lightmap reader
(`Bundle::load_lightmap`, S7) read the KTX2 level offsets as `usize`
and called the native-only KTX2 decoder, so the web host did not build
on wasm32 (no stream builds the wasm, by the rules of parallel work).
The reader is cfg'd out of wasm32 and reads the level table as `u64`
(c5274b67). The second run: **gate PASS (855 s), web FAIL (455 s),
conformance PASS (4 s), release PASS (720 s)**.

- The full gate passed all nine steps: fmt 1 s, clippy 5 s, drift 7 s,
  wasm-build 459 s, test 217 s (1,100 passed), wasm-opt 210 s, size-gate
  49 s, bun-test 176 s, determinism 450 s.
- Shipping wasm brotli bytes: 3D lanes 3,482,855 (83 % of 4 MiB); 2D
  lanes 3,303,555 (78 %); 3D publish 3,443,932 (82 %); 2D publish
  3,271,785 (78 %). The 3D lanes module grew 313,534 bytes since the F1
  figure, the depth waves 1 to 3 and ui/style together (waves 1 and 2
  ran no wave end, so there is no figure per wave); the largest module
  keeps 17 % of headroom.
- Every determinism build agrees: math 6117185439544271997, bouncing
  boxes 15551159556835976588, physics3d 917465598921449438, physics2d
  4867922716644344347, animation 6030136770684287795. Native lanes used
  four threads and made 1,660 parallel calls.
- Conformance: tsrs 0.18.0-ts7.1.0-dev.20260929 against tsgo 7.0.2, no
  difference.
- Release: 0.2.0 packed into `dist/release/0.2.0/` and the install test
  passed on darwin-arm64 (75 s) and in the linux-x64 container (49 s).
  The tree was dirty when they were packed, so `publish` would refuse
  them; nothing was published and the registry's 0.2.0 stands.
- **The browser tiers** failed eleven comparisons, each read and
  diagnosed before anything was re-recorded:
  - `fluids`, both tiers, 98 % outliers: the frame was the default
    scene, not the tank. The run's engine log said "an entity named
    'light' already exists": S7's fx-showcase `src/light.ts` spawns its
    state entity as `light`, and the fluids station spawns its light
    orange fluid as `light`. Native `og test` never hits it because each
    station's playtest is its own run; the browser E5 run shows every
    station in one world. The state entity is `render-light` now (and
    `tests/baked.og.ts` reads it there); both tiers pass against the
    unchanged Metal reference.
  - `looks-paper-craft` and `looks-voxel-worlds` (webgl2, their own
    tier references since 95ea28a8 for SSAO/GTAO): every outlier lay in
    the "og.ui over the frame" badge. Every catalogue preset names a UI
    style since ui/style and the fx-showcase presets `extend` them, so
    the badge wears the preset's style; the two webgl2 references
    predated that. Re-recorded from the webgl2 run; against Metal the
    new frames differ only in the scene (RMSE 0.019 and 0.011), as
    before.
  - The same finding failed one native check the merged gate had not
    run yet, `og test templates/fx-showcase --filter looks`: "look:none
    restores the plain diorama" (RMSE 0.04576, the badge again). A look
    owns only what it names and `none.ogt` has no `ui` line, so the
    last preset's style stayed on the HUD; no slug names the prototype
    style (`UiDesc.style` documents `null` as it). `resetLooks` now also
    calls `og.ui.configure({ style: null })`; the suite passes its 78
    checks, with `none` back to RMSE 0.00000.
  - `volumetric` (webgl2, 2.857 %) and og-tests `volumetric-fog`
    (webgl2, 0.530 %): WebGL2's fog is the sun-only raymarch by S7's
    design, so the tier draws no lantern or rect-light scattering. Both
    frames read as complete renders (the window shafts and the floor
    patch, every surface present) and are recorded as webgl2 tier
    references.
  - `fluids3d` (webgpu, RMSE 0.01074 against 0.01) and `water` (webgl2,
    0.01091): the difference lies in the fluid and water surfaces,
    spread over the whole surface, no region missing. Native passes
    both against Metal (0.00653 and under 0.01), and S7's ambient
    change is the one shader change of the wave that reaches them. Both
    recorded as tier references (`webgpu/fluids3d.png`,
    `webgl2/water.png`), the first webgpu tier reference of the
    showcase.
  - third-person `beacon` (webgl2, RMSE 0.01025): a drift of the same
    kind over the whole frame, read as complete; re-recorded. Native
    third-person passes 10/10.
  `scripts/check.sh web-reference` was then rerun alone and passed
  every comparison and harness in 345 s (E5 on both tiers, the 37 looks
  on both, og-tests' 102 gfx scenes on both, third-person, the kit
  sheets, lane-panic and a11y).
- **Owed past this wave's end:** S1's raycast hierarchy refresh is
  still unmeasured as a cost of its own (the perf tests passed at their
  baselines, nothing more); lantern-vale's `rock-4` physics drift; the
  froxel streaks; the S7 list above; S8's views draw opaque meshes,
  terrain and sky only; S6's probe captures skip skinned, terrain,
  scatter and transparent meshes.

## Main gate, wave 13 (2026-10-21)

Wave 13 started as wave 12's streams merged, on the web dogfood's twelve
findings (examples/WEB-FINDINGS.md), three streams of four lanes, and two
more when lanes freed. Merged so far, in order: webgl2-render-2
(c20d2c3b; findings 3, 4, 5, verified in Chrome on both tiers by the
coordinator before the merge) and publish-testing (6a81c453; findings 6,
7, 8, 10). In flight: web-boot (findings 1, 2, 9, 11, 12), shadow-flags
(row 87), dogfood-5 (a fourth game, a racing time trial), and wave 12's
perf-games. The round sections below have each.

- **`scripts/check.sh functional`**: the webgl2-render-2 lane
  (gate 55) and the publish-testing lane (gate 56) both failed at
  bun-functional on one test, `terrain-stream.test.ts`, whose premise
  441f4e6e removed: the first frame that draws a streamed terrain now
  settles every wanted tile, so the streamed run no longer has tiles
  waiting at the station's first frame; the test expects the settle
  since e565ede7 and a streaming test whose tiles arrive later is owed
  (row 88). Every other step passed (test 145 s, drift 37 s), and the
  games' suites after the merge (fx-showcase, Ember Vale, topdown,
  Hollowbell, Lantern Vale, Kestrel; Kestrel's `replay` failed once
  under load at 13 s with "the run wrote no end.json" and passed alone
  at 49 s, 4 checks) and the kit's and Hollowbell's after
  publish-testing passed. The lane at 8a23392a passed in 94 s (5 of 5;
  the kit's 34 playtests with `settings-saved` and Kestrel's 15 after
  it) and main was pushed there. Later merges of this wave, each with
  its lane and the suites after it: no-renderer (71c0e9dd fast-forward,
  182 s, 5 of 5), vox-stylise (96be0866, 153 s, 5 of 5; fx-showcase's
  `looks-inked-comic-3d` re-recorded, d539ce13), dogfood-5 (2471dad6,
  90 s; Meridian Loop's seven frames re-recorded on main's engine,
  9782d90d), web-boot-2 (0dffd7be, 129 s), ui-five (5c835a67, six
  generated wiki pages regenerated at the merge) with templates-title
  (015a0ffa) and the kit's 68 frames re-recorded (46e01e62): the lane
  failed one Rust test, og-runtime's no_renderer (ui-five's `macKeys`
  left false on the renderer-less surface; a surface set without it
  takes the platform's keys since ed9e1a62) and Meridian Loop's
  `pub-pause` moved with the kit (6e36bc82); vehicle-physics (7c34f4dd):
  the lane the same test, and the suites Kestrel's save test (a 36 KB
  world in one part since the physics save dropped the mesh colliders;
  9fe3a417) and Meridian Loop's handling-bound playtests (retuned by
  kit-words); racing-ui (c060c875): fmt on the coordinator's own line
  (3802bf70); dogfood-6 (4d888752, 143 s, 5 of 5, Brasswick's 8
  playtests green) and main pushed there. The perf-games merge (2b896be5, a
  fast-forward) failed its lane at bun-functional in the knowledge
  package's drift check alone: og-game §14 named the engine's systems
  as `og.ui.end`, which the check read as an API member (prose since
  5ae8b801; the fourteen project suites after the merge all passed).
  The web-boot merge (b74c5d47) passed in 184 s (5 of 5: test 172 s,
  bun-functional 116 s, drift 55 s; the kit's and the three games'
  suites after it) and main was pushed at e3a7ac9d. shadow-flags'
  merge and lane follow, then this wave's `--all` and web-reference.

**The day's wrap-up (2026-10-22, main at 0e0d71af):** the kit-2d merge
is the last of the day (its lane passed in 95 s after the package's
test matchers gained `toBeLessThanOrEqual` and `toMatchObject`;
20954c20 re-recorded the level select's frames). Three streams were
stopped mid-way at the owner's request and their work committed as WIP
on their branches, pushed to origin: `kit-words` (the kit HUDs' words,
the kit's tables and subsets, Meridian Loop on the new HUD fields and
retuned to the vehicle defaults; Meridian Loop's `ghost`, `lap`,
`rerun`, `locales` and `frames` playtests fail on main until it lands),
`platformer-2d` (Brasswick's runtime findings 6, 5, 3, 13, 8, 7, 2, 14,
1) and `dogfood-7` (a local-multiplayer party game, begun). `perf-2`
stays parked. Still owed at this point: the wave's end (`--all` and
web-reference at the last merge), Brasswick's and Meridian Loop's
browser frames on both tiers and the published-page harness on the
new games, and the rounds for the three stopped streams.

**F1, two of three (2026-10-22, from an InstaBox-stage orb, no GPU):**
`kit-words` (origin/kit-words abad0059) merged at a61fd4f4 and
`platformer-2d` (origin/platformer-2d 7e5084b5) at c9845b1e, CHANGELOG
holding one `0.2.0 (unreleased)` section for both streams' API
additions (the kit HUDs' label fields; `og.WorldRestored`,
`og.PARTICLES_ARMED`, `CharacterController.pushStrength`,
`PlatformerController.pushBodies/pushStrength`,
`TilemapLayer.colliderLayers/colliderMask`, `PlatformAnswer.time`), the
wiki regenerated, `og api check` clean; `release.sh bump 0.2.0` waits
for the next publish. The functional lane on the merged tree found two
debts of main, neither from the streams: `cargo fmt` of
`og-cli/src/doctor.rs` (45848398), and og-cook's twelve tests that
still read `templates/game/assets/models/trickster.glb` after R6 moved
it out of the template (3f914f69): the fixture is now declared once in
`og-cook/tests/support` as the template's assets plus the engine's own
copy from `crates/og-tests/assets`, staged by `stage_template`
(bf7db9f4; 111 of 114 og-cook tests here, the other three want a
`vulkan-cpu` reference frame this orb has none of). What the orb could
not do and the owner's metal run owes: Meridian Loop's `race`, `ghost`,
`results`, `settings-de` (perhaps `pub-straight`) frames move with the
new HUD rows (the splits in rows, no sector capsule, the delta under
the lap time, a time as the results headline) and are to be re-recorded
there; Brasswick's `door`, `stomp`, `gate` on lavapipe sit at RMSE
0.0101 against 0.01 (edge dither like the passing title frame, nothing
moved) and are the same metal check; og-tests' five lavapipe-vs-metal
gfx frames (atmosphere-night, atmosphere-moon, sprites-array,
tilemap-lit, water-underwater) are the CPU tier set aside above.
`dogfood-7` (Rumpus Isle) is still in flight on its thread.

The owner's Metal follow-up passed `scripts/check.sh functional` on
ff952176 in 90 s: fmt 2 s, clippy 1 s, test 44 s, bun-functional 88 s,
drift reused. The GPU references were reused from the preceding passing
Metal run because their inputs were unchanged. That first run found
`PARTICLES_ARMED` missing from the runtime exports; ff952176 supplies it.
The gate does not run the examples' playtests. Separate runs passed
Brasswick's eight suites with all nine frames at RMSE 0, including door,
stomp and gate. Meridian Loop's eight non-frame suites passed; its frames
and published suites initially failed on eleven stale references.
Each before/after pair was inspected before recording in d07963ef:
`jump`, `slide`, `countdown`, `race`, `pub-countdown`, `pub-straight`
show the new split rows without place or boost controls (the remaining
thin bar is the speed meter). `ghost` also reflects the intended air-yaw
retune, with delta under the lap time. `pause`, `board`, `settings-de`
keep their UI over the retuned trajectory; the latter two are captured
later. `results` shows total time instead of points, with updated lap
records and a later capture. Both affected suites then reproduced all
eleven recorded frames at RMSE 0. No Brasswick reference changed.

**F1, the third (dogfood-7, Rumpus Isle, merged 7aa1b801 the same
day):** the stream's thread stopped on the model's usage limit with its
commits unpushed; the coordinator recovered every file it changed from
the thread's Changes capture onto `origin/dogfood-7` (90f8750f) and a
second thread finished it from there (117e27a8). The sixth dogfood game
is `examples/rumpus-isle`: a local-multiplayer party game (four seats,
lobby, berry, bump and crumble rounds, a podium), 40 assets made by
`tools/make-assets.ts`, nine playtests (216 checks) passing with eleven
frames unchecked in the orb, `og polish` taste 100 and feel 93.
`FINDINGS.md` has fourteen findings (no blocker, four annoyances, ten
nits), each with its encounter, workaround and proposed fix: fixed in
the stream, #14 (`og test --filter <path>`, 8dfd2416, the regression
failing on the old CLI) and the knowledge notes for #7, #9, #10 and
#12 (baf28bfe); left as proposed engine work, #1 profile-aware look
ranking, #2 typed Font and Skin field factories, #3 `worldToScreen`
name-tag guidance, #4 the reserved command names, #5 the colon syntax
with several arguments, #6 named-output collisions, #8 `dump` of vector
arrays, #9 a contact-time velocity or impulse on the collision event
(both physics backends, the reflected event, host parity), #11 f32 and
JSON precision, #13 polish advice aligned with its three-of-four rule.
The Mac recorded and inspected all eleven Metal frames in e9b69719:
title with records, three-seat lobby, Bump Ring card at 3, two critters
meeting, Moss splashing into the sea, the 3/2/1 scoreboard, six berries
with zero counters, the crumble grid, pause, French settings, and the
9/8/7 podium. The scoreboard capture moved one second later to clear the
iris and finish its counters; subsequent commands moved with it. All
nine suites passed (239 checks), every frame reproduced at RMSE 0, and
the functional lane passed in 91 s. Contact name tags still overlap;
the skin's four-pixel display shadow is heavy and tile cracks are subtle
in the still frame. The orb's earlier five inspected lavapipe captures
were not used as references.

**Wave-end figures (waves 12 and 13 so far, the measure worktree at
d56f3490, after the shadow-flags merge; no lavapipe run and no Chrome
benchmarks, both set aside on 2026-10-22):**

- **`scripts/check.sh --all`**: passed, 9 of 9 steps in 871 s (wasm-build
  321 s, wasm-opt 330 s, test 315 s, determinism 236 s, bun-test 216 s,
  size-gate 59 s, drift 14 s, clippy 13 s, fmt 2 s).
- **Wasm** (brotli): 3D lanes 3,147,276 B (75 % of the 4 MiB budget; raw
  11,647,595), 2D lanes 2,976,665 (70 %), 3D publish 3,110,573 (74 %), 2D
  publish 2,941,945 (70 %); a number to read, the budget being deferred
  work (row 14).
- **Determinism**: every build agrees (native debug, release with lanes
  off and on at 4 threads over 1,660 parallel calls, wasm32 scalar, the
  3D and 2D lanes modules): math 6117185439544271997, physics3d
  16705304060282334008, physics2d 6277863875895225966, anim
  6030136770684287795.
- **`scripts/check.sh web-reference`**: 474 s; every frame passed on both
  tiers (fx-showcase's five stations and 20 E5 stations, the 37 looks,
  og-tests' 87 gfx scenes on each tier, og-ui's 16 kit playtests on
  each, Ember Vale's three, the a11y checks 26 of 26 on each), and one
  check of the lane-panic harness failed in its single-threaded run:
  the crash's tick was 30 where the check wanted more than 30, the
  fault being armed once the probe's tick passes 30 and the cull it trips
  still tick 30's (the probe counts the tick in progress; the lanes run
  passed at a later tick). The check accepts 30 or later since this
  commit (bdc1d850); rerun on the measure build, both runs pass every check.

**F1 wave-end follow-up (2026-10-22, measure pinned at da1d3ea3,
before dogfood-7):** `scripts/wave-end.sh` finished in 1,722 s with
gate passing (682 s), web failing (354 s), conformance passing (6 s),
and release passing (680 s). No lavapipe reference stage ran.

- The full gate passed all nine steps: fmt 1 s, clippy 14 s,
  wasm-build 314 s, test 203 s (947 passed, 12 skipped), wasm-opt 202 s,
  size-gate 43 s, drift 14 s, bun-test 162 s, determinism 258 s.
- Shipping wasm brotli bytes: 3D lanes 3,169,321; 2D lanes 3,001,064;
  3D publish 3,132,262; 2D publish 2,966,074. The largest fits 4 MiB.
- Every determinism build agrees: math 6117185439544271997,
  bouncing boxes 15551159556835976588, physics3d 917465598921449438,
  physics2d 4867922716644344347, animation 6030136770684287795.
  Native lanes used four threads and made 1,660 parallel calls.
- Both browser tiers failed the locale playtest because the runner
  treated 25 diagnostic HUD screenshots as references. The native test
  deliberately compares only its original 15 frames. f3d2d858 shares
  that list with the browser runner; both tiers then passed all 15,
  and native locales passed all 147 checks without new references.
- The other browser failures are stale `stylise-pixel` and
  `stylise-palette` Metal references (RMSE 0.24104 and 0.17555 on both
  tiers). A fresh native capture is pixel-identical to WebGPU, removing
  the same dark horizon band. This matches the earlier grazing-plane
  outline correction in 8a9338b6, not either F1 merge. Those two scenes
  have no native reference test, so the passing native gate did not
  cover them. Their references remain unchanged for the owning stream.
  All other browser comparisons passed; lane-panic passed 23 checks
  and a11y passed 26 per tier.
- The 0.1.1 tarballs packed and passed the install test on darwin-arm64
  and linux-x64. The Linux container has no renderer and reported its
  smoke and ledge frames unchecked. No package was published.
  Determinism also refreshed six local path-package versions in its
  separate Cargo.lock from 0.0.1 to 0.1.1; this generated-only change
  was not retained.

The owner's follow-up re-recorded `stylise-pixel` and `stylise-palette`
for 8a9338b6 after inspecting both old/native pairs: only the dark band
along the grazing ground disappeared. Both browser tiers use these
shared Metal references, not duplicate tier copies. `scripts/check.sh
web-reference` then passed every comparison and harness in 339 s.
An initial main-checkout attempt used stale wasm left by its functional
lane and failed at startup; the passing run used the measure lane's
fresh wasm through `OG_GATE=1 OG_WASM_OUT=<measure>/target/og-wasm
sh packages/engine/scripts/build.sh` before the browser gate.

## Main gate, wave 12 (2026-10-21)

Wave 12 started as soon as wave 11's last stream merged (the owner's
instruction: no further question, no CI). Its streams and merges, in
order: records-11 (37ee1908, docs, fast-forwarded with the Kestrel
comments' fix 15f4e9eb), render-audio-3 (5bedf86b; rows 85 and 86),
skins-palette (38f6435d; rows 77, 78, 79 and 84), web-dogfood (ccc462d7;
the fourth dogfood round, of the web path), and perf-games (in flight).
Four of the five coding streams were stopped by the session's rate limit
for an hour and resumed from where they stopped. The round sections
below have each.

- **`scripts/check.sh functional`**: render-audio-3 (5bedf86b) passed
  in 168 s (5 of 5 steps; fx-showcase, Kestrel, topdown and Hollowbell's
  suites after it). skins-palette and web-dogfood were merged as one
  batch (6c422e09, 4b9bcac6) after the lavapipe pruning (36b326c3), and
  their lane failed twice before the engine changed: the web dogfood
  harness read `brotliCompressSync` and `constants` from `node:zlib`,
  which the package's ambient declaration lacked (fc6fefa9), and then
  the package's three typechecks, two of them `bun x` of the same
  typescript at once, raced on its bin under the gate ("Package
  typescript does not provide a binary named tsc"; they run one after
  another since 2675f9bc). A kit suite failure beside them (the scifi
  phone pack's idle cell, 0.636 % outliers) was the stale dev addon of
  a lane that stopped before its build, not a defect: the sheet passes
  at skins-palette's own head and on main once rebuilt. The first green
  lane after render-audio-3 is wave 13's (8a23392a, 94 s), which covers
  both streams with wave 13's first two merges.
- **The web findings** (web-dogfood, examples/WEB-FINDINGS.md): 0
  blockers, 7 annoyances, 5 nits; wave 13 takes them.

**Wave-end figures: pending** (measured once perf-games lands).

- **`scripts/check.sh --all`**: pending.
- **`scripts/check.sh web-reference`**: pending (owed on the browser
  tiers: og-ui `kit-skin-scifi-phone`, idle's, roguelike's and
  tower-defense's re-recorded frames, Kestrel's `chapters`).
- **Wasm**: pending.
- **Determinism**: pending (a bare og-audio world registers one more
  type since f89f1885).
- **Perf**: perf-games measures the three games on the release addon
  (its section when it lands).
- **vulkan-cpu**: pending; wave 11's second wave-end run (with records)
  is in progress and this wave's moved frames join the next.

## Main gate, wave 11 (2026-10-21, ce93d34b)

Since the wave 10 record (a91e6139, then 1ea7b182 deferring wave 10's
lavapipe pass to this wave's end), main took wave 10's last stream and
wave 11's four, each once its branch had merged main, in this order:

1. knowledge-3 (9f5084e9, its merge with main; docs, IDL doc text and the
   regenerated wiki).
2. lanes-per-profile, wave 10's last (c638a46f, recorded at a91e6139 with
   the wave 10 figures; row 71 closed). Its functional lane passed at
   a949527a after two fixes on main: dad31ce2 (`build.sh` takes a lanes or
   publish2d module the gate has not built yet as a warning under the
   functional lane's stale-module run, `OG_WASM_STALE=1`, and a stop
   otherwise) and a949527a (the handle read timing test compares warm
   against cold reads, an order of magnitude, not a bare microsecond the
   machine's load can miss).
3. nav-ai (a7cd639f).
4. physics-anim-2 (7b1973a7, then 9bc9fc75: the game's strike frame
   re-recorded on the merged engine and the flash refusal's wording in
   feel.test).
5. runtime-ui-2 (ce93d34b, a fast-forward over its merge with main
   c5f5061c, the game's title and bag frames re-recorded on the merged
   engine).

The round sections below have each.

- **`scripts/check.sh functional`**, each merge: knowledge-3 5 of 5 in
  177 s; lanes-per-profile passing at a949527a; nav-ai 5 of 5 in 160 s;
  physics-anim-2 passing in 93 s at 9bc9fc75 (after the strike frame and
  the flash wording); runtime-ui-2 at ce93d34b 5 of 5 in 169 s
  (every archetype, example and template suite passing after it but
  four archetype pad playtests, idle, rhythm, roguelike and tower-defense,
  which pressed the d-pad before A at the title and now landed on the
  second item since the first holds the focus; they press A at once since
  the commit after 15f4e9eb).
- **The Kestrel Stair's findings** (dogfood-3, wave 10): all 21 fixed on
  main, #16, #17 and #18 by wave 10, the rest by this wave's streams
  (nav-ai #1, #3, #5, #7, #8, #9; physics-anim-2 #4, #10, #11, #21;
  runtime-ui-2 #6, #13, #14, #15, #19, #20; knowledge-3 #2, #12). The
  game's workarounds for #3, #5, #6, #7, #10, #13, #14, #15, #19 and #21
  are gone in the streams' own commits, and its foes flash (#11) and
  squash in z (#4). Open: the `chapters` frame still shows the results
  screen, so #15's named cards are under no reference frame (row 84).
  `examples/kestrel-stair/FINDINGS.md` §After wave 11 has one line each.

**Wave-end figures** (the `--all` and web-reference runs at 15f4e9eb in
the measure worktree, logs not in the repository, stated here once).

- **`scripts/check.sh --all`** at 15f4e9eb: 9 of 9 in 656 s (fmt 2 s,
  clippy 17 s, wasm-build 306 s with the shipping `wasm` profile and the
  two lanes modules, test 186 s with nextest 861 passed and 12 skipped,
  wasm-opt 191 s, size-gate 37 s, drift 17 s, bun-test 156 s, determinism
  230 s).
- **`scripts/check.sh web-reference`**: passed in 377 s at 15f4e9eb,
  every frame on both tiers (og-tests `model-flash`, the Kestrel Stair's
  re-recorded frames and the ui-kit inventory and levels frames judged
  against `metal` there).
- **Wasm** (brotli -q 11, the shipping `wasm` profile) at 15f4e9eb: 3D
  lanes 3,116,018 B (74.3 % of 4 MiB, the largest download), 2D lanes
  2,948,772 B (70.3 %), 3D publish 3,077,098 B (73.4 %), 2D publish
  2,912,251 B (69.4 %); against 0e77b6b6 (3D lanes 3,095,052 B) +21 kB,
  the cost of three lifecycle members and a bytes wire type
  (runtime-ui-2), the 64-byte instance record (physics-anim-2),
  `og.ReverbZone`, `og.ai.restart` and the checks against fields.
- **Determinism**: passed in the `--all` above (230 s at 15f4e9eb):
  every build agrees at math_digest 6117185439544271997,
  bouncing_boxes_hash 15551159556835976588, physics3d_hash
  16705304060282334008, physics2d_hash 6277863875895225966, anim_hash
  6030136770684287795. The physics hashes moved by design this wave
  (`world_hash` includes the registered types: physics registers
  `og.Parent`, audio `og.ReverbZone`, and `og.Model`'s layout grew);
  physics-anim-2 did not run the tool on its branch, and nav-ai's crates
  are not in it (its host debug run matched the recorded values).
- **Perf**: not run (the perf lane).
- **vulkan-cpu**: two real runs of `scripts/wave-end.sh` (stages cross,
  prepare, gfx, ui, playtests, table). The first, at 15f4e9eb with
  `--record og-ui`: cross 264 s, gfx 184 s, ui 23 s and table 8 s passed;
  prepare failed in 2 s (the examples' generated assets, gitignored, were
  not made in the measure worktree: `music/dusk.wav` missing) and the
  playtests stage, 4,417 s, failed Hollowbell's 13 and Lantern Vale's 11
  on that and ui-kit's 14 frame suites on vulkan-cpu copies that predate
  the pixel pack's label weight and this wave's grids. The prepare stage
  makes a project's generated assets since the commit after 15f4e9eb,
  and the second run, at e7d6cb5e (the wave 11 record) with `--record
  og-ui,ui-kit,hollowbell,lantern-vale,kestrel-stair,fx-showcase`, passed
  every stage: prepare 12 s, gfx 233 s, ui 31 s, playtests 8,964 s (101
  playtests: fx-showcase 22, Hollowbell 13, Lantern Vale 11, match3 7,
  runner 8, topdown 7, ui-kit 33), table 9 s. Its table against `metal`
  (`target/evidence/wave-end/vulkan-cpu-vs-metal.tsv`, 252 frames)
  judged every copy by the reference-frame rule (36b326c3): kept where
  lavapipe differs from `metal` by design, fx-showcase 16 of 62 (fifteen
  looks presets, worst `looks-detailed-doodle` RMSE 0.0451 with 15.0 %
  outliers, and `moon` 0.0128; was 42), Hollowbell `combat` (0.0123) and
  `talk` (0.0105; was 8 of 9), og-tests 7 of 79 (`stylise-pixel` 0.0226,
  `water-underwater` 0.0225, `tilemap-lit` 0.0265, `stylise-palette`
  0.0116, `atmosphere-night` 0.0131, `atmosphere-moon` 0.0130,
  `sprites-array` 0.0126; was 11); dropped everywhere else, where every
  frame is within tolerance of `metal`: og-ui's 85 (the kit's included;
  worst `kit-desktop-widgets` 0.0086, 0.159 %; was 82 copies), Lantern
  Vale's 7 (worst `far` 0.0058; was 7), match3's 3, runner's 3 and
  topdown's 4 (all under 0.0031). Eight lavapipe captures committed under
  a scratchpad path by d08fdc93 went with them. Kestrel Stair did not run:
  the script listed only projects that already had a `vulkan-cpu/`
  directory, which the rule itself empties for a project that matches
  `metal`; it now lists every project with `references/metal/` (163
  playtests in 13 projects), and Kestrel joins wave 12's run. Rows 54, 74
  and 82 are closed.

## Main gate, wave 10 (2026-10-21, 9d220790)

Since the waves 8 and 9 record (3bb57785 and cd6b10ac, the cross-host
physics floor at 9e7e462b, the wave-end figures at e16e9cec) and the
vulkan-cpu references of their lavapipe pass (f29ffef9), main took wave
10's first three streams, each once its branch had merged main:
physics-3d (bb7c1e81, a fast-forward to its merge with main over its
work commit 876dd4ea),
cook-ui-words (a31d6cbe, then the coordinator's topdown combat playtest
at d5228a31) and ci-green (53b876f7, its head 3816e4a4, then the
coordinator's 9d220790: the workflow by hand only, roguelike's dive frame
re-recorded). The round sections below have each. The wave is not over:
dogfood-3, lanes-per-profile (row 71) and cook-determinism (row 80) are
still running.

- **`scripts/check.sh functional`**, each merge: physics-3d 5 of 5 in
  158 s, cook-ui-words 5 of 5 in 138 s, ci-green 5 of 5 in 119 s (at 9d220790, with the workflow set to dispatch only and roguelike's dive frame) (the
  coordinator fills).

**Wave-end figures** (the `--all` and web-reference runs at 0e77b6b6, the
lanes-per-profile branch with main merged, in the measure worktree; logs
not in the repository, stated here once), measured once the wave's last streams had
merged.

- **`scripts/check.sh --all`** at 0e77b6b6: 9 of 9 in 839 s (fmt 2 s,
  clippy 12 s, wasm-build 359 s with the shipping `wasm` profile and the
  two lanes modules, test 263 s with nextest 848 passed and 12 skipped,
  wasm-opt 274 s, size-gate 55 s, drift 9 s, bun-test 202 s, determinism
  308 s). Also on that tree in Chrome: `tests/web/export.ts` 51 checks
  (runner and match3, 2D games, on the 2D lanes module), `player.ts` 65,
  `wire.ts` 125, all passing.
- **`scripts/check.sh web-reference`**: passed in 411 s at 0e77b6b6, every frame on both tiers.
- **Wasm** (brotli -q 11, the shipping `wasm` profile) at 0e77b6b6, a
  lanes module per physics profile since 706833d5: 3D lanes 3,095,052 B
  (73.8 % of 4 MiB, the largest download), 2D lanes 2,928,498 B (69.8 %),
  3D publish 3,055,713 B (72.9 %), 2D publish 2,894,396 B (69.0 %);
  against the one lanes module at 9d220790 (3,472,308 B, 82.8 %) a 3D
  page on lanes downloads 377 kB less and a 2D one 544 kB less; the
  single-threaded modules byte-identical. Row 71 closed. Before that, the
  wave's text said:
  lanes-per-profile changes which modules ship (row 71).
- **Determinism**: passed in the `--all` above (308 s at 0e77b6b6): every
  build agrees, the two new per-profile lanes builds included, at
  math_digest 6117185439544271997, bouncing_boxes_hash
  15551159556835976588, physics3d_hash 12669269624230193360,
  physics2d_hash 12448362945291953010 (the tool splits the profiles since
  a06c1eac), anim_hash 6030136770684287795. Moved by design this wave: physics_hash
  947364242694612608 to 15539601638348908910 (physics-3d: the 3D
  scenario's walker stands a skin above the floor; the 2D scenario's world
  hash unchanged at 12448362945291953010); math_digest,
  bouncing_boxes_hash and anim_hash unchanged in the stream's runs.
- **Perf**: not run (the perf lane).
- **vulkan-cpu**: not run for wave 10. The first real run of `scripts/wave-end.sh` (its lavapipe stages, recording og-ui's frames) was started at a91e6139 and stopped in its cross-build stage, because it holds both gate locks for the whole run and wave 11's merges would have waited behind it; it runs at wave 11's end (row 82). Before that, the first real run of
  `scripts/wave-end.sh` (row 82). Copies that predate this wave's metal
  re-records: og-ui's `widget-set`, `kit-phone-widgets`,
  `kit-phone-shooter`, `kit-phone-racing` and the ten `kit-skin-*` frames
  abd2bf9b re-recorded (the palette round and the meter's label),
  Hollowbell's `title` and `play` (f3bf909e, the hills); the og-ui ones
  go with row 54's through the `ui` stage.

## Main gate, waves 8 and 9 (2026-10-21, 517d8c4e)

Since the wave 7 record (731a555d, with the lavapipe pass at 884ad090 and
web-reference at 5e50479f) main took, each a fast-forward of its branch
once the branch had merged main: wave 8's api-surface round two
(81bb31f0), taa-webgl2 (f3229ef8, then Lantern Vale's skin ink at 402442cc
and the record at 161bad3b) and wasm-size (bf8dcd16, its record at
84422979); the second dogfood game, Hollowbell (5cc97a90); and wave 9's
four streams from Hollowbell's findings, in this order: render-2d
(6eba0883) with knowledge-2 (5b7539dd, then the coordinator's 48d172f3),
physics-2d (08a33390, the record at 8b083f00) and ui-runtime (517d8c4e).
The round sections below have each. The waves' end is being measured at
517d8c4e in the `origin-engine-measure` worktree.

- **`scripts/check.sh functional`**, each merge: wave 8, api-surface round
  two 5 of 5 in 142 s, taa-webgl2 5 of 5 in 123 s, wasm-size 5 of 5 in
  177 s; wave 9, render-2d with knowledge-2 5 of 5 in 97 s (after the
  grammar line: the first run failed only the knowledge package's new
  drift check on `lit`), physics-2d 5 of 5 in 165 s, ui-runtime 5 of 5 in
  147 s.

**Wave-end figures** (the `--all` and web-reference runs at 517d8c4e in
the measure worktree, logs not in the repository, stated here once; the
vulkan-cpu bullet follows its pass).

- **`scripts/check.sh --all`** at 517d8c4e: 8 of 9 steps, failed at
  bun-test (fmt 1 s, clippy 14 s, wasm-build 246 s with the shipping
  `wasm` profile, test 185 s with nextest 835 passed and 12 skipped,
  wasm-opt 191 s, size-gate 48 s, drift 13 s, determinism 162 s, bun-test
  150 s). The one failure: the cross-host physics scene
  (`tests/support/physics-hash.ts`) has a kinematic paddle driven by a
  Velocity, which really sweeps since physics-2d's kinematic rule (the
  scene always said it sweeps; it never had), and a body it flung left the
  40 m floor (body 63 at y −34.5 after 600 ticks), failing the settle
  check; both hosts agreed on the hash. The functional lane had not run
  the test (its web module was stale). The floor is 200 m across since
  9e7e462b (body 63 at y 0.35 in both profiles) and the five cross-host
  tests pass against the fresh module. Nothing else failed. The last
  whole run before it: wave 7's (8 of 9 at 767ecf84, the section below).
- **`scripts/check.sh web-reference`**: passed in 373 s at 517d8c4e,
  every frame on both tiers: og-tests gfx per tier (`tilemap-lit`,
  `stylise-palette` and `stylise-retro` judged against `metal`,
  `taa-sharp` on WebGL2 too), the fx-showcase stations and the looks
  station's 37 (WebGL2's paper-craft and voxel-worlds against their own),
  the og-ui kit frames (`screen-layers` is an og-ui crate test, not a
  browser frame), third-person, Ember Vale, a11y. Row 74's browser part is
  closed by it.
- **Wasm** (brotli -q 11, the shipping `wasm` profile) at 517d8c4e:
  lanes 3,476,755 B (82.9 % of 4 MiB, the largest download), 3D publish
  3,058,677 B (72.9 %), 2D publish 2,895,647 B (69.0 %). Against
  wasm-size's figures after its merge with 161bad3b (lanes 3,464,929 B,
  82.6 %): +12 kB on lanes, +10 kB on 3D publish, +13 kB on 2D publish,
  the cost of `world.snapshot`/`restore`, the platformer's own casts, lit
  tile layers and the templates' label fields. Under 80 % stays row 71's.
- **Determinism**: passed in the `--all` above (162 s at 517d8c4e):
  every build agrees (host debug, host release with lanes off and on four
  threads, wasm32 scalar and simd128) at math_digest
  6117185439544271997, bouncing_boxes_hash 15551159556835976588,
  physics_hash 947364242694612608, anim_hash 6030136770684287795.
  Physics-2d changed the physics hash by design: 18126409214070009362 before, 947364242694612608 after
  (the platformer's own move, then `impulse` growing the component
  record; its section below); `math_digest`, `bouncing_boxes_hash` and
  `anim_hash` unchanged there.
- **Perf**: not run (the perf lane); Hollowbell's own `og polish` frame
  time is the only number of the wave, taken on a loaded machine (its
  README says so).
- **vulkan-cpu** (the og-lavapipe image, og-gfx and the node addon
  cross-built with zigbuild at 517d8c4e): every og-tests scene captured
  and judged by the reference-frame rule, applied whole this time: a copy
  kept only where lavapipe differs from `metal` by design. Kept
  (RMSE/outliers against `metal`): `atmosphere-moon` 0.0130,
  `atmosphere-night` 0.0131, `sprites-array` 0.0126,
  `sprites-shadow-terrain` 2.4 %, `stylise-pixel` 0.0226, `stylise-print`
  1.7 %, `terrain-grass` 0.7 %, `terrain-near` 1.2 %, `water-underwater`
  0.0225; added `tilemap-lit` (0.0265, 6.3 %: the lit tiles' light map
  on llvmpipe) and `stylise-palette` (0.0116, 0.55 %); dropped 50 copies
  within tolerance of `metal` (RMSE 0.0009 to 0.0067, up to 0.33 %
  outliers), `stylise-retro` (0.0027), `taa` and `taa-sharp` (0.0014)
  among them. fx-showcase's three 240-line looks presets keep copies
  (pixel-3d-hybrid 0.0103, low-resolution-3d 7.8 %,
  monochrome-dither-3d 0.0138). Hollowbell: eight of nine frames keep a
  copy (1.0 to 2.9 % outliers: the lit cave's light map and the pixel
  grid on llvmpipe), `results` judged against `metal` (0.0036). Both
  playtest suites then pass on lavapipe (looks 78 checks, frames 21
  checks). Still not run on lavapipe: og-ui's reference frames (row 54,
  and row 74's `screen-layers`, `kit-skin-pixel` and
  `kit-skin-pixel-phone`).
- **Not run here**: the phone paths of `--device`, the three.js
  comparison (row 51), the lanes wasm's physics step in Chrome (row 52),
  the native device latency tests (row 40).

## Main gate, wave 7 (2026-10-21, 767ecf84)

Since this record's last update (the closure-docs stream, merged at
4358c7f) main took, each a fast-forward of its branch once the branch had
merged main: the og-cook freshness fix (d243e461, on main), render parity
(26649d4c), physics lanes round three (1f0ecdc3, then b00639cb and
2133be1c on main), the dogfood game (3e6cf675; its findings on main at
7f88c046, corrected at 6c804499) and wave 7's five streams from those
findings, in this order: cook-vox (6232102a), api-runtime (661305ff),
reflect-string (6996a8e1), ui-text (167579c6) and render-style (e36fbf1f,
then the coordinator's 767ecf84). The round sections below have each. The
wave's end is being measured in the `origin-engine-measure` worktree.

**Wave-end figures** (the `--all` and web-reference runs at 767ecf84 in
the measure worktree, logs not in the repository, stated here once; the
web-reference and lavapipe bullets were completed after the WebGL2 TAA fix,
cf5f2b9, which the first web-reference run led to).

- **`scripts/check.sh --all`** at 767ecf84: 8 of 9 steps, failed at
  bun-test (fmt 3 s, clippy 46 s, wasm-build 554 s with the shipping
  `wasm` profile, test 309 s with nextest 805 passed and 12 skipped,
  wasm-opt 169 s, size-gate 54 s, drift 45 s, determinism 427 s,
  bun-test 203 s). The one failure was the slow reload test
  (`reload-asset.test.ts`): it asked for more than 2,000 plain-red pixels
  of the edited hull and found 1,898, the same count on main, because
  project.ogt's seed now applies to the template (api-runtime) and moves
  its frame; the test asks for a hull-sized count since 7deac5d7 and
  passes. Nothing else failed. The last whole run before it: 9 of 9 in
  886 s at efb3bdc (the section below).
- **`scripts/check.sh functional`**, each wave 7 merge: cook-vox 5 of 5 in
  219 s, api-runtime 5 of 5 in 161 s, reflect-string 5 of 5 in 227 s,
  ui-text 5 of 5 in 184 s, render-style 5 of 5 in 147 s. The lane on the
  cook fix: 5 of 5 in 117 s (the section below).
- **`scripts/check.sh web-reference`**: passed in 379 s at cf5f2b9 (the
  measure worktree, after the TAA still-camera fix): og-tests gfx 83
  scenes per tier (WebGPU 31 s, WebGL2 49 s), the fx-showcase stations
  and the looks station's 37 frames per tier (WebGL2's worst against
  `metal` neon-vector 0.0098, paper-craft and voxel-worlds against their
  own by design), the og-ui kit frames, third-person, Ember Vale, a11y 26
  of 26 per tier. The first run, at 767ecf84, had failed in 457 s on one
  thing: WebGL2's 30 ink, outline and posterize looks frames at RMSE
  0.0104 to 0.0461 against `metal` (WebGPU passed everything). A bisect
  in the browser put the same failure at 721886b7, 26649d4c, 6c804499 and
  167579c6 (the same 30 frames, equal to four decimals), and no WebGL2
  looks reference had ever existed: the looks station had never passed
  on WebGL2 on record, so this was a defect the measurement exposed, not
  a wave 7 regression. Its cause and fix are in the render-style round
  two section below.
- **Wasm** (brotli -q 11, per download, the shipping `wasm` profile) at
  767ecf84: lanes 3,496,803 B (83.4 % of 4 MiB, the largest download), 3D
  publish 3,075,241 B (73.3 %), 2D publish 2,909,811 B (69.4 %). Against
  efb3bdc (lanes 3,403,984 B, 81.2 %): +93 kB on lanes, +88 kB on 3D
  publish, +88 kB on 2D publish, the cost of cook round four, stylisation
  round four, the 52-byte instance, `og.physics.stats`, `str(N)` fields
  and the wave's runtime and renderer changes. The budget holds; the
  growth is a target to pursue, not a reason to hold a merge.
- **Determinism**: passed in the `--all` above (427 s at 767ecf84: every
  build agrees). Physics lanes round three ran
  the determinism tool's physics scenarios natively at b748cd40: physics
  18126409214070009362 with lanes off, on four and eight threads and with
  every pass split on four and eight (`tests/evidence/P4/physics-3/
  README.md` §Determinism), unchanged.
- **Perf**: pending (the perf lane). Render parity lowered the
  spheres-20k baselines (6.675 to 5.245 ms, spheres-20k-moving 7.759 to
  5.276 ms) by the perf test's method with the release og-gfx; the perf
  lane itself did not run on its branch.
- **vulkan-cpu** (the og-lavapipe image, og-gfx and the node addon
  cross-built with zigbuild at 617fb862): every og-tests scene captured
  (77) and judged against `metal` by the reference-frame rule; the copies
  settled at 9bd0064: `sprites-shadow-terrain` added (2.4 % outliers
  against `metal`), `stylise-pixel` (RMSE 0.023, 3.8 %) and
  `terrain-near` (1.2 %) re-recorded, and the stale copies of
  `stylise-bands`, `stylise-halftone`, `stylise-material`, `stylised`,
  `skinned-hooks`, `overlay` and `showcase` dropped (each within RMSE
  0.007 and 0.16 % of `metal`); the four new scenes and `materials-array`
  within tolerance of `metal` (no copy). fx-showcase's looks station: all
  37 captured, 17 copies kept and re-recorded (the pixel, dither,
  hatching, pattern and heavy-ink presets: RMSE 0.007 to 0.045, up to
  16 % outliers against `metal`), 20 dropped (within RMSE 0.008 and 0.5 %
  of `metal`); the suite then passes on lavapipe (78 checks). Lantern
  Vale: `start` keeps a copy (0.8 % outliers), the other six judged
  against `metal` (RMSE 0.002 to 0.003); the frames suite passes on
  lavapipe (17 checks). Not run on lavapipe: og-ui's reference frames
  (row 54 keeps those).
- **Not run here**: the phone paths of `--device`, the three.js
  comparison (row 51), the lanes wasm's physics step in Chrome (row 52),
  the native device latency tests (row 40).

## Main gate, 2026-10-21 (fe00f56)

Phase 4 merged its streams into main after 50eb7f7 (the round sections
below, newest first). From be179fc a merge passes `scripts/check.sh
functional` (fmt, clippy, every test with the GPU reference frames, drift,
the packages' tests on the native host), and the wasm build, its size,
determinism and the browser tiers are measured once at a wave's end
(AGENTS.md §Build and gate discipline, docs/ci.md §When to run what). The
wave's end was measured in the `origin-engine-measure` worktree at efb3bdc
(AGENTS.md's lane pool; the figures below are from that run's logs, which
the repository does not hold, so they are stated here once). Cook round
four and stylisation round four merged after it (1cdc3da) and took the
functional lane each; the full lane has not run on a tree with them.

- **`scripts/check.sh --all`**: 9 of 9 in 886 s, in the `origin-engine-measure`
  worktree at efb3bdc (the main of the wave's end, before cook round four
  and stylisation round four merged): fmt 2 s, clippy 2 s, wasm-build 407 s
  (the shipping `wasm` profile), test 251 s, wasm-opt 237 s, size-gate 72 s,
  drift 2 s, bun-test 222 s, determinism 235 s. The last whole full lanes
  before it are the rounds' own, path-aware, on their merged branches: the
  web host and tools round 9 of 9 at 4e121cd, ibl-post 9 of 9 in 899 s at
  3a19da8, cook-anim 9 of 9 in 207 s (nextest 624 passed, 9 skipped);
  physical materials, ui-text, shadows-aa and ui-shell passed every step
  but bun-test, which failed on load timeouts or a stale cook (materials',
  ui-text's and shadows-aa's then passed alone or after `og cook`;
  ui-shell's was not rerun). Their sections have the figures.
- **`scripts/check.sh functional` on main**: 5 of 5 in 117 s on the cook fix below
  (fmt 2 s, clippy 5 s, test 115 s, bun-functional 110 s, drift skipped
  unchanged).
  Since c736dea and 222ae43 it skips the tests that compare the two hosts'
  world hashes when the web module predates a crate change
  (`OG_WASM_STALE`); the full lane runs them. The lane at 4358c7f failed
  its bun step on a stale cook of the engine package's terrain fixture:
  cook round four changed `og.Terrain` (layers from texture arrays,
  e8de98d) without a `COOKER_VERSION` bump, and the incremental cook kept
  the old blob (stamp, size and hash agreed), which the loader refused
  ("holds type f447c7a2bbf6fee6, expected c0bc83477a1bc3ed"). The cook now
  reads a kept output's header against the type hash and schema version of
  its kind (`format::expected_header`) and cooks it again when they moved
  (og-cook's `a_kept_blob_whose_type_moved_is_cooked_again`).
- **`scripts/check.sh web-reference`**: passed in 372 s at efb3bdc (the
  measure worktree): og-tests gfx 73 scenes per tier (WebGPU 37 s, WebGL2
  51 s), fx-showcase's stations and the looks station's 37 frames per tier,
  og-ui kit frames (frames-desktop 22, frames-phone 22, players 1, locales
  15, six skins at 21 desktop and 21 phone each) per tier, third-person 3
  per tier, Ember Vale on both tiers, a11y 23 of 23 per tier. Earlier runs
  on record: ibl-post's on its branch passed (427 s); ui-shell's at a71c66b
  failed 36 fx-showcase frames (18 per tier, RMSE 0.0101 to 0.0149 against
  their own references, stale since the UI text round kerned the HUD) and
  passed everything else, after which 47 fx-showcase and 8 topdown browser
  frames within tolerance of `metal` were deleted (74e291c); the
  stylisation round's, after phase B, passed once the runner asked again
  for a deferred capture answered null (d08fdc9), the eight stylise scenes
  within RMSE 0.00016 of `metal` on both tiers.
- **Wasm** (brotli -q 11, per download), the shipping `wasm` profile at
  efb3bdc (the `--all` above, `target/evidence/size-gate.json`): lanes
  3,403,984 B (81.2 % of 4 MiB, the largest download), 3D publish
  2,987,050 B (71.2 %), 2D publish 2,821,474 B (67.3 %). Before
  stylisation phase B (docs/ci.md, 5e9cbcf): lanes 3,320,578 B (79.2 %;
  3,198,197 B, 76 %, at 50eb7f7), 3D publish 2,947,818 B (70.3 %), 2D
  publish 2,783,920 B (66.4 %); phase B and the rounds since cost the lanes
  module 83 kB. The path-aware lane's `wasm-gate` profile builds modules
  5.3 to 5.9 % larger. Cook round four and stylisation round four, merged
  after efb3bdc: not measured.
- **Determinism**: passed in the `--all` above (235 s at efb3bdc). The physics lanes rounds ran
  the determinism tool (host debug, host release with lanes off and on four
  threads, wasm32 scalar and simd128) and its physics scenarios natively
  with every pass split on four and eight lanes: every build agrees,
  physics 18126409214070009362, unchanged (`tests/evidence/P4/physics-lanes/
  determinism.md`, `tests/evidence/P4/physics-lanes-2/README.md`).
- **vulkan-cpu** (the og-lavapipe image, binaries cross-built with
  zigbuild): the wave-end lavapipe pass (4d6188b) kept sprites-array's own
  frame (RMSE 0.0126 against `metal`: nearest-filtered pixel sprites),
  dropped stylise-palette's (blue noise brings it within 0.0077) and judges
  stylise-ink-depth, sprites-shadow and sprites-array-fx against `metal`
  (0.0002, 0.0015, 0.0053); fx-showcase's looks frames after round two
  (7f8b010): five re-recorded as the tier's own (RMSE 0.012 to 0.014),
  watercolor-wash and painterly-3d judged against `metal` (0.0030, 0.0035);
  Ember Vale's three frames within RMSE 0.0043 of `metal` (row 47,
  493ea96). Recorded on `metal` after that pass and not yet run on
  lavapipe: owed row 54.
- **Not run here**: the phone paths of `--device`, the three.js comparison
  after the lean pipelines (row 51), the lanes wasm's physics step in
  Chrome (row 52), the native device latency tests (row 40).

## Main gate, 2026-10-20 (50eb7f7)

The rounds below (lanes, culling, og-ui tiers) and the showcase round
deferred their final gate to main after their merges; the five fix merges
that followed them (fix-showcase, fix-lanes, fix-docs, fix-culling,
fix-terrain-gl) had none of their own. This is that gate, run once on
main at 50eb7f7 after the last merge, on the M4 Pro at load averages 6 to
15 with no other build running.

- **fix-terrain-gl** (735a82d, merged 50eb7f7): the third-person
  playtest's title and dawn frames failed on WebGL2 at 78 % outliers with
  a flat white terrain. Cause: wgpu's GL backend picks the GL texture
  target from the descriptor, so a one-layer square 2D array becomes
  `TEXTURE_2D` and a multiple-of-six-layer one a cube map, and a
  `sampler2DArray` bound to either reads zero with no validation error.
  Ember Vale's 129 x 129 vale is one 128-quad tile, so its height, normal
  and control atlases had one layer each (fx-showcase's 257 x 257 valley
  has five, which is why the terrain station passed). `gl_safe_array_layers`
  in og-render allocates at least two layers and skips multiples of six;
  the cascade shadow map, the terrain atlases and the "no control" white
  texture use it, and callers still address only the layers they asked
  for. A bisect at 817b79e (before the lanes and culling rounds) failed the
  same way, so this was never a regression of those rounds.
- **`OG_GATE_JOBS=2 scripts/check.sh --all`: PASSED, 9 of 9 in 299 s.**
  fmt 1 s, clippy 5 s, drift 8 s, determinism 9 s (lanes on, 4 threads,
  1200 parallel calls, every build agrees), wasm-build 101 s, wasm-opt
  91 s (og_host_web-publish 11,808,230 B, -lanes 11,861,764 B after
  `-Oz`), test 106 s (nextest: 553 passed, 9 skipped), size-gate 52 s
  (largest download 3,198,197 B brotli, 76 % of 4,194,304), bun-test 89 s
  (110 engine tests in 28 files, 19 knowledge tests in 2 files, 0 failed).
  No timeouts: the bun-test flakes the earlier rounds hit were load, not
  code.
- **`scripts/check.sh --only web-reference`: PASSED in 250 s.**
  fx-showcase 23 stations on webgpu and webgl2 (46 captures), E5 stations
  18 of 18 on both tiers, og-tests gfx 53 scenes on webgpu (21.9 s) and 53
  on webgl2 (32.5 s), og-ui kit frames 16 of 16 playtests on each tier,
  and third-person Ember Vale title, beacon and dawn on webgpu (RMSE
  0.0013, 0.0048, 0.0007) and webgl2 (0.0043, 0.0004 against its own
  beacon reference, 0.0018). Every other tier falls back to the metal
  references; none was re-recorded.
- **Not run here**: the vulkan-cpu (lavapipe container) references, the
  phone paths of `--device`, the Three.js comparison and the native
  device latency tests remain as the rounds left them (row 47 and the
  culling round's terrain-grass note still stand; the note is resolved in
  the P4 perf round).

## Wave 12, perf-games: the three games timed by system, and the engine's share of their frames cut (2b896be5)

Wave 12's fifth stream measured the three dogfood games on the release
addon (lane-4) and fixed what the profiles showed, one commit each,
with the three screenshots and the world hash per game identical
before and after. The machine never went under load 8 for a whole run
(the lavapipe VM at 15 to 27 until 23:10, then 7 to 16 with the gate and
an iOS simulator on it), so the figures below are from the second
window, before and after interleaved in 6 unpaced rounds and 3 paced at
60 Hz; "before" is main plus only the measuring commit. Method and
breakdowns in `tests/evidence/P4/perf-games/` (a README and one file
per game).

- **Timing by system (2b61462a)**: every system is timed by name; `og run
  --json` gains `frames` (step and render median, p95, max) and `systems`
  (each system's mean ms), and the probe's dump fills in its per-system
  `ms` (og-game §14).
- **Cloth and rope (465ab16d)** stop searching the level mesh beyond the
  point's radius: Kestrel's og.physics 0.98 → 0.49 ms release, 1.19 →
  0.49 dev, 3.67 → 0.93 with the lanes off.
- **`og.i18n.t` (381e3d59)** no longer copies every string table per
  call (7 to 9 % of the frame; the `Ui` system now costs about half).
- **Sprites, flipbooks, afterimages and tile layers (2c35ae03)** are
  read in place instead of decoded to JSON each frame: Hollowbell's
  share 16.6 → 5.2 % release, 25.8 → 15.9 % dev; a test checks every
  engine record's fields read the same as the JSON.
- **Record calls (f3ad73cb)** stop copying their layout and their
  answer; **the UI theme's JSON (3b2d1c21)** is built once while the
  theme is unchanged; **system match counts (0b87fc55)** are recounted
  only after a structural change; **drop shadows over terrain
  (b2d8ef7d)** sample heights only (Lantern Vale 6.6 → 3.2 % of a dev
  frame).

| Game | Addon | step median | step p95 | render median | render p95 (unpaced; paced) |
|---|---|---|---|---|---|
| Hollowbell | release | 0.66 → 0.53 | 1.36 → 1.09 | 0.70 → 0.60 | 1.15 → 1.42; 1.36 → 0.89 |
| Hollowbell | dev | 3.06 → 2.06 | 4.58 → 3.70 | 2.92 → 1.90 | 3.61 → 2.60 |
| Kestrel Stair | release | 1.98 → 1.28 | 2.90 → 2.31 | 1.21 → 1.50 | 2.63 → 4.40; 1.71 → 1.26 |
| Kestrel Stair | dev | 5.29 → 2.98 | 8.85 → 4.00 | 2.24 → 1.98 | 3.69 → 2.50 |
| Lantern Vale | release | 0.90 → 0.71 | 1.88 → 1.78 | 2.14 → 3.71 | 7.17 → 7.79; 1.48 → 1.60 |
| Lantern Vale | dev | 2.77 → 2.13 | 4.55 → 4.98 (paced 3.90 → 2.78) | 2.49 → 2.27 | 3.29 → 3.71 |

All three meet the targets at that load; `og polish` scores frame time
100 on release for all three, on dev Hollowbell 0 → 91 (the "before"
run hit a load spike), Kestrel 81 → 100, Lantern Vale 82 → 85. Unpaced
render includes the wait for the GPU (Metal command buffers), which is
why some unpaced "after" renders rose while the paced ones fell. Under
load 20 to 25 with the lanes on, Kestrel's release step p95 was 17.7
ms, 4.45 with `OG_LANES=off`. Each game's README gained a frame-time
paragraph with game-side advice (Hollowbell: the `Ui` system; Kestrel:
per-entity `world.get`/`world.set` and the banners' cloth iterations;
Lantern Vale: bloom about half its GPU frame, auto exposure next, and
the per-entity calls in `Slimes` and `Hero`). Tried and dropped:
remembering a sprite's drop-shadow fit while nothing moves (Lantern
Vale's shadowed sprites move every frame); an `Arc<Layout>` map (25
sites; sharing the fields inside `Layout` did the same). Owed: rows 89
and 90.

## Wave 13, kit-2d: the platformer HUD draws what it is given, the level select has a way out, names a reader can read, HUD groups clear the shell's control (34dd34d3)

Brasswick's UI, shell and platform-skill findings (lane-4): 4, 10, 11,
12, 15 and 16.

- **#4.** The left capsule and the `Score` capsule were drawn
  unconditionally with `?? 0` defaults; `PlatformerHudDesc.coins` and
  `score` are optional (left out, no capsule), `maxLives: 0` with no
  `health` draws no hearts capsule, an empty left group is dropped; the
  kit template sets all three, so no kit frame moves (a game that
  relied on the always-drawn `0` now draws none).
  `tests/ui-platformer.test.ts`, two of three failing on the old kit.
- **#10.** The screen was a modal with `ignoreBackdrop: true` and no
  control of its own; a `Footer/Back` button (`LevelSelectDesc.backLabel`)
  and a tap on the scrim close it, answering `closed` as Escape and B
  do (`tests/ui-levels.test.ts`). Shop, inventory and leaderboard share
  the helper and the gap, left so their frames stay.
- **#11, #12.** A card's name was its text plus each icon's atlas name
  in English, and tab rows, radio groups and selects fell back to their
  id; `IconDesc.label` and `ImageDesc.label` (an empty string marks
  decoration), a `label` on `TabsDesc`, `RadioDesc` and `SelectDesc`; a
  card is named by its level plus `starsLabel` ("{stars} of {max}
  stars"), a locked one by `lockedLabel`; `SettingsDesc.sectionLabel` and
  `languagesLabel`, and `qualityLabel` now names the presets too (they
  were "Quality" in English, unreported). og-ui
  `groups_and_icons_take_the_names_their_descs_give` and ui-levels'
  `dump_ui` in Spanish; the og-i18n skill lists the fields. Brasswick
  takes the strings in its tables and its locales playtest now asserts
  Apartado, Idiomas, "1. La escalera de la caldera 0 de 3 estrellas" and
  Volver.
- **#15.** Since racing-ui every kit HUD is anchored groups and already
  kept out of the shell's control (Hollowbell's `Status` and Brasswick's
  HUD too; the platformer, idle and racing tests pass on main and
  would fail on 21b0b0cc, the engine Brasswick ran on). What remained:
  on a phone, moving a group right to clear the control pushed it onto
  its neighbour (the shooter's vitals onto its kills and pause, 107 to
  1,147 px² of overlap; the puzzle's level/moves onto its score, 3,540
  to 6,970). Anchored groups are placed together before the other
  layers, and a corner group steps down past the control when moving
  along the edge would overlap more of the other groups or of the area
  outside the safe area; only with `covered` set, so no frame moves
  (`a_corner_group_steps_down_past_the_control_when_moving_along_lands_on_another`;
  `tests/ui-hud-corner.test.ts`, one test per HUD on desktop, phone
  portrait and landscape). Not fixed: the puzzle HUD on a 390 px phone
  overlaps its own score group even without the control.
- **#16.** og-platform-sdk §3 says systems run in `GameDesc.systems`
  order, so the answers are applied in or before the system that
  builds the title (list the answers system first, or read the answers
  at the top of the UI system).

Frames that move with the level select's Back button, re-recorded by
the coordinator at the merge: og-ui `kit-desktop-levels`,
`kit-phone-levels` and the twelve skin sheets, Kestrel's `chapters`,
Brasswick's `levels`.

## Wave 13, dogfood-6: Brasswick, a 2D puzzle-platformer in a lamplit clockworks (4f14111a)

The sixth dogfood round (lane-1): a fifth game, 2D and tile-based,
written from the knowledge package alone, in `examples/brasswick`.
Three tilemap levels in the engine's own map format, generated by
`tools/make-assets.ts` from `tools/levels.ts` and placed side by side
in one world, with parallax layers, 2D lighting and lights; a
character that runs, jumps with coyote time, wall-slides and
wall-jumps; pushable crates, pressure plates, doors, a gate on a 2D
slider joint, a lift and one-way grates; stompable enemies; gears to
collect; checkpoint lamps that save the whole world through
`og.platform` and a progress record; the title, level select, pause,
settings and results screens; English and Spanish; music and sound;
keyboard and pad. 8 playtests with 147 checks (a keys-only replay of
level 1 in two runs to the same hash, a save across a relaunch, the
Spanish locale) and 9 metal frames; all green before the merge (the
lane's addon predates four physics bindings, so the suite reruns on
main). The published build on WebGPU through dogfood.ts: first frame
290 ms, 60 ticks/s, a clean console, the saved language back after a
reload and behind the play gate.

**Findings (examples/brasswick/FINDINGS.md): 16, 0 blockers, 9
annoyances, 7 nits.** Runtime and channel: #6 `og.PlatformerController`
shoves any dynamic body at its full run speed whatever its mass, its
doc does not say it pushes and it has no `pushBodies`; #13 `og run`
ends at the first tick past the script's last step when a restore
moves the clock forward, the remaining steps never run; #8 nothing
marks the tick after a restore and every place in the world rolls
back; #5 a map's collision layers all take one `colliderLayers`; #3 a
one-shot emitter spawned ahead of its use fires at tick 0; #7 the slot
answers carry `time` 0. UI and shell: #4 `platformerHud` draws hearts
and a score a game does not have; #10 `levelSelect` has no way out for
a pointer; #11 a level card's accessible name reads its stars in
English; #12 the settings template names two groups in English; #15 the
shell's Back handle covers the HUD's top-left (Hollowbell's first heart
too); #9 `og.ui.back()` before the tick's dialogs took their Back (fixed
on main by racing-ui, the same cause as Meridian Loop's #9). Docs and
the scaffold: #1 `create` has no 2D game; #2 the flipbook grammar is
not in the package; #14 og-feel's jump sound "on the same tick" where
the event arrives the next; #16 an answer applied by a system after the
title's shows the defaults behind the gate. Not done: no `og polish`,
no WebGL2 or browser frames, no keys-only playthrough of level 3. The
fixes are the platformer-2d and kit-2d streams.

## Wave 13, racing-ui: the racing HUD in the game's words, Back under a dialog, a time as a result, texture repeat, the outline on a large ground, the shell's control kept clear (3189ae6a)

Meridian Loop's UI, render, shell and knowledge findings (lane-3): 8, 9,
10, 13, 15, 7, 18 and 17.

- **#8.** The HUD's words were hard-coded English, unlike the other kit
  screens; `RacingHudDesc` gains `lapLabel`, `bestLabel`, `pauseLabel`,
  `boostLabel`, `brakeLabel`, `leftLabel`, `rightLabel`, and for a time
  trial `delta`, `splits` (label, time, delta signed with a plain `-`),
  `racers: 0` to hide the place and `boostMeter: false`; the defaults
  draw exactly as before. Test `tests/ui-race.test.ts` (a German HUD
  with no English left, the delta and splits, no place or meter).
- **#9.** The HUD asked for Back before the pause dialog was built that
  tick; `og.ui.back()` now answers false while an unlocked dialog or
  menu was on top last frame (a locked dialog lets Back through). Test
  `a_hud_never_takes_back_while_a_dialog_is_open`. A game that draws its
  HUD under pause (Kestrel, the kit's pause screen) now closes pause on
  Escape: their suites run at the merge.
- **#10.** `ResultsDesc.format` (`points` or `time`) and `scoreLabel`:
  with `time` the score counts up as m:ss.mmm, the lower time is the
  best, a best of 0 shows no best line (the results case of
  ui-race.test.ts: "1:36.233", "Bestzeit 1:35.100").
- **#13.** Render warnings were logged with tick 0 and one flag allowed
  one warning per process; each missing character is named once, on
  the tick it is first drawn, with the font's asset and its charset
  (inferred from its glyphs; the charset table moved to
  `og_cook::format::charset`, one copy); every render warning carries
  its real tick (`tests/missing_glyphs.rs`).
- **#15.** A geometry option, not a material one: box, plane and sphere
  descs gain `uvRepeat` (the material sampler already wraps; a material
  `repeat` would have been a second path through every lit and shadow
  shader). New og-tests scene `ground-grid` (a 1,600 m plane, a grid
  line every 10 m under posts every 10 m; `uvRepeat` 1 draws no lines).
- **#7.** The outline pass compared a neighbour's depth with the pixel's
  own, so a ground at a grazing angle read as edge everywhere; when the
  neighbour and the opposite pixel are the same object facing the same
  way, the test is against where that plane continues (capped at twice
  the depth), and the distance term grows a line to at most 4× its
  width (fxpost.wgsl, pbr_prepass.wgsl). New scene
  `stylise-outline-ground` (`a_large_ground_keeps_its_outline_only_where_it_ends`;
  the old shaders turn half the frame cyan). Re-recorded, each against
  main's shaders on the same tree: fx-showcase
  `looks-bold-comic-illustration` (457 pixels, RMSE 0.0044: a box's top
  face seen nearly edge-on is no longer one dark band),
  `looks-rubber-hose-animation` (237, 0.0082, the same), `post` (160,
  0.0035: the ground's far edge a one-pixel line instead of 3 to 4
  rows), og-tests `stylise-outline-voxels` (138, 0.0044: lines across
  voxel faces seen edge-on gone). `looks-neon-vector` was re-recorded
  once on a stale addon and put back: the change does not move it.
- **#18.** New `Viewport.covered`: the shell measures its Back control
  (the round, icon-only size) and passes it through `Shell.resize`, and
  og-ui moves any anchored group that would overlap it along its edge,
  so the racing HUD's place and lap sit to its right
  (`anchored_groups_keep_out_of_the_host_control`). Untested in a
  browser; the control's name beside it for the first 4 s is not kept
  clear.
- **#17.** og-core §2 gains "Recording, replaying and ghosts" on Meridian
  Loop's code, og-game §17 points to it.

Not done: the kit's string tables for the racing HUD's words (its CJK
subsets would be re-made); Meridian Loop does not use the new fields
yet; the other HUDs' fixed words ("Pause", "Jump", "kills", "moves
left", "Target").

## Wave 13, vehicle-physics: wheel rays on the chassis's layers, forces and a centre of mass, light snapshots, a mid-tick save that completes its tick (e9eae253)

Meridian Loop's physics and API findings (lane-5): 3 (the blocker), 2,
1, 16, 11, 12, 4, 5, 6 and 14.

- **#3.** The wheel-ray filter in vehicle3.rs had no collision groups;
  the rays now use the chassis collider's layers and mask. Test
  `cars_on_layers_that_do_not_meet_pass_through_each_other` (the second
  car sat at y 2.05 on the first's roof; both at 0.875 on the road now;
  a control case keeps cars on meeting layers stacked).
- **#2.** Four one-shot calls on the pattern of `og.camera.shake`:
  `og.physics.applyForce` and `applyImpulse` (an optional world point),
  `applyTorque`, `applyTorqueImpulse`, queued and applied by the tick's
  physics step in call order (a force lasts one step, an impulse applies
  at once, a hit stop drops forces and keeps impulses, pending pushes
  are in snapshots and the hash, refused without an `og.RigidBody`);
  `og.RigidBody.centerOfMass` (an offset, the inertia the shape's);
  rapier's raycast car copied into og-physics
  (`raycast_vehicle.rs`, Apache-2.0, unchanged but for making roll
  influence and skid info public) so `og.Vehicle.rollInfluence` exposes
  the controller's own roll setting, default 0 (rapier's 0.1 flips the
  default car at full lock above about 20 m/s). Tests: forces.rs (6,
  the 2D profile, the hit stop, the snapshot, a weighted box righting
  itself) and a vehicles test (0.1 tips the car, 0 or a lowered centre
  keeps it upright).
- **#1.** Defaults for a 1,000 kg car: `engineForce` 2,500 N per driven
  wheel, `maxSpeed` 40, `brakeForce` 2,000 (now newtons per wheel,
  applied as force × the tick, where it was rapier's per-step impulse
  cap), `wheelMass` 20; 4.7 m/s after 1 s, about 8 m/s² braking. og-feel
  §12 gains a vehicle table (road car, arcade racer, truck, kart) and
  tells authors to set `og.RigidBody.mass`. The reported `maxSpeed`
  overshoot did not reproduce (20.00 for 20 on flat ground).
- **#16.** `WheelPose` gains `slip` (1 − rapier's skid info) and
  `sideSpeed`; `og.Vehicle` a written `slip`, the largest of its
  wheels'; 0 in 2D.
- **#11.** The physics save wrote the whole rapier world, triangles
  included; it now cuts out the geometry of mesh, convex-hull and
  height-field colliders and stores each one's handle and scale, the
  decode rebuilding them from the registered asset (a save naming an
  unregistered mesh is refused, naming it). saves.rs: a 7,200-triangle
  ground with a hull and a height field, 630,593 bytes before, 12,239
  now, the same hash and future after the restore.
- **#12.** The restore completes the tick: a snapshot taken inside a
  system names that system (snapshot format 4), and after restoring it
  the next step runs the systems after that one, physics included, with
  the tick's own events, then ends the tick (deferring the snapshot was
  impossible, `snapshot()` returns its text at once). Take the snapshot
  as the system's last write: anything after the call is not in the
  save (`world.snapshot`/`restore`, og-core §2, og-game §17). Tests:
  og-ecs snapshot.rs (completion and refusal) and
  `crates/og-runtime/tests/mid_tick_save.rs` (saved in a system at tick
  8, restored at tick 3, the same hash at tick 30, `applyImpulse`
  through the host ABI). Not checked: on the frame that completes a
  restored tick, UI drawn by systems before the saver is probably
  missing for that one frame.
- **#4, #5, #6 (docs).** `str` holds at most 4,096 bytes and there is no
  array field (numbered fields, an entity per item, or chunked text);
  handles go in their own field kinds (`Asset()`, `Texture()`,
  `Sound()`, `Entity()`), never `u32`; a system's query requires every
  component of its reads and writes, and `world.query` stays refused
  inside a system by design (a system reaches what it declared). IDL
  docs and og-core §4.
- **#14.** `og.Model.visible` (default true); the renderer skips every
  draw of a hidden model, static or animated
  (`a_hidden_model_draws_nothing_posed_or_not`).

After the merge: the layouts of `og.RigidBody`, `og.Vehicle` and
`og.Model` changed, so an existing save holding them needs `{ migrate:
true }`; the physics save format changed, so physics worlds' hashes
differ from earlier recorded ones and an older engine refuses a
format-4 snapshot; `tests/save.test.ts` was edited to the mid-tick
behaviour; Meridian Loop pins `rollInfluence: 0.1` and `brakeForce`
156,000 N to keep its tuned handling (not bit-exact: its suite, its
`LAP1 = 2046` in ghost.og.ts and its frames are checked at the merge).

## Wave 13, ui-five: text editing by grapheme, bounds in the a11y tree, miter joins, the kit on the icon set (e6c445bc)

Rows 59, 60 and 61, on lane-2.

- **Row 59, text editing.** The caret moved by one char and stopped
  between a base and its accent; arrows, Backspace, Delete, a press and
  line wrapping now step over a whole grapheme cluster (a base and its
  marks, an emoji with its tone, a ZWJ family). ← and → were logical
  everywhere; in a right-to-left run they move the way they point, for
  single steps and word moves, and a selection collapses to the arrow's
  side. Under mac keys only Ctrl+A/E worked; Ctrl+F, B, N, P, D, H, K,
  U and W now do as on a Mac and nothing off one (`UiKey` gains them,
  plus `=`, `-`, `0` and their keypad versions). The native macOS host
  edits with mac keys (a script holding Ctrl or Meta edits differently
  on a Mac than on Linux unless it pins `macKeys` through
  `og.ui.previewViewport`; the IDL, README and skill say so). A mixed
  text's runs share one baseline centred on the extents of every face
  the line uses (labels, fields, text areas, rich spans). A `[wave]`,
  `[shake]` or `[pulse]` span that mixes faces, runs right to left or is
  shaped animates glyph by glyph, a shaped run's joined letters moving
  together. Tests failing on the old code: fields.rs
  `the_caret_and_backspace_step_by_grapheme_clusters`,
  `arrows_move_as_they_point_in_right_to_left_runs`,
  `mac_keys_honour_the_emacs_keys`; runs.rs
  `a_mixed_labels_runs_share_a_baseline_that_holds_every_face`,
  `a_moving_span_that_mixes_faces_moves`; og-runtime
  `a_native_host_edits_with_the_platforms_keys`. Frame moved:
  `text-mixed-runs` (its Arabic lines up 1 px, the Arabic face's deeper
  descender). Not done: an emacs kill buffer (Ctrl+Y, Ctrl+T) and fully
  visual caret movement across a bidi run's edge (the README's owed
  list).
- **Row 60, accessibility.** The tree carried no number bounds and the
  mirror guessed 0 to 100 for values ending in `%`. `A11yNode` gains
  `valueMin`, `valueMax` and `valueNow` (null for none): a slider its
  range, a bounded stepper its bounds, an unbounded one its value, a
  meter 0 to max, a progress 0 to 100; the web mirror writes
  `aria-valuemin`, `aria-valuemax` and `aria-valuenow`. Tests:
  `ranges_carry_their_bounds_and_value` (a11y.rs) and
  `tests/web-a11y-range.test.ts` (its bounds test fails on the old
  mirror; its reader-adjust test drives the range's `input` and
  `change`; the stand-in DOM now bubbles events).
- **Row 61, materials and input.** Every icon stroke was distance
  minus half its width, so joins came out round; a path's
  `stroke-linejoin` is honoured (miter to `stroke-miterlimit`, bevel,
  round) and a set declares a default with a `join` line (og-cook
  icons.rs; three tests fail on the old code). The kit's screens draw
  from the icon set (`iconoir.ogt` declares `join round`, gains 8
  iconoir SVGs and the kit's 17 names, 81 entries; hammer, hexagon,
  bowling-ball and emoji-talking-angry stand in for sword, gem, bomb
  and skull); style packs keep their own icons; the materials frame is
  byte-identical. A two-finger scroll never kept its velocity and now
  flings on lift, and a float rounding at the scroll clamp that ended
  every fling early is fixed; the zoom view takes focus and `+`/`=`,
  `-`, `0` and the arrows (tests `two_fingers_lifting_off_a_list_fling_it`,
  `a_focused_zoom_view_takes_keys`). Frame moved: `materials` (its quest
  list flings on to Quest 4 to 6). The glass blur pyramid is left
  alone (optimisation). The kit's desktop and phone frames on the
  default skin move with the icon set and are re-recorded by the
  coordinator at the merge (`OG_UI_CAPTURE=1 og test`), with the cooker
  version bumped to r5 so a user's icon set re-cooks with its joins.

## Wave 13, templates-title: the archetypes' opening titles drawn finished behind the gate (991f6435)

The web-boot-2 pattern applied to the templates (lane-4): idle, rhythm,
roguelike, third-person and tower-defense (the five whose title is a
`mainMenu`) gain `leftTitle` on their `Game` resource and pass
`instant: !g.leftTitle` (idle's save writes named fields, so the flag
is not saved). No frame moved: the five record their title between
0.5 s and 1.0 s, after the entrance, so `title.png` is byte-identical
in all five (roguelike's `dive` at 0.048 % and third-person's three at
RMSE ≤ 0.002 differ the same with the change removed: the engine's).
Tower-defense's title at tick 3 used to be the panel popping in with
only "Path Guard" drawn and is now the whole title. Not changed:
match3, runner and topdown draw their titles as text and sprites with
no panel; the kit opens on its gallery (default skin, no entrance);
`templates/game`'s menu is a plain column. The og-ui skill's template
table gained `instant` on its title row.

## Wave 13, web-boot-2: the opening title drawn finished behind the play gate (b5350d48)

The web-boot stream's leftover (lane-5), cut to this one item on
2026-10-22 (the pipeline warm-up after the tap dropped as optimisation).
Kestrel plays no fade of its own: its skin's `motion { enter slideUp;
stagger 40 }` slides the title panel in and staggers its buttons from
tick 1 over about 30 ticks, and the gate frame is a still from ticks 2
to 9, so the panel was frozen part-way; Hollowbell (`enter slideDown`)
and Lantern Vale (`enter scale`, stagger 35) do the same, and no API
let a game turn a panel's skin entrance off (`og.ui.transition`'s
`instant` does not reach it). New `instant` field on `AnchorDesc` and
`MainMenuDesc`: the panel shows finished the first time it is built, no
entrance, no stagger, its buttons usable at once, and only on that
first build (a panel that comes back enters normally); og-ui
`open_anchor`, og-runtime's mapping, the kit's `mainMenu`; the three
games pass `instant: !g.leftTitle`, so only the opening title is
affected. Kestrel's tick 1 now shows the whole settled title; the
games' 25 metal frames are byte-identical. Docs: og-platform-sdk §3
says what the frame behind the gate is and how a title is built, og-ui
§6 and web-export §The shell point to it. Test: og-ui
`an_instant_panel_is_settled_and_answering_on_its_first_tick_and_enters_when_it_comes_back`
(skin.rs; fails with `instant` ignored). One WebGPU run made before the
scope cut reached the stream: Kestrel's capture behind the gate went
from RMSE 0.119 to 0.025 against the metal title, the rest being the
title's slow camera between tick 1 and the reference's tick 72. Not
done: the archetype templates and the kit build their titles with
`mainMenu` too and still play the entrance behind the gate (several
record their title at 0.3 s, so switching them moves references);
Lantern Vale's first ticks look foggier and brighter while the exposure
settles (the engine's adaptation, not the game's).

## Wave 13, dogfood-5: Meridian Loop, a night time trial on a neon circuit (7b927dd9)

The fifth dogfood round (lane-3): a fourth game of a genre the three
did not cover, written from the knowledge package alone, in
`examples/meridian-loop`. A generated 956 m loop with banked bends and a
jump over a gap; a car on `og.Vehicle` with keyboard and pad; a lap
timer with two sector splits, each with its delta against the record;
the best lap's ghost replayed from recorded poses; a watch-replay that
saves the world during the countdown, restores it and feeds the
recorded inputs, reproducing the race tick for tick (laps of 2,046,
1,859 and 1,841 ticks matched); the record, its ghost, the settings and
a leaderboard through `og.platform`; English and German; engine audio
pitched by the revs, a tyre squeal, smoke and skid marks; the title,
pause, settings, results and leaderboard screens; every asset from
`tools/make-assets.ts`. 10 playtests and 14 metal frames (10 from
frames.og.ts, 4 from published.og.ts); 9 of 10 passed on the last full
run before the merge, `replay`'s lap count failing because its script
ended at 90 s before the second lap (moved to 110 s and rerun alone:
two laps, a 1,710-tick record; the suite reruns on main after the
merge). One WebGPU play through dogfood.ts (its play plan committed):
boot download 5,288,182 B brotli over 58 files (the engine 3.16 MB,
cooked 1.70 MB, one music chunk), first frame 696 ms with lanes on, 60
ticks/s, step p95 2.65 ms, the language back after a reload, the end
report on Back, no console error.

**Findings (examples/meridian-loop/FINDINGS.md): 18, 1 blocker, 10
annoyances, 7 nits.** The blocker, #3: `og.Vehicle`'s wheel rays land on
any collider below them whatever its `layers` and `mask`, so two cars
that must not touch stand on each other (the ghost is poses, not a
second car). Physics and API: #2 no force, torque, impulse or
centre-of-mass control, so the car rolled over at moderate lock; #1
og-feel §12 has no vehicle row and the vehicle's defaults drive a
chassis of a few kilograms; #16 WheelPose has no slip; #11
`world.snapshot()` carries the level's triangle-mesh colliders (400 KB
of the 512 KiB slot); #12 a snapshot taken inside a system and restored
skips the rest of that tick, undocumented, and a replay from it
diverges; #6 a system's query is every component of its reads and
writes and `world.query` is refused inside a system, neither said; #4 no
array field and `str` at most 4,096 bytes, unsaid; #5 which column holds
an asset handle, unsaid; #14 `og.Model` has no `visible`; #15 no
texture repeat. UI: #8 `racingHud`'s words are English and fixed with no
place for a time trial's numbers; #9 it takes Back under an open dialog,
so Escape never closes pause over it; #10 `resultsScreen`'s score is a
whole number counted up, so a time cannot be the headline; #13 the
missing-glyph warning always says tick 0, once a process; #7 the
neon-vector outline washes a large ground plane; #18 the shell's Back
button covers the HUD's top-left on the published page; #17 nothing in
the package says how a game records and replays its play or makes a
ghost. The fixes are the next streams.

## Wave 13, vox-stylise: `.vox` key frames read, stylise's footprint along the depth (ee07a3c1)

Rows 63 and 55, on lane-4.

- **Row 55, stylise in surface space (725b9a4e).** The footprint that
  picks a pattern's octave and the canvas level was one pixel's width
  across the view ray; on a floor seen at a low angle the finer
  octave's marks were smaller than a pixel along the depth, aliased, and
  the aliasing changed as one octave faded into the next: the moiré arcs
  on `stylise-halftone-surface`'s far floor. stylise.wgsl now measures
  the footprint along the depth gradient (the longer of the steps to the
  neighbouring pixels across and down, each rebuilt at its own depth,
  a silhouette on one side ignored, capped at 32× the width across the
  ray), and dot edges and stroke widths follow the two screen-space
  steps on each projection plane, so marks stay a pixel wide and crisp
  (object space uses the instance's own steps, an even footprint at
  instance edges). Re-recorded on metal: `stylise-halftone-surface`
  (RMSE 0.078 from the old shader: the arcs gone, the dots in even
  receding rows that flatten to wide ellipses towards the horizon, the
  tone smooth, the dots near sphere rims a little coarser),
  `stylise-surface` (0.035) and `stylise-object` (0.044): the hatched
  floor shadows are legible crossed strokes instead of a dense grey and
  the far canvas grain is calmer. Every other og-tests scene renders
  exactly as on main's shader (checked before the scope correction);
  after the merge, fx-showcase's looks suite moved one frame,
  `looks-inked-comic-3d` (its floor hatched in surface space: RMSE 0.047,
  4.98 % outliers, the far floor's moiré gone), re-recorded on metal by
  the coordinator; the other 36 looks frames are unmoved. Row 55 is
  closed.
- **Row 63, `.vox` animation (431d9da1, 900158dc).** The importer read
  only the first of an `nTRN`'s per-frame dicts, so a part moving
  across frames stayed put. og-cook now keeps every key sorted by `_f`
  and held until the next (MagicaVoxel's sampling, as ogt_vox does it;
  not checked against a file the editor wrote), walks the scene graph
  once per frame up to the last key, splits each part into runs of one
  model at one rotation (a rotation change is its own mesh, swapped in
  like a frame's model), gives a moving part a STEP translation channel
  on its joint beside the scale channel (a hidden frame holds the nearest
  shown frame's position), takes `voxOrigin bottom` from the lowest
  voxel over every frame, and leaves a file static when its keys change
  nothing: all seven `.vox` files in the repo convert to byte-identical
  glTF. Tests: two unit tests, the cook test
  `a_keyed_transform_cooks_to_a_moving_joint` (fails on the old importer:
  the keyed file cooked as static), and the og-tests scene
  `vox-keyframes` (it writes a `.vox`, cooks it, samples the clip with
  og-anim and skins the part as og-runtime does; og-tests gains og-cook
  and og-anim as native-only dependencies, the scene in a `NATIVE_SCENES`
  list that `og-gfx --all` includes and the browser set leaves out).
  Its frame on metal: two copies of a grey slab with a blue post; on
  the left (frame 0) the red block rests on the slab left of the post,
  on the right (frame 2) it is lifted clear of the slab right of the
  post, its shadow on the slab. Not done: a visibility track so a folded
  frame costs no joint and no draw (a new channel type through og-anim's
  sampling, blending, graph layers and retargeting, a glTF extension, a
  schema bump and the runtime skipping hidden draws), a cost matter
  deferred with the optimisation rows; `_loop` and MagicaVoxel's
  playback range are still not read (`voxFps` and `voxClip` stand in).

## Wave 13, no-renderer: a playtest without a renderer passes on its other checks and says so (71c0e9dd)

The engine side of the CPU sandbox decision (lane-1): without a GPU
adapter, every project's `og test` passes and each screenshot is
reported "no renderer, frame unchecked"; the world hashes match a run
with a renderer.

- **Before.** Renderer creation never panicked (the addon returned an
  error; the `.expect("a headless adapter")` calls are only in the
  engine's own GPU suites, which need a GPU by design), but nothing
  could deny the renderer for a test (`WGPU_BACKEND` was unread), `og
  run --headless` exited 0 with each screenshot a WARN, `og test`
  exited 1 (match3 failed 3 of 7: "ran without warnings" and "the run
  wrote no dealt.png"), the UI laid out on 1280×720 whatever size the
  playtest asked for (a non-default size could diverge), `og polish`
  scored frame time from the step alone, and `og doctor` counted no
  renderer as a problem and suggested lavapipe. The probe's `hello`
  already said `none` and `capture` already answered null.
- **Now.** `OG_RENDERER=none` (og-render; docs/ci.md "Switches in the
  environment") makes creation report "no GPU adapter" as a machine
  without one does; the screenshot step writes no PNG and logs an info
  line (og-probe); the play surface keeps the requested size with no
  renderer (og-host-node); `og run --json` lists the skipped frames in
  `unchecked`; playtests gain `t.frame(files, check)`, whose checks run
  only with a renderer and otherwise print `~ <file>: no renderer, frame
  unchecked` with `run.rendered` and `run.unchecked` beside it (a frame
  read outside `t.frame` ends the check function and passes with a note);
  `og test`'s summary adds "N frame(s) unchecked" with the exit status
  unaffected; `og polish` says "no renderer"; `og doctor` reports the
  tier as `none`. Every frame check of the templates and examples moved
  inside `t.frame` (the shared `checkFrames` helpers, ui-kit's skins.ts,
  fx-showcase's station.ts); the e2b addon script no longer expects a
  frame. Docs: the og-build and og-game skills, web-export, the og-probe
  and og-render READMEs, the IDL text regenerated.
- **Tests.** `packages/engine/tests/no-renderer.test.ts` (fails on the
  old code: "2 of 3 suite(s) failed") and og-runtime's
  `tests/no_renderer.rs` (the old screenshot warning fails its
  no-warnings assertion). Before the scope correction the stream also
  ran all 14 projects' 176 playtests with and without the switch:
  every one passes both ways with identical hashes (fx-showcase's 15
  station playtests count their frame checks inside a helper, every
  field check still runs). Not done: the engine's own GPU suites still
  need a renderer by design; `capture` with no renderer answers plain
  null, `hello` is where the tier reads `none`.

## Wave 13, shadow-flags: a model's and a mesh's shadow flags reach the renderer, models casting by default (7dc09488)

Row 87's stream, on lane-2, after webgl2-render-2 made `og.Mesh.castShadow`
reach the renderer and left the other three flags declared but
unapplied.

- **The default.** `GltfDesc` keeps the three.js names `castShadow` and
  `receiveShadow` as booleans with `default true`: a field left out, or
  no desc at all, means "casts and receives", `false` turns one off; the
  same shape as `StandardMaterialDesc.styled { default true }`, so `{}`
  stays a sensible zero value, and a negated name would have read as a
  double negative. `Mesh.receiveShadow` is on by default too (every mesh
  received before; the frames prove nothing moved); `Mesh.castShadow`
  stays off, as in three.js.
- **How it applies.** `og.Model` gained its own `castShadow` and
  `receiveShadow` (default true, at the end of its layout): how a game
  turns shadows off per instance and how the flags show in reflection
  and scene text. A desc's flags are stored on the asset and copied into
  the `og.Model` that `world.instantiate` or a scene's `model` line
  creates (a later `og.load.gltf` of the same path replaces them for
  models instantiated after it; a non-boolean flag is refused). A scene
  entity with both a `model` line and a `Model { … }` table now gets one
  `og.Model` (the table's fields plus the line's asset), where it pushed
  two. In the renderer `receive_shadows` rides in the sign of the
  instance record's layer slot, so no batch splits and a receiving
  instance is the same bytes as before; the lit and physical shaders skip
  the shadow-map reads where it is off; skinned models carry both flags
  and non-casters leave the shadow passes.
- **Frames.** New og-tests scene `shadow-flags` (metal): a low sun over
  a field; in front a trickster model added as-is throws its shadow left
  and one with cast off has none; behind, a 5 m wall shades a row of
  four, the box and trickster that receive dark, the two that do not lit
  orange-red and purple. Kestrel's crypt through its frames script with
  a `set` at 7.9 s: `hero-body.og.Model.castShadow=false` removes only
  the courier's shadow from the floor and right wall (31,612 pixels);
  `stair.og.Model.receiveShadow=false` lights the whole crypt as if
  unshadowed (508,920 pixels). Unchanged: all 112 project frames
  (fx-showcase 62, the eight archetypes 25, the three games 25) are
  byte-identical to the base commit's through `og run --commands` on
  both builds (five of them, idle `card`, match3 `clearing`,
  tower-defense `title`, Kestrel `records`, Lantern Vale `settings-zh`,
  miss the reference identically on both: the stream's way of running
  the scripts outside the playtest runner, not rendering), and every
  og-tests frame is identical when its scene runs alone (`terrain-grass`
  shows 29 pixels off by one level in `og-gfx --all` from the order
  scenes run in, on the base too).
- **Tests.** `crates/og-tests/tests/shadows.rs` (3: the reference, models
  casting as their instances say, surfaces with receive off lit
  unshadowed under the sun and a shadowed point light, every changed
  pixel gaining light and matching a frame where the wall casts
  nothing; 0 pixels change on shaders that ignore the flag);
  `crates/og-runtime/tests/shadow_flags.rs` (6: the IDL defaults, a model
  loaded with no flags, with them off and per instance, an animated
  walker, a mesh's flags, a cooked scene's text round-trip, a bare
  `model` line taking the load's flags; 4 fail on the old runtime); an
  encoding unit test in instances.rs. `GltfDesc.scene` stays ignored as
  before. No vulkan-cpu or browser reference for `shadow-flags`: judged
  against `metal` at the wave's end. Row 87 is closed.

## Wave 13, web-boot: the title behind the gate, the WebGL2 first frame, the boot's bytes and an engine memory readout (69c75852)

Wave 13's first stream took the web findings about boot and the build
(examples/WEB-FINDINGS.md #1, #2, #9, #11, #12), on lane-5. Measured
with `tests/web/dogfood.ts --boot` (new, with `--label` and a log of
every request; the Worker's boot timings on `__runtime_probe__.boot`)
in headless Chrome 150, 1280×720, localhost, the median of 3 runs on a
machine at load about 15; "before" is the dogfood round's release wasm
with the current TypeScript, "after" one `OG_RELEASE=1` build made
before the merge. Three findings did not hold as the brief stated them.

- **#1, the title behind the gate (2b46f6f8).** After the first tick
  the Worker takes one more tick per round of store answers, up to 8,
  waiting up to 300 ms before the first frame and applying later
  answers for 5 s; ordinary ticks with answers applied as usual, so a
  replay is unaffected, no API change (`tests/settle.test.ts`).
  Translated behind the gate: false before, true after for Hollowbell
  (both tiers) and Kestrel (日本語, with its save slot), at 4 to 26 ms on
  the first frame. Kestrel's own menu fade is still part-way in that
  frame: game-side.
- **#2, chunked music (8b088cc6).** The premise was wrong: chunks were
  already fetched only near the playhead, one music chunk (63 to 123
  kB) per game at boot, and the real boot is 4.48 / 4.58 / 5.54 MB
  (Lantern Vale, Hollowbell, Kestrel), not the 6.8 to 10.4 MB the
  dogfood round summed from the directory. What landed: a chunked sound
  may live in a streaming group (its first chunk in the pack, the rest
  read while the group is resident) and the boot progress bar no longer
  counts chunks it never fetches. Boot download and first frame are
  unchanged (Kestrel at 20 Mbit/s 3,385 → 3,393 ms); no game's music
  moved into a group, since title music plays at boot.
- **#11, the WebGL2 first frame (f976f3e0, ca95b156).** The cause was
  52 GL programs compiled one by one at renderer creation, most for
  passes the first frame never draws (water, fluids, terrain, 2D
  lights, glass, bloom, 11 effect passes). Pipelines are made when first
  drawn and the rest warmed in idle time behind the gate (Lantern Vale:
  29 in about 0.7 s on WebGL2, 35 in 0.17 s on WebGPU); the renderer's
  creation fell from 461 to 546 ms to 127 to 155 ms.
  `KHR_parallel_shader_compile` cannot help, wgpu reads each shader's
  status at once. A pipeline not yet warmed compiles on its first draw,
  so a very fast tap can still hitch once.

| Game | WebGL2 first frame, before → after | WebGPU |
|---|---|---|
| Lantern Vale | 1,405 → 703 ms | 488 → 338 ms |
| Hollowbell | 1,118 → 701 ms | 433 → 358 ms |
| Kestrel Stair | 1,197 → 705 ms | 649 → 373 ms |

- **#12, `og build`'s totals (854ce5d6)** print each page's boot
  download (lanes and single-threaded, brotli, split by engine, cooked,
  shell and Basis) and what comes later; both tiers fetch the same
  files (og-build §7, web-export). Hollowbell: 4,323,854 B on the lanes
  page, not the directory's 9.7 MB.
- **#9, engine memory (b8f5110b).** The native host had no member to
  match, so both hosts gained probe member `memory`, one shape: wasm
  memory on the web or process memory natively, the engine's heap (both
  hosts count allocations now), entities, streaming budget, audio; a
  publish page answers through `__runtime_probe__.memory()`. At boot
  28 to 39 MiB of wasm memory on WebGPU, 35 to 50 on WebGL2; after 600
  ticks of Kestrel 54 MiB, heap peak 49 MiB (`tests/hosts.test.ts` on
  both hosts).

Frames: Hollowbell's 9 on WebGL2 match `metal` (worst RMSE 0.00014);
Lantern Vale's same 2 of 7 as the dogfood round before webgl2-render-2
(its misses were #3 and #5). Each wasm module grew about 16 kB brotli;
the 3D lanes module is 3,174,528 B, 75.7 % of the budget (the size gate
measures it at the wave's end). The stream re-applied its harness
changes onto main's `play-origin.ts` split and kept both texts of
og-platform-sdk §3's rule (the coordinator reads that page at the
merge).

## Wave 13, publish-testing: the published page under a script, the text entry's name, clean pool Workers and the kit's settings (71d0d574)

Wave 13's third stream took the web findings about what ships and the
shell (examples/WEB-FINDINGS.md #6, #7, #8, #10), on lane-1.

- **#6, a publish build could not run a playtest's script.** The publish
  probe keeps hello, tick, state, pick, entity, input and capture and
  nothing drove a published page with them. New harness
  `packages/engine/tests/web/published.ts` (`bun tests/web/published.ts
  --project examples/hollowbell [--webgl] [--no-build]
  [--no-isolation]`): it runs `og build --publish`, serves the result as
  the platform opens a release, and plays steps in the playtests' syntax
  with real Chrome input events (keys, typed text, mouse presses on the
  controls it finds in the accessibility mirror); `screenshot` is the
  probe's capture compared with the project's references under
  og-tests' rule, `state` and `mirror` feed the checks, a reload stands
  in for a playtest's `then` relaunch, and a command-channel step
  (`warp`, `show`, `dump_*`) is refused, never skipped. The engine
  Worker runs on a clock the harness controls (one tick per frame, held
  while a capture's readback lands), installed through the DevTools
  protocol before the Worker starts, so the built files are untouched
  and every capture is of the tick it names. The stand-in play origin
  and the mirror-driven input moved out of dogfood.ts into
  `tests/web/play-origin.ts`; the frame comparison into
  `tests/support/png.ts`, which reference.ts reads too. Hollowbell 13 of
  13 on WebGPU, WebGL2 and without isolation (the tick-60 frame matches
  `metal`'s title exactly; a typed name, Begin and a held ArrowRight
  reach the game; the console clean); the kit 18 of 18 (the gallery
  frame exact, a saved 简体中文 back after a reload). Run against the old
  pool helper, the old textarea and the old kit, it failed on each. Not
  in any `check.sh` lane; og-build §6 and docs/web-export.md say how it
  runs. Most existing playtests cannot run through `--playtest` (dump
  and game-command steps), so its two plans live in the harness.
- **#7, the hidden textarea was `aria-hidden` while focused.** It is
  hidden only while no field is focused; a focused field's textarea
  takes its name, placeholder and text from the field's node in the
  mirror, whose own element hides and whose dialog owns the textarea,
  so a reader meets one named text box (text-input.ts, a11y.ts,
  shell.ts). New `tests/web-text-entry.test.ts` drives the real shell
  against a stand-in Worker and document (`tests/support/dom.ts`),
  failing 3 of 3 on the old code.
- **#8, eight deprecated-parameter warnings per lanes page.** New
  `scripts/rayon-worker.sh` rewrites wasm-bindgen-rayon's
  `pkg.default(data.module, data.memory)` to the single-object form on
  both paths build.sh puts a lanes module in dist, and stops the build
  if the helper has neither form; no version moved (docs/ci.md). A test
  in `tests/web-export.test.ts` checks the shipped snippets for any
  positional init call.
- **#10, the kit never read its saved settings.** `templates/ui-kit/src/main.ts`
  asks for its settings on the first tick and applies every saved field
  when the answer arrives, the settings screen through the same code;
  new playtest `settings-saved.og.ts` (a first run saves Mute and
  简体中文; a relaunch with the store kept asks on tick 0 and comes back
  muted with a Chinese menu). og-platform-sdk §3 now says settings are
  a pair (load on the first tick, apply what you save) with the kit's
  code as the example, and og-i18n §5 how to restore the language;
  validate.ts passes. examples/WEB-FINDINGS.md gained a "Fixed" section
  for the four.

**The coordinator's run (the measure worktree at 5137ee44, after the
merge):** `bun tests/web/published.ts` on Hollowbell and the kit, both
tiers: Hollowbell 13 of 13 on WebGPU and on WebGL2 (the engine Worker on
the harness's clock, the title's New in the mirror behind the gate, a
real click opening it, the tick-60 title RMSE 0 against `metal`, the
name dialog's focus in a named text box no aria-hidden hides, "Wren"
typed, Begin, ArrowRight moving Wren from x 16.5 to 21.0, nothing on the
console), the kit 18 of 18 on both (Mute, 简体中文 and Done saved once,
the reload asking the play origin for og.settings and coming back in
简体中文, the console clean).

## Wave 13, webgl2-render-2: Lantern Vale's ink, Kestrel's crypt and the exposure's start on WebGL2 (c178e216)

Wave 13's second stream took the web findings that are rendering on the
WebGL2 tier (examples/WEB-FINDINGS.md #3, #4, #5), on lane-2. Both of the
coordinator's guesses in the brief were wrong: the crypt had nothing to
do with the sun or a roof, and the ink was not a GL difference in the ink
or outline passes.

- **#4, the crypt lit as if roofless (51a291fa).** The crypt's lamp is a
  small glass box with its shadowed point light inside. The engine
  ignored `og.Mesh.castShadow`, so the box cast a shadow: on metal and
  WebGPU the six cube faces stored the box and shut the lamp's light in;
  on WebGL2 the two paraboloid hemispheres never stored it, so the lamp
  lit the whole crypt. Forcing the no-compute tier on metal reproduced
  Chrome's frame exactly (RMSE 0.0456, 7.58 % outliers). `og.Mesh.castShadow`
  now reaches the renderer (off by default, as in three.js; `og.Model`
  meshes still always cast). A second defect found on the way: the
  paraboloid projection was mirrored, so it stored the faces turned
  towards the light, which is why it missed the box and why it drew a
  curved shadow band down boxes' lit faces (shadow_pass.wgsl,
  lighting.wgsl). New og-tests scene `shadows-crypt` (a roofed room, a
  low sun reaching the floor only through the door, a point light inside
  a glass box that casts no shadow); it applies on every tier and needs
  no WebGL2 reference: paraboloids are within RMSE 0.0036 of cube faces
  (0.0148 before). fx-showcase meshes whose shadows its frames show set
  `castShadow`, so those frames stayed.
- **#3, Lantern Vale's ink and sparks (0082ead5).** At boot the engine
  loaded the game's look to read its antialias setting; loading it
  pulled in textures the web host cannot decode before the renderer
  exists, so it failed silently and the web renderer fell back to the
  tier's method: TAA on WebGPU (the look's own, so nothing showed) and
  4× MSAA on WebGL2, which leaves the ink and outlines unsoftened. It
  reads the look file alone now. The sparks (CPU particles on WebGL2) are
  by design but need no WebGL2 reference either: with TAA the no-compute
  tier draws lantern 0.0017, slime 0.0026, far 0.0056 from metal. Test in
  crates/og-runtime/tests/web_findings.rs, failing on the old code.
- **#5, the exposure's start (441f4e6e), and the WebGPU title and start
  misses of #3.** Adaptation already stepped per tick; where it started
  was host-dependent twice over: a streamed terrain had none of its
  tiles on the first frame it was drawn, so that frame showed no ground
  and metered 2.5 EV too bright, and the title stayed about 1.5 stops
  under for two seconds (a terrain now settles its tiles on its first
  frame, as a capture does); and the web draws a frame at tick 0 where
  `og run` does not (a tick-0 frame now meters but never seeds the
  adaptation). Emulating the web's tick-0 frame natively reproduced the
  web exactly before (title RMSE 0.0310) and matches within 0.0067 after.
  Test in tests/exposure.rs, failing on the old code.
- **IDL (c178e216)**: `castShadow`, the adaptation rule on `autoExposure`
  and `adaptSpeedUp`, and "not applied yet" on `Mesh.receiveShadow` and
  `GltfDesc.castShadow`/`receiveShadow` (row 87). Re-recorded on metal:
  Kestrel `crypt` (RMSE 0.0458 from the old reference) and `bag`
  (0.0274), Lantern Vale `title` (0.1734) and `start` (0.0445), og-tests
  `point-shadows-paraboloid` (0.0021), `shadows-crypt` new.

**Browser verification (the coordinator, in the measure worktree at
c178e216 merged with 48133d07, headless Chrome 1280×720).** Every frame
against `metal`, no WebGL2 reference added: Lantern Vale 7 of 7 on both
tiers (worst `far` 0.0073; WebGL2's lantern 0.0059 where #3 had 0.0189,
and WebGL2 now draws within 0.0001 of WebGPU on every frame); Kestrel
Stair 9 of 9 on both (WebGL2's crypt 0.0033 where #4 had 0.0456 with
7.56 %, bag 0.0021); og-tests' gfx scenes on both (`shadows-crypt` WebGPU
0, WebGL2 0.0036; `point-shadows-paraboloid` 0 and 0.00004;
`point-spot-lights` 0 and 0.0027; WebGL2's `showcase` and `overlay`
against their own references 0.0081 and 0.0080, as predicted); Ember
Vale on WebGL2 (title 0.0043, beacon 0.0020 against its WebGL2 reference,
dawn 0.0019). Lantern Vale keeps no vulkan-cpu copy since 36b326c3, so its
moved frames are judged on lavapipe at wave 12's end. Native ANGLE stays
uncommitted (row 70; it also needs `ANGLE_DEFAULT_PLATFORM=metal`).

## Wave 12, web-dogfood: the three games and the kit published and played in Chrome (ccc462d)

The fourth dogfood round, of the web path the native rounds never
exercised: each of examples/lantern-vale, examples/hollowbell,
examples/kestrel-stair and templates/ui-kit published as an author would
(`og build --publish`, the 2D modules for Hollowbell), served as the
platform's play page opens a release (the top-level page, `__OG_GAME`, a
stand-in play origin answering Player protocol 2 over HTTP, the build's
`_headers` making the pages isolated, the service worker active) and
played in headless Chrome on both tiers with real clicks through the
accessibility mirror (a publish build has no command channel, so the
playtests' scripted sequences and the frame comparisons ran on the dev
export of the same project, through `tests/web/reference.ts`). Measured
at 7526ff3c with one release build (`OG_RELEASE=1 sh scripts/build.sh`,
15 min) and one Chrome at a time; the harness is
`packages/engine/tests/web/dogfood.ts`, the findings and the full tables
examples/WEB-FINDINGS.md.

| Game | Boot download (brotli) | Tier | First frame (ms) | Step p50/p95 (ms) | Render p50/p95 (ms) | Frames vs metal |
|---|---|---|---|---|---|---|
| Lantern Vale (3D) | 8.29 MB | WebGPU | 547 | 1.34 / 1.57 | 0.55 / 0.69 | 5 of 7 |
| | | WebGL2 | 1,742 | 1.46 / 1.81 | 1.06 / 1.22 | 2 of 7 (worst 0.038) |
| Hollowbell (2D) | 6.76 MB | WebGPU | 533 | 1.44 / 1.62 | 0.71 / 0.79 | 9 of 9, pixel-identical |
| | | WebGL2 | 1,075 | 1.51 / 1.73 | 0.97 / 1.12 | 9 of 9, pixel-identical |
| Kestrel Stair (3D) | 10.39 MB | WebGPU | 752 | 5.04 / 6.79 | 0.68 / 0.83 | 9 of 9 (worst 0.0046) |
| | | WebGL2 | 1,368 | 5.85 / 7.48 | 1.34 / 1.64 | 7 of 9 (crypt 0.046, 7.6 %) |
| UI kit (2D) | 11.55 MB | WebGPU | 529 | 0.90 / 1.02 | 0.30 / 0.37 | 22 of 22 |
| | | WebGL2 | 1,092 | 0.93 / 1.11 | 0.43 / 0.53 | 22 of 22 |

Every game held 60 ticks a second on the lanes pool (8 Workers); the
single-threaded engine is 3.09 MB brotli (3D) and 2.92 MB (2D), the
lanes modules 3.13 and 2.96 MB; store reads and writes, Kestrel's
`leaderboard.top` and the start, ready, sample and end reports reached
the stand-in origin; the language saved through settings came back after
a reload in all three games; no unnamed node on any title; no page
errors; without isolation the single-threaded engine reaches the first
frame 100 to 190 ms sooner but Kestrel's step goes from 5.0 to 8.1 ms; at
20 Mbit/s the first frame is 2.8 to 3.5 s for the games and 5.7 s for the
kit, past the 2.0 s budget.

Findings: 0 blockers, 7 annoyances, 5 nits. (1) The first screen behind
the play gate is the game's tick 1, before saved settings and saves
apply: the title shows in English with Continue disabled until the tap.
(2) Chunked music is all fetched before the first frame (a chunked sound
cannot go in a streaming group): 47 to 56 % of each game's boot download.
(3) The WebGL2 fallback draws Lantern Vale's ink outlines and lantern
sparks differently: five of seven frames miss `metal`. (4) WebGL2 lights
Kestrel's crypt as if it had no roof (not diagnosed). (5) Lantern Vale's
first two seconds are brighter on the web on both tiers (auto exposure).
(6) A publish build cannot run a playtest's script, so nothing scripted
checks the bytes that ship. (7) The hidden text-entry textarea is
`aria-hidden` but takes focus. (8) Every lanes page logs eight warnings
from the pool Workers it cannot fix. (9) No way to read the engine's
memory on the web (`measureUserAgentSpecificMemory` never resolves, the
probe reports none). (10) The UI kit saves its settings and never reads
them. (11) The WebGL2 first frame is two to three times WebGPU's. (12)
`og build`'s printed total is not what a player downloads. Wave 13 takes
them.

## Wave 12, skins-palette: every skin under the packs' rule, badges on either side, Hollowbell's Skip, Kestrel's chapters (38f6435)

Rows 77, 78, 79 and 84.

- 3da271b9, row 77: brightness changed, not hue (og-taste §3): sparkorb's
  gold a darker amber under its cream labels (`#ad690a`; the bright gold
  stays in `accentInk`, the glow and the HUD, so idle's gold words and
  icons moved from `accent` to `accentInk`), its danger lighter;
  lampdive's fills and subtle ink, with the pixel pack's `textWeight body
  0.5` and `px_range 8` (its pressed primary label read 3.91:1 in the
  frame without them); lanebeat's hover the scifi value; pathguard's
  orange deeper with a cream `accentFg`, darker inks, a deep danger;
  Hollowbell's panel one step darker from its own 32 colours (the coral
  danger kept); Lantern Vale's success, warn and info back to the
  fantasy pack's deep values; Kestrel unchanged (its bag's count reads 6:1
  by colour; polish's lower reading was its sampling, #20).
- b358d6a8, rows 77 and 78: `Srgba::readable_on` (and `READABLE` 4.5,
  `over_linear`) moves an ink towards white or black until it reads
  4.5:1 on a ground; a status or accent badge on a ground on the other
  side of the scale from the palette's panels (a light pack's badge on
  its dark stage) moves its ink, in its panel it keeps the palette's; the
  neutral badge moves `fgMuted` where it does not read on `fill`, and
  where no light ink can (scifi's mid-blue) the fill darkens; meter
  labels take the same rule. og-ui sees only the palette, not a skin's
  panel art, so it steps in only on the other side of the scale and the
  palettes stay responsible for their panels, which the test checks.
  `pack_contrast.rs` covers all 14 skins (the seven packs, the four
  archetype skins, the three examples') under all three tests including
  the rendered-frame reading, with a stack's count on a slot, meter
  labels, every badge tone on both grounds; the og-ui README and
  og-taste §3. Before to after: sparkorb primary 3.42/2.93 to 7.22/5.52,
  pathguard primary 3.30/4.20/2.99 to 7.75/7.64/8.28 and its danger in a
  panel 2.96 to 4.80, lampdive's placeholder 3.57 to 5.01, the neutral
  badge in scifi, neon and lanebeat 2.15 to 4.65, light packs' badges on
  the dark stage 1.05 to 3.53 up to 4.57 to 4.64, Lantern Vale's badges
  3.15/2.72/4.07 to 4.67/4.68/4.70, Hollowbell's danger badge 4.26 to
  4.50 (at the bar, no margin). `make-skins.py` reproduces the six packs
  byte for byte; none of them changed.
- dcdcd17c, rows 79 and 84: Hollowbell's `talk.skip` key (Skip / 跳过)
  passed as `skipLabel`, its Chinese subset regenerated with 过, its
  locales test asserting it; Kestrel's chapters script taps Title after
  the results' 700 ms wipe (16.6 s), sets `Game.chaptersReached` so all
  four cards show names, opens the chapters at 17.6 s and shoots at
  19.2 s.
- Frames re-recorded on metal (6cb4d28f, 529bdd1d; RMSE against the
  old): Kestrel `chapters` 0.1835 (the four named cards now), og-ui
  `kit-skin-scifi-phone` 0.0163 (the "Lv" badges on a darker fill),
  idle title 0.0203, orbit 0.0155, card 0.0211, roguelike title 0.0152
  and over 0.0210, tower-defense title 0.0253, wave 0.0171, end 0.0224.
  Roguelike `dive` (0.0076) and every Hollowbell and Lantern Vale frame
  (at most 0.0066) within tolerance, kept.
- Left out: the vulkan-cpu copies of the moved frames (the wave's end);
  sparkorb's accent is the most visible character change (a darker gold
  was the only value reading on its neon button art in every state).

Functional lane on the merge: see the wave 12 gate section.

## Wave 12, render-audio-3: the mesh flash on the instance record, parented audio (5bedf86)

Rows 85 and 86. Merged after main's e7d6cb5e.

- 7ab860a0, row 85: a flashing `og.Mesh` puts its `flash` and `flashColor`
  into its instance record (the one `og.Model` uses since 5bbf4909), so
  a flashing mesh is one draw with its unflashed twins and the 16-level
  material copies are gone; the amount is continuous, the colour 8 bits
  per channel, `og.tween.flash` unchanged for callers (`og.Mesh` has no
  tint, so none was added). The copies had a second defect: each was
  made once and kept its material as it was then, so a material updated
  mid-flash (a dissolve) flashed stale. `crates/og-runtime/tests/
  mesh_flash.rs` fails on the old code. A scripted frame of fx-showcase's
  stylised station with three flashes against the old path: the rim
  sphere at 0.5 within 1/255 (an old level), the toon sphere at 0.3
  within 2/255 (against the old 0.3125: the continuous amount is the
  intended one), the dissolve box correctly fully dissolved where the
  stale copy drew white fragments; whole frame RMSE 0.0045. No og-tests
  reference moves (those scenes never go through `og.Mesh`); no template
  or example frame should (topdown and the archetypes flash sprites,
  Kestrel flashes models, fx-showcase a material); Kestrel and topdown
  at RMSE 0 or near, fx-showcase within tolerance.
- f89f1885, row 86: sources, the listener and reverb zones are placed at
  their `og.Transform` composed down `og.Parent` (source gain, pan,
  distance dulling and the occlusion ray, the listener's position and
  rotation, zone weights), through one helper, `og_core::math::world_pose`,
  that `og_physics::world_pose` keeps calling in the same order (physics
  bit-for-bit unchanged); og-audio mirrors `og.Parent` through a new
  query and composes nothing when nothing is parented. Tests in
  `crates/og-audio/tests/parented.rs` (a source on a moving parent, a
  listener under a turned rig, a scaled zone under a parent; all fail
  with the old reading) and a `world_pose` test in og-core. The og-audio
  README's owed line closed; og-audio §6. A game's hash moves only with
  parented audio (none of the templates or examples has any); a bare
  og-audio world registers one more type.
- Left out: the browser tiers not checked for the mesh flash (the wave's
  end); rows 85 and 86 closed here.

Functional lane on the merge: see the wave 12 gate section.

## Wave 11, runtime-ui-2: typed resource handles, borrows by view, save slots, focus, grids, polish's ink (ce93d34)

From The Kestrel Stair's findings. Merged at ce93d34b (a fast-forward).

- 27e245f6, #19: a resource handle is a typed view of the resource's
  packed bytes, no JSON: one shared copy per world and resource, fetched
  again only on the first read after an engine call or a system
  boundary; writes held and sent in one call before the next engine
  call, at the end of the system, command or facts call, at once outside
  game code. Three lifecycle ABI members and a bytes wire type in
  og-idl's generator (napi's own Uint8Array was 30 to 100 µs a call; the
  native side copies). A test writes 200 fields a tick of every field
  kind with engine calls between, matching the JSON path's hash and
  values, replaying from a save, the same hash on native and web; the
  benchmark's world hash unchanged (20dc498c962b8fe0). Measured on the
  game's `Game` (111 fields) and `Prefs`, dev addon, median µs, under
  load: a Game field write 93 to 0.03, a read 0.9 to 0.02, a read after
  an engine call 143 to 2, write-call-read 240 to 26 (mostly the two
  strings' checks in the dev build), `world.resource` 205 to 1.5 to 5;
  Prefs write 4.7 to 0.03. The game's step 7.5 to 4.8 ms median, its feel
  score 79 to 97. og-core §4 carries the cost model; the game's plain-copy
  workaround is gone.
- c0d29c75, #6: a system's columns are borrowed only when it takes a
  view (`a.column`), held until it returns, so `world.get`/`world.set`
  work on a matched entity until a view of that column is taken; the
  refusal then names the view and the way through. The game's `OneByOne`
  component left its six systems.
- 67aca309, #13: 64 KiB a key is the platform's limit (64 keys, 1 MiB
  per player and game), so `og.platform.saveSlot`/`loadSlot`: parts in
  whichever of two banks of keys (`<slot>.a0..`, `<slot>.b0..`) the last
  save did not use, the manifest written last at the revision read, so a
  save cut short leaves the previous whole and two devices saving at
  once get a conflict, not a mix; one request a call; `loadSlot` checks
  the parts against the manifest's length and checksum, `metaOnly` reads
  the title-screen data. Tighter encoding measured and not taken on the
  game's save (201 KB raw, 78 KB of text: deflate 9 saves 0.7 %, LZMA
  31 % at the cost of a second decoder in every build). og-game §17,
  og-core §2, og-platform-sdk; the game's own splitting gone.
- da50fea3, #14: `ButtonDesc.autofocus` (a button takes the focus when
  it first appears outside a dialog; `mainMenu`'s first enabled item has
  it), and a `navigate` queued while nothing has focus lands on the first
  item however many were queued; the ui-kit a11y playtest expects the
  menu's first item focused.
- 83ddce5d, #15: the grid lays out exactly the `columns` asked at any
  width (cells share the width, keep their height); `levelSelect` gives
  a named level a wider card with smaller wrapped type and an ellipsis
  after two lines; `inventoryScreen` widens to its columns; the ui-kit
  demo asks 6 inventory columns and 4 named cards, `tests/grid.og.ts`
  checks the rows and that every name fits, desktop and phone. The game's
  chapters have names again, its bag six slots a row.
- fbd6e3a2, #20: polish measures the ink over the text's own glyphs
  (pixels at least halfway to the strongest ink, the farthest twentieth
  averaged); og-build §5. Contrast: Kestrel 97 to 98 ("You" on the
  records row 2.14 to 9.94:1), Hollowbell 93, Lantern Vale 98 to 100. Also
  a reload that changes a resource's layout hands out a fresh handle.
- Frames re-recorded on metal: ui-kit inventory 0.0676 and levels 0.0792
  (phone 0.0695, 0.1101), the twelve `kit-skin-*` sheets' inventory and
  levels cells (0.066 to 0.283), the game's bag 0.1456 and title 0.0218.
- Left out: no vulkan-cpu copies (the wave's end); the game's chapters
  frame still shows the results screen (its playtest's timing is off),
  the named chapters checked by an `og run` screenshot instead (row 84).

Functional lane on the merge: see the wave 11 gate section.

## Wave 11, physics-anim-2: world-space cloth, a 3D squash, a model's own flash, reverb zones (7b1973a)

From The Kestrel Stair's findings. Merged at 7b1973a7 after nav-ai, the
nine generated wiki pages that conflicted regenerated and the game's
strike frame re-recorded on the merged engine (the warden flashing white
and the court's imp standing where nav-ai leaves it).

- 12fa298a, #10: ropes, cloths, fluids, water lines and ragdolls are
  placed at their entity's world position composed down the `og.Parent`
  chain (`og_physics::world_pose`), as are the runtime's ragdoll targets
  and water-height reads; rigid bodies and colliders keep treating their
  Transform as world space ("bodies are roots"), now stated in the
  physics README. A cloth pinned to a child follows its parent's move,
  turn and a grandparent (`tests/soft.rs`). The game's cape is a child of
  the body again, its per-tick placement gone.
- 17851d64, #4: `SquashDesc.impact` is a `Partial<Vector3>` (z left out
  or 0 stays at rest, so 2D calls are unchanged; the IDL refuses an
  explicit default on a Partial, so the defaults 1.25, 0.8, 0 are in the
  doc text); og-feel §5.
- 5bbf4909, #11: the instance record grows from 52 to 64 bytes for a
  per-instance flash amount, flash colour and tint (`og_render::InstanceFx`,
  skinned instances too), applied by every lit shader (full, lean,
  array-map, physical); `og.Model` gains `tint`, `flash` and `flashColor`
  and `og.tween.flash` takes an og.Model entity. All 80 metal gfx frames
  but the new scene unchanged; `model-flash` (three instances of one
  model, one flashing white, one tinted amber, two skinned cylinders, one
  flashing red) recorded on metal, its test checking the flash adds no
  draw and leaves the twin untouched. The game's struck foe flashes white
  alone; its combat test checks the other warden stays unflashed.
- a4169c06, #21: `og.ReverbZone` (a box or sphere with a fade distance,
  bus, send, room size and damping); each tick og.audio writes how far
  inside each zone the listener is (`weight`) and the strongest zone
  blends the shared reverb's room, damping and its bus's send towards
  its own, `og.Audio` and `og.AudioBusFx` keeping the open-air values; a
  function of the listener's position, so deterministic and restored;
  og-audio §4 and §6, `tests/mix.rs`. The game's crypt is a zone and its
  Fx system's hand-rolled easing is gone.
- Frames re-recorded on metal: the game's `strike` (0.0147 on the branch,
  0.0301 after the merge with nav-ai's), `chapters`, `records` and
  `results` (0.0218, 0.0196, 0.0218: the summit look's own sky light now
  shows, the old easing having kept writing 0.45 over its 0.7); og-tests
  `model-flash` new. Under tolerance: `explore` 0.0026, `crypt` 0.0054,
  `bag` 0.0057.
- Hashes move by design, since `world_hash` includes the registered
  types: physics registers `og.Parent`, audio `og.ReverbZone`, and
  `og.Model`'s layout grew; the determinism tool was not run on the
  branch (the wave's end). Game replays still end two runs on one hash.
- Left out: `og.Mesh`'s flash still draws through a copy of its material
  at 16 levels (moving it onto the instance record would move frames);
  zones and the listener read their own `og.Transform` (parented audio
  stays owed in the og-audio README); `model-flash` and the four game
  frames owed on vulkan-cpu and the browser tiers (the wave's end).

Functional lane on the merge: see the wave 11 gate section.

## Wave 11, nav-ai: the unplanned agent, restart, perception, bounds, checks against fields (a7cd639)

From The Kestrel Stair's findings. Merged at a7cd639f.

- 8af9ea4e, #7 (blocker): a new `og.NavAgent` no longer stands on
  polygon 0. Before it moves, its position is checked against its
  current polygon; if that does not hold it, the nearest polygon within
  its height and twice its radius (at least 1 m) is used, which also
  covers an agent the game teleports; an agent at rest is not moved at
  all. The test puts polygon 0 on a high terrace and idle agents on a
  court and a lower terrace: they keep their positions, each reaches a
  target on its own level, and one teleported while idle walks from
  where it lands; against the old code it reproduces the finding (the
  agent at y 8.1). #9: `NavMeshInfo` bounds come from the polygons
  (before, the top was the floor's origin plus the agent's height).
- 2f4b2f41, #5: `og.ai.restart(world, entity)` in the IDL, both hosts
  and the runtime: at the next think a machine goes back to its initial
  state and runs `enter` again, a tree starts at its root, timers,
  cooldowns and the blackboard reset, what it saw and heard is forgotten,
  the speed it started with kept. #8: an agent no longer hears its own
  entity's sounds or anything parented under it, `NoiseDesc.source`
  excludes game noises the same way, and a parented sound is heard at
  its world position (it was heard at its local one). #3: a `check` can
  compare against a key, `Component.field` or `Resource.field`, read on
  every evaluation, a missing reference failing the check with a warning;
  a cook test and a runtime test.
- 5daa77c8, 94e043d4, #1: og-game, og-feel and og-ai name
  `templates/archetypes/third-person` as the 3D action pattern; the
  "see nav-ai" placeholder knowledge-3 left in og-ai filled, and its
  habit changed to "idle or seeking" since an idle agent is safe now.
- Workarounds removed: the game's (2d4cda64: foes spawn idle with their
  minds on, `Foe.warm`/`awake`/`spare` gone, each behaviour loads once
  and reset calls `og.ai.restart`, the flee and reach thresholds are
  Tuning fields the behaviours read) and the third-person archetype's
  two-handle swap (807ad88d). The game's foes playtest runs 0.8 s later
  (foes wait out their patrol's first pause), its move playtest stops the
  court's imp patrolling, `strike.png` re-recorded (the terrace warden
  standing where it was placed).
- Determinism: the tool does not link og-nav or og-ai; its host debug run
  matches the recorded values. The game's world hashes move by design
  (foes idle instead of seeking); its replay test still ends two runs on
  one hash.
- Left out: og-nav's README costs table not re-measured.

Functional lane on the merge: 5 of 5 in 160 s; the Kestrel Stair,
third-person, topdown and Lantern Vale suites passing after it.

## Wave 11, knowledge-3: the navmesh, behaviour and animation-graph grammars, a clip's markers, the 3D facts (9f5084e)

From The Kestrel Stair's findings (#2, #12). Merged at 9f5084e9 (its
merge with main), the wave's first.

- 34d32e4e, #2 and #12: og-ai §1 writes every line of a `type navmesh`
  file with its default (from og-cook's `cook_navmesh`) and the game's
  glTF-baked one; §2 places each agent at its `og.nav.nearest` point,
  proved in a playtest; §3 every line of a behaviour (every node, option,
  operator and og.ai's own keys, from og-ai's parser), the game's warden
  as the machine. og-game §9 gives a model's whole `animation` import
  block, how a clip carries its markers (`extras."og.events"` or
  `event`), root motion, and every line of a `type animgraph` with the
  third-person hero's graph. Every block cooked (navmesh, behaviour) or
  registered (`og.anim.graph`) in a scratch project. `scripts/validate.ts`
  extends wave 9's two-way drift check to the three grammars and the
  markers: per grammar the names an author chooses, a reader of the keys
  its parser looks up by name (arms with alternatives, `.get`/`.get_all`,
  `*_of` lookups, option lists, `BUILTIN_KEYS`) and the skill each lives
  in; eight tests in `tests/grammar.test.ts`, one per kind of drift. The
  IDL's pointers at og-ai's and og-anim's READMEs (the ai module's doc,
  `GraphDesc.text`) and `og.nav.load`'s doc point at the skill sections
  now; the wiki and types regenerated.
- d049c598: each skill's What works carries the game's measured 3D
  facts: og-game the cook-time and run-time retargets onto three rigs
  through blend trees with markers intact, root motion, the glTF navmesh
  bake, the controller over triangle meshes, cloth and rope, the snapshot
  across a relaunch and the cook's times; og-ai (a new What works) the
  bake and the trees and machines with occluded sight, steering and an
  obstacle; og-audio, og-build, og-core, og-feel, og-i18n,
  og-platform-sdk, og-taste and og-ui theirs. Facts only.
- 6f617ae2: og-game's checklist gains the 3D step (5 of eight: bake the
  navmesh from the level, load it and check `og.nav.nearest` at every
  spawn and route point before writing the behaviours), and its mistakes
  list the habit #7 would have been: every navmesh agent at its
  `og.nav.nearest` point, with a playtest reading its position and status
  a few ticks in (og-ai §2).
- Left as placeholders for the streams still running: a check against a
  field and restarting a behaviour in og-ai ("see nav-ai", filled by
  5daa77c8), a reverb zone in og-audio's What works ("see
  physics-anim-2"), a save slot's helper in og-platform-sdk's ("see
  runtime-ui-2") and the resource handle's cost in og-game's mistakes
  list ("see runtime-ui-2", rewritten by 27e245f6); the last three filled
  from those streams' commits by wave 11's records (records-11).

Functional lane on the merge: 5 of 5 in 177 s.

## Wave 10, lanes-per-profile: a lanes module per physics profile (0e77b6b)

Row 71. One `og_host_web-lanes.wasm` carried both physics profiles and
was the largest download (3,472,308 B brotli, 82.8 % at 9d220790); a page
already downloads its own profile's publish module, so a lanes module per
profile loses no game anything.

- 706833d5: `wasm-crates.txt` gains two `lanes` lines (`lanes2d` =
  publish, physics2d, shaping; `lanes` = the same with physics3d), both
  built by `scripts/wasm-lanes.sh` into the shared `target/wasm-mt`, 2D
  first; `check.sh` and `size-gate.sh` take every `-lanes*` module
  through wasm-build, wasm-opt and `opt.txt` (the gate labels them "3D
  lanes" and "2D lanes"); `build.sh` puts each lanes module with its
  `snippets/` beside its profile's publish module in gate and release
  mode, `pack.sh` refuses a package missing either; `web-build.ts` takes
  the lanes files from the profile's directory for `og build --publish`;
  `og doctor` reports `wasm-lanes-2d`; `hosts.test.ts` loads each
  profile's lanes module under bun (64 bodies on one thread, 1,024 on
  four Workers with every step on the pool; after 600 ticks the snapshot
  equals the native host's and restores natively to the native hash, 2D
  967187ae3438d30b; each module refuses the other profile's world),
  `web-export.test.ts` and `tests/web/export.ts` check the layout. The
  old both-profiles module is gone (a third lanes build per gate for no
  page); a published build's layout is unchanged (the same
  `origin-engine-lanes*` and `snippets/` names), only the package gained
  `dist/web-publish-2d`'s lanes files.
- a06c1eac: the determinism tool builds and runs the two lanes modules
  too and splits `physics_hash` into `physics3d_hash` and
  `physics2d_hash`.
- 6e5806dd: docs/web-export.md §Variants and §Lanes, docs/ci.md (§The
  lanes wasm with the cost, §Switches, the table of what runs where), the
  wasm-size README's wave 10 section and
  `tests/evidence/P4/wasm-size/lanes-per-profile-determinism.md`.
- The cost of the extra lanes build (the profiles share std and every
  dependency): path-aware lane 39 s cold, 14 s after an og-physics edit,
  8.5 s after an og-runtime edit; shipping profile 1 min 46 s cold (the
  old single module took 3 min 20 s), 64 to 83 s after an og-physics
  edit; each module still relinks og-host-web, about 6 s; the
  determinism step's two lanes builds about 23 s each.
- 0e77b6b6, the merge with main, also restored the eight sections of
  docs/ci.md that the CI removal commit fb4d0d9e had cut by accident
  (§Running the gate locally through §Browser reference tiers' lead,
  §When to run what, §The lanes wasm and §Switches among them: 438
  lines); main's §CI text kept.
- Left out: `OG_RELEASE=1` `build.sh` and `pack.sh` not run (a full
  release build); the determinism lanes builds run on one thread under
  node (the four-Worker comparison is in `hosts.test.ts`); found on main
  and left: `cargo clippy --manifest-path tools/wasm-determinism/Cargo.toml`
  fails on a duplicated attribute in og-anim's test support (the gate
  does not lint that tool).

Measured on the branch before the merge (the gate section above): `--all`
9 of 9 in 839 s, web-reference 411 s, the three web tests in Chrome.

## Wave 10, dogfood-3: The Kestrel Stair, the third dogfood game (9605360)

examples/kestrel-stair, a 3D third-person action-adventure of about five
minutes written from the knowledge package alone (engine e16e9cec, then
main f1a31849 merged at the end): a courier climbs a ruined hillside
monastery to deliver a letter to the summit bell before 19:30; thorn imps
chase, bite and flee when hurt, stone wardens close in and slam after a
wind-up; three sigils raise the summit gate. It exercises what the first
two games did not: a glTF level written by code with slopes, stairs,
ledges and a crypt under `og.CharacterController`; a navmesh baked from
the level, behaviour trees and state machines, steering avoidance; three
skinned bodies with different rigs sharing one clip library, retargeted
at cook time for the hero and at run time for the enemies (one through a
`type rig` file), blend trees, root motion on the warden's lunge; a cape,
four banners and a crypt chain with a lamp; spatial sources, a reverb
run by the game in the crypt, a streamed synthesised track; a look
extending `watercolor-wash` with a day-night clock and
`luminous-layered-painting` for the ending; the paper skin with
`cutscene`, `tutorialOverlay`, `inventoryScreen`, `levelSelect` and
`leaderboardScreen`; English and Japanese (Noto Sans JP subset by the
ui-kit script); a save slot through `world.snapshot()`/`world.restore()`
across a relaunch and a leaderboard entry through `og.platform`. 13
playtests, nine metal frames (title, intro, explore, strike, crypt, bag,
chapters, records, results), a two-process replay on one hash, the
relaunch, the a11y tree. `og polish`: taste 98 (contrast 97), feel 79
(frame time 0 to 9 on a machine busy with other builds; step 7.5 ms
median, p95 10 to 30 ms). On main after the merge: 13 of 13.

FINDINGS.md: 21 findings, 1 blocker, 12 annoyances, 8 nits; top five #7,
#10, #19, #13, #6 (with #2). Three were fixed by wave 10 before the merge
and confirmed after it: #16 (`sliding` true on level ground) and #17
(stairs and slopes) by the shared mover, #18 (the templates' words) by
row 76. Wave 11's streams took the rest: nav-ai (#7 the unplanned
NavAgent snapped to polygon 0, blocker; #5 restarting a behaviour; #8
Perception hearing its own agent; #9 NavMeshInfo's bounds; #3 `check`
against a field; #1 the third-person archetype named), physics-anim-2
(#10 cloth, ropes and the rest reading the entity's own Transform as
world space; #4 SquashDesc.impact a Vector2 in 3D; #11 no hit flash on a
glTF instance; #21 no reverb zone), runtime-ui-2 (#19 a resource field
write costing in proportion to the whole resource, 34 ms a tick as
og-core shows it; #6 `world.get`/`set` refused inside a system by the
declaration alone; #13 a 77 KB snapshot over the store's 64 KiB a key;
#14 the title menu taking no focus; #15 `levelSelect` ignoring
`columns` and clipping names, `inventoryScreen` off by one; #20 polish's
contrast measure on short text in wide labels), knowledge-3 (#2 the
navmesh, behaviour and animation-graph grammars; #12 clip markers).

## Wave 10, cook-determinism: basisu's padding bytes, a torn pipeline cache (033182e)

Row 80 (`cooking_twice_gives_identical_bytes` failing 2 of 5 on the CI
runner): the cause was inside basisu, the C++ texture encoder, not the
cook. Its UASTC rate-distortion pass keeps a hash map whose key is an
8-byte and a 4-byte field hashed over all 16 bytes, so the four padding
bytes, never written, go into the hash with whatever the stack held: the
pass could miss a key it had just inserted and the blocks it picked
followed what that cook thread had run before. The UASTC textures
(`stone_n`, trickster's 1 and 2) differed, which moved their hashes in
the manifest, and the test named only the first differing file, the
manifest (it sorts first). Found with valgrind in a trixie container
(8,282 uninitialised-value errors, all at that map's insert and find in
`uastc_rdo_blocks`); upstream has the same fix; our `basis-universal-sys`
0.3.1 bundles basisu 1.16, older than it. Never reproduced on the Mac
(clang's garbage repeats): 45 cooks at 2, 8 and 12 threads, 26 under
emulation, 9 loops under load, all equal. The candidate causes ruled out:
the cook times were never in `.cook.ogt` (they live in `.cook-times.ogt`,
which the test excluded), every result, note, warning and removed entry
is collected by plan index, no hash map's order reaches a file, groups
are assigned before anything cooks.

- 0b245141: upstream's one-line fix carried as a trimmed copy of
  `basis-universal-sys` 0.3.1 in `third_party/basis-universal-sys`
  (4.2 MB: encoder, transcoder, `zstd.h`, licences), swapped in by
  `[patch.crates-io]` with `third_party` excluded from the workspace;
  valgrind then reports 0 errors and the Mac's cooked bytes do not change.
  The alternative not taken: UASTC RDO off by default (blobs 10 to 33 %
  larger, the template's `cooked/` 2,068 to 2,164 KB, the cook about twice
  as fast); `-ftrivial-auto-var-init=zero` is not enough (gcc does not
  zero the temporaries the map is called with).
- 9ffc36cb: og-render's pipeline cache wrote every renderer's save of the
  same key to the same `<key>.tmp`, so two concurrent saves (two og-gfx
  runs, or bun test processes on the addon's default
  `~/.cache/origin-engine/pipelines`) could leave a torn file; each save
  writes its own temp file now.
- 78b8d0d8: `every_thread_count_cooks_the_same_bytes` (30 cooks of a
  smaller template copy, 10 each on 1, 2 and 8 threads, the three series
  at once, every file against the first one-thread cook, the cook times
  compared by asset list; 9 s in a dev build; an injected
  thread-dependent byte fails it on the first two-thread run);
  `cooking_twice_gives_identical_bytes` lists every differing file with
  sizes and the first differing byte; the README's §Determinism and tests
  table.
- Row 81 (the particle test's one SIGSEGV on Lavapipe): nothing unsafe
  or racy in og-render's CPU particle path or the test (slots sized to
  `max_particles`, the upload buffer recreated on change, no buffer
  resized while mapped, the readback after the GPU finished); the test
  runs two renderers with their own Vulkan devices on two threads at
  once, so a Mesa/Lavapipe thread-safety crash is the best reading, not
  confirmed (the run's log was gone). No change.

Functional lane on the merge: 5 of 5 in 156 s.

## Wave 10, ci-green: CI on GitHub Actions, then off for the build-out (53b876f)

Branch `ci-green`, merging main as it moved (45722335, 9b61c2ad,
ee24f0c0, bffc91fc, 97d1996f, b1be5af1, 8050347e), head 3816e4a4,
merged at 53b876f7. Its job was the workflow green on GitHub's runner;
the repository's owner then took GitHub CI off for the build-out
(9d220790, below).

- What ran on Actions: three green jobs on a tree merged with main,
  `quick` in 12.5 min, `functional` 35 min and `gate` 60 min (after one
  re-run: a particle test crashed with SIGSEGV on the first, row 81); a
  dispatched gate 94 min and the perf nightly 145 min, both cold. The
  final head was not run: the account's Actions budget was exhausted.
- 84b6ba2f, d086425a: every job runs in a `debian:trixie` container with
  `--init`, whose Lavapipe (LLVM 19.1.7) is the og-lavapipe image's that
  records the `vulkan-cpu` references; on the runner's own Ubuntu 24.04
  (LLVM 20.1.2) `atmosphere-night` and `atmosphere-moon` came out at
  RMSE 0.0105 and 0.0103 (tolerance 0.01). `--init` reaps orphans as a
  machine does (`og dev`'s signal test saw its stopped game as a zombie
  under the container's bare PID 1).
- 1d00b96e: one provisioning action (`.github/actions/provision`: disk,
  toolchain, a `target/` cache per lane, Lavapipe, ALSA's headers, node,
  the pinned tools), a functional job beside quick on PRs, uploads that
  survive the exhausted artifact quota, `scripts/perf.sh` for the nightly
  (`--measure`: no baseline exists for a shared runner); e2cd6c22 the
  timeouts from the measured runs (quick 45 min, functional 90, gate 240,
  perf 240) and perf.sh printing each test's frame time.
- e13a4b91: no `A && B || C` in the gate scripts (SC2015, which CI's
  ShellCheck 0.9 failed), and `install-tools.sh` pins ShellCheck 0.11.0.
- Settings for the two-vCPU runner: a295e7b4 `OG_BUN_TEST_TIMEOUT` (60 s
  in CI, passed to `bun test --timeout`; render tests took 5 to 25 s on
  Lavapipe) and `OG_GATE_JOBS=1` in the gate job (bun-test beside test
  starved both), render.test's untonemapped sprite within one step of 188
  (Lavapipe's sRGB encode gives 187; 711ecc7e its matcher); 454c7433 the
  knowledge and create packages pass the timeout too; b364c585
  `OG_TEST_TIME_SCALE` (3 in CI) for the two tests with long limits of
  their own (the terrain station's and the chunked music's `og run`s).
- a23d99e5, the one engine change: an og-tests gfx scene with no
  reference on the adapter's tier is judged against `metal`'s and says
  so, frame by frame as og-ui, the templates and the browser tiers do,
  which the vulkan-cpu copies pruned by f29ffef9 need. 7d677e2f deletes
  og-ui's stale vulkan-cpu `text-bidi` (it predated a2c5e44b's metal
  re-record and missed CI's Lavapipe at RMSE 0.02025; judged against
  `metal` now).
- ddec5756, a56a2ccb, 3816e4a4: `scripts/wave-end.sh`, the wave's end in
  one serial run holding both gate locks, stages `gate`, `web`, `cross`,
  `prepare`, `gfx`, `ui`, `playtests`, `table` (og-ui's GPU tests in the
  image, `--record og-ui` recording their vulkan-cpu frames; every
  template, archetype and example with `references/vulkan-cpu/`, the
  examples included), and `scripts/frames-vs-metal.ts`, its table of
  every vulkan-cpu frame against `metal`. Only ever run with `--dry-run`
  (row 82).
- Docs (cb33649f, 1327549d, e2cd6c22, 3816e4a4): docs/ci.md §CI (the
  container and why, the provisioning, the jobs with their measured
  times and timeouts, the nightly as a measurement, the artifact quota),
  §The wave-end run, §When to run what; AGENTS.md's gate bullets.
- Not fixed, intermittent in CI: og-cook's
  `cooking_twice_gives_identical_bytes` failed 2 of 5 runs (the cook's
  manifest differs between thread counts, row 80) and the particle
  test's SIGSEGV once (row 81).
- The decision (the owner, 2026-10-21; 9d220790): GitHub CI is off
  during the build-out. The workflow runs by hand only
  (`workflow_dispatch`; push, pull_request and the nightly schedule off),
  the local gate is the signal (the functional lane per merge, the
  wave-end run per wave), docs/ci.md §CI says so; the triggers come back
  when the repository is published (row 83).
- Frames: no `metal` frame moved on the branch (its one deletion is
  og-ui's vulkan-cpu `text-bidi`, above). With its merge the coordinator
  re-recorded roguelike's `dive` on `metal` (9d220790; lampdive's success
  tone from cook-ui-words, RMSE 0.0146).

Functional lane on the merge (at 9d220790, with the workflow set to dispatch
only and roguelike's dive frame): 5 of 5 in 119 s.

## Wave 10, cook-ui-words: the cook's unread Tiled properties, the templates' words, the badge palette (a31d6cb)

Branch `cook-ui-words` from main at e16e9cec, merged main at 4e6c46f0
(f29ffef9) and merged at a31d6cbe after physics-3d. Rows 75, 76 and 69.

- 64f79249, row 75: `og cook` warns through `Report.warnings` on every
  Tiled custom property the importer does not read (a tile layer's other
  than `autotile`, `collision`, `lit`, `z`, `repeatx` and `repeaty`, or
  one of those with a value of another type; an embedded tile's other
  than `solid`; the map's, a group's, an object layer's and an image
  layer's own; an object's class-typed one), naming the property, where
  it sits and the names read there, on every cook (an up-to-date map
  warns again), failing nothing. Test in og-cook `tests/tilemap.rs`;
  og-game §8 says so and its Tiled table gains the tile's `solid`.
  f3bf909e: Hollowbell's `tools/make-assets.ts` wrote the far layer's
  `repeat` = x, which the new warning named; it writes `repeatx` true and
  the hills fill the sky left of x 333 px where they used to stop (title,
  play and results re-recorded on `metal`: RMSE 0.0872, 0.0870, 0.0320,
  every change in the hills' region; README, FINDINGS #2 and #12).
- 77cb5b0a, row 76: `InventoryDesc.hintLabel` and `countLabel`,
  `LeaderboardDesc.retryLabel`, `TutorialDesc.skipLabel`, `nextLabel`,
  `doneLabel` and `stepLabel` ("{step} of {count}", filled by the
  template), `CutsceneDesc.skipLabel`, each with its English default.
  `templates/ui-kit` passes them from its five tables, an
  `offline:<on|off>` command shows the leaderboard's error and Retry, the
  JP, KR and Arabic subsets re-made by `make-locale-fonts.py`, and the
  locales playtest asserts each in Chinese with no English left (and the
  tutorial's defaults in English); og-i18n §2 lists the fields, with its
  Japanese count corrected to 246.
- dcd8c31e, row 69: the badge's 16 % tint of its tone mixes as the
  platform's CSS does (`Srgba::tint_over`: kept translucent, at the alpha
  whose linear blend has the sRGB mix's luminance) instead of in linear
  light, where a light ink's tint lifted a dark panel so far that no ink
  reached 4.5:1 on the pixel pack's. Then the palette: the tones deepened
  on the light-panel skins (cartoon, fantasy and Ember Vale's copy,
  paper, pathguard; cartoon's warn #f2a516 at 1.63:1 is #7b5102),
  lightened on the dark ones (sparkorb's danger, lampdive's success and
  danger), `accentSoft` more transparent on sparkorb, lampdive and
  lanebeat; meters, toasts and status dots keep their hues;
  `make-skins.py` reproduces the packs byte for byte. `pack_contrast.rs`
  enforces success, warn, danger, info and the accent badge at 4.5:1 in
  a panel (and on the screen where both are dark or both light) for the
  seven packs and sparkorb, lampdive, lanebeat and pathguard; the neutral
  and inverse badges are printed. og-taste §3 and og-ui's README say so.
- abd2bf9b: a meter's label is drawn twice, clipped at the fill's edge
  (`Shape::Text` gains a clip frame), each half in whichever of `fg` and
  `fgInverse` reads better on what it is over: the deepened greens put
  cartoon's "72 / 100" at 2.3:1, and neon's white sat on bright green at
  1.3:1. Test in og-ui `tests/widgets.rs`.
- Frames re-recorded on `metal` (RMSE against main's; the sheets compare
  per cell, each moved a cell past 0.01): og-ui `widget-set` 0.01137,
  `kit-phone-widgets` 0.01611, `kit-phone-shooter` 0.01235,
  `kit-phone-racing` 0.01409, `kit-skin-cartoon` 0.00727 and `-phone`
  0.00971, `-fantasy` 0.00333 and `-phone` 0.00491, `-neon` 0.00363 and
  `-phone` 0.00615, `-paper` 0.00319 and `-phone` 0.00473,
  `kit-skin-pixel-phone` 0.00338, `kit-skin-scifi-phone` 0.00388;
  Hollowbell's three above. After the merge roguelike's `dive` moved
  (lampdive's success tone, 0.0146), re-recorded by the coordinator at
  9d220790 with the ci-green merge. Their vulkan-cpu copies wait for the
  wave's end (the gate section above).
- Left out: the archetype skins' buttons and fields (sparkorb primary
  3.42:1, lampdive's danger button 2.58:1 and placeholder 3.57:1,
  pathguard primary 3.30:1) and the examples' own skins (Hollowbell
  danger 4.26:1; Lantern Vale success 3.15, warn 2.72, info 4.07:1), row
  77; the neutral badge (scifi 2.15:1, neon 2.81:1) and the light packs'
  badges on a dark stage (about 2:1), row 78; Hollowbell's cutscene
  "Skip", still English in Chinese (it needs a table key and 过 in its
  subset), row 79.

Functional lane on the merge: 5 of 5 in 138 s.

## Wave 10, physics-3d: one mover for both controllers, collision layers (bb7c1e8)

Branch `physics-3d` from main at e16e9cec; work commit 876dd4ea, merged
main (f29ffef9) at bb7c1e81, fast-forwarded. Rows 72 and 73.

- Row 72 measured first, on rapier's character controller with Lantern
  Vale's capsule (radius 0.3, 1.6 m, 6 m/s): a run stalled to 0 to
  0.4 m/s for a tick about once a second on one box, on adjacent boxes
  and on a triangle-mesh floor; the capsule stood 1 mm above the floor
  instead of its 2 cm skin and crept up 0.1 mm a tick; one placed on the
  floor sank 9 cm; 0.2 m stairs with 0.3 m treads stopped it at the
  first step; a 30° slope was climbed at 4.4 m/s across.
- 876dd4ea: the platformer's move (e86b55a5) becomes
  `crates/og-physics/src/mover.rs`, one mover for `og.CharacterController`
  and `og.PlatformerController` in both profiles: exact casts, the skin
  kept by the controller, lift two skins, move across, settle; walkable
  slopes at full speed across; a ray just inside an edge for the ground
  under a shape resting on it; a step tried from `stepHeight` up when the
  low move is blocked, taken only when it goes further and lands on
  walkable ground (not a dynamic body) at most `stepHeight` over the
  ground it stood on; `gravityScale` 0 slides freely. The slope limit's
  cosine is `og_core::math::cos`, for determinism. The IDL docs of both
  controllers, og-game's controllers paragraph and the og-physics README
  (§The controllers' move) say so.
- `crates/og-physics/tests/character_ground.rs`, 7 tests, each failing on
  the old controller: full speed every tick with the centre one skin
  above a box, straight and diagonal; adjacent boxes; a trimesh and a
  height field; 190 placements standing a skin over the floor; stairs up
  and down; steps of 0.1 to 0.345 m climbed at `stepHeight` 0.35 and of
  0.36 to 0.6 m not; a 30° hill both ways at speed and a 50° slope
  stopping it.
- Row 73: both controllers' query filters take the collider's collision
  groups, so `og.Collider.layers` and `mask` decide what blocks them as
  for bodies; two tests in `tests/controllers.rs`.
- Determinism: physics_hash 947364242694612608 to 15539601638348908910
  by design (the 3D walker stands a skin above the floor); the 2D
  scenario's world hash unchanged at 12448362945291953010 (the shared
  move leaves the platformer as it was); math_digest, bouncing_boxes_hash
  and anim_hash unchanged.
- Frames: moved within tolerance only, none re-recorded (Lantern Vale
  `far` 0.0055 and `slime` 0.0025, third-person `beacon` 0.0020,
  fx-showcase `terrain` 0.0065; the runner archetype byte-identical).
  Every archetype, example and template suite passes after the merge
  but topdown's combat, which failed on main already: the test sampled
  the bite's knock-back two ticks into the hit stop; the coordinator's
  d5228a31 reads it at 0.3 s (12.3 to 11.80 m).

Functional lane on the merge: 5 of 5 in 158 s.

## Wave 9, ui-runtime: live resource handles, game saves, the templates' words, the pixel pack's labels (517d8c4)

From Hollowbell's findings; ten of the twenty. Merged main twice
(0c347445 after render-2d and knowledge-2, 5579bd2d after physics-2d and
8b083f00) and fast-forwarded.

- df79a7dd, #6 (blocker): a `world.resource()` handle is live: every
  getter reads the store through `og_world_resource`, the value kept only
  while no engine call or dispatch happened since, so a loop reading a
  tuning field costs one call (4.3 µs cold on the dev addon, 0.008 µs
  warm); og.Time's handle moves with the ticks. Tests in
  `packages/engine/tests/runtime.test.ts`. Hollowbell keeps one handle
  per scope as plain practice, its comments marked fixed.
- cb57ea4c, #1: `world.snapshot()` and `world.restore()`, a game's save
  of the whole world in every build, publish included, the same on both
  hosts. The save is text, `og-save:1:` and the base64 of the deflated
  world snapshot with every type's schema hash and a checksum, a JSON
  value for og.platform's store (Hollowbell's world 25 KiB, an empty one
  11 bytes). A restore brings the world back to its hash and keeps the
  session (input, og.Audio's volumes, the platform's calls and clock
  shifted with og.Time's jump, the probe's scheduled steps); from a
  system or a command it applies when the tick ends (the probe's new
  after-tick hook). A save whose types changed is refused naming the
  type and the fields that differ (`World::check_snapshot`), or mapped
  with `{ migrate: true }`. The engine bug it found: a snapshot taken
  inside a system met the columns that system holds (its typed views
  stay borrowed while it runs) and the entities its queued spawns had
  reserved, alive and in no archetype yet; `SystemContext::snapshot`
  now reads the held columns as they are and writes those entities
  freed, so the world it restores to is whole (og-ecs
  `a_system_snapshots_the_world_while_it_holds_its_columns`). Tests:
  og-ecs `snapshot.rs` and `migrate.rs`, `packages/engine/tests/save.test.ts`.
  og-game §17 and og-core say how to save progress; docs/design.md
  updated. Hollowbell keeps its JSON save record by choice
  (`src/save.ts` says why: a checkpoint game restarts at its lantern,
  the record survives any change to its types, and a restore would bring
  back session state Continue would have to reset).
- 8a7d8702, #11 and #12: `SettingsDesc` gains `title` and a label per
  tab, slider, toggle, preset and Done; `ShopDesc` `priceLabel`,
  `buyLabel`, `shortLabel` (Not enough), `ownedLabel` and `emptyLabel`,
  each with its English default (the rule of `ResultsDesc.bestLabel`).
  `templates/ui-kit` passes them from its five string tables
  (`settings.*`, `shop.*`), with the JP, KR and Arabic subsets remade by
  `make-locale-fonts.py` (which reproduces the committed ones from the
  sources its README now names), and its locales playtest asserts the
  settings screen and the shop in Chinese with no English left; og-i18n
  §2 lists the fields. Hollowbell's shop passes its tables' words and
  its locales test asserts no English word of the template is left (its
  Chinese subset re-made); its own settings dialog stays by choice
  (`src/ui.ts` says why: one page with exactly its settings).
- 3a78409c, #10: a dialogue box takes the focus on the frame it appears,
  unless a dialog is open over it, so a cutscene answers Enter, Space
  and A from its first line (dialogs already did). og-ui `tests/widgets.rs`
  opens a dialogue over a focused HUD button and a dialog by state;
  og-ui §4 says what opens takes the focus. Hollowbell's navigate
  workaround is gone.
- ad36ce7c and e528effe, #20: the finding's diagnosis was wrong. The
  label was always drawn in `accentFg`; DotGothic16's one-pixel strokes
  fell between device pixels at 16 px (its dots 61.5 units of 1000 with
  a 28-unit side bearing), so no pixel took more than about half the
  ink. og-ui skins take a `textWeight <role> <device px>` line (the
  role's glyphs and their shadow thickened through the distance field);
  the pixel pack has `textWeight body 0.5` with DotGothic16 cooked at
  `px_range 8` (at 4 the shader allows no thickening at 16 px), and
  `make-skins.py` still reproduces every pack byte for byte.
  `pack_contrast.rs` gained a test that draws every pack's primary and
  secondary buttons through og-render with the pack's own cooked fonts
  and reads each label in every state from the frame as og polish does:
  the old pixel pack's primary label measured 3.44, 3.76 and 2.96:1
  (rest, hovered, pressed), now 9.11, 12.69 and 5.92:1. og-taste §3 says
  what the test reads and what a copied pack needs. Hollowbell's copy of
  the pack takes the same two lines at e528effe; its pale-gold accent
  and 16 px ramp stay as its look.
- 536f8397, #17, #18, #19 (probe nits): `get`, `dump_entity` and the
  dumps answer an enum by its variant's name (`set` and `patch` take the
  name or the index; `entity` keeps numbers for Tune); a step naming a
  file an earlier step of the same script wrote is written beside it as
  `<stem>-2.<ext>` with a warning naming both steps; output names
  starting with `.` are refused, the output directory keeping them for
  the harness, and a playtest's `store: "keep"` store moved to
  `<out>/.store/platform.json`. Tests in og-probe `tests/commands.rs`;
  the IDL's command text at cb57ea4c. Hollowbell's music test reads the
  bus by name.
- 7158ad23, #14: the in-repo examples type-check: Hollowbell's and
  Lantern Vale's `tsconfig.json` map `@origingame/engine` and its
  playtest module through `paths` to `packages/engine`, with a
  `host.d.ts`, and `bun run typecheck` runs `tsc`; og-game §2 says how a
  project type-checks.
- Frames re-recorded on `metal`: og-ui `kit-skin-pixel` (RMSE 0.01266,
  0.409 % outliers) and `kit-skin-pixel-phone` (0.00971, 0.310 %) at
  ad36ce7c; Hollowbell's nine at 1e706a5a and again over main's at
  517d8c4e, against main's references: title 0.0166, play 0.0080, combat
  0.0088, gloam 0.0076, shop 0.0221, pause 0.0164, settings-zh 0.0172,
  results 0.0222, talk 0.0513 (the old reference showed the workaround's
  focus ring round the box). The Hollowbell, Lantern Vale, ui-kit,
  fx-showcase and third-person suites pass unchanged after the merge.
  Owed: the pixel skin frames' vulkan-cpu copies (abd61fb8) predate the
  label weight; Hollowbell's frames are `metal` only (row 74).
- Left out: the other templates' English literal words (row 76);
  Hollowbell keeps its JSON save record and its own settings dialog by
  choice; no `og check` command (`og` still bundles without
  type-checking; the knowledge package says to run `tsc`).

Functional lane on the merge: 5 of 5 in 147 s.

## Wave 9, physics-2d: the platformer's casts, knock-back, the kinematic rule (08a3339)

From Hollowbell's findings (examples/hollowbell/FINDINGS.md), the second
dogfood game, a 2D pixel platformer written from the knowledge package.

- e86b55a5, #16 (blocker) and #15: `og.PlatformerController` stuttered
  on flat ground (its velocity dropping to near 0 every few ticks, a
  second of running covering 4.7 m at a 6.5 m/s top speed) and sank 7 mm
  into a tilemap floor at some x, unable to run. Cause: rapier's
  character controller casts with its skin as a target distance, which
  parry resolves inaccurately (1.98 cm of travel reported for 1 cm, from
  2 cm above a 74 m floor), so a run grazing the floor hit it at random
  with a tilted normal and rapier's 2D slope rule threw the whole step
  away (no horizontal direction to slide along in 2D); and a cast that
  starts inside the skin falls back on a contact query that returns
  nothing at some positions, so the character fell 11 mm in and stuck.
  The controller no longer uses rapier's character controller
  (`crates/og-physics/src/platformer.rs`): exact casts (accurate to
  2e-5 m from 1 mm away), the skin gap kept by the controller itself,
  and when not rising it lifts two skins, moves across (keeping its x
  speed up walkable slopes) and drops back onto the ground a skin above.
  Six tests in `crates/og-physics/tests/platformer_ground.rs`, all
  failing on the old code: full `maxSpeed` every tick on one box, on 12
  adjacent boxes and on tilemap collision rectangles (og-tilemap as a
  dev-dependency), a 30° hill at full speed and a 60° slope stopping, a
  controller written to the floor's height at 190 positions standing and
  running, the feet one skin above the floor across repeated runs. In
  the game the #16 repro reads 6.5 m/s every tick and runs 6.14 m in a
  second; the #15 repro stands at y 16.01 and runs.
- 1e47f188, #5: `og.PlatformerController.impulse` (Vector2, m/s), added
  to the velocity on the next step then reset; an upward one leaves the
  ground and cancels coyote time. IDL, og-feel §3 (beside hit stop),
  og-game §13; a controllers test.
- 177af1c5, #13: one rule, the one the skill promised: a kinematic body
  with a nonzero `og.Velocity` moves by it each tick and its Transform is
  written back; writing its Transform teleports it; without a Velocity
  it stays where its Transform puts it. The IDL (BODY_KINEMATIC,
  RigidBody.kind, Velocity), the og-physics schema docs, og-game §13,
  docs/design.md and the og-physics README state it; the bodies test
  that asserted the old behaviour is replaced by one test per rule, and
  a controllers test rides a platform moving by its velocity. A game that
  moved a kinematic body's Transform by hand and also set its Velocity
  would now move twice; Hollowbell's lift was the only case in the
  repository (af5b3d39: the lift reverses its velocity at the ends, the
  hero's knock-back is an impulse, `WARP_LIFT` is gone, `move.og.ts`
  checks the run distance against the tuning, 2.89 m measured against
  0.5 × 6.5 − 6.5²/140 with 0.15 m tolerance).
- Determinism (the native tool, lanes off): physics
  18126409214070009362 before; 6597690808174327434 after the platformer
  move (by design, the 2D scenario's walker moves differently);
  947364242694612608 after `impulse` (the component record grew),
  unchanged by the kinematic rule (the scenario's platform has no
  Velocity) and after the merge. `math_digest`, `bouncing_boxes_hash`
  and `anim_hash` unchanged. The og-physics README records them.
- Frames: none moved (Hollowbell 13, Lantern Vale 11 and the runner
  archetype's suites pass unchanged after the merge; the courier stands a
  skin, 1 cm, above the floor within tolerance).
- Left out: `og.CharacterController` (3D) still uses rapier's controller
  and likely carries the same cast inaccuracy (row 72); the platformer's
  query filter ignores the collider mask (row 73).

Functional lane on the merge: 5 of 5 in 165 s.

## Wave 9, render-2d: lit tile layers, the pixel grid's fit, sprites over the UI (6eba088)

- 9c5b5d6f, #9 (blocker): tiles were drawn with the lit flag off, so
  `og.Lighting2D` never lit a tilemap. A tile layer inside the lit range
  is lit as a sprite there is (ambient, each Light2D's falloff and
  colour, occluder shadows), through the same sprite pipeline and light
  map on every tier. Opt-out: a Tiled layer's bool property `lit` false,
  or `lit false` in a `.ogt` layer block, carried by the cooked blob's
  new `UNLIT` bit (maps cooked before stay lit), reported by
  `TileLayerInfo.lit`; the IDL docs of the tilemap module and Lighting2D
  say so, and og-game §8's grammar and Tiled table carry it (48d172f3,
  after the knowledge package's drift check asked for it). Tests:
  `tile_layers_in_the_lit_range_take_the_2d_light` (fails with the old
  flag), og-tilemap's and og-cook's tests reading `lit`, and the gfx
  scene `tilemap-lit` (a dark cave of three tile layers, a torch and a
  crystal, a pillar's and a ledge's shadows, a window sky layer opted
  out) with its metal reference.
- aff16274, #7: the pixel grid's fit rule: the height sets the scale;
  `pixelUpscale` is honoured whenever height × scale fits the output's
  height (with the border that leaves), else and for 0 the largest scale
  that fits; the width may lose its last column to keep the scale. 240
  lines at 1280×720 draw at 3x as 426 columns with a one-pixel border
  each side instead of 2x inside a wide one. The IDL strings of
  `pixelHeight` and `pixelUpscale`, og-taste §12 (with knowledge-2's
  table of scale and border per `pixelHeight` at 720p, 1080p, 1440p, 4K
  and three phone sizes) and the README give the arithmetic; a unit test
  covers 720p, 1080p and a 2532×1170 phone both ways.
- dd7c0685, #8: a render constant `UI_LAYER` = 1000 (exported from
  packages/engine): screen sprites and screen text at 1000 or below draw
  under every UI element as before; above it they draw over every UI
  element, modals and the transition cover included, in their own
  order, so a game draws such sprites only while their screen shows.
  og-game §10's sentence rewritten; og-ui `tests/screen_layers.rs` with
  reference `screen-layers.png` (fails when the rule is removed).
- Hollowbell (6eba0883): the `MapShade` tint, the light-pool sprites,
  their system, component and asset are gone, the map in the lit range;
  light multiplies the map now instead of a glow added over it, so the
  near-black back wall is redrawn at its full-light colour, the lights
  about twice as bright and Wren's lantern fades in daylight;
  `lighting.og.ts` measures 61 near Wren and 56 under the torch against
  26 for the rock; the look back at 240 lines with a 15-cell view.
- Frames re-recorded on metal: `stylise-palette` 0.26666 and
  `stylise-retro` 0.55622 (240-line scenes, now 3x); new `tilemap-lit`
  and og-ui `screen-layers`; by the coordinator at 48d172f3, fx-showcase's
  `looks-pixel-3d-hybrid` (0.50), `looks-low-resolution-3d` (0.38) and
  `looks-monochrome-dither-3d` (0.70), the 240-line presets, and all nine
  Hollowbell frames (0.03 to 0.13). Owed on vulkan-cpu and the browser
  tiers: `tilemap-lit`, `stylise-palette`, `stylise-retro` (row 74).

Functional lane on the merge (with knowledge-2): 5 of 5 in 97 s after
the grammar line; the first run failed only the knowledge package's new
drift check on `lit`.

## Wave 9, knowledge-2: the grammars with drift checks, Japanese fonts, what works (49d7a1d)

Every wiki page is generated from the IDL (the validator refuses a page
without the generated header), so the grammars that are not IDL went into
the skills.

- d7e3cd02, Hollowbell #2 and #3: og-game §8 gives every line of a
  `type tileset` and a `type tilemap` file with defaults, the
  neighbour-mask rules of both auto-tiling modes, the topdown archetype's
  `dungeon.ogt` as the example, how `../tiles/x.tsx` resolves to
  `tiles/x.ogt`, a table of the Tiled fields the cook reads, and a map's
  import settings; og-game §10 gives every emitter line with its default,
  the one-shot pattern and Hollowbell's `fx/hit.ogt`; `og.load.emitter`'s
  IDL doc points there (the "particles page" it named never existed).
  The drift check in `scripts/validate.ts` runs both ways: a word in a
  doc block the parser never reads fails, and a key the parser reads that
  no doc block writes fails, the Tiled table against the importer too;
  `tests/grammar.test.ts` (10 tests) proves each kind of drift is caught.
- 95a82284, #4: og-i18n §3 says what the repository ships (the ui-kit's
  Japanese subset holds only its own 177 characters), where a whole Noto
  face comes from, how to subset with `templates/ui-kit/tools/
  subset-font.py` (the examples' tool, promoted and made general: it
  exits 1 naming every character the source font lacks), and the
  fallback rule: never take Japanese from the Chinese face.
- 5e1a84b1, 3edec3f0: og-taste §12's pixel table (computed with
  render-2d's fit rule) and phone guidance.
- 30dc7d75, 49d7a1d0: every skill but og-ai opens with a "What works"
  section of facts the two games measured; og-game opens with the
  seven-step order both games found productive and the three mistakes
  (the resource handle rule, a look replacing resources whole, where probe
  outputs go); og-build §1 and §3 say where outputs go.
- Found, not fixed: Hollowbell's far-hills layer sets a string property
  `repeat` = x that the cook does not read (only `repeatx`/`repeaty`);
  its FINDINGS #2 says `repeat` was confirmed by `TileLayerInfo`, which
  has no such field. A cook warning on unknown Tiled layer properties:
  row 75.

## Wave 8, dogfood-2: Hollowbell, the second dogfood game (5cc97a9)

Branch `dogfood-2` from main at 5e50479f, merged main (wave 8's three
streams, to 84422979) as 75183a78, its README and findings rechecked
there at 5cc97a90 (the playtests and og polish gave the same results) and
fast-forwarded. An example only: no engine code changed.

- **The game** (c16d6e4c, a9401332, 15430319, 62e7ae58, 86da6e32,
  3981652f): `examples/hollowbell`, a five-minute 2D pixel-art platformer
  written from the knowledge package as a game author would, to exercise
  the engine's 2D half as Lantern Vale did its 3D half: a Tiled map over
  a blob47 auto-tiled tileset with parallax layers and 100-odd objects,
  `og.PlatformerController` (one-way planks, an air jump, a kinematic
  lift), flipbooks with frame events, `og.Lighting2D` with lights and
  occluders, a look on a pixel grid with the game's own palette and a
  second look for the results, texture arrays, particles, hit stop and
  knock-back, `og.Follow2D` and a camera zone, og.ui on a copy of the
  pixel skin pack in English and Chinese, `str(N)` fields, saves to
  og.platform's store, streamed music. Every asset is generated by
  `tools/make-assets.ts` (69 cooked cold in 3.6 s). Thirteen playtests
  (move, combat, gloam, shop, save, settings, name, win, locales,
  lighting, music, a two-process replay, frames), nine reference frames
  on `metal` (title, play, combat, talk, shop, pause, settings-zh, gloam,
  results); og polish taste 99 (contrast 93: the pixel pack's primary
  button labels, #20), feel 92 (frame time 72 on a busy machine).
- **`FINDINGS.md`**: 20 findings, each with its repro and workaround: 3
  blockers (#6 two `world.resource()` handles disagree, #9
  `og.Lighting2D` never lights a tilemap, #16 the platformer's run
  stutters on flat ground), 10 annoyances (#1, #2, #3, #7, #10, #11,
  #12, #13, #15, #20), 7 nits (#4, #5, #8, #14, #17, #18, #19); top
  five #9, #16, #6, #1 (no game-side world snapshot), #11 (the settings
  screen English in every locale) with #12 (the shop's words).
- **Taken by wave 9**: physics-2d #5, #13, #15, #16; render-2d #7, #8,
  #9; knowledge-2 #2, #3, #4; ui-runtime #1, #6, #10, #11, #12, #14,
  #17, #18, #19, #20 (the sections above, and
  `examples/hollowbell/FINDINGS.md` §After wave 9 for each finding's
  commit and workaround). Left open: #12's remainder, the other
  templates' English words (row 76); #2's remainder, a `repeat` layer
  property the cook does not read (row 75); `og` type-checking a project
  itself (#14, no `og check`, not a row).

Left out: the frames on the other tiers (row 74).

## Wave 8, wasm-size: where the lanes module's bytes are, two trims, a report tool (bf8dcd1)

The lanes module was 3,498,640 B brotli (83.4 % of `wasm_core_brotli_bytes`)
at 161bad3b. Measured per crate (each crate's part brotli-compressed on its
own, `scripts/wasm-size-report.ts`): og-runtime 362 kB, rapier3d 327 kB,
the data section 324 kB, rapier2d 246 kB, og-physics 232 kB, og-render
232 kB, parry3d 203 kB, the exports with og-runtime's bodies inlined
199 kB, naga 192 kB; the two physics profiles together about 935 kB; the
largest functions `Renderer::render` (138 kB), `PhysicsState::decode`
(54 kB), og-runtime's `extract` and `assemble`. Raw size points at the
wrong things: 230 kB raw of rayon generic copies were 9 kB of brotli, and
the `wasm-gate` profile is about 6.6 % larger than what ships.

- 6d3f45a4, og-lanes: `map`, `for_each_mut` and `for_each_indexed` go
  through one non-generic parallel loop instead of a rayon copy per item
  and result type (about 200 copies): 3,498,258 to 3,489,535 B (8.7 kB);
  results the same item for item, og-physics' split-against-serial lanes
  tests pass.
- c51e8d05, og-render: a `build.rs` copies `shaders/` with its `//`
  comments removed and a `wgsl!` macro includes that copy; every token
  and line stays where it was, the `.wgsl` files are untouched, a test
  checks the copies line for line against the sources (one anchor in the
  terrain code found a comment and now finds `fn band(`): 3,489,535 to
  3,466,050 B (23.5 kB). The functional lane's GPU reference tests pass
  on the merge: no frame moved.
- bf8dcd16: `scripts/wasm-size-report.sh <module>` builds and optimises
  a module as it ships and prints the size gate's number;
  `scripts/wasm-size-report.ts` gives each crate's raw and own-brotli
  bytes and the largest functions; `tests/evidence/P4/wasm-size/` holds
  the README and notes; docs/ci.md "When the size gate fails" points at
  the script.

After the merge with 161bad3b (shipping `wasm` profile, brotli -q 11):
lanes 3,464,929 B (82.6 %, from 83.4 %), 3D publish 3,048,477 B
(72.7 %), 2D publish 2,882,459 B (68.7 %). Under 80 % needs about
143 kB more, and the one thing that gives it without removing a feature
is a lanes module per physics profile (row 71). Tried and not paying:
og-host-web at opt-level z (5.9 kB larger), `RECORDS` as a static (the
copies exist only in `wasm-gate`), the probe's command code behind
`publish` (og.ai sends its commands through the probe's inbox),
cook-only code (none in the module), `format!`/Debug sites (about 40,
nothing worth taking). Rejected: og-runtime at z (frame time),
`panic_immediate_abort` (a lane's panic must wake the engine thread),
`optimize_for_size` for std (could reorder unstable sorts and move a
hash), dropping field docs from publish (reflection serves them),
`-Zlocation-detail=none` for lanes only (81.0 %, and every crash on a
lanes page loses its file, line and column).

Functional lane on the merge: 5 of 5 in 177 s.

## Wave 8, taa-webgl2: the GL backend read TAA's depth as 0 (f3229ef)

Row 68's cause, found natively on ANGLE's Metal renderer (the GL layer
Chrome's WebGL2 uses on macOS; wgpu's `angle` feature enabled locally,
not committed): `taa.wgsl` read the `Depth32Float` buffer with
`textureLoad`, wgpu's GL backend binds no sampler to such a read, the
depth texture stays on GL's default linear filtering, which GLSL ES 3.0
calls incomplete, and an incomplete texture reads 0. So every pixel was
reprojected from the near plane: wrong motion for a moving camera
(`taa-sharp` at RMSE 0.074 against `metal` on ANGLE, poles broken into
dashes), and for a still camera at the looks station's 2,800 m offset
the near-plane points lost enough precision that the current and
previous projections rounded differently, up to 0.025 px of false
motion per frame, above the shader's 1e-3 px own-texel test, which is
the sideways drift of render-style round two (a standalone TAA harness
measured it: 0.025 px on ANGLE with depth 0, 5.5e-4 px with the real
depth, 3.8e-5 px on Metal). naga's GLSL declares `precision highp`, so
precision qualifiers were not it; given the depth, the shader matches
Metal on ANGLE within one half-float step for still and moving cameras.

- 1a9037a9: the resolve reads the depth through its own nearest sampler
  (binding 5) at the pixel's centre, the same texel as before, so the GL
  backend binds a sampler. cf5f2b9's CPU still-camera check stays. On
  ANGLE `taa-sharp` goes to RMSE 0.00028 and `taa-motion` 0.00010 against
  `metal`. `taa-sharp` applies on every tier (`scenes::Applies`), so
  web-reference judges a moving-camera TAA frame against `metal` on
  WebGL2: 0.00028 there, 0.00024 on WebGPU (the coordinator's run at
  f3229ef, every gfx scene and the looks station passing on both tiers;
  WebGL2 looks worst neon-vector 0.0098 against `metal`). og-render
  `tests/shaders.rs`: TAA's GLSL must sample the depth with `textureLod`,
  never `texelFetch` (fails on the old shader). og-tests `tests/gfx.rs`:
  a still camera converges to a fixed point at the same sub-pixel
  position across the frame (a fixed point alone would not have caught
  the old drift, itself a steady state), and a camera that moves and
  stops reaches the image of one still all along; both pass on ANGLE
  with the CPU check off too.
- Metal: every TAA scene byte-identical (`taa`, `taa-sharp`, `showcase`,
  `overlay`, `sprites-untonemapped` at RMSE 0, max 0); the 39 looks-script
  frames byte-identical to main's. No reference re-recorded. og-render
  README (the TAA note, a fourth GL constraint, the table row) and
  docs/ci.md's `scenes::Applies` sentence updated.
- Left out: the native GL path (row 70); restructuring the reprojection
  to avoid world coordinates (it would move `taa-sharp`'s metal bytes,
  and with the real depth the rounding left is 5.5e-4 px, under the
  test); the browser's drift reproduced end to end natively (the harness
  reproduces it at the 2,800 m offset, symmetric about the centre rather
  than the 0-to-2 px left-to-right measured in Chrome).

Functional lane on the merge: 5 of 5 in 123 s.

## Wave 8, api-surface round two: the reverse surface check, string fields in IDL records (81bb31f)

- 3c036fa4, row 64: `packages/engine/tests/api-surface.test.ts` fails on
  any member a game can reach that the IDL does not declare (the package
  exports, `og` and every `og.<module>`, World, Query, ArchetypeView,
  System, Game, Rng and RngStream, every engine resource object, the
  EngineProbe), read from the IDL catalog in `schema/api.schema.json`
  (members that exist only in TypeScript, `host false`, count as
  declared); the allow-list holds one entry, `World.id` (the engine id
  the runtime's free functions pass to the host, taken by no call), and
  fails if an entry no longer exists. It found 17 column factories
  (`Texture`, `ToneMapping`, `Tween`, `TerrainData`, ...) and the
  constants `MAX_POINTERS`, `MAX_GAMEPADS` and `SOUND_CULLED` declared
  and documented but never exported (importing one gave undefined); they
  are exported, and a test checks every column factory and constant,
  with a component of one field of every column kind registered and read
  back. `tests/web-shell-crash.test.ts` runs the real Worker and shell
  under bun with the native host bound and a fake port, canvas and
  document: a throw in `GameDesc.start` reaches the shell's `onCrash`
  once, with the message and stack, primed and with autoplay, and later
  probe calls are refused (the card's DOM in `shell/page.ts` is not
  exercised). The runtime `Engine` class is not checked (the host's
  lifecycle object, never held by game code).
- a2e48f3f, row 65 (a) and (b): an IDL component or resource field can be
  `field name str.48` (shown `str<48>`; capacity 1 to 4096 bytes, a
  default must fit, refused on other field kinds), generated into the
  `.d.ts` (a `string` with a `@capacity` tag), `bindings.json`, the
  schema, `FieldDecl.capacity` in the record table and the wiki; the
  runtime and `og cook` store it as a fixed-size string. No engine record
  had a natural string field (adding one would change world layouts), so
  the IDL fixture `valid.ogt` uses it and four broken fixtures cover the
  refusals. `crates/og-runtime/tests/scene_string.rs` cooks a scene with
  a 16-byte string component, loads it through `og.load.scene`,
  instantiates it and reads every name back (CJK, quotes, empty, full).
- ae9ae608, row 65 (c): `strings.test.ts` gained the web-host half
  (skipped on a stale wasm as hosts.test.ts's are); run by the stream
  against the measure lane's module at cf5f2b9: every answer and the
  world hash equal on both hosts.
- 93625d6e, 81bb31f0, Lantern Vale #24: the "New best" badge takes its
  colours from the skin; `pack_contrast.rs` now measures a badge in
  every tone; the accent badge read pixel 3.90, cartoon 3.78, fantasy
  and Ember 4.12, scifi 4.37 on a panel, so pixel's and scifi's badge
  tint is more transparent and cartoon's, fantasy's and Ember's accent
  ink darker (`#6e470d`): every pack 4.72:1 or better, `make-skins.py`
  reproducing all six byte for byte; Lantern Vale's own skin copy took
  the same ink at 402442cc. The other badge tones stay under 4.5:1: row
  69.
- Frames: none moved (ui-kit 32, Ember Vale and Lantern Vale suites pass
  unchanged after the merge).

Functional lane on the merge: 5 of 5 in 142 s.

## Wave 7, render-style round two: TAA on the GL backend (cf5f2b9)

The first web-reference run of the wave (above) failed WebGL2's looks
station on 30 of 37 frames, and a browser bisect showed the failure
predated wave 7 (same 30 frames at 721886b7, before render parity). Three
diagnostics in the browser, each the looks station captured on both
tiers and WebGL2 compared against WebGPU rather than `metal`: with FXAA
in place of TAA, 34 of 37 agree within RMSE 0.003 (the stylise chain
renders the same on both tiers); with TAA's jitter zeroed and its
history kept, 36 of 37 still differ; with TAA's jitter kept and its
history discarded every frame, 34 of 37 agree. So the single jittered
renders are identical on both tiers and the divergence is in the
resolve's history loop on the GL backend: with a still camera the
history crept sideways frame after frame (0 px at the left of the
frame, 1 px in the middle, 2 px at the right), which the looks'
posterize, ink and outlines turned into whole moved lines, while the
plain stations absorbed it (RMSE 0.005 to 0.008 on WebGL2 against 0.001
to 0.002 on WebGPU). The three that disagree in every diagnostic,
paper-craft, voxel-worlds and sculpted-clay, turn on `PostEffects.ao`,
which is GTAO on the full tier and SSAO on WebGL2 by design (og-render
README, "Ambient occlusion by tier").

- 617fb862 (93d433b6): TAA takes the pixel's centre from the window
  position rather than the interpolated uv, for the reprojection and the
  history read, and a pixel where nothing moved reads its own history
  texel unfiltered. Metal: all 37 looks frames byte-identical,
  `taa-sharp` (a sliding camera) moved by RMSE 0.00029 and was
  re-recorded. It did not change the WebGL2 outcome.
- cf5f2b9e: the CPU decides a still camera (the history's view-projection
  equal to the current one within a millionth per entry) and passes it in
  the TAA uniform; then every pixel reads its own history texel with no
  reprojection computed on the GPU. Moving cameras keep the reprojecting
  path. Metal byte-identical, `taa-sharp` included. On WebGL2 the looks
  station then passes against `metal` on 35 frames (worst neon-vector
  0.0098); paper-craft (0.0187) and voxel-worlds (0.0113) keep WebGL2
  references of their own by the AO rule (95ea28a; sculpted-clay, 0.0062,
  is judged against `metal`), and docs/ci.md's reference-frame rule says
  so.
- 17675497: the og-render README's TAA note corrected (the first commit
  had not fixed WebGL2) and the AO-by-tier paragraph.

What is not explained: why the GL backend's reprojected read of the
history differs from WebGPU's for a view-projection that is bit-identical
on the CPU (the shader's own motion test did not pass there). The fix
removes the computation for the still case rather than finding the
arithmetic difference; a moving camera on WebGL2 still takes the
reprojecting path, and whether it drifts there is not measured (no looks
frame moves its camera). Owed row 68.

Functional lane on each merge: 617fb862 5 of 5 in 229 s, cf5f2b9e 5 of
5 in 125 s. web-reference at cf5f2b9: 379 s, every frame (the gate
section above).

## Wave 7, render-style (2026-10-21)

Branch `render-style` from main at 6c804499, the last of the wave: it
merged main (the other four streams) as e36fbf1f and main fast-forwarded
to it; the coordinator's 767ecf84 then re-recorded the frames that moved
by design and took Lantern Vale's three workarounds out. Functional lane 5
of 5 in 147 s. No evidence directory: the commits, each with a scripted
`og run --commands` repro of its finding read before and after,
og-render's README (the material style, hull outlines, depth bands, its
scene list's round five) and og-taste are the record.

- **A material's outline draws silhouettes and contact lines**
  (7e3ae322, FINDINGS #15): the prepass writes each fragment's object id
  (1 to 255 from the instance's transform, the skinned instance's and not
  its joints'; 0 for terrain and scattered copies) into the outline target
  beside the colour and width, and the outline pass lines a crease of a
  material's own outline only where the ids differ: the object's
  silhouette, a part in front of another, its creases against other
  objects, never the edges between its own faces (a `.vox` model's
  unwelded quads had become a net of lines). `PostEffects.outline` still
  lines every crease. Chosen over smoothed outline normals, which need a
  vertex stream WebGL2's skinned prepass has no room for; neighbouring
  objects share an id one time in 255. og-tests `stylise-outline-voxels`.
- **`StandardMaterialDesc.styled`, and toon by physical materials**
  (51e765e9, #16): `styled false` keeps a material exactly as declared
  under any look (`og.StandardMaterial` version 6; mode bit 8 turns the
  frame's shadow tint and outline distance off for it). A look's toon
  passes by a material with a physical lobe (clear coat, sheen,
  transmission, iridescence, anisotropy, an ior other than 1.5, or their
  maps): it keeps its physical shading and takes the rest of the style;
  its own `toon` still makes it toon. og-tests `stylise-toon-physical`.
- **`Sprite.toneMapped`** (46c4d3ea, #17): `SpriteDesc.toneMapped` and
  `og.Sprite.toneMapped` false keep an alpha-blended sprite's colours as
  given (unexposed, uncurved, outside bloom and fog, still placed, sorted
  and depth tested in the world); flagged in the HDR alpha in the main
  pass so the depth test works on every tier, the tonemap mixing by it
  only on frames that have such a sprite (no other reference moved).
  og-tests `sprites-untonemapped`.
- **Depth bands** (b9c3ed87, #19): `depthBandsFlatten` 0 is a tint (the
  lit colour times the band's colour at unit luminance), and the bands
  step in OkLCh, so a plum-to-gold ramp passes rose and amber instead of
  grey-brown; og-taste §12 says when bands work (side, three-quarter and
  high views; not top-down). og-tests `stylise-bands-tint`.
- **The dusk hours** (8ce76910, #18): og-taste §8 names 16.5 to 17.25 h
  for a warm dusk with sun shadows (the sun 20° to 10° up), not 17 to
  18 h; DayNight's path was right. og-render's
  `the_dusk_hours_keep_sun_shadows` holds it.
- **The merge** (767ecf84): Lantern Vale's dusk look takes the anime cel
  preset's toon and outlines as they are, its bubbles are `toneMapped
  false` instead of dimmed, its dawn look names `MaterialStyle { toon
  false }`, and `styleModels` keeps only the lanterns' clear coat.
- **Frames** (`metal`, RMSE against the references they replace):
  stylised 0.01571, skinned-hooks 0.00786 (the tubes' cap rims),
  stylise-pixel 0.02435, stylise-palette 0.01196, stylise-halftone
  0.02378 and stylise-halftone-surface 0.01954 (the outline, 7e3ae322);
  stylise-bands 0.06253 in total (0.00593 of it the outline, then the
  tint, b9c3ed87); stylise-material 0.10975 in total (0.05549 the
  outline, then its physical objects keeping their lobes, 51e765e9). New:
  stylise-outline-voxels, stylise-toon-physical, sprites-untonemapped and
  stylise-bands-tint. At 767ecf84: nine fx-showcase looks frames
  (anime-cel-3d, bold-comic-illustration, decorative-storybook,
  illuminated-manuscript, inked-comic-3d, paper-craft and
  rubber-hose-animation at 0.011 to 0.030 by their outlines;
  layered-silhouette 0.134 and luminous-layered-painting 0.308, where
  flatten 0 now shows the shading through the near band) and Lantern
  Vale's seven (0.003 to 0.031).

Left out: the moved frames' `vulkan-cpu` copies and the browser tiers
(row 67).

## Wave 7, ui-text (2026-10-21)

Branch `ui-text` from main at 6c804499, merged main as 3a579929 and
167579c6 and fast-forwarded. Functional lane 5 of 5 in 184 s. No evidence
directory: the commits, og-ui's README and the og-ui, og-taste and
og-build skills are the record.

- **The one-second frame** (8b87652b, #32): not the switch to Chinese.
  The three playtests that switch are the three that take screenshots,
  and the step time of a capturing tick counted the capture, its
  1280 × 720 PNG deflated by miniz_oxide at opt-level 0 (0.95 s). The
  capture's wall time now goes to `RenderState.capture_ms`, outside the
  step time, and miniz_oxide, adler2 and simd-adler32 are optimised in the
  dev profile (a screenshot's deflate 940 to 160 ms). og polish's frame
  max (metal, dev): locales 948.7 to 20.0 ms, frames 998.1 to 16.4 ms,
  settings 768.7 to 16.3 ms.
- **og polish** (213ed08f, #25): an element whose bounds differ on the
  ticks before and after a shot (a toast sliding out) is not measured for
  contrast; og-build §5 says so, and why thin CJK strokes measure lower.
- **`og.ui.configure` keeps what it is not given** (34d86c21, #11): the
  desc is merged over the last one (null clears a field); a UI left with
  no font logs a warning naming `og.load.font`; `applyLook` passes only
  the skin; ui-kit's `applyTheme` passes every field.
- **Skin ids relative to the skin file** (9d00fb21, 57beedc6, #6): an id
  is read beside the skin first (`atlas.png`, `./`, `../`), then from
  `assets/`; the cook and `og.load.skin` resolve alike; the six ui-kit
  packs name their own files relatively (`make-skins.py` still reproduces
  them byte for byte); absolute ids keep working.
- **Placeholders at 4.5:1** (ddde1e74, #29): og-ui's `pack_contrast.rs`
  measures a text field per pack (placeholder and text, at rest and
  focused, on the screen and in a panel); the focus ring is cut under
  every control (it showed through a field frame's 94 % centre: scifi
  3.52:1 and neon 3.00:1 focused); `fgSubtle` moved for pixel, cartoon,
  fantasy and ember. Every pack at 4.70:1 or better.
- **A field named by its label** (dbcdf1ad, #23): a textbox takes the
  label before it in its row as its name, its placeholder as the new
  `A11yNode.description` (the web mirror's `aria-placeholder`).
- **og.ui's own words localisable** (7d982be7, #24):
  `ResultsDesc.bestLabel` (`"Best {best}"`), `newBestLabel` and
  `shareLabel`; Lantern Vale passes the first two from its tables.
- **`type:` keeps its text** (8a288955, #21, #22): `type:`, `compose:` and
  `commit:` take their last argument to the `;` exactly; the textField
  doc and og-ui §7 say that bare keys type only until the world's first
  `og.TextInput`; og-ui §7 holds the one rule for mixed-script labels and
  §3 points to it.
- **Frames**: none re-recorded; og-ui's suite passes unchanged.

Left out: the packs' CJK fallback font stays outside the pack, by
decision (row 66).

## Wave 7, reflect-string (2026-10-21)

Branch `reflect-string` from main at 6c804499 (294888a2), merged main as
6996a8e1 and fast-forwarded. Functional lane 5 of 5 in 227 s. No evidence
directory: the commit, og-core's and og-ecs's READMEs and the og-core and
og-ui skills are the record.

- **`str(N)`** (294888a2, FINDINGS #7): a fixed-capacity UTF-8 field type,
  1 to 4096 bytes, zero-padded (`FieldType::Str(cap)`, `strN` in JSON, in
  snapshots and in the layout hash, where `str64` and `str256` keep their
  names and hashes; `Str<N>` in Rust); `pack_str`/`unpack_str` refuse a
  text over the capacity with the fix named, never truncating. The world
  takes strings in components and resources; a migrating restore keeps a
  text that fits a new capacity. Read and written by the probe's `get:`
  and `set:` and its dumps, scene text (one quoted string), the codec (a
  JS string in and out), TypeScript's `str(capacity, value?, options?)`
  (a `Uint8Array` view of stride the capacity), the generated `.d.ts` and
  the wiki. Tests: og-core `reflect.rs`, og-ecs `field_types.rs`, og-cook
  `scene_string.rs`, og-probe `strings.rs`, the engine package's
  `strings.test.ts` (native host).
- ui-kit's field and area values live in its `Typed` resource; Lantern
  Vale's twelve `u32` code points become `Prefs.name: str(48)`.
- **Frames**: none.

Left out: row 65.

## Wave 7, api-runtime (2026-10-21)

Branch `api-runtime` from main at 6c804499, merged main (cook-vox) as
661305ff and fast-forwarded. Functional lane 5 of 5 in 161 s. No evidence
directory: the commits, their tests and the og-probe README are the
record.

- **The blocker** (0e3419b0, FINDINGS #9): `og.load.textureArray` and
  `EngineProbe.listen`, declared, typed and bound on both hosts, were
  missing from the runtime's objects. Both are there, and
  `packages/engine/tests/api-surface.test.ts` walks every call of
  `types/bindings.json` to the runtime object a game reaches it through.
  Lantern Vale loads `sprites/faces.ogt` with it and its flipbook-array
  sheet goes (3eeafa8a: 11 suites pass, frames unchanged on `metal`).
- **A throw in `start` fails the run** (5dfc5a7b, #10): advance, frame and
  probe calls refuse with the error and its stack, `og run`, `og test` and
  `og dev` exit non-zero, the web shell shows the crash.
- **project.ogt's `tick`, `seed`, `scenes` and `scene`** (457e659a, #8):
  `og` passes the manifest's text (`CreateOptions.project`) on run, test,
  dev, polish and the play page; they fill the `GameDesc` fields a game
  leaves unset, and one set in both places to different values refuses
  the engine, naming both; `Engine.rate` answers the rate the engine runs
  at.
- **The probe** (38472c71, #12, #13, #14, #20, #30, og-feel's #31): `set:`
  takes `true`/`false` for a bool and refuses outside a field's min/max
  naming the range (`FieldOptions` says which writes are checked); more
  unnamed `get:` steps on one tick write `get-<tick>-2.json`, ...;
  `get:og.Time` answers the whole of og.Time (rate and the time scale);
  `get` answers the light og.DayNight drives as it is drawn, a pure
  function of the tick; og-feel reads values with `get:`.
- **Knowledge** (c364d491, #1, #28): og-audio loops with
  `AudioSource.loop`; og-game §13 says `og.Velocity` is optional and where
  `world.set` refuses; the validator's name check reads `key: value` spans
  and a token's object literal.
- **Frames**: none.

Left out: row 64.

## Wave 7, cook-vox (2026-10-21)

Branch `cook-vox` from main at 6c804499, the first of the wave to merge
(fast-forward to 6232102a). Functional lane 5 of 5 in 219 s. No evidence
directory: the commits, og-cook's README (the `.vox` row, §What phase 2
owes) and the og-game and og-build skills are the record.

- **A `.vox` without a scene graph is centred** (42e622cb, FINDINGS #27):
  each model sits at the origin about `floor(size / 2)`, where a graph of
  one identity transform puts it (a unit test compares the glTF bytes for
  even, odd and one-voxel sizes and both origins; `tests/vox.rs` cooks
  the same voxels with and without a graph to the same bounds). Lantern
  Vale's generated files drop their workaround graph and cook to
  byte-identical blobs (c962b1e6: 0 of 118 outputs re-cooked, the seven
  frames at RMSE 0.00000 on `metal`).
- **Animated `.vox`** (42e622cb, #5): MagicaVoxel's frames (an `nSHP` of
  several models keyed by `_f`, or several graph-less models, 0.98's
  `PACK`) cook to a skinned glTF of mesh swaps: each frame's model skinned
  whole to a joint of its own and, per clip, a STEP scale channel per
  joint at `voxFps` (default 10); `voxClip <name> <first> <last>` names
  ranges (default one clip `frames`); without a skeleton the model draws
  frame 0. Each frame sits about its own pivot (6232102a, read in a
  scripted run of a three-frame file: red, green, blue, red again).
- **A model's clips** (6e5a369b, #2, #3, #4): `GltfInfo.animations` lists
  the clip names of the model's `#anim` blob (no clip decoded);
  `og.anim.clip` gains `(info: Animation) => ClipAsset` (the IDL's first
  overloads, of `host false` fns only); og-runtime `tests/model_clips.rs`
  plays an animated `.vox` on `og.AnimationPlayer`; the IDL's
  `og.load.gltf` doc, og-game §5 and §9 and og-build §7 name `voxScale`,
  `voxOrigin`, `voxFps` and `voxClip`.
- **Frames**: none moved.

Left out: row 63.

## Lantern Vale, the dogfood game (2026-10-21)

Branch `dogfood-game` from main at 721886b7, merged main as 42825a61 and
3e6cf675 (after physics lanes round three and 2133be1c) and
fast-forwarded; `FINDINGS.md` landed on main at 7f88c046, corrected at
6c804499 (#26 is a nit: the dev profile does optimise its heavy crates
per package).

- **The game** (ab6d157a, 64c78455, 2f924b3e): `examples/lantern-vale`, a
  five-minute 3D top-down adventure written from the knowledge package as
  a game author would: generated assets (`tools/make-assets.ts`: `.vox`
  models, a skinned voxel slime glTF, terrain maps, textures, a dusk HDR,
  a 4-minute music track left uncommitted; `tools/subset-font.py`), a look
  and `og.render.applyLook`, physical materials, `og.Environment`, a
  streaming group, the character controller, `og.physics.overlap`,
  chunked music and og.ui screens in English and Chinese. Eleven
  playtests, seven reference frames on `metal`; og polish taste 99, feel
  90 (its README).
- **`FINDINGS.md`**: 32 findings, each with its repro and workaround: 1
  blocker (#9, `og.load.textureArray` not a function), 15 annoyances, 16
  nits after the correction; top five #9, #16, #15, #11, #27.
- **Frames**: title, start, lantern, slime, far, settings-zh and results
  on `metal` only.

Left out: wave 7 took every finding (`examples/lantern-vale/FINDINGS.md`
§After wave 7 says which stream and commit, and what stays open); the
frames on the other tiers (row 67).

## Phase 4, physics lanes round three (2026-10-21)

Branch `physics-3` from main at dca74cbb, merged main (render parity) as
1f0ecdc3 and fast-forwarded. Evidence: `tests/evidence/P4/physics-3/`
(machine, load and every command in its README).

- **og-lanes** (c17015d2): `calls_here` (whether a stretch of this
  thread's own work split) and `sort` (a sample sort that leaves
  `slice::sort_unstable`'s order).
- **The step around rapier** (40133e7d, e3a03e69): per-phase stats; the
  sync planned by ranges of bodies and done in body order; the readout
  reads the contact graph by 2,048 edges a task (on the pool only: below
  it splitting cost more than it saved), sorts keys and indices with
  `og_lanes::sort` and finds the bodies that left by one walk; the gather
  decodes rows by 1,024. A pen of 2,000: 1.51 to 1.24 ms around rapier
  lanes off, 1.64 to 1.20 on four lanes.
- **`og.physics.stats`** (4c00b124): `PhysicsPhase` and `PhysicsStats` in
  the IDL on both hosts (the web host times a step on the Worker it runs
  on). On main, b00639cb gives the engine package's test declarations
  `toBeGreaterThanOrEqual` and 2133be1c makes the three core tests that
  load the web module skip on a stale one, as the cross-host tests do,
  since the IDL gained `og_physics_stats`.
- **Grains that did not pay, kept off with their numbers**: ragdolls
  planned alone (b692e2b4; 16 to 256 ragdolls cost 0.04 to 0.13 ms more
  split, `Grains::ragdolls` off), the sync's and readout's per-body plans
  (never paid to 4,000 bodies on any thread count, `Grains::bodies` off)
  and a 3D fluid's densities gathered on the lanes (65f3e2d0, bit for bit
  the scatter's sums; b748cd40 put the scatter back: 8,000 particles on
  four lanes 8.37 ms scattered, 8.45 gathered). Relaxation, viscosity and
  mixing stay serial: Gauss-Seidel chains as long as the fluid.
- **Five and six threads** (b748cd40) have soft grains of their own: five
  split ropes from eight (800 point passes), six from 16 (1,600); cloths
  split from four on both (2,000).
- Docs (acd0fc4f): og-physics's and og-lanes's READMEs,
  `docs/web-export.md` §Lanes.
- **Determinism**: physics 18126409214070009362 natively lanes off, on
  four and eight threads, and with every pass split on four and eight
  (23,066 parallel calls each), unchanged.
- **Frames**: none.

Left out: the lanes wasm's step and `og.physics.stats` through the web
host in Chrome (row 52).

## Phase 4, render parity round (2026-10-21)

Branch `render-parity` from main at 463bc737, merged main (with the cook
fix d243e461) as 6f8eee08, its evidence at 26649d4c, fast-forwarded.
Evidence: `tests/evidence/P4/parity/`; row 50 has the summary.

- **Kept sorts and spheres** (038cc380, 5a96a149): the sort into (mesh,
  material) runs is redone only when the submission's keys or a drawn
  mesh's bounds change, an instance's sphere and GPU transform only when
  its transform changes; the per-frame pass made cheaper at opt-level 0.
- **52-byte instances** (8f4468f2, 6f8eee08): the xyz of an affine
  matrix's four columns (48 bytes) and, after the merge with main's array
  maps, the texture-array layer at location 8; points an instance moves
  are projected through `instance.wgsl`'s `mul_point` so Metal's sums stay
  the 4x4's.
- **The 16-byte compact vertex** (8d619852), opt-in per mesh
  (`VertexEncoding::Compact`, quantised): with every mesh compact all 68
  scenes draw, the largest difference showcase's RMSE 0.011.
- **wgpu's indirect-call validation off** (07902610; GPU culling's draws
  are in range by construction) and **objc2 without debug assertions in
  the dev profile** (cbcc062c; gltf-reference's dev CPU 3.83 to 1.00 ms).
- **Measured** (interleaved rounds, M4 Pro): the dev CPU of spheres-20k
  4.27 to 1.76 ms, spheres-20k-moving 4.27 to 2.96, gltf-reference 3.89
  to 0.95; release 1.60 to 1.13. The frame is GPU-bound at about 5.4 ms
  (from 5.6), bound by the field's 7.3 million triangles a pass, which
  neither encoding moves natively; the §7 reference scene unchanged.
  Parity with three.js is not reached natively (5.4 ms against three.js's
  1.65 ms in Chrome); design §7 is unchanged.
- **Perf baselines lowered** (96ac0c89): spheres-20k 6.675 to 5.245 ms,
  spheres-20k-moving 7.759 to 5.276 ms; gltf-reference's 1.385 ms stays.
- **Frames**: every Metal og-gfx frame byte-identical (68); after the
  merge 71 of 73, `stylise-object` now equal to its reference and
  `materials-array` 13 pixels off by 1/255 (RMSE 0.00001).

Left out: the compact encoding is on nowhere by default (cooked models
could set it, moving the frames of untextured cooked parts within
tolerance); the browser comparison with three.js (row 51).

## Fix on main: og-cook freshness (2026-10-21)

d243e461 on main after 4358c7f. The functional lane at 4358c7f failed its
bun step: cook round four changed `og.Terrain` (e8de98d) without a
`COOKER_VERSION` bump, the incremental cook kept the old terrain blob
(stamp, size and hash agreed) and the loader refused it ("holds type
f447c7a2bbf6fee6, expected c0bc83477a1bc3ed"). A kept output is now fresh
only if its header holds its kind's type hash and schema version
(`format::expected_header`); og-cook's
`a_kept_blob_whose_type_moved_is_cooked_again` patches a blob's type
hash under a consistent manifest and sees it cooked again. The lane then
passed 5 of 5 in 117 s (Main gate, 2026-10-21 (fe00f56), above).
Frames: none.

## Phase 4, cook round four (2026-10-21)

Branch `cook-4` from main at dca74cb, merged as 1cdc3da (after this
record's base, fe00f56). No evidence directory: og-cook's and og-render's
READMEs and the commits are the record. It closes what cook round three
left.

- **Materials sample texture arrays** (55c0c53): `StandardMaterial`
  (version 5) gains `mapArray`/`mapLayer` and `normalMapArray`/
  `normalMapLayer`; `og.Mesh.materialLayer` overrides both per instance,
  riding beside the instance transform, so meshes of one material on
  any layers stay one batch. One variant of the standard lit, prepass and
  shadow pipelines draws them; physical and shader materials keep their 2D
  path. og-tests `materials-array` (its `metal` reference f1d9233;
  `vulkan-cpu` is row 54).
- **Terrain layers from texture arrays** (e8de98d): `type terrain`'s
  `layers` and `layerNormals` arrays with `albedoLayer`/`normalLayer` per
  layer, two sampled textures instead of eight, through a variant of the
  terrain layout and pipelines (`og.Terrain` meta v3, layer rows v2); a
  terrain of separate textures draws exactly as before.
- **A KTX2 cube's own prefiltered chain** (4262118): `prefiltered true`
  takes a GGX-prefiltered KTX2 mip chain as it is (`bake::from_prefiltered`);
  the irradiance is still projected from level 0.
- **The `.vox` pivot** (88d2150): a committed fixture turned, mirrored and
  with its axes cycled checks the importer against ogt_vox's documented
  pivot; the importer already agreed.
- Docs (566250d, 077d64b): og-render's §Array maps, og-cook's owed list
  down to animation.

Left out: compressed clips resident as f32 (row 56); the materials-array
frame on lavapipe (row 54).

## Phase 4, stylisation round four (2026-10-21)

Branch `stylise-4` from main at dca74cb, merged as 721886b (after this
record's base, fe00f56). It closes what round three left but the octave
bands.

- **Object space** (2dc63ee): `StyliseSpace::Object` (capability
  `objectSpace`): the canvas, hatching and halftone triplanar in each mesh
  instance's own space, from an object pass after the prepass writing the
  object-space position and normal (`stylise_object.rs`); terrain,
  scattered copies and vertex-offset materials keep the world pattern.
  og-tests `tests/stylise_space.rs`: the scene moved with its camera
  differs by 0 pixels in object space, 1.07 % in world space.
- **Hatching from depth** (9f4751f): `Stylise.hatchingDepth`: silhouettes
  and creases darken the tone the strokes follow.
- **Drop shadows bent over hollows** (947142f): `ShadowGround::bend` (a
  5 × 3 grid carried down the sun), a bent shadow drawn as 4 × 2 cells;
  the plane path unchanged (sprites-20k byte-identical).
- **Surface space in a preset** (665b285): stylise-halftone-surface, and
  inked-comic-3d hatched on the surfaces with `hatchingDepth`; the looks
  station's HUD names the preset shown (b76ca77). Docs: 47d4676.
- **Frames** (721886b): stylise-object and stylise-halftone-surface new on
  `metal`; sprites-shadow-terrain moved (a hollow and a figure whose shadow
  dips into it), every looks frame moved (the HUD label) and
  looks-inked-comic-3d by its preset. Their vulkan-cpu copies are the wave
  end's (row 54).

Left out (og-render README): the octave fade's bands on a receding floor
(row 55); object space leaves terrain, scattered copies and vertex-offset
materials in world space and cuts no alpha holes; a bent shadow bridges
features narrower than a cell of its grid.

## Phase 4, UI round four (2026-10-21)

On main after cook round three (dca74cb): af5f296..fe00f56 and 4390233,
no merge commit. No evidence directory: the commits and og-ui's README
(§What phase 2 owes) are the record.

- **Command told apart from Control** (af5f296): `InputFrame.meta` beside
  `ctrl`; `Viewport.macKeys` picks the convention (on a Mac Cmd+A/C/X/Z,
  Cmd+←/→ to the line's ends, Option by words, Ctrl+A/E; elsewhere Ctrl
  takes the shortcuts and words, Home/End the line's ends). The web shell
  sets it on Apple platforms and releases the keys a Mac's browser sends no
  keyup for under Command; native hosts leave it off, so scripts edit alike
  on every OS. og-input holds a protocol key event's modifiers for its
  tick; the probe's `key_down`/`key_up` take Shift+, Ctrl+, Alt+ and Meta+
  chords.
- **Words by UAX #29** (da15c95): og-i18n's `word_segments` and
  `graphemes`; word moves, word deletes, double clicks and the undo
  history's typing steps read them (Han and Hiragana a word per character,
  a Katakana run one word, Thai, Lao, Khmer and Myanmar a grapheme cluster).
- **A font per grapheme cluster** (83d882e): a base and its marks stay in
  one run, joiners and variation selectors ask no face; every draw item of
  text-mixed-runs, text-bidi and the kit's locale strings is unchanged.
- **Accessibility** (187cfeb, c1894c3): a stepper's spinbutton is
  adjustable like the slider (the mirror renders both as a native range
  held a step from either end); reduced transparency also flattens scrims
  and translucent fills, and `UiTheme` gains `contrast`, `reducedMotion`
  and `reducedTransparency`.
- **The kit** (d53640c): a mixed CJK, Latin and Arabic label
  (`menu.mixed`) on the locale screens in all five locales, its playtest
  asserting no missing glyph; off by default, so no kit frame moves.
  4390233 renames a variable of `tests/web/a11y.ts` the engine package's
  typecheck caught shadowing another. Docs: fe00f56 (og-ui, og-input and
  og-probe READMEs, the og-ui skill).
- **Frames**: none moved.

Left out (og-ui README §What phase 2 owes): rows 59 and 60.

## Phase 4, cook round three (2026-10-21)

Branch `cook-3` from main at 463bc73, merged as dca74cb. No evidence
directory: og-cook's README (the formats table, §What phase 2 owes and its
record paragraph) and the commits are the record.

- **Environments from KTX2 cube maps and six faces** (b6fa5e1, df6dfcf):
  og-render's `bake::bake_cube` bakes six square faces into the same
  prefiltered levels and irradiance as the equirectangular path (a smooth
  sky given both ways within 2 % in irradiance and under 1 % mean per
  texel); `type environment` takes a KTX2 cube (level 0 of RGBA16F,
  RGB16F, RGBA32F, RGB32F, RGB9E5, B10G11R11 or BC6H, plain, Zstandard or
  zlib) or `faces` with six `.hdr`/`.exr` images. `cook::bc6h` decodes all
  14 modes and agreed with bcdec on every texel of 2,000,000 random blocks;
  the fx-showcase studio cut into a 64-square RGBA16F cube bakes within
  0.2 % mean per texel and 0.17 % in irradiance of its equirectangular map.
- **One fetch queue** (ab8b9f8): every streamed read goes through one
  queue, sound chunks nearest a playhead first (og-audio's
  `chunk_distances`), then the loading groups' packs by priority, at most
  four in flight (`MAX_FETCHES`); `dump_stream`'s `fetch` lists them;
  og-runtime `tests/stream_fetch.rs`.
- **Flipbooks in streaming groups** (5ff26cf): an `array true` sheet packs
  its array and layers with it; on unload the array is kept while a sprite
  of the sheet draws from it, and the sheet's sprites follow it back under
  its new renderer handle.
- **Cook times** (767610b): `cooked/.cook-times.ogt` per asset, kept out
  of the manifest, the determinism comparison and the export; `og cook
  --report` prints the slowest ten, `og cook --watch` times each change
  against the 300 ms reload budget, as `og dev` does.
- **.vox** (b592116): mirrored `nTRN` frames flip their winding (24 of the
  48 frames faced inwards without it); `model { voxScale voxOrigin }`.
- **og-cook README** (faa2cb5): every blob in the formats table, the owed
  list cut to what remains.
- **Frames**: none moved.

Left out: texture arrays in materials and terrain layers, a KTX2 cube's
own prefiltered mip chain taken as it is, the `.vox` pivot of odd-sized
models turned or mirrored. *Resolved in cook round four (55c0c53, e8de98d,
4262118, 88d2150).*

## Phase 4, stylisation round three (2026-10-21)

Branch `stylise-3` from main at 463bc73, merged as 96dcdfe; its references
recorded on main (da96248, 7f8b010).

- **Surface space** (d79ec8c): `StyliseSpace` (screen or surface) for the
  canvas, hatching and halftone (`paintCanvasSpace`, `hatchingSpace`,
  `halftoneSpace`; capability `surfaceSpace`): each pixel's point rebuilt
  from the prepass depth against a 1024 m grid origin under the camera, its
  world normal weighting three planar projections; the canvas sampled at
  `paintCanvasWorldScale` texels a metre, hatching and halftone laid in
  metres, the finer of two power-of-two octaves fading in and out.
  `inkDepthThreshold` (0.04) and `inkNormalThreshold` (0.4) are fields.
  The screen paths are unchanged: every existing og-tests frame is
  byte-identical on Metal.
- **Drop shadows on the ground** (c9afcf3): `ShadowGround { height,
  normal }` per shadow, `ShadowGround::fit` to uneven ground,
  `SpriteDesc.dropShadowHeight`, og-runtime fitting each shadow to a drawn
  og.Terrain under its anchor; a shadow's alpha is multiplied by the sun's
  cascaded light, so it fades inside a wall's shadow. sprites-20k moves by
  at most 2/255 on 261 pixels on Metal (the vertex shader's codegen) and is
  re-recorded (da96248).
- **The looks station in the browser** (70b1187): the playtest's script in
  `templates/fx-showcase/tests/looks-script.ts`, shared with
  `reference.ts --looks`; 37 shots per tier against `metal` in
  `scripts/check.sh web-reference`.
- **References**: stylise-surface and sprites-shadow-terrain on `metal`
  (da96248); the looks frames on lavapipe after round two (7f8b010: five
  re-recorded as the tier's own, two judged against `metal`).
- Docs (d1785af): og-render README's round three, og-taste on surface
  space.

Left out: the surface patterns are triplanar in world space, not in each
object's own space; a drop shadow lies on one plane, so it floats a little
over a hollow narrower than itself; no preset uses surface space. *All
three resolved in round four (2dc63ee, 947142f, 665b285).* The octave
fade's faint bands on a long receding floor (row 55); the browser looks run itself (row 53); the two new frames
on lavapipe (row 54).

## Phase 4, UI fields round (2026-10-21)

Branch `ui-fields-2` from main at 463bc73, merged as 82d3181; its
references recorded on main (a2c5e44).

- **A font per run** (54c8767, c55e8bb): a label the role's font draws
  only in part is laid out per run, each character from the first of the
  role's, the locale's, the game's fallbacks and the skin's fonts that has
  it, each run measured, kerned, shaped and drawn with its own font; a text
  no font draws whole takes the mixed font, so only the characters no font
  has are missing. Only labels that mix faces move: text-bidi's mixed
  line, re-recorded (a2c5e44).
- **Fields** (a6e50f0, c55e8bb): Alt or Ctrl with the arrows by words,
  Ctrl+Home/End, a double click selects the word and a triple click the
  line, a drag from them extends by words or lines, undo and redo (100
  steps, 8 controls, typing grouped by words), drag autoscroll past a text
  control's edge; `save_state` version 8.
- **Accessibility** (7beee65): `prefers-reduced-transparency`
  (`Viewport.reducedTransparency`, `UiDesc.reducedTransparency`) draws a
  skin's glass as opaque surfaces; the mirror puts a childless slider in
  the page as a native range, so a screen reader's own adjust gesture
  becomes increment or decrement.
- **References** (9a0f727, a2c5e44): text-fields-2 (a reference scene the
  browser tiers draw too) and text-mixed-runs (the kit's cooked faces) on
  `metal`; og-ui holds 85 reference frames.
- Docs (be66f81): og-ui's README and skill, the IDL's textField and
  textArea docs.

Left out: the three frames on lavapipe (row 54); og-ui's text remainder
(row 59; Command, UAX #29 words and grapheme runs came in UI round four).

## Phase 4, knowledge round three (2026-10-21)

On main from 463bc73 (8e2794d..efb3bdc, before the UI fields merge), no
merge commit and no evidence directory.

- **The validator** (8e2794d) checks every `og.*` name and `Type.field` a
  skill writes against the generated API; og-audio's `soundInfo` answers a
  `Result` and the skills name its field, not a `.value` chain (efb3bdc).
- **The skills teach phase 4** (0cd2021, e19d456): og-game's looks (§15),
  light, exposure, environment maps, shadows and antialiasing (§16),
  sprite drop shadows, texture arrays, stepped and compressed animation,
  physical materials, `.vox` models, streaming without an unload call, the
  physics queries and the lanes; og-taste §1's premise table and §12's
  style step, its table held to the catalogue row by row; og-ui's text and
  accessibility; og-i18n, og-build §7, og-audio's chunked music, og-feel's
  stepFps, og-core's lanes. A test names each phase 4 API's skill.
- **Every game gets a look** (3517983, c2c1370): `@origingame/create-game
  --look <slug>` names a catalogue preset and `--premise` picks the style
  whose premise fits best (asked on a terminal otherwise); the game's
  `assets/looks/<id>.ogt` keeps the template's blocks under it and
  `GameDesc.look` points at it. templates/game extends faceted-low-poly
  (no reference frames); the idle archetype's look carries what its start
  set (the sparkorb skin, `toneMapped false`, `environmentIntensity 0`), so
  its frames stay. The package carries the catalogue.
- **Platform ask 12** (02cfd57): the wizard's style step
  (docs/platform-asks.md); the archetypes' and the knowledge package's
  READMEs name the look (e45c448).

Left out: the platform's side of ask 12 (row 62).

## Phase 4, gate and lane policy (2026-10-21)

On main, by the coordinating session; AGENTS.md §Build and gate discipline
and docs/ci.md hold the rules.

- **One place runs the gate** (378a6aa): parallel agents write code, check
  their crate and run its unit tests one at a time; the coordinator alone
  runs the gate, the review and the reference captures, serially (three
  worktree gates and a lavapipe container took the load average past 100
  and every one of them failed on timeouts).
- **A shorter full lane** (51d561f): the wasm variants share one wasm32
  target directory (wgpu, naga and og-render build once), the size gate
  compresses every module at once, determinism is path-aware,
  packages/engine's reload, reload-asset and web-export tests run under
  `--all`, in CI and in a plain `bun run test` only (`OG_GATE_ALL`),
  `OG_GATE_JOBS` defaults to 2 and the gate renices itself by 10;
  `shellcheck -x` passes again (CI had failed on SC1091 since 2026-10-19).
- **The `wasm-gate` profile** (0b066c6, 5e9cbcf): the path-aware lane
  builds the wasm with 16 codegen units (cold: wasm-build 293 to 205 s,
  wasm-opt 113 to 85 s, modules 5.3 to 5.9 % larger), `--all` and CI the
  shipping `wasm` profile (`OG_WASM_PROFILE` overrides); the engine
  package's anim, lanes and web-export tests cook every run (the cooker's
  r3 and r4 had failed them on stale `cooked/`); `scripts/sweep.sh` prunes
  stale incremental sessions.
- **The functional lane** (be179fc, c736dea, 222ae43): `scripts/check.sh
  functional` is what a merge passes; the wasm build, its size,
  determinism and the browser tiers are a wave's end; the cross-host hash
  tests skip on a web module older than a crate change (`OG_WASM_STALE`).
- **The lane pool** (964cc29, ad63c34): `scripts/lane.sh take lane-N
  <stream>` puts a stream on a persistent worktree whose `target/` stays
  warm across streams, `free` returns it (an idle lane sits detached at
  main), and `origin-engine-measure` carries the wave-end runs pinned at a
  commit.
- **Tests under load** (74ab809, 8b93b20): the terrain edit test takes 30 s
  (1 s alone, 5.1 s beside the gate's native tests); og-runtime's
  stream_release fixture is per test process (nextest ran its two tests in
  one directory at once).

Left out: a wave-end `--all` and web-reference on main with their summary
filed (row 53; `scripts/wave-end.sh` is in flight on `ci-green`).

## Phase 4, physics lanes round two (2026-10-21)

Branch `physics-lanes-2` from main at c1c5e5a, merged as 463bc73.
Evidence: `tests/evidence/P4/physics-lanes-2/`; the lanes round's note
(Phase 3, lanes round, "Round two") has the summary.

- **The pool threshold per thread count** (439e4e7, b6ff249):
  `og_lanes::install_from` from `og_physics::pool_bodies`: 500 bodies on
  two to four lanes, 1,000 on five or six, 1,500 from seven, where every
  step forced on the pool beat lanes off (four lanes -6 % at 512 bodies,
  -23 % at 1,000, -34 % at 2,000; eight from about 1,500, slower below).
- **Soft bodies on the lanes** (b6ff249, 0dfb694): ropes, cloths and fluid
  sets step side by side, a fluid's neighbour search and collisions split
  by ranges of particles (`og_physics::Grains`: ropes from 1,600 point
  passes on up to four lanes and 4,800 above, cloths 2,200 and 4,000,
  fluid sets 1,000 particle substeps, 2D particles 256 a task, 3D 512);
  ragdolls are flat in every mode and stay serial.
- **Only the relay reaches the pool** (d8c4096): queries outside the step
  run through `og_lanes::serial`; the lanes wasm's engine thread belongs to
  its one-thread pool once the relay runs (og-physics `tests/lanes.rs`
  injects a panic in a query).
- `bench/lanes.ts --pile` (bad2b4e): the pile of 1,000 and 2,000 boxes in
  Chrome's three modes and natively.

Measured (release, ms per tick, the committed thresholds and grains, the
least disturbed round, M4 Pro at load 7.8 to 28.8; every scene ends in one
world hash in every mode):

| scene | off | 4 lanes | 8 lanes |
|---|---|---|---|
| the pile (500 bodies) | 1.445 | 1.069 | serial |
| a pen of 1,000 | 2.009 | 1.701 | serial |
| a pen of 2,000 | 5.145 | 3.706 | 5.595 |
| 128 ropes | 0.933 | 0.284 | 0.243 |
| 32 cloths | 1.256 | 0.387 | 0.317 |
| 8 3D fluids of 1,000 | 6.380 | 1.863 | 1.262 |
| a 2D fluid of 8,000 | 15.190 | 6.703 | 5.781 |

Eight lanes stay serial below 1,500 bodies; at 2,000 they were within the
noise of serial at this load (the first round measured -15 % at load 7).
The determinism tool's physics hash stays 18126409214070009362 with lanes
off, on four, and with every pass split on four and eight (11,327
parallel calls).

Left out: the lanes wasm's step timed and hashed in Chrome (`bun
bench/lanes.ts --chrome --pile`, row 52).

## Phase 4, sprite arrays round two (2026-10-21)

Branch `sprites-arrays-2` from main at c1c5e5a, merged as 2d387ac; its
reference recorded on main (11a0607). Round one is the resources round's
array sprites (below).

- **og-render** (8cba44b): array sprites glow (array glow runs after the
  plain ones, so no existing run moves), take the soft dot and draw in the
  overlay (`Overlay::sprite_arrays`, `overlay_array.wgsl`); 2D particles
  from an array (`ParticleEmitter::array`, by age, at a rate or at random;
  `round`); colour LUTs on generation-tagged handles (`remove_lut`), each
  counted in `texture_bytes`.
- **og-cook, og-anim** (b6cb38c): an `array true` flipbook also cooks its
  frames into `<sheet>#array`, one layer per frame; an emitter's `array`
  line and `round true`; a streaming group may hold a `.cube` LUT.
- **IDL and runtime** (3c3722d, 8150acf): `og.Sprite.array` and
  `arrayLayer`, `FlipbookInfo.array`, screen sprites and emitters of
  arrays; a grouped LUT loads with its group and is kept while the frame
  grades with it; what the frame keeps after its group unloaded counts in
  the budget (`StreamBudget.kept`, `StreamStatus.kept`, dump_stream's kept
  assets with their bytes).
- **Frames**: sprites-array byte-identical; sprites-array-fx on `metal`
  (11a0607), judged against `metal` on lavapipe (RMSE 0.0053, 4d6188b).
  Docs: a64d546.

Left out: nothing named; arrays in materials and terrain came in cook
round four (55c0c53, e8de98d).

## Phase 4, stylisation round two (2026-10-21)

Branch `r-stylise-2` from main at c1c5e5a, merged as f21f1f7.

- **Built-in canvases and palettes** (827652f): paper, linen and
  watercolour canvases (256², tileable) and the lcd2, handheld4, dusk16,
  bright32 and 1bit palettes, made by the committed `generate.ts` (CC0),
  named `"@canvas/<name>"` and `"@palettes/<name>"`; the cook adds what a
  look names. watercolor-wash, paper-craft and painterly-3d lay the
  canvases; monochrome-pixel and monochrome-dither-3d quantise to handheld4
  and 1bit.
- **Blue noise** (e1a248f): a 64 × 64 void-and-cluster threshold tile
  (4,096 B; its test makes it again byte for byte) for
  `DitherPattern::BlueNoise`; interleaved gradient noise kept as
  `interleavedGradient`.
- **Ink on depth edges** (9db993c): `Stylise.inkDepth` inks the
  depth-normal prepass's silhouettes and creases beside the luminance
  edges.
- **Stepped animation** (9783414): `og.AnimationPlayer.stepFps` holds each
  pose for 1/stepFps seconds; time, markers and root motion stay
  continuous.
- **Sprite drop shadows** (9b22f11): `SpriteDesc.dropShadow` and
  `Sprite.dropShadow` cast a flattened dark copy along the sun onto y = 0,
  at most five heights long.
- **The catalogue** (735600b): pixel-3d-hybrid and sculpted-clay are full,
  so every style is `full` or `content`; og-taste §12's table matches it.
- **Frames** (f21f1f7): stylise-ink-depth and sprites-shadow new on
  `metal`; stylise-palette moved (blue noise), five looks frames moved with
  the built-in canvases and palettes, and three (illuminated-manuscript,
  heavy-ink-shadows, minimal-geometric) because a look switch now resets
  the TAA history (the resources round); every other frame unchanged. The
  vulkan-cpu copies are the wave end's (4d6188b, 7f8b010).

Left out: hatching on luminance only, the canvas in screen space, drop
shadows on y = 0 and unshadowed by the cascades, fixed inkDepth
thresholds. *The last three closed in round three (d79ec8c, c9afcf3),
hatching from depth in round four (9f4751f).*

## Phase 4, physics lanes round (2026-10-21)

Branch `web-physics-lanes` from main at d08fdc9, merged as c1c5e5a.
Evidence: `tests/evidence/P4/physics-lanes/`; the lanes round's note
(Phase 3, lanes round, "Resolved later") has the summary.

- **The step on the lanes** (1fd9c8d, 651a983): rapier's `parallel` build
  in the lanes wasm too (`target_feature = "atomics"`), reaching the Worker
  pool through a relay Worker; the whole og-physics step inside
  `og_lanes::install` with a body grain of 200 a lane thread; the contact
  readout one pass, split by 512 pairs on the pool.
- **Same world** (dba5ceb): the determinism tool's physics hash adds a
  1,000-body pen; lanes.test.ts steps a headless pile of 1,000 boxes with
  lanes off and on four threads (the same hash, physics calls on every step
  on four lanes, none off).
- Docs (88aadf4, 427cf03): og-lanes' README, og-physics' "The step on the
  lanes", design §2.2, docs/web-export.md's relay.

Measured (release, ms per tick, the middle of three rounds, M4 Pro at load
6.5 to 8.8; one world hash per scene in every mode):

| scene | off | 4 lanes | 8 lanes | 4, every step on the pool | 8, every step on the pool |
|---|---|---|---|---|---|
| the pile (500 bodies) | 1.450 | 1.450 (serial) | 1.461 (serial) | 1.026 | 1.318 |
| a pen of 16 | 0.022 | 0.022 (serial) | 0.022 (serial) | 0.045 | 0.077 |
| a pen of 512 | 0.881 | 0.886 (serial) | 0.886 (serial) | 0.821 | 1.177 |
| a pen of 1,000 | 1.914 | 1.523 (pool) | 1.958 (serial) | 1.533 | 2.122 |
| a pen of 2,000 | 4.664 | 3.194 (pool) | 3.982 (pool) | 3.181 | 3.826 |

Determinism (`determinism.md`, at dba5ceb): host debug, host release with
lanes off and on four threads (483 parallel calls), wasm32 scalar and
simd128 agree (math 6117185439544271997, world 15551159556835976588,
physics 18126409214070009362, anim 14507017669422143574).

Left out: the lanes wasm's step in Chrome (row 52); round two above moved
the grain to a threshold per thread count.

## Phase 4, lean pipelines round (2026-10-21)

Branch `r-lean` from main at d08fdc9, merged as bcd35e6. Evidence:
`tests/evidence/P4/lean/`. Closes owed row 50's remainder (cfe146a).

- **Lean lit, prepass and shadow pipelines** (bddb741): a plain material
  (opaque, untextured, no toon, rim, flash, dissolve, outline, material
  shader, physical lobe or alpha cut) draws its unskinned instances through
  `pbr_lean.wgsl`: 10 interpolated floats instead of 23 (the prepass 14),
  only the attributes read, and no discard where the fragments take no
  screen derivative (not flat shading, no decals in the frame); casters
  neither skinned, dissolving nor hooked read positions alone. At most 12
  lean lit and 6 lean prepass pipelines, made at first use.
- **`OG_GFX_SERIAL`** (f96b39e): og-gfx waits for the GPU after each frame
  and prints `gpu=`, an upper bound (the GPU clocks down between frames).

Measured (Metal, dev og-gfx, 1280x720, median frame ms of five interleaved
rounds, load 6 to 10):

| scene | main (d08fdc9) | lean, discard kept | lean (bddb741) |
|---|---|---|---|
| spheres-20k | 7.115 | 6.273 | 4.585 |
| spheres-20k-moving | 7.015 | 6.220 | 4.627 |
| gltf-reference | 3.730 | 3.692 | 3.786 |

All 64 captured Metal frames are byte-identical to main's; 45 scenes draw
lean batches. A 3x4 instance transform was measured not worth it here (the
whole upload of 20,000 moving transforms costs 0.04 ms natively). Shader
text +5,908 B raw; the size gate was not run.

Left out: the three.js comparison run again (row 51); a smaller instance
encoding is in flight on `render-parity` (52-byte instances).

## Phase 4, resources round (2026-10-21)

Branch `resources` from main at d08fdc9, merged as b571a77.

- **Removal** (01700f2): `remove_environment_map`, `remove_texture_array`
  and `remove_palette` with generation-tagged handles, as the perf round's
  meshes, textures, materials and fonts; a frame naming a removed one is
  refused (`InvalidHandle`); the image-based light and the stylisation's
  group are keyed by slot revision; `texture_bytes` counts maps and
  palettes.
- **Array sprites** (01700f2): `RenderScene::sprite_arrays` through a
  sprite pipeline variant sampling `texture_2d_array`, one run per array
  and blend whatever the layer; og-tests `sprites-array` (its `metal`
  reference 5c53b2d).
- **A look switch resets TAA** (ec71fcb): a look that adds or removes a
  stylisation pass, the pixel grid, retro 3D or the material style cuts
  the history like a camera cut (`Stylise::chain_bits`,
  `FrameStats::taa_history`); a setting alone keeps it.
- **Streaming groups let go** (0fc358a, ade313c): a group may hold texture
  arrays, environment maps, palettes and looks; its unload removes them
  from the renderer, leaves their handles pending and adds them back under
  the same handles; one the frame still uses is kept, with a warning, until
  it is not. `SpriteDesc.array` and `arrayLayer`.
- Docs (c2ea491): og-render's §Resources and removal; the ibl-post,
  cook-anim and perf notes resolved; no owed row added.

Left out: arrays beyond world sprites. *Resolved in the sprite arrays
round two (8cba44b, 8150acf) and cook round four (55c0c53, e8de98d).*

## Phase 4, stylisation round (2026-10-21)

On main, by the coordinating session (AGENTS.md §Build and gate
discipline: agents write code, one session merges and gates). Phase A
(the r-look stream, e795264) had landed the `type look` asset, the
`og.Stylise` and `og.MaterialStyle` resources as data, palettes, and the
style catalogue (`packages/knowledge/looks/catalogue.json`). Phase B,
here:

- **og-render** (`README.md` §Phase 4: stylisation): the pixel grid
  (logical size, whole-number upscale, letterboxed), the chain after the
  tonemap (paint, lines, quantize, FXAA, upscale), depth bands, retro-3D
  vertex snapping, affine texturing and depth bits, the material style
  (toon, rim, outline and flat defaults, shadow tint, outlines thinning
  with distance). `STYLISE_DRAWN` lists all of it; every earlier reference
  frame is unchanged (og-gfx `--all` on Metal: the same RMSE lines as
  before, the rest at 0).
- **og-runtime**: the pixel snap of the 2D camera and world sprites
  (`fx::snap_to_grid`).
- **og-cook**: MagicaVoxel `.vox` files cook as models (greedy-meshed,
  flat-shaded, vertex colours; `cook/vox.rs`), and the 37 presets of
  `templates/looks/` are the built-in looks `extends "<slug>"` resolves.
- **fx-showcase**: the `looks` station and `look:<slug>`, with
  `tests/looks.og.ts` taking one frame per preset (37) and proving
  `look:none` restores the plain diorama.
- **Knowledge**: the catalogue's entries are `presetReady`, 21 `full`, 7
  `partial` (what a game must supply: a canvas or palette asset, sprite
  shadows, stepped animation), 9 `content`; og-taste §1 names a start
  look per pack.

Tiers: the eight stylise scenes match `metal` on WebGPU and WebGL2 at
RMSE ≤ 0.00016 (web-reference's og-tests run, no browser reference of
their own). On `vulkan-cpu` (the og-lavapipe-bun image, og-gfx and the
addon cross-built with zigbuild) the eight scenes and the 37 looks frames
were recorded as references of their own: most sit within RMSE 0.01 of
`metal`, and the quantised looks differ by design, since a dither or ink
threshold flips whole pixels where llvmpipe's shading differs by a step
(`stylise-pixel` RMSE 0.022, 3.8 % outliers; `looks-detailed-doodle`
0.045, 15 %; `looks-8bit-pixel`, `looks-topdown-tile-pixel` 0.015; the
median looks frame 0.0057). web-reference's first fx-showcase WebGPU run
failed once on a deferred capture answering null beside a cross-build;
`tests/web/reference.ts` now asks again, as the web's capture contract
says, and the run passes.

Left out: the browser tiers of the looks station (the station's playtest
runs natively and on lavapipe), an engine canvas texture, a true
blue-noise texture (interleaved-gradient noise stands in), depth edges in
the ink pass. *Resolved: the canvases, blue noise and depth ink in
stylisation round two (827652f, e1a248f, 9db993c), which also made the
seven partial catalogue entries full (735600b); the looks station in
web-reference in round three (70b1187).*

## Phase 4, UI shell round (2026-10-21)

Branch `ui-shell` from main at 7696db4, merged as bdfd8ae. Evidence:
`tests/evidence/P4/ui-shell/` (`NOTES.md` indexes it).

- **The accessibility mirror and the page's preferences** (d99c7c9): the
  `a11y` injected kind (an engine extension of protocol 5: focus, activate,
  increment or decrement the control at a path) and `og.A11yAction`; the
  web shell mirrors og-ui's tree into the page (an invisible element per
  node at its bounds, ARIA role, label, value and states, focus following
  the UI's, status and alert nodes into live regions), and a screen
  reader's click or focus there is a request; `prefers-reduced-motion` and
  `prefers-contrast` reach the theme every frame. `tests/web/a11y.ts` in
  web-reference: 23 of 23 checks on each tier.
- **Materials** (d99c7c9): glass over overlay layers (the overlay drawn
  into the glass source, the pyramid rebuilt before each such glass run);
  the edge light on opaque dark surfaces; icon sets cooked like fonts
  (`type icons`, SVG paths into an `og.Font` blob; the kit ships 64 iconoir
  icons, MIT); aspect ratios; two touches scroll a container, pinch and pan
  `og.ui.zoomView`.
- **Style packs at their bars** (7214094): every pack's labels at 4.5:1 as
  drawn, primary and secondary at 7:1 (`pack_contrast.rs`); the lowest row
  is 4.54:1, the lowest primary or secondary label 7.15:1.
- **The kit** (8d9a378): the `materials` screen and the two-finger
  playtest.
- **References**: og-ui on `metal` (a71c66b) and `vulkan-cpu` (abd61fb), 83
  frames, every vulkan-cpu one without outliers against `metal` (RMSE at
  most 0.0019); the template vulkan-cpu frames the UI text round left stale
  (10cf48d: match3 7, runner 8, fx-showcase 21 suites); fx-showcase's and
  topdown's browser frames deleted where `metal` holds them (74e291c: 47
  and 8; WebGL2's water stays, no reflection march there by design).

Glass cost (`glass-cost.md`, release, Metal, 1280×720, best of three runs
of 600 frames, load 43 to 48):

| case | ms per frame | over no glass |
|---|---:|---:|
| no glass | 0.414 | |
| four panels over the scene (one pyramid) | 1.886 | +1.47 |
| the four over an overlay bar | 1.419 | +1.01 |
| two layers of two panels, a bar under each (two pyramids) | 4.604 | +4.19 |

Size (brotli, against main 376f3cd): 3D publish +8,268 B (2,921,812), 2D
publish +8,591 B (2,758,190), lanes +7,332 B (3,297,822, 78.6 % of 4 MB).
Gate (path-aware, on 18550a3): fmt, clippy, drift, determinism (904 s),
wasm-build (1,047 s), wasm-opt (372 s), test (618 s) and size-gate
passed; bun-test failed on five timeouts at load 60 to 80, not rerun.
web-reference at a71c66b: the main gate above.

Left out: og-ui's accessibility, materials and input remainders (rows 60
and 61), a quieter rerun of the glass cost (row 61).

## Phase 4, shadows and antialiasing round (2026-10-21)

Branch `r-shadows-aa` from main at 20a6d47, merged as bcb0e4c. Evidence:
`tests/evidence/P4/shadows-aa/` (`NOTES.md`); the design is og-render's
README §Shadows and antialiasing by tier.

- **og-render** (f507334): up to 8 shadowed point and spot lights on the
  full tier and 4 on WebGL2, the brightest at the camera first;
  `PointShadows` (`Auto`, `Cube`, `Paraboloid`: two hemispheres, two
  passes instead of six); PCSS soft sun shadows (`shadowSoftness`) and
  cascade blending (`cascadeBlend`), both off by default; FXAA 3.11 on any
  tier, and where the HDR target cannot be multisampled; TAA's Catmull-Rom
  history and sharpening. 51 og-gfx frames byte-identical on Metal before
  and after.
- **API** (5bd6bc8): `RenderOptions.pointShadows`, `taaHistory`,
  `taaSharpen`, `Antialias.fxaa`, `DirectionalLight.shadowSoftness` and
  `cascadeBlend`; `RenderCapabilities.maxShadowedLights` and
  `pointShadows`.
- **Particles** (6fb84fd): the CPU path (WebGL2) bounces off
  `RenderScene::particle_colliders`, which og-runtime fills from og.Collider
  boxes, spheres and terrain height fields.
- **Scenes** (556b838, 4ddf4f2): shadows-soft, point-shadows-paraboloid,
  shadows-local-8, fxaa, taa-sharp on `metal` and `vulkan-cpu` (RMSE 0.00135
  to 0.00178 against `metal`); Ember Vale's WebGL2 beacon re-recorded
  (d1821c9: sparks bounce, RMSE 0.01852 to 0.01645 against `metal`, still
  over 0.01 by design).

What `Auto` picks and the costs (frame ms; Chrome for Testing best of 7
runs of 60 frames; Metal release, three runs at load 58 to 78):

| frame | WebGL2 | WebGPU | metal |
|---|---|---|---|
| point-spot-lights, cube faces (7 passes) | 0.702 | 0.445 | 0.57–0.89 |
| point-spot-lights, paraboloids (3 passes) | 0.503 | 0.435 | 0.44–0.53 |
| shadows-local-8, cube (WebGL2 4 lights) | 1.085 | 1.332 | 2.19–2.70 |
| shadows-local-8, paraboloids | 0.602 | 0.600 | 0.82–1.01 |
| shadows-soft hard / PCSS 0.03 + blend 0.2 | 0.618 / 1.01 | 0.412 / 0.635 | 0.40–0.43 / 0.83–0.87 |
| thin geometry: no AA / FXAA / 4× MSAA | 0.668 / 0.75 / 1.113 | 0.412 / 0.495 / – | 0.42 / 0.52–0.54 / – |
| TAA bilinear / Catmull-Rom + 0.5 sharpen | – | 0.493 / 0.552 | 0.63–0.67 / 0.69–0.82 |

So `Auto` is cube faces on the full tier and paraboloids on WebGL2. Size
(brotli, merged with main 376f3cd, at b6b447d): 3D publish 2,926,692 B
(+13,148, 69.8 % of 4 MiB), 2D publish 2,759,931 B (+10,332), lanes
3,300,932 B (+10,442). Gate (path-aware, never `--all`): every step passed
at 6fb84fd, b6b447d and 2651887 but bun-test, which failed on load
timeouts, stale template cooks (cooker r4) and main's `toBeDefined`
typecheck (fixed in 2651887); the same 119 tests pass run directly.

Left out (not this round's): fx-showcase's `fx` and `2d` browser frames
stale against their own references after the UI text round's kerning.
*Resolved in the UI shell round (74e291c): deleted, judged against
`metal`.*

## Phase 4, perf round (2026-10-20)

Branch `r-perf` from main at 5dc3aa0. Evidence: `tests/evidence/P4/perf/`.

- **Owed row 50, the per-instance cost** (dedf817, 3bec9d9, ef3ee89): a
  CPU profile of the engine Worker found 85 % of the sphere field's frame in
  `og_lanes::wait_for`: extraction decoded every og.Mesh row to JSON on the
  lanes, and the lanes wasm's Workers serialised on its one allocator (the
  step near 4,096 spheres is og_lanes splitting at 2 x 512 x 8). og.Mesh and
  og.Transform are now read from their columns, the hierarchy is dense and
  reused, and the wasm decodes JSON rows serially. Chrome, 20,000 spheres:
  174 ms (137.6 ms reproduced) to 5.96 ms on WebGPU and 5.79 ms on WebGL2
  at load 33 to 62 (4.1 ms at load 14); the §7 reference scene now meets
  1.5x on both tiers (WebGPU 2.04x, WebGL2 1.54x against InstancedMesh).
  The field is 0.31x of three.js's WebGPU InstancedMesh and 0.48x of
  WebGL2's: it is GPU-bound now, in the lit pass's per-vertex and
  per-fragment cost (`threejs-spheres.md`), which stays owed in og-render's
  README. Guards: og-runtime `tests/extract_cost.rs` (in the gate: linear,
  under 40 ms in dev, no og.Mesh row on the lanes; perf
  `extract-spheres-20k`) and og-tests' perf `spheres-20k-moving`;
  `tools/bench/profile-spheres.ts`, `packages/engine/bench/spheres.ts`.
- **Instance upload dirty tracking** (12749c1): only changed 4 KiB blocks
  are written, both culling paths, `FrameStats::instance_upload_bytes`.
- **Resources can be removed** (db105bd): `remove_mesh/texture/material/
  font`, generation-tagged handles, slots reused; the runtime unloads a
  streaming group's model textures through `remove_texture`. *Extended in
  the resources round (01700f2, 0fc358a): `remove_environment_map`,
  `remove_texture_array` and `remove_palette` the same way, and a group's
  maps, arrays, palettes and looks are let go when it unloads (kept with a
  warning while the frame uses them); `FrameStats::texture_bytes` counts
  arrays, maps and palettes. The sprites-arrays round adds `remove_lut`, LUTs
  in groups, and the kept assets' bytes in the stream budget (`kept`).*
- **Shaders** (912e333, 8779ca8, bd193d1): every WGSL module of both tiers
  validated by naga and translated to MSL, SPIR-V or GLSL ES 3.0 in the
  gate; a Vulkan pipeline cache across runs (`OG_PIPELINE_CACHE`); lavapipe
  keeps no pipelines in it and Metal and the browsers have none in wgpu, so
  first frames are unchanged there (`shaders-and-pipeline-cache.md`).
- **Cluster build in compute**: measured, not kept (CPU build 38 to 53 us
  for 256 lights; the cheapest compute build 97 to 134 us of CPU and 279 to
  385 us of GPU; `clusters-compute.md`).
- **Depth prepass every frame**: measured, kept on demand (spheres-20k
  +66 %); design §2.6 changed (38d4d21, `prepass.md`).
- **vulkan-cpu terrain-grass** (5d9a04e): a stale reference since 1ced372;
  re-recorded in the og-lavapipe container, every other vulkan-cpu scene
  passes (`terrain-grass/`).
- **Lanes tests** (69953d4): lanes.test.ts no longer expects the
  extraction site to split; lane-panic.ts injects its fault at the cull site.
- **Frames**: every og-gfx frame on Metal is byte-identical to main's
  og-render (51 of 51); one reference re-recorded (vulkan-cpu
  terrain-grass).

## Phase 4, cook and animation round (2026-10-20)

Branch `cook-anim` from main at 5dc3aa0, merged with main 7696db4 and
merged as 376f3cd. Evidence: `tests/evidence/P4/cook/` (`NOTES.md`).

- **Animation samples** (ff6f056): `animation { compress true tolerance
  t }` (off by default, t 1e-4) cooks rotations as 48-bit smallest-three
  and translations, scales and morph weights as unorm16, then drops every
  frame its neighbours rebuild within t; always and losslessly, constant
  channels are stored once and morph targets keep only the vertices they
  move. Every committed animated model's `#anim`: 179,465 B before,
  161,898 B f32 now, 133,722 B compressed, the samples 62,504 to 42,968 to
  14,770 B; the largest lane error 9.97e-5, node error 0.149 mm; the posed
  trickster from both cooks at RMSE 0.0028 (`anim-compress.md`).
- **Long sounds in chunks** (5ceeb3b): `sound { stream true chunk 4 }`, or
  a QOA file over 1 MiB, cooks chunks of whole QOA frames that the
  playheads stream (0.5 s behind to 4 s ahead); a 3-minute 48 kHz stereo
  track (6,989,848 B of QOA, 46 chunks) looped with a crossfade mixes
  bit-identical to the whole file with at most 918,200 B resident
  (`sound-chunks.md`).
- **KTX2 and texture arrays** (047b250): KTX2 sources' orientation and
  swizzle honoured; `type texture-array` cooks to `og.TextureArray`
  uploaded as one `D2Array` (`og.load.textureArray`); the `toktx`
  fallback's new test found two bugs (a shared temp directory, an empty
  version in every stamp), both fixed.

Size (brotli, merged with 7696db4): 3D publish 2,913,544 B (+13,565,
69.5 % of 4 MiB), 2D publish 2,749,599 B (+14,155), lanes 3,290,490 B
(+12,533). Gate (path-aware, on the merged tree): 9 of 9 in 207 s (test
163 s: nextest 624 passed, 9 skipped). No reference frame moved
(compression is opt-in; cooker r4 re-cooks everything).

Left out: texture arrays had no consumer. *Resolved in the resources
round (01700f2, 0fc358a, ade313c) and the sprite arrays round two
(8cba44b, 8150acf).* Compressed clips are rebuilt to f32 at load, so a
resident clip is no smaller (row 56).

## Phase 4, UI text round (2026-10-20)

Branch `ui-text` from main at b62dd65, merged with main at 9b7f45c, 5dc3aa0
and 20a6d47; main moved to it at 7696db4. Evidence:
`tests/evidence/P4/ui-text/` (`NOTES.md`).

- **Kerning, error correction, bidi** (5234b15): kerning pairs from the
  cook to both text layouts, msdfgen's error correction, bidi by UAX #9.
- **Editing** (1a43fe7): the typed-text stream (og-input's `text` kind and
  `og.TextInput`), the probe's `type:`, `compose:` and `commit:`,
  selection, the clipboard, text areas, input-method composition, ellipsis
  and `maxLines`, the caret blink; the web shell's hidden textarea; glyphs
  outside a cooked charset rendered at run time from the font's outlines
  (`runtime true`; 16 CJK characters at 32 px/em 1.26 ms on average, at
  most 2.37 ms, `glyph-costs.json`).
- **The kit** (301c79f): the display and rounded faces, a text screen,
  run-time CJK glyphs on the locale screens; zh-CN glyphs three archetypes
  drew missing, and floating text takes the UI's font (2a8d66d).
- **Frames**: og-ui text-bidi, text-kerning and text-edit new on `metal`
  and `vulkan-cpu` (203d067, 033bf00); the kit's 70 frames re-recorded and
  three new on both (d4af2af, ea5e9b1; vulkan-cpu against `metal` without
  outliers, RMSE at most 0.0019); 49 template frames on `metal` (2031f94,
  `templates-metal-moved.tsv`).

Size (brotli, before the merges): publish 3,205,718 to 3,243,486 B
(+37,768, 77.3 % of 4 MB), rustybuzz 130,265 B of it; after merging
20a6d47: 3D publish 2,899,979 B (+36,158), 2D publish 2,735,444 B
(+38,408), lanes 3,277,957 B (+40,485, 78.2 %). Gate (path-aware, after
the last merge): every step passed but bun-test, which failed once on a
stale cook of the reference scene and passed after `og cook`.

Left out: the templates' vulkan-cpu and browser frames were not
re-recorded. *Resolved: vulkan-cpu in the UI shell round (10cf48d), the
browser frames deleted where `metal` holds them (74e291c), Ember Vale's
WebGL2 beacon re-recorded in the shadows and antialiasing round
(d1821c9).* og-ui's cooked-font text frames in the browser tiers (row 58).

## Phase 4, image-based lighting and post round (2026-10-20)

Branch `r-ibl-post` from main at b62dd65, merged with main 9b7f45c and
5dc3aa0 and merged as 20a6d47. Evidence: `tests/evidence/P4/ibl-post/`
(`NOTES.md`).

- **AgX and exposure adaptation** (191e53b): `Tonemap::Agx` beside ACES
  (`RenderSettings.toneMapping`); `Exposure::Auto` adapts by
  `adaptSpeedUp`/`adaptSpeedDown` EV a second over the scene's ticks, no
  wall clock; the adapted EV is carried by snapshot, restore and sweep.
- **Environment maps** (3e360bf, 274a8aa): `type environment` cooks an
  equirectangular `.hdr`/`.exr` into a prefiltered RGB9E5 map;
  `og.load.environment` and `og.Environment` (map, intensity, rotation,
  background) light the scene in the sky's place.
- **The environment station** (6a707ad, b664627, 8386437): five spheres
  under a cooked studio, AgX, adapting exposure, on every tier.

References (RMSE against `metal`): post-agx on vulkan-cpu 0.00128, webgpu
and webgl2 0.00004; environment-map 0.00097, 0.00001, 0.00000; the
station 0.00113, 0.0005 and 0.00777 (WebGL2's MSAA in place of TAA, its
own reference). No existing reference moved. Determinism: two renderers
adapting over 330 ticks draw byte-identical frames and a restore redraws
tick 120 byte for byte; the station ends on world hash db857402d8f25a35 on
macOS and on lavapipe. Cook: the studio 0.52 s, 132,343 B. Size (brotli,
after merging 5dc3aa0): 3D publish 2,863,821 B (+9,956, 68.3 %), 2D
publish 2,697,036 B (+8,775), lanes 3,237,472 B (+6,690, 77.2 %). Gate
(path-aware, at 3a19da8): 9 of 9 in 899 s; web-reference passed (427 s).

Left out: streamed maps were never evicted. *Resolved in the resources
round (01700f2, 0fc358a).* Environments from KTX2 cube maps and six faces
came in cook round three (df6dfcf).

## Phase 4, web host and tools round (2026-10-20)

Branch `web-host-tools` from main at b62dd65. Evidence:
`tests/evidence/P4/web-host-tools/`.

- **A panic inside a lane** (a4f0231): the lanes wasm's engine thread no
  longer waits on rayon's latch; og-lanes hands each parallel call to the
  pool and waits on a latch of its own that a recorded panic also rings,
  then panics with the lane's line, so the Worker traps instead of
  sleeping in `Atomics.wait` (the lanes round's "Left"). Every panic is
  recorded with its message, location, tick and lane site and reaches the
  page as a structured host error (`{ kind, message, tick, lane,
  location }`: the Worker's crash, `Shell.error`, `onCrash`,
  `__runtime_probe__.error`, the error card); the shell adds a 15 s frame
  deadline (`hang`). `tests/web/lane-panic.ts` (in web-reference): the
  crash is on the page 9 ms after an injected fault (lanes, `lane:
  "extract"`) and 6 ms single-threaded; with rayon's wait put back the
  page hangs until the deadline. docs/web-export.md §Errors.
- **`og dev` and signals** (6a44eaf): SIGINT, SIGTERM and SIGHUP stop the
  game's whole process group, then og ends by the signal
  (`crates/og-cli/README.md`; the tooling round's "Left out").
- **A publish wasm per profile** (53a36fd, row 44): sizes in the owed
  table; `og build --publish` ships the variant of the profile and records
  og-build.json `variant`.
- **Fantasy pack and Ember Vale's HUD** (718fb43, row 48); **og.IkChain's
  world space** in the IDL (868d1b2, row 49).
- **GPU culling threshold** (5b0bff4): measured again on Metal with the
  cull serial, the crossover is still about 16,000 instances (GPU/CPU
  0.98 street, 1.06 aerial at 16,384), so `GPU_CULLING_MIN_INSTANCES`
  stays (`culling-threshold.md`; the culling round's "Not done").
- **three.js comparison** (df9c42a, `threejs-comparison.md`,
  `tools/bench/threejs-compare.ts`, headless Chrome for Testing 150, load
  41 to 120): the §7 reference scene runs at 495 fps on WebGPU and 328 on
  WebGL2 against three.js WebGL2's 255 (InstancedMesh) and 219 (one Mesh
  each), so §7's 1.5× holds on WebGPU (1.94×, 2.26×) and not on the WebGL2
  fallback against InstancedMesh (1.29×). A field of 20,000 spheres each
  moved every frame runs at 6 fps on both tiers against three.js's 76 and
  378: owed row 50.
- **Gate**: `scripts/check.sh` (path-aware full lane) at 4e121cd PASSED, 9
  of 9: fmt 6 s, clippy 67 s, drift 97 s, wasm-build 877 s, test 649 s
  (nextest 566 passed, 9 skipped), wasm-opt 588 s, size-gate 103 s (3D
  publish 2,835,047 B, 2D 2,672,159 B, lanes 3,212,339 B: 76 % of the
  per-download budget), bun-test 160 s (114 engine and 19 knowledge
  tests), determinism 1,591 s; load 35 to 120 with a lavapipe container
  and other gates on the machine. `--all` is the coordinator's.

## Phase 4, physical materials round (2026-10-20)

Branch `r-materials` from main at b62dd65, merged as 9b7f45c. Evidence:
`tests/evidence/P4/materials/` (`NOTES.md`).

- **The physical lobes on both tiers** (6d3c5fb, 3c6c900): three.js's
  MeshPhysicalMaterial fields on `StandardMaterial` (version 4): clearcoat
  maps, sheen, transmission with volume and `ior`, iridescence, anisotropy;
  a physical material draws its lit pass through `physical.wgsl`, and
  WebGL2's sixteen texture units hold its fourteen scalar maps packed into
  one RGBA8 array. The standard modules are split at their old line
  boundaries, their text unchanged.
- **The cook** (3691e40): glTF's `KHR_materials_clearcoat`, `_sheen`,
  `_transmission`, `_volume`, `_ior`, `_iridescence` and `_anisotropy` into
  `og.ModelMaterial` version 2 (cooker r3).
- **WebGL2** (a46413b): naga renamed a second `gl` local to the reserved
  `gl_1`; renamed, and og-render's `tests/webgl2.rs` now fails any
  `gl_<digit>` identifier.
- **The station** (b568308): fx-showcase's `physical` station and its
  playtest; docs in the og-game skill and og-render's README (c6ea371).

Tiers (`materials-physical`, RMSE against `metal`): webgl2 0.00000,
webgpu 0.00003, vulkan-cpu 0.0018 (7 outlier pixels); the station on
vulkan-cpu 0.00128. The 48 earlier og-gfx frames are byte-identical before
and after on Metal and on lavapipe. Size (brotli): publish 3,205,718 to
3,224,361 B (+18,643, 76.87 % of 4 MiB), lanes +17,361 B. Gate: the
concurrent path-aware run (load 45 to 60) failed bun-test on four 5 s
timeouts; on the same tree each step then passed on its own (test 575 of
575, determinism 215 s, bun-test 44 s).

Left out: nothing named. A transmissive object shows the opaque scene, the
sky and terrain behind it, not water or other transparent objects, as in
three.js (og-render's README §Physical materials).

## Phase 3, lanes round (2026-10-19)

Branch `p3-lanes`, 211fc40..HEAD. Parallel systems (docs/design.md §2.2)
and the §7 reference scene measured with lanes off and on. Evidence:
`tests/evidence/P3/lanes/` (commands, machine load and tables in its
`README.md`).

- **Lanes** (100419e, b22e3e5, 8b91954): `crates/og-lanes` wraps rayon
  (`map` and `for_each_mut`, both in input order, `install` for a rayon
  library) behind one switch, `OG_LANES` (`off`, `0` or `1` serial, `N`
  threads, unset the cores up to 8) and the probe's `lanes:<on|off>`;
  `dump_lanes` and `og_web_lanes` report threads, pool and the parallel
  calls that ran, in total and per site (`anim`, `extract`, `cull`,
  `physics`; lanes.test.ts needs the first three to split). On lanes: og-anim's graph evaluation and palettes (64
  skeletons a task), extraction's row decoding (512 rows a task), og-render's
  frustum cull (1024 bounds a task; scheduling only, recording stays on the
  engine thread) and rapier's solver natively. A call splits only from two
  tasks' worth: waking the pool cost 0.5 to 2.6 ms a call on the loaded M4
  Pro against 21 us for the reference scene's whole animation pass, so the
  §7 scene stays serial everywhere but the native physics step. With lanes
  off a rayon library runs inline on the calling thread (`use_current_thread`;
  hop p50 1 us, was 110 us). TS systems keep their order on the engine
  thread.
- **Same world, lanes on or off** (a39d4f8, a236969, 6fcfea8): the gate's
  determinism step runs the host release with lanes off and on four threads
  and fails when the second ran nothing in parallel;
  `packages/engine/tests/lanes.test.ts` runs the reference scene plus a
  crowd (2000 boxes, 160 walkers, 48 crates) in two processes and finds the
  same world hash and captured pixels at ticks 60, 180 and 300 and the same
  replay with lanes switched mid-run. Results in completion order fail it.
- **The lanes wasm** (b09df7f, 2728f9c): two web modules, decided and
  documented in `docs/web-export.md` §Lanes. An atomics module cannot load
  on a page that is not cross-origin isolated, so `origin-engine_bg.wasm`
  stays single-threaded and `og build --publish` (isolation not `off`) also
  ships `origin-engine-lanes_bg.wasm` (`scripts/wasm-lanes.sh`: `-Z
  build-std` with `RUSTC_BOOTSTRAP=1`, its own target dir; docs/ci.md §The
  lanes wasm). The shell takes it when the page is `crossOriginIsolated`
  with more than 2 cores (`min(cores - 1, 8)` Workers through
  wasm-bindgen-rayon) and otherwise, or when the pool fails, boots the
  single-threaded module with the same API; the mode is on `ready.lanes`,
  the log, `__runtime_probe__.lanes` and `call("lanes")`. The service
  worker precaches the variant the shell's rule picks (`src/web/lanes.ts`,
  shared) and, once the engine is ready, caches the variant the page booted
  (`og-cache`). A pool that does not start within 10 s or whose Worker fires
  `error` (a CSP without `worker-src blob:`; `tests/web/export.ts` serves
  one) is terminated and the single-threaded module boots. COOP/COEP stay
  per publication (`_headers`, `origingame.runtime.json`).
- **Wasm** (the gate's size-gate at 1f56ad7): publish 3,153,927 B
  brotli, lanes 3,156,913 B (75.3 % of the 4 MiB budget, which the size
  gate now holds per file; the round grew the single-threaded module by
  about 4 KB).
- **Frame times** (e30fa0a, `bench/lanes.ts`, release, 1280x720, 600
  frames, M4 Pro at load 27 to 35 natively and up to 110 in Chrome), p50 /
  p95 / p99 ms:

  | run | lanes off | lanes on |
  |---|---|---|
  | Metal, reference scene | 6.968 / 14.668 / 17.598 | 6.475 / 9.031 / 10.329 (8 threads, physics only) |
  | Metal, crowd | 6.063 / 9.528 / 10.413 | 5.808 / 6.717 / 7.831 (8 threads, 5940 parallel calls) |
  | Chrome WebGPU, not isolated | 1.2 / 2.3 / 3.1 | (single-threaded) |
  | Chrome WebGPU, isolated | 1.19 / 2.235 / 3.225 | 1.265 / 2.305 / 3.16 (8 Workers, 0 parallel calls) |

  Each native pair ends in the same world hash and the three Chrome runs
  capture the same frame at tick 660. **§7 budget (16.7 ms)**: p95 held
  in every run and p99 in all but one, native lanes off at 17.598 ms, the
  GPU wait under host load (CPU p99 3.8 ms); single frames went over (max
  20.6 ms native, 33.1 ms in Chrome not isolated); earlier runs at load
  75 to 220 reached 23 to 59 ms at p95/p99 in both modes, so it should be
  repeated on a quiet machine. In Chrome lanes change nothing for this
  scene: nothing crosses the grains and physics is serial on the web. The
  Three.js comparison is an owner measurement and was not done.
- **Resolved later** (2026-10-21, branch `web-physics-lanes`): rapier runs
  in parallel in the lanes wasm too (its `parallel` feature under
  `target_feature = "atomics"`, reaching the Worker pool through a relay
  Worker, crates/og-lanes/README.md), and the step has a body threshold:
  `og_lanes::install` goes to the pool only from `og_physics::BODY_GRAIN`
  (200) bodies for every lane thread, so a small world, the reference
  scene's included, steps on the engine thread with lanes on, natively and
  on the web, and no longer pays the pool hop. The whole og-physics step
  runs inside it and its contact readout splits by pairs; lanes off and on
  give the same world (tools/wasm-determinism with a 1,000-body pen,
  og-physics's `tests/pile.rs`, lanes.test.ts's 1,000-box pile).
  Native timings: `tests/evidence/P4/physics-lanes/`; the browser timing
  and the lanes wasm's hashes are the coordinator's measurement.
- **Round two** (2026-10-21, branch `physics-lanes-2`): the step's pool
  threshold is per thread count (`og_physics::pool_bodies`: 500 bodies on
  two to four lanes, 1,000 on five or six, 1,500 from seven, through
  `og_lanes::install_from`), so four lanes no longer leave their 7 to 15 %
  between 500 and 800 bodies serial; ropes, cloths and fluid sets step side
  by side and the fluids' neighbour search and collisions split by ranges
  of particles (`og_physics::Grains`; ragdolls measured and left serial);
  queries outside the step run through `og_lanes::serial`, and the lanes
  wasm's engine thread belongs to its one-thread pool from
  `og_lanes_ready` on, so nothing reaches a rayon pool from the engine
  thread but the relay with its panic latch (og-physics `tests/lanes.rs`
  injects a panic in a query). The determinism tool's physics hash does not
  move (18126409214070009362). `bench/lanes.ts --chrome --pile` times the
  lanes wasm's step on 1,000 and 2,000 boxes: the coordinator's run.
  Native measurements: `tests/evidence/P4/physics-lanes-2/`.
- **Gate**: not completed on this branch (the main gate at 50eb7f7 above
  passed everything). `OG_GATE_JOBS=2 scripts/check.sh
  --all` at 1f56ad7, 2026-10-19, M4 Pro at load 127 to 237: fmt, clippy,
  drift, wasm-build, test (Rust workspace), wasm-opt, size-gate and
  determinism (lanes on: 4 threads, 1200 parallel calls, every build
  agrees) passed; bun-test was stopped from outside mid-run (exit 143,
  the gates moved to main after merge), with two engine tests over their
  5 s timeout at that load (asset-stream, render trails) and
  web-export.test.ts unfinished. Earlier, `scripts/check.sh quick` at
  e30fa0a passed fmt, clippy, drift and bun-quick and failed test-quick on
  og-audio's wall-clock mix bound (1.7 s against 1 s in debug at load 55
  to 122; og-audio is untouched here). `bun test tests/lanes.test.ts`
  passes and fails with `og_lanes::map` broken to completion order.
- **Pre-existing**: `tests/web/export.ts --webgl` fails on base 211fc40 too
  (overlay.wgsl `textureNumLevels` on WebGL2, from the UI round's 42d7294;
  fixed on p2-showcase).
- **Left**: `og run --device` (another branch this round) must serve a
  build with its `_headers` so a phone gets lanes; the platform's pin bump
  and ask 11 (`docs/platform-asks.md`: keep COOP/COEP on lanes releases,
  allow `blob:` Workers, check `__runtime_probe__.lanes` on Play); the
  crowd has no Chrome measurement; whether
  the lanes wasm's 0.07 ms a frame over the single-threaded one (mean, same
  serial work) is noise or the atomics build is not separated; a panic
  inside a lane on the lanes wasm leaves the engine Worker in
  `Atomics.wait` and the page hung without the crash screen (the shell has
  no deadline on a frame or an engine call to notice; docs/web-export.md
  §Lanes). *The panic resolved in the P4 web host and tools round
  (a4f0231: a structured crash 9 ms after the fault, and a 15 s frame
  deadline); ask 11 still waits on the platform, its text now saying the
  lanes run the physics step on the web too.*

## Status 2026-10-19, Phase 2, showcase round

Branch `p2-showcase` from main at b2532cd. The phase 2 deliverable "one
mid-size 3D showcase" is `templates/archetypes/third-person`, **Ember
Vale** (1253988, assets 20dc944, `create --archetype third-person`
314e252): a lamplighter on og.CharacterController walks, runs, wades and
jumps over a 128 m terrain vale with an og.Water lake; holding F by a
beacon walks it to the plinth and an og.IkChain reaches its right hand into
the bowl until the beacon catches (fire, smoke and ember particles, a
light, a scorch decal and a rune that glows, bloom, AO, god rays, the LUT
and grain under og.DayNight as the run's clock). Three sentinels on
og.NavAgent think with og.ai behaviours and og.Perception: patrol, alarm,
chase, grab (a heart, a hit stop, a shake). Both characters play an
og.AnimationGraph (a 1D idle, walk and run blend by speed, the hero's air
state). Four beacons before night win; the hearts or night lose; the
summary, Again or Title. Screens are og.ui on the fantasy pack's tokens
with MSDF text (Cinzel, Newsreader, a Noto Sans SC subset), English and
简体中文, touch controls on a phone; every `Tuning` field is Tune's; a
voice per event with ducked music; settings and a best-run record go
through `og.platform` (the win merges against a stale write from another
device). The publish build plays behind the tap gate. Only the public
`@origingame/engine` API; the characters, clips, textures, LUT and
emitters are the engine's own CC0 assets from fx-showcase, the rest is
written by `tools/make-assets.py` (no download; not trickster.glb or
dropship.glb). It typechecks in the gate through
`packages/engine/tsconfig.templates.json`'s `templates/archetypes/*/src`
glob (row 12).

### Evidence

- **metal**: `og test templates/archetypes/third-person` passes 8 suites
  (flow, frames, lose, pause, phone, replay, sentinels, win; 109 checks)
  through real input; frames title, beacon and dawn at 0.000 % against
  `references/metal/`.
- **webgpu and webgl2**: `tests/web/reference.ts --project
  templates/archetypes/third-person` (Chrome for Testing, the release
  `dist/web`) matches the same three frames against `references/webgpu/`
  and `references/webgl2/` at 0.000 %. Against the metal references:
  webgpu RMSE 0.00131 (title), 0.00475 with 0.060 % outliers (beacon),
  0.00067 (dawn); webgl2 RMSE 0.00428, 0.0181 and 0.00180. The webgl2
  beacon, the one over the harness's 0.01 against metal, is the same scene
  (the hand in the bowl, fire, smoke and light alike), and a per-spark
  read of the three beacon captures against `references/metal/beacon.png`
  (2026-10-20) finds the difference by design on two counts. Particle
  collision against the depth buffer runs only with compute shaders, which
  WebGL2 lacks, so its ember sparks fall through the ground instead of
  bouncing: of metal's 75 sparks 61 match a webgl2 spark, and the 17
  metal-only and 12 webgl2-only sparks are 48.5 % of the squared error
  (0.419 % of the pixels as outliers). The full tier antialiases with TAA
  and WebGL2 with 4× MSAA: matched sparks differ by about half a pixel
  (median +0.60 px long, +0.70 px wide, centres 1.91 px apart) and are
  29.6 % of the error, the rest of the frame 21.9 %. Without the unmatched
  sparks webgl2 would be at RMSE 0.0130, still over 0.01 for the MSAA
  against TAA difference alone; webgpu matches metal spark for spark (75
  matched, median shift 0.18 px). On 2026-10-20 the webgpu references and
  the webgl2 title and dawn were dropped as duplicates of metal (each
  within the tolerance against it; docs/ci.md: a tier gets its own
  reference only for a difference it makes by design), so both tiers are
  judged against `references/metal/` except the webgl2 beacon, which
  keeps `references/webgl2/beacon.png` by design. (2026-10-21, the
  shadows-aa round: the CPU particle path now bounces off the scene's
  colliders, so WebGL2's sparks bounce too; against main's re-recorded
  metal beacon the WebGL2 frame went from RMSE 0.0185 to 0.0165, still
  over 0.01 for the MSAA against TAA difference and the sparks' paths
  after a bounce, and its reference was re-recorded; docs/ci.md §Browser
  reference tiers, `tests/evidence/P4/shadows-aa/`.)
  `scripts/check.sh web-reference` now runs these frames on both tiers.
- **vulkan-cpu**: not recorded (the gate does not run templates on it).
- **og polish** (`tests/evidence/P2/showcase/`, metal, release addon):
  taste 95, feel 85. Seven findings, recorded and not fixed: six labels
  under 4.5:1 on the fantasy pack's shared palette (Set out 3.36:1, the
  footer 3.5, Resume 3.84, Again 3.99, 完成 4.41, 语言 4.43), and on the
  phone 2 of 3 presses without a ui-bus sound (the HUD's Jump and Kindle
  answer on sfx). Frame time 67: the phone playtest at 1170×2532 (render
  p95 14.6 ms) under a load average of 20–30; 6.8 ms p95 or less at
  1280×720.
- **Wasm**: the publish wasm is **3,176,490 B brotli, 75.7 % of
  4,194,304 B** after merging main at 44d12a8 (3,061,324 B, 73.0 %,
  before it, against main's 3,061,522 B then): the game adds nothing to the
  engine; the growth is main's (3D SPH and the rounds before it).

### Engine fixes

- **og-nav** (d947221): a bake panicked on a triangle wholly above the
  bake box (the vale's cliffs over the navmesh bounds); such cells are
  skipped like those below it, with a test.
- This round also found and fixed the deferred start that the probe's
  `hello` broke since d2bd748 (f58417a) and the WebGL2 glass panic since
  42d7294 (c5ab008). Main fixed both the same day (4c2079a, merged in
  cf6aeaf, with `runtime.test.ts` and `crates/og-render/tests/webgl2.rs`);
  the merge f728a2a keeps main's code and tests and drops this branch's
  variants and its `core.test.ts` `defer` case. Ember Vale's metal
  references were recorded with the start after the renderer attaches, as
  main's fix also does: after the merge they pass at 0.000 % outliers
  (RMSE at most 0.0004, main's other render changes), not re-recorded.

A documentation gap, left as is: an og.IkChain target in `"world"` space
is in the frame of the skeleton body's Transform parent (here the hero's
unrotated capsule), not true world; `src/hero.ts` subtracts the hero's
position.

### Checks

The main gate at 50eb7f7 (top of this file) later passed the whole gate
and web-reference, including the WebGL2 and gfx runs stopped below.

On f728a2a (the merge of main 44d12a8), 2026-10-19, load average 50 to
220: `OG_GATE_JOBS=2 scripts/check.sh` **PASSED, 9 of 9 in 830 s** (fmt
2 s, clippy 17 s, wasm-build 2 s, test 190 s with 551 tests, wasm-opt
135 s, size-gate 42 s, drift 2 s, bun-test 199 s with 109 engine tests,
determinism 630 s). `og test` on metal with the release addon:
third-person 8 suites, idle 10, rhythm 11, roguelike 9, tower-defense 9
(the four failed on this branch before the merge and pass with main's
re-recorded frames, 7a5e320). `scripts/check.sh --only web-reference`
was stopped by the coordinator partway: fx-showcase's webgpu frames and
the 18 E5 stations (fluids3d among them) on WebGPU passed; on WebGL2
cloth, controller and fluid passed before it stopped, the rest of WebGL2
and the og-tests gfx scenes did not run. Ember Vale's own webgpu and
webgl2 references were last verified (0.000 %) before the merge, on
484f2e0. Before the merge, on 484f2e0, `--all` passed every step but a
bun-test load timeout (rerun alone, passed) and web-reference passed in
384 s.

### Not done

Owed rows 47–49: vulkan-cpu references for Ember Vale, the polish findings
above (the fantasy pack's palette is shared by every game that uses it;
the HUD buttons' bus is the template's), the IkChain space note in the
IDL's docs. Unverified after the merge: the rest of web-reference's
WebGL2 run, Ember Vale on webgpu and webgl2, and `og test` of ui-kit,
fx-showcase, runner, match3, topdown and game (all passed before it).

## Phase 3, culling and device round (2026-10-19)

Branch `p3-culling`, off 211fc40, with main merged in at cf6aeaf (acaf7e2).
The round covered GPU-driven culling, `og run --device`, and the shore foam
the streaming round left. Evidence is in `tests/evidence/P3/culling/`,
`tests/evidence/P3/device/` and `tests/evidence/P3/streaming/foam/`.

- **GPU-driven culling** (7d845e1, 53cdc86): on an adapter with compute and
  indirect draws (native Metal, Vulkan and DX12, and the browser's WebGPU),
  og-render culls every mesh pass in compute: the camera, the planar
  reflection, the cascades and the local shadow faces.
  - The visible instances are compacted in sorted order by a deterministic
    prefix sum, and each batch is one `draw_indexed_indirect`.
  - The camera pass also drops instances whose box lies behind a depth
    pyramid. The pyramid is built from last frame's visible occluders, drawn
    at this frame's camera.
  - WebGL2, `Culling::Cpu`, object motion blur and frames over the
    storage-binding limit keep the CPU path.
  - `FrameStats` gains `gpu_culling`, `occluded`, `occluder_draws` and
    `counts_lag`, and `og-gfx` takes `OG_GFX_CULLING=cpu`.
  - `crates/og-tests/tests/culling.rs` renders eight scenes both ways over
    several frames. It checks for the same picture, the same frustum counts
    and never more drawn.
  - On lavapipe, all 47 og-gfx scenes then present (48 with main's fluid-3d, which the merged gate covers) compare with their references exactly
    as the CPU path does. That needed one fix first, 53cdc86: llvmpipe
    reports level 0's size for every pyramid level, so each level's size is
    now computed from the screen's.
- **Culling review fixes** (branch `fix-culling`):
  - `Culling::Auto` takes the GPU path only from
    `GPU_CULLING_MIN_INSTANCES` (16,384) instances, from the measurements
    below: about 1.1 ms saved at 40,000 instances, 0.4 to 1 ms lost on
    scenes of tens. `Culling::Gpu` forces the GPU path, and `og-gfx` takes
    `OG_GFX_CULLING=gpu|cpu|auto`. The 47 gfx scenes are small, so under
    `Auto` they now cull on the CPU; `culling.rs` covers the GPU path with
    `Culling::Gpu`.
  - A frame with no occluder material, or no earlier camera visibility
    (the first frame, after the scan buffer grew), skips the occluders'
    pass, their depth and the pyramid, and culls in one compute pass.
  - The counts are copied back only after
    `Renderer::request_culling_counts` (og-tests-web asks every frame;
    natively `finish_stats` copies the last frame's late).
  - The buffers are sized for the frame's passes; a readback ring buffer
    in flight is kept when they regrow.
  - `culling.rs` compares the two paths at channel 2, 0.01 % outliers and
    RMSE 0.002, checks `gpu_culling` on every frame, and checks that the
    triangles drawn plus those hidden (`FrameStats::occluded_triangles`,
    counted on the GPU per hidden instance) cover the CPU's
    `in_view_triangles`.
- **Culling measured** (1f94224, b026d89, 757eca0) on og-gfx's perf-only
  city scenes, 40,000 instanced buildings at 1280x720.
  - **Metal**: over 40 interleaved rounds, `Renderer::render` takes 5.27 ms
    against 6.28 ms before on city-street, a paired ratio of 0.81. On
    city-aerial it takes 5.13 against 5.09 (0.97). Against the same binary
    culling on the CPU, the ratios are 0.83 and 0.86.
  - On the street, occlusion leaves 8,150 of 129,938 triangles.
  - Process CPU time per frame (user+sys) does not separate the paths beyond
    the noise: 3.0 to 3.6 ms for every variant.
  - The CPU saving is modest because both paths share the larger work:
    sorting, the instance upload, the bounds and encoding. The GPU saving is
    in the triangles.
  - **Chrome**: WebGPU with GPU culling takes 0.94 to 1.26 ms per frame,
    against 1.55 to 1.81 ms culling on the CPU, about 35 % less. WebGL2
    takes 1.84 to 2.27 ms.
  - `reference-gfx.ts` gains `--cpu-culling` and `--perf-out`, and times the
    city scenes when `--scene` names them.
  - Every native figure was measured on a machine shared at load averages
    of 25 to 200, so read the medians and ratios, not single runs
    (`culling/NOTES.md`).
- **`og run --device chrome|android[:serial]|iphone[:udid]`** (c8a0b9f,
  05e2df6) cooks and serves the publish build with COOP/COEP. It opens the
  page in Chrome over the DevTools protocol (Android through adb) or in
  Safari through safaridriver, taps the gate, and times the first frame and
  the played frames. It writes `og-out/device/{report.md,device.json}` in
  og polish's shape.
  - Chrome on WebGPU: first frame 270 ms, display p95 9.82 ms at 120 Hz.
  - Chrome on WebGL2, run again after main's glass fix: first frame
    1125 ms, engine work p95 1.41 ms.
  - `packages/engine/tests/web/device.ts` passes 27 of 27 checks.
  - The phone paths are written, and their prerequisites documented, but
    they are not exercised (owed row 40 remains the owner's).
- **Shore foam on vulkan-cpu** (ec54c60): large water now draws in cells of
  at most 256 m (`MAX_CELL`), so `terrain-far`'s 24 km sea is 94 x 94
  cells, not one quad. llvmpipe draws the foam line and the lake again, and
  Metal's foam band narrows to the same thin line. Metal against vulkan-cpu
  goes from RMSE 0.01771 to 0.00534. The sin/cos hypothesis was refuted,
  and `water.wgsl` is unchanged.
  - Re-recorded: `metal/terrain-far.png`, where the band is now a thin
    line and the old reference fails at 0.01561.
  - Re-recorded: `vulkan-cpu/terrain-far.png`, where the foam line and the
    lake edge appear (0.00944).
  - No other reference moved.
- **web-reference** (`scripts/check.sh --only web-reference`, after the
  render changes): PASSED on both tiers. That covers fx-showcase's
  stations, the E5 stations and og-tests' 47 gfx scenes, with GPU culling
  on WebGPU.
- **Gate** (`OG_GATE_JOBS=2 scripts/check.sh --all` on 6a99ead, at load
  averages of 150 to 300): fmt, clippy, wasm-build, test (535 passed, 9
  skipped), wasm-opt, size-gate and drift passed. bun-test failed on three
  tests over bun's 5 s default timeout. A rerun failed on one, the trails
  test (5.6 s). With `--timeout 60000`, all eight render tests pass in
  15.4 s. Determinism did not run, because the gate stops at bun-test. The
  final gate is run on main after the merge: see the main gate at 50eb7f7
  at the top of this file (passed, 9 of 9 and web-reference).
  - On the debug addon, the GPU path makes render.test.ts's small
    multi-renderer tests about 30 % slower: trails 1.8 s against 1.4 s with
    `Culling::Cpu`. Release og-gfx shows the same on small scenes:
    point-spot-lights 1.0 to 1.6 ms per frame against 0.6 ms, and showcase
    3.5 to 3.7 ms against 3.15 ms.
  - Pipeline creation is not the cost (`GpuCulling::new` takes 2 to 5 ms).
    The overhead is per frame, and below some instance count the CPU path
    is cheaper.
- **Wasm**: the publish build is 3,168,670 B brotli (75 % of the 4 MiB
  budget). That is +103,727 B over the streaming round's 3,064,943 B, for
  the cull shaders and pipelines, the depth pyramid and `--device`'s
  host-side code.
- **Left**:
  - Natively, GPU culling's CPU saving is about a fifth on an occluded view
    and about nothing on an open one, until the shared per-instance work
    (sorting and the whole-buffer upload; dirty tracking is still owed)
    shrinks.
  - In a browser loop that never yields, the GPU counts arrive late
    (`counts_lag` 237 in the timing loop). The frame is unaffected.
  - The phone paths of `og run --device` have not been run on a phone.
  - The 16,384-instance threshold comes from the round's measurements,
    taken before the review fixes cut the GPU path's fixed per-frame cost;
    it has not been measured again.
  - `vulkan-cpu` `terrain-grass` fails at RMSE 0.01488, on main as well as
    here. It predates this round. *Resolved in the P4 perf round (5d9a04e):
    a stale reference; 1ced372 changed the scatter and re-recorded only
    metal's (`tests/evidence/P4/perf/terrain-grass/`).*

## Phase 3, streaming round (2026-10-19)

Branch `p3-streaming`, 932dc00..HEAD. Terrain streaming, asset streaming
groups and the phase 2 terrain round's leftovers. Evidence:
`tests/evidence/P3/streaming/` (break-and-restore notes in
`break-and-restore.md`).

- **Terrain placement** (9941ffd, 6b9221b): og-render draws a terrain
  through a transform, and extraction hands it the entity's world matrix;
  og.terrain.sample, bounds and edit take the entity's rotation and scale.
- **Terrain tiles** (8c0e8bf, bc5c5ea, 8bc9174): og-render draws a
  terrain from a pyramid of 128-quad tiles in texture arrays, choosing
  them with `terrain_tiles_wanted` within `TerrainLod::budget` (16 MiB a
  terrain by default), each after its parent with a fade. og cook writes
  the pyramid (og.Terrain schema 2, every tile zlib'd on its own), and
  og-runtime's streamer decodes the wanted tiles off the frame (a worker
  thread natively; the browser's DecompressionStream on the web, the
  engine finishing at most 2 ms of tiles a frame), keeps twice the budget
  decoded and hands the renderer at most eight new tiles a frame. A
  capture settles first. The tick reads terrains through physics and
  og.terrain.sample only, never through what is resident.
- **Streaming and the world hash** (27e2f62): the probe's
  `terrain_stream:<stream|resident>[:<MiB>]` and dump_stream's `terrain`
  section; `packages/engine/tests/terrain-stream.test.ts` runs the
  fx-showcase terrain station 600 ticks resident and streamed at 0.25 MiB
  and finds the same world hash on every tick
  (`terrain/determinism.json`).
- **Frame times** (451f376, 9c7c986): `bench/terrain-fly.ts` flies the
  2 km island and a new 8 km terrain (4097², ridged) at 60 m on the native
  host, timing engine.frame plus the GPU wait (`og_node_wait_gpu`). On the
  M4 Pro at 1280x720 every run's p99 is 2.1 to 2.6 ms and no frame exceeds
  8.2 ms; streaming the 8 km terrain decodes 473 tiles at 16 MiB and 202
  at 4 MiB, the streamer spends at most 0.12 ms a frame, and handing eight
  tiles costs about 1 ms (`terrain/fly/frame-times.md`, `NOTES.md`).
- **Edits are world state** (9024a12, 6a8b6ee): every og.terrain.edit
  joins an edit log in the world's og.terrain block; snapshots keep it,
  the world hash covers it and a restore replays it.
- **Scatter** (1ced372): og-render places scattered copies once per
  32 x 32-cell chunk, caches their instances and draws each chunk with one
  instanced draw, fading copies by a dither at full size in every pass.
  The 36,000-tuft grass costs +0.46 ms (was +4.1 to +4.5 ms;
  `costs-p2-after-scatter.md`, `terrain-grass-before-after.png`).
- **Asset streaming groups** (8f19c0b, 8ae1f46, 1317314, a3d3870,
  79e8dc7, 98db532, 5481b09): a scene's `stream` blocks (distance, volume
  or request-only triggers, priority) are cooked into one pack per group
  with a proxy for each grouped glTF; the runtime admits wanted groups by
  priority within a memory budget (256 MiB by default) and swaps real
  meshes, textures and sounds in behind the stand-ins' handles, so the
  world hash never depends on what is resident; IDL module `stream`
  (og.stream.status, request, release, budget, setBudget). `og build --publish`
  ships the first group as files the shell preloads and each other group
  as its pack, fetched on demand and cached by the service worker for
  offline play. `packages/engine/tests/asset-stream.test.ts` flies a
  three-group path natively and on the web, online and offline
  (`groups/`).
- **Shared render change**: 8ae1f46 adds a replaced-texture pass to
  og-render's sync step (`RenderScene::set_texture`,
  `set_texture_upload`), outside the terrain, scatter and loading paths.
- **Gate**: `OG_GATE_JOBS=2 scripts/check.sh --all` at 58c2466,
  2026-10-19, Apple M4 Pro shared with other agents' work: **PASSED, 9 of
  9 steps in 134 s**: fmt 1 s, clippy 2 s, wasm-build 7 s, test 85 s (470
  Rust tests, 5 skipped), wasm-opt 76 s, size-gate 48 s, drift 1 s,
  bun-test 38 s, determinism 1 s.
- **Wasm**: publish wasm 3,064,943 B brotli, 73.1 % of the 4 MiB budget
  (+86,482 B over the terrain round's 2,978,461 B): the tile pyramid and
  its streamer, the scatter chunks, the streaming groups and the
  replaced-texture pass.
- **Shore foam on vulkan-cpu**: traced to the sea's single 24 km
  triangle, which llvmpipe interpolates metres off once the near plane
  clips it; any tessellation brings the foam and the lake back
  (`foam/NOTES.md`, 58c2466). Fixed in the culling round, which draws large
  water in cells of at most 256 m (ec54c60).

## Status 2026-10-19, shell own-window round

A top-level Stage build with the platform's Stage overlay in its own
document now has that overlay as its Player: when the page is not framed,
carries no `window.__OG_GAME` and was told its own origin as the Stage host
(`window.__OG_STAGE_HOST__ === location.origin`, the og-probe wire's own
test, now one helper, `shell/wire.ts` `stageInPage`), the shell's Player
bridge (`shell/player.ts`) speaks Player protocol 2 to its own window by
`postMessage`, hears only its own window at its own origin, and ignores its
own echo (a `hello` naming a `substrate` or listing no `capabilities`). The
overlay's answers count as the Player's: its `hello` with `started: true`
starts play with no gate of the shell's, og.platform's store and saves go
to it, and nothing of og.platform's reaches the Stage origin's
`localStorage` (the page's memory answers what the overlay does not); no
service worker is registered there. Framed pages and `__OG_GAME` pages are
unchanged, and any other top-level page still plays alone on the browser's
storage. A `result`'s `time` now passes through the bridge from every
transport (framed, HTTP, own window), so `og.platform.info().clock` reads
`player` as soon as a Player stamps its answers. Details:
`docs/web-export.md` §Player protocol 2; what the overlay must do to answer
is ask 10 in `docs/platform-asks.md`.

Proof: `tests/platform-stage.test.ts` (6 tests against a stand-in overlay
answering in the same window), `tests/web/player.ts` in Chrome (65 checks,
22 of them new: the Stage build with an in-document stand-in overlay at
desktop and phone sizes, and a silent overlay that never answers). Each
fails when its code is broken: without the echo filter, the `time`
pass-through, the source and origin check, the `self` peer, the `__OG_GAME`
precedence or the page's memory store. Transcripts, screenshots and
receipts: `packages/engine/tests/evidence/P2/shell-self/`.

## Status 2026-10-19, phase 2, tooling round

Branch `p2-tooling`; it closes owed rows 22 and 34 and records row 35's
stop (the table below says which commits). Evidence:
`tests/evidence/P2/tooling/` (`web-gfx-webgpu.json`, `web-gfx-webgl2.json`,
`m4-m5-tooling.md`, `m4-gltf-reload.json`, `m5-sweep-8-frames.json`,
`wasm-build-relink.md`, `break-and-restore.md`, before and after frames).

- **og-tests' gfx scenes in the browser** (22): `crates/og-tests-web`
  renders every scene of og-tests on an OffscreenCanvas in headless
  Chrome, on WebGPU and on WebGL2, and `tests/web/reference-gfx.ts`
  compares each with its tier's reference or metal's under og-tests' RMSE
  rule. All 47 applicable scenes pass on each tier (31 s and 52 s on the
  Apple M4 Pro, ANGLE Metal). WebGL2 has references of its own for `msaa`
  (only that tier renders it), `showcase` and `overlay` (GPU particles on
  the CPU path) and `water-3d` (cubemap reflection); `taa` does not render
  there. The web-reference step runs both tiers.
- **Browser frame times** (22): the seven perf scenes per tier beside
  metal's in `crates/og-tests/perf/<tier>-chrome-apple-m4-pro/`; WebGPU
  against WebGL2, ms per frame: showcase 2.15 / 2.58, spheres-20k 6.49 /
  6.89, sprites-20k 1.11 / 1.23, gltf-reference 1.38 / 1.64 (taken at load
  averages 50 to 191).
- **M4, glTF edit to reload** (34): through `og dev` on the scaffolded
  template, eight edits of the dropship's hull colour reload in 90 to
  113 ms (og dev's figure; 145 to 251 ms from the write). The first run met
  the time but not the change: the reload kept the models already loaded.
  Fixed in og-runtime (changed cooked glTF loads again into its asset
  index) and guarded by `packages/engine/tests/reload-asset.test.ts`.
- **M5, timed sweep** (34): `og probe sweep` over the trickster's
  `Spin.speed`, 8 values of 30 ticks, gives 8 distinct 1280×720 frames in
  298 to 559 ms round trip (target 1 s).
- **wasm-build relink**: the publish variant builds in its own target dir,
  so a rerun with nothing changed costs 0.4 and 0.5 s instead of 16.7 and
  12.2 s (cargo 1.99 keeps one fingerprint per unit whatever its features).
- **wgpu and naga 31** (35): not on crates.io; the newest is 30.0.1
  (2026-08-22), which the engine already uses. The row stays open.

Gate: `OG_GATE_JOBS=2 scripts/check.sh --all` on dbf3ae1 (this commit adds
documents only), with other agents' builds on the machine (load 18 to 48):
**PASSED, 9 of 9 steps in 115 s**: fmt 2 s, clippy 0 s, wasm-build 1 s,
test 67 s, wasm-opt 62 s, size-gate 45 s, drift 1 s, bun-test 20 s,
determinism 25 s. The run before it failed bun-test once: `i18n.test.ts`'s
`beforeAll` cook passed bun's 5 s hook timeout at load 40 and passed on the
rerun. `scripts/check.sh web-reference`'s new steps pass on both tiers
(above). The publish wasm is **3,029,921 B brotli, 72.2 %** of the budget
(the render round's merge recorded 3,028,729 B, +1,192 B: the model
refresh).

Left out: the wgpu 31 upgrade (unpublished); `og dev` killed by a signal
from outside leaves its bun child running (a clean exit and Ctrl-C stop
it; the reload test kills both); og-tests' `og-gfx` capture binary stays
native (it reads scenes from disk; the browser harness replaces it there);
browser frame times were taken with other agents' gates on the machine, so
they are upper bounds.

## Status 2026-10-19, player-page round

The engine's page is the play page itself (design §4): `og build` writes
`<meta name="og-player" content="frame page">` and og-build.json's
`player { protocol: 2, transports: [frame, page] }` into every build, and a
top-level page the platform published (`window.__OG_GAME`) speaks Player
protocol 2 to its play origin over same-origin HTTP (`shell/transport.ts`:
`/__player/v2/<game>/<release>/{hello,call,report}`, the hand-off code from
the fragment redeemed in the hello's body, the Player's credential keys and
outbox, session reports from the person's start), as origingame
`docs/player-protocol.md` §The top-level engine page specifies. Framed pages
keep the `postMessage` path unchanged, so a release built now plays under
today's Player too. At the top level the shell draws what the Player page
drew: the cover's backdrop behind the boot splash and Back (with its own
gate, `100dvh`, the safe areas, one audio unlock, fullscreen on the gesture
and the PWA hint, which it already had). The og-probe wire also answers the
page's own window when told its own origin, for the Stage overlay the
platform injects into a top-level Stage build. Details:
`docs/web-export.md` §The top-level page; the platform's side is ask 9 in
`docs/platform-asks.md`.

Proof: `tests/platform-page.test.ts` (6 tests against a stand-in play
origin), `tests/web/player.ts` (37 checks in Chrome: the framed Player, then
the top-level page at desktop and phone sizes), `tests/web/wire.ts` (125,
with the in-page host; its Tune checks moved off `Tuning.gravity`, which
the runner dropped in E7), `tests/web/export.ts` (40). Transcripts,
screenshots and receipts: `tests/evidence/P2/player-page/`.

What the platform's pin round must do: move
`agent-server/scripts/e2b/origin-engine.json` to this round's merge,
rebuild the layout and remount the knowledge (`og-platform-sdk` changed),
and have the platform's player-page work live first: the
`/__player/v2/` routes on game hosts, the game-host ingress admitting `POST`
with `Authorization` there, Play's 307 to a `page` release's entry, and the
Stage page's in-document mode. Until then a release built from this shell
is framed exactly as before.

Owed to that pin round from this side: the platform's Stage page now
answers a framed build's Player protocol as a stand-in Player (play
started, saves in its own memory), but a top-level Stage build has no
parent, so this shell plays it alone, draws its own gate and keeps
`og.platform` in the Stage origin's browser storage, which every Game's
builds share. Before the pin moves, the shell must also speak protocol 2
to its own window when `window.__OG_STAGE_HOST__` is its own origin (as
the og-probe wire already does), and the Stage overlay answer it there.

## Status 2026-10-19, Phase 2, core round

Branch `p2-core`, 11d6820..71dabed. It closes the engine defects and gaps
the archetypes round (below) found. Each public addition went through the
IDL in its own commit, with its skill text and the generated wiki.

- **og.Rng from reads, loud undeclared access** (d2bd748, 71dabed): a fork
  reads the seed and the label and moves nothing, so a system with og.Rng
  in `reads` forks the same streams as one that writes it, where it used to
  answer NaN. The reproduction is `packages/engine/tests/fixtures/rng-readonly`.
  An undeclared resource or event read or write throws in og run and
  og test, naming the system and the record. A publish build refuses it
  and logs it once per system and access through the probe's diagnostics.
  og-core states both rules.
- **The host's clock** (fc97734): `og.platform.now`, a `time` on every
  og.platform answer, and `og.platform.info(world).clock` (`player`,
  `browser`, `system`, `fixed` or `none`). These are host input between ticks, so a replay
  records them. Natively the clock is the system's; `og run --clock` and
  `og test` fix it. On the web the Worker uses the browser's clock until
  the Player's hello and answers carry `time` (`docs/platform-asks.md`,
  open ask 9). An answer that already carries a `time` is taken as is.
- **A kept store** (cd1f0d3): an `og test` spec with `store: "keep"` runs
  each `then` step as a new process over one local platform store, on a
  host clock the spec moves. `relaunch:<seconds>` stays for one-process
  tests.
- **Input `at`** (56fc770): key and pointer events carry when they happened
  within the tick that reads them (-1 to 1 of a tick). The time is handed
  in with the event, so the world stays deterministic on both hosts.
- **og.audio.outputLatency** (094df16): the time from world time to the
  speaker. On the web it is the AudioContext's output and base latency
  plus the ring's prime. Natively it is the device's latency (cpal's playback
  instant less its callback instant) and the mixer's buffer, and headless it
  is 0.
- **AudioSource.startOffset** (f3fb31f): a clip can start part-way, to the
  sample (og-audio's `mix.rs` asserts the PCM).
- **Archetypes** (d2bd748, 4868361, b6d889a): their workarounds are gone.
  - The roguelike's Foes and Loot declare og.Rng in their reads.
  - The idle game stamps its save with og.platform.now and credits the
    time away from the store answer's `time`. `tests/away.og.ts`
    relaunches two hours later in a second process.
  - The rhythm game judges a press at its `at`, against what was heard
    (the output latency, plus the player's own offset), and resumes from
    the beat it paused on (`tests/timing.og.ts`, `tests/pause.og.ts`).
  - Each README's Known limits keeps only what is still open.
- **Playtests**: every template's `og test` passes.

  | Template | Suites |
  |---|---|
  | game | 2 |
  | ui-kit | 25 |
  | fx-showcase | 10 |
  | idle | 10 |
  | match3 | 6 |
  | rhythm | 11 |
  | roguelike | 9 |
  | runner | 8 |
  | topdown | 7 |
  | tower-defense | 9 |

- **og polish** (`tests/evidence/P2/core/polish/`):

  | Archetype | Taste | Feel |
  |---|---|---|
  | idle | 97 | 74 |
  | roguelike | 99 | 80 |
  | rhythm | 99 | 74 |

  Taste is unchanged from the archetypes round, and every feel row but
  frame time reads 92 to 100. The frame-time row is 0 because these runs
  used the debug og at load 28. The archetypes round's runs used a release
  og. Run og polish on the rhythm game from its own directory: its tests'
  chart lookup falls back to the working directory.
- **Gate**: `OG_GATE_JOBS=2 scripts/check.sh --all` at 71dabed,
  2026-10-19, at load 16–29, **PASSED, 9 of 9 steps in 147 s**. The steps
  took fmt 1 s, clippy 1 s, wasm-build 9 s, test 105 s, wasm-opt 104 s,
  size-gate 31 s, drift 1 s, bun-test 25 s and determinism 2 s. Builds were
  warm from the path-aware run before it.

  Two earlier runs did not count. The first `--all` stopped at clippy on
  a `manual_contains` lint in the undeclared-access check, which 71dabed
  fixes. The path-aware run before this gate timed out e5's every-feature
  hash test at 5 s under twelve parallel files. That test passes alone in
  0.66 s and in this gate.

  Path-aware runs after the gate, with only this section and the evidence
  changed, hit two timing bounds at load 28–36. e5's hash test timed out
  again. og-audio's `one_second_of_a_busy_mix_renders_in_bounded_time` took
  1,621.9 ms against its 1,000 ms debug bound. Both pass when run alone,
  and both tests predate this round. The round's mixer change touches only
  a voice that has not started yet.
- **Wasm**: the publish wasm is 2,982,720 B brotli, 71 % of the budget,
  +4,259 B over 2,978,461 B.
- **Merge of main** (44df5d7: the animation and physics round and the
  render round): only the generated files (bindings.json and the wiki,
  regenerated by `og api gen`) and this document conflicted. api.ogt,
  shell.ts and worker.ts merged cleanly, keeping both the input `at` and
  the compressed-texture Worker. `OG_GATE_JOBS=2 scripts/check.sh --all`
  on the merge, 2026-10-19, at load 38–194: **PASSED, 9 of 9 steps in
  423 s**. The steps took fmt 3 s, clippy 1 s, wasm-build 27 s, test 58 s,
  wasm-opt 69 s, size-gate 53 s, drift 2 s, bun-test 126 s and determinism
  416 s. Before it, the path-aware run at load 148 failed the audio-mix
  timing bound again (1,606.1 ms). The publish wasm is 3,033,764 B brotli,
  72 %, +5,035 B over the render round's merge (3,028,729 B).
- **Merge of main** (7438978: the UI round): again only the generated
  bindings and wiki (regenerated) and this document conflicted.
  `OG_GATE_JOBS=2 scripts/check.sh --all` on the merge **PASSED, 9 of 9
  steps in 246 s**. The steps took fmt 3 s, clippy 1 s, wasm-build 25 s,
  test 166 s, wasm-opt 150 s, size-gate 69 s, drift 2 s, bun-test 41 s and
  determinism 14 s. The publish wasm is 3,061,522 B brotli, 72 %.

  The archetypes' `og test` fails on main at 7438978 alone (built from
  `git archive`), with the same failures as on this merge, so it is not the
  merge:
  - Every `frames` suite misses its Metal references (the UI round's
    restyle, for example tower-defense's title at RMSE 0.033).
  - Tower-defense's title takes no tap or pad press before about 0.35 s,
    so `lose`, `pad`, `pause` and `replay`, which press Play at 0.3 to
    0.4 s, never start a round. A tap at 0.4 s starts one.
- **Evidence**: `tests/evidence/P2/core/break-and-restore.md` breaks every
  new check once.
- **Not done**:
  - The TS types do not restrict `world.resource` to the declared set.
    IDL callbacks carry no generics, so that needs an og-idl generator
    change; the runtime check is the guard meanwhile.
  - A gamepad press carries no `at`, because the shell polls pads once a
    display frame.
  - The native device latency is not tested: the gate has no audio
    device and builds without `native-audio`. The web value is tested.
  - The Player's own clock waits on ask 9.

## Status 2026-10-19, Phase 2, UI round

Branch `p2-ui` from main at 2dae5fa (`phase1-e7` and the gate-speed
work). It closes owed rows 15–19, 21, 33, 41, 42, 45 and 46 (the table
below names each commit) and nothing else:

- **Rich text** (row 33, 6139efe): `og.ui.richText` lays out markup from
  the string table: `[b]`, `[color=]`, `[size=]`, `[font=]`, `[icon=]`,
  `[link=]` (a child element keys, pads and touch reach) and per-glyph
  `[wave]`, `[shake]`, `[pulse]`; shaped and right-to-left pieces in visual
  order. The kit's confirm and alert messages carry it in five locales.
- **Accessibility tree** (row 33, 38694c6): each element's role, name,
  value and states in `dump_ui` and `og.ui.a11y()`; focus order is the
  tree's.
- **Glass** (row 33, 42d7294): og-render blurs the frame under overlay
  panels through a six-level pyramid; skin line `glass <part> <blur> <fill>
  [tint]`; the scifi pack's dialogs and panels are glass (+0.83 ms a frame
  for four large panels, `tests/evidence/P2/ui/glass-cost.md`).
- **Scrolling both ways** (row 33, 79efd08): `ScrollDesc.axes`, the handoff
  to the outer scroller, a fling, a draggable scrollbar.
- **Text**: per-glyph tracking from the ramp and a distance-field glow
  (17, 18, 0df11b1); the kit's small type at 9 px (42, f9bad51).
- **Input and feel**: UI sounds in their input's tick (15, 59ce0b4),
  spring hover and press (16, 814fdad), focus moves heard (41, ca81fb8),
  pads 1–3 with focus of their own (21, ca179c0), the shop's refusal
  shaken and heard (45, d00f20c), the kit's hits felt (ffc0eb3), runner's
  held press heard (46, 54b59df).
- **`og skin pack`** (19, f4c236b): nine-patch PNGs to a skin's atlas and
  lines.

Row 43's engine side (the platform's danger tokens in og-ui's tokens and
a button contrast test) is cf932f8 and 5d982db from `p2-anim-physics`;
this round does not repeat it, and the merge of main keeps both.

### Gate

`OG_GATE_JOBS=2 scripts/check.sh --all` on f4c236b (this commit adds
documents and polish evidence only), 2026-10-19, Apple M4 Pro with four
other rounds building on the machine (load average 30): 169 s, fmt 2 s,
clippy 2 s, wasm-build 14 s, test 107 s, wasm-opt 108 s, drift 2 s,
determinism 2 s; bun-test failed on one test,
`tests/e5.test.ts`'s every-feature hash at 5,265 ms against bun's 5,000 ms
timeout under twelve parallel workers (the file takes 756 ms alone), and
cancelled size-gate. `--only bun-test size-gate` then passed both (19 s;
72 engine tests in 13.8 s). `og test templates/ui-kit` passes 29 suites and
runner 8; references this round re-recorded say why in their commits, with
before and after images in `tests/evidence/P2/ui/rerecord/`, and
`tests/evidence/P2/ui/break-and-restore.md` has how each new test was broken
once.

### Wasm

**2,974,147 B brotli, 70.9 % of the 4,194,304 B budget**, from 2,943,180 B
(70.2 %): +30,967 B for rich text, the accessibility tree, glass (its
shaders and pyramid), two-axis scrolling, extra pads, springs and the glow.

After merging main (44df5d7: terrain and water, the archetypes, the
animation and physics round with og-ui's danger tokens, the render round),
19 kit references and og-ui's sample-screen, changed on both sides, are
re-recorded with this round's text and main's danger ink together (0.04 to
0.50 % outliers against main's, 3.2 and 6.6 % for the scifi sheets' glass;
RMSE 0.006 to 0.045; before and after in
`tests/evidence/P2/ui/rerecord/merge-main-*.png`), and the IDL's outputs
regenerated. `OG_GATE_JOBS=2 scripts/check.sh` **PASSED, 9 of 9 in 965 s**
at load 40–70, then `--all` **PASSED, 9 of 9 steps in 111 s**: fmt 2 s,
clippy 1 s, wasm-build 10 s, test 82 s, wasm-opt 74 s, size-gate 25 s,
drift 2 s, bun-test 18 s, determinism 8 s. `og test` passes ui-kit (29
suites) and runner (8, now `templates/archetypes/runner`). The publish wasm
is **3,057,601 B brotli, 72.9 %** for all the rounds together.

### Polish

`og polish` before (E7) and after (`tests/evidence/P2/ui/polish/`), on
metal with other rounds' builds on the machine:

| Template | Taste | Feel | Findings |
|---|---|---|---|
| ui-kit | 90 → 91 | 62 → 70 | hit feedback 0 → 100, sound on input 38 → 73, UI sounds 62 → 65, text size → 98; frame time 0 under load (E7 read it 0 too); contrast 84 is row 43's default primary and danger labels, until `p2-anim-physics` merges |
| runner | n/a | 99 → 95 | 0 (row 46's flow input closed; sound on input 100); frame time 80 under load, not a regression in the game |

### Left out

- A shell mirroring `og.ui.a11y()` into the DOM for screen readers: the
  tree is there to mirror; the web shell's mirror is the Player round's.
- Glass over other overlay content (it blurs the scene only) and bidi
  reordering inside one rich-text span (pieces are reordered, not runs
  within a piece).
- The touch HUD's action buttons sound on the sfx bus, not the ui bus, so
  ui-kit's UI sounds stays at 65; the sounds-off playtest is silent by
  intent.
- Packs whose fonts have small x-heights still measure 7–8 px on some
  labels; a pack's ramp line for base raises them.

### Fix on main (`p2-ui-fix`, from b2532cd)

Four archetypes failed `og test` on main after the round:
- idle, rhythm, roguelike and tower-defense in `frames`;
- tower-defense in lose, pad, pause and replay, whose title refused a tap on
  Play before 0.367 s.

The cause was og-ui's stagger. It kept each staggered control inert until its
own place in the cascade had played. The round's springs rest the hover at
exactly zero, so skinned buttons joined the cascade for the first time.

Now the cascade is only seen. A control takes input once the panel's own
entrance, still inert, has played, and the title takes a tap from 0.267 s as
before the round. New test: `skin.rs`
`a_staggered_panels_controls_take_presses_once_the_panel_has_arrived`, broken
twice (break-and-restore.md).

Nine archetype frames were re-recorded for this round's text: the tokens'
tracking on display titles and headings, and the distance-field glow of idle's
and rhythm's `textGlow`. Each reason and the three-build attribution are in
`tests/evidence/P2/ui/archetype-frames.md`, with before and after images in
`rerecord/archetype-*.png`.

`og test` passes:
- the seven archetypes: idle 10, rhythm 11, roguelike 9, tower-defense 9,
  match3 6, runner 8 and topdown 7 suites;
- fx-showcase 18, game 2 and ui-kit 29 suites.

`OG_GATE_JOBS=2 scripts/check.sh` **PASSED, 9 of 9 in 1059 s** at load 30–50.
web-reference was not run, because no og-ui reference changed. The publish
wasm is 3,061,786 B brotli, 72 %.

### Fix on main (`p2-ui-webgl`, from 7a5e320)

The tooling agent's web-reference run found two failures on main.

**Every WebGL2 renderer panicked, from 42d7294 (the overlay glass).** wgpu's GL
backend translates each shader to GLSL ES 3.0 when it creates a pipeline.
`fs_glass` asked its pyramid for its level count with `textureNumLevels`, which
GLSL ES 3.0 lacks (`Features(TEXTURE_LEVELS)`), so the overlay pipeline could
not be created. fx-showcase on WebGL2 reported "engine did not start:
unreachable", and every graphics scene failed there. The design's §8 reduced
tier has no texture queries.

Now the glass pass hands the shader the level count. The pyramid's own sprite
params, `[0, 0, 0, LEVELS]`, are bound in place of the zero params (`glass.rs`
`pyramid_params`), and `fs_glass` reads `sprite_params.msdf.w`. The level of
detail is the same number as before, so the WebGPU and native images are
unchanged. New test: `crates/og-render/tests/webgl2.rs` runs wgpu's own naga
translation (WGSL to GLSL ES 300, WebGL flags) over every entry point of
`overlay.wgsl` and `glass.wgsl`, so a texture query or other GLSL ES 3.0 gap
fails natively without a browser. Broken once (break-and-restore.md).

**fx-showcase on WebGPU failed every station at b2532cd, and still did on
7a5e320.** Its start threw `models/trickster.glb#tex0: not transcoded`, so the
scene was never built (post 99.5 % outliers, RMSE 0.75). It is not the UI round
or the texture streaming round. Since d2bd748 (undeclared access is loud), the
TypeScript `Engine` constructor learns the build from the probe's `hello`, and
every probe call ran a tick boundary. With `deferStart`, the tick boundary is
where GameDesc.start runs. So start ran inside `createEngine`, before the Worker
attached its renderer and transcoded the bundle's Basis textures, and `og run`
with a renderer started before its offscreen surface was attached. Now `hello`,
which only says who is speaking, runs no tick boundary (`engine.rs`
`answer_call`). New test: `packages/engine/tests/runtime.test.ts`, with
deferStart, start waits for the first tick boundary and `hello` is not one.
Written before the fix and failing on it (break-and-restore.md).

`scripts/check.sh --only web-reference` **PASSED in 65 s**:
- fx-showcase 5 of 5 stations on WebGPU and 5 of 5 on WebGL2;
- E5 17 of 17 on each tier.

No reference was re-recorded.

`OG_GATE_JOBS=2 scripts/check.sh` passed fmt, clippy, wasm-build, test, wasm-opt,
size-gate and drift. bun-test timed out once, at load 140, in web-export's
`the same inputs give the same bytes`; that test passes alone in 0.2 s. A
rerun, `scripts/check.sh --only bun-test`, **PASSED** with the engine package's
108 tests. The publish wasm is 3,151,777 B brotli, 75 %.

## Status 2026-10-19, phase 2, og-ui tiers round

Branch `p2-ui-tiers` from 44d12a8 (evidence: `tests/evidence/P2/ui-tiers/`,
`NOTES.md` there indexes it). Owed row 20 is done: every og-ui reference
frame exists on vulkan-cpu, and the browser tiers are checked against
metal (their recorded copies were later dropped as duplicates).

- **vulkan-cpu** (2131b15): all 74 frames (og-ui's four scenes, 22 desktop
  and 21 phone kit screens, 15 locale screens, 12 skin sheets), recorded
  with `OG_UI_CAPTURE=1` on lavapipe in the og-lavapipe image with
  zigbuild Linux binaries. The scenes move into
  `crates/og-ui/tests/support/scenes.rs` so the browser can build them too.
- **Browser, og-ui's scenes** (de92f08): og-tests-web includes that file
  and draws `og-ui/<name>`; `reference-gfx.ts` judges them against
  `crates/og-ui/references/<tier>/`, else metal. Both tiers match metal
  exactly except widget-set (RMSE 0.00046, as the native metal run).
- **Browser, the kit** (8f1d3af): reference.ts takes `--size`,
  `--references` and `--sheet`; `tests/web/reference-ui.ts` runs ui-kit's
  16 reference playtests through it (the full set, 70 frames; nothing left
  out, a tier takes 100 to 140 s) and `scripts/check.sh web-reference`
  runs it on both tiers.
- **Against metal** every frame is within the rule on every tier. Eleven
  frames have outliers, the same on vulkan-cpu, webgpu and webgl2, at most
  0.038 % (RMSE 0.0089, `kit-desktop-settings`): the large titles' edges
  and the gallery's scrollbar thumb, a fraction of a pixel of coverage.
  The three non-metal tiers agree there with no outliers between them, so
  the difference is metal's. Every other frame is within RMSE 0.0040.
- **Break and restore**: a mirrored vulkan-cpu sample-screen fails og-ui's
  gfx test on lavapipe; a wrong webgl2 locale frame, a wrong webgpu skin
  sheet and a mirrored webgpu glass fail the browser runners; restored,
  all pass.
- **Afterwards**: the webgpu and webgl2 copies of all 74 frames were
  dropped as duplicates of metal (none differs by the tier's design,
  docs/ci.md §Browser reference tiers); both browser tiers are judged
  against `crates/og-ui/references/metal/`, with the figures above as the
  record of what they show against it.

### Gate

`OG_GATE_JOBS=2 scripts/check.sh` on 8f1d3af (the docs and evidence of
the next commit were uncommitted and touch no check): fmt, clippy (75 s),
wasm-build, wasm-opt (212 s), size-gate (3,176,490 B brotli, 75 % of
4,194,304; no crate in the wasm build changed this round), drift (186 s)
and test (722 s) passed. bun-test failed after 506 s on five timeouts
and no assertion, while a lavapipe container ran beside it (104 pass):
render.test.ts "trails, beams and ribbons" and "material and sprite
effects" at 5 s and web-export.test.ts "the same inputs give the same
bytes" at 120 s (the known load flakes), plus asset-stream.test.ts
"what is resident never changes the simulation" at 5 s and "og build
--publish ships each group as its pack" at 120 s (not on the known list;
this round changes nothing they run). This stopped determinism. Neither the `--only bun-test`
rerun nor determinism nor `scripts/check.sh --only web-reference` ran:
the coordinator now runs the gate on main after the merge (done at
50eb7f7, top of this file: all passed, kit frames 16 of 16 on both tiers). The browser
kit and scene frames were checked directly instead: `reference-ui.ts`
and `reference-gfx.ts` without `--record` reproduce every frame on
webgpu and webgl2 at RMSE 0.
On lavapipe, gfx 5 of 5 and kit_gfx 1 of 1 pass at RMSE 0 against
`vulkan-cpu`; `og test templates/ui-kit` without capture passed 17 of 29
suites (frames-desktop, frames-phone, locales, skin-cartoon,
skin-cartoon-phone, skin-fantasy-phone and the eleven playtests without
frames) before it was stopped for time. skin-fantasy and the neon, paper,
pixel and scifi skin suites (nine), plus sounds, touch and tutorial,
passed only in the recording run, so their comparison against the
recorded `vulkan-cpu` sheets is unverified.

## Status 2026-10-19, phase 2, 3D SPH round

Branch `p2-sph` (evidence: `tests/evidence/P2/sph/`, `NOTES.md` there
indexes it). Owed row 29 is done.

- **The simulation** (6dcdc98, 87ea4bd): `og.Fluid3D`, a 3D particle fluid
  by double density relaxation (Clavet et al. 2005), the 2D fluid's model,
  rather than position-based fluids: one neighbour search and two passes a
  substep instead of PBF's solver iterations, stable at 60 Hz with two
  substeps, and groups (four fluids at most, layering by `density`,
  blending by `mixing`), emitters (`emitRate` up to `maxCount`) and sink
  boxes carry over. Neighbours come from a spatial hash in a fixed order;
  particles collide with the 3D colliders and the terrain's height field
  and stay in optional `bounds`. All of it is og.physics state (snapshots,
  restore, the world hash). CPU per 60 Hz tick, native and wasm32 alike:
  about 0.9 ms for 2,000 particles at one substep, 1.8 ms at the default
  two, 4 ms for 4,000 and 10 ms for 8,000, about double where they pile
  deep.
- **Determinism** (a513e20): the beyond-rigid-bodies scenario pours two 3D
  fluids of one group into a terrain bowl through an emitter and a sink;
  host debug and release and wasm32 scalar and simd128 agree (physics_hash
  8301574762329360262).
- **The public API** (71257f0): api.ogt's physics module declares
  `og.Fluid3D` and `og.physics.fluid3dParticles`, regenerated with `og api
  gen` into the bindings, schema, d.ts, ABI, glue and wiki.
- **The surface** (200b6fe, 7fc64e7, 2f9e908, 792aa2b): og-render draws
  every visible 3D fluid as one screen-space surface on all four tiers (no
  storage buffer, no compute): sphere imposters into a depth and a
  thickness target, a separable bilateral smoothing, and a composite in
  the main pass that refracts and absorbs the scene behind, lights the
  body with the sun, its cascades, the image-based light and the clustered
  local lights, reflects the environment and writes its depth. About
  1.1 ms of GPU on the M4 Pro for a fluid covering a third of a 720p frame,
  whatever the particle count. Marching cubes was neither built nor
  measured; the case against it (a density grid and a mesh rebuilt every
  frame, on the CPU for WebGL2) is reasoning only. Shared render-graph
  changes: `post.rs` gains `FramePass::Fluid3d` and its five transients
  (`fluid.depth`, `fluid.zbuf`, `fluid.thick`, `fluid.blur`,
  `fluid.smooth`), read by the main pass; `lib.rs` forces the depth
  prepass and the split main pass while a fluid has particles.
- **fx-showcase** station 23, fluids3d (332ec84): a blue pool and a heavy
  orange fluid poured into it by an emitter, sinking under it, with a
  playtest of 12 checks and references on metal, vulkan-cpu (1974892),
  webgpu and webgl2 (829143e), all within the RMSE rule against metal.
- **Docs** (7e63835): the og-game skill and the wiki teach og.Fluid3D and
  its CPU and GPU cost; og-physics' and og-render's READMEs gain their
  phase 2 sections.

### Gate and wasm

`OG_GATE_JOBS=2 scripts/check.sh --all` on 8658977 (this section and
the NOTES' last edit uncommitted, documentation only), 2026-10-19, with
other agents' gates on the machine (load average 28 to 96): **PASSED, 9 of
9 steps in 412 s**: fmt 1 s, clippy 0 s, drift 2 s, wasm-build 27 s,
wasm-opt 123 s, test 125 s, size-gate 47 s, bun-test 27 s, determinism
407 s (every build agrees, physics_hash 8301574762329360262). The run
before it (fmt 4 s, clippy 65 s, wasm-build 23 s, wasm-opt 98 s, size-gate
64 s, drift 32 s, test 360 s) failed at bun-test at load 96 on two timeouts
in code this round does not touch: i18n.test.ts's beforeAll (`og cook` of
ui-kit, 5 s) and web-export.test.ts (120 s); alone they pass in 4 s, and
the rerun passed. The publish wasm is **3,052,777 B brotli, 72.8 % of the
4,194,304 B budget**, 24,048 B (0.57 points) more than the 3,028,729 B
before this round. Earlier, `scripts/check.sh web-reference` passed in
168 s (18 of 18 stations on webgpu and webgl2, fluids3d included).

## Status 2026-10-19, phase 2, render round

Branch `p2-render`; it closes owed rows 23, 28, 30, 31 and 32 (the table
below says which commits). Evidence: `tests/evidence/P2/render/`
(`forward-plus.md`, `materials-and-hooks.md`, `compressed-textures.md`,
`break-and-restore.md`, before and after frames).

- **Forward+** (30): point and spot lights are clustered on the CPU into 16
  x 9 tiles by 24 logarithmic slices (10-40 us for 256 lights), storage
  buffers on the full tier and 16 KiB uniform tables on WebGL2; up to
  `MAX_LIGHTS` 256 in view, 64 per cluster, the brightest at the camera kept
  past either cap. Frame time, 8-light loop against clustered: metal
  `lights-256` 0.24 ms (8 lights drawn) against 0.58 ms (243 drawn; drawing
  them all per pixel costs 5.8 ms), `point-spot-lights` 0.42-0.66 against
  0.46-0.56 ms; Chrome's lights station 1.0-1.2 against 1.7-2.0 ms on
  WebGPU and 1.8-1.9 against 2.2-2.4 ms on WebGL2, its fx station unchanged.
  The phase 1 lighting line, the one not met as written, is met.
- **The moon** (28): a disc in the procedural sky with its phase, size,
  intensity, colour and direction (`og.Sky`, `og.DayNight`).
- **Materials** (31): normals by the cofactor matrix under non-uniform
  scale, vertex tangents in glTF's frame (file, cook or generated; normal
  maps had read inverted), clearcoat, and the `materials` gfx scene.
- **Material shaders** (23): on skinned meshes, and in the shadow passes and
  the depth prepass; `og.render.setMaterialShader` for a loaded model's
  materials.
- **Compressed textures** (32): cooked textures reach the GPU as BC7/BC5,
  ASTC 4x4 or ETC2/EAC with their mips (the template's: 2.5 MB instead of
  10.1 MB, load time unchanged); normal maps now cook with y in alpha.
- **vulkan-cpu**: the probe, `og.render.capabilities`, `og polish` and the
  reference directories name a CPU Vulkan adapter (lavapipe, llvmpipe,
  SwiftShader) `vulkan-cpu`, a GPU one `vulkan`.

Gate: `OG_GATE_JOBS=2 scripts/check.sh --all` on 872b002 (this commit adds
documents only), with other agents' builds on the machine (load 75):
**PASSED, 9 of 9 steps in 173 s**: fmt 5 s, clippy 14 s, wasm-build 37 s,
test 107 s, wasm-opt 102 s, size-gate 27 s, drift 9 s, bun-test 23 s,
determinism 3 s. `scripts/check.sh web-reference` (Chrome for Testing 150)
matches fx-showcase's five frames and twelve stations on WebGPU and WebGL2;
`og test` passes fx-showcase (13 suites) on metal and on lavapipe and the
game template (2). The publish wasm is **2,958,860 B brotli, 70.5 %** of the
budget (2,943,180 B before the round, +15,680 B).

After merging main (9c9d6d3: terrain and water, the archetypes, animation
and physics), terrain's lit shader and the water reflection pass use the
clusters too (the reflection builds its own over the main view's kept
lights), and the render stations moved to `src/render.ts` as 19-22 at x
2100-2300; the moon frame was re-recorded on every tier for the move
(`fx-showcase-moon-relocated-before-after.png`). `OG_GATE_JOBS=2
scripts/check.sh --all` on the merge **PASSED, 9 of 9 steps in 291 s**:
fmt 4 s, clippy 2 s, wasm-build 19 s, test 225 s, wasm-opt 202 s,
size-gate 42 s, drift 10 s, bun-test 35 s, determinism 28 s.
`scripts/check.sh web-reference` matches all 17 stations on WebGPU and
WebGL2; `og test` passes fx-showcase (18 suites) on metal and lavapipe.
The publish wasm is **3,028,729 B brotli, 72.2 %** (main recorded
3,013,367 B, +15,362 B).

Left out: clearcoat maps (WebGL2's skinned lit pipeline has no texture slot
left; delivered in P4 with the other physical lobes, crates/og-render's
README §Physical materials), a compute cluster build (the CPU build costs tens of microseconds),
upload timings in the browser (the Worker's transcoding is timed, the GPU
upload natively), and ASTC and ETC2 frames off Apple silicon (lavapipe and
desktop Chrome take BC).

## Phase 2, terrain and water round (2026-10-19)

Branch `p2-terrain-water`, 0ee78f1..HEAD on 2dae5fa. Closes owed row 24.

- **Terrain** (0ee78f1, 290f4a4, 9a9d6da, 5be9281): a `type terrain`
  .ogt (a 16-bit grey PNG or an EXR heightmap, an RGBA control map, up to
  four layers with slope and altitude rules) cooked into zlib'd 16-bit
  tiles of 128 samples, deterministically; drawn by og-render as a
  geometry clipmap (nested `ring` × `ring` levels snapped to twice their
  spacing, the finer level's hole cut from one index buffer, CDLOD's morph
  at the edges: no cracks at six camera positions, fewer levels from afar),
  normals built on the GPU, layers planar or triplanar weighted by control
  map × slope × altitude and sharpened by their heights, casting and
  receiving cascaded and local shadows under the fog; a rapier height
  field (SHAPE_HEIGHTFIELD, split along the renderer's diagonal) that the
  character controller walks on; a navmesh `.ogt`'s `terrain` entry bakes
  its walkable slopes. IDL module `terrain`: og.Terrain, og.terrain.load,
  sample, bounds, edit (raise, lower, flatten, smooth, paint, seen by the
  renderer, physics and sample), layer, setLayer; og-game §11 and
  `wiki/terrain.md`.
- **Water** (row 24; 0ee78f1, 5be9281): og.Water's planarReflection (the
  camera mirrored in the surface, an oblique near plane at the water,
  0.25 to 1 of the resolution, cheaper than the screen-space march it
  replaces), shadows (0 to 1), caustics on what lies below, and the
  underwater view (the water's fog, Snell's window from below).
- **Scatter** (b35a73b, fd4456d): og.TerrainScatter grows instanced copies
  of a mesh over a layer near the camera, fixed to a grid on the terrain,
  shrinking away with distance; og-tests' terrain-grass scene.
- **fx-showcase** (15ff230): the `terrain` station, a 256 m valley with a
  path painted in the control map, a lake with a planar reflection of the
  mountain, and a character walking up a knoll, on all four tiers.
- **References**: og-tests terrain-near, -mid, -far, -grass,
  water-planar, -shadows, -underwater on metal and vulkan-cpu (reproduced
  at RMSE 0; every earlier vulkan-cpu reference still passes); the
  fx-showcase station on metal, vulkan-cpu, webgpu and webgl2.
- **Costs and frame times** (`tests/evidence/P2/terrain-water/`): terrain
  +0.1 to +0.55 ms on the M4 Pro, its shadow passes +0.2 to +0.35 ms,
  water shadows and caustics within noise, the underwater view +0.4 ms,
  the 36,000-tuft grass +4.1 to +4.5 ms, all CPU submit. Every scene draws
  in under 2.5 ms on metal (grass 4.3 ms); the terrain station's median
  frame is 1.34 ms on Chrome WebGPU and 2.43 ms on WebGL2, under the
  8.3 ms desktop budget.
- **Gate**: `OG_GATE_JOBS=2 scripts/check.sh --all` at ad06e6e,
  2026-10-19, Apple M4 Pro shared with four other agents' work:
  **PASSED, 9 of 9 steps in 559 s** (9 min 19 s): fmt 3 s, clippy 14 s,
  wasm-build 184 s, test 219 s, wasm-opt 200 s, size-gate 61 s, drift 6 s,
  bun-test 150 s, determinism 2 s.
- **Wasm**: publish wasm 2,978,461 B brotli, 71.0 % of the budget
  (+35,281 B over 2,943,180). The terrain's load-time decoding lives in
  og-cook and miniz_oxide, both at opt-level z in `[profile.wasm]`.
- **Not done** at the round's end, since closed by the phase 3 streaming
  round below unless noted: terrains were decoded whole at load (now
  streamed tile by tile, bc5c5ea); a terrain lay along world x and z
  (now turned and scaled with its entity, 9941ffd, 6b9221b); edits were
  asset changes outside snapshots (now world state, 9024a12); scattered
  copies shrank instead of fading and were rebuilt on the CPU every frame
  (now dithered and placed once per cached chunk, 1ced372); lavapipe draws
  no shore foam on `terrain-far`'s 24 km single-quad sea (fixed by
  ec54c60, which draws large water in cells of at most 256 m:
  `tests/evidence/P3/streaming/foam/NOTES.md`).

## Status 2026-10-19, Phase 2, archetypes round

The four Discover archetypes still owed are built on the engine (branch
`p2-archetypes`): `templates/archetypes/tower-defense` (Path Guard),
`roguelike` (Lamp Dive), `idle` (Spark Orb) and `rhythm` (Three-Lane Beat).
Each is a complete small game with a title, pause, settings (volumes,
language, reduced motion; the rhythm's offset), an end card, keyboard,
gamepad and touch, safe areas and a phone layout, an og-ui HUD on a
recoloured style pack, en and zh-CN tables, synthesised sounds (CC0, by
its `tools/make-assets.py`, byte-for-byte deterministic, as is the cook),
Metal reference frames and a README naming the skill sections it
exercises. The create CLI scaffolds all seven archetypes
(`--archetype`), and the knowledge skills (og-ai, og-audio, og-feel,
og-game, og-i18n, og-platform-sdk) point at them.

| Archetype | Playtests (`og test`) | `og polish` taste / feel | Publish archive (zip) | The game's own content |
|---|---|---|---|---|
| tower-defense | 9 of 9 | 99 / 100 | 5,738,666 B | 1,386,864 B |
| roguelike | 9 of 9 | 99 / 100 | 5,620,824 B | 1,295,752 B |
| idle | 9 of 9 | 97 / 94 | 6,151,017 B | 1,803,759 B |
| rhythm | 10 of 10 | 99 / 99 | 6,039,807 B | 1,822,905 B |

The archive is `og build . --publish`'s `dist/` zipped, most of it the
engine's wasm and web runtime; the game's own content (`cooked/` and
`game.js`) is under the Discover prototypes' 2 MB asset budget in each.
The polish reports (`tests/evidence/P2/archetypes/<name>/`) were made with
og and the native addon built release from this checkout: on this machine
(load 15 to 70 from other agents' work) frame time's p95 follows the load,
not the games, whose median step and render are 1–3 ms
(`tests/evidence/P2/archetypes/frame-time.md`, with a per-tick series).
`tests/evidence/P2/archetypes/break-and-restore.md` breaks one check of
each.

Gate: `OG_GATE_JOBS=2 scripts/check.sh --all` on 3c5c7d4 with this
commit's documents and skill pointers, 2026-10-19, load 11–35: **PASSED,
9 of 9 steps in 99 s** (fmt 1 s, clippy 0 s, wasm-build 6 s, test 59 s,
wasm-opt 62 s, size-gate 28 s, drift 1 s, bun-test 22 s, determinism
1 s; the publish wasm 2,943,180 B brotli, 70 % of the budget). The four
archetypes' `og test` runs (9, 9, 9 and 10 suites) pass beside it.

Engine gaps the games work around, each in its README's Known limits:
og.ui's `settingsScreen`, `resultsScreen` and `idleHud` carry English
words and number formats of their own, so the settings, end cards and the
idle HUD are built from widgets; `og polish` reads og.ui's phone title
menu (a bottom sheet) as HUD. Closed by the Phase 2 core round (above):
og.platform's answers carry the host's time and og.platform.now reads it,
so the idle game credits the time away from the save's stamp (a second
`og test` process over a kept store is its test; `relaunch:<seconds>`
stays for tests in one process); og.audio.outputLatency and the input
events' `at` let the rhythm game judge a press when it happened and as it
was heard (the taps' calibration stays, as the player's own offset); an
AudioSource's `startOffset` starts a clip part-way to the sample, so a
resume carries on from the beat it paused on. The two engine defects met
on the way are fixed there too: a system with `og.Rng` in `reads` only
forks the same streams as one that writes it (the roguelike's Foes and
Loot declare it in their reads), and an undeclared resource or event access throws in og run
and og test and is refused and logged once per system in a publish build.

## Status 2026-10-19, phase 2, animation and physics round

Branch `p2-anim-physics` (evidence: `tests/evidence/P2/anim-physics/`,
`NOTES.md` there indexes it). Owed rows 25, 26 and 27 are done; 29 (3D SPH)
was not started and is left for the next round.

- **Animation retargeting** (aaea84c, 707b6b2): og-anim maps a humanoid's
  22 bones from its names (Mixamo, VRM, Unity, Rigify, Unreal, Blender
  detected; a `type rig` .ogt overrides the scheme or any bone) and
  retargets a clip onto another rig: rotations moved in model space with
  each bone's rest corrected, the hips' travel and height scaled by the
  legs' ratio (root motion kept), planted feet held on the ground by
  two-bone leg IK, lowering the hips where a leg cannot reach. The cook
  writes an animation library's clips into a model retargeted
  (`animation { rig library footIk }` in its import settings, re-cooked
  when the rig or the library changes); `og.anim.retarget`,
  `og.anim.humanoid` and `og.load.rig` do it at run time, cached. Feet stay
  planted with zero height error and slide on rigs at 1.267 and 0.689 times
  the library's legs (11.5 mm off with forward kinematics alone).
- **2D ragdolls** (row 27; bf15ab1, 53331df): og.Ragdoll in the 2D profile,
  rapier2d capsules on hinges about z limited by `jointLimit`, following
  the animation at weight 0 and blending as in 3D; `og.Physics.profile` can
  now be set at run time.
- **Cloth self-collision** (row 26; bf15ab1): `og.Cloth.thickness` (a
  sorted spatial hash, pairs pushed apart every pass, a point moving at
  most half the thickness a step), `substeps` and `friction`. It costs 5 to
  6 times the plain cloth: about 0.5 ms a tick for a 20 x 20 sheet, 2.4 ms
  at 32 x 32.
- **Fluids** (row 25; bf15ab1, 53fa607): up to sixteen metaball fluids a
  frame, packed four to a field; `og.Fluid2D.group`, `density` and
  `mixing`: fluids of one group (four at most) step as one particle set,
  heavier ones sinking, held apart at their boundary unless they mix, their
  mix evening out where they touch and drawn as one surface whose colour
  follows it. A group costs one fluid of all its particles plus 7 to 14 %.
- **fx-showcase** gains four stations, retarget, ragdoll2d, cloth and
  fluids (4a716f7), each with a playtest and references on metal,
  vulkan-cpu, webgpu and webgl2 (f7a0d88, f270817).
- **Determinism**: `tools/wasm-determinism` (the gate's determinism step)
  now hashes og-physics' beyond-rigid-bodies scenario with the new cloth,
  fluid group and 2D ragdoll in 3D and 2D, and og-anim's walk with its
  retargeting; host debug and release and wasm32 scalar and simd128 agree
  (ca02ca3). f37a586 restores the E5 joints station's 3D ragdoll bit for
  bit after the 2D ragdoll refactor.
- **og-ui** (cf932f8, done here at the coordinator's request though it is
  og-ui's): the platform's new danger tokens, the danger button on the
  danger tint, a contrast test of every button role (row 43), and the og-ui
  and ui-kit references the colour touched, re-recorded.

### Gate and wasm

`OG_GATE_JOBS=2 scripts/check.sh --all` on 5d982db, 2026-10-19, with other
agents' builds on the machine: **PASSED, 9 of 9 steps in 743 s**: fmt 5 s,
clippy 21 s, drift 5 s, determinism 46 s, wasm-build 235 s, wasm-opt
305 s, test 347 s, size-gate 92 s, bun-test 154 s. The publish wasm is
**2,975,644 B brotli, 70.9 % of the 4,194,304 B budget**, 32,464 B
(0.8 points) more than the E7 round's 2,943,180 B.
After merging main (11d6820, the terrain and water round and the
archetypes) in 00238f6, `OG_GATE_JOBS=2 scripts/check.sh --all` on 00238f6
**PASSED, 9 of 9 steps in 123 s** (incremental after the path-aware run),
with the publish wasm at **3,013,367 B brotli, 71.8 %** for both rounds
together.

## Status 2026-10-19, E7 round

Phase 1 is closed by the E7 round (tag `phase1-e7`). Against
`docs/design.md` §11 phase 1 it delivers the ECS with reflection, the fixed
deterministic tick and snapshots, `.ogt`/`.ogb` and the cook (glTF,
textures, scenes, fonts, sound, tilemaps, navmeshes), PBR with shadows and
IBL, input, camera, audio, the command channel and the probe (protocol 5
plus the wire), hot reload, `og`, the IDL pipeline (d.ts, wiki, schema, the
hosts' glue), the project template and three archetype games (runner,
match3, topdown) where two were asked, all proved by gfx references on
metal, vulkan-cpu, WebGPU and WebGL2 and by playtests. It also brought
forward much of phase 2: skeletal animation with blend trees and IK, rapier
in 2D and 3D with character, platformer and vehicle controllers, joints,
ropes, cloth, 2D fluids and ragdolls, navmeshes and og.ai, particles, the
post stack, decals, atmosphere and water, material shaders, the og-ui
toolkit on the platform's tokens with MSDF text and six style packs, Tune,
i18n, og.platform over the Player protocol and `og polish`. Lighting is
the one phase 1 line not met as written: forward lighting is an 8-light
uniform loop, not Forward+ (row 30). Phase 2 starts with terrain (a
clipmap), Forward+ clustered lighting, animation retargeting, and the L
items of the owed table: planar water (24), 3D SPH (29), Forward+ (30) and
og-ui's rich text, accessibility tree, glass and two-axis scrolling (33).

### Gate

`scripts/check.sh --all` on db65e5a (this commit adds documents only),
2026-10-19, Apple M4 Pro with another agent's platform validation on the
machine: **PASSED, 9 of 9 steps in 237 s**: fmt 1 s, clippy 4 s,
wasm-build 44 s, test 42 s, wasm-opt 37 s, size-gate 21 s, drift 4 s,
bun-test 83 s, determinism 1 s. Beside it: `og test` passes on all six
templates (game 2 suites, ui-kit 25, fx-showcase 9, runner 8, match3 6,
topdown 7; 160 s in all), and `make web-reference` (Chrome for Testing
150) matches all 26 browser frames, fx-showcase's five and eight E5
stations on WebGPU and WebGL2 (27 s). Every committed reference frame
passes; the ones this round re-recorded say why with before and after
images (ba1042a kinsoku, c6d5793 the ragdoll, 93d86e1 the lazy stations,
706bee2 the other tiers). `tests/evidence/E7/break-and-restore.md` has how
each change's test was broken once.

### Wasm

The publish wasm (`og_host_web-publish`, wasm-opt -Oz, brotli -q 11):
**2,943,180 B brotli, 70.2 % of the 4,194,304 B budget** (3,021,619 B,
72.0 %, before the round's `[profile.wasm]`, fcca471: the load-time crates
at opt-level z; the per-tick crates at z cost a quarter of a frame or half
the physics step again) (`tests/evidence/E7/wasm-size.md`: every experiment, its
speed, and what going below 65 % takes).

### Polish

`og polish` on the six templates (`tests/evidence/E7/polish/`; taste n/a
where the HUD is screen-space text rather than og.ui):

| Template | Taste | Feel | Findings | Closed this round |
|---|---|---|---|---|
| game | 100 | 87 | 1 (idle frames: the guard playtest stages its scenario without pressing Start) | — |
| ui-kit | 90 (73 before) | 62 | 111 (rows 41–43, 45; the rest read a catalogue as a game) | polish read dialogs, sheets and the cutscene as HUDs, dimmed text under scrims and touch controls (da05a4d) |
| fx-showcase | 100 | 97 | 0 | the E5 stations built lazily (93d86e1) |
| runner | n/a | 99 | 0 (row 46 below the threshold) | — |
| match3 | n/a | 100 (88) | 0 | the dealing press heard (c49780a) |
| topdown | n/a | 97 (86) | 0 | a hit taken holds the world; footsteps heard (62f52f3) |


### Owed

After E7: 14 rows done (1–13 and the platform asks, 37), row 14 partly (the
wasm at 70.2 %, not under 65 %), and open 25 for the engine's phase 2
(15–35, 41, 42, 45, 46), 3 for the platform (36, 38, 43) and 3 for the
owner (39, 40, 44). After the phase 2 UI round (P2 UI below): 11 more done
(15–19, 21, 33, 41, 42, 45, 46); row 43's engine side came with the
animation and physics round (cf932f8).

After the phase 2 animation and physics round: rows 25, 26, 27 and 43 done
too; 29 (3D SPH) still open.

After the phase 2 3D SPH round: row 29 done too.

After the phase 2 showcase round: three new rows open (47–49), row 12 now
covers the archetypes' typecheck too.

After the web host and tools round (P4 below): rows 44 (the engine's
part), 48 and 49 done; row 50 new (the per-instance render cost the
three.js comparison found).

After phase 4 (2026-10-21, the P4 sections above): rows 47 (493ea96) and
50 with its remainder (dedf817, bddb741) done; still open for the engine
14 and 35 (35 waits on wgpu 31), for the platform 36, for the owner 39, 40
and row 44's budget meaning; twelve new rows (51–62) for what the phase 4
rounds left out and no later round or stream in flight took up (cook
round four and stylisation round four, merged after this record's base,
closed their rounds' other left-outs). In flight on unmerged branches, so
not rows: 52-byte instances (`render-parity`), `scripts/wave-end.sh`
(`ci-green`). *(render-parity merged at 26649d4c: row 50.)*

After render parity, physics lanes round three, the dogfood game and wave
7 (2026-10-21, 767ecf84; the sections above): no row closed. Render
parity finished row 50's remainder natively and ran no browser (row 51
stays); physics lanes round three measured natively only (row 52 stays);
the wave-end run of row 53 is in progress (Main gate, wave 7). Five new
rows (63–67) for what the wave 7 streams left out and said so; every
Lantern Vale finding is fixed or in one of them
(`examples/lantern-vale/FINDINGS.md` §After wave 7). Still in flight on an
unmerged branch, so not a row: `scripts/wave-end.sh` (`ci-green`).

After waves 8 and 9 and the second dogfood game (2026-10-21, 517d8c4e;
the sections above): row 68 closed (taa-webgl2, 1a9037a9); rows 64 and 65
were worked by api-surface round two (its section) and still read open
here; eight new rows (69–76) for what the streams left out and said
so. Every Hollowbell finding is fixed or in row 75 or 76
(`examples/hollowbell/FINDINGS.md` §After wave 9). The wave-end
measurements are pending (Main gate, waves 8 and 9).

After waves 10 and 11 and the third dogfood game (2026-10-21, ce93d34b;
the sections above): rows 71 (lanes-per-profile), 72 and 73
(physics-3d), 69, 75 and 76 (cook-ui-words) and 80 (cook-determinism)
closed; seven new rows for wave 10's left-outs (77–83, 80 closed since)
and three for wave 11's (84–86). Every Kestrel Stair finding is fixed,
its `chapters` frame owed in row 84
(`examples/kestrel-stair/FINDINGS.md` §After wave 11). The wave-end run
of rows 54, 74 and 82 is in progress (Main gate, wave 11).

Every item a round's status left owed or out, in one place; the round
sections below point here instead of repeating them. Size: S a day or
less, M a few days, L a workstream. Owner: **engine** is this repository
(E7 when this round closes it, else phase 2), **platform** is
`origingame/origingame` (read only from here), **owner** is a decision or
resource only the owner can give.

| # | Item | Round | Size | Owner | State |
|---|---|---|---|---|---|
| 1 | `og.ui.shake` for game code (only a refused press shakes today) | E3 | S | engine, E7 | done, da47e5e |
| 2 | Nine-slice edges and centre tiled, not stretched, when the skin says so | E3 | S | engine, E7 | done, 4a19260 |
| 3 | Phone references for the six ui-kit skin packs (desktop only) | E3 | S | engine, E7 | done, d7c95b3 |
| 4 | Camera zones that scroll between rooms instead of only clamping; topdown's door uses it | E4 | S | engine, E7 | done, 0c630ce |
| 5 | Core commands that read and set resources; game commands' usage text in `help` | E4 | S | engine, E7 | done, cea6fab |
| 6 | `og.tilemap.getTile` on a terrain layer answers the resolved tile, not the terrain bit | E4 | S | engine, E7 | done, 0006005, e7f4966 |
| 7 | Ragdoll bodies follow the animation at weight 0 (the runtime never called `set_ragdoll_targets`) | E5 | S | engine, E7 | done, c6d5793 |
| 8 | runner on the 2D platformer controller, topdown on the 2D character movement | E5 | S | engine, E7 | done, 142b9da, 497deee |
| 9 | Japanese line breaking with kinsoku rules in og-ui (CJK breaks between any two characters) | E6 | S | engine, E7 | done, ba1042a |
| 10 | `og polish`'s text measures count icon names and disabled buttons | E6 | S | engine, E7 | done, 891b4b6 |
| 11 | fx-showcase's E5 stations built lazily per station (its playtests and polish frame time dropped to 100/88) | E6 | S | engine, E7 | done, 93d86e1 |
| 12 | `templates/fx-showcase/src/main.ts`'s type errors; templates are not typechecked in the gate | E6 | S | engine, E7 | done, c5295be; the archetypes too, through `tsconfig.templates.json`'s `templates/archetypes/*/src` glob, third-person (Ember Vale) among them |
| 13 | Native and Chrome world hashes of the three archetypes compared (match3 differed, not investigated) | E6 | S | engine, E7 | done, 85c2bd0 |
| 14 | The publish wasm under 65 % of the 4 MB brotli budget without removing features (71 % before E7) | E7 | M | engine, E7 | deferred until the features are complete (2026-10-22, "Features before optimisation"); was: partly: 72.0 % to 70.2 % (fcca471); under 65 % is row 44. Open after phase 4: per download (docs/ci.md, the shipping `wasm` profile, recorded by 5e9cbcf on 2026-10-21 before stylisation phase B) 3D publish 2,947,818 B (70.3 %), 2D publish 2,783,920 B (66.4 %), lanes 3,320,578 B (79.2 %): each within §7's 4 MB per download, none under 65 %; phase 4 added 93,953 B to the 3D publish module (2,853,865 B at 5dc3aa0); not measured after 5e9cbcf (row 53) |
| 15 | UI sounds start one tick after the input (`og.ui.sound` runs after `og.audio`) | E3 | S | engine, P2 UI | done, 59ce0b4 |
| 16 | Springs for the skin's hover and press (eased curves today) | E3 | S | engine, P2 UI | done, 814fdad |
| 17 | Per-glyph tracking from a skin's ramp (needs per-glyph spacing in og-render's text) | E3 | M | engine, P2 UI | done, 0df11b1 |
| 18 | A real text glow (an MSDF outline in og-render; four offset copies today) | E3 | M | engine, P2 UI | done, 0df11b1 |
| 19 | An atlas tool for skins drawn by hand, beyond `make-skins.py` | E3 | M | engine, P2 UI | done, f4c236b |
| 20 | `vulkan-cpu` and browser references for og-ui's scenes, the UI kit, the skin packs and the locale screens (metal only) | E2, E3, E6 | M | engine, phase 2 | done, 2131b15, de92f08, 8f1d3af (all 74 frames on vulkan-cpu; the browser tiers, first recorded too, are judged against metal since their copies were dropped as duplicates; the browser kit runner in web-reference) |
| 21 | Pads beyond the first in og-ui | E2 | M | engine, P2 UI | done, ca179c0 |
| 22 | Browser runs of og-tests' own gfx scenes (a wasm harness) and GL-tier references (`msaa` among them) | M3, E2 | M | engine, phase 2 | done, 0ac7c34, c43947e, 9470fc3, eaf0517 |
| 23 | Shader hooks on skinned meshes; shadows and the depth prepass displaced by `vertex_offset` | E5 | M | engine, phase 2 | done, 12e93b5, 49dfd0d (`og.render.setMaterialShader`), aa81258 |
| 24 | Planar water reflection, shadowed water, an underwater view | E5 | L | engine, phase 2 | done, 0ee78f1 (og-render), 5be9281 (og.Water fields), 15ff230 (fx-showcase terrain station) |
| 25 | More than four metaball fluids a frame, fluids that interact | E5 | M | engine, phase 2 | done, bf15ab1, 53fa607, 4a716f7 (sixteen a frame, groups that layer by density and blend by mixing) |
| 26 | Cloth self-collision | E5 | M | engine, phase 2 | done, bf15ab1, 4a716f7 |
| 27 | 2D ragdolls | E5 | M | engine, phase 2 | done, bf15ab1, 53331df, 4a716f7 |
| 28 | A moon disc in the night sky | E5 | S | engine, phase 2 | done, 1df9797, 9037875, a5cdb89 |
| 29 | 3D SPH (3D water is the height field) | E5 | L | engine, phase 2 | done, 6dcdc98, a513e20, 71257f0, 200b6fe, 2f9e908, 792aa2b, 332ec84 (double density relaxation in 3D, a screen-space surface on all four tiers, the fluids3d station) |
| 30 | Forward+ clustered lights beyond the 8-light uniform loop (CPU tile table on WebGL2) | M3, M6 | L | engine, phase 2 | done, a061113, 1296a02, f381a66 |
| 31 | A normal matrix for non-uniform scale, vertex tangents, clearcoat, and a gfx scene of the M6 material features | M6 | M | engine, phase 2 | done, 12e93b5, 49dfd0d, eb869b8, aa81258 |
| 32 | The runtime loaders transcode to the tier's compressed texture format and upload it (RGBA8 today) | M4, M6 | M | engine, phase 2 | done, 028c2fc |
| 33 | og-ui's rich text, accessibility tree, glass, two-axis scrolling (`crates/og-ui/README.md` §What phase 2 owes) | M6 | L | engine, P2 UI | done, 79efd08, 6139efe, 38694c6, 42d7294 |
| 34 | The M4 "glTF edit to reload in 300 ms" run and the M5 timed 8-frame `sweep`, filed as evidence (`og dev` prints the first) | M4, M5 | S | engine, phase 2 | done, 3d7ad18, 5119a09 |
| 35 | wgpu and naga 31 when published, kept only if every reference frame passes unchanged | Toolchain | S | engine, phase 2 | open, waiting on wgpu upstream: not published (2026-10-19 the newest on crates.io is 30.0.1, of 2026-08-22; 2026-10-21 `cargo search wgpu` and `cargo search naga` still list 30.0.1) |
| 36 | `og polish` measures frame time on the tier it runs on (metal here), not on the executors' lavapipe | E6 | S | platform | open, waiting on the platform (2026-10-21) |
| 37 | The eight platform asks (`docs/platform-asks.md`) | E6 | M | platform | done: landed on the platform 2026-10-19, the engine speaks it (96f4ebd) |
| 38 | The platform side of the cutover: merge `origin-engine-cutover`, pin this revision | E1 | S | platform | done: the platform merged the cutover (origingame `7e5789d89`) and pinned `cf3cb0b`; on 2026-10-19 the pin moved to this main, `eecf01d` (`agent-server/scripts/e2b/origin-engine.json`), with the knowledge mount, local layout (lanes variant, rust-src remapped), runner tests, workspace stack and a real-Portal page-release check on it (origingame `goals/origin-engine/evidence/pin-eecf01d.md`); production follows once the game-host ingress admits `/__player/v2/` and the E2B templates are rebuilt |
| 39 | A third native tier beside `metal` and `vulkan-cpu` (the browser tiers stand in) | E4 | M | owner | open, waiting on the owner (a machine or runner with another GPU; 2026-10-21) |
| 40 | Mobile measurements: the iPhone first frame and frame times (Safari, WebGPU), the mid-range Android view benchmark | M6, E1 | S | owner (devices) | open, waiting on the owner's devices (2026-10-21); design §7's phone budgets stay unmeasured until then |
| 41 | og-ui plays no sound when keyboard or gamepad focus moves (ui-kit's sound on input 38 in og polish) | E7 | S | engine, P2 UI | done, ca81fb8 |
| 42 | The kit's small type: the shop cards' item names (`caption`, 7 CSS px of ink), the idle HUD's generator names and the `meta` units, under og polish's 9 px | E7 | S | engine, P2 UI | done, f9bad51 |
| 43 | The default tokens' primary and danger button labels at 3.2:1 and 2.9:1 against their fill (og polish wants 4.5:1; WCAG's large text 3:1) | E7 | S | platform (the design tokens) | done: the platform's tokens of b71ae8fac, og-ui synced with the danger tint in cf932f8 (every role 4.5:1 or more, `crates/og-ui/tests/button_contrast.rs`) |
| 44 | A publish wasm per profile (measured 63.3 % 3D, 59.5 % 2D at opt-level 3): two shipping variants, `og build --publish` choosing by the game's profile, and the size budget read per download instead of as the sum of every wasm the gate builds | E7 | M | owner (the budget's meaning), then engine | deferred until the features are complete (2026-10-22, "Features before optimisation"); was: done (the engine's part), 53a36fd: 3D publish 2,853,865 B (68 %), 2D publish 2,688,261 B (64 %), lanes (both profiles, as before) 3,230,782 B (77 %, the largest download), measured after merging main at 9b7f45c (physical materials); the budget per download in design §7, docs/ci.md §Updating a budget, docs/web-export.md §Variants. The budget's meaning stays the owner's: the per-download reading is the engine's, written into design §7 by bf94ccb, and no owner decision is recorded (2026-10-21) |
| 45 | `og.ui.shake` shown in a template (ui-kit's shop refusing a purchase) | E7 | S | engine, P2 UI | done, d00f20c |
| 46 | runner: a press during the game-over card's hold is silent (og polish: 1 of 3 inputs in `flow`) | E7 | S | engine, P2 UI | done, 54b59df |
| 47 | vulkan-cpu references for the third-person archetype (metal recorded; webgpu and webgl2 judged against metal except the webgl2 beacon, which keeps its own reference by design: compute-only particle collision, MSAA against TAA) | P2 showcase | S | engine, phase 2 | done 2026-10-21: the three frames recorded on lavapipe (og-lavapipe-bun, the cross-built addon; `tests/frames.og.ts` with `OG_FX_CAPTURE=1`, 360 s under emulation) sit within RMSE 0.0043 of `metal` with no outlier pixel (title 0.0041, beacon 0.0043, dawn 0.0029), so by docs/ci.md §Browser reference tiers they are judged against `metal` and no `vulkan-cpu` copy is kept; the WebGL2 beacon stays by design |
| 48 | The fantasy pack's labels under 4.5:1 (Set out 3.36, footer 3.5, Resume 3.84, Again 3.99, 完成 4.41, 语言 4.43 in Ember Vale's polish) and third-person's HUD Jump and Kindle without a ui-bus sound | P2 showcase | S | engine, phase 2 | done, 718fb43 (polish contrast 100, lowest 4.65:1; UI sounds 3 of 3 on the phone; `crates/og-ui/tests/pack_contrast.rs`) |
| 49 | The IDL's og.IkChain docs: a `"world"` target is in the skeleton body's Transform parent's frame, not true world | P2 showcase | S | engine, phase 2 | done, 868d1b2 |
| 50 | The engine's render cost per instance of one mesh: 20,000 moving spheres take 174 ms a frame on WebGPU and WebGL2 (extraction plus og-render; the tick 0.3 ms), doubling near 4,096 instances and about 8.7 us an instance beyond (`tests/evidence/P4/web-host-tools/threejs-comparison.md`; the spike drew the same field in 4.6 ms) | P4 web host | M | engine, phase 4 | done dedf817: extraction decoded every og.Mesh row to JSON on the lanes, and the lanes wasm's Workers waited on one allocator (85 % of the frame in og_lanes::wait_for); og.Mesh and og.Transform are read from their columns, the wasm decodes rows serially. The field's frame went from 137.6 to about 4 ms; what remains against three.js is the lit pass's GPU cost per instance (P4 perf round, `tests/evidence/P4/perf/`). That remainder (branch `r-lean`, bddb7411): plain materials draw through lean lit, prepass and shadow pipelines (10 interpolated floats instead of 23, only the attributes read, no discard where no screen derivative is taken; og-render README, Lit pipeline variants); natively spheres-20k 7.1 to 4.6 ms a frame on an M4 Pro, the §7 reference scene unchanged, every Metal og-gfx frame byte-identical (`tests/evidence/P4/lean/`); the instance transform stays 64 bytes (a 3x4 would save a quarter of an upload costing 0.04 ms). Then (branch `render-parity`, 038cc380 to 5a96a149, `tests/evidence/P4/parity/`): instances are 52 bytes on the GPU (an affine matrix's columns' xyz and the array-map layer; the projections of the points they move go through `instance.wgsl` `mul_point` so Metal's sums stay the 4x4's), plain meshes can be 16-byte compact vertices (`VertexEncoding::Compact`, opt-in: quantised), the sort and the bounding spheres are kept across frames, wgpu's indirect-call validation is off and objc2 has no debug assertions in the dev profile; every Metal og-gfx frame byte-identical. Natively the dev CPU of spheres-20k went from 4.27 to 1.76 ms (moving 4.27 to 2.96, gltf-reference 3.89 to 0.95), release 1.60 to 1.13 ms; the frame is GPU-bound at 5.4 ms (from 5.6), the GPU bound by the field's 7.3 million triangles, which neither encoding moves natively; the §7 reference scene unchanged. The browser comparison with three.js is to be run again (row 51) |
| 51 | The three.js comparison (`tools/bench/threejs-compare.ts`) run again after the lean pipelines: before them the 20,000-sphere field was 0.31x of three.js's WebGPU InstancedMesh and 0.48x of WebGL2's, the §7 reference scene 2.04x and 1.54x | P4 lean | S | engine, phase 4 (a wave-end measurement) | deferred until the features are complete (2026-10-22, "Features before optimisation"); was: open (render parity, 26649d4c, measured natively only: `tests/evidence/P4/parity/`) |
| 52 | The lanes wasm's physics step timed and hashed in Chrome: `bun bench/lanes.ts --chrome --pile` (1,000 and 2,000 boxes, the three modes), left to the coordinator by both physics lanes rounds | P4 physics lanes | S | engine, phase 4 (a wave-end measurement) | deferred until the features are complete (2026-10-22, "Features before optimisation"); was: open (round three, 1f0ecdc3, measured natively only; `og.physics.stats` through the web host is unmeasured too) |
| 53 | A wave-end `scripts/check.sh --all` and `web-reference` on main after phase 4, their summary filed (none is recorded after 50eb7f7: the 2026-10-21 gate above; the looks station's 37 browser frames per tier have never run on record) | P4 gate | S | engine, phase 4 | open; `scripts/wave-end.sh` landed with ci-green (53b876f7) and has not run for real (row 82); wave 7's and waves 8 and 9's runs are recorded in their Main gate sections |
| 54 | Reference frames owed after the wave-end lavapipe pass (4d6188b). `vulkan-cpu` for frames recorded or moved on `metal` since: og-tests `stylise-surface`, `stylise-object`, `stylise-halftone-surface` (no lavapipe run), `sprites-shadow-terrain` (moved twice, da96248 and 721886b) and `sprites-20k` (re-recorded on `metal` in da96248, its vulkan-cpu copy from 1e0dee6); og-ui `text-fields-2`, `text-mixed-runs` (no lavapipe run) and `text-bidi` (re-recorded on `metal` in a2c5e44, its vulkan-cpu copy 033bf00's); fx-showcase's looks frames, every one moved by the HUD's preset label (b76ca77, 721886b) while the 35 vulkan-cpu copies predate it. og-tests `materials-array` too (55c0c53; its `metal` reference f1d9233) | P4 stylisation 3 and 4, UI fields, cook 4 | S | engine, phase 4 (a wave-end capture) | closed: wave 11's second wave-end run (e7d6cb5e, `--record og-ui,...`) recorded og-ui's 85 frames on lavapipe and every one is within tolerance of `metal` (worst `kit-desktop-widgets` RMSE 0.0086, 0.159 %), so og-ui keeps no vulkan-cpu copy at all (36b326c3; Main gate, wave 11) |
| 55 | Stylise in surface space: the octave fade shows as faint bands on a long receding floor (the footprint is measured across the pixel, not along the depth gradient) | P4 stylisation 3 | S | engine, phase 4 | closed at 725b9a4e (Wave 13, vox-stylise): the footprint measured along the depth gradient; three stylise frames re-recorded |
| 56 | Compressed animation clips are rebuilt to f32 at load, so a resident clip is no smaller in memory; og-anim sampling the quantised keys would be | P4 cook-anim | M | engine, phase 4 | deferred until the features are complete (2026-10-22, "Features before optimisation"); was: open |
| 57 | The CPU frustum cull's lane grain: natively with lanes on it splits from 2,048 bounds and then costs two to three times its serial time on a loaded machine (`tests/evidence/P4/web-host-tools/culling-threshold.md`, noted, not changed) | P4 web host | S | engine, phase 4 | deferred until the features are complete (2026-10-22, "Features before optimisation"); was: open |
| 58 | og-ui's cooked-font text frames in the browser tiers (`text-bidi`, `text-kerning`, `text-edit`, `text-mixed-runs` cook their fonts, which og-tests-web cannot) | P4 UI text | M | engine, phase 4 | open |
| 59 | og-ui text, what the UI text, fields and UI four rounds left (og-ui README §What phase 2 owes): the caret and Backspace step by character, not grapheme cluster; a mixed text's runs share the role face's baseline; word moves are logical in right-to-left runs; emacs keys beyond Ctrl+A/E; native hosts on macOS edit with Ctrl; a `[wave]`, `[shake]` or `[pulse]` span that mixes faces is drawn still | P4 UI fields, UI four | M | engine, phase 4 | closed at e6c445bc (Wave 13, ui-five) but for an emacs kill buffer and visual caret movement across a bidi edge, in the og-ui README's owed list |
| 60 | og-ui accessibility: the slider's and spinbutton's mirror carry no `aria-valuemin`/`-max` (the tree has no bounds); a reader's adjust checked through the native range's `input` event in Chrome, not with VoiceOver itself | P4 UI shell, UI four | S | engine, phase 4 | closed at e6c445bc (Wave 13, ui-five): the tree carries valueMin/valueMax/valueNow, the mirror writes the aria range |
| 61 | og-ui materials and input: an icon set's single-channel distance rounds miter joins; the kit's other screens keep the 32-cell PNG icon grid; each glass layer with overlay content under it rebuilds the blur pyramid, and the glass cost wants a quieter rerun (`tests/evidence/P4/ui-shell/glass-cost.md`); no fling after a two-finger scroll; a zoom view takes no keys | P4 UI shell | M | engine, phase 4 | closed at e6c445bc (Wave 13, ui-five) for the joins, the icon set, the fling and the zoom view; the glass blur pyramid's cost deferred with the optimisation rows |
| 62 | Platform ask 12: the game wizard's style step (a catalogue entry from the premise, the look written, its three checks for the blind critic; `docs/platform-asks.md`) | P4 knowledge 3 | M | platform | open, waiting on the platform |
| 63 | `.vox` animation, the rest (og-cook README §What phase 2 owes): MagicaVoxel's `nTRN` key frames are not read (a transform's first frame places it throughout, so a part moving across frames does not move), and every folded frame still costs a joint and a draw (its vertices run through the skinning shader at scale 0; a visibility track in og-anim would make a fold free) | W7 cook-vox | M | engine, phase 4 | the key frames closed at 431d9da1 and 900158dc (Wave 13, vox-stylise); the visibility track (a folded frame's joint and draw) deferred with the optimisation rows; `_loop` and the playback range still not read |
| 64 | The runtime's API surface the other way round: `packages/engine/tests/api-surface.test.ts` finds every IDL call on the runtime, but nothing checks for runtime members the IDL does not declare; and no browser test shows a throw in `start` on the web shell (5dfc5a7b's tests are `og run`'s and the native runtime's) | W7 api-runtime | S | engine, phase 4 | closed at 3c036fa4 (api-surface round two): the reverse check fails on an undeclared member, with a one-entry allow-list (`World.id`), and found 17 column factories and three constants never exported; the shell crash path tested under bun (tests/web-shell-crash.test.ts); the runtime `Engine` class stays unchecked (the host's lifecycle object) |
| 65 | String fields, the rest: IDL records (the components, resources and events of `crates/og-api/api.ogt`) cannot declare a `str(N)` field with its capacity; loading a cooked scene with a string field into a live world has no test of its own (og-cook's `scene_string.rs` cooks it); the web host is untested for strings (`strings.test.ts` runs on the native host) | W7 reflect-string | S | engine, phase 4 | closed at a2e48f3f and ae9ae608 (api-surface round two): `field name str.48` in IDL records through every generated output, the cooked-scene string load test in og-runtime, the web-host half of the cross-host string test (run by the stream against the measure lane's module at cf5f2b9, and by the full lane since) |
| 66 | The skin packs' CJK fallback (`fonts/NotoSansSC-Regular-subset.ttf`, 2.5 MB, shared by every pack and every project from the game template) stays outside the pack by decision (9d00fb21): a copied pack's `fallback` line still names it from `assets/`, and og-taste §1 says what to do with that line | W7 ui-text | S | engine | open by decision (Lantern Vale #6's remainder) |
| 67 | Reference frames owed after render-style (on top of row 54): `vulkan-cpu` copies not re-recorded for og-tests `stylised`, `skinned-hooks`, `stylise-pixel`, `stylise-halftone`, `stylise-bands` and `stylise-material` (moved on `metal` by 7e3ae322, 51e765e9 and b9c3ed87; their lavapipe copies predate), no lavapipe run of the new `stylise-outline-voxels`, `stylise-toon-physical`, `sprites-untonemapped` and `stylise-bands-tint`, fx-showcase's nine looks frames re-recorded at 767ecf84 and Lantern Vale's seven frames (`metal` only, re-recorded at 767ecf84); the browser tiers have not run on any of them | W7 render-style | S | engine, phase 4 (a wave-end capture) | closed: vulkan-cpu at 9bd0064 (the lavapipe pass), the browser tiers at cf5f2b9 (web-reference 379 s, every frame; WebGL2 keeps paper-craft and voxel-worlds by design, 95ea28a) |
| 68 | TAA on the WebGL2 tier with a moving camera: the reprojecting path still runs there and whether its history drifts as the still case did (render-style round two) is unmeasured, since no browser reference moves its camera through TAA on WebGL2 (`taa-sharp` is full-tier only); and the arithmetic reason the GL backend's reprojected read differed for a bit-identical view-projection is not found | W7 render-style 2 | S | engine, phase 4 | closed at 1a9037a (taa-webgl2, merged f3229ef): wgpu's GL backend read TAA's depth as 0 (a depth texture read with `textureLoad` and no sampler bound stays on linear filtering, which GLSL ES calls incomplete), so every pixel reprojected from the near plane: wrong motion for a moving camera, and near-plane rounding up to 0.025 px at the 2,800 m looks station for a still one, above the own-texel test. The depth is sampled with a nearest sampler now; `taa-sharp` applies on every tier and is WebGL2's moving-camera frame against `metal` (RMSE 0.00028); two og-tests tests pin the still-camera fixed point natively |
| 69 | og-ui's status badge tones (success, warn, danger, info) under 4.5:1 in most packs (cartoon warn 1.63:1; `pack_contrast.rs` prints them and does not enforce): their colours are shared with meters, toasts and status dots, so they need a palette round of their own; the archetype skins `sparkorb`, `lampdive` and `lanebeat` are outside the pack test | W8 api-surface 2 | S | engine, phase 4 (UI) | closed at dcd8c31e (cook-ui-words, merged a31d6cbe): the badge's tint mixes as the platform's CSS does (`Srgba::tint_over`) instead of in linear light; the tones deepened on the light-panel skins (cartoon's warn #f2a516, 1.63:1, is #7b5102) and lightened on the dark ones, `accentSoft` more transparent on the archetype skins, `make-skins.py` byte-identical; `pack_contrast.rs` enforces success, warn, danger, info and the accent badge at 4.5:1 for the seven packs and sparkorb, lampdive, lanebeat and pathguard. abd2bf9b draws a meter's label in two inks split at the fill's edge (cartoon's "72 / 100" was 2.3:1, neon's 1.3:1). What it left: rows 77 and 78 |
| 70 | A native GL-tier lane: the taa-webgl2 round confirmed the GL defect natively on ANGLE's Metal renderer (wgpu's `angle` feature, ANGLE's libEGL from a Chromium bundle), which would give og-gfx a `gl` backend option and the browser-shaped frames a native run; not committed because it needs ANGLE's dylibs on the machine | W8 taa-webgl2 | M | engine, phase 4 (tooling) | set aside with the CPU sandbox requirement (2026-10-22); an option if the GPU-less tier returns |
| 71 | A lanes module per physics profile: measured with wave 8's trims, 3D lanes 3,088,792 B (73.6 %) and 2D lanes 2,919,700 B (69.6 %) against the one lanes module's 3,464,929 B (82.6 %); no game loses anything (a page already downloads its own profile's publish module). It changes what ships: `wasm-crates.txt`, `check.sh`, `size-gate.sh`, `wasm-lanes.sh`, packages/engine's `build.sh`, `pack.sh`, `web-build.ts` and `doctor.ts`, the determinism and browser tests that run 2D worlds on the lanes module, docs/web-export.md, and one more lanes build per full gate; needs the gate and the browser tiers | W8 wasm-size | M | engine, phase 4 (web export) | closed at 706833d5 (merged 0e77b6b6, on main at c638a46f, recorded at a91e6139): 3D lanes 3,095,052 B (73.8 %), 2D lanes 2,928,498 B (69.8 %); `--all` 9 of 9, web-reference and the web export, player and wire tests passing on the branch |
| 72 | `og.CharacterController` (3D) still uses rapier's character controller, whose cast with the skin as a target distance parry resolves inaccurately (the cause of the platformer's stutter and sinking, e86b55a5); likely the same in 3D, unmeasured | W9 physics-2d | M | engine, phase 4 (physics) | closed at 876dd4ea (physics-3d, merged bb7c1e81): measured first (rapier's controller stalled to 0 to 0.4 m/s a tick about once a second on a box, adjacent boxes and a trimesh, stood 1 mm above the floor instead of its 2 cm skin and crept up 0.1 mm a tick, sank 9 cm when placed on the floor, refused 0.2 m stairs, climbed a 30° slope at 4.4 m/s); one mover for both controllers in both profiles (`crates/og-physics/src/mover.rs`), `tests/character_ground.rs` with 7 tests failing on the old controller; physics_hash 947364242694612608 to 15539601638348908910 by design |
| 73 | The platformer's query filter ignores the collider mask (collision layers) | W9 physics-2d | S | engine, phase 4 (physics) | closed at 876dd4ea (physics-3d, merged bb7c1e81): both controllers' query filters take the collider's collision groups (`og.Collider.layers`/`mask`), two tests in `tests/controllers.rs` |
| 74 | Reference frames owed after render-2d: `vulkan-cpu` and the browser tiers for og-tests `tilemap-lit` (new), `stylise-palette` and `stylise-retro` (240-line scenes, now 3x), og-ui `screen-layers` (new), Hollowbell's nine frames and the three 240-line looks presets on vulkan-cpu, and og-ui `kit-skin-pixel` and `kit-skin-pixel-phone` on vulkan-cpu (their copies, abd61fb8, predate the pixel pack's label weight) | W9 render-2d | S | engine, phase 4 (a wave-end capture) | closed: the second wave-end run (e7d6cb5e) recorded `screen-layers`, `kit-skin-pixel` and `kit-skin-pixel-phone` on lavapipe within tolerance of `metal`; og-ui keeps no vulkan-cpu copy (36b326c3; Main gate, wave 11) |
| 75 | The cook does not warn on an unknown Tiled layer or object property (Hollowbell's `repeat` = x is silently ignored; the read ones are `repeatx`/`repeaty`); a warning naming the property and the og-game table would catch it | W9 knowledge-2 | S | engine, phase 4 (cook) | closed at 64f79249 (cook-ui-words, merged a31d6cbe): `og cook` warns through `Report.warnings` on every Tiled custom property it does not read, naming the property, where it sits and the names read there (a test; og-game §8's table gains `solid`); Hollowbell's map tool writes `repeatx` at f3bf909e and the hills repeat (title, play and results re-recorded: 0.0872, 0.0870, 0.0320) |
| 76 | og.ui templates whose own words still have no desc field, after ui-runtime gave `SettingsDesc` and `ShopDesc` theirs (8a7d8702; Hollowbell #12's remainder): `inventoryScreen`'s hint ("Select an item. Drag it to another slot to move it.") and "Count", `leaderboardScreen`'s "Retry", `tutorialOverlay`'s "Skip", "Next"/"Done" and its "1 of 3" step count, `cutscene`'s "Skip" (`levelSelect` has none: its title is a field). A localised game shows them in English | W9 ui-runtime | S | engine, phase 4 (UI) | closed at 77cb5b0a (cook-ui-words, merged a31d6cbe): `InventoryDesc.hintLabel`/`countLabel`, `LeaderboardDesc.retryLabel`, `TutorialDesc.skipLabel`/`nextLabel`/`doneLabel`/`stepLabel` ("{step} of {count}"), `CutsceneDesc.skipLabel`; the ui-kit demo passes them from its five tables (an `offline:` command for the leaderboard's error state), its locales playtest asserts them in Chinese, og-i18n §2 lists them. Hollowbell's cutscene "Skip" is row 79 |
| 77 | Palette round two: the archetype skins' buttons and fields under 4.5:1 (sparkorb's primary button 3.42:1, lampdive's danger button 2.58:1 and its field placeholder 3.57:1, pathguard's primary 3.30:1; row 69's test holds only the status tones and the accent badge), and the examples' own skins, outside `pack_contrast.rs` (Hollowbell's danger 4.26:1; Lantern Vale's success 3.15, warn 2.72, info 4.07:1) | W10 cook-ui-words | S | engine, phase 4 (UI) | closed at 3da271b9 and b358d6a8 (merged 38f6435d): all 14 skins under the packs' rule, every row 4.5:1 or better (primary and secondary labels at rest and hovered 7:1) |
| 78 | Badges the palette round left: the neutral badge (scifi 2.15:1, neon 2.81:1; printed by `pack_contrast.rs`, not enforced) and the light packs' badges on a dark stage (about 2:1; the test holds a badge in its panel, and on the screen only where screen and panel are both dark or both light) | W10 cook-ui-words | S | engine, phase 4 (UI) | closed at b358d6a8: `Srgba::readable_on` moves a badge's or meter label's ink on the other side of the scale, the neutral badge's ink or fill made to read |
| 79 | Hollowbell's cutscene "Skip" is still English in Chinese: the mole's talk does not pass `CutsceneDesc.skipLabel` (77cb5b0a); it needs a table key and 过 in the game's Chinese subset (FINDINGS #12) | W10 cook-ui-words | S | example (Hollowbell) | closed at dcdcd17c: `talk.skip`, 跳过, the subset regenerated |
| 80 | The cook's manifest differs between thread counts: og-cook's `cooking_twice_gives_identical_bytes` (an all-cores cook against a one-thread cook of the game template, byte for byte) failed 2 of 5 runs on GitHub's two-vCPU runner | W10 ci-green | S | engine, phase 4 (cook) | closed at 0b245141 (basisu's padding bytes, the vendored one-line fix) with 78b8d0d8's thread-count test; the pipeline cache's torn file at 9ffc36cb |
| 81 | A particle test crashed with SIGSEGV once on Lavapipe (ci-green's gate job on GitHub's runner; its re-run passed); not reproduced, not diagnosed | W10 ci-green | S | engine, phase 4 | set aside with the CPU sandbox requirement (2026-10-22): no lavapipe run is scheduled; the reading stands if it returns |
| 82 | `scripts/wave-end.sh` (ci-green: ddec5756, a56a2ccb, 3816e4a4) has never run for real, only with `--dry-run`; row 54's og-ui frames on lavapipe (and row 74's `screen-layers`, `kit-skin-pixel` and `kit-skin-pixel-phone`) depend on its `ui` stage (`--record og-ui`), as do the vulkan-cpu copies that predate wave 10's metal re-records (og-ui's kit frames from abd2bf9b, Hollowbell's `title` and `play` from f3bf909e) | W10 ci-green | S | engine, phase 4 (a wave-end capture) | closed: two real runs at wave 11's end, the first (15f4e9eb) failing prepare and the playtests on the examples' ungenerated assets and stale kit copies, the second (e7d6cb5e, after the prepare stage learned to make them) passing every stage in 9,282 s; its copies judged by the rule in 36b326c3 (Main gate, wave 11) |
| 83 | GitHub CI removed during the build-out (the owner's decision, 2026-10-21: the workflow and its provisioning action deleted at the commit after aa9e4f8e, after a day as `workflow_dispatch` only; docs/ci.md §CI keeps the summary and names the history commits 53b876f7 and 9d220790 that hold them); the local gate is the only signal. What brings it back: the repository published, restoring the workflow from history with its push, pull_request and nightly triggers | W10 ci-green | S | process | open by decision |
| 84 | The Kestrel Stair's `chapters` playtest frame shows the results screen (the script's timing is off), so the named chapter cards are not under a reference frame; fix the script and re-record | W11 runtime-ui-2 | S | examples | closed at dcdcd17c: the script taps Title after the wipe and opens the chapters; `chapters.png` re-recorded (0.1835) |
| 85 | `og.Mesh`'s hit flash still draws through a copy of its material at 16 levels; moving it onto the per-instance flash record (5bbf4909) would make it one draw and move existing frames | W11 physics-anim-2 | S | engine, phase 4 (render) | closed at 7ab860a0 (merged 5bedf86b): the flash on the instance record, one draw, the stale-copy defect gone with it |
| 86 | Audio zones and the listener read their own `og.Transform`: parented audio (a sound on a child entity, a listener under a camera rig) is not composed down `og.Parent` as cloth is since 12fa298a (the og-audio README's owed line) | W11 physics-anim-2 | S | engine, phase 4 (audio) | closed at f89f1885 (merged 5bedf86b): `og_core::math::world_pose` shared by physics and audio; sources, the listener and zones composed down `og.Parent` |
| 87 | `GltfDesc.castShadow`/`receiveShadow` and `Mesh.receiveShadow` are declared and documented "not applied yet" (c178e216): applying `GltfDesc.castShadow` as declared (off by default) would remove the shadows of every model loaded without it (Lantern Vale, Ember Vale, fx-showcase, the game template), so whether a glTF model casts by default, as `og.Model` does today, or follows three.js is a contract decision | W13 webgl2-render-2 | S | engine, phase 4 (render) | closed at 892a3c94 (merged in wave 13): models cast and receive by default (`GltfDesc.castShadow`/`receiveShadow` default true, `og.Model` carrying both per instance), `Mesh.receiveShadow` applied, 112 project frames byte-identical (Wave 13, shadow-flags) |
| 88 | A terrain streaming test whose tiles arrive after the first frame: since 441f4e6e the first frame that draws a streamed terrain settles every wanted tile (as a capture does), and the fx-showcase terrain station wants its five tiles at that frame, so `tests/terrain-stream.test.ts` (e565ede7) now proves the hashes agree with nothing streaming after the settle; a station whose walk wants tiles beyond the first frame's (a larger terrain, or a camera that moves off it) would prove the stream itself leaves the simulation alone, and the settle's cost on the web's tick-0 frame (every wanted tile decoded in one frame) is unmeasured | W13 webgl2-render-2 | S | engine, phase 4 (terrain) | deferred until the features are complete (2026-10-22, "Features before optimisation"); was: open |
| 89 | perf-games' tables and `og polish` rows on an idle machine (load under 8 for the whole run): every run of 2b896be5 shared the machine with the lavapipe VM, the gate or an iOS simulator | W12 perf-games | S | engine, phase 4 (a wave-end measurement) | deferred until the features are complete (2026-10-22, "Features before optimisation"); was: open |
| 90 | perf-games' follow-ups: the lanes on a loaded machine (Kestrel's physics 0.93 ms serial, 2.9 to 4.2 ms on the pool at load 20 to 25; the grains were measured idle); release sprite extraction still looks each field up by name (about 5 % of Hollowbell's frame); `og.ui.end` re-measures every text each tick; Lantern Vale's dev playtests' step spikes (p95 7 to 9.5 ms against a 2.2 ms median), unprofiled | W12 perf-games | M | engine, phase 4 (perf) | deferred until the features are complete (2026-10-22, "Features before optimisation"); was: open; the text measurement cache and the painter-level grid are on branch perf-2 (6f7f9f6c, a53bb596) |
| 91 | One version everywhere, written by `scripts/release.sh bump`, held equal by the knowledge test; `create` writes it; `og` compares `project.ogt`'s `engine` with the installed package (`doctor` line, run/test/build warning) | release R1 | S | engine, release | done, 3f914f69 (`scripts/release-bump.ts`; `og` prints the pin notice and the doctor its `pin` line) |
| 92 | The knowledge published: inside `@origingame/engine` as `knowledge/` (the owner, 2026-10-22: one package less), `mount.ts --package <installed engine dir>` | release R2 | S | engine, release | done, 3f914f69 and the fold-in commit (a published template's listed binaries are create-game's) |
| 93 | `scripts/release.sh` (`bump`, `pack`, `verify`, `publish`), `pack.sh` packing create and knowledge, `dist/release/<version>/manifest.json`, `CHANGELOG.md`; the wave-end `release` stage | release R3 | M | engine, release | done, 3f914f69 (`scripts/release.sh`, `CHANGELOG.md`, `dist/release/<version>/manifest.json`, wave-end `release` stage) |
| 94 | The install test `tests/install/` (an empty directory, `bun add`, `create-game`, `og doctor`, `og check`, `og test`, `og build --publish`, the runner archetype, the knowledge validate and mount check) on darwin-arm64 and in the `node:22-bookworm-slim` container, from tarballs and `--registry` | release R4 | M | engine, release | written, 3f914f69 (`tests/install/run.ts`, `tests/install/Dockerfile`: node 22 + bun 1.4.2); its first pass is row 98's evidence |
| 95 | `og doctor` and `og --version` outside a checkout (the package's and the addon's version, both wasm files, tsrs, the renderer) | release R5 | S | engine, release | done, 3f914f69 (the doctor's `pin` line; `--version` and the package lines were there) |
| 96 | The template's `trickster.glb` and `dropship.glb` (from `origingame-examples`, licence unnamed) replaced by engine-owned or CC0 models, or the owner's leave to redistribute; every binary in every template and archetype with a named licence | release R6 | S | engine, release (owner for the leave) | done for the packages, 3f914f69: `dropship.glb` is MIT (core-business's LICENSE, named); `trickster.glb` (edg-king-match, no licence) left the template, the `trickster` entity is the CC0 walker; the repository's test copy (`crates/og-tests/assets/`, fx-showcase) is the owner's call before the repository goes public |
| 97 | Package metadata: `repository.url` naming the repository the packages are built from, `homepage`, `bugs`; the README's install section | release R7 | S | engine, release | done, 3f914f69 (`repository` fran0220/origin-engine with `directory`, `homepage`, `bugs`; the README's Install section; the package is `@origingame/create-game` since `npm create @origingame/game` maps to it) |
| 98 | Publish `0.1.0` (`npm login`, the `@origingame` scope: owner), `release.sh verify --registry`, tag `v0.1.0`, `tests/evidence/release/0.1.0.md` | release R8 | S | owner, then engine | done 2026-10-22: `@origingame/engine`, `engine-darwin-arm64`, `engine-linux-x64`, `create-game` at 0.1.0 on npm (packed from 53c3629e), `verify --registry` passing on both platforms, tag `v0.1.0`; `tests/evidence/release/0.1.0.md` |

## 0. Rules that shape the plan

- **The IDL is the contract between workstreams.** `crates/og-api/api.ogt`
  is written first and every crate implements against it. A workstream that
  needs an API change edits the IDL in its own commit; generated outputs are
  CI-checked for drift.
- **Evidence per milestone, not per sprint.** Each milestone names the
  scripted run, the gfx reference frames or the conformance suite that proves
  it. A milestone without its evidence is not done.
- **Four parallel workstreams, one integrator.** Core, Render, Assets, Tools
  run as separate agent threads on their own crates; the integrator merges
  weekly, owns the IDL and the templates, and runs the full suite.
- **2D is in phase 1.** Five of the six Discover archetypes are 2D; the
  profile (orthographic camera, sprite batch, 2D physics via rapier2d) costs
  about two weeks on top of the 3D core and makes the first games real.
- **Both backends from day one.** WebGPU and WebGL2 are the same code with a
  tier flag; the spike has already paid for the differences. Native Metal
  (developers) and Vulkan (executors) come for free from wgpu.

## 1. Workstreams and crates

| Workstream | Crates | Owns |
|---|---|---|
| **Core** | `og-core`, `og-ecs`, `og-idl`, `og-api`, `og-runtime` | ids, handles, reflection macro and schema, `.ogt`/`.ogb`, world, archetypes, queries, schedule, snapshot/restore, deterministic rng and math, the IDL and its generators |
| **Render** | `og-render`, `shaders/` | render graph, materials, instancing, lights, CSM, IBL, sky, mips, MSAA/TAA, bloom/tonemap, 2D sprite and text batches, capture readback |
| **Assets** | `og-cook`, `templates/` | glTF import (meshopt, LODs, tangents), KTX2/Basis, audio (QOA), MSDF fonts, scene compile, cook stamps, the project template and archetypes |
| **Tools** | `og-probe`, `og-cli`, `og-host-web`, `og-host-node`, `packages/engine`, `packages/create`, `packages/knowledge` | command channel, probe, dumps, `og` (build/run/cook/probe/test/doctor), the two hosts, the npm packages, wiki generation |
| **Integrator** | workspace, CI, `tests/`, `docs/` | IDL stewardship, weekly merges, gfx/perf suites, templates in sync, the milestone evidence |

## 2. Milestones

| # | Weeks | Deliverable | Evidence |
|---|---|---|---|
| M0 Scaffold | 1 | workspace with every crate stubbed, CI (fmt, clippy, test, wasm build, size gate, generated-file drift), `og` skeleton, `tests/` harness with gfx reference compare, the spike renderer moved into `og-render` behind the new crate boundary | CI green on an empty game; the spike frame reproduced from `og-render` byte-identical to `.spike-out/native.png`. **Filed** (`tests/evidence/M0/`, 2026-10-17): `spike-frame-reproduced.png` (byte-identical to the spike). CI green: the gate run at 49ed935 is `tests/evidence/M6/check-summary.md`. |
| M1 World | 1–3 | `og-core`: `#[derive(Reflect)]` → schema JSON, `.ogt` reader/writer with round-trip, handles, rng, math; `og-ecs`: archetype storage, queries, system schedule, resources, events, snapshot/restore | logic tests: 10k-entity structural churn, snapshot→restore→identical world hash, `.ogt` round-trip on every template file; a Rust-only "bouncing boxes" world advanced 3600 ticks twice gives the same hash. **Filed** (`tests/evidence/M1/`, 2026-10-17): `cross-target-determinism.md` (bouncing boxes 3600 ticks, the same hash on host and wasm32). The churn, snapshot and `.ogt` round-trip tests run in the gate (`cargo test -p og-ecs`, `og-core`) with no file of their own. |
| M2 API | 2–4 | `og-api`: `api.ogt` v1 (world, components, systems, resources, transform, camera, input, time, rng, load, probe), generators for Rust traits, wasm-bindgen glue, napi glue, `.d.ts`, JSON schema, wiki pages; typed views over columns with tick-stamped invalidation; `og-host-web` Worker + OffscreenCanvas; `og-host-node` | a TypeScript hello-world (spawn 10k entities, one `Move` system) runs identically on both hosts: same world hash after 600 ticks; the spike's view benchmark re-run through the real API within 2× of the raw loop. **Filed** (`tests/evidence/M2/`, 2026-10-17): `hello-world-hash-node.json`, `hello-world-hash-web.json`, `hello-world-two-hosts.md`, `view-benchmark.json`, `wasm-size.json`. |
| M3 Renderer | 3–6 | render graph; material system from reflected material assets; automatic instancing by (mesh, material); directional + point/spot lights; 3-cascade CSM; IBL + sky pass; mip generation; MSAA (GL) / TAA (WebGPU); bloom, tonemap, exposure; 2D sprite batch and MSDF text; capture readback on both hosts | gfx reference frames for 12 scenes (committed PNGs, per backend tier); the spike's glTF scene within ±10 % of the spike's frame time; 20k moving sprites ≤ 2 ms on the M4 Pro. **Filed** (`tests/evidence/M3/`, 2026-10-17): `showcase-metal.png`, `showcase-vulkan-cpu.png`, `perf-baselines.json`, `feature-costs.md`. 11 Metal and 11 vulkan-cpu references are committed (`crates/og-tests/references/`); the browser tiers run og-tests' scenes since the phase 2 tooling round, with WebGL2 references for `msaa`, `showcase`, `overlay` and `water-3d` (`tests/evidence/P2/tooling/web-gfx-*.json`). |
| M4 Cook | 5–8 | `og cook`: glTF → `.ogb` (meshopt vertex/index codec, cache ordering, LOD chain, tangents), textures → KTX2 Basis (UASTC for normals, ETC1S for colour), audio → QOA, fonts → MSDF atlas, scenes `.ogt` → `.ogb`; incremental by hash; cook stamps checked at load; runtime loaders; hot reload by snapshot + bundle swap | cook of the template and archetypes is deterministic (same bytes twice); a glTF edit → `og cook` → reload shows the change in ≤ 300 ms with world state kept; cooked size of the template ≤ 60 % of its sources. **Filed** (`tests/evidence/M4/`, 2026-10-17): `cook-determinism.json`, `cook-report-template.md` (cooked 35.7 % of sources), `cooked-gltf-reference.png`, `cooked-gltf-reference-compare.json`, `texture-psnr.md`. The glTF edit → reload run is filed in `tests/evidence/P2/tooling/m4-m5-tooling.md` (8 reloads, worst 112.8 ms in `og dev`, 251 ms from the write; the edit shows since 3d7ad18). |
| M5 Probe | 6–9 | `og-probe`: command channel (core actions of design §3.3), module-registered commands, probe protocol 5 members + `capture`, `sweep`, `reload`, `snapshot`, `restore`, `dump`, `commands`; native loopback WebSocket; in-page request ring; `publish` trim | the platform's probe conformance suite (`origin/packages/contracts`) green on both hosts; `og run --commands "0.5:touch_move_to:Start;0.6:touch_start;0.7:touch_end;2.0:screenshot;2.2:dump_world;2.5:quit"` on the template produces the frame and the dump; a `sweep` over a tunable yields 8 frames in ≤ 1 s native. **Filed** (`tests/evidence/M5/`, 2026-10-17): `probe-conformance-test-output.md`, `probe-conformance-transcript.json`, `scripted-run-dump-world.json`. The timed 8-frame `sweep` is filed in `tests/evidence/P2/tooling/m4-m5-tooling.md` (298 to 559 ms); the `og run --commands` run on the template is M6's `tools-template-*`. |
| M6 Games | 8–11 | input (pointer, keys, gamepad, touch), camera controllers, audio (AudioWorklet ring, cpal), 2D physics (rapier2d) and 3D rigid bodies (rapier3d, static + dynamic, no character controller yet), minimal UI (panels, buttons, text, stable ids), project template, `npm create @origingame/game`, two archetypes rebuilt (`runner` 2D, `match3`), wiki generated, skills and scaffold in `packages/knowledge` | both archetypes playable in a browser and through `og run --commands`; their playtest scripts pass headless; the knowledge package validates against the platform's `packages/game-knowledge` schema; first-frame on an iPhone and the Android view benchmark recorded. **Filed so far** (`tests/evidence/M6/`, 2026-10-17 UTC; partial, M6 is not done): Core `core-cross-target-determinism.md`, `core-input-edges-transcript.json`, `core-break-and-restore.md`; Render `render-showcase-metal.png`, `render-showcase-vulkan-cpu.png`, `render-shadow-map-256.png`, `render-shadow-map-2048.png`, `render-texture-formats.md`, `render-test-run.md`, `render-feature-costs.md`, `render-perf.md`; UI `ui-sample-screen-metal.png`, `ui-dump-ui-sample.json`, `ui-test-run.md`; Audio `audio-determinism.json`, `audio-mix-timing-release.json`, `audio-mix-timing-debug.json`, `audio-scene-waveform.png`, `audio-test-run.md`; ECS `ecs-state-blocks.md`; Physics `physics-pile-determinism.json`, `physics-pile-determinism-debug.json`, `physics-wasm-size.json`, `physics-test-run.md`; Tools `tools-template-run-frame.png`, `tools-template-dump-world.json`, `tools-template-dump-entity.json`, `tools-template-run.md`; gate `check-summary.md`, `size-gate.json`. Not yet: the archetypes, their playtests, the knowledge package against the live platform schema, the mobile numbers. |
| M7 Platform | 11–13 | `@origingame/engine` published (wasm + native addon per platform via optionalDependencies, `.d.ts`); E2B `engine` class image (bun + engine + lavapipe); runner `og.rs` for `stage_capture`/`stage_record`/`level_report`; mode table without substrate; one Discover prototype produced end to end by a thread on the new engine | the thread's session transcript with frames; `engine` image ≤ 600 MB; `bun deploy/ovh/state.ts` shows the new class live; the platform goal's phase-1 row filled in |

Dependencies: M1 → M2 → everything TypeScript-facing; M0 → M3 (renderer
work starts from the spike immediately); M4 needs M1's `.ogb`; M5 needs M2's
hosts; M6 needs M3, M4, M5; M7 needs M6.

### Status 2026-10-19, E6 round

- **Navigation and AI** (bf15ac2..8e631ee): og-nav (grid A* and jump point
  search without corner cutting, flow fields, a tiled navmesh baked by
  `og cook` from `type navmesh` .ogt with obstacles that rebuild only their
  tiles, steering, the `og.nav` agent system; 200 agents a tick in 0.25 ms
  on a 256 grid) and og-ai (behaviour trees and state machines as `type
  behaviour` .ogt, checks on blackboard keys and component and resource
  fields, sight cones against a physics ray and hearing, the game's
  commands; 200 guards think in 0.28 ms a tick over og.nav's 0.24), IDL
  modules `nav` and `ai`, `dump_ai` and `dump_nav`. The topdown slime runs
  on a tree; templates/game has a 3D guard on a state machine and a baked
  navmesh.
- **Audio** (802c95b..a3ce21b): per-bus inserts (high- and low-pass,
  compressor, ducking, a reverb send), per-voice low-pass for distance and
  occlusion, voice priority and a cap, sample-accurate `startAt`, fades,
  the `og.Music` clock with `og.MusicBeat`; `listen` (probe member and
  command) and `og run --audio`; 18 procedural presets synthesised by
  `og cook` (`og sfx`). 32 voices through every insert mix in 1 % of real
  time; the busy scene renders the same PCM twice.
- **Platform services** (917a861, 88764c5): `og.platform` (store with
  revisions, leaderboards, telemetry, Origin Point, AI marks, settings
  helpers) answered at a later tick; a local JSON store natively, Player
  protocol 2 on the web with a localStorage fallback; `dump_platform`.
  `docs/platform-asks.md` records eight asks.
- **Localization** (7ae351a..ee3b32e): og-i18n (string tables, ICU
  messages, CLDR plurals, negotiation, direction), rustybuzz shaping and
  right-to-left layout in og-ui, per-locale font chains, the cook's key
  check, ui-kit in en, zh-CN, ja, ko and ar against metal references
  (`tests/evidence/E6/i18n/locale-sheet.png`).
- **og polish** (da276d8): playtests played headless and scored against
  og-taste and og-feel (contrast, text size, HUD, transitions, UI sounds,
  latency, hit feedback, idle ticks, frame time, sound on input, locale
  keys), reports with frames and fixes mapped to skill sections. Its first
  run found runner and match3 silent (their sounds were cooked, never
  played), ui-kit's UI sounds off by default and templates/game's Start
  press silent; all four fixed (a98019f..1fcb224). Reports for the six
  templates are in `tests/evidence/E6/polish/`.
- **Knowledge**: skills og-ai, og-audio, og-i18n and og-platform-sdk;
  og-taste §11 and og-feel §14 for the polish measures; og-game's map and
  performance budget.
- Full gate at b6fa67a in 4 min 5 s; publish wasm 2,862,533 bytes brotli
  (68 % of the budget, +433,997 over E3's 2,428,536), shaping (rustybuzz, ttf-parser)
  the largest new share (`tests/evidence/E6/wasm-size.json`). Evidence
  `tests/evidence/E6/`.
- Topdown's `vulkan-cpu` references for the slime on og.ai were recorded
  after the merge into main, in the og-lavapipe container with the x86_64
  Linux build (reproduced at RMSE 0; within RMSE 0.0031 of `metal`).
- Owed: §Owed rows 9–13, 20 and 36–37.

### Status 2026-10-23, debt round (in progress)

- The owed items of depth wave 1 and `ui/style`, one branch
  `debt/<stream>` per stream, two or three Claude Code threads on Opus
  5.5 high at a time (the AGENTS.md rule, 52ccd97e), merged in the order
  they report: `lantern-vale-physics` (586019b8: the engine's capped push
  is right, the example's rocks were 279 kg), `style-names` (c1784e47),
  `render-targets` (34122f3a), `render-lod` (b2918766; its og-render
  lib.rs and api.ogt conflicts with render-targets resolved at the merge:
  probe faces cull with `!gpu_cull`, the probe transparents join the
  GPU-cull range remap, the view passes bind no probe). Metal references
  re-recorded per merge (89af43d4, 7f5d7bd9, 50f1dafd), each frame read.
- Functional gate at 50f1dafd: PASS 5/5 in 200 s (fmt 2 s, clippy 13 s,
  test 196 s, drift 71 s, bun-functional 99 s).
- Second batch merged: `render-light` (941f01d1), `geometry-video`
  (001f1818), `two-d-math` (b0b8c4ee; `IkTarget2D.weight` now multiplies
  the constraint's weight, kept as a behaviour change), `probe-rooms`
  (c373e511; api.ogt and og-runtime's api.rs resolved by hand,
  `gpuTimestamps` beside render-light's capabilities). Metal references
  re-recorded and read per merge; the cook_report test takes the intro's
  ninth track (05ac0003).
- Found on Metal after the probe-rooms merge: every GPU pass read 0 ms.
  Apple GPUs sample counters only at an encoder's own stage boundaries
  (the empty marker pass sampled nothing, a working one measured
  itself) and the samples complete only with the command buffer (a
  resolve encoded in the frame read zeros and stale values). Fixed in
  0d2d92d2: the frame's encoder is `timing::Encoder`, every pass writes
  its own pair, a graph pass is reported in queue order, and the resolve
  is submitted on `on_submitted_work_done`. The trace fixture's 20
  frames all time on an M-series (main 0.2 ms, taa 0.2 ms, tonemap
  0.04 ms); the 60 gfx references are unmoved.
- Every stream is merged. The wave's end (`scripts/wave-end.sh`) once
  after the functional gate of this batch.
- Left for a later round: a list column in records (og-core; eight
  custom post slots until then), the raycast's per-tick hierarchy
  refresh, rollback and the platform's ask #16, og-ui's nine-slice band
  on buttons shorter than their corners, the games' "skin" prose.

### Status 2026-10-19, E5 round

- **Atmosphere, water, material shaders and physics beyond rigid bodies**
  (db38b41..HEAD, branch `e5`):
  - **og-physics** gains joints (fixed, hinge and slider with limits and
    motors, spring, rope, ball) and the 3D kinematic character controller
    (slopes, steps, ground snapping, moving platforms, pushing).
  - It gains the 2D platformer controller (jump height and time to apex,
    coyote time, jump buffer, apex hang, jump cut, air and wall jumps,
    one-way platforms with drop-through, corner correction) with
    `og.CharacterEvent`, and vehicles (rapier's raycast car in 3D, sprung
    motorised wheel joints in 2D).
  - Also new: Verlet ropes and cloth, a 2D particle fluid (double density
    relaxation), height-field and 2D spring-line water with splashes and
    buoyancy, and ragdolls built from an og-anim skeleton and blended from
    the animation.
  - All of it is inside the og.physics state block. A 3D and a 2D world
    with every feature hash the same twice and after a restore mid-run, and
    the hashes are bit-identical on aarch64 macOS and x86_64 Linux.
  - **og-render** gains fog (distance and height, sun scattering),
    procedural sky looks drawn per pixel, `day_night` and weather presets
    (rain, snow, dust, mist).
  - It gains planar water (depth colour, foam, refraction, screen-space
    reflections on the full tier and the sky cubemap on WebGL2), 2D water
    strips and metaball fluids in the sprite order.
  - It gains material shader hooks (vertex offset, surface colour, normal,
    emissive), which og-cook compiles with naga for every back end and
    refuses with naga's message, and mesh replacement.
  - All of it is off by default, and every earlier reference frame is
    unchanged.
  - **TypeScript** reaches it through the IDL's physics module, the new
    atmosphere module (`og.Fog`, `og.Sky`, `og.DayNight`, `og.Weather`,
    `og.Water`, `og.Water2D`, `timeOfDay`), `og.render.shaderMaterial` and
    `setShaderParam`, and `og.load.shader`.
  - **Tests and references**: `templates/fx-showcase` has eight new
    stations, each with its playtest and references on `metal`,
    `vulkan-cpu`, `webgpu` and `webgl2`. og-tests has nine new scenes with
    `metal` and `vulkan-cpu` references and behaviour tests.
  - **Knowledge**: og-game (atmosphere and water, material shaders, physics
    beyond rigid bodies), og-feel §12 (controller tunables by genre) and
    og-taste §8 (atmosphere by intent).
  - **Cost** on the M4 Pro: water 0.66 ms per frame. The 2D fluid costs
    1.15 ms per tick at 1,000 particles. Every other feature costs under
    0.12 ms.
  - **Wasm**: 2,664,999 bytes brotli, 63 % of the budget (60 % before),
    with no feature behind a cargo flag.
  - Evidence: `tests/evidence/E5/`.
- Owed: §Owed rows 7–8 and 23–29.

### Status 2026-10-18, E3 round

- **UI style** (d612bc3..24d3663): og-ui's skin layer: a `Skin` the theme
  carries (nine-slice frames per element class from one atlas, corner,
  divider and heading ornaments, panel tiles, shadows and glows per class,
  text shadows and glows per font role, press squash and hover lift, fonts per
  role with a CJK fallback, an icon set, motion and sounds), cooked from
  `.ogt` files of `type skin` (`og.UiSkin` blobs) and loaded with
  `og.load.skin`; `UiDesc.skin` and `sounds`, `og.ui.counter`,
  `og.ui.burst`, `og.ui.screenTransition` (fade, wipe, iris, dissolve,
  slide; colour or texture, over every layer through the overlay pass),
  `TransitionKind.pop`, `ElementDesc.frame`, `LabelDesc.heading`; UI events
  sound through og-audio on the ui bus (`og.ui.sound`). The default skin
  dresses nothing: every token reference frame is unchanged and the
  captures were pixel-identical before and after. Six style packs (pixel,
  cartoon, fantasy, scifi, paper, neon) in `templates/ui-kit/assets/skins/`
  with generated atlases and icons and OFL fonts, held to 4.5:1 by
  `og-ui/tests/packs.rs`; the kit's 20 screens in every pack against one
  metal sheet per pack; the og-taste skill. Full gate at 24d3663 in 4 min 2 s
  (wasm 2,428,536 bytes brotli, 57 % of the budget; +59,841 over E1).
  Evidence `tests/evidence/E3/`. Owed: §Owed rows 1–3 and 15–20.

### tsrs 2026-10: type checking in the author's loop

The execution package `goals/tsrs/plan.md` (its commit `684db7cb`, deleted
at its end; Git history keeps it) put tsrs, the Rust port of TypeScript 7
(oxc-project/tsrs, pinned at 0.18.0-ts7.1.0-dev.20260929), into `og`:

- `og check` runs it as a pinned child process (`crates/og-cli/src/check.rs`)
  found by `OG_TSRS`, the engine package's `optionalDependencies`
  (`@tsrs/<platform>`), or `PATH` (`scripts/install-tools.sh`, SRI-checked);
  diagnostics are values (`--json`: one `check` event). `og run`, `og test`
  and `og build` check before they bundle and stop on a type error
  (`--no-check`; never for `--publish`); `og dev` checks each save beside
  the rebuild and reports it on the reload without refusing it. `og doctor`
  reports tsrs against its pin.
- Measured (`tests/evidence/tsrs/wave-0.md`): at 4 checker threads tsrs takes
  20–47 % of tsgo 7.0.2's wall clock on every config, 23 ms to start, 32.5 MB
  installed; a 22-file corpus of game-code errors (`scripts/tsrs-corpus/`)
  gave the same 23 diagnostics, messages byte for byte. In `og dev` the check
  lands in 33–48 ms and does not move the rebuild
  (`tests/evidence/tsrs/wave-2.md`).
- Found on the way: ten of the eleven templates (fx-showcase, ui-kit, the
  eight archetypes) had never been checked as the projects they scaffold;
  their playtest helpers' `node:` imports and globals were 10–16 errors each
  under `types: []`, tsgo and tsrs alike. Each now carries a `host.d.ts`.
- **The switch (owner's delegation, 2026-10-22).** The package proposed
  switching the repository's authority from tsgo to tsrs after two wave
  ends with no difference. Decided instead to switch at once: the first
  comparison was already complete (19 configs and the corpus, no miss, no
  extra, no message difference, against tsgo 7.0.2 and against the
  7.1.0-dev tsrs ports), and the comparison stays as the wave end's
  `conformance` stage, so a regression in a later tsrs is caught where a
  version moves. Since then `packages/engine`'s `typecheck` is `og check
  --tool tsrs` over its three configs (`scripts/typecheck.ts`, after
  `build.sh`; 0.25–0.28 s against 1.16–1.26 s for the three `bun x tsc`),
  og-idl's declaration test runs tsrs, and no template, example or package
  depends on `typescript`. tsgo remains only as `TSGO_VERSION` in check.rs:
  the comparison and the fallback on Windows and Intel macOS, which tsrs
  does not build for.
- Pins (docs/ci.md §Toolchain): `TSRS_VERSION` and the five SRIs in
  `scripts/install-tools.sh`, copied by the engine's `optionalDependencies`;
  they move together at a wave's end with a tsrs release, after
  `scripts/tsrs-diff.sh` passes on it. The engine image keeps the optional
  dependencies (docs/platform-asks.md #13).
- Owed to the merging session: the functional lane and an `og dev` reload
  measurement on Metal with this og (the orb where it was built has only
  lavapipe, whose first frame takes 650 ms and over).

### Toolchain 2026-10

- **2026-10-19** (c1290aa..HEAD): Rust 1.97.0 to 1.99.1 with every crate
  on edition 2024 (clippy's new as_chunks and let-chain lints applied),
  binaryen 123 to 133 plus `--low-memory-unused`, wasm-bindgen 0.2.127 to
  0.2.129, TypeScript 5.9.3 to 7.0.2 (0.26 s instead of 1.2 s for the
  engine typecheck), cargo-nextest, cargo-zigbuild and zig newer locally.
  The shipping wasm is 1.0 % smaller (2,518,068 bytes brotli, 60 % of the
  budget); no reference frame or determinism number moved. The template's
  LFS attributes are stored as `gitattributes` and renamed by the
  scaffolder, so this repository's checkouts are clean again; fx-showcase's
  references on all four tiers carry the one-line badge of 00653bf. wgpu
  31 is not published yet (§Owed row 35). Versions: `docs/ci.md` §Toolchain; evidence
  `tests/evidence/toolchain-2026-10.md`.

### Build speed 2026-10

- **2026-10-19** (edd9736..HEAD): the inner loop, measured with another
  agent on the machine. The full lane went from 165–320 s to 103 s with
  `--all` and 41 s after a template-only change, the quick lane from
  83–154 s to 19–26 s, `cargo nextest run --workspace` from 112 s to
  20–29 s. The causes were not the compiler: 252,881 stale object files
  in `target/debug/deps` that every Metal device open read (1.4 s per GPU
  test; the dev profile now has no debuginfo), og-cook's tests cooking the
  template once per process and one at a time (now once per test build,
  under a file lock, in parallel), an O(n²) vertex search, and nextest
  starting the longest work last. og-api is three crates (og-idl, og-api,
  og-runtime), so a doc-only IDL edit rebuilds nothing; the gate is
  path-aware (GPU tests, wasm steps, drift, web-reference skipped when
  their inputs did not change; `--all` and CI run everything), wasm-opt
  and brotli reuse results by content, sccache is the wrapper in
  `.cargo/config.toml` (a fresh worktree's test build about 70 s) and
  parallel rounds reuse `scripts/worktree.sh` worktrees. No reference frame
  was re-recorded. Rules: `docs/ci.md` §Lanes, AGENTS.md §Build and gate
  discipline; evidence `tests/evidence/build-speed-2026-10.md`.

### Status 2026-10-18, E4 round

- **2D and game feel** (b6da6a1..4e63a59, branch `e4-2d-feel`): `og.TimeScale`
  in every world (`og.time.setScale`, `og.time.hitStop`; physics, animation,
  flipbooks and scaled tweens step by `scaledDt`, systems, the UI and the
  probe keep ticking); og-render's 2D batch sorted by layer then y, with
  nearest textures, nine-slice, chunked tile layers, text outlines and
  layered trails; `og.Sprite` (y sorting, flips, flash, blob shadow,
  nine-slice) and `og.Afterimage`; og-anim flipbooks (grid, named or
  Aseprite / TexturePacker atlas sheets, clips with fps, loop / pingPong /
  once, speed, `og.FlipbookEvent`) and `texture { kind pixel }`; the new
  og-tween (every numeric field, 22 easings, delay, loops, yoyo, sequence
  and parallel, `og.TweenCompleted`, squash and flash presets) and
  og-tilemap (tilesets with collision and animated tiles, 47-blob and
  16-bitmask auto-tiling, chunked layers with parallax and repeat, merged
  static colliders, deterministic `setTile` edits that re-tile their
  neighbours) with Tiled `.tmj` and LDtk `.ldtk` importers in og-cook;
  camera feel (trauma shake, punch, kick, Follow2D dead zone, look-ahead and
  per-axis damping, `og.CameraZone`, `og.CameraRail`; frame-only) and
  `og.render.floatingText`. Every piece has a state block, so snapshots
  replay. The archetypes `runner` and `match3` are rebuilt on it and
  `topdown` (Tomb Room) is new; each has frames and replay playtests with
  `metal` and `vulkan-cpu` references (topdown also `webgpu` and `webgl2`
  through the generalised web reference runner). A 256×256 four-layer map
  draws in 0.22 ms at 1280×720 on the M4 Pro (budget 1 ms). Fixed on the
  way: a scene component's omitted fields cooked as zero instead of the IDL
  default, presets started with a hit stop showing nothing until it ended,
  and a sheet's `filter linear` ignored on a pixel texture. Knowledge: the
  og-feel skill and og-game's 2D and feel sections. Evidence
  `tests/evidence/E4/`. Owed: §Owed rows 4–6 and 39.

### Status 2026-10-18, E2 round

- **Render and Runtime** (96adafe..393c2d6): every E1 effect is declared
  from TypeScript on both hosts (`og.PostEffects`, `og.Lighting2D`,
  `og.Light2D`, `og.Occluder2D`, `og.Decal`, `og.ParticleSystem`,
  `og.Trail`, `og.Beam`, `og.render.ribbon`, `og.load.lut`,
  `og.load.emitter`, sprite and material effect fields; off by default);
  og-render's screen-space overlay pass with per-pixel clips, which og-ui
  now draws through (no camera plane, no CPU clipping, sharp under TAA);
  `og.Pointers` (five pointers) and `og.Gamepad` (four pads) in og-input
  through the input ring, read by og-ui's touch controls and focus
  navigation; `templates/fx-showcase` with five stations and references on
  `metal`, `vulkan-cpu`, `webgpu` and `webgl2` (the browser tiers from
  `make web-reference`, each within og-tests' tolerance of `metal`,
  docs/ci.md §Browser reference tiers). Fixed on the way: immediate-mode
  sprites and text outliving their tick, effect passes that aliased the
  prepass textures they bind, dissolved pixels drawn black on Metal, the
  overlay blending encoded sRGB on the WebGPU canvas, three WebGL2 defects
  (TAA's depth read, 18 of 16 GL texture slots in the lit skinned
  pipeline, the local shadow map created as a cube-map array) and the
  Worker's probe call trapping the wasm on a non-string argument.
  Evidence `tests/evidence/E2/`. Owed: §Owed rows 20–22.

### Status 2026-10-18, E1 rounds

After M7 (the platform cutover, `docs/m7-platform-notes.md`) the E1 rounds
ran as four parallel streams on one tree, gated by `scripts/check.sh` end
to end (wasm 2,368,695 bytes brotli, 56% of the 4 MB budget; 1,099,937 at
M6: og-anim, og-ui, og-physics and the effects are the growth).

- **Animation** (0790651): og-anim and the cook's skin/clip/morph import;
  the `anim` IDL module (`og.load.animation`, `og.anim`) on both hosts
  with the walker posed identically on the native addon and in wasm
  (`packages/engine/tests/anim.test.ts`). Evidence `tests/evidence/E1/anim-*`.
- **UI kit**: about thirty widgets, focus navigation, motion, breakpoints,
  theme variants and UI state format 2 in og-ui; nineteen screen templates
  and the toast service in `og.ui` (103 functions); the `templates/ui-kit`
  gallery with 42 metal references at desktop and phone sizes and a
  two-host hash match in Chrome (WebGPU); the `og-ui` skill. Evidence
  `tests/evidence/E1/ui-*`. Owed then: the og-render overlay pass with
  per-instance clips, a gamepad resource and multi-touch in og-input (all
  delivered in E2, above).
- **Render effects**: AO, outlines, depth of field, motion blur, god rays,
  LUT grading, vignette, aberration, grain, flash, camera shake, decals,
  deterministic GPU/CPU particles, trails, ribbons and beams, 2D lights
  with soft shadows and glow, per-sprite effects, GPU skinning and
  blendshapes, toon/rim/dissolve (`crates/og-render/README.md` §E1
  effects). Fourteen reference scenes on `metal` and `vulkan-cpu`, 25
  effects tests, features-off references byte-identical. Costs
  `tests/evidence/E1/vfx-feature-costs.md`. Owed: the overlay pass (delivered in E2) and
  browser runs of the effect scenes (§Owed row 22).
- **Web export** (`docs/web-export.md`): project.ogt's `publish` block,
  `og-build.json` with every file's bytes and sha256, `_headers`,
  `origingame.runtime.json`, the PWA manifest and service worker, byte-
  deterministic output; the shell's splash, gate, error card and install
  hint; the og-probe wire for Stage and Player parents (request ring when
  isolated, postMessage otherwise) and the Player protocol 2; gamepad
  input; `og run` reports a dying game process at once; `og doctor`
  outside a project. Browser evidence `tests/evidence/E1/web-*`.
- **Still pending**: §Owed rows 38 and 40.

### Status 2026-10-18, M6

The M6 crates are committed (8d160fe render, 4fb20c7 ui, 55ac8a2 audio,
5c79c75 tools, d214078 ECS state blocks, 49ed935 physics) and
`scripts/check.sh` passes end to end at 49ed935 (all nine steps; wasm
1,099,937 bytes brotli of the 4 MB budget). What each crate's README owed
then has since landed or is in §Owed: the overlay pass and the `ui` module
in the IDL (E1, E2), og-audio's host wiring, effects and `listen` (E6),
og-physics in the IDL with its controllers (E5), the depth prepass (E1's
effects), the archetypes (E4) and the template's playtest starting on its
Start button (E6); still open are §Owed rows 30–34 and the mobile
measurements, row 40 (§5: scheduled for week 9, they need devices).

## 3. The first two weeks, concretely

Week 1 (M0, integrator + all four):

1. Workspace: create the crates with `lib.rs` stubs and the dependency graph
   of design §Repository layout; `Cargo.toml` pins (wgpu 30, glam 0.34,
   rapier 0.2x, gltf 1.4, meshopt, basis-universal, naga); `rust-toolchain.toml`.
2. CI (GitHub Actions): `cargo fmt --check`, `clippy -D warnings`, `cargo
   test`, wasm lib build + `wasm-opt` + brotli size gate (fail above 4 MB),
   generated-file drift check, `bun test` for the TypeScript packages.
3. `tests/`: the gfx harness (render a scene headless, compare to a committed
   PNG with a per-pixel tolerance and an RMSE threshold, write the diff on
   failure), the perf harness (frame-time baseline files with tolerance).
4. Move the spike renderer into `og-render` behind `Renderer::new(&RenderDesc)`
   and `render(&World)`; keep the spike's reference frame as the first gfx test.
5. Write `api.ogt` v1 as text (Core) while Render and Assets start their
   crates; the integrator reviews it against the design's §3.2 example.

Week 2 (M1 begins, M2 generators begin):

1. Core: the `Reflect` derive and schema JSON; the `.ogt` grammar and parser
   with round-trip tests on the design's examples.
2. Core: archetype storage and queries with the 10k-churn test.
3. Tools: the IDL parser and the first generator (`.d.ts`), driven by
   `api.ogt`; the `og` CLI skeleton (`og --version`, `og doctor`).
4. Render: material assets from reflected structs; automatic instancing keyed
   by (mesh, material); the sky pass and mip generation.
5. Assets: glTF → meshopt-encoded vertex streams as the first `.ogb` payload.

## 4. How the work is run

- Each workstream is one long-lived agent thread on its own crates with the
  IDL and this document as its brief; cross-crate changes go through the
  integrator's thread. The platform's own thread family is the natural tool
  for this once the repository is registered with Agent Service; until then
  the threads run from the developer's machine.
- Weekly: integrator merges, runs the full gfx and perf suites, updates this
  document's milestone table with evidence links, and publishes a
  `0.0.x` of `@origingame/engine` to the platform's internal registry so the
  M7 integration can start against real packages early.
- Every milestone's evidence is a file in the repository (`tests/evidence/
  M<n>/`), never a chat message.
- The platform repository changes nothing until M7, except
  `goals/origin-engine/plan.md` recording progress. The Godot distribution
  and the Three.js substrate keep shipping meanwhile.

## 5. Risks specific to phase 1

| Risk | Mitigation |
|---|---|
| The IDL generators become the bottleneck for every workstream | the `.d.ts` generator ships in week 2 and the two host generators by week 4; until then hosts are hand-written against the IDL and replaced, not extended |
| Renderer scope creep (every feature in §2.6 at once) | M3's 12 reference scenes are the list; anything not in a scene waits for phase 2 |
| rapier's wasm size pushes past the budget | measured in M6 with the size gate; the fallback is a feature-gated physics crate loaded only by projects that declare physics in `project.ogt` |
| Basis encoder build complexity in the cook | `basis-universal` crate first; if its build is fragile on CI, `toktx` as an external tool found by `og doctor` |
| Mobile numbers slip again for lack of devices | M6 names them as evidence; the integrator schedules the device run in week 9, not week 13 |
