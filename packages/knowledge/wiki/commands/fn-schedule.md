<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`commands`](../commands.md) · [index](../index.md)

<a id="fn-schedule"></a>

### `og.commands.schedule`

Schedule a script on simulation time, from the current tick.

```ts
og.commands.schedule(world: World, script: string): Result<CommandsReport>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `script` | `string` | no | `t:action[:args];...` with t in simulation seconds. |

**Returns** `Result<CommandsReport>`: The report, or the first malformed step.

Binding symbol `og_commands_schedule` · publish builds: dropped

```ogt
fn schedule {
  doc "Schedule a script on simulation time, from the current tick."
  param world World "The world."
  param script String "`t:action[:args];...` with t in simulation seconds."
  returns Result.CommandsReport "The report, or the first malformed step."
}
```

<sub>Module `commands` line 106</sub>
