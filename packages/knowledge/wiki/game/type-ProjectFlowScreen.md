<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`game`](../game.md) · [index](../index.md)

<a id="type-ProjectFlowScreen"></a>

### `ProjectFlowScreen` (enum)

Who draws the project's startup dialogue.

A string: one of the variants below. The zero value is `"engine"`.

| Variant | Doc |
|---|---|
| `"engine"` | The built-in FlowScreen. |
| `"game"` | The game's own UI. |

```ogt
enum ProjectFlowScreen {
  doc "Who draws the project's startup dialogue."
  variant engine "The built-in FlowScreen."
  variant game "The game's own UI."
}
```

<sub>Module `game` line 12</sub>
