<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-resume"></a>

### `og.ui.resume`

Let a held play go on.

```ts
og.ui.resume(play: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `play` | `number` | no | The play. |

**Returns** `boolean`.

Binding symbol `og_ui_resume` · publish builds: kept

```ogt
fn resume {
  doc "Let a held play go on."
  param play u32 "The play."
  returns bool
}
```

<sub>Module `ui` line 2258</sub>
