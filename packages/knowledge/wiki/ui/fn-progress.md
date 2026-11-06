<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-progress"></a>

### `og.ui.progress`

A track filled to a fraction.

```ts
og.ui.progress(id: string, value: number, desc?: ProgressDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `value` | `number` | no | How full it is, from 0 to 1. |
| `desc` | `ProgressDesc` | yes | Its width. |

**Returns** `void`.

Binding symbol `og_ui_progress` · publish builds: kept

```ogt
fn progress {
  doc "A track filled to a fraction."
  param id String "Its id."
  param value f32 "How full it is, from 0 to 1."
  param desc ProgressDesc "Its width."
  returns Void
}
```

<sub>`api.ogt` line 16291</sub>
