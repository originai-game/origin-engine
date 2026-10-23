# The game template

What `npm create @origingame/game` writes: the smallest whole Origin Engine
project, and the hello of the engine's design grown into a scene. A menu's
Start button (or S, or Enter) spins the trickster and drops a crate onto
the ground; a guard patrols the yard, sees only inside its cone and not
through the wall, raises the alarm, chases, searches and goes back.

```sh
og run .     # headless with the probe; og dev . reloads on save
og test .    # tests/smoke.og.ts, tests/guard.og.ts and tests/intro.og.ts
og polish .  # the taste and feel scores of those playtests
og build .   # the play page
```

## What it demonstrates, and the skills it exercises

| Part | Where | Skill |
|---|---|---|
| The project: `project.ogt`, the entry's `og.game` default export, `cooked/` from `assets/` | `project.ogt`, `src/main.ts` | og-game §2, §5 |
| Components, a resource, systems over archetype columns, one job each | `Health`, `Velocity`, `Spin`, `Run`; `Move`, `StartSpin`, `Drop`, `Rotate` | og-core §4, §5; og-game §3 |
| The frame's art direction as data: a look extending a catalogue preset, applied by `GameDesc.look` | `assets/looks/origin-game.ogt` (a created game's is `<id>.ogt`), `og.game({ look })` | og-taste §1, §12 |
| A text scene spawned before the first tick, a glTF model, a font, a box with a collider that falls | `assets/scenes/main.ogt`, `start` | og-game §4, §6, §13 |
| A menu panel whose button answers with a look and a sound (og sfx's click preset) | `Menu`, `assets/audio/click.ogt` | og-ui §1, §3; og-taste §6; og-audio §2, §3 |
| Every word the menu shows from a string table, the base one in the source language | `assets/strings/en.ogt`, `Menu` | og-i18n §1, §2 |
| A guard as data: a state machine on og.ai, a navmesh cooked from the yard, sight cones against the wall | `assets/ai/guard.ogt`, `assets/nav/yard.ogt` | og-ai §1–§5 |
| Commands with their usage in `help`, facts for playtests | `commands`, `commandHelp`, `facts` | og-game §3; og-build §1 |
| A cutscene as data: a camera on a rail, the trickster's clips and walk, narration from an `.srt`, sounds, an event key running `spawn_wave`, fades and markers, played by `timeline:play:intro` | `assets/timelines/intro.ogt`, `narration.srt`, `tests/intro.og.ts` | og-game §9 "Cutscenes" |
| Playtests that assert on dumps and facts, never on the code | `tests/` | og-build §3, §4 |

`og polish .` (2026-10-19): taste 100, feel 87 (tests/evidence/E7/polish/
in the engine repository). The feel row below 90 is idle frames: the guard
playtest stages its scenario without pressing Start, so for half its ticks
only the guard stands still at a corner of its search, and the smoke
playtest waits half a second on the title before its press. A game made
from the template should keep a playtest that plays it.

Asset licences: `LICENSES.md`.
