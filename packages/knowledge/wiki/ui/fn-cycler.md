<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-cycler"></a>

### `og.ui.cycler`

An option cycler, the console settings row: `‹ High ›` with pips. Left and Right (a pad's d-pad or stick) change it while it has the focus, the arrows take a press, and a press on the value steps forward; the value slides in from the side it came (selection on the style's motion). Arrows follow the writing direction. Plays select. Parts: `Cycler.frame[hover|focused|disabled]`, `Cycler.arrow[prev|next][disabled]`, `Cycler.pip[current]`. Role spinbutton reading the option; its arrows are buttons under a finger and named images on a fine pointer (Left and Right on the focused cycler are theirs). Probe: `<id>`, `<id>/Prev`, `<id>/Middle/Clip/Value#i` (the option shown), `<id>/Next`, `<id>/Middle/Pips/Pip#i`.

```ts
og.ui.cycler(id: string, options: readonly string[], index: number, desc?: CyclerDesc): ChoiceIndexResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `options` | `readonly string[]` | no | The options. |
| `index` | `number` | no | The current one from game state. |
| `desc` | `CyclerDesc` | yes | Its label and look. |

**Returns** `ChoiceIndexResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn cycler {
  doc "An option cycler, the console settings row: `‹ High ›` with pips. Left and Right (a pad's d-pad or stick) change it while it has the focus, the arrows take a press, and a press on the value steps forward; the value slides in from the side it came (selection on the style's motion). Arrows follow the writing direction. Plays select. Parts: `Cycler.frame[hover|focused|disabled]`, `Cycler.arrow[prev|next][disabled]`, `Cycler.pip[current]`. Role spinbutton reading the option; its arrows are buttons under a finger and named images on a fine pointer (Left and Right on the focused cycler are theirs). Probe: `<id>`, `<id>/Prev`, `<id>/Middle/Clip/Value#i` (the option shown), `<id>/Next`, `<id>/Middle/Pips/Pip#i`."
  host false
  param id String "Its id."
  param options List.String "The options."
  param index i32 "The current one from game state."
  param desc CyclerDesc "Its label and look."
  returns ChoiceIndexResult
}
```

<sub>Module `ui` line 3985</sub>
