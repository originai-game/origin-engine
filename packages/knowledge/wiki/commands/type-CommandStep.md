<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`commands`](../commands.md) · [index](../index.md)

<a id="type-CommandStep"></a>

### `CommandStep` (struct)

One parsed step of a command script.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `at` | `number` | `0` | — | `s` | — | When it runs, in simulation seconds from the start of the script. |
| `action` | `string` | `""` | — | — | — | The action name. |
| `args` | `readonly string[]` | `[]` | — | — | — | The arguments. |

```ogt
struct CommandStep {
  doc "One parsed step of a command script."
  field at f32 "When it runs, in simulation seconds from the start of the script." {
    unit s
  }
  field action String "The action name."
  field args List.String "The arguments."
}
```

<sub>`api.ogt` line 23892</sub>
