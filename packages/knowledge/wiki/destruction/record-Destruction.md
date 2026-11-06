<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="record-Destruction"></a>

### `og.Destruction` (resource)

The world's destruction budgets.

Token `og.Destruction`, registered under the schema name `og.Destruction`; its value type is `Destruction`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `maxChunks` | `number` | `256` | 1 … 4096 | — | tunable | The most loose chunks alive at once, over every destructible: past it the oldest loose islands (the smallest first among those that broke in the same tick) despawn, each with every chunk it carries. |
| `debrisVolume` | `number` | `0.004` | ≥ 0 | `m3` | tunable | A loose island smaller than this is debris: it despawns debrisLifetime after it broke loose. |
| `debrisLifetime` | `number` | `8` | ≥ 0 | `s` | tunable | How long debris lasts (0: until maxChunks takes it). |
| `fxLifetime` | `number` | `3` | ≥ 0 | `s` | — | How long a breakFx or hitFx burst's entity lasts before og.destruction despawns it. |
| `stressIterations` | `number` | `0` | 0 … 1000 | — | — | The stress solver's iterations when a destructible's support changes (Blast's maxIter). 0, the default, solves until it converges (at most six times its bonds), so a large structure's weight reaches every bond it rests on; a cap is cheaper on a slow device, but an unconverged solve loads distant bonds too little, and what should fall can stand. |
| `stressBudget` | `number` | `200000` | ≥ 0 | — | tunable | The stress solver's work in one tick, across every destructible, in bond-iterations (one solver iteration over one bond). A solve that would start past it waits for the next tick, so a tick's worst case is this plus one solve (six iterations a bond) whatever comes down at once; the order is the destructibles' entity order, so a replay waits the same way. The 200-chunk brick wall's largest solve takes about 41000. 0 sets no limit. |

```ogt
resource Destruction {
  doc "The world's destruction budgets."
  field maxChunks u32 "The most loose chunks alive at once, over every destructible: past it the oldest loose islands (the smallest first among those that broke in the same tick) despawn, each with every chunk it carries." {
    default 256
    min 1
    max 4096
    tunable true
  }
  field debrisVolume f32 "A loose island smaller than this is debris: it despawns debrisLifetime after it broke loose." {
    default 0.004
    min 0
    unit m3
    tunable true
  }
  field debrisLifetime f32 "How long debris lasts (0: until maxChunks takes it)." {
    default 8
    min 0
    unit s
    tunable true
  }
  field fxLifetime f32 "How long a breakFx or hitFx burst's entity lasts before og.destruction despawns it." {
    default 3
    min 0
    unit s
  }
  field stressIterations u32 "The stress solver's iterations when a destructible's support changes (Blast's maxIter). 0, the default, solves until it converges (at most six times its bonds), so a large structure's weight reaches every bond it rests on; a cap is cheaper on a slow device, but an unconverged solve loads distant bonds too little, and what should fall can stand." {
    default 0
    min 0
    max 1000
  }
  field stressBudget u32 "The stress solver's work in one tick, across every destructible, in bond-iterations (one solver iteration over one bond). A solve that would start past it waits for the next tick, so a tick's worst case is this plus one solve (six iterations a bond) whatever comes down at once; the order is the destructibles' entity order, so a replay waits the same way. The 200-chunk brick wall's largest solve takes about 41000. 0 sets no limit." {
    default 200000
    min 0
    tunable true
  }
}
```

<sub>Module `destruction` line 120</sub>
