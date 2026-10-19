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

## roguelike (Lamp Dive)

Break: `startDive` in `templates/archetypes/roguelike/src/ui.ts` keeps the
last dive's strike instead of starting from the tuning's (permadeath
broken: a blade found survives the lamp going out):

```diff
-  g.attack = t.heroAttack;
+  if (g.attack === 0) g.attack = t.heroAttack;
```

`tests/lose.og.ts` failed the check that the next dive starts from nothing:

```
FAIL lose (13 checks, 21265 ms): the lamp goes out: the summary, and the next dive starts from nothing
  - from nothing: floor 1, full health, the blade gone, no kills, no turns
```

Restore: the line reverted, `lose` passes again:

```
PASS lose (13 checks, 17711 ms): the lamp goes out: the summary, and the next dive starts from nothing
```
