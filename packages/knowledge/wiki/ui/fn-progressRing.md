<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-progressRing"></a>

### `og.ui.progressRing`

A progress ring: an arc filled clockwise from the top on the style's ui spring, or a spinning arc that grows and shrinks as it turns (Material's indeterminate ring, which reads as working at any size); a label in the middle. Under reduced motion the spinner pulses its opacity instead of turning.

Parts: ProgressRing.track, ProgressRing.fill[<tone>][spinning], ProgressRing.label. States: group mode (determinate, indeterminate). Accessibility: progressbar (busy when spinning). Probe ids: <id>, <id>/Fill, <id>/Label.

```ts
og.ui.progressRing(id: string, desc?: ProgressRingDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ProgressRingDesc` | yes | Its value and size. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn progressRing {
  doc "A progress ring: an arc filled clockwise from the top on the style's ui spring, or a spinning arc that grows and shrinks as it turns (Material's indeterminate ring, which reads as working at any size); a label in the middle. Under reduced motion the spinner pulses its opacity instead of turning."
  doc "Parts: ProgressRing.track, ProgressRing.fill[<tone>][spinning], ProgressRing.label. States: group mode (determinate, indeterminate). Accessibility: progressbar (busy when spinning). Probe ids: <id>, <id>/Fill, <id>/Label."
  host false
  param id String "Its id."
  param desc ProgressRingDesc "Its value and size."
  returns Void
}
```

<sub>Module `ui` line 5479</sub>
