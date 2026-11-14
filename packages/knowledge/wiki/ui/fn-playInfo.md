<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-playInfo"></a>

### `og.ui.playInfo`

Where a play is; null once it is gone.

```ts
og.ui.playInfo(play: number): PlayInfo | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `play` | `number` | no | The play. |

**Returns** `PlayInfo | null`.

Binding symbol `og_ui_play_info` · publish builds: kept

```ogt
fn playInfo {
  doc "Where a play is; null once it is gone."
  param play u32 "The play."
  returns Option.PlayInfo
}
```

<sub>Module `ui` line 1211</sub>
