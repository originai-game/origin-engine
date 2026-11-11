<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `system` module

Systems: functions the engine calls once per tick with their matched archetypes.

A system declares what it reads and writes; a write to an undeclared component is refused at registration with the field name. Its query is every component of its reads and writes (none of `without`). world.query is refused inside one: another archetype is a system of its own, and what crosses between systems goes in a resource. Resources and events it reaches through the world must be declared too: undeclared, they throw in a dev or release build and fail the tick; publish refuses them, logged once per system.

- **TypeScript**: functions are named exports of the package: `import { … } from "@origingame/engine"`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Core** (what most games use): `SystemDesc`, `defineSystem`; 10 more are advanced (`og api show system --all`).
- **Index**: [every module and guide](index.md).

```ogt
module system {
  doc "Systems: functions the engine calls once per tick with their matched archetypes."
  doc "A system declares what it reads and writes; a write to an undeclared component is refused at registration with the field name. Its query is every component of its reads and writes (none of `without`). world.query is refused inside one: another archetype is a system of its own, and what crosses between systems goes in a resource. Resources and events it reaches through the world must be declared too: undeclared, they throw in a dev or release build and fail the tick; publish refuses them, logged once per system."
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
