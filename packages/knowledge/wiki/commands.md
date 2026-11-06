<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `commands` module

The command channel: actions scheduled on simulation time, from `og run --commands "t:action[:args];..."` or the probe's commands().

Core actions ship with the engine; a module or a game registers more. Unknown actions warn and continue. Not in publish builds.

Outputs are relative to the output directory: an absolute path, a `..` component or a name starting with `.` (kept for the harness, such as a playtest's kept store) is refused. A file an earlier step of the script wrote is never overwritten: the later step's is written as `<stem>-2.<ext>` with a warning naming both steps.

- **TypeScript**: functions are the namespace `og.commands`; components, resources and events are tokens on `og`.
- **Publish builds**: dropped (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module commands {
  doc "The command channel: actions scheduled on simulation time, from `og run --commands \"t:action[:args];...\"` or the probe's commands()."
  doc "Core actions ship with the engine; a module or a game registers more. Unknown actions warn and continue. Not in publish builds."
  doc "Outputs are relative to the output directory: an absolute path, a `..` component or a name starting with `.` (kept for the harness, such as a playtest's kept store) is refused. A file an earlier step of the script wrote is never overwritten: the later step's is written as `<stem>-2.<ext>` with a warning naming both steps."
  publish false
  # declarations follow, each on its own page
```

<sub>`api.ogt` line 23833</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`CoreAction`](commands/type-CoreAction.md) | enum |  | The actions the engine ships. |
| [`CommandHandler`](commands/type-CommandHandler.md) | callback |  | A registered action. |
| [`CommandDesc`](commands/type-CommandDesc.md) | desc struct |  | How a registered action describes itself in `og run --help` and the wiki. |
| [`CommandStep`](commands/type-CommandStep.md) | struct |  | One parsed step of a command script. |
| [`CommandInfo`](commands/type-CommandInfo.md) | struct |  | One action as help lists it. |
| [`CommandsReport`](commands/type-CommandsReport.md) | struct |  | What scheduling a script did. |
| [`og.commands.register`](commands/fn-register.md) | function | `(name: string, handler: CommandHandler, desc?: CommandDesc): Result<void>` | Register an action into the command channel. |
| [`og.commands.parse`](commands/fn-parse.md) | function | `(script: string): Result<readonly CommandStep[]>` | Parse a script without scheduling it. |
| [`og.commands.help`](commands/fn-help.md) | function | `(world: World): readonly CommandInfo[]` | Every action the channel knows: core actions in CoreAction order, then registered ones by name. |
| [`og.commands.schedule`](commands/fn-schedule.md) | function | `(world: World, script: string): Result<CommandsReport>` | Schedule a script on simulation time, from the current tick. |
