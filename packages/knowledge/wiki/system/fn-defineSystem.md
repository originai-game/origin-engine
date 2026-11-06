<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`system`](../system.md) · [index](../index.md)

<a id="fn-defineSystem"></a>

### `defineSystem`

Declare a system.

```ts
defineSystem(name: string, desc: SystemDesc, run: SystemRun): System
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The system's unique name, PascalCase. |
| `desc` | `SystemDesc` | no | Reads, writes and order. |
| `run` | `SystemRun` | no | The body. |

**Returns** `System`.

Binding symbol `og_system_define_system` · publish builds: kept

```ogt
fn defineSystem {
  doc "Declare a system."
  param name String "The system's unique name, PascalCase."
  param desc SystemDesc "Reads, writes and order."
  param run SystemRun "The body."
  returns System
}
```

<sub>`api.ogt` line 1401</sub>
