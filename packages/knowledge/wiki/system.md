<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `system` module

Systems: functions the engine calls once per tick with their matched archetypes.

A system declares what it reads and writes; a write to an undeclared component is refused at registration with the field name.

Its query is every component of its reads and writes together: an entity matches only when it has all of them (and none of `without`). A component declared to read one entity with world.get (the player's car) therefore narrows the loop to entities that have it too, often to none, silently. world.query is refused inside a system, by design (a system reaches only what it declared, so its access is known before it runs): loop over each archetype in a system of its own, declaring only that archetype's components, and carry what crosses between systems in a resource.

Resources and events it reaches through the world must be declared too: in a dev or release build (og run, og test) an undeclared read or write throws with the system's name and the record and fails the tick; a publish build refuses it and logs it once per system in the engine log, and the game keeps running. og.Rng among the reads may fork (a fork reads the seed only and leaves the stream as it is); a draw moves the stream, so it needs og.Rng among the writes.

- **TypeScript**: functions are named exports of the package: `import { … } from "@origingame/engine"`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module system {
  doc "Systems: functions the engine calls once per tick with their matched archetypes."
  doc "A system declares what it reads and writes; a write to an undeclared component is refused at registration with the field name."
  doc "Its query is every component of its reads and writes together: an entity matches only when it has all of them (and none of `without`). A component declared to read one entity with world.get (the player's car) therefore narrows the loop to entities that have it too, often to none, silently. world.query is refused inside a system, by design (a system reaches only what it declared, so its access is known before it runs): loop over each archetype in a system of its own, declaring only that archetype's components, and carry what crosses between systems in a resource."
  doc "Resources and events it reaches through the world must be declared too: in a dev or release build (og run, og test) an undeclared read or write throws with the system's name and the record and fails the tick; a publish build refuses it and logs it once per system in the engine log, and the game keeps running. og.Rng among the reads may fork (a fork reads the seed only and leaves the stream as it is); a draw moves the stream, so it needs og.Rng among the writes."
  place top
  # declarations follow, each on its own page
```

<sub>Module `system` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`StateWhen`](system/type-StateWhen.md) | enum |  | When a system bound to a state machine's value runs. |
| [`StateDesc`](system/type-StateDesc.md) | struct |  | A state machine (GameDesc.states): a name and the values it moves between; it holds one at a time, and world.setState moves it as the next tick begins. |
| [`RunCondition`](system/type-RunCondition.md) | callback |  | Whether a system runs this tick: called before its run, after the systems before it, with the system's reads and writes as read-only access. |
| [`SystemDesc`](system/type-SystemDesc.md) | desc struct |  | A system's declared access and order. |
| [`HookEvent`](system/type-HookEvent.md) | enum |  | What a hook reacts to. |
| [`HookDesc`](system/type-HookDesc.md) | desc struct |  | A hook's event and declared access. |
| [`HookRun`](system/type-HookRun.md) | callback |  | A hook's body: called once per entity the component is added to or removed from, wherever the change came from (a system's structural changes as they apply when it returns, or a call between ticks), … |
| [`Hook`](system/type-Hook.md) | interface |  | A declared hook. |
| [`defineHook`](system/fn-defineHook.md) | function | `<C extends AnyComponent>(name: string, component: C, desc: HookDesc, run: HookRun): Hook` | Declare a hook on a component; list it in GameDesc.hooks. |
| [`SystemRun`](system/type-SystemRun.md) | callback |  | A system's body. It is called once per tick; loop over `q.archetypes()`, then over rows. |
| [`System`](system/type-System.md) | interface |  | A registered system. |
| [`defineSystem`](system/fn-defineSystem.md) | function | `(name: string, desc: SystemDesc, run: SystemRun): System` | Declare a system: the game's code that runs every tick (movement, AI, scoring), with the components it reads and writes and its order among the other systems. |
