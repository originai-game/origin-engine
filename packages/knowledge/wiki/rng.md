<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `rng` module

Deterministic randomness. Every random draw comes from an Rng stream; the sequence for a seed is identical on every host and in the Web scaffold's rng.ts.

Inside a system: forking needs og.Rng among the reads (a fork is derived from the seed and the label, and leaves the main stream where it is); drawing from og.Rng itself (next, range, integer, chance, weighted, permutation) moves its state, so it needs og.Rng among the writes. A stream a fork returned is the system's own to draw from.

- **TypeScript**: functions are none at module level (its members live on the types below); components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module rng {
  doc "Deterministic randomness. Every random draw comes from an Rng stream; the sequence for a seed is identical on every host and in the Web scaffold's rng.ts."
  doc "Inside a system: forking needs og.Rng among the reads (a fork is derived from the seed and the label, and leaves the main stream where it is); drawing from og.Rng itself (next, range, integer, chance, weighted, permutation) moves its state, so it needs og.Rng among the writes. A stream a fork returned is the system's own to draw from."
  # declarations follow, each on its own page
```

<sub>Module `rng` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`RngStream`](rng/type-RngStream.md) | interface |  | One generator stream, forked from the Rng resource for a subsystem that must not perturb the main sequence. |
| [`og.Rng`](rng/record-Rng.md) | resource |  | The world's main random stream, seeded from GameDesc.seed (or the replay seed). |
