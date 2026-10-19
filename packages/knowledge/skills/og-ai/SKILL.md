---
name: og-ai
description: Enemies and NPCs on Origin Engine — navigation with og.nav (grids from costs or a tilemap's collision, cooked navmeshes, flow fields, agents that plan and steer) and minds with og.ai (behaviour trees and state machines as .ogt data, sight cones and hearing, blackboards, the game's commands), and how to prove them with dump_ai, dump_nav and playtests. Use when a game needs anything that chases, patrols, flees, guards or wanders, or when an enemy's behaviour is wrong.
---

An enemy on Origin Engine is three records on an entity and one data file:
`og.NavAgent` (where it goes, moved by `og.nav`), `og.AiAgent` (which
behaviour it runs, ticked by `og.ai`), `og.Perception` (what it sees and
hears), and a behaviour in `assets/ai/<name>.ogt`. Game code does not move
enemies or switch their states by hand; it sets facts (components,
resources) the behaviour checks, and handles the commands it raises. The
API is in `reference/nav.md` and `reference/ai.md`; this skill is how to
use them.

## 1. Pick the navigation

| World | Navigation | How |
|---|---|---|
| A tilemap (top-down, dungeon) | a grid from its collision | `og.nav.gridFromTilemap(world, { entity: map })` (`inflate` blocks cells around walls for wide agents) |
| An open 2D or 3D arena | a grid from costs | `og.nav.grid(world, { width, height, cellSize, origin, costs })`; costs 0 blocked, 1 normal, up to 255 to avoid |
| Rooms, slopes, floors in 3D | a navmesh baked at cook time | `assets/nav/<level>.ogt` of `type navmesh` gives the agent (radius, height, climb, slope), the cell and tile sizes, the bounds and the geometry (`box`, `scene` or `model` entries); `og cook` voxelises it; `og.nav.load(world, "nav/<level>.ogt")` answers `{ mesh }` |
| Many units to one goal (crowds, tower defence) | a flow field | `og.nav.flowField(world, { grid, targets })`; one field serves every agent |

Doors and crates that block for a while are navmesh obstacles
(`og.nav.setObstacle`): only the tiles under them rebuild. Never bake a
navmesh at run time; the wasm does not carry the voxeliser.

## 2. The agent

```ts
world.insert(guard, og.NavAgent, { nav: yard.mesh, speed: 1.5, acceleration: 12, radius: 0.4, faceMovement: true, turnSpeed: 8 });
world.insert(guard, og.AiAgent, { behaviour: mind.behaviour });
world.insert(guard, og.Perception, { sight: 9, fov: 120, eyeHeight: 1.6, team: 2 });
world.insert(hero, og.Perceivable, { team: 1, eyeHeight: 1 });
```

- `radius` keeps grid paths clear of walls; set it to the body's.
- `faceMovement` turns the body to where it walks (3D +Z forward, 2D +X).
- `drive: false` keeps planning and reporting but leaves the motion to the
  game: set it during a knock-back or a stun, back to true after.
- og.nav re-plans when the target moves past `repathDistance`, every
  `repathTicks`, and when the agent is pushed more than 1.5 m (or four
  radii) off its path. `status` says `NAV_ARRIVED` or `NAV_UNREACHABLE`;
  read it rather than measuring distances.

## 3. Write the mind as data

```text
type behaviour
version 1
---
name "slime"
route beat {
  point 11 6.5 0
  point 16 6.5 0
}
tree {
  selector {
    sequence flee {
      check Slime.hp below 30        # a component field of the agent
      flee {
        distance 6
      }
    }
    sequence bite {
      check sees
      check targetDistance below 0.9
      command bite {                 # the game's command, the agent's handle first
        cooldown 1
      }
    }
    sequence chase {
      check sees
      chase {
        speed Tuning.slimeSpeed      # a resource field, read every think
      }
    }
    patrol beat {
      wait 0.6
    }
  }
}
```

- A **tree** for reactive enemies (re-checked from the top every think, so
  a higher branch that becomes true takes over); a **machine** (`machine {
  initial idle; state idle { ...; to alert { check heard } } }`) for NPCs
  whose mode should stick (patrol, alert, search) with enter actions and
  timed transitions (`after 3`).
- Checks read the blackboard's built-in keys (`sees`, `heard`, `target`,
  `targetDistance`, `lastSeen`, `sinceSeen`, `heardAt`, `elapsed`), your
  own keys (`key alert 0`, `set alert 1`), `Component.field` of the agent
  and `Resource.field`. An unknown operand fails its check and logs one
  warning naming the fix: read the run's warnings.
- Keep damage, death, loot and knock-back in the game: a behaviour raises
  `command bite` and the game's command handler applies it. Tune numbers
  live in components and resources (`speed Tuning.slimeSpeed`), never as
  copies in the .ogt.
- Telegraph attacks: a `wait 0.3` or a wind-up state before the command
  (og-feel §2). An enemy that hits on the tick it arrives feels unfair.
- `thinkEvery 2` or `3` for crowds: agents are staggered, so the cost
  spreads across ticks.

## 4. Perception

- `fov` 90–140 for guards (360 for things that sense all round), `sight`
  a little past the screen's half width, `occlude: true` so walls hide.
  Before an agent first moves its cone points the way its Transform faces.
- Hearing: spatial `og.AudioSource`s that start within `hearing × volume`
  are heard; `og.ai.noise(world, { position, loudness })` for silent events
  (a thrown rock, a door).
- `memory` is how long a lost target stays the target; `lastSeen` lets a
  search state walk to where it was.

## 5. Prove it

- `dump_ai:<entity>[:file]` writes the behaviour, state, running node, the
  path from the root, status, blackboard and perception;
  `dump_nav:<entity>[:file]` the path, the corner it heads for and status.
- A playtest per behaviour beat (`templates/archetypes/topdown/tests/ai.og.ts`
  and `flee.og.ts`, `templates/game/tests/guard.og.ts`): place the hero with
  a command, advance, dump, assert `node`, `state`, `sees` and positions.
  `templates/archetypes/roguelike/tests/foes.og.ts` proves a turn-based
  hunt (a tree that acts only on `Game.foesTurn`) and
  `templates/archetypes/tower-defense/tests/maze.og.ts` a flow field
  re-routed round placements and a placement refused because `findPath`
  no longer reaches the goal.
  Include the negative: a wall that hides the hero, a target outside the
  cone.
- Determinism: a playtest with `runs: 2` over a chase must end on one hash.
- `og polish` reports damage moments without hit feedback and inputs nobody
  answered (og-build §5); enemies that bite with no hit stop show up there.
