# og explore (`packages/engine/src/node/explore.ts`)

`og explore` searches a game's states for invariant violations, systems
that throw, and the assertion sites a run should reach. It is written in
this repository from the papers and public documentation below. No code
is taken from any implementation: several of them are under licences the
engine cannot take code from (noted per source).

## The search: Go-Explore

- Adrien Ecoffet, Joost Huizinga, Joel Lehman, Kenneth O. Stanley, Jeff
  Clune. *Go-Explore: a New Approach for Hard-Exploration Problems.*
  arXiv:1901.10995, 2019. <https://arxiv.org/abs/1901.10995>
- Adrien Ecoffet, Joost Huizinga, Joel Lehman, Kenneth O. Stanley, Jeff
  Clune. *First return, then explore.* Nature 590, 580–586, 2021.
  <https://doi.org/10.1038/s41586-020-03157-9>

What explore takes from them:
- An archive of cells, each a state abstraction with the shortest
  trajectory known to reach it and a way to return there. The engine's
  whole-world snapshot and restore (with the input, RestoreDesc.session)
  is the return; the papers restore an emulator's state.
- Cells from domain knowledge: here, each reflected field bucketed, which
  the game narrows with GameDesc.explore.
- Choosing a cell by count-based weights (1/√(1 + times chosen) and
  1/√(1 + times seen)), then exploring from it with random actions that
  repeat for a few steps.
- A cell reached again by a shorter trajectory takes it.

Explore's own additions: the time as 64 slices of the horizon, with half
the returns choosing a state and then one of its times and half a time
slice and then one of its cells, and a bonus for a cell that brought a new
field bucket or a new assertion firing.

The official implementation (<https://github.com/uber-research/go-explore>)
is under the Uber Non-Commercial License; none of it is used or
consulted for code.

## Goals and findings: Antithesis

- Antithesis, *Sometimes assertions* and *Assertions in Antithesis*,
  documentation, 2024–2026. <https://antithesis.com/docs/properties_assertions/>

The assertion kinds (always, sometimes, reachable, unreachable) are the
engine's og.assert (`crates/og-api/api/assert.ogt`); explore takes from
Antithesis the use of sometimes and reachable sites as the search's
guidance (a firing is novelty) and of their coverage as the report, and a
finding as a deterministic, replayable input history.

## State as coverage: AFL, libFuzzer, IJON

- Michał Zalewski, *american fuzzy lop*, technical whitepaper, 2014–2016.
  <https://lcamtuf.coredump.cx/afl/technical_details.txt>. AFL is under
  Apache-2.0 (<https://github.com/google/AFL>).
- AFL++ (<https://github.com/AFLplusplus/AFLplusplus>) is under
  AGPL-3.0-or-later as a whole, with some files under Apache-2.0; none of
  it is used.
- LLVM, *libFuzzer – a library for coverage-guided fuzz testing*,
  documentation. <https://llvm.org/docs/LibFuzzer.html>. Apache-2.0 with
  LLVM exceptions.
- Cornelius Aschermann, Sergej Schumilo, Ali Abbasi, Thorsten Holz.
  *IJON: Exploring Deep State Spaces via Fuzzing.* IEEE S&P 2020.
  <https://doi.org/10.1109/SP40000.2020.00117>

What explore takes from them: AFL's hit-count buckets, applied to field
values instead of edge counts (small integers exact, larger ones by power
of two); "new bits" and libFuzzer's features as the novelty that keeps a
cell (a field's bucket never seen); IJON's point that a deep state space
is searched by annotating the program's state, which og.assert and the
reflected fields already are.

## Shrinking: delta debugging

- Andreas Zeller, Ralf Hildebrandt. *Simplifying and Isolating
  Failure-Inducing Input.* IEEE TSE 28(2), 183–200, 2002.
  <https://doi.org/10.1109/32.988498>

ddmin over a trace's presses (a press's down and up go together), each
candidate replayed from tick 0 and kept when it still reaches the finding
by the same tick.

## Licences

Engine code only, under the repository's MIT licence. The papers and
documentation are cited for their methods.
