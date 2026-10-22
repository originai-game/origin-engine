# Changelog

One version across `@origingame/engine` (the knowledge inside it as
`knowledge/`), `@origingame/engine-darwin-arm64`, `@origingame/engine-linux-x64`
and `@origingame/create-game` (docs/release.md §3). Before 1.0.0 a minor
bump carries API additions or changes, one line each, written at the merge
that changes `crates/og-api/api.ogt`; a patch bump carries fixes and frames.

## 0.2.0 (2026-10-23)

- Windows x64: build and pack the MSVC `og.exe` and napi addon as
  `@origingame/engine-win32-x64`, with DX12 rendering and the same TypeScript
  API. `og check` deliberately uses the pinned tsgo/Bun fallback, reported
  by doctor. Windows paths, source line endings and HDR fixture checkout
  bytes are portable. Release validation and remaining frame differences:
  `tests/evidence/release/windows-x64.md`.

API additions and changes (a minor bump, §3), by the stream that landed
each:

- API: every word of the kit HUDs is a field with its English default
  (kit-words, 2026-10-22): `PlatformerHudDesc.jumpLabel`,
  `ShooterHudDesc.killsLabel` and `fireLabel`, `PuzzleHudDesc.movesLabel`
  and `targetLabel` (`{target}`), `IdleHudDesc.levelLabel` (`{level}`) and
  `rateLabel` (`{rate}`), and `pauseLabel` on each of them (the pause
  button's name for a screen reader); a localised game passes its string
  table's (`og.i18n.t`). Additive; the defaults are the words the HUDs
  showed before.

The `platformer-2d` stream (Brasswick's runtime findings,
`examples/brasswick/FINDINGS.md`, 2026-10-22):

- `og.WorldRestored` (event, module world): delivered the tick after a
  `world.restore` applies, with `from` (the tick the world was at) and
  `tick` (the saved tick), for systems that reset what lives in the session
  (#8). `world.restore`'s doc says so, and that an `og run --commands`
  script goes on to its last step whichever way a restore moved og.Time.
- `og.PARTICLES_ARMED` (const, module render): `ParticleSystem.startTick`'s
  default, never written; a one-shot (`looping false`) emitter stays armed
  and emits nothing until a tick is written, a looping one emits as if
  started at tick 0 (#3). `startTick` carries the default.
- `og.CharacterController.pushStrength` and
  `og.PlatformerController.pushBodies` (default true) and `pushStrength`
  (newtons, default 1000, tunable): the push on the dynamic bodies a
  controller walks or runs into is an impulse capped by the strength, so a
  light body is shoved and a heavy one holds (#6). Unit `newton` added.
- `og.TilemapLayer.colliderLayers` and `colliderMask`: a collision layer's
  own `og.Collider` layers and mask, from its `.ogt` line or Tiled int
  property; 0 takes `Tilemap.colliderLayers` and every layer (#5).
- `PlatformAnswer.time`: a slot call (`saveSlot`, `loadSlot`) answers the
  time of the store answer that ended it (#7). `CharacterEvent`'s doc says
  the event is read the tick after the move (#14).

Also:

- `og run --commands`: the script ends when its last step has run, wherever
  a restore moved og.Time (#13).
- `create-game --profile 2d` writes the 2D starter `templates/game-2d`
  (shipped as `template-2d/`), as does a premise that says 2D, platformer,
  side-scroller, metroidvania or pixel art without saying 3D (#1). The
  install test scaffolds and tests it.
- Knowledge: og-game §8 gives the flipbook grammar line by line (#2);
  og-feel §12 reads the jump event the tick after (#14).

The `dogfood-7` stream (Rumpus Isle, a local-multiplayer party game,
`examples/rumpus-isle/FINDINGS.md`, 2026-10-22):

- `og test --filter <path>`: selects suite files by a substring of their
  project-relative path before anything runs; an empty selection fails
  (#14). No API change.
- Knowledge: og-build on headless rendering (`OG_RENDERER=none`, the
  lavapipe fallback) (#7); og-ui on the UI's entrance timing and the
  presses it rejects (#10) and the iris midpoint (#12); og-game on the
  post-solver velocity a contact event carries (#9).

## 0.1.1 (2026-10-22)

- Fix: the published template, archetypes and knowledge scaffold carried no
  `.gitignore` (npm drops the file from every tarball), so a scaffolded game
  and the platform's overlay had none. Stored as `gitignore` in the packages
  and written back as `.gitignore` by `create-game` and the knowledge mount,
  as `gitattributes` already was. The install test checks both files.
- No API change.

## 0.1.0 (2026-10-22)

The first published version: the API as `crates/og-api/api.ogt` stands at
the tag (modules math, reflect, world, system, game, transform, time, rng,
input, camera, render, load, physics, atmosphere, anim, tween, tilemap,
audio, nav, ai, platform, i18n, ui, systems, commands, probe, terrain,
stream), the `og` tool (run, dev, test, polish, build, cook, probe, check,
sfx, skin, doctor, api), the project template and eight archetypes, the
ten `og-*` skills and the generated wiki. Everything before it is the
build-out recorded in `docs/phase-1.md`.
