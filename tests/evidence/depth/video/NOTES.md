# S3 `video`: evidence (docs/depth.md §3)

Branch `depth/video`. Renderer: lavapipe (`vulkan-cpu`) in a CPU-only
sandbox; ffmpeg 5.1.9 (libx264, libvpx-vp9, aac, libopus). The Metal
reference of the gfx test is left to the coordinator.

## What landed

| Commit | Change |
|---|---|
| 59c80245 | Every-frame capture without the probe's round trip: og-probe `Capture::frame`/`RawFrame`, `Probe::frame_rgba`, `render_snapshot`/`render_restore` (`probe/record.rs`, unit test); og-runtime `RenderBackend::read_frame` and `record.rs` (absent from publish builds); og-host-node `render.rs`. |
| 7771e44d | `og render`: `--commands`, `--timeline`, `--fps` (must divide the tick rate; the refusal names the valid ones), `--from/--to`, `--size`, `--out dir|.mp4|.webm`, `--frames png|jpg`, `--audio` (the capture mixer's WAV, muxed), `--contact-sheet <cols>` (time and tick under each cell), `--jobs N` (snapshot-restored ranges, concatenated by ffmpeg), `--json`, `frames.json` with per-frame sha256 and the audio's. `og doctor` reports ffmpeg; without it the frames are written instead. |
| 62fb4683 | `packages/engine/tests/og-render.test.ts`: two runs byte-identical (frames and audio), `--jobs 3` equals one job, a bad fps refused. |
| 644dbbe6 | og-cook `type video`: ffmpeg decodes into an `og.Video` blob, `<id>#frames` (KTX2 texture array, one layer per frame, at most 256) and `<id>#audio` (a sound, QOA 48 kHz). ffprobe plans the cook; a warning past 64 MiB of frames. |
| 55696f48 | Video in: IDL `module video` (`Video`, `VideoInfo`, component `VideoPlayer`), `og.load.video` / `og.load.videoInfo`; the `og.video` system picks the layer from the tick and keeps the sound in step; og-render `FrameTexture` so the current frame is a `Texture` for materials, `og.Sprite` and UI images. |
| 686c168e | `packages/engine/tests/og-video.test.ts` (fixture `tests/fixtures/video`): the gfx test, reference name `references/<tier>/video-90.png`. |
| baaf43d5 | `og doctor` reports ffprobe. |
| 34626b44 | og-game skill §18 "Video: footage in, videos out". |
| 64d92413 | docs/design.md §2.9 "The video producer". |
| 5246f700 | `templates/tools/render.sh` (the example) and `templates/tools/reviews.md`. |

## Frames read

**`game-contact-sheet-vulkan-cpu.png`**: `og render templates/game
--commands "0.2:touch_move:60,130;0.4:touch_start;0.5:touch_end;1:key_down:KeyW;2.5:key_up:KeyW"
--fps 30 --to 3 --size 640x360 --out clip.mp4 --audio --contact-sheet 6 --jobs 3`
(`game-frames.json` beside it). Eighteen cells, 0.000 s/t0 to 2.967 s/t178:

- t0: the 3D scene before the UI's first layout: grey floor, the ship on its
  legs, the wooden figures, the grey pillar and the gold crate in the air.
- t10, t20: the "Origin Engine" panel with "Press Start to begin" and the
  orange Start button.
- t32 onwards (after the touch at 0.4 s): the panel reads "Spinning 1.5
  rad/s" in orange with a grey "Started" button: the press landed.
- t62 to t116: the crate falls past the pillar, tumbling (tilted at t94 and
  t104), and comes to rest upright beside it by t116.
- t126 to t178: the crate at rest and the camera still. KeyW (1 s to
  2.5 s) moves nothing visible at this scale; the script's own effect is
  the Start press.

`clip.mp4`: ffprobe gives H.264 with 90 frames in 3.000 s and AAC 3.000 s.
frames.json: 90 frames, 78 distinct hashes (the still stretch at the end).

**`video-90-vulkan-cpu.png`**: the gfx test's frame at tick 90 of the video
fixture (ffmpeg testsrc2 at 192x108, 4 s, 30 Hz tick, the sprite filling the
view): the source's burned-in time `00:00:03.000` and frame number `90` are
readable over the colour bars, magnified with nearest filtering. Its RMSE
against the source's frames 89/90/91 is 0.140/0.103/0.134: frame 90 is the
nearest. A seek stops the sound for one tick and restarts it at startOffset
31/30 s.

## Numbers

- templates/game, 3 s at 640x360 and 30 fps: run 1 equals run 2 on all 90
  frame hashes and the audio hash; `--jobs 2` and `--jobs 3` give the same
  hashes as one job.
- mp4 (H.264 + AAC) and webm (VP9) both mux; `OG_FFMPEG=/nonexistent` writes
  frames only with a warning; `--fps 7` is refused, naming 6 and 10.
- Speed on lavapipe: 1280x720 at 2.55 frames/s in one job, 6.71 with
  `--jobs 4`; 640x360 at about 9 frames/s in one job, 19.4 with `--jobs 3`.
- Video cook: a 320x180 5 s clip at `maxSeconds 4`, `size 160x90`: 120
  frames, 681 KiB cooked, a 4.00 s sound.

## Shared files touched

- `crates/og-api/api.ogt`: `module video` appended at the end; `load.video`
  and `load.videoInfo` appended before `event AssetReady`. Generated `.d.ts`,
  bindings and wiki regenerated; `og api check` passes.
- `crates/og-cli/src/main.rs`: the `render` dispatch and its usage line
  (plus `render.rs`, `contact_sheet.rs`; `doctor.rs` ffmpeg and ffprobe).
- `crates/og-probe`: `hooks.rs` (`Capture::frame`, `RawFrame`), `probe.rs`
  (`mod record`), `lib.rs` exports.
- `crates/og-render`: the `lib.rs` mod/export line and `video.rs`; no
  frame-graph or WGSL change.
- `crates/og-runtime`: `engine.rs`, `extract.rs`, `assets.rs`
  (`texture_array_data`), `codec.rs`, `lib.rs`, new `record.rs`, `video.rs`.
- Both hosts' `host.rs`; og-cook `format.rs`, `runtime.rs`, `cook/mod.rs`.
- `packages/engine`: `src/env.d.ts`, `index.ts`, `runtime.ts`,
  `node/index.ts`, new `node/render.ts`.

## Left out

- `--timeline <name>` is sugar over `0:timeline:play:<name>` plus the
  duration read from the timeline's `.ogt`; it works once stream S2
  (`depth/timeline`) lands. The §3 acceptance run on `--timeline intro`
  waits for it.
- The Metal reference `video-90.png`: the coordinator's.
- No render of a video on a UI image or as a material map: the frame is a
  `Texture`, so both take it, but neither was rendered here.
- The web host's `Capture::frame` returns none: `og render` is native only.
- Several players of one video share one texture (the first player's frame).
