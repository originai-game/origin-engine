<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-pushMass"></a>

### `og.ui.pushMass`

Show instances of a template for this tick only (overhead bars, nameplates, status icons rebuilt every tick with their values).

```ts
og.ui.pushMass(template: number, instances: readonly MassInstance[]): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `template` | `number` | no | From defineMass. |
| `instances` | `readonly MassInstance[]` | no | The instances. |

**Returns** `void`.

Binding symbol `og_ui_push_mass` · publish builds: kept

```ogt
fn pushMass {
  doc "Show instances of a template for this tick only (overhead bars, nameplates, status icons rebuilt every tick with their values)."
  param template u32 "From defineMass."
  param instances List.MassInstance "The instances."
  returns Void
}
```

<sub>Module `ui` line 7717</sub>
