<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-seconds"></a>

### `og.ui.seconds`

The UI's clock in seconds: frames built over the tick rate (simulation time, never the wall clock).

```ts
og.ui.seconds(): number
```

**Returns** `number`.

Binding symbol `og_ui_seconds` · publish builds: kept

```ogt
fn seconds {
  doc "The UI's clock in seconds: frames built over the tick rate (simulation time, never the wall clock)."
  returns f64
}
```

<sub>Module `ui` line 948</sub>
