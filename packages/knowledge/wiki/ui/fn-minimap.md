<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-minimap"></a>

### `og.ui.minimap`

A mini-map frame: the map in a lineStrong frame, markers as dots (the player primary and ringed), an N mark and a zone label.

```ts
og.ui.minimap(id: string, desc?: MinimapDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `MinimapDesc` | yes | Map, markers and label. |

**Returns** `void`.

Binding symbol `og_ui_minimap` · publish builds: kept

```ogt
fn minimap {
  doc "A mini-map frame: the map in a lineStrong frame, markers as dots (the player primary and ringed), an N mark and a zone label."
  param id String "Its id."
  param desc MinimapDesc "Map, markers and label."
  returns Void
}
```

<sub>Module `ui` line 1434</sub>
