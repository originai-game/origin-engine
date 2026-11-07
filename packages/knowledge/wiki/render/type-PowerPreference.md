<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-PowerPreference"></a>

### `PowerPreference` (enum)

Which GPU the browser's adapter is asked for (requestAdapter's powerPreference), on a laptop with two. The adapter is made once, at the boot: a change applies on the next load (DisplayInfo.restartNeeded), which the web shell keeps for the page.

A string: one of the variants below. The zero value is `"highPerformance"`.

| Variant | Doc |
|---|---|
| `"highPerformance"` | The discrete GPU where there is one: the frame rate's choice. |
| `"lowPower"` | The integrated GPU: the battery's choice, cooler and quieter. |

```ogt
enum PowerPreference {
  doc "Which GPU the browser's adapter is asked for (requestAdapter's powerPreference), on a laptop with two. The adapter is made once, at the boot: a change applies on the next load (DisplayInfo.restartNeeded), which the web shell keeps for the page."
  variant highPerformance "The discrete GPU where there is one: the frame rate's choice."
  variant lowPower "The integrated GPU: the battery's choice, cooler and quieter."
}
```

<sub>Module `render` line 2757</sub>
