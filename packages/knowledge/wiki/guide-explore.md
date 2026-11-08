<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `explore` guide: Exploring for bugs

How `og explore` searches a game's states for the bugs its playtests do not script, what it reports, how to act on a finding, and how to shape the search with GameDesc.explore. Read after the testing guide: explore checks the same invariants.

The API: `og api show assert`, `og api show ExploreDesc`, `og api show RestoreDesc`, `og api show probe`, and `runPlaytest` in `@origingame/engine/playtest`. `og explore --help` gives the flags.

<a id="guide-explore-1"></a>
### `explore §1` What it does

A playtest checks the paths its script plays. `og explore` plays the
paths nobody wrote: it returns to states it has reached and tries other
inputs from there, and reports what broke.

- It keeps an archive of cells. A cell is a state abstraction: each
  field of the game's resources and components, bucketed (small integers
  exactly, larger ones by power of two, fractions in eighths), plus the
  time in 64 slices of the horizon. Each cell keeps the shortest input
  trace known to reach it and the world's snapshot there.
- Each step chooses a cell, favouring those chosen least and those that
  brought something new (half the time by state first, half by time
  first); restores its snapshot (the whole world, input
  included); and plays a burst of the game's own inputs from it: its
  button actions pressed through their key or pad bindings, the intents
  GameDesc.explore names, and waits.
- Everything goes through the recorded input path, the command channel a
  playtest uses, so every trace is a command script.
- The search is seeded: the same seed, budget and build search the same
  way. Change `--seed` to search differently.

The method is Go-Explore's (Ecoffet et al., 2019 and 2021), with state
buckets after AFL's hit counts and IJON's state annotations, and the
assertion kinds as the search's goals after Antithesis. The sources and
their licences are in the engine's `third_party/explore.md`.

<a id="guide-explore-2"></a>
### `explore §2` What it finds

A finding is the first time the search meets one of these:

- a violation: an `og.assert.always` that failed, an
  `og.assert.unreachable` that ran, a default invariant (a field out of
  its range, a non-finite number) or a playtest invariant;
- an error: a system that threw;
- a reach: an `og.assert.sometimes`, `og.assert.reachable` or
  `og.assert.each` bucket that fired. Assertions of these kinds are the
  search's goals: put one on each state worth reaching (the boss's second
  phase, the shop's empty stock, a level's last room), and the report
  says whether the search got there.

Each finding is shrunk by delta debugging to the fewest presses that
still reach it by the same tick, then replayed from tick 0 three times:
twice in the searching process and once in a fresh `og run`, and, where
the engine's web module is built, on the wasm host. `deterministic` is
true when every replay ended on the same world hash; a false one is a
determinism bug of its own (determinism §1), reported with each hash.

<a id="guide-explore-3"></a>
### `explore §3` The report

`og explore --json` prints the report; `report.json` in `--out` holds the
same. It is facts, never a verdict: what was found, how, at what cost.

| Field | What it holds |
|---|---|
| `findings[]` | `kind` (`violation`, `error`, `reached`), `site`, `tick`, the violating `entity`, `component`, `field` and `value`, `found` (worker, ticks and ms into the search), `shrunk` (presses before and after, replays spent), `repro` (`commands`, `ticks`, `hash`, the repro file), `replays` (the hash of each replay), `deterministic`, `notes`, `playtest` |
| `coverage` | the assertion sites the game declares, those that fired, and `uncovered`: each sometimes, reachable or each site the search never fired |
| `cells` | the archive's size, the fields its cells read, the distinct field buckets seen, the cells evicted for memory |
| `used` | ticks, wall time, returns to cells, peak memory, the archive's size, why each worker stopped |
| `inputs` | what the search pressed and sent |

For each finding `--out` also holds `<id>.repro.json` (the trace) and
`<id>.og.ts`, a bun:test playtest that plays it. A violation's or an
error's playtest fails until the game is fixed: copy it into `tests/`,
fix the game, and keep it as the regression test. A reach's playtest
requires the site and passes.

<a id="guide-explore-4"></a>
### `explore §4` The workflow

1. State the game's rules as invariants (testing §2): ranges on fields,
   `og.assert.always` on the rules no range expresses, `sometimes` on the
   states a player should reach. Explore finds nothing the game does not
   state.
2. `og explore --seconds 60 --json`. Read `findings` first, then
   `coverage.uncovered`.
3. For each violation or error: run its `.og.ts` with `og test` to see it
   fail, read the tick, entity, field and value, fix the cause in the
   game, and keep the playtest.
4. For each uncovered site: either the state is unreachable (a bug of its
   own, or a site to delete), or the search needs help (§5).
5. Run again with another `--seed` and a larger budget before calling the
   game clean; a search that found nothing is a measurement of where it
   looked, not a proof.

<a id="guide-explore-5"></a>
### `explore §5` Shaping the search

GameDesc.explore steers it; each field is optional.

- `cells`: the fields that make a cell, as `Record` or `Record.field`.
  The default is every field of every game record, which is right for
  small games. A game with many entities or noisy fields (timers, particle
  counters, a camera's position) finds more with the few fields that are
  progress: the level, the room, the inventory's count, the boss's phase.
- `ignore`: fields left out of cells, `@tick` to drop the time slices for
  a game whose state does not depend on when things happen.
- `actions`: the actions the search presses, when not every button action
  should be (a pause or a quit button). Axis actions are pressed by each
  key of their bindings only when named here.
- `intents`: the intents it sends, each with its payloads; without
  `values` it sends the defaults and each field's values one at a time
  (an enum's variants, a bool's both, a small integer range's every
  value, a number range's ends and middle). Intents are how UI buttons
  reach gameplay, so a game driven by its UI explores through them.

The flags bound it: `--seconds`, `--ticks` (ticks played, shared across
workers), `--memory` (the archive's snapshots; the least promising cells
go first), `--horizon` (the latest tick a burst starts at) and `--jobs`
(worker processes, each its own engine and seed: N jobs search N ways at
once). `--first` stops every worker at the first violation or error, for
a check that only needs to know whether one exists; only violations and
errors are then shrunk and reported as findings, the sites reached are in
`coverage`.

<a id="guide-explore-6"></a>
### `explore §6` Pitfalls

- A cell is only as good as its fields. A field that changes every tick
  for no reason (a frame counter, a float accumulator) splits every state
  into many; `ignore` it.
- A rule held only in code the search cannot see is not checked. A bug
  that leaves every field in range and every assertion true is not found:
  add the assertion that states the rule.
- Input outside the command channel (a mouse path, a drag, text) is not
  explored; give the gameplay an intent and name it in `intents`.
- A finding the shrink cannot reproduce fails the run with "the replay is
  not deterministic": something outside the world (a module variable, a
  `Math.random`, the wall clock) fed the simulation (determinism §1).
- The playtest a finding writes replays from the project's seed and the
  playtest clock: a game whose start depends on a `--seed` or `--clock`
  the search did not use plays another game.
- The search plays one process's engine; it does not explore lockstep
  rooms or a playtest's `then` relaunches.
