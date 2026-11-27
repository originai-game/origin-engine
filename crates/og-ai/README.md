# og-ai

Enemy and NPC AI as data (`docs/design.md` §2.7): checked behaviour trees
and tick state machines, perception, per-agent blackboards and the `og.ai`
system. Pure Rust over og-core and og-ecs, including wasm32. It links no
audio: the host hands it the tick's sounds through `AiDesc::hearing`.

## Author and runtime boundaries

Games export `file.behaviour` from `content/**/*.ts` using
`@origingame/engine/author`. The generated `file:behaviour` schema lists
the complete input. Subtrees use canonical `ref('base/ai/patrol')` tokens;
all reached declarations and helper imports participate in cook invalidation.
There are no OGT behaviour files, directory lookups or text parsers.

The same `definition::Behaviour<R>` shape compiles through
`Behaviour::from_definition`. The reference resolver supplies native trees;
the caller owns reference identity and cycle detection. Cook resolves
checked project inputs; `og.ai.create(world, definition)` resolves cooked
subtree ID strings. `Behaviour::to_bytes/from_bytes` validates native
indices, numeric domains, keys, routes and state transitions. Loading needs
no source files or sibling assets: subtrees are already inlined.

## Execution

- `selector` and `sequence` recheck from the first child each think.
  `steps` resumes its running child, restarting after failure or completion.
  These semantics follow BehaviorTree.CPP, not its XML or plugin ABI.
- A higher-priority branch halts the lower running branch and resets its
  sequence cursor. Game actions receive `AI_INTERRUPTED`. Commands keep
  their explicit cooldowns.
- `invert` flips success/failure; `succeed` converts either to success.
  Both preserve running. Parallel completion policy, repeat counts,
  decorator durations and utility scoring/inertia are stated by the game.
- Checks read keys, agent component fields or resource fields, with a
  literal `value` or a dynamic `against` operand. `truthy` takes neither.
  Unknown operands fail and report the behaviour and node.
- Movement runs through `og.NavAgent`. Speeds are fixed numbers or operand
  names; omitted speeds retain the agent's starting speed. Commands queue
  before the next tick; game actions run until `og.ai.finish`.
- Machines run at most one transition per think: exit, destination entry,
  then its explicit body. Initial entry runs once, including after a
  restart; restoring a snapshot preserves it. Self-transitions run exit
  and entry. Only set, stop and command are instant entry/exit actions.
  There are no event queues, parallel states or run-to-completion loops.
- Subtrees merge keys with parent declarations taking precedence. Routes
  of the same name must have identical points; patrol indices are remapped.

## Perception and state

Built-in keys are `self`, `position`, `sees`, `heard`, `target`,
`targetPosition`, `targetDistance`, `lastSeen`, `sinceSeen`, `heardAt`,
`sinceHeard` and `elapsed`. They cannot be declared or set by a behaviour.
Sight uses team, range, view cone and host occlusion rays; hearing uses
radius and world positions through parent chains. Agents do not hear
their own sounds or sounds from their children.

`install(world, records, AiDesc)` installs perception then decisions, before
the game's systems; navigation moves afterwards. Agents run in query order.
Wander and sight are deterministic. The `og.ai` state block snapshots
blackboards, timers, machine entry state, perception memory, pending noises
and navigation targets. `AiSystem::restart` resets those on the next think,
keeps the initial speed, and uses the agent's new position as wander home.

The public AI contract test round-trips native fields and checks two runs
plus snapshot/restore. The cook contract covers checked tree/machine input,
canonical subtree resolution, invalid definitions and transitive invalidation.
