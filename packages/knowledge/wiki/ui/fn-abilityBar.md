<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-abilityBar"></a>

### `og.ui.abilityBar`

An ability bar: abilitySlot for each slot, in a row (or a column) straight on the scene (a style may dress AbilityBar.frame as a slim bar), evenly spaced; a row with more slots than the HUD band holds wraps into even rows whose columns line up (eight slots on a phone are two rows of four).

Parts: AbilityBar.frame and each slot's. Accessibility: a group named by its slots. Probe ids: <id>/<key>.

```ts
og.ui.abilityBar(id: string, desc?: AbilityBarDesc): AbilityBarResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `AbilityBarDesc` | yes | The slots. |

**Returns** `AbilityBarResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn abilityBar {
  doc "An ability bar: abilitySlot for each slot, in a row (or a column) straight on the scene (a style may dress AbilityBar.frame as a slim bar), evenly spaced; a row with more slots than the HUD band holds wraps into even rows whose columns line up (eight slots on a phone are two rows of four)."
  doc "Parts: AbilityBar.frame and each slot's. Accessibility: a group named by its slots. Probe ids: <id>/<key>."
  host false
  param id String "Its id."
  param desc AbilityBarDesc "The slots."
  returns AbilityBarResult
}
```

<sub>`api.ogt` line 21856</sub>
