<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TimerMode"></a>

### `TimerMode` (enum)

Which way a timer runs.

A string: one of the variants below. The zero value is `"countdown"`.

| Variant | Doc |
|---|---|
| `"countdown"` | Counts down (the default). |
| `"stopwatch"` | Counts up. |

```ogt
enum TimerMode {
  doc "Which way a timer runs."
  variant countdown "Counts down (the default)."
  variant stopwatch "Counts up."
}
```

<sub>Module `ui` line 6233</sub>
