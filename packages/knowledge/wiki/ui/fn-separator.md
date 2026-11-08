<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-separator"></a>

### `og.ui.separator`

A separator between groups: the style's line (its divider ornament where the style has one and no label is given), across or upright, with a label in its middle between two lines. Role separator. Parts: `Separator.line[strong]`. Probe: `<id>`, `<id>/Label`.

```ts
og.ui.separator(id: string, desc?: SeparatorDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `SeparatorDesc` | yes | Its direction, label and weight. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn separator {
  doc "A separator between groups: the style's line (its divider ornament where the style has one and no label is given), across or upright, with a label in its middle between two lines. Role separator. Parts: `Separator.line[strong]`. Probe: `<id>`, `<id>/Label`."
  host false
  param id String "Its id."
  param desc SeparatorDesc "Its direction, label and weight."
  returns Void
}
```

<sub>Module `ui` line 3268</sub>
