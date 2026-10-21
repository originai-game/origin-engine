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

## What works

The Kestrel Stair (`examples/kestrel-stair`, a 3D third-person action
game written from this package alone) baked its navmesh from the level's
glTF with one `model` line (§1), and five agents ran on it: imps on a
behaviour tree (patrol, chase, a 0.25 s wind-up and a bite, flight when
hurt) and wardens on a machine (patrol, alert, chase, slam, search), each
raising the game's commands, with sight occluded by the level,
`og.Steering` separation and avoidance, and `og.nav.setObstacle` closing
the summit gate until it rose. Two processes of a full session of chases
end on one world hash.

## 1. Pick the navigation

| World | Navigation | How |
|---|---|---|
| A tilemap (top-down, dungeon) | a grid from its collision | `og.nav.gridFromTilemap(world, { entity: map })` (`inflate` blocks cells around walls for wide agents) |
| An open 2D or 3D arena | a grid from costs | `og.nav.grid(world, { width, height, cellSize, origin, costs })`; costs 0 blocked, 1 normal, up to 255 to avoid |
| Rooms, slopes, floors in 3D | a navmesh baked at cook time | `assets/nav/<level>.ogt` of `type navmesh` (every line below) gives the agent, the cell and tile sizes, the bounds and the geometry; `og cook` voxelises it; `og.nav.load(world, "nav/<level>.ogt")` answers `{ mesh }` |
| Many units to one goal (crowds, tower defence) | a flow field | `og.nav.flowField(world, { grid, targets })`; one field serves every agent |

Doors and crates that block for a while are navmesh obstacles
(`og.nav.setObstacle`): only the tiles under them rebuild. Never bake a
navmesh at run time; the wasm does not carry the voxeliser.

### The navmesh file

The geometry is every `scene`, `model`, `terrain` and `box` entry
together (at least one must give triangles); every other line tunes the
bake. Every line, with its default:

```text
type navmesh
version 1
---
agent {                        # the agent the mesh is for
  radius 0.4                   # metres kept from walls: the floor is eroded by it
  height 1.8                   # headroom: floor under a lower ceiling is left out
  climb 0.4                    # the highest step or ledge walked up or down
  slope 45                     # degrees: a steeper triangle is not floor
}
cellSize 0.2                   # metres across a voxel column (half the radius or less keeps narrow doors open)
cellHeight 0.1                 # metres of a voxel's height
tileSize 32                    # cells per tile side, 8 to 256: an obstacle rebuilds only the tiles it touches
maxError 1.3                   # cells a simplified wall outline may stray from the voxels
scene "scenes/level.ogt"       # a scene's static geometry: its models, and its static colliders as boxes
model "models/level.glb" {     # every triangle of a glTF's default scene (repeatable)
  position 0 0 0               # where the level places it (default 0 0 0)
  quaternion 0 0 0 1           # x y z w (default 0 0 0 1)
  scale 1 1 1                  # (default 1 1 1)
}
terrain "terrain/vale.ogt" {   # a terrain's surface (repeatable)
  position 0 0 0               # where its entity stands (default 0 0 0)
  step 1                       # every step-th height sample (default 1: all of them)
}
box {                          # a solid box the level lacks: a plinth, a crate (repeatable)
  center 0 0.5 0               # (default 0 0 0)
  size 1 1 1                   # whole sizes (default 1 1 1)
  quaternion 0 0 0 1           # (default 0 0 0 1)
}
bounds {                       # the box baked; left out: the triangles' own bounds
  min -10 -10 -10              # (default -10 -10 -10 inside the block)
  max 10 10 10                 # (default 10 10 10)
}
```

- Floors are triangles facing up (glTF's counter-clockwise front faces);
  give the agent the body's radius and height, and `climb` the stair's
  riser or more.
- A scene's dynamic bodies are left out; its sphere and capsule colliders
  bake as their bounding boxes, convex and mesh colliders only through
  their model. A terrain bakes only the cells inside `bounds`.
- `bounds` cuts what the agents may reach: a min y a little over a lake's
  surface leaves the lake a hole they walk round.
- The cook refuses any other line (naming it), a scene, model or terrain
  missing under `assets/`, no triangles at all, and a level over 16
  million columns (raise `cellSize` or bound it); its report notes the
  triangles, cells, polygons and tiles it baked.

The Kestrel Stair's, baked from its glTF level (`tools/level.ts` writes it
beside the model):

```text
type navmesh
version 1
---
agent {
  radius 0.5
  height 2.6
  climb 0.3
  slope 36
}
cellSize 0.25
cellHeight 0.1
tileSize 32
model "models/stair.glb" {
  position 0 0 0
}
bounds {
  min -23 -1 -67
  max 23 14 45
}
```

Before writing a mind, prove the mesh: load it in `start` and check
`og.nav.nearest(world, mesh, spawn)` at every spawn point and route
point (null, or a point far from it, is a place the agents cannot stand;
og-game's order, step 5).

## 2. The agent

```ts
const post = og.nav.nearest(world, yard.mesh, spawn) ?? spawn; // on the mesh
world.set(guard, og.Transform, { position: post });
world.insert(guard, og.NavAgent, { nav: yard.mesh, target: post, seeking: true, speed: 1.5, acceleration: 12, radius: 0.4, faceMovement: true, turnSpeed: 8 });
world.insert(guard, og.AiAgent, { behaviour: mind.behaviour });
world.insert(guard, og.Perception, { sight: 9, fov: 120, eyeHeight: 1.6, team: 2 });
world.insert(hero, og.Perceivable, { team: 1, eyeHeight: 1 });
```

- Place every agent on the mesh, at its `og.nav.nearest` point (idle, or
  seeking a target there: an agent that has not planned yet stays where
  it was put, and a teleported one walks on from where it lands), and
  prove it in a playtest: a few ticks in, each agent stands where it was
  put (its height too) and its `status` is not `NAV_UNREACHABLE`.
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
      check targetDistance below Tuning.slimeReach   # compared with a resource field, read every think
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
  a higher branch that becomes true takes over); a **machine** (below) for
  NPCs whose mode should stick (patrol, alert, search), with enter and
  exit actions and timed transitions.
- Checks read the blackboard's built-in keys, your own keys (`key alert
  0`, `set alert 1`), `Component.field` of the agent and `Resource.field`,
  compared with a literal or with another key or field read each time
  (`check Foe.hp below Tuning.fleeHp`). An unknown operand fails its check
  and logs one warning naming the fix: read the run's warnings.
- Keep damage, death, loot and knock-back in the game: a behaviour raises
  `command bite` and the game's command handler applies it. Tune numbers
  live in components and resources (`speed Tuning.slimeSpeed`, `check
  targetDistance below Tuning.slimeReach`), never as copies in the .ogt.
- A new run or a respawn: put the agent back (its Transform, its
  NavAgent's target with `seeking: false`) and call
  `og.ai.restart(world, agent)`: a machine starts again in its initial
  state, a tree at its root, timers, cooldowns and the blackboard reset.
- Telegraph attacks: a `wait 0.3` or a wind-up state before the command
  (og-feel §2). An enemy that hits on the tick it arrives feels unfair.
- `thinkEvery 2` or `3` for crowds: agents are staggered, so the cost
  spreads across ticks.

### The behaviour file

A file holds a name, its keys, its routes and one `tree` or one
`machine`. Every line of a tree, with its default:

```text
type behaviour
version 1
---
name "slime"                   # what dump_ai shows (default "behaviour")
key alert 0                    # a blackboard key of yours and its start: a number, x y z, or true/false (repeatable)
# og.ai's own keys, written every think, which checks read and `key` or `set` may not name: self,
# position, sees, heard, target, targetPosition, targetDistance, lastSeen, sinceSeen, heardAt,
# sinceHeard, elapsed (seconds in the machine's state)
route beat {                   # a patrol route, declared before the tree names it (repeatable)
  point 11 6.5 0               # x y z (at least one)
}
tree {                         # one root node; or a `machine` (below), never both
  selector main {              # children in order: the first that does not fail decides; re-checked from the top every think
    sequence hurt {            # children in order while they succeed; re-checked from the first every think
      check Slime.hp below 30  # <operand> [is | not | below | above | atMost | atLeast <value>]; no operator: truthy; the value may be a key or field
      flee {                   # flee [key]: away from the key's position (default target)
        distance 6             # until this far (default 6)
        speed 3                # metres per second, or a key or field read every think (default: the NavAgent's speed)
      }
    }
    steps attack {             # a sequence that resumes at its running child
      move_to lastSeen {       # move_to <key> | <x y z>: succeeds on arrival, fails when unreachable
        speed Tuning.slimeSpeed
      }
      wait 0.3                 # runs this many seconds, then succeeds
      command bite 2 {         # the game's command: the agent's handle, then the words after the name
        cooldown 1             # fails for this long after it ran (default 0)
      }
    }
    parallel watch {           # every child every think
      policy all               # all (default): succeeds when all do, fails when one does | one: the reverse
      chase {                  # chase [key]: after an entity key's position (default target)
        reach 0.5              # succeeds within this (default 0.5)
      }
      repeat 0 {               # its one child, N times while it succeeds (0: forever)
        cooldown 2 {           # after its child succeeds, fails for this many seconds
          timeout 5 {          # fails when its child runs longer than this
            invert {           # success is failure and failure success
              succeed {        # success whatever the child answers (running stays running)
                set alert 1    # a key of yours to a value; succeeds
              }
            }
          }
        }
      }
    }
    patrol beat {              # round a route, resuming at the nearest point
      wait 0.6                 # seconds at each point (default 0)
      loop true                # false: succeeds at the route's end (default true)
    }
    wander {                   # to random points round where it started
      radius 4                 # (default 4)
      wait 1                   # seconds at each (default 0)
    }
    stop                       # stops moving; succeeds
  }
}
```

- An action without `speed` moves at its og.NavAgent's `speed`. A value
  in `key`, `set` and `check` is a number, three numbers (a point) or
  `true`/`false`; a `check`'s may also be a key, `Component.field` or
  `Resource.field`, read every time the check runs (`check Foe.hp below
  Tuning.fleeHp`). Command names are snake_case.
- The cook refuses an unknown entry, node or option, a route that is not
  declared, a decorator without exactly one child and a malformed value,
  naming the line.

A machine replaces the tree. Every think it checks the current state's
transitions in order and takes the first that holds (running the exit
actions, then the new state's enter actions), then ticks the state's
nodes as one sequence. The Kestrel Stair's warden:

```text
type behaviour
version 1
---
name "warden-0"
route round {
  point 6 4 -2
  point 6 4 -14
  point -4 4 -12
}
machine {
  initial patrol               # the state it starts in (default: the first)
  state patrol {
    patrol round {             # a state's nodes, ticked while in it
      wait 1.6
      speed Tuning.wardenPatrol
    }
    to alert {                 # to <state>: when every check holds ...
      check sees
      check Game.phase is 1
    }
  }
  state alert {
    enter {                    # instant actions on entering: set, stop, command (an `exit { }` block: on leaving)
      command alert
    }
    chase {
      speed Tuning.wardenChase
      reach 1.6
    }
    to slam {
      check targetDistance below Tuning.wardenReach
    }
    to search {
      check sees is false
    }
    to patrol {
      check Game.phase not 1
    }
  }
  state slam {
    enter {
      command slam
    }
    stop
    to alert {
      after 1.8                # ... and the state has lasted this many seconds (default 0)
    }
  }
  state search {
    move_to lastSeen {
      speed Tuning.wardenPatrol
    }
    to alert {
      check sees
      check Game.phase is 1
    }
    to patrol {
      after 4
    }
  }
}
```

## 4. Perception

- `fov` 90–140 for guards (360 for things that sense all round), `sight`
  a little past the screen's half width, `occlude: true` so walls hide.
  Before an agent first moves its cone points the way its Transform faces.
- Hearing: spatial `og.AudioSource`s that start within `hearing × volume`
  are heard, except the agent's own (its entity's and its children's: an
  imp's chitter does not alert the imp); `og.ai.noise(world, { position,
  loudness, source })` for silent events (a thrown rock, a door).
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
  `templates/archetypes/third-person` is the 3D action pattern: sentinels on
  a cooked navmesh with behaviour machines (patrol, alert, grab) raising
  the game's commands, restarted for a new run (`tests/sentinels.og.ts`,
  `tests/restart.og.ts`).
  Include the negative: a wall that hides the hero, a target outside the
  cone.
- Determinism: a playtest with `runs: 2` over a chase must end on one hash.
- `og polish` reports damage moments without hit feedback and inputs nobody
  answered (og-build §5); enemies that bite with no hit stop show up there.
