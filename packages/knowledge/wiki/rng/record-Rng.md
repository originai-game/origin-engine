<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`rng`](../rng.md) · [index](../index.md)

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
  fn weighted {
    doc "An index drawn in proportion to its weight (one draw): `weighted([1, 3])` is 1 three times in four. Zero, negative and non-finite weights are never drawn; -1 when none is positive."
    param weights List.f64 "One weight per choice."
    returns i32
  }
  fn permutation {
    doc "0 to n - 1 in a drawn order (Fisher-Yates, n - 1 draws). A shuffled copy of an array is `rng.permutation(a.length).map((i) => a[i])`."
    aliases "shuffle, random order, Fisher-Yates, shuffle a deck"
    param n u32 "How many indices."
    returns List.u32
  }
  fn fork {
    doc "A stream for a subsystem, derived from the seed and a label, so drawing from it leaves the main sequence alone. The same label gives the same stream every time, on every host, wherever the main stream is; og.Rng among a system's reads is enough."
    param label String "The subsystem's name, such as `spawner`."
    returns RngStream
  }
}
```

<sub>Module `rng` line 46</sub>

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

<sub>Module `rng` line 54</sub>

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

<sub>Module `rng` line 58</sub>

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

<sub>Module `rng` line 64</sub>

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

<sub>Module `rng` line 70</sub>

<a id="member-Rng-weighted"></a>

#### `Rng.weighted`

An index drawn in proportion to its weight (one draw): `weighted([1, 3])` is 1 three times in four. Zero, negative and non-finite weights are never drawn; -1 when none is positive.

```ts
Rng.weighted(weights: readonly number[]): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `weights` | `readonly number[]` | no | One weight per choice. |

**Returns** `number`.

Binding symbol `og_rng_weighted` · publish builds: kept

```ogt
fn weighted {
  doc "An index drawn in proportion to its weight (one draw): `weighted([1, 3])` is 1 three times in four. Zero, negative and non-finite weights are never drawn; -1 when none is positive."
  param weights List.f64 "One weight per choice."
  returns i32
}
```

<sub>Module `rng` line 75</sub>

<a id="member-Rng-permutation"></a>

#### `Rng.permutation`

0 to n - 1 in a drawn order (Fisher-Yates, n - 1 draws). A shuffled copy of an array is `rng.permutation(a.length).map((i) => a[i])`.

Also known as: shuffle, random order, Fisher-Yates, shuffle a deck

```ts
Rng.permutation(n: number): readonly number[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `n` | `number` | no | How many indices. |

**Returns** `readonly number[]`.

Binding symbol `og_rng_permutation` · publish builds: kept

```ogt
fn permutation {
  doc "0 to n - 1 in a drawn order (Fisher-Yates, n - 1 draws). A shuffled copy of an array is `rng.permutation(a.length).map((i) => a[i])`."
  aliases "shuffle, random order, Fisher-Yates, shuffle a deck"
  param n u32 "How many indices."
  returns List.u32
}
```

<sub>Module `rng` line 80</sub>

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

<sub>Module `rng` line 86</sub>
