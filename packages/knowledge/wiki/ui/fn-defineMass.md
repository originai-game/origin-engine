<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-defineMass"></a>

### `og.ui.defineMass`

Declare a mass element template by name and answer its handle; declaring the name again replaces it and keeps the handle.

```ts
og.ui.defineMass(name: string, desc?: MassTemplateDesc): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | Its name. |
| `desc` | `MassTemplateDesc` | yes | What it draws and how it moves. |

**Returns** `number`.

Binding symbol `og_ui_define_mass` · publish builds: kept

```ogt
fn defineMass {
  doc "Declare a mass element template by name and answer its handle; declaring the name again replaces it and keeps the handle."
  param name String "Its name."
  param desc MassTemplateDesc "What it draws and how it moves."
  returns u32
}
```

<sub>Module `ui` line 7712</sub>
