<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`commands`](../commands.md) · [index](../index.md)

<a id="fn-register"></a>

### `og.commands.register`

Register an action into the command channel. GameDesc.commands does this for a game's own actions.

```ts
og.commands.register(name: string, handler: CommandHandler, desc?: CommandDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The action name, snake_case, such as `spawn_wave`. |
| `handler` | `CommandHandler` | no | What it does. |
| `desc` | `CommandDesc` | yes | Its usage line and doc. |

**Returns** `Result<void>`: Refused for a name a core action or another registration already uses.

Binding symbol `og_commands_register` · publish builds: dropped

```ogt
fn register {
  doc "Register an action into the command channel. GameDesc.commands does this for a game's own actions."
  param name String "The action name, snake_case, such as `spawn_wave`."
  param handler CommandHandler "What it does."
  param desc CommandDesc "Its usage line and doc."
  returns Result.Void "Refused for a name a core action or another registration already uses."
}
```

<sub>`api.ogt` line 23916</sub>
