# @origingame/engine-knowledge

What an agent needs to write a game on Origin Engine, as the platform's
knowledge packs (`knowledge/` of origin-agent, the platform's agent
repository; `packages/game-knowledge/production` of OriginGame before the
agent-repo split):

| Part | Path | Mounted at (`knowledge.json`) |
|---|---|---|
| API wiki, one page per IDL module plus `index.md` | `wiki/` (generated: `og api gen --wiki`; `og api check` fails on drift) | `game/skills/og-game/reference/` |
| Skills, numbered `## N.` sections cross-referenced as `og-feel §4`: `og-game` (the entry point and the map of the others: the project, ECS, scenes, assets, loading, input, 2D, animation, effects, atmosphere and water, material shaders, physics, performance, looks, light and shadows), `og-core` (engine rules), `og-build` (the `og` loop, the evidence discipline, and `og polish`: §5 is the one list of what it measures), `og-feel` (game feel: time scale, hit stop, tweens, camera, juice amounts, controller tunables by genre), `og-ui` (screens, HUDs, styles, motion, text entry, accessibility, gestures), `og-taste` (art direction: a look chosen from the premise out of the style catalogue `looks/catalogue.json`, atmosphere by intent, the done checklist), `og-ai` (navigation and enemy minds as data), `og-audio` (events, presets, the mix, music), `og-i18n` (string tables, fonts, line breaking, right to left), `og-model` (a 3D model built as code from a reference image: the intake's detail inventory, eight gated passes, the og-geom generators' recipes, budgets, the `type model` asset, and the review's `view` and `turntable` frames and `og compare` sheet), `og-platform-sdk` (`og.platform`: saves, settings, leaderboards, telemetry, and how to ask for what is missing; the platform's own `og-platform` covers the Player). Six craft skills beside them teach no module and point to the og-* skills for the API: `architecture-discipline` (the project's layers, splitting, the first playable, loud failure, tuning in data), `component-forge` (a unit built as code from a measured reference), `level-texturing` (a gray level painted from views with `og bake projection`; its driver is `scripts/texture-level.ts` beside it), `localization` (the discipline around og-i18n), `recipe-craft` (the retrieval ladder for example code, vendoring, distilling) and `visual-acceptance` (judging frames against references, baselines and the play camera). Each IDL module's `skill` names the one that teaches it; the wiki's index groups modules by it, and `bun run test` checks that skill names the module's page, no other does, and every section reference resolves | `skills/<name>/SKILL.md` | `game/skills/<name>/` |
| Guides: hand-written pages an agent reads beside a skill, held to the same rules as the skills (every `og.*` name and `Type.field` in the API, section references resolved, each named by og-game, listed in `GUIDES`): `three-to-og` maps three.js's classes and idioms to the engine's and names the gaps; `og migrate three` cites its sections | `guides/<name>.md` | `game/skills/og-game/guides/` |
| Scaffold: the project template, its engine pin and binary digests | `scaffold/` and the engine repository's `templates/game` | `game-scaffold/origin-engine-ts/` |

`bun run test` validates the package against the platform's rules
(`rules/platform.json`, transcribed from the platform by
`bun scripts/sync-rules.ts <origingame checkout>`; each rule names its
source) and runs the tests in `tests/`. It also holds the skills to the IDL: every
`og.*` name a skill writes (`og.render.applyLook`, `og.Sprite.array`,
code blocks included) and every `Type.field` of a struct, desc, component,
resource or event (`SpriteDesc.dropShadow`) must exist in
`packages/engine/schema/api.schema.json`; the few `og.` names that are not
API (a store key, an engine-owned component the dumps show) are listed
with their reason in `NOT_API`. `bun scripts/validate.ts --bundle`
prints the text payload in the platform's `readProductionBundle` shape
(`{ schemaVersion: 1, files }`).

`bun scripts/mount.ts <origin-agent checkout> [--check] [--commit <rev> | --package <dir>]`
writes the payload into the checkout's `knowledge/` (text files only; it
owns `game/skills/<each of its skills>/` and `game-scaffold/origin-engine-ts/` there and
removes what the package no longer has); `--check` exits 1 on any
difference, `--commit` mounts a commit instead of the working tree and
`--package` the knowledge of an installed engine. origin-agent itself runs
`scripts/knowledge/mount-engine.ts`, which imports `installed`, `mount` and
`summary` from the installed engine's `knowledge/scripts/mount.ts` with its
own knowledge root: those three signatures are a public contract
(AGENTS.md). What the platform side looked like at M7: the repository's
`docs/m7-platform-notes.md`.
