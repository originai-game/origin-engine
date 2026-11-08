<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-openScroll"></a>

### `og.ui.openScroll`

Open a vertical scroll container, closed by close().

```ts
og.ui.openScroll(id: string, desc?: ScrollDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ScrollDesc` | yes | Its size and spacing. |

**Returns** `void`.

Binding symbol `og_ui_open_scroll` · publish builds: kept

```ogt
fn openScroll {
  doc "Open a vertical scroll container, closed by close()."
  param id String "Its id."
  param desc ScrollDesc "Its size and spacing."
  returns Void
}
```

<sub>Module `ui` line 602</sub>
