# M7 knowledge mount

Engine knowledge package as committed at c20e014 (`--commit c20e014`), mounted into the OriginGame platform production tree (`packages/game-knowledge/production`). The working tree wiki was being regenerated concurrently and is not mounted.

## 1. First mount into a platform tree without the engine knowledge (copy of the platform production tree, og-* skills and origin-engine-ts removed)

```text
$ bun packages/knowledge/scripts/mount.ts <copy> --commit c20e014
mount: engine knowledge c20e014 -> <copy>/packages/game-knowledge/production
owned: game-scaffold/origin-engine-ts/, game/skills/og-build/, game/skills/og-core/, game/skills/og-game/
text files: 31; binaries left to distribution.json: 6
changes: 31 added, 0 changed, 0 removed
  added   game-scaffold/origin-engine-ts/.gitignore
  added   game-scaffold/origin-engine-ts/LICENSES.md
  added   game-scaffold/origin-engine-ts/assets/fonts/NotoSansSC-Regular-subset.ttf.ogt
  added   game-scaffold/origin-engine-ts/assets/fonts/OFL.txt
  added   game-scaffold/origin-engine-ts/assets/scenes/main.ogt
  added   game-scaffold/origin-engine-ts/assets/textures/stone_n.png.ogt
  added   game-scaffold/origin-engine-ts/distribution.json
  added   game-scaffold/origin-engine-ts/origin-engine.json
  added   game-scaffold/origin-engine-ts/package.json
  added   game-scaffold/origin-engine-ts/project.ogt
  added   game-scaffold/origin-engine-ts/src/main.ts
  added   game-scaffold/origin-engine-ts/tests/smoke.og.ts
  added   game-scaffold/origin-engine-ts/tsconfig.json
  added   game/skills/og-build/SKILL.md
  added   game/skills/og-core/SKILL.md
  added   game/skills/og-game/SKILL.md
  added   game/skills/og-game/reference/camera.md
  added   game/skills/og-game/reference/commands.md
  added   game/skills/og-game/reference/game.md
  added   game/skills/og-game/reference/index.md
  added   game/skills/og-game/reference/input.md
  added   game/skills/og-game/reference/load.md
  added   game/skills/og-game/reference/math.md
  added   game/skills/og-game/reference/probe.md
  added   game/skills/og-game/reference/reflect.md
  added   game/skills/og-game/reference/render.md
  added   game/skills/og-game/reference/rng.md
  added   game/skills/og-game/reference/system.md
  added   game/skills/og-game/reference/time.md
  added   game/skills/og-game/reference/transform.md
  added   game/skills/og-game/reference/world.md
```

## 2. Second mount: idempotent

```text
mount: engine knowledge c20e014 -> <copy>/packages/game-knowledge/production
owned: game-scaffold/origin-engine-ts/, game/skills/og-build/, game/skills/og-core/, game/skills/og-game/
text files: 31; binaries left to distribution.json: 6
changes: 0 added, 0 changed, 0 removed
```

## 3. The real platform checkout (/Volumes/app/origingame), after the mount

```text
$ bun packages/knowledge/scripts/mount.ts /Volumes/app/origingame --check --commit c20e014
check: engine knowledge c20e014 -> /Volumes/app/origingame/packages/game-knowledge/production
owned: game-scaffold/origin-engine-ts/, game/skills/og-build/, game/skills/og-core/, game/skills/og-game/
text files: 31; binaries left to distribution.json: 6
differences: 0 added, 0 changed, 0 removed
exit 0
$ diff -r <copy>/packages/game-knowledge/production /Volumes/app/origingame/packages/game-knowledge/production (mounted paths only)
identical: game/skills/og-build
identical: game/skills/og-core
identical: game/skills/og-game
identical: game-scaffold/origin-engine-ts
```

## 4. Binaries left to distribution.json (not in the UTF-8 payload)

```text
assets/audio/blip.wav  44144 bytes  sha256 4fe30d6e883337c7
assets/fonts/NotoSansSC-Regular-subset.ttf  2513672 bytes  sha256 9235b106a17bed31
assets/models/dropship.glb  562864 bytes  sha256 c7bf4c177bf360bc
assets/models/trickster.glb  1419580 bytes  sha256 7c40f67f386c609a
assets/textures/stone.png  112774 bytes  sha256 8622957415b5174f
assets/textures/stone_n.png  212633 bytes  sha256 c3bfb1778ace9010
```

## 5. Platform checks after the mount

- `bun run --cwd packages/game-knowledge typecheck`: passes.
- `bun run --cwd packages/game-knowledge test`: 178 pass, 3 fail. The three
  failures are in `production-bundle.test.mjs` and are the same before the
  mount: this checkout has Git LFS smudged, so
  `game-scaffold/origin-engine/origin/brand/splash.png` (added in platform
  `1e03549a6`) is a real PNG and `readProductionBundle` refuses non-UTF-8.
  `production.test.mjs` (20 skills) passes.
- The same `production-bundle.test.mjs` and `production.test.mjs` in a copy of
  the package with `splash.png` restored to its committed LFS pointer: all
  pass (155 files, 20 skills).
- `agent-server`: `bun test test/knowledge.test.ts test/prompts.test.ts`
  (packs loaded from the real production tree): 23 pass.
- Engine side: `bun run test` in `packages/knowledge` (validator + mount
  tests): 15 pass; `bun scripts/sync-rules.ts <platform> --check`: matches.

## What the platform schema needed that the package lacked

- Text only: the platform payload (`readProductionBundle`) is UTF-8 text;
  the template's six binaries must not be written into the tree. The mount
  writes text files only and leaves the binaries to `distribution.json`.
- Registration: `production.test.mjs` pins the skill inventory and
  `production-bundle.test.mjs` the payload file count; both needed the new
  numbers (20 skills, 155 files).
- Self-collision: once mounted, the package's own skills and overlay show
  up as "existing platform skills/overlays" to `sync-rules.ts`; it now
  excludes what the package mounts.
- Not enforced by any platform test, but carried by every other overlay:
  `.gitattributes` (LFS for media) and `.origin/setup` (snapshot
  preparation). `templates/game` has neither (asked in
  `docs/m7-platform-notes.md`).
