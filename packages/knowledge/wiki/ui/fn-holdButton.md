<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-holdButton"></a>

### `og.ui.holdButton`

A hold-to-confirm button: held (a press on it, a touch), it fills from its start edge over holdMs, ticking up in pitch, and confirms with the style's confirm sound and a pop; let go early, it drains back. Keys and pads take its toggle alternative (XAG 107: activate starts it, it runs by itself, activate again or moving the focus stops it), as the pointer does with toggle. Its progress is UI state (in snapshots). Parts: `HoldButton.base[<variant>][holding][hover|pressed|focused|disabled]`, `HoldButton.fill`. Role button, its fill as its value (percent). Probe: `<id>`, `<id>/Button`, `<id>/Button/Label`, `<id>/Button/Fill`.

```ts
og.ui.holdButton(id: string, label: string, desc?: HoldButtonDesc): HoldResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `label` | `string` | no | Its label. |
| `desc` | `HoldButtonDesc` | yes | Its role, time and alternative. |

**Returns** `HoldResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn holdButton {
  doc "A hold-to-confirm button: held (a press on it, a touch), it fills from its start edge over holdMs, ticking up in pitch, and confirms with the style's confirm sound and a pop; let go early, it drains back. Keys and pads take its toggle alternative (XAG 107: activate starts it, it runs by itself, activate again or moving the focus stops it), as the pointer does with toggle. Its progress is UI state (in snapshots). Parts: `HoldButton.base[<variant>][holding][hover|pressed|focused|disabled]`, `HoldButton.fill`. Role button, its fill as its value (percent). Probe: `<id>`, `<id>/Button`, `<id>/Button/Label`, `<id>/Button/Fill`."
  host false
  param id String "Its id."
  param label String "Its label."
  param desc HoldButtonDesc "Its role, time and alternative."
  returns HoldResult
}
```

<sub>Module `ui` line 3616</sub>
