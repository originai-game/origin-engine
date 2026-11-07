<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-panel"></a>

### `og.ui.panel`

A card built by `build`: openPanel(id, desc), build(), close().

```ts
og.ui.panel(id: string, desc: PanelDesc, build: UiBuild): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `PanelDesc` | no | Its layout and look. |
| `build` | `UiBuild` | no | Makes its children. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn panel {
  doc "A card built by `build`: openPanel(id, desc), build(), close()."
  host false
  param id String "Its id."
  param desc PanelDesc "Its layout and look."
  param build UiBuild "Makes its children."
  returns Void
}
```

<sub>Module `ui` line 638</sub>
