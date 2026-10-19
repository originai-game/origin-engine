# Archetypes round: break and restore

One playtest check per archetype was broken on purpose by changing the game
(never the test), seen to fail, and passed again once the change was
reverted. The runs use the native host (`og run` through
`packages/engine/src/node/playtest-run.ts`, as `og test` does).

## tower-defense (Path Guard)

Break: `pathsOpen` in `templates/archetypes/tower-defense/src/systems.ts`
answers `true` when the gate cannot reach the doorstep:

```diff
-  if (!reaches(GATE.x, GATE.y)) return false;
+  if (!reaches(GATE.x, GATE.y)) return true;
```

`tests/maze.og.ts` failed six of its thirteen checks: the hedge that closes
the last gap was planted and paid for, and the visitors walked through it.

```
FAIL maze (13 checks, 4806 ms): hedges route the visitors through the gap; a closed path, a stone, a taken cell and an empty purse are refused
  - a hedge in the gap would close the path: refused: expected 2, got 0
  - and not set: expected 6, got 7
  - and not paid for: expected 140, got 130
  - four refusals in all: expected 4, got 3
  - the first visitor crossed the hedge row (0 samples on it)
  - and walked on below it (y 12.80)
```

Restore: the line reverted, `maze` passes again:

```
PASS maze (13 checks, 4725 ms): hedges route the visitors through the gap; a closed path, a stone, a taken cell and an empty purse are refused
```
