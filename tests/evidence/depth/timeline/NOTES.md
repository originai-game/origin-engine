# Depth S2 `timeline`: evidence

Branch `depth/timeline` (from main at 02241fbb). Everything here was run
in an orb on lavapipe (CPU only). Following docs/depth.md §0, this stream
did not run `scripts/check.sh`, `og test`, a lane, web-reference, or a
wasm or release build.

## What landed

| Commit | What |
|---|---|
| 6a42abe9 | `crates/og-timeline`: arc-length paths (catmullRom, bezier, linear; closed; tension); the `type timeline` model, parsed and written as canonical text; evaluation as a function of time alone; `.srt` and word-timestamp `.json` caption import |
| 10cf1818 | og-cook: `type timeline` cooked to `og.TimelineAsset` blobs (caption files imported and stamped, named sounds and models checked); `Bundle::timeline`; a scene's `path "<name>" { ... }` cooked into its blob |
| 7eba1bea | og-ui: `Ui::cover`, the screen-transition cover at an amount driven from outside |
| c1a85099 | og-api: the `path` and `timeline` modules, `og.load.timeline`, `og.CameraRail.path`; generated bindings and wiki |
| b244cf13 | og-runtime: the `og.path` and `og.timeline` systems, the `og.timeline` state block, the `timeline:<verb>` commands, scene paths and the CameraRail path; both hosts; the TS namespaces |
| 1e350211, 85206c9e | the Stop that `timeline:play` queues before its Play no longer cancels it; a stopped timeline evaluates nothing; `timeline-<name>` can be found with og.world.find; the runtime tests |
| ed5bc3a6, e902adbf | `templates/game`: `timelines/intro.ogt` and `narration.srt`, the playtest `tests/intro.og.ts` with reference frames, `tests/frames.ts` and `host.d.ts` (copied from the archetypes), og-cook's template output count |
| (this commit) | docs/design.md §2.7 "Timeline and paths", the og-game skill's "Cutscenes: timelines and paths", the template README, these notes |

**Exact seeking.** A timeline's time is a whole number of steps of
1/(rate × 1000) s. Each tick adds `round(speed × 1000)` steps, and a seek
sets `round(t × rate × 1000)`. Every track is evaluated at that time.

## Tests

- `cargo test -p og-timeline --features import`: 13 pass.
- `crates/og-cook/tests/timeline.rs`: 3 tests. Each was broken once in the
  code it covers and failed. The whole og-cook suite passes.
- `crates/og-runtime/tests/timeline.rs`:
  - `seeking_to_4s_hashes_as_playing_to_4s` compares a world played 240
    ticks with one sought to 4 s: the probe's world hash, og.Timeline,
    og.Camera and the box's Transform are equal. The seek runs only the
    retroactive event it jumps over. A mutant that accumulates time in f32
    (`written + dt`) fails it: time 3.9999969 instead of 4.0.
  - `playback_raises_markers_once_and_hands_the_camera_back_at_the_end`:
    the events arrive as Started, Marker "one", Marker "five", Ended, each
    once. The marker at 1 s arrives at the right tick. After the end, the
    camera's position, quaternion and fov equal what they were before.
  - `timeline_play_and_seek_from_the_command_channel`: `0:timeline:play:t`
    creates `timeline-t` and plays it, and `timeline:seek:5:t` moves it.
    This test fails without the Stop-before-Play fix.
- `cargo nextest run -p og-runtime -p og-ui -p og-cook`: 282 tests. After
  the template count in `og-cook/tests/template.rs` was updated from 15 to
  16 outputs, they all pass.
- `og api check`: clean. `bun run typecheck` in packages/engine: ok.
  `cargo clippy -D warnings` on og-timeline, og-cook, og-runtime and og-ui:
  clean.
- `og-host-web` was checked with a native `cargo check` only (no wasm
  build).

## Frames read (lavapipe, 1280×720, `og run templates/game --headless`)

The acceptance script `0:timeline:play:intro;4:screenshot;8:screenshot`
was run, plus 0.5, 9.5 and 10.5 s in a second run. The files are in this
directory.

- `intro-0.5.png`: the camera is at the left start of the "flyby" rail,
  high, looking at the yard's centre. Compared with the template's own
  view, it is lower-left and looks across the wall. The fade-in cover
  (key 0, mode in, 1 s) has dimmed the whole frame, menu included, to
  about half. No caption shows yet (the first cue starts at 0.8 s).
- `intro-4.png`: the camera has moved further along the rail, closer and
  to the right, and the dropship fills the centre. The caption "The
  trickster walks out to meet it." (the .srt's second cue, type step lg)
  is in a dark pill at the bottom centre. The trickster is mid-stride on
  the `walk` clip, between (2 0 0) and (0.5 0 1.5): the dump puts it at
  (0.734, 0, 1.266). There is no cover.
- `intro-8.png`: the camera is further right. Two more dropships are in
  the air, spawned by the event key `spawn_wave:2` at 5.5 s (the world has
  10 entities). The trickster stands at (0.5 0 1.5) after its `wave` clip.
  The caption reads "Here they come." (the third cue).
- `intro-9.5.png`: the camera is at the rail's end. The fade-out (key 9)
  has dimmed the frame to about half. No caption shows (the third cue
  ended at 8.8 s).
- `intro-10.5.png`: the timeline has ended (`time` 10, `playing` false).
  The camera is handed back to the template's own view, the same framing
  as the baseline. No cover or caption shows. The trickster stays where
  the transform track left it: transform values are held, not restored.

The playtest `templates/game/tests/intro.og.ts` names the reference frames
`references/<tier>/intro-4.png` and `intro-8.png` (compared with
`tests/frames.ts`). Its exact script was run through `og run`, and every
assertion on the dumps was checked by hand: the captions at 4 s and 8 s,
the bottom anchor (y 670), time 3.983 at 4 s, the trickster on its way,
10 entities at 8 s, the end at 10 s, and no caption after the end.

## For the coordinator

- **Metal references to record:** `templates/game/references/metal/intro-4.png`
  and `intro-8.png` (`OG_FX_CAPTURE=1 og test templates/game`). No webgl2
  references were made either.
- **Shared files touched:**
  - `crates/og-api/api.ogt`: the `path` and `timeline` modules are
    appended at the end of the file; `og.CameraRail` gains a last field
    `path`; `load` gains `timeline`. Nothing was reordered.
  - og-runtime's schedule (`engine.rs` `assemble`): the timeline events are
    registered from Rust types and skipped in the RECORDS loop. `og.path`
    then `og.timeline` are installed after `feel::install_front` (og.tween,
    og.flipbook) and before og.anim, giving the order ... og.tween,
    og.flipbook, og.path, og.timeline, og.anim, og.tilemap, og.camera ...
    `create_inner` calls `timeline::commit` and `register_commands`, and
    `destroy` calls `timeline::retire`.
  - `feel.rs`: `install_middle` takes `engine_id`, so og.camera's rail can
    read a path handle.
  - Small touches: `assets.rs` (scene paths are registered at spawn),
    `codec.rs` (the `Path` and `TimelineAsset` kinds), and `pub(crate)` on
    `modules::step`, `modules::anchor_of`, `feel::look_rotation` and
    `anim::load_animation`.
  - Both hosts' `host.rs` and `host_p2.rs`; `packages/engine/src`
    (`p2.ts`, `index.ts`, `runtime.ts`); `og-ui/src/juice.rs`.
  - `og-cli/src/main.rs` was not touched.
  - Template: `assets/timelines/`, `tests/intro.og.ts`, `tests/frames.ts`,
    `host.d.ts`, `tsconfig.json` (includes `host.d.ts`), `README.md`.
  - `og-cook/tests/template.rs`: the expected output count is now 16.
- **Naming decision:** the path desc is `og.CurveDesc`, not depth.md's
  `PathDesc`, because `og.nav` already declares `PathDesc`.

## Left out, or not verified

- Metal and webgl2 reference frames (left to the coordinator).
- Sound was not heard: the headless run uses the null audio device. The
  audio and music tracks spawn og.AudioSource voices, which the dumps show
  but no capture was taken of.
- Hot reload with a timeline playing was not exercised. Loaded timelines
  and paths are per-engine assets kept across a reload, and the playback
  state is a state block.
- og.render tube geometry from a path (the path module's doc says
  og.path.sample feeds it) belongs to S1 and is not wired here.
- At the end, transform and animation tracks leave their last values;
  only the camera is handed back. This matches the IDL doc.
