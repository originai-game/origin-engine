<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `rng` module

Deterministic randomness. Every random draw comes from an Rng stream; the sequence for a seed is identical on every host and in the Web scaffold's rng.ts.

Inside a system: forking needs og.Rng among the reads (a fork is derived from the seed and the label, and leaves the main stream where it is); drawing from og.Rng itself (next, range, integer, chance) moves its state, so it needs og.Rng among the writes. A stream a fork returned is the system's own to draw from.

- **TypeScript**: functions are none at module level (its members live on the types below); components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-core`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module rng {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 612</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`RngStream`](#type-RngStream) | interface | One generator stream, forked from the Rng resource for a subsystem that must not perturb the main sequence. |
| [`og.Rng`](#record-Rng) | resource | The world's main random stream, seeded from GameDesc.seed (or the replay seed). Its state is part of every snapshot. |

## Components, resources and events

<a id="record-Rng"></a>

### `og.Rng` (resource)

The world's main random stream, seeded from GameDesc.seed (or the replay seed). Its state is part of every snapshot.

Token `og.Rng`, registered under the schema name `og.Rng`; its value type is `Rng`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `seed` | `number` | `0` | — | — | read only | The seed the stream started from. |
| `state` | `number` | `0` | — | — | read only | The generator state; snapshot and restore carry it. |

```ogt
resource Rng {
  doc "The world's main random stream, seeded from GameDesc.seed (or the replay seed). Its state is part of every snapshot."
  field seed u32 "The seed the stream started from." {
    readonly true
  }
  field state u32 "The generator state; snapshot and restore carry it." {
    readonly true
  }
  fn next {
    doc "Uniform in [0, 1)."
    returns f64
  }
  fn range {
    doc "Uniform in [low, high)."
    param low f64 "The inclusive lower bound."
    param high f64 "The exclusive upper bound."
    returns f64
  }
  fn integer {
    doc "A uniform integer in [low, high]."
    param low i32 "The inclusive lower bound."
    param high i32 "The inclusive upper bound."
    returns i32
  }
  fn chance {
    doc "True with probability p."
    param p f64 "The probability, between 0 and 1."
    returns bool
  }
  fn fork {
    doc "A stream for a subsystem, derived from the seed and a label, so drawing from it leaves the main sequence alone. The same label gives the same stream every time, on every host, wherever the main stream is; og.Rng among a system's reads is enough."
    param label String "The subsystem's name, such as `spawner`."
    returns RngStream
  }
}
```

<sub>`api.ogt` line 648</sub>

<a id="member-Rng-next"></a>

#### `Rng.next`

Uniform in [0, 1).

```ts
Rng.next(): number
```

**Returns** `number`.

Binding symbol `og_rng_next` · publish builds: kept

```ogt
fn next {
  doc "Uniform in [0, 1)."
  returns f64
}
```

<sub>`api.ogt` line 656</sub>

<a id="member-Rng-range"></a>

#### `Rng.range`

Uniform in [low, high).

```ts
Rng.range(low: number, high: number): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `low` | `number` | no | The inclusive lower bound. |
| `high` | `number` | no | The exclusive upper bound. |

**Returns** `number`.

Binding symbol `og_rng_range` · publish builds: kept

```ogt
fn range {
  doc "Uniform in [low, high)."
  param low f64 "The inclusive lower bound."
  param high f64 "The exclusive upper bound."
  returns f64
}
```

<sub>`api.ogt` line 660</sub>

<a id="member-Rng-integer"></a>

#### `Rng.integer`

A uniform integer in [low, high].

```ts
Rng.integer(low: number, high: number): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `low` | `number` | no | The inclusive lower bound. |
| `high` | `number` | no | The inclusive upper bound. |

**Returns** `number`.

Binding symbol `og_rng_integer` · publish builds: kept

```ogt
fn integer {
  doc "A uniform integer in [low, high]."
  param low i32 "The inclusive lower bound."
  param high i32 "The inclusive upper bound."
  returns i32
}
```

<sub>`api.ogt` line 666</sub>

<a id="member-Rng-chance"></a>

#### `Rng.chance`

True with probability p.

```ts
Rng.chance(p: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `p` | `number` | no | The probability, between 0 and 1. |

**Returns** `boolean`.

Binding symbol `og_rng_chance` · publish builds: kept

```ogt
fn chance {
  doc "True with probability p."
  param p f64 "The probability, between 0 and 1."
  returns bool
}
```

<sub>`api.ogt` line 672</sub>

<a id="member-Rng-fork"></a>

#### `Rng.fork`

A stream for a subsystem, derived from the seed and a label, so drawing from it leaves the main sequence alone. The same label gives the same stream every time, on every host, wherever the main stream is; og.Rng among a system's reads is enough.

```ts
Rng.fork(label: string): RngStream
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `label` | `string` | no | The subsystem's name, such as `spawner`. |

**Returns** `RngStream`.

Binding symbol `og_rng_fork` · publish builds: kept

```ogt
fn fork {
  doc "A stream for a subsystem, derived from the seed and a label, so drawing from it leaves the main sequence alone. The same label gives the same stream every time, on every host, wherever the main stream is; og.Rng among a system's reads is enough."
  param label String "The subsystem's name, such as `spawner`."
  returns RngStream
}
```

<sub>`api.ogt` line 677</sub>

## Interfaces

<a id="type-RngStream"></a>

### `RngStream` (interface)

One generator stream, forked from the Rng resource for a subsystem that must not perturb the main sequence.

```ogt
interface RngStream {
  doc "One generator stream, forked from the Rng resource for a subsystem that must not perturb the main sequence."
  # members follow, each quoted below
```

<sub>`api.ogt` line 617</sub>

<a id="member-RngStream-seed"></a>

#### `RngStream.seed` (property)

The stream's derived seed.

```ts
readonly seed: number
```

Binding symbol `og_rng_stream_get_seed` · publish builds: kept

```ogt
prop seed u32 "The stream's derived seed."
```

<sub>`api.ogt` line 619</sub>

<a id="member-RngStream-next"></a>

#### `RngStream.next`

Uniform in [0, 1).

```ts
RngStream.next(): number
```

**Returns** `number`.

Binding symbol `og_rng_stream_next` · publish builds: kept

```ogt
fn next {
  doc "Uniform in [0, 1)."
  returns f64
}
```

<sub>`api.ogt` line 620</sub>

<a id="member-RngStream-range"></a>

#### `RngStream.range`

Uniform in [low, high).

```ts
RngStream.range(low: number, high: number): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `low` | `number` | no | The inclusive lower bound. |
| `high` | `number` | no | The exclusive upper bound. |

**Returns** `number`.

Binding symbol `og_rng_stream_range` · publish builds: kept

```ogt
fn range {
  doc "Uniform in [low, high)."
  param low f64 "The inclusive lower bound."
  param high f64 "The exclusive upper bound."
  returns f64
}
```

<sub>`api.ogt` line 624</sub>

<a id="member-RngStream-integer"></a>

#### `RngStream.integer`

A uniform integer in [low, high].

```ts
RngStream.integer(low: number, high: number): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `low` | `number` | no | The inclusive lower bound. |
| `high` | `number` | no | The inclusive upper bound. |

**Returns** `number`.

Binding symbol `og_rng_stream_integer` · publish builds: kept

```ogt
fn integer {
  doc "A uniform integer in [low, high]."
  param low i32 "The inclusive lower bound."
  param high i32 "The inclusive upper bound."
  returns i32
}
```

<sub>`api.ogt` line 630</sub>

<a id="member-RngStream-chance"></a>

#### `RngStream.chance`

True with probability p.

```ts
RngStream.chance(p: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `p` | `number` | no | The probability, between 0 and 1. |

**Returns** `boolean`.

Binding symbol `og_rng_stream_chance` · publish builds: kept

```ogt
fn chance {
  doc "True with probability p."
  param p f64 "The probability, between 0 and 1."
  returns bool
}
```

<sub>`api.ogt` line 636</sub>

<a id="member-RngStream-fork"></a>

#### `RngStream.fork`

A fresh stream on a seed derived from this stream's seed and a label.

```ts
RngStream.fork(label: string): RngStream
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `label` | `string` | no | The subsystem's name; the same label always gives the same stream. |

**Returns** `RngStream`.

Binding symbol `og_rng_stream_fork` · publish builds: kept

```ogt
fn fork {
  doc "A fresh stream on a seed derived from this stream's seed and a label."
  param label String "The subsystem's name; the same label always gives the same stream."
  returns RngStream
}
```

<sub>`api.ogt` line 641</sub>
