<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `assert` module

Assertions: properties of the game checked as it runs, after the Antithesis SDK's. `og.assert.always(world.get(hero, Health).hp >= 0, "hp never negative", { hp })` holds on every call, `og.assert.sometimes(boss.phase === 2, "boss reaches phase 2")` must hold at least once in a run, `og.assert.reachable("door opened")` must run at least once, `og.assert.unreachable("fell through the floor")` must never run, and `og.assert.each(level, cleared, "each level cleared")` keeps one bucket per distinct key, each of which must hold once.

Each is identified by its name and the place it is called from (`src/player.ts:42`, which the build records for every `og.assert.<kind>(...)` call, so a site that never runs is still listed). A call that passes costs a lookup in the runtime, without a call into the engine; nothing an assertion does reaches the world, so a run with them hashes and replays as one without.

Violations and coverage go through the probe (`assertions`) into `og run --json`'s `assertions` and `og test`'s results: the tick, the name and place, and the values (`details`). At the end of a run, every `sometimes`, `reachable` and `each` bucket that never fired is listed as uncovered: a sometimes that never fires is a test that never got there. A playtest's `require` names the ones that must fire.

Beside them, every tick of a development or test build checks the default invariants from reflection: no NaN or infinity in a reflected float, a number inside its field's min..max where it declares one, no Entity field naming a despawned entity, and, where the world declares og.WorldBounds, every entity's place inside them. They read the components that changed (where the world tracks their changes) or every row, never write, and name the entity, component, field and value of their first failure.

Not in publish builds: every member does nothing there, so a game may leave its assertions in.

- **TypeScript**: functions are the namespace `og.assert`; components, resources and events are tokens on `og`.
- **Publish builds**: dropped (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module assert {
  doc "Assertions: properties of the game checked as it runs, after the Antithesis SDK's. `og.assert.always(world.get(hero, Health).hp >= 0, \"hp never negative\", { hp })` holds on every call, `og.assert.sometimes(boss.phase === 2, \"boss reaches phase 2\")` must hold at least once in a run, `og.assert.reachable(\"door opened\")` must run at least once, `og.assert.unreachable(\"fell through the floor\")` must never run, and `og.assert.each(level, cleared, \"each level cleared\")` keeps one bucket per distinct key, each of which must hold once."
  doc "Each is identified by its name and the place it is called from (`src/player.ts:42`, which the build records for every `og.assert.<kind>(...)` call, so a site that never runs is still listed). A call that passes costs a lookup in the runtime, without a call into the engine; nothing an assertion does reaches the world, so a run with them hashes and replays as one without."
  doc "Violations and coverage go through the probe (`assertions`) into `og run --json`'s `assertions` and `og test`'s results: the tick, the name and place, and the values (`details`). At the end of a run, every `sometimes`, `reachable` and `each` bucket that never fired is listed as uncovered: a sometimes that never fires is a test that never got there. A playtest's `require` names the ones that must fire."
  doc "Beside them, every tick of a development or test build checks the default invariants from reflection: no NaN or infinity in a reflected float, a number inside its field's min..max where it declares one, no Entity field naming a despawned entity, and, where the world declares og.WorldBounds, every entity's place inside them. They read the components that changed (where the world tracks their changes) or every row, never write, and name the entity, component, field and value of their first failure."
  doc "Not in publish builds: every member does nothing there, so a game may leave its assertions in."
  publish false
  # declarations follow, each on its own page
```

<sub>`api.ogt` line 25830</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`AssertKind`](assert/type-AssertKind.md) | enum |  | What an assertion asks. |
| [`AssertRecord`](assert/type-AssertRecord.md) | struct |  | One assertion call the runtime reports to the engine (og.assert.record): its first call, a violation, or its first firing in a report. |
| [`AssertOrigin`](assert/type-AssertOrigin.md) | enum |  | Who declared an assertion. |
| [`AssertBucket`](assert/type-AssertBucket.md) | struct |  | One bucket of an each. |
| [`AssertSite`](assert/type-AssertSite.md) | struct |  | One assertion in the report: by its kind, name and place. |
| [`AssertViolation`](assert/type-AssertViolation.md) | struct |  | A site's first violation. |
| [`AssertUncovered`](assert/type-AssertUncovered.md) | struct |  | A sometimes, reachable or each bucket that never fired. |
| [`AssertCost`](assert/type-AssertCost.md) | struct |  | What the default invariants cost, measured outside the tick. |
| [`AssertReport`](assert/type-AssertReport.md) | struct |  | The run's assertions since the report began (the probe's assertions member). |
| [`og.WorldBounds`](assert/record-WorldBounds.md) | resource |  | Where the world's entities may be: naming it turns on the default invariant that every entity's place (its og.MatrixWorld translation, else a root's og.Transform.position) lies inside min..max on eve… |
| [`og.assert.always`](assert/fn-always.md) | function | `(condition: boolean, name: string, details?: unknown \| null): void` | The condition holds on every call: a false one is a violation, reported with its tick, its name, its place and `details`. |
| [`og.assert.sometimes`](assert/fn-sometimes.md) | function | `(condition: boolean, name: string, details?: unknown \| null): void` | The condition holds on at least one call in the run; one that never held is listed as uncovered when the run ends. |
| [`og.assert.reachable`](assert/fn-reachable.md) | function | `(name: string, details?: unknown \| null): void` | This line runs at least once in the run; one that never ran is listed as uncovered. |
| [`og.assert.unreachable`](assert/fn-unreachable.md) | function | `(name: string, details?: unknown \| null): void` | This line never runs: running it is a violation. |
| [`og.assert.each`](assert/fn-each.md) | function | `(key: unknown, condition: boolean, name: string): void` | One bucket per distinct key (a level, a weapon, an enemy kind): each bucket the run saw must hold its condition once (Antithesis's SOMETIMES_EACH). |
| [`og.assert.record`](assert/fn-record.md) | function | `(world: World, record: AssertRecord \| null): number` | The one call the members above make into the engine: a site's first call, a violation or a first firing. |
