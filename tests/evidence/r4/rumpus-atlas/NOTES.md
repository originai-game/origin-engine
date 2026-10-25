# Rumpus cartoon atlas reconciliation

Branch `r4/rumpus-atlas`, based on candidate `42206f80` plus the accepted
contact-frame recording `52d7553d` (the coordinator's equivalent is
`e5d08f07`). Only this branch's final commit needs cherry-picking there.

The old ignored atlas had SHA-1 `526fa0a2b60166086dd05519f8ac42ccf47bd22f`.
The committed cartoon atlas has `4218685acafe94ae31b89c60b5fa7b368436f25b`.
`bun examples/rumpus-isle/tools/make-assets.ts` copied the current pack,
then the existing frame suite recorded all eleven Metal references:

```sh
OG_FX_CAPTURE=1 OG_ENGINE_PACKAGE="$PWD/packages/engine" packages/engine/native/og.darwin-arm64 test examples/rumpus-isle --filter frames.og.ts
OG_ENGINE_PACKAGE="$PWD/packages/engine" packages/engine/native/og.darwin-arm64 test examples/rumpus-isle --filter frames.og.ts
python3 tests/evidence/r4/rumpus-atlas/compare.py
```

Both frame runs passed. The second reports 30 checks, zero pixel differences
for every reference. The dev addon was built from the candidate source;
no full gate was run for this fixture-only change, per the coordinator.

## Atlas-only differences from the accepted contact frames

Outliers mean any RGB channel differs by more than 32/255. The existing
limits remain 0.5% outliers and RGB RMSE 0.01; no tolerance changed.
Every before/after/difference panel was inspected. Changes are confined
to UI atlas borders and corners, from capsule shapes to the current cartoon
radii. No terrain, water, islands, palms, critters, berries, or podium geometry
changed. UI labels and layout are unchanged.

| Frame | Changed pixels | Outliers % | RMSE | Changed UI region |
|---|---:|---:|---:|---|
| title | 12,937 | 0.39562 | 0.02476 | Title panel, Play and Settings outlines |
| lobby | 17,833 | 0.66602 | 0.03254 | Lobby panel, player-card/control edges, colour/ready/back/start buttons |
| card | 11,947 | 0.48014 | 0.02793 | Top HUD, countdown card, bottom player badges |
| bump | 5,742 | 0.26367 | 0.01858 | Top HUD/Pause and bottom player badges |
| splash | 6,107 | 0.27387 | 0.01884 | Top HUD/Pause and bottom player badges |
| scores | 13,477 | 0.42394 | 0.02493 | Score dialog and Next round button |
| berry | 6,389 | 0.28114 | 0.01906 | Top HUD/Pause and bottom player badges |
| crumble | 5,985 | 0.26986 | 0.01875 | Top HUD/Pause and bottom player badges |
| pause | 23,361 | 0.67491 | 0.03014 | Pause dialog/buttons, HUD and badges |
| settings-fr | 16,136 | 0.45747 | 0.02531 | Settings dialog, Terminé, HUD and badges |
| podium | 13,599 | 0.45540 | 0.02819 | Results dialog and button outlines |

## The guard rejects stale inputs before recording

The frame test compares SHA-256 of the copied `atlas.png` and `icons.png`
against `templates/styles/cartoon`. A mismatch fails with
`run bun tools/make-assets.ts` and returns before the recorder is called.
There is no new shared helper or dependency.

Verified two negative controls with `OG_FX_CAPTURE=1`:

- The real stale atlas fails after one check (`stale-atlas.log`).
- The current atlas with one byte of icon pixel content changed fails on
  the icons check (`stale-icons.log`). The original icons were restored.
- Neither negative control modified any reference, verified by `git diff`.

The Mac main checkout's ignored Rumpus atlas and icons were refreshed from
its committed cartoon pack. Both now match their sources; main's branch,
commits and tracked working tree are unchanged. Its `git status --short`
remained empty. No owner process was stopped during this task.
