<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-cooldown"></a>

### `og.ui.cooldown`

A cooldown: a disc whose primary ring fills clockwise from the top as it runs out, bright and full when ready, a label or icon in the middle. A press reports clicked (ready or not).

```ts
og.ui.cooldown(id: string, remaining: number, desc?: CooldownDesc): Response
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `remaining` | `number` | no | How much of the cooldown is left, 0 to 1: 1 just used, 0 ready. |
| `desc` | `CooldownDesc` | yes | Size, label and icon. |

**Returns** `Response`.

Binding symbol `og_ui_cooldown` · publish builds: kept

```ogt
fn cooldown {
  doc "A cooldown: a disc whose primary ring fills clockwise from the top as it runs out, bright and full when ready, a label or icon in the middle. A press reports clicked (ready or not)."
  param id String "Its id."
  param remaining f32 "How much of the cooldown is left, 0 to 1: 1 just used, 0 ready."
  param desc CooldownDesc "Size, label and icon."
  returns Response
}
```

<sub>Module `ui` line 1628</sub>
