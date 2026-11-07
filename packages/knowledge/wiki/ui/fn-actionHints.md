<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-actionHints"></a>

### `og.ui.actionHints`

The action bar: each action the top layer accepts as its glyph and hint; on touch, buttons.

```ts
og.ui.actionHints(id: string, desc?: ActionHintsDesc): ActionHintsResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ActionHintsDesc` | yes | The hints and the edge. |

**Returns** `ActionHintsResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn actionHints {
  doc "The action bar: each action the top layer accepts as its glyph and hint; on touch, buttons."
  host false
  param id String "Its id."
  param desc ActionHintsDesc "The hints and the edge."
  returns ActionHintsResult
}
```

<sub>Module `ui` line 3042</sub>
