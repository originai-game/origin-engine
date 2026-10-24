# r3/release-f4: release row F4 (browser frames and the published page on Brasswick and Meridian Loop)

Base 34db4fde (origin/main when started). Branch `r3/release-f4`. Orb, no
GPU, no Chrome: no browser was run, no reference frame was recorded or
committed (the rules).

## What landed, per commit

1. **5c9fd577** `web-reference: Brasswick's and Meridian Loop's frames on both tiers`
   - `packages/engine/tests/web/reference.ts`: `--playtest <file>` (relative
     to `--project`) bundles a frames playtest and takes its script and shots
     from it, as `reference-ui.ts` does the kit's: the commands without
     `screenshot`, `quit` and `dump_*`, each `<t>:screenshot:<f>` a shot at
     tick round(60 t), the playtest's `size` as the canvas. A playtest with
     `then`, `room` or `seed` is refused (one process on the game's seed),
     and so is `--script`/`--shots` beside it. `--print` prints the
     derived script and shots and exits, with no cook, export or Chrome.
     `reference-<tier>.json` now records the script it ran.
   - `scripts/check.sh` (shared): `step_web_reference` runs
     `--project examples/<game> --playtest tests/frames.og.ts --out
     dist/test-reference/<game>` on webgpu and `--webgl`, for brasswick and
     meridian-loop. Its result goes into the step's return (`p`). `rule_web`
     now reads `examples/brasswick examples/meridian-loop`. No new step and
     no `step_needs` change: this is inside the existing web-reference step.
   - `docs/ci.md` (shared): the web-reference rule's inputs and the step
     row name the two games; a paragraph under §Browser reference tiers
     after Ember Vale says what runs and how the frames are derived.
2. **eaef1a22** `published.ts: plans for Brasswick and Meridian Loop`
   - A plan can name a `playtest` of its project. Its steps run beside the
     plan's own, and its `quit` is dropped.
   - `brasswick`: frames.og.ts's presses (Level select, Escape, New game)
     with `title.png`@60 and `start.png`@228 against metal's. Then
     ArrowRight held, Escape (pause), Settings → Tab#3 → Option#1
     (Español) → Done, and a reload whose title is in Spanish
     ("Partida nueva"). Nine checks over state, mirror and the play
     origin's store calls. frames.og.ts's other seven frames need game
     commands (`unlock`, `warp`, `play`, `language`, `finish`), which a
     publish build does not have, so they are not on the page. The
     `0.2:unlock:2` is left out: neither title.png nor start.png shows the
     select.
   - `meridian-loop`: `tests/published.og.ts`'s own steps and four frames
     (`pub-title`@72, `pub-countdown`@180, `pub-straight`@510,
     `pub-pause`@600, already in references/metal). The race's state is
     taken beside each frame. Then Settings from pause → Tab#3 →
     Option#1 (Deutsch) → Done, and a reload in German ("Zeitfahren").
     Eight checks.
   - The header lists both plans.
3. **d26ff83f** `tests/web/device.ts: --project runs og run --device chrome on another project`
   (templates/game stays the default; output `dist/test-device/<name>`).
4. **2ad47548** `web-reference: make the games' generated assets on a checkout without them`.
   Both games' style art and fonts (`assets/styles/*/{*.png,*.ttf}`) and
   music are gitignored and written by `tools/make-assets.ts`. Without them
   `og build --publish` and the cook fail: "the style names asset(s)
   neither beside it …", seen here. The step runs the generator when the
   atlas or the wav is missing, as `wave-end.sh` `stage_prepare` does.
   `published.ts` and `og run --device` do not: run the generator once on
   a fresh checkout (the commands below do).

## How it was checked (no browser)

- `bun run typecheck` in packages/engine (OG_BIN = this checkout's dev
  `og`): tsrs ok on all three configurations after each commit.
- `shellcheck -x scripts/check.sh` clean; `bash -n` ok.
- `reference.ts --print` on the derived scripts:
  - brasswick: `title@60, levels@120, start@228, door@384, stomp@477, gate@816, pause@870, settings-es@930, results@1092`
  - meridian-loop: `title@72, jump@189, slide@285, countdown@438, race@1140, ghost@4620, pause@4956, results@6900, board@7044, settings-de@7272`
  - third-person (control): `title@54, beacon@204, dawn@432` and the same
    script as check.sh's hand-written line, so the derivation matches what
    the gate already ran.
  Each name is a file in `examples/<game>/references/metal`, and each tick is
  the one the frames playtest captures at.
- `published.ts` with no Chrome got as far as its plan, then stopped at
  the publish build (before the assets were made). It printed the merged
  scripts (31 and 33 steps, one relaunch each).
- `og check examples/brasswick`, `og check examples/meridian-loop`: ok. `og cook` of both: ok (after make-assets).
- `og run --device chrome examples/<game> --chrome /nonexistent`: both
  pass the project and options checks and refuse in one line for the
  missing Chrome, exit 1. A real device run needs Chrome.
- Brasswick's plan natively: the same steps, with `get`/`dump_ui` in
  place of state/mirror, under `og run --headless`. Results: title phase 0
  at tick 60; the `Levels` elements in the UI at 1.6 s; level 1 in play at
  3.8 s; x 3.5 → 9.96 after ArrowRight; paused after Escape; Prefs.language
  1 (es) and one `store.set og.settings` with `language: "es"`. A second
  process with the same `--platform-store`: `store.get og.settings`, Prefs
  es, `new` reads "Partida nueva".
- Meridian Loop's plan natively: not finished. The orb's CPU renderer took
  300 s to reach tick ~72 (title phase 0 was read). Its frames and plan
  steps are the playtest's own (`published.og.ts`, green on Metal per F1),
  and the settings taps are og.ui.settingsScreen's ids, the same kit as
  Brasswick's.

## The browser frames owed (for the coordinator, on the Mac, from this branch)

Nothing is recorded. Every frame is judged against `metal` (a tier with no
own reference falls back per frame). The commands below show whether each
passes. Record a tier's own reference (`--record`) only for a difference
the tier makes by design (docs/ci.md), after inspecting it.

Once per checkout:

    (cd examples/brasswick && bun tools/make-assets.ts)
    (cd examples/meridian-loop && bun tools/make-assets.ts)
    OG_GATE=1 sh packages/engine/scripts/build.sh

The web-reference frames: 9 + 10 frames × 2 tiers = 38. Run in
packages/engine:

    bun tests/web/reference.ts --project examples/brasswick --playtest tests/frames.og.ts --out dist/test-reference/brasswick
    bun tests/web/reference.ts --webgl --project examples/brasswick --playtest tests/frames.og.ts --out dist/test-reference/brasswick
    bun tests/web/reference.ts --project examples/meridian-loop --playtest tests/frames.og.ts --out dist/test-reference/meridian-loop
    bun tests/web/reference.ts --webgl --project examples/meridian-loop --playtest tests/frames.og.ts --out dist/test-reference/meridian-loop

The published page: brasswick title.png and start.png, meridian-loop's four
pub-*.png, × 2 tiers = 12 frames, plus the plans' checks:

    bun tests/web/published.ts --project examples/brasswick
    bun tests/web/published.ts --project examples/brasswick --webgl
    bun tests/web/published.ts --project examples/meridian-loop
    bun tests/web/published.ts --project examples/meridian-loop --webgl

The device runs (no frames; first frame and frame times, both tiers):

    bun tests/web/device.ts --project examples/brasswick
    bun tests/web/device.ts --project examples/meridian-loop

The whole step: `scripts/check.sh web-reference`.

If a tier's frame is off by design, record it with the same reference.ts
line plus `--record`, inspect it, and commit it under
`examples/<game>/references/<tier>/`.

## Shared files touched

- `scripts/check.sh`: `rule_web` adds the two example directories;
  `step_web_reference` adds the two games on both tiers (and the
  make-assets guard), with result `p` in its return. No new step,
  `step_needs` unchanged.
- `docs/ci.md`: the web-reference rule-inputs row, the web-reference step
  row, and one paragraph in §Browser reference tiers.

## Left out

- The frames that need game commands are not on the published page:
  Brasswick's door, stomp, gate, levels with two open, pause,
  settings-es and results, and Meridian Loop's frames.og.ts set. They need
  game commands, and the publish build has none. They are covered on both
  tiers by web-reference instead.
- Not added to `tests/web/device.ts`'s default run, nor to web-reference:
  `published.ts` and `device.ts` are not in web-reference today (hollowbell's
  and ui-kit's plans are not either). Whether to add them is a decision
  below.
- `packages/engine/bun.lock`, which a local `bun install` wrote, is not
  committed.

## Decisions for the coordinator

1. Meridian Loop's run on the web-reference tiers ends at tick 7272 (about
   two minutes of race, one tick and one render per call). Expect it to be
   the slowest station in the step. If it is too slow for a wave end,
   options are a shorter frames playtest for the web or `--only`-style
   selection. Both change what F4 measures; your call.
2. Whether `published.ts` for the two games (and hollowbell's and ui-kit's)
   joins `step_web_reference`. F4's "web-reference passes with them" is
   met by the frames; the published-page plans exist and run by hand like
   hollowbell's.
3. Meridian Loop's published plan was not run natively here (CPU only).
   The first Mac run is its first check: the Settings → Tab#3 → Option#1
   path from pause, and the reload in German.
