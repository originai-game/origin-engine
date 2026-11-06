<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`commands`](../commands.md) · [index](../index.md)

<a id="type-CommandHandler"></a>

### `CommandHandler` (callback)

A registered action. It runs at the start of the tick it was scheduled for.

Game code supplies this function.

```ts
type CommandHandler = (world: World, args: readonly string[]) => void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `args` | `readonly string[]` | no | The `:`-separated arguments after the action name. |

```ogt
callback CommandHandler {
  doc "A registered action. It runs at the start of the tick it was scheduled for."
  param world World "The world."
  param args List.String "The `:`-separated arguments after the action name."
}
```

<sub>`api.ogt` line 23972</sub>
