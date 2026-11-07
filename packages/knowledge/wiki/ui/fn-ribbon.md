<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-ribbon"></a>

### `og.ui.ribbon`

A ribbon: a diagonal band wrapping a corner of the open container with its folded tail, a notched banner across its top or bottom, or a stamp slapped across its middle (a New best!: it lands from large and turned, squashes, settles and puffs dust, emphasis spring). The open container should clip its corners for a corner ribbon.

Parts: Ribbon.band[<corner>][<tone>], Ribbon.fold, Ribbon.text. States: group slap (waiting, landing, settled). Sounds: confirm when it lands. Accessibility: status named by the text. Probe ids: <id>, <id>/Text.

```ts
og.ui.ribbon(id: string, desc?: UiRibbonDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `UiRibbonDesc` | yes | Its words and place. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn ribbon {
  doc "A ribbon: a diagonal band wrapping a corner of the open container with its folded tail, a notched banner across its top or bottom, or a stamp slapped across its middle (a New best!: it lands from large and turned, squashes, settles and puffs dust, emphasis spring). The open container should clip its corners for a corner ribbon."
  doc "Parts: Ribbon.band[<corner>][<tone>], Ribbon.fold, Ribbon.text. States: group slap (waiting, landing, settled). Sounds: confirm when it lands. Accessibility: status named by the text. Probe ids: <id>, <id>/Text."
  host false
  param id String "Its id."
  param desc UiRibbonDesc "Its words and place."
  returns Void
}
```

<sub>Module `ui` line 5707</sub>
