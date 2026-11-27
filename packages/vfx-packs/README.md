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
| `cinematic-realism` | 89 (weapons and projectiles, impacts by material with their marks, explosions, vehicles, fire, smoke and water, character states, interaction and destruction, magic, ambience, ui; one-shot effects carry a synthesised sound); cinematic-realism and muted |
| `cartoon-2d` | 59 (hits, shooting, explosions, movement, water, pickups and rewards, status, magic, environment, destruction; flat 2D, supersampled, 2 px warm contour); cartoon-2d |
| `cartoon-2d-watercolor` | 59 (the same effects in light washes with a pigment rim); cartoon-2d watercolor |
| `cartoon-2d-storybook` | 59 (the same effects with a thin coloured line); cartoon-2d storybook |
| `cartoon-2d-rubber-hose` | 59 (the same effects in faded primaries with a 3 px black ink line); cartoon-2d rubber-hose |
| `geometric` | 9 (explosion, hit-spark, gunfire, buff/pickup, magic-impact, smoke, splash, ambient/trail, ui); minimal-geometric and faceted-low-poly |
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
