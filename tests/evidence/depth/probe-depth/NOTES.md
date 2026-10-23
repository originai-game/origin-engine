# S5 `probe-depth`: evidence

Stream S5 of `docs/depth.md` §5, on branch `depth/probe-depth`, rebased onto
main `24116264` (wave 1 merged). All runs are on this orb's lavapipe
(`tier vulkan-cpu`), at the default 1280×720 play surface.

## Commits (oldest first)

| commit | what |
|---|---|
| `21376885` | og-render: the entity-id pass on the CPU (`id_pass.rs`, new) |
| `8fdfb6ad` | og-probe: `describe`, `expect` (six operations, as actions, a program op and members), `expectations`, `save_scene` over a `SceneWriter`, `timings.systems`/`.passes` |
| `5c219a30` | og-runtime: the scene view (`describe.rs`, new): the id pass over the extracted frame; `pick` answers from it |
| `6f1f1a14` | IDL members and actions, generated outputs, hosts, `og run --json` expectations and exit 1, expect-only playtests |
| `dc1bf646` | templates/game: the smoke playtest as expect steps at 1280×720 |
| `27cd8cb1` | scene write-back: `og_cook::scene_text` (new), og-runtime `scene_save.rs` (new), the exact round-trip test |
| `642c8fae` | `og run --trace`: og-cli `trace.rs` (new), og-runtime `trace.rs` (new), og-render per-pass timing, `state().timings` filled |
| `17cea5f6` | diagnostics: og-cook `cook/stats.rs` (new), `og cook --report` → `og-out/cook-report.json`, og-cli `doctor_project.rs` (new) |
| `01f717b8` | design §3.3/§3.4, og-game §21 |

## Acceptance

### 1. describe on the first frame: player, floor, HUD; boxes contain the pick points

The template's tick-0 frame has no HUD, because its UI is built by
systems each tick and no tick has run yet. The first frame that shows the
game as played is tick 1. `describe-first-frame.json` is `og probe
describe` at tick 1 (paused there by `0.016:pause`). The id pass is
640×360, a quarter of the 1280×720 frame's pixels:

```text
order name      box (x, y, w, h)        point        occlusion
0     ground    0, 420, 1280, 300       639, 569     0.072
1     dropship  384, 248, 398, 204      583, 349     0
2     crate     276, 62, 142, 134       347, 129     0
3     wall      232, 338, 224, 220      343, 447     0.002
4     trickster 776, 358, 32, 170       791, 443     0
5     guard     178, 358, 48, 124       201, 419     0
ui Hud (column), Menu (panel), Title "Origin Engine", Status "Press Start to begin", Start (button, 44,116 63x36)
camera perspective fov 50 at (0.4, 1.8, 8) looking at (0.4, 1.8, 7)
```

`og probe pick` at each entity's point, on the same paused world:

```text
ground point (639,569) inside box: True; pick -> ground
dropship point (583,349) inside box: True; pick -> dropship
crate point (347,129) inside box: True; pick -> crate
wall point (343,447) inside box: True; pick -> wall
trickster point (791,443) inside box: True; pick -> trickster
guard point (201,419) inside box: True; pick -> guard
```

The template has no entity named "player". The ones the player acts on
are the crate (dropped on Start), trickster and guard. The floor is
`ground` and the HUD is `Hud`/`Menu`.

**Frame read: `first-frame.png`** (tick 1, `screenshot:full`, 1280×720).
The HUD panel is at the top left (24–234 × 24–168), with "Origin Engine",
"Press Start to begin" and the orange Start button at (44, 116, 63×36), as
`ui` says. The crate hangs at 276–418 × 62–196, the wall at 232–456 ×
338–558, the dropship spans 384–782, the guard mannequin is at x 178–226,
the trickster at x 776–808, and the floor starts at the horizon, y ≈ 420.
Every described box matches the pixels.

### 2. The playtest as expect actions, failing on a broken copy

`templates/game/tests/smoke.og.ts` is expect steps only: an event, two
state checks, the UI text, a world box, a collision event, a region colour
and two screen boxes. On the template, `og run --commands <its steps>`
gives 9 passed, 0 failed, exit 0. The playtest runner reports `PASS smoke
(9 checks)`.

The broken copy is `target/broken-game`: the template with `Drop` removed
from `systems`. It exits 1 with 6 passed and 3 failed
(`broken-copy-run.json`):

```text
FAIL tick 120 entity:crate:within:-3,0.4,-1,1,0.6,2 | observed [-2.4, 4, 0.5] | 'crate's Transform.position is outside the box
FAIL tick 120 event:og.CollisionStarted | observed 0 | 0 og.CollisionStarted event(s) were sent since the script began
FAIL tick 120 entity:crate:within:380,360,280,240 | observed {"height": 134, "width": 142, "x": 276, "y": 62} | 'crate's visible box leaves the expected box
```

**Frames read:**

- `smoke-tick120.png` (the template at tick 120). The menu reads
  "Spinning 1.5 rad/s" and "Started". The crate has landed tilted in front
  of the wall, inside 380–660 × 360–600. The mannequins reach out.
- `broken-tick120.png` (the broken copy at tick 120). The same menu
  state, but the crate still hangs at 276–418 × 62–196, exactly the
  observed box of the third failure.

### 3. A 300-tick trace, checked against the trace event schema

Command: `og run . --headless --ticks 300 --commands "0:pause;0:resume"
--trace trace.json`. The result is `trace-300.json.gz`, and
`trace-300-check.txt` holds the checker's output:

```text
events 10520 threads {1: 'engine', 2: 'render', 3: 'load', 4: 'cook'}
spans by (tid, cat) {(4, 'cook'): 1, (1, 'tick'): 300, (1, 'system'): 8400, (2, 'render'): 300, (2, 'pass'): 1500, (3, 'load'): 10}
ticks 300 first tick 1 last tick 300
overlapping (not nested) spans 0
errors 0 []
```

The checker (a one-off script, not committed) holds the trace to the Trace
Event Format's JSON object form, as Perfetto's importer reads it:

- `traceEvents` is a list;
- every `X` event has a name, `ts`, `dur` ≥ 0 and integer `pid`/`tid`;
- `thread_name` and `process_name` metadata have `args.name`;
- every span on a thread nests inside its parent.

The file was not opened in the Perfetto UI, because the orb has no
browser session for it.

## Scene write-back

- **Round-trip test:** `crates/og-cook/tests/scene_write_back.rs`. It
  cooks and loads, changes values (positions that are not short decimals,
  a bool, and a field the text lacks), rewrites, cooks and loads again.
  The result equals the changed `SceneDesc` exactly. Only lines 8, 12 and
  19 change, comments and order are kept, and a second save changes
  nothing. The unit tests are in `og_cook::scene_text` (5).
- **On the template (`save-scene.diff`):** after `teleport:trickster`,
  `set:marker.og.Transform.scale`, `set:ground.og.Transform.scale` and
  `set:Run.chimed`, `save_scene:saved.ogt` changed 5 lines. Saving again
  gave an identical file. `Run.chimed` is reported as "not written: a
  resource; scene text has no resources".
- The diff also shows `ground position 0 -0.1 0` and the trickster's
  quaternion `0.70710677`. These are values the running game holds (the
  template's code moves them), not artefacts. A write-back bakes whatever
  the world holds for the scene's entities.

## Diagnostics

- `cook-report.json` is `og cook --report` on templates/game: 13 assets,
  16620 LOD-0 triangles, no flags. Examples:
  - dropship: LODs 16512/8990/5223, 16870 vertices;
  - walker: 12 bones, clips idle/walk/run/wave/turn/blink;
  - stone textures: 512² in etc1s and uastc.
- `doctor-tight-budgets.json` comes from a copy with `budgets {
  triangles 10000 textureSize 256 bones 8 }`. `og doctor --project .
  --json` reports 4 over-budget errors (dropship triangles 16512, walker
  12 bones, two 512 textures) and exits 1. The template itself gives "ok"
  and exits 0.

## Shared files touched

- `crates/og-api/api.ogt`: `CoreAction` describe/expect/save_scene, the
  structs for describe/expect/save, `ProbeTimings.systems/passes`, and the
  `EngineProbe` members describe/expect/expectations/saveScene. The
  generated files were regenerated with `api gen` and `api gen --wiki`;
  `api check` passes. The wiki changes are line numbers only, except
  probe.md.
- **og-probe:**
  - `probe.rs`: actions, members, tuned fields, timing detail;
  - `rpc.rs` and `command.rs`;
  - `hooks.rs`: `SceneView`, `SceneWriter`, `SavedScene`;
  - new files `describe.rs` and `expect.rs`;
  - tests: `conformance.rs` and a new `expect.rs`.
- **og-runtime's schedule:** `probe.rs`'s before/after-tick hooks call
  `trace::before_tick`/`after_tick`, and `with_probe` sets the timing
  detail. `engine.rs` gains `scene_source` and `pass_ms`, plus the trace
  in `render()`. Also touched:
  - `extract.rs`: `RenderBackend::set_clock`/`pass_timings`, with defaults;
  - `extract.rs` and `feel.rs`: draw ids;
  - `assets.rs`: a load span around `blob()`;
  - `api.rs`.
- **og-render's frame graph:** `lib.rs` gains `Renderer::set_clock`,
  `pass_timings` and two fields. The pass loop takes a clock reading
  before and after each pass. Rendering is unchanged.
- **og-cli:**
  - `main.rs`: mods;
  - `run.rs`: `--trace`, `prepare_traced`;
  - `cook.rs`: `--report [out.json]`, `COOK_REPORT`, budgets;
  - `doctor.rs`: dispatch to `--project`;
  - new files `trace.rs` and `doctor_project.rs`.
- **og-cook:** `lib.rs`, `cook/mod.rs`, new files `scene_text.rs` and
  `cook/stats.rs`, and the test `tests/scene_write_back.rs`.
- **Hosts:** og-host-node and og-host-web `host.rs` (probe members) and
  `lib.rs` (backend clock and pass timings; node: `og_node_trace`,
  `og_node_trace_take`).
- **packages/engine:**
  - `src/runtime.ts`, `shell/page.ts`;
  - `src/node/run.ts`: `--trace`, expectations, exit 1;
  - `src/node/index.ts`, `src/node/playtest-run.ts`, `src/playtest.ts`.
- **Docs:** `docs/design.md` §2.5, §3.3, §3.4; og-game skill §21. The
  knowledge package's `bun run test` passes 42/42.

## Metal references owed

None. No gfx reference frame was added or moved. The id pass runs on the
CPU and has unit tests, not reference frames. The per-pass timing reads a
clock and draws nothing differently.

## Left out, and limits

- **Lanes in the trace.** "Lanes as threads" is read as the trace's four
  lanes (engine, render, load, cook). og-lanes' worker threads are not
  traced one by one; they would need per-job timing in og-lanes.
- **Span layout.** Systems and passes are laid end to end inside their
  tick or frame: the engine records their lengths, not their starts. The
  cook's outputs are laid end to end inside the cook span, because they
  cook on several threads with no start recorded (`args.laidEndToEnd`).
  Pass times are CPU encode time, not GPU time. On lavapipe the GPU's
  share dominates: the passes sum to well under 1 ms of a roughly 77 ms
  frame.
- **Native only.** `--trace` exists only in native `og run`. The web host
  has the `state().timings` passes (its backend implements the clock) but
  no trace file.
- **Write-back scope.**
  - It rewrites the boot scene (`GameDesc.scene`) only, not scenes
    instantiated with `og.load.scene`.
  - It does not add entities spawned at run time, and it does not write
    resources (the cook has no resource entry in scene text).
  - On the web it refuses, because there is no project directory to read
    the text from.
- **describe:** `occlusion` counts scene geometry only. A HUD over an
  entity does not count, so at tick 1 the crate is not occluded even
  where a 640×360 screenshot's HUD covers it.
- **Report contents.** The cook report reads models, textures and
  animations. Sounds, fonts, navmeshes and the rest report bytes only.
- **Stale reports.** `og doctor --project` warns that a report is stale by
  comparing modification times (cooked manifest newer than the report).
