<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`commands`](../commands.md) · [index](../index.md)

<a id="fn-help"></a>

### `og.commands.help`

Every action the channel knows: core actions in CoreAction order, then registered ones by name.

```ts
og.commands.help(world: World): readonly CommandInfo[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose channel is asked. |

**Returns** `readonly CommandInfo[]`.

Binding symbol `og_commands_help` · publish builds: dropped

```ogt
fn help {
  doc "Every action the channel knows: core actions in CoreAction order, then registered ones by name."
  param world World "The world whose channel is asked."
  returns List.CommandInfo
}
```

<sub>`api.ogt` line 23930</sub>
