<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-PhysicsStats"></a>

### `PhysicsStats` (struct)

What the last physics step cost, phase by phase, and where it ran: for finding where physics time goes. Timings are wall clock and vary run to run; the simulation never reads them.

See `physics §3`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | — | — | The tick the step ran (0 before the first). |
| `bodies` | `number` | `0` | — | — | — | The rapier bodies it stepped (with ragdoll bones and vehicle wheels). |
| `threads` | `number` | `0` | — | — | — | The lanes pool's threads at the time (1: lanes off or single-threaded). |
| `pool` | `boolean` | `false` | — | — | — | Whether the whole step ran on the lanes pool (from a number of bodies per thread count), where rapier splits its broad phase, narrow phase and solver. |
| `ms` | `number` | `0` | — | `ms` | — | Wall-clock time of the whole og.physics system, gather to write-back. |
| `phases` | `readonly PhysicsPhase[]` | `[]` | — | — | — | Every phase, in the order they run. |

```ogt
struct PhysicsStats {
  doc "What the last physics step cost, phase by phase, and where it ran: for finding where physics time goes. Timings are wall clock and vary run to run; the simulation never reads them."
  guide "physics §3"
  field tick f64 "The tick the step ran (0 before the first)."
  field bodies u32 "The rapier bodies it stepped (with ragdoll bones and vehicle wheels)."
  field threads u32 "The lanes pool's threads at the time (1: lanes off or single-threaded)."
  field pool bool "Whether the whole step ran on the lanes pool (from a number of bodies per thread count), where rapier splits its broad phase, narrow phase and solver."
  field ms f64 "Wall-clock time of the whole og.physics system, gather to write-back." {
    unit ms
  }
  field phases List.PhysicsPhase "Every phase, in the order they run."
}
```

<sub>Module `physics` line 2164</sub>
