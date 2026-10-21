# What the web found

The fourth dogfood round, and the first of the web path, which the three
native dogfood games never exercised. Each of
[`lantern-vale`](lantern-vale) (3D), [`hollowbell`](hollowbell) (2D) and
[`kestrel-stair`](kestrel-stair) (3D), and `templates/ui-kit` (2D), was
published as its author would publish it (`og build --publish`, the `og`
binary, the project's own `profile`: the 2D games got the 2D publish and
lanes modules, the 3D ones the 3D), served the way the platform opens a
release (the top-level page on its play origin, `window.__OG_GAME` in its
entry, a stand-in play origin answering Player protocol 2 over HTTP at
`/__player/v2/…` with a store and a leaderboard, the build's own `_headers`
so the page is cross-origin isolated and its service worker installs), and
played in headless Chrome on both tiers (WebGPU, and the WebGL2 fallback
through the shell's `?webgl=1`). Nothing in the engine was changed; the
harness carries every workaround and names its finding.

Engine 7526ff3c (wave 11's end), `OG_RELEASE=1 sh scripts/build.sh` in
lane-1 (release `og`, the shipping `wasm` profile, `wasm-opt -Oz`; 15 min
with other streams building). Chrome for Testing 150.0.7871.115 headless
(`--use-angle=metal`), an Apple M-series Mac with 12 cores, 1280×720 at
DPR 1, localhost. Ratings as in the games' FINDINGS: **blocker** (a
documented path that does not work on the web), **annoyance** (costs real
time, or silently wrong), **nit** (small, cosmetic, docs only).

| | Blocker | Annoyance | Nit | Total |
|---|---|---|---|---|
| Count | 0 | 7 | 5 | 12 |

Top five: #1, #2, #4, #3, #6.

## How it was played

`packages/engine/tests/web/dogfood.ts` (one game, one tier a run; Chrome
one at a time):

1. `og build <project> --publish --out packages/engine/dist/dogfood/<id>/publish`,
   then every file's bytes and brotli `-q 11`.
2. The page opened at `/play/<game>/<release>/` with an init script that
   timestamps every node of the accessibility mirror as it appears; the
   first frame is the shell's `og:first-frame` mark, the title the first
   time the title's own node (Lantern Vale's `play`, Hollowbell's `new`,
   Kestrel's `new`, the kit's "UI kit") is mirrored.
3. A real click on the play gate; then each press is a real mouse click at
   the centre of the control's mirrored element (found by its og-ui path
   or name), and keys are CDP key events: start a run (Set out; New game
   then Begin; New journey, Enter and Skip twice; the kit's platformer),
   600 ticks of play holding the game's movement keys, the game's own quit
   (Escape, Quit), the leaderboard screen where the game reads one
   (Kestrel's Records), the settings with the language switched and saved
   (简体中文, 日本語), a reload under the service worker, the gate again,
   and Back, which must report `end` to the play origin.
4. Measured on the page: `__runtime_probe__.hello()` (tier), `.lanes`, the
   Workers (CDP targets), `__og_timings__` over the 600 ticks (each played
   frame's engine work and display interval), `state().timings` sampled
   every 100 ms (step and render), the browser's processes' resident memory
   at the end, every console line of the page, its Workers and the
   browser's log, every call and report the play origin received, and the
   mirror and Chrome's own accessibility tree on the title.
5. `--frames`: the game's frames playtest (`tests/frames.og.ts`; the kit's
   `frames-desktop.og.ts`), bundled and imported as `og test` does, its
   commands without the screenshots, `quit` and dumps, run by
   `tests/web/reference.ts` on the dev export (one tick and one render per
   frame, as `og run` drives the native host), each screenshot at its tick
   compared with the game's **metal** reference by web-reference's measure
   (an outlier is a channel off by more than 32/255; the bar is 0.5 %
   outliers and RMSE 0.01), then 600 more frames timed (`--time 600`). A
   publish build cannot run a script (#6), so this is the dev export of the
   same project and engine.
6. Two variants of the play run: `--no-isolation` (a host without the
   build's COOP/COEP: the single-threaded engine) and `--throttle 20` (every
   file through one shared 20 Mbit/s link, charged at its brotli size, 40 ms
   before each response; DevTools' own network emulation does not reach the
   engine Worker's fetches, which are almost every byte).

## Measurements

### The published package

Brotli `-q 11`. A page downloads one engine variant (lanes when isolated,
else single), the shell, the Basis transcoder and the cooked first group;
the service worker precaches the same, and the icons.

| Game | Variant | Files | Package raw / brotli | Engine wasm, single (raw / brotli) | Engine wasm, lanes | Cooked (files, raw / brotli) | of which music (chunks, raw) | Shell (index, play, game, worklet, sw) | Basis | Boot download (lanes page, brotli) |
|---|---|---|---|---|---|---|---|---|---|---|
| Lantern Vale | 3d | 131 | 30.28 MB / 11.45 MB | 11,222,583 / 3,090,955 | 11,483,131 / 3,129,769 | 110, 5.98 / 4.83 MB | 60, 4.67 MB | 348,506 / 82,206 | 598,206 / 221,302 | 8.29 MB |
| Hollowbell | 2d | 125 | 27.31 MB / 9.76 MB | 10,624,085 / 2,922,956 | 10,881,490 / 2,961,947 | 104, 4.19 / 3.47 MB | 21, 3.04 MB | 369,457 / 87,903 | 598,206 / 221,302 | 6.76 MB |
| Kestrel Stair | 3d | 132 | 34.23 MB / 13.55 MB | 11,222,583 / 3,090,955 | 11,483,131 / 3,129,769 | 111, 9.90 / 6.93 MB | 40, 6.21 MB | 383,032 / 91,857 | 598,206 / 221,302 | 10.39 MB |
| UI kit | 2d | 77 | 33.80 MB / 14.54 MB | 10,624,085 / 2,922,956 | 10,881,490 / 2,961,947 | 56, 10.72 / 8.26 MB | — | 346,521 / 81,645 | 598,206 / 221,302 | 11.55 MB |

The glue is 287,336 B (25.7 kB brotli) single and 295,958 B (26.9 kB)
lanes, plus wasm-bindgen-rayon's 2,947 B `workerHelpers`. Every wasm is
under the 4 MiB brotli budget (the largest, the 3D lanes module, 74.6 %).
`og build --publish` printed no warning for any project, chose `3d`/`2d`
right, shipped both lanes modules and read the 7 player members back from
the copied wasm.

### Playing it (1280×720, 600 ticks of play)

First frame and title are ms from navigation on localhost; the title is on
the first frame in every game (the shell primes one tick behind the gate).
Step and render are the probe's (CPU; render is the encoding, not the
GPU), sampled; work is the Worker's per played frame; interval the display
interval (Chrome headless ran its display at 120 Hz, so p50 is 8.3 ms).
Frames: tick-matched shots of the frames playtest on the dev export, against
metal; "web frame" is `reference.ts --time 600` on the single-threaded dev
export (one tick and one render per call; median / p95, ms). Memory is the
resident memory of Chrome's renderer process (the page, the engine Worker
and the pool) and GPU process at the end, after a reload (#9).

| Game | Tier | First frame | Title | Lanes (Workers) | Step p50 / p95 | Render p50 / p95 | Work p50 / p95 | Interval p95 | Ticks/s | Web frame | Memory, renderer / GPU | Console | Frames vs metal (worst RMSE, outliers) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Lantern Vale | WebGPU | 547 | 547 | on, 8 (+ engine, + sw) | 1.34 / 1.57 | 0.55 / 0.69 | 1.65 / 2.24 | 8.3 | 60.0 | 2.70 / 3.94 | 742 / 208 MB | 16 W (#8) | 5 / 7 (title 0.0314, start 0.0358: #5) |
| Lantern Vale | WebGL2 | 1,742 | 1,741 | on, 8 | 1.46 / 1.81 | 1.06 / 1.22 | 2.58 / 4.00 | 28.4 | 58.7 | 3.74 / 8.05 | 750 / 194 MB | 16 W | 2 / 7 (start 0.0378; lantern 0.0189: #3) |
| Hollowbell | WebGPU | 533 | 533 | on, 8 | 1.44 / 1.62 | 0.71 / 0.79 | 1.98 / 2.39 | 16.7 | 59.9 | 2.62 / 3.13 | 752 / 171 MB | 16 W, 1 a11y W (#7) | **9 / 9, every frame identical (RMSE 0)** |
| Hollowbell | WebGL2 | 1,075 | 1,075 | on, 8 | 1.51 / 1.73 | 0.97 / 1.12 | 2.31 / 2.82 | 16.7 | 60.0 | 2.93 / 3.42 | 750 / 186 MB | 16 W, 1 a11y W | **9 / 9, identical** |
| Kestrel Stair | WebGPU | 752 | 752 | on, 8 | 5.04 / 6.79 | 0.68 / 0.83 | 4.73 / 7.94 | 16.7 | 59.9 | 10.35 / 11.26 | 672 / 168 MB | 16 W | 9 / 9 (strike 0.0046, 0.019 %) |
| Kestrel Stair | WebGL2 | 1,368 | 1,367 | on, 8 | 5.85 / 7.48 | 1.34 / 1.64 | 5.83 / 8.95 | 8.3 | 60.1 | 10.36 / 11.36 | 792 / 196 MB | 16 W | 7 / 9 (crypt 0.0456, 7.56 %; bag 0.0274: #4) |
| UI kit | WebGPU | 529 | 529 | on, 8 | 0.90 / 1.02 | 0.30 / 0.37 | 1.10 / 1.40 | 16.7 | 59.8 | 1.06 / 1.68 | 802 / 189 MB | 16 W | 22 / 22 (widgets 0.0086, 0.159 %) |
| UI kit | WebGL2 | 1,092 | 1,091 | on, 8 | 0.93 / 1.11 | 0.43 / 0.53 | 1.12 / 1.61 | 8.3 | 60.0 | 1.25 / 1.83 | 1,143 / 237 MB | 16 W | 22 / 22 (widgets 0.0086) |

W: console warnings; no page error, no exception, no failed request of the
build (the one 404 is the Portal stand-in page's `/favicon.ico`). Every
texture reached the renderer as BC7 on both tiers.

Beside the table:

- **Lanes or not** (`--no-isolation`, WebGPU): the single-threaded engine
  starts sooner (first frame 438 ms against 533 for Hollowbell, 566 against
  752 for Kestrel: the lanes module and its pool cost 100 to 190 ms) and
  steps slower where there is work to spread (Kestrel's step 8.1 / 8.9 ms
  against 5.0 / 6.8; Hollowbell's 1.4 / 1.7 either way). Both pages play
  and say why lanes are off (`the page is not cross-origin isolated
  (COOP/COEP)`).
- **A real link** (`--throttle 20`, WebGPU): first frame 2,830 ms (Lantern
  Vale), 3,109 (Hollowbell), 3,474 (Kestrel), 5,737 (the kit): every one
  past the design's 2.0 s budget, and the music is about half of it (#2).
  The reload under the service worker: 264 to 298 ms there (290 to 558 ms unthrottled on WebGPU).
- **The platform**: the play origin received, per game, the first tick's
  reads (`og.settings`; Hollowbell `save`, `best`; Kestrel `run`,
  `progress`), each write (Hollowbell's lantern `save` and settings,
  Kestrel's `progress` and settings, the kit's settings), Kestrel's
  `leaderboard.top { n: 10, board: "ascent" }` from Records (answered with
  three stand-in rows), and the reports `start`, `ready`, `sample`, then
  `end` on Back (with `keepalive`). The saved language came back after the
  reload in the three games (#1 for when), never in the kit (#10).
- **The a11y tree** on the title: Lantern Vale 5 nodes (heading, 2 texts,
  2 buttons), Hollowbell 6, Kestrel 7, the kit 32 (22 texts, 10 options of
  the gallery's list); no node unnamed; Chrome's own tree reads the shell's
  Back and Play and every title button by name. Every press of the run was
  found by its mirrored element and landed as a real click on the canvas.
- **The title as the page shows it**, the engine's own capture against the
  metal title: behind the gate (tick 1) Lantern Vale 0.155, Hollowbell
  0.178, Kestrel 0.119 RMSE (#1); after the tap (ticks 94 to 121, the
  reference's 60 or 72) 0.104, 0.039, 0.019, and the kit 0.

## Findings

1. **The first screen behind the play gate is the game's tick 1, before
   anything the platform answered** — annoyance. The shell primes one tick
   behind the gate and holds the clock (docs/web-export.md §The shell),
   and og.platform's answers are applied "at the start of the next tick",
   so behind the gate a returning player sees the defaults: the title in
   English after the settings were saved in 简体中文 or 日本語, Hollowbell's
   Continue disabled, and the screen's own transition at its first frame
   (Hollowbell's menu panel at low opacity over a lit scene). The tap then
   flips the title into the saved language. In all three games, both tiers;
   the store had answered `og.settings` long before the tap. Repro: `bun
   tests/web/dogfood.ts --project examples/hollowbell` in
   `packages/engine`, then compare
   `dist/dogfood/hollowbell/webgpu/reloaded-gate.png` with
   `reloaded-played.png` (`play.reload.translatedBehindGate` false,
   `translated` true). Where the docs misled: og-platform-sdk §3 ("Read
   saves on the first tick and show defaults until they answer; never block
   the title on the platform") and web-export's "the first screen is drawn
   behind it" together read as "the title behind the gate is the game's
   title"; neither says the answers wait for the tap. Workaround: none from
   game code (no tick runs behind the gate); the harness waits for the
   gate's tap before it checks the language.
2. **Chunked music is all fetched before the first frame** — annoyance.
   The boot fetches the first group, which is everything no streaming group
   holds, and the cook refuses a chunked sound in a group (og-build §7), so
   every chunk of the music arrives before the title: 4.67 MB of Lantern
   Vale's 8.29 MB boot download, 3.04 of Hollowbell's 6.76, 6.21 of
   Kestrel's 10.39 (QOA barely compresses: 47 to 56 % of the brotli bytes).
   At 20 Mbit/s the first frame is 2.8 to 3.5 s for the three games (5.7 s
   for the kit, whose 3.4 MB Chinese font is in its first group), against
   the 2.0 s budget. Repro: `bun tests/web/dogfood.ts --project
   examples/kestrel-stair --no-build --throttle 20`; `ls
   dist/dogfood/kestrel-stair/publish/cooked/music | wc -l` (40 chunks).
   Where the docs misled: og-build §7, "Big games: what the cook packs and
   the page fetches", lists `sound { stream true }` as a size knob beside
   streaming groups; nothing says the page still fetches every chunk at
   boot. Workaround: none (the games keep their music).
3. **The WebGL2 fallback draws Lantern Vale's ink and sparks differently**
   — annoyance. Five of seven frames miss metal on WebGL2 (RMSE 0.0172 to
   0.0378, against 0.0052 to 0.0072 on WebGPU for the same gameplay shots):
   the diff is the silhouettes of the voxel models (trees, the hero, the
   slime, its bubble) drawn as heavier dark lines (the anime cel look's
   `Stylise` ink) and the lantern's burst. Hollowbell and the kit match
   to the pixel on WebGL2. Repro: `bun tests/web/dogfood.ts --project
   examples/lantern-vale --no-build --frames --webgl`, then
   `dist/dogfood/lantern-vale/webgl2/frames/webgl2/lantern.diff.rgba`
   (`magick -size 1280x720 -depth 8 rgba:lantern.diff.rgba diff.png`).
   Where the docs misled: web-export says of the fallback that "both give
   the same frame" (of culling); the look's ink is not said to differ.
   Workaround: none; reported as numbers.
4. **WebGL2 lights Kestrel Stair's crypt as if it had no roof** —
   annoyance. The crypt frame on WebGL2 is 7.56 % outliers, RMSE 0.0456
   (WebGPU 0.001 %, 0.0021): the altar, the pillar's bricks and the floor
   take warm light where metal and WebGPU have them in the crypt's shade,
   and the bag frame over the same place is off by 0.0274. Every frame
   outside the crypt matches on WebGL2 (0.0012 to 0.0046). Not diagnosed
   (it reads like the sun's shadow under the roof missing). Repro: `bun
   tests/web/dogfood.ts --project examples/kestrel-stair --no-build
   --frames --webgl`; `crypt.png` beside `references/metal/crypt.png`.
   Workaround: none.
5. **Lantern Vale's first two seconds are brighter on the web, on both
   tiers** — annoyance. The title (tick 60) and the run's start (tick 132)
   are about 7/255 brighter everywhere (mean 98.7 against 91.3 red),
   RMSE 0.031 and 0.036 with almost no outliers (0.07 %); from tick 240
   the frames match (0.003 to 0.007). The look is AgX with auto exposure
   adapting 1 EV/s, so the web's exposure starts or adapts differently
   (perhaps its luminance readback lands later); the other games have no
   auto exposure and do not show it. Repro: as #3 without `--webgl`.
   Workaround: none.
6. **A publish build cannot run a playtest's script** — annoyance. The
   brief's "same scripted sequence the playtests use" has no path on what
   ships: the publish probe keeps hello, tick, state, pick, entity, input
   and capture (no `commands`, `advance` or `pause`; `autoplay=0` then
   holds a clock nothing can move), the game's own commands (`warp`,
   `give`, `language`) are command-channel actions, and `og run --device`,
   the documented way to play the publish build in a browser, takes no
   `--commands`. So the frames above are the dev export's, and on the
   published page the run was driven by real clicks on the controls the
   accessibility mirror locates. Where the docs say it: og-build §6 ("the
   command channel is gone"), correctly; what is missing is any scripted
   check of the bytes that ship. Workaround: `tests/web/dogfood.ts` (the
   mirror-driven play, and `--frames` through `reference.ts`).
7. **The hidden text-entry textarea is `aria-hidden` but takes focus** —
   annoyance (accessibility). Opening Hollowbell's name dialog (a
   `textField` focused) puts the DOM focus in the shell's hidden textarea
   (`src/web/text-input.ts`), which carries `aria-hidden="true"`; Chrome
   logs "Blocked aria-hidden on an element because its descendant retained
   focus … Element with focus: <textarea>" and a screen reader lands on an
   unlabelled text box instead of the mirrored field. Repro: `bun
   tests/web/dogfood.ts --project examples/hollowbell --no-build`, then
   `grep aria-hidden dist/dogfood/hollowbell/webgpu/console.txt`.
   Workaround: none.
8. **Every lanes page logs eight warnings it cannot fix** — nit. Each pool
   Worker prints "using deprecated parameters for the initialization
   function; pass a single object instead": wasm-bindgen-rayon's
   `workerHelpers.no-bundler.js` calls `pkg.default(data.module,
   data.memory)`, which wasm-bindgen 0.2.129's glue warns about. 16 per
   play run here (two loads), in every game and on both tiers; a game
   author's console is never clean. Repro: `grep -c "deprecated parameters"
   dist/dogfood/*/webgpu/console.txt`. Workaround: none.
9. **There is no way to read the engine's memory on the web** — nit
   (observability). The probe's hello, state and lanes report nothing of
   the wasm memory, and `performance.measureUserAgentSpecificMemory()`
   never resolved within 45 s in headless Chrome 150 (isolated, with
   `ForceEagerMeasureMemory`), so the table gives the renderer process's
   resident memory (672 to 1,143 MB: the page, the engine Worker, eight
   pool Workers and the reload, not the game's own figure). Repro:
   `play.memory` is null and `play.memoryApi` is "function" in any
   `dogfood.json`. Workaround: process RSS through
   `SystemInfo.getProcessInfo` and `ps`.
10. **The UI kit saves its settings and never reads them** — nit
    (template). Done on the kit's settings screen calls
    `og.platform.saveSettings` (the store took `og.settings` with
    `language: "zh-CN"`), but the kit makes no `loadSettings` call, so a
    reload comes back in English; the three games, which copied the
    pattern further, do read theirs. Repro: `bun tests/web/dogfood.ts
    --project templates/ui-kit --no-build` (`play.reload.storeReads`
    empty). Workaround: none needed for this round.
11. **The WebGL2 fallback's first frame is two to three times WebGPU's**
    — nit. 1,742 ms against 547 (Lantern Vale), 1,368 against 752
    (Kestrel), 1,075 / 1,092 against 533 / 529 (the 2D two), on
    localhost, the wasm and the bundle already local; `og:ready` alone
    moves from 381–540 to 847–935 ms. Inside the 2.0 s budget here, but
    it adds to #2 on a real link. Repro: any `--webgl` run's
    `play.timings`. Workaround: none.
12. **`og build`'s total is not what a player downloads** — nit (docs).
    `og build --publish` prints a total of every file (Hollowbell
    27,306,962 B, 9,732,067 brotli), which counts both engine variants (a
    page takes one), the Basis licence files, the icons and og-build.json;
    the page's download is 6.76 MB brotli. og-build §7 says "compare og
    cook's byte count before and after: the bundle is the player's
    download", which is the cooked part only. Repro: `og build
    examples/hollowbell --publish --out /tmp/hb` (its `total` line).
    Workaround: the table above sums the files a lanes page fetches.

## Fixed

The publish-testing round (branch `publish-testing`) fixed these four
(docs/phase-1.md records the rounds).

6. Fixed: `packages/engine/tests/web/published.ts` runs a script of
   input and capture steps (the playtests' syntax; `--playtest` takes a
   playtest file) on `og build --publish`'s output, served as the platform
   opens a release (`tests/web/play-origin.ts`, now shared with
   dogfood.ts), through the shipped probe's `tick`, `state` and `capture`
   and real input events, on a clock the harness gives the engine Worker
   (one tick a frame, held while a capture lands; the build unchanged); a
   command-channel step is refused. Hollowbell's title is tick-exact
   against metal (RMSE 0 on WebGPU and WebGL2), New game, a typed name
   and Begin reach the game, ArrowRight moves Wren; the kit's gallery is
   exact, and its saved 简体中文 comes back after a reload (#10); the
   console stays clean (#7, #8). Documented in og-build §6 and
   docs/web-export.md §The `og` commands around the export.
7. Fixed: the textarea is aria-hidden only while no field is focused;
   focused, it is named after the field's mirrored node (aria-label,
   placeholder, its text) and owned by the field's dialog, the mirror's
   own element for the field stepping aside (`A11yMirror.standIn`).
   `tests/web-text-entry.test.ts`; published.ts finds the focus in a named
   text box and no "Blocked aria-hidden" with Chrome's accessibility on.
8. Fixed: `scripts/rayon-worker.sh` (called by build.sh wherever a lanes
   module lands in dist) rewrites the helper's init to
   `pkg.default({ module_or_path, memory })`; the pinned versions stay.
   `tests/web-export.test.ts` checks the shipped scripts; a lanes page logs
   nothing (published.ts, dogfood.ts).
10. Fixed: the kit asks `og.platform.loadSettings` on its first tick and
    applies what it saved (`followSettings`); `tests/settings-saved.og.ts`
    proves it across a `store: "keep"` relaunch, published.ts across a
    reload. og-platform-sdk §3 and og-i18n §5 now say settings are a pair.

## What worked well

- **Publishing was one command and silent.** `og build --publish` built
  all four projects without a warning, cooked incrementally (143 to 234 ms
  where little had changed; the kit's first cook 4.7 s), picked the 3D or 2D modules from `profile`,
  shipped both lanes modules beside them, wrote `_headers`,
  `origingame.runtime.json`, the manifest and the service worker, and proved
  its probe trim from the copied wasm's own hello.
- **The page booted everywhere, fast.** First frame 529 to 752 ms on
  WebGPU from navigation, the title on that same frame, 264 to 558 ms on a
  reload under the service worker; cross-origin isolated from the build's
  own `_headers`, lanes on with eight Workers (`min(cores - 1, 8)`) on both
  tiers, and without COOP/COEP the single-threaded engine with the reason
  said. No page error, no Worker exception, no failed request of the build, in
  any page load.
- **The 2D path is exact.** Hollowbell's nine frames (the title, play, a
  strike, the talk, the shop, pause, the Chinese settings, the gloam, the
  dawn results) are byte-identical to metal on WebGPU and on WebGL2, and the
  kit's 22 desktop screens are within RMSE 0.0086 on both. Kestrel's nine
  match on WebGPU (worst 0.0046).
- **60 ticks a second with room to spare.** Over 600 ticks of play every
  game held 58.7 to 60.1 ticks/s with the engine's work per frame at p95
  1.4 to 8.95 ms (Kestrel, the heaviest, steps 5.0 ms median with its
  lanes: 38 % less than on one thread).
- **The platform round trip held.** Player protocol 2 over HTTP did what
  docs/web-export.md says: the hello, the store reads on the first tick and
  writes at the revision read, a leaderboard read from the game's Records
  screen, `start`, `ready`, `sample`, and `end` with keepalive on Back; the
  saved settings came back after a reload in every game that reads them.
- **The accessibility mirror is good enough to drive the game.** Every
  control on every title had a role and a name, Chrome's tree read them,
  and the harness found each control of each run (menus, dialogs, the
  settings' tabs and language choices, pause) by its og-ui path in the
  mirror and pressed it with a real click where it was drawn.

## Reproduce

From `/Volumes/app/origin-engine-lane-1` (or any checkout), one Chrome at a
time:

```sh
cd packages/engine
OG_RELEASE=1 sh scripts/build.sh                       # once: release og, every web module (15 min)
for p in examples/lantern-vale examples/hollowbell examples/kestrel-stair templates/ui-kit; do
  bun tests/web/dogfood.ts --project $p --frames             # publish, play on WebGPU, the frames
  bun tests/web/dogfood.ts --project $p --no-build --frames --webgl
done
bun tests/web/dogfood.ts --project examples/kestrel-stair --no-build --no-isolation   # single-threaded
bun tests/web/dogfood.ts --project examples/kestrel-stair --no-build --throttle 20    # 20 Mbit/s
```

Each run writes `dist/dogfood/<id>/<tier>[-single|-20mbit]/dogfood.json`
(everything above), `console.txt`, the page's screenshots, the engine's
captures and, with `--frames`, `frames/reference-<tier>.json` and the
shots with their diffs.
