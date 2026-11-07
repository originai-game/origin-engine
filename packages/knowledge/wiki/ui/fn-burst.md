<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-burst"></a>

### `og.ui.burst`

Throw rays and sparks from the element built just before (a star landing, a reward), once, from when it first appears. Off under reduced motion.

```ts
og.ui.burst(desc?: BurstDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `BurstDesc` | yes | Count, ink, reach and timing. |

**Returns** `void`.

Binding symbol `og_ui_burst` · publish builds: kept

```ogt
fn burst {
  doc "Throw rays and sparks from the element built just before (a star landing, a reward), once, from when it first appears. Off under reduced motion."
  param desc BurstDesc "Count, ink, reach and timing."
  returns Void
}
```

<sub>Module `ui` line 1847</sub>
