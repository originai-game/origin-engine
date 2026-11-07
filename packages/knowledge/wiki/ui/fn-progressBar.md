<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-progressBar"></a>

### `og.ui.progressBar`

A progress bar: a rounded track, a fill that moves to its value on the style's ui spring (never jumping back on a retarget), a buffer ahead of it, a sweeping sheen while it works, segments for steps, a sweeping band when indeterminate, and a caption and value over it. Reaching 1 flashes the fill once and plays success.

Parts: ProgressBar.track[<style>], ProgressBar.buffer, ProgressBar.fill[<tone>][done], ProgressBar.step[filled], ProgressBar.label, ProgressBar.value. States: group progress (working, done) and group mode (determinate, indeterminate). Accessibility: progressbar named by the label, 0 to 100 (busy when indeterminate). Probe ids: <id>, <id>/Track, <id>/Fill, <id>/Label, <id>/Value.

Also known as: loading bar, progress, Unity Slider fill, Godot ProgressBar

```ts
og.ui.progressBar(id: string, desc?: ProgressBarDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ProgressBarDesc` | yes | Its value and look. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn progressBar {
  doc "A progress bar: a rounded track, a fill that moves to its value on the style's ui spring (never jumping back on a retarget), a buffer ahead of it, a sweeping sheen while it works, segments for steps, a sweeping band when indeterminate, and a caption and value over it. Reaching 1 flashes the fill once and plays success."
  doc "Parts: ProgressBar.track[<style>], ProgressBar.buffer, ProgressBar.fill[<tone>][done], ProgressBar.step[filled], ProgressBar.label, ProgressBar.value. States: group progress (working, done) and group mode (determinate, indeterminate). Accessibility: progressbar named by the label, 0 to 100 (busy when indeterminate). Probe ids: <id>, <id>/Track, <id>/Fill, <id>/Label, <id>/Value."
  aliases "loading bar, progress, Unity Slider fill, Godot ProgressBar"
  host false
  param id String "Its id."
  param desc ProgressBarDesc "Its value and look."
  returns Void
}
```

<sub>Module `ui` line 5534</sub>
