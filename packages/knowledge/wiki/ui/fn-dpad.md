<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-dpad"></a>

### `og.ui.dpad`

A d-pad of four arms (`Up`, `Down`, `Left`, `Right`), each at least 44 CSS px.

```ts
og.ui.dpad(id: string, desc?: DpadDesc): DpadResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `DpadDesc` | yes | Its size. |

**Returns** `DpadResult`.

Binding symbol `og_ui_dpad` · publish builds: kept

```ogt
fn dpad {
  doc "A d-pad of four arms (`Up`, `Down`, `Left`, `Right`), each at least 44 CSS px."
  param id String "Its id."
  param desc DpadDesc "Its size."
  returns DpadResult
}
```

<sub>Module `ui` line 1456</sub>
