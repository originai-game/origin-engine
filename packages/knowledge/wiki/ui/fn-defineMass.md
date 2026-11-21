<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-defineMass"></a>

### `og.ui.defineMass`

Declare a mass element template by name and answer its handle; declaring the name again replaces it and keeps the handle. Invalid timing text returns an error naming the text and required syntax; it never falls back to linear.

```ts
og.ui.defineMass(name: string, desc?: MassTemplateDesc): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | Its name. |
| `desc` | `MassTemplateDesc` | yes | What it draws and how it moves. |

**Returns** `Result<number>`.

Binding symbol `og_ui_define_mass` · publish builds: kept

```ogt
fn defineMass {
  doc "Declare a mass element template by name and answer its handle; declaring the name again replaces it and keeps the handle. Invalid timing text returns an error naming the text and required syntax; it never falls back to linear."
  param name String "Its name."
  param desc MassTemplateDesc "What it draws and how it moves."
  returns Result.u32
}
```

<sub>Module `ui` line 1495</sub>
