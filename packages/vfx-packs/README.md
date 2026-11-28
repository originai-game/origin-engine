# Downloadable effect packs

Each `<pack>/` here (an `extension.json` of `kind` `"vfx"`, docs/vfx-packs.md
§1) ships with every engine release as `<version>/vfx/<pack>.tar.gz`, not in
the engine archives, and a game installs it with `og vfx add <pack>`
(docs/vfx-packs.md §5, docs/release.md §1). Text lives in Git; binary textures
and previews are pinned in `og-assets.json` (docs/ci.md §The asset store).
Nothing under a pack's `preview/` is archived. The few packs every engine carries are
in `packages/engine/extensions/` instead; a pack is in one place, never both.

| Pack | Effects |
|---|---|
| `cinematic-realism` | 89 realistic effects, independent of any look: explosions, fire and smoke, muzzle flashes, impacts and marks, sparks, steam, dust and fog. The bundled `realistic-starter` supplies seven smaller realistic effects. Discover both with `og vfx list`. |
| `cartoon` | 59 (hits, shooting, explosions, movement, water, pickups and rewards, status, magic, environment, destruction; flat 2D, supersampled, 2 px warm contour); Cartoon |
| `low-poly` | 81 (72 bright tactile mesh effects plus 9 hard-geometry mechanisms under `geometric/`); Low-poly |
| `effekseer-andrewfm01` | 15 (beam, explosion, fire, hit-spark, magic-impact, magic-projectile, portal, smoke, ui) |
| `effekseer-effekseer01` | 15 (basic, beam) |
| `effekseer-hato01` | 4 (ambient) |
| `effekseer-magicalxspiral` | 101 (explosion, fire, magic-impact, magic-projectile, portal, shield, slash, smoke, weather) |
| `effekseer-nextsoft01` | 20 (buff, fire, heal, magic-impact, magic-projectile, shield, slash, weather) |
| `effekseer-pierre01` | 11 (ambient, buff, fire, magic-impact, magic-projectile, weather) |
| `effekseer-pierre02` | 12 (explosion, heal, magic-impact, magic-projectile, portal, slash, smoke) |
| `effekseer-suzuki01` | 3 (buff, slash, weather) |
| `effekseer-tktk01` | 71 (buff, fire, heal, hit-spark, magic-impact, magic-projectile, slash) |
| `effekseer-tktk02` | 33 (fire, hit-spark, magic-impact, magic-projectile, slash) |
