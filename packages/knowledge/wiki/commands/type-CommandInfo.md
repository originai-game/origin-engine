<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`commands`](../commands.md) · [index](../index.md)

<a id="type-CommandInfo"></a>

### `CommandInfo` (struct)

One action as help lists it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | The action name. |
| `usage` | `string` | `""` | — | — | — | Its argument form, such as `screenshot[:path][:full\|:N]`. |
| `doc` | `string` | `""` | — | — | — | One sentence on what it does. |
| `core` | `boolean` | `false` | — | — | — | Shipped with the engine (true) or registered by a module or a game. |

```ogt
struct CommandInfo {
  doc "One action as help lists it."
  field name String "The action name."
  field usage String "Its argument form, such as `screenshot[:path][:full|:N]`."
  field doc String "One sentence on what it does."
  field core bool "Shipped with the engine (true) or registered by a module or a game."
}
```

<sub>Module `commands` line 69</sub>
