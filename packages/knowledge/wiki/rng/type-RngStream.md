<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`rng`](../rng.md) · [index](../index.md)

<a id="type-RngStream"></a>

### `RngStream` (interface)

One generator stream, forked from the Rng resource for a subsystem that must not perturb the main sequence.

```ogt
interface RngStream {
  doc "One generator stream, forked from the Rng resource for a subsystem that must not perturb the main sequence."
  # members follow, each quoted below
```

<sub>Module `rng` line 6</sub>

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

<sub>Module `rng` line 8</sub>

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

<sub>Module `rng` line 9</sub>

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

<sub>Module `rng` line 13</sub>

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

<sub>Module `rng` line 19</sub>

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

<sub>Module `rng` line 25</sub>

<a id="member-RngStream-weighted"></a>

#### `RngStream.weighted`

An index drawn in proportion to its weight (one draw): `weighted([1, 3])` is 1 three times in four. Zero, negative and non-finite weights are never drawn; -1 when none is positive.

```ts
RngStream.weighted(weights: readonly number[]): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `weights` | `readonly number[]` | no | One weight per choice. |

**Returns** `number`.

Binding symbol `og_rng_stream_weighted` · publish builds: kept

```ogt
fn weighted {
  doc "An index drawn in proportion to its weight (one draw): `weighted([1, 3])` is 1 three times in four. Zero, negative and non-finite weights are never drawn; -1 when none is positive."
  param weights List.f64 "One weight per choice."
  returns i32
}
```

<sub>Module `rng` line 30</sub>

<a id="member-RngStream-permutation"></a>

#### `RngStream.permutation`

0 to n - 1 in a drawn order (Fisher-Yates, n - 1 draws). A shuffled copy of an array is `rng.permutation(a.length).map((i) => a[i])`.

```ts
RngStream.permutation(n: number): readonly number[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `n` | `number` | no | How many indices. |

**Returns** `readonly number[]`.

Binding symbol `og_rng_stream_permutation` · publish builds: kept

```ogt
fn permutation {
  doc "0 to n - 1 in a drawn order (Fisher-Yates, n - 1 draws). A shuffled copy of an array is `rng.permutation(a.length).map((i) => a[i])`."
  param n u32 "How many indices."
  returns List.u32
}
```

<sub>Module `rng` line 35</sub>

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

<sub>Module `rng` line 40</sub>
