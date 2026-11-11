<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`commands`](../commands.md) · [index](../index.md)

<a id="type-CommandDesc"></a>

### `CommandDesc` (desc struct)

How a registered action describes itself in `og run --help` and the wiki.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `usage` | `string` | `""` | — | — | — | The argument form, such as `<count>`. |
| `doc` | `string` | `""` | — | — | — | One sentence on what it does. |

```ogt
struct CommandDesc {
  doc "How a registered action describes itself in `og run --help` and the wiki."
  desc true
  field usage String "The argument form, such as `<count>`."
  field doc String "One sentence on what it does."
}
```

<sub>Module `commands` line 56</sub>
