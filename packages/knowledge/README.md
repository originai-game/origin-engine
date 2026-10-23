# @origingame/engine-knowledge

What an agent needs to write a game on Origin Engine, as the platform's
knowledge packs (`packages/game-knowledge/production` of OriginGame):

| Part | Path | Mounted at (`knowledge.json`) |
|---|---|---|
| API wiki, one page per IDL module plus `index.md` | `wiki/` (generated: `og api gen --wiki`; `og api check` fails on drift) | `game/skills/og-game/reference/` |
| Skills, numbered `## N.` sections cross-referenced as `og-feel §4`: `og-game` (the entry point and the map of the others: the project, ECS, scenes, assets, loading, input, 2D, animation, effects, atmosphere and water, material shaders, physics, performance, looks, light and shadows), `og-core` (engine rules), `og-build` (the `og` loop, the evidence discipline, and `og polish`: §5 is the one list of what it measures), `og-feel` (game feel: time scale, hit stop, tweens, camera, juice amounts, controller tunables by genre), `og-ui` (screens, HUDs, styles, motion, text entry, accessibility, gestures), `og-taste` (art direction: a look chosen from the premise out of the style catalogue `looks/catalogue.json`, atmosphere by intent, the done checklist), `og-ai` (navigation and enemy minds as data), `og-audio` (events, presets, the mix, music), `og-i18n` (string tables, fonts, line breaking, right to left), `og-platform-sdk` (`og.platform`: saves, settings, leaderboards, telemetry, and how to ask for what is missing; the platform's own `og-platform` covers the Player). Each IDL module's `skill` names the one that teaches it; the wiki's index groups modules by it, and `bun run test` checks that skill names the module's page, no other does, and every section reference resolves | `skills/<name>/SKILL.md` | `game/skills/<name>/` |
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

`bun scripts/mount.ts <origingame checkout> [--check] [--commit <rev>]`
writes the payload into the platform checkout's
`packages/game-knowledge/production` (text files only; it owns
`game/skills/og-*/` and `game-scaffold/origin-engine-ts/` there and removes
what the package no longer has); `--check` exits 1 on any difference and
`--commit` mounts a commit instead of the working tree. What the platform
side looks like: the repository's `docs/m7-platform-notes.md`.
