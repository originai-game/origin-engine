# @origingame/engine-knowledge

What an agent needs to write a game on Origin Engine, as the platform's
knowledge packs (`packages/game-knowledge/production` of OriginGame):

| Part | Path | Mounted at (`knowledge.json`) |
|---|---|---|
| API wiki, one page per IDL module plus `index.md` | `wiki/` (generated: `og api gen --wiki`; `og api check` fails on drift) | `game/skills/og-game/reference/` |
| Skills `og-core` (engine rules), `og-build` (the `og` loop and the evidence discipline), `og-game` (ECS, scenes, assets, loading) | `skills/<name>/SKILL.md` | `game/skills/<name>/` |
| Scaffold: the project template, its engine pin and binary digests | `scaffold/` and the engine repository's `templates/game` | `game-scaffold/origin-engine-ts/` |

`bun run test` validates the package against the platform's rules
(`rules/platform.json`, transcribed from the platform by
`bun scripts/sync-rules.ts <origingame checkout>`; each rule names its
source) and runs the tests in `tests/`. `bun scripts/validate.ts --bundle`
prints the text payload in the platform's `readProductionBundle` shape
(`{ schemaVersion: 1, files }`).
