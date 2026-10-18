# og-ai

Enemy and NPC AI as data (`docs/design.md` §2.7): behaviour trees and
state machines declared in `.ogt` (`type behaviour`), perception (sight
cones against physics occlusion, hearing from sounds and noises),
per-agent blackboards, and the `og.ai` system. Pure Rust over og-core,
og-ecs, og-nav's records and og-audio's events; builds for wasm32.

## The asset

```text
type behaviour
version 1
---
name "slime"
key alert 0                      # blackboard keys: a number, x y z, or true/false
route beat {                     # patrol routes
  point 11 6.5 0
  point 16 6.5 0
}
tree {
  selector {
    sequence flee {
      check Slime.hp below 20    # a component field of the agent
      flee {
        distance 6
      }
    }
    sequence attack {
      check sees
      check targetDistance below 0.9
      command bite {             # the game's command, with the agent's handle
        cooldown 1
      }
    }
    sequence chase {
      check sees
      chase {
        speed 2
      }
    }
    patrol beat {
      wait 0.6
    }
  }
}
```

| Node | Does |
|---|---|
| `selector [label]` | children in order; the first that does not fail decides; re-checked every tick, so a higher branch that becomes true halts a lower one |
| `sequence [label]` | children in order while they succeed; re-checked from the first every tick |
| `steps [label]` | a sequence that resumes at its running child |
| `parallel [label] { policy all\|one; ... }` | every child every tick |
| `invert`, `succeed`, `repeat N`, `cooldown S`, `timeout S` | decorators of one child |
| `check <operand> [is\|not\|below\|above\|atMost\|atLeast <value>]` | a blackboard key, `Component.field` of the agent or `Resource.field` (a `.x/.y/.z` lane of a vector); no operator: truthy |
| `move_to <key \| x y z> { speed }` | to a point; succeeds on arrival, fails when unreachable |
| `chase [key] { speed; reach }` | after an entity key's position (default `target`) |
| `flee [key] { speed; distance }` | away until `distance` away |
| `patrol <route> { wait; speed; loop }` | round a route, resuming at the nearest point |
| `wander { radius; speed; wait }` | to random points around where it started |
| `wait S`, `stop`, `set <key> <value>` | |
| `command <name> [args...] { cooldown }` | queues the game command `name` with the agent's handle first, run before the next tick; fails while cooling down |

A machine replaces the tree:

```text
machine {
  initial idle
  state idle {
    enter {
      set mood 1          # instant actions: set, stop, command
    }
    patrol post           # the state's body: nodes, ticked while in it
    to alert {            # transitions in order; every check holds and `after` seconds passed
      check heard
    }
  }
  state alert {
    move_to heardAt
    to idle {
      after 1.5
    }
  }
}
```

Built-in keys, written every think: `self`, `position`, `sees`, `heard`,
`target`, `targetPosition`, `targetDistance`, `lastSeen`, `sinceSeen`,
`heardAt`, `sinceHeard`, `elapsed` (seconds in the state).

## Records

- `og.AiAgent` (component): `behaviour`, `active`, `thinkEvery`; written:
  `status` (`AI_RUNNING`, `AI_SUCCESS`, `AI_FAILURE`), `state`, `node`,
  `changes`.
- `og.Perception` (component): `sight`, `fov`, `eyeHeight`, `occlude`,
  `occluders`, `hearing`, `memory`, `team`, `forward`; written: `target`,
  `sees`, `lastSeen`, `sinceSeen`, `heard`, `heardAt`, `sinceHeard`.
- `og.Perceivable` (component): `team`, `eyeHeight`.
- `og.AiChanged` (event): `entity`, `state`, `node`, `status`, when the
  state or the running node changes.

## The system

`install(world, records, AiDesc { plane, ray })` registers the records and
adds `og.ai`, which the runtime runs before the game's systems (og.nav
moves agents after them). Per tick, in query order: perception (the
nearest perceivable of another team within sight, inside the cone around
`forward` or the way the agent last moved, not behind a collider by the
host's `SightRay`; spatial sounds that started and `noise` calls within
`hearing × loudness`), then every `thinkEvery` ticks the tree or machine,
then the running action's target and speed go to the agent's
`og.NavAgent` (no moving action stops it) and its commands to the queue
(`take_commands`). Checks read the agent's components and the world's
resources from their packed bytes, by the names the runtime passes. An
unknown operand fails its check and logs one warning naming the fix.

State: the `og.ai` block holds every agent's memory (node timers, the
machine's state and when it started, the blackboard, the path to the
running node), the ticks targets were last seen and heard, the way agents
last moved and pending noises.

## Tests

`cargo test -p og-ai` (`tests/ai.rs`; each broken once,
`tests/evidence/E6/break-and-restore.md`): malformed behaviours refused
with their line; a guard that patrols, chases on sight, bites on a
cooldown and flees when a component field drops; a resource fact that
redirects it and a selector that gives the patrol back; walls, the view
cone and range hiding a target and hearing by radius; a machine on facts
and timers with its enter and exit actions; two runs and a restored
snapshot deciding the same.
