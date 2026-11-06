<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `assert` module

Assertions: properties of the game checked as it runs, after the Antithesis SDK's. `og.assert.always(world.get(hero, Health).hp >= 0, "hp never negative", { hp })` holds on every call, `og.assert.sometimes(boss.phase === 2, "boss reaches phase 2")` must hold at least once in a run, `og.assert.reachable("door opened")` must run at least once, `og.assert.unreachable("fell through the floor")` must never run, and `og.assert.each(level, cleared, "each level cleared")` keeps one bucket per distinct key, each of which must hold once.

Each is identified by its name and the place it is called from (`src/player.ts:42`, which the build records for every `og.assert.<kind>(...)` call, so a site that never runs is still listed). A call that passes costs a lookup in the runtime, without a call into the engine; nothing an assertion does reaches the world, so a run with them hashes and replays as one without.

Violations and coverage go through the probe (`assertions`) into `og run --json`'s `assertions` and `og test`'s results: the tick, the name and place, and the values (`details`). At the end of a run, every `sometimes`, `reachable` and `each` bucket that never fired is listed as uncovered: a sometimes that never fires is a test that never got there. A playtest's `require` names the ones that must fire.

Beside them, every tick of a development or test build checks the default invariants from reflection: no NaN or infinity in a reflected float, a number inside its field's min..max where it declares one, no Entity field naming a despawned entity, and, where the world declares og.WorldBounds, every entity's place inside them. They read the components that changed (where the world tracks their changes) or every row, never write, and name the entity, component, field and value of their first failure.

Not in publish builds: every member does nothing there, so a game may leave its assertions in.

- **TypeScript**: functions are the namespace `og.assert`; components, resources and events are tokens on `og`.
- **Publish builds**: dropped (members may override).
- **Index**: [every module and name](index.md).

```ogt
module assert {
  doc "Assertions: properties of the game checked as it runs, after the Antithesis SDK's. `og.assert.always(world.get(hero, Health).hp >= 0, \"hp never negative\", { hp })` holds on every call, `og.assert.sometimes(boss.phase === 2, \"boss reaches phase 2\")` must hold at least once in a run, `og.assert.reachable(\"door opened\")` must run at least once, `og.assert.unreachable(\"fell through the floor\")` must never run, and `og.assert.each(level, cleared, \"each level cleared\")` keeps one bucket per distinct key, each of which must hold once."
  doc "Each is identified by its name and the place it is called from (`src/player.ts:42`, which the build records for every `og.assert.<kind>(...)` call, so a site that never runs is still listed). A call that passes costs a lookup in the runtime, without a call into the engine; nothing an assertion does reaches the world, so a run with them hashes and replays as one without."
  doc "Violations and coverage go through the probe (`assertions`) into `og run --json`'s `assertions` and `og test`'s results: the tick, the name and place, and the values (`details`). At the end of a run, every `sometimes`, `reachable` and `each` bucket that never fired is listed as uncovered: a sometimes that never fires is a test that never got there. A playtest's `require` names the ones that must fire."
  doc "Beside them, every tick of a development or test build checks the default invariants from reflection: no NaN or infinity in a reflected float, a number inside its field's min..max where it declares one, no Entity field naming a despawned entity, and, where the world declares og.WorldBounds, every entity's place inside them. They read the components that changed (where the world tracks their changes) or every row, never write, and name the entity, component, field and value of their first failure."
  doc "Not in publish builds: every member does nothing there, so a game may leave its assertions in."
  publish false
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 25706</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`AssertKind`](#type-AssertKind) | enum | What an assertion asks. |
| [`AssertRecord`](#type-AssertRecord) | struct | One assertion call the runtime reports to the engine (og.assert.record): its first call, a violation, or its first firing in a report. |
| [`AssertOrigin`](#type-AssertOrigin) | enum | Who declared an assertion. |
| [`AssertBucket`](#type-AssertBucket) | struct | One bucket of an each. |
| [`AssertSite`](#type-AssertSite) | struct | One assertion in the report: by its kind, name and place. |
| [`AssertViolation`](#type-AssertViolation) | struct | A site's first violation. |
| [`AssertUncovered`](#type-AssertUncovered) | struct | A sometimes, reachable or each bucket that never fired. |
| [`AssertCost`](#type-AssertCost) | struct | What the default invariants cost, measured outside the tick. |
| [`AssertReport`](#type-AssertReport) | struct | The run's assertions since the report began (the probe's assertions member). |
| [`og.WorldBounds`](#record-WorldBounds) | resource | Where the world's entities may be: naming it turns on the default invariant that every entity's place (its og.MatrixWorld translation, else a root's og.Transform.position) lies inside min..max on every tick. A world that never names it has no bounds and keeps its hash. |
| [`og.assert.always`](#fn-always) | function | The condition holds on every call: a false one is a violation, reported with its tick, its name, its place and `details`. |
| [`og.assert.sometimes`](#fn-sometimes) | function | The condition holds on at least one call in the run; one that never held is listed as uncovered when the run ends. |
| [`og.assert.reachable`](#fn-reachable) | function | This line runs at least once in the run; one that never ran is listed as uncovered. |
| [`og.assert.unreachable`](#fn-unreachable) | function | This line never runs: running it is a violation. |
| [`og.assert.each`](#fn-each) | function | One bucket per distinct key (a level, a weapon, an enemy kind): each bucket the run saw must hold its condition once (Antithesis's SOMETIMES_EACH). Buckets seen that never held are listed as uncovered, and every bucket is in the probe's report, the hints a search of the game's states starts from. |
| [`og.assert.record`](#fn-record) | function | The one call the members above make into the engine: a site's first call, a violation or a first firing. Game code calls those, not this. With null, it only answers the report's epoch. |

## Functions

<a id="fn-always"></a>

### `og.assert.always`

The condition holds on every call: a false one is a violation, reported with its tick, its name, its place and `details`.

```ts
og.assert.always(condition: boolean, name: string, details?: unknown | null): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `condition` | `boolean` | no | What must hold. |
| `name` | `string` | no | What it means, in a few words: `hp never negative`. |
| `details` | `unknown \| null` | yes | Values worth seeing when it fails, as `{ hp, damage }`. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn always {
  doc "The condition holds on every call: a false one is a violation, reported with its tick, its name, its place and `details`."
  host false
  param condition bool "What must hold."
  param name String "What it means, in a few words: `hp never negative`."
  param details Option.Json "Values worth seeing when it fails, as `{ hp, damage }`." {
    optional true
  }
}
```

<sub>`api.ogt` line 25836</sub>

<a id="fn-sometimes"></a>

### `og.assert.sometimes`

The condition holds on at least one call in the run; one that never held is listed as uncovered when the run ends.

```ts
og.assert.sometimes(condition: boolean, name: string, details?: unknown | null): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `condition` | `boolean` | no | What must hold at least once. |
| `name` | `string` | no | What it means: `boss reaches phase 2`. |
| `details` | `unknown \| null` | yes | Values worth seeing at the first time it holds. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn sometimes {
  doc "The condition holds on at least one call in the run; one that never held is listed as uncovered when the run ends."
  host false
  param condition bool "What must hold at least once."
  param name String "What it means: `boss reaches phase 2`."
  param details Option.Json "Values worth seeing at the first time it holds." {
    optional true
  }
}
```

<sub>`api.ogt` line 25846</sub>

<a id="fn-reachable"></a>

### `og.assert.reachable`

This line runs at least once in the run; one that never ran is listed as uncovered.

```ts
og.assert.reachable(name: string, details?: unknown | null): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | What reaching it means: `door opened`. |
| `details` | `unknown \| null` | yes | Values worth seeing the first time. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn reachable {
  doc "This line runs at least once in the run; one that never ran is listed as uncovered."
  host false
  param name String "What reaching it means: `door opened`."
  param details Option.Json "Values worth seeing the first time." {
    optional true
  }
}
```

<sub>`api.ogt` line 25856</sub>

<a id="fn-unreachable"></a>

### `og.assert.unreachable`

This line never runs: running it is a violation.

```ts
og.assert.unreachable(name: string, details?: unknown | null): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | What reaching it would mean: `fell through the floor`. |
| `details` | `unknown \| null` | yes | Values worth seeing when it does. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn unreachable {
  doc "This line never runs: running it is a violation."
  host false
  param name String "What reaching it would mean: `fell through the floor`."
  param details Option.Json "Values worth seeing when it does." {
    optional true
  }
}
```

<sub>`api.ogt` line 25865</sub>

<a id="fn-each"></a>

### `og.assert.each`

One bucket per distinct key (a level, a weapon, an enemy kind): each bucket the run saw must hold its condition once (Antithesis's SOMETIMES_EACH). Buckets seen that never held are listed as uncovered, and every bucket is in the probe's report, the hints a search of the game's states starts from.

```ts
og.assert.each(key: unknown, condition: boolean, name: string): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `key` | `unknown` | no | The bucket: a string, a number, or a small object of either (`{ level, weapon }`). |
| `condition` | `boolean` | no | What must hold once in the bucket. |
| `name` | `string` | no | What it means: `each level cleared`. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn each {
  doc "One bucket per distinct key (a level, a weapon, an enemy kind): each bucket the run saw must hold its condition once (Antithesis's SOMETIMES_EACH). Buckets seen that never held are listed as uncovered, and every bucket is in the probe's report, the hints a search of the game's states starts from."
  host false
  param key Json "The bucket: a string, a number, or a small object of either (`{ level, weapon }`)."
  param condition bool "What must hold once in the bucket."
  param name String "What it means: `each level cleared`."
}
```

<sub>`api.ogt` line 25874</sub>

<a id="fn-record"></a>

### `og.assert.record`

The one call the members above make into the engine: a site's first call, a violation or a first firing. Game code calls those, not this. With null, it only answers the report's epoch.

```ts
og.assert.record(world: World, record: AssertRecord | null): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world the assertion was called in. |
| `record` | `AssertRecord \| null` | no | The call, or null. |

**Returns** `number`: The report's epoch: it moves on each time the probe's assertions member starts a new report.

Binding symbol `og_assert_record` · publish builds: dropped

```ogt
fn record {
  doc "The one call the members above make into the engine: a site's first call, a violation or a first firing. Game code calls those, not this. With null, it only answers the report's epoch."
  param world World "The world the assertion was called in."
  param record Option.AssertRecord "The call, or null."
  returns u32 "The report's epoch: it moves on each time the probe's assertions member starts a new report."
}
```

<sub>`api.ogt` line 25882</sub>

## Components, resources and events

<a id="record-WorldBounds"></a>

### `og.WorldBounds` (resource)

Where the world's entities may be: naming it turns on the default invariant that every entity's place (its og.MatrixWorld translation, else a root's og.Transform.position) lies inside min..max on every tick. A world that never names it has no bounds and keeps its hash.

Token `og.WorldBounds`, registered under the schema name `og.WorldBounds`; its value type is `WorldBounds`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `min` | `Vector3` | `{ x: -1000, y: -1000, z: -1000 }` | — | `m` | worldspace `world` | The lowest corner. |
| `max` | `Vector3` | `{ x: 1000, y: 1000, z: 1000 }` | — | `m` | worldspace `world` | The highest corner. |
| `off` | `boolean` | `false` | — | — | — | Leave the bounds unchecked. |

```ogt
resource WorldBounds {
  doc "Where the world's entities may be: naming it turns on the default invariant that every entity's place (its og.MatrixWorld translation, else a root's og.Transform.position) lies inside min..max on every tick. A world that never names it has no bounds and keeps its hash."
  field min Vector3 "The lowest corner." {
    default -1000 -1000 -1000
    worldspace world
    unit m
  }
  field max Vector3 "The highest corner." {
    default 1000 1000 1000
    worldspace world
    unit m
  }
  field off bool "Leave the bounds unchecked."
}
```

<sub>`api.ogt` line 25821</sub>

## Types

<a id="type-AssertKind"></a>

### `AssertKind` (enum)

What an assertion asks.

A string: one of the variants below. The zero value is `"always"`.

| Variant | Doc |
|---|---|
| `"always"` | Its condition holds on every call: a false one is a violation. |
| `"sometimes"` | Its condition holds on at least one call in the run. |
| `"reachable"` | It runs at least once in the run. |
| `"unreachable"` | It never runs: a call is a violation. |
| `"each"` | Its condition holds once for each distinct key it was called with. |

```ogt
enum AssertKind {
  doc "What an assertion asks."
  variant always "Its condition holds on every call: a false one is a violation."
  variant sometimes "Its condition holds on at least one call in the run."
  variant reachable "It runs at least once in the run."
  variant unreachable "It never runs: a call is a violation."
  variant each "Its condition holds once for each distinct key it was called with."
}
```

<sub>`api.ogt` line 25714</sub>

<a id="type-AssertRecord"></a>

### `AssertRecord` (struct)

One assertion call the runtime reports to the engine (og.assert.record): its first call, a violation, or its first firing in a report.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `AssertKind` | `"always"` | — | — | — | The assertion's kind. |
| `name` | `string` | `""` | — | — | — | Its name. |
| `location` | `string` | `""` | — | — | — | Where it is called (`src/player.ts:42`); empty when the build did not record one. |
| `condition` | `boolean` | `false` | — | — | — | The condition (true for reachable and unreachable, which have none). |
| `key` | `string` | `""` | — | — | — | The bucket of an each, as text; empty otherwise. |
| `details` | `unknown` | `null` | — | — | — | What the call passed as details (null for none). |
| `tick` | `number` | `0` | — | — | — | The tick of the call. |
| `declared` | `boolean` | `false` | — | — | — | A site the build found, sent before it runs: listed, not counted as a call. |

```ogt
struct AssertRecord {
  doc "One assertion call the runtime reports to the engine (og.assert.record): its first call, a violation, or its first firing in a report."
  field kind AssertKind "The assertion's kind."
  field name String "Its name."
  field location String "Where it is called (`src/player.ts:42`); empty when the build did not record one."
  field condition bool "The condition (true for reachable and unreachable, which have none)."
  field key String "The bucket of an each, as text; empty otherwise."
  field details Json "What the call passed as details (null for none)."
  field tick u32 "The tick of the call."
  field declared bool "A site the build found, sent before it runs: listed, not counted as a call."
}
```

<sub>`api.ogt` line 25723</sub>

<a id="type-AssertOrigin"></a>

### `AssertOrigin` (enum)

Who declared an assertion.

A string: one of the variants below. The zero value is `"game"`.

| Variant | Doc |
|---|---|
| `"game"` | The game's og.assert calls. |
| `"default"` | The engine's default invariants from reflection. |
| `"playtest"` | A playtest's `invariant:` steps (PlaytestDesc.invariants). |

```ogt
enum AssertOrigin {
  doc "Who declared an assertion."
  variant game "The game's og.assert calls."
  variant default "The engine's default invariants from reflection."
  variant playtest "A playtest's `invariant:` steps (PlaytestDesc.invariants)."
}
```

<sub>`api.ogt` line 25735</sub>

<a id="type-AssertBucket"></a>

### `AssertBucket` (struct)

One bucket of an each.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | The bucket's key, as text. |
| `fired` | `boolean` | `false` | — | — | — | Whether its condition held once. |
| `firstTick` | `number \| null` | `null` | — | — | — | The first tick it held. |

```ogt
struct AssertBucket {
  doc "One bucket of an each."
  field key String "The bucket's key, as text."
  field fired bool "Whether its condition held once."
  field firstTick Option.u32 "The first tick it held."
}
```

<sub>`api.ogt` line 25742</sub>

<a id="type-AssertSite"></a>

### `AssertSite` (struct)

One assertion in the report: by its kind, name and place.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `AssertKind` | `"always"` | — | — | — | Its kind. |
| `origin` | `AssertOrigin` | `"game"` | — | — | — | Who declared it. |
| `name` | `string` | `""` | — | — | — | Its name. |
| `location` | `string` | `""` | — | — | — | Where it is called; for a default invariant, the record and field (`Health.hp`). |
| `ran` | `boolean` | `false` | — | — | — | Whether it was called in this report (a site the build found may never run). |
| `fired` | `boolean` | `false` | — | — | — | Whether a sometimes held, a reachable ran, or an each's bucket held. |
| `firstTick` | `number \| null` | `null` | — | — | — | The first tick it fired. |
| `failures` | `number` | `0` | — | — | — | How many calls violated it. |
| `buckets` | `readonly AssertBucket[]` | `[]` | — | — | optional | An each's buckets, in the order first seen. |
| `bucketsDropped` | `number` | `0` | — | — | optional | An each's buckets past the 1,024 kept. |

```ogt
struct AssertSite {
  doc "One assertion in the report: by its kind, name and place."
  field kind AssertKind "Its kind."
  field origin AssertOrigin "Who declared it."
  field name String "Its name."
  field location String "Where it is called; for a default invariant, the record and field (`Health.hp`)."
  field ran bool "Whether it was called in this report (a site the build found may never run)."
  field fired bool "Whether a sometimes held, a reachable ran, or an each's bucket held."
  field firstTick Option.u32 "The first tick it fired."
  field failures u32 "How many calls violated it."
  field buckets List.AssertBucket "An each's buckets, in the order first seen." {
    optional true
  }
  field bucketsDropped u32 "An each's buckets past the 1,024 kept." {
    optional true
  }
}
```

<sub>`api.ogt` line 25749</sub>

<a id="type-AssertViolation"></a>

### `AssertViolation` (struct)

A site's first violation.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `AssertKind` | `"always"` | — | — | — | The site's kind. |
| `origin` | `AssertOrigin` | `"game"` | — | — | — | Who declared it. |
| `name` | `string` | `""` | — | — | — | Its name. |
| `location` | `string` | `""` | — | — | — | Its place. |
| `tick` | `number` | `0` | — | — | — | The first tick it failed. |
| `entity` | `string \| null` | `null` | — | — | — | The entity a default invariant found it on (its probe id). |
| `component` | `string \| null` | `null` | — | — | — | The record: a component or a resource. |
| `field` | `string \| null` | `null` | — | — | — | The field. |
| `value` | `unknown` | `null` | — | — | — | What was there (a NaN or an infinity as text). |
| `expected` | `unknown` | `null` | — | — | — | What was expected: a range, live, finite, the bounds. |
| `message` | `string` | `""` | — | — | — | What went wrong, in words. |
| `details` | `unknown` | `null` | — | — | — | What the call passed as details. |
| `failures` | `number` | `0` | — | — | — | How many calls violated the site in this report. |
| `lastTick` | `number` | `0` | — | — | — | The last tick it failed. |

```ogt
struct AssertViolation {
  doc "A site's first violation."
  field kind AssertKind "The site's kind."
  field origin AssertOrigin "Who declared it."
  field name String "Its name."
  field location String "Its place."
  field tick u32 "The first tick it failed."
  field entity Option.String "The entity a default invariant found it on (its probe id)."
  field component Option.String "The record: a component or a resource."
  field field Option.String "The field."
  field value Json "What was there (a NaN or an infinity as text)."
  field expected Json "What was expected: a range, live, finite, the bounds."
  field message String "What went wrong, in words."
  field details Json "What the call passed as details."
  field failures u32 "How many calls violated the site in this report."
  field lastTick u32 "The last tick it failed."
}
```

<sub>`api.ogt` line 25767</sub>

<a id="type-AssertUncovered"></a>

### `AssertUncovered` (struct)

A sometimes, reachable or each bucket that never fired.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `AssertKind` | `"always"` | — | — | — | The site's kind. |
| `name` | `string` | `""` | — | — | — | Its name. |
| `location` | `string` | `""` | — | — | — | Its place. |
| `key` | `string` | `""` | — | — | optional | An each's bucket that never held. |
| `ran` | `boolean` | `false` | — | — | optional | Whether it was called at all (false: the run never got there). |

```ogt
struct AssertUncovered {
  doc "A sometimes, reachable or each bucket that never fired."
  field kind AssertKind "The site's kind."
  field name String "Its name."
  field location String "Its place."
  field key String "An each's bucket that never held." {
    optional true
  }
  field ran bool "Whether it was called at all (false: the run never got there)." {
    optional true
  }
}
```

<sub>`api.ogt` line 25785</sub>

<a id="type-AssertCost"></a>

### `AssertCost` (struct)

What the default invariants cost, measured outside the tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `fields` | `number` | `0` | — | — | — | The reflected fields they check. |
| `ticks` | `number` | `0` | — | — | — | The ticks they checked in this report. |
| `values` | `number` | `0` | — | — | — | The values they read in this report. |
| `totalMs` | `number` | `0` | — | `ms` | — | Their time in this report. |
| `meanUs` | `number` | `0` | — | — | — | Their mean time a tick, in microseconds. |
| `maxUs` | `number` | `0` | — | — | — | Their slowest tick, in microseconds. |

```ogt
struct AssertCost {
  doc "What the default invariants cost, measured outside the tick."
  field fields u32 "The reflected fields they check."
  field ticks u32 "The ticks they checked in this report."
  field values f64 "The values they read in this report."
  field totalMs f64 "Their time in this report." {
    unit ms
  }
  field meanUs f64 "Their mean time a tick, in microseconds."
  field maxUs f64 "Their slowest tick, in microseconds."
}
```

<sub>`api.ogt` line 25798</sub>

<a id="type-AssertReport"></a>

### `AssertReport` (struct)

The run's assertions since the report began (the probe's assertions member).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `epoch` | `number` | `0` | — | — | — | The report's number: take moves it on. |
| `failures` | `number` | `0` | — | — | — | Violating calls, summed over the sites. |
| `violations` | `readonly AssertViolation[]` | `[]` | — | — | — | Each violated site's first violation, in the order they happened. |
| `uncovered` | `readonly AssertUncovered[]` | `[]` | — | — | — | What never fired: a run whose sometimes never held did not test what it names. |
| `sites` | `readonly AssertSite[]` | `[]` | — | — | — | Every site seen or declared (default invariants once they fail), each with its buckets: the hints a search of the game's states starts from. |
| `sitesDropped` | `number` | `0` | — | — | — | Sites past the 4,096 kept. |
| `defaults` | `AssertCost` | `{ fields: 0, ticks: 0, values: 0, totalMs: 0, meanUs: 0, maxUs: 0 }` | — | — | — | The default invariants' cost. |

```ogt
struct AssertReport {
  doc "The run's assertions since the report began (the probe's assertions member)."
  field epoch u32 "The report's number: take moves it on."
  field failures u32 "Violating calls, summed over the sites."
  field violations List.AssertViolation "Each violated site's first violation, in the order they happened."
  field uncovered List.AssertUncovered "What never fired: a run whose sometimes never held did not test what it names."
  field sites List.AssertSite "Every site seen or declared (default invariants once they fail), each with its buckets: the hints a search of the game's states starts from."
  field sitesDropped u32 "Sites past the 4,096 kept."
  field defaults AssertCost "The default invariants' cost."
}
```

<sub>`api.ogt` line 25810</sub>
