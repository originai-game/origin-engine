<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`commands`](../commands.md) · [index](../index.md)

<a id="fn-parse"></a>

### `og.commands.parse`

Parse a script without scheduling it.

```ts
og.commands.parse(script: string): Result<readonly CommandStep[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `script` | `string` | no | `t:action[:args];...` with t in simulation seconds. |

**Returns** `Result<readonly CommandStep[]>`: The steps, or the first malformed step.

Binding symbol `og_commands_parse` · publish builds: dropped

```ogt
fn parse {
  doc "Parse a script without scheduling it."
  param script String "`t:action[:args];...` with t in simulation seconds."
  returns Result.List.CommandStep "The steps, or the first malformed step."
}
```

<sub>`api.ogt` line 24003</sub>
