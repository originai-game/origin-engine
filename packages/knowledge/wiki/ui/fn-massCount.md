<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-massCount"></a>

### `og.ui.massCount`

The live spawned instances of a template.

```ts
og.ui.massCount(template: number): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `template` | `number` | no | From defineMass. |

**Returns** `number`.

Binding symbol `og_ui_mass_count` · publish builds: kept

```ogt
fn massCount {
  doc "The live spawned instances of a template."
  param template u32 "From defineMass."
  returns u32
}
```

<sub>Module `ui` line 3766</sub>
