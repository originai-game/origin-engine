<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-radioGroup"></a>

### `og.ui.radioGroup`

A radio group: one of several choices, as rings with labels or as cards. A press, Enter, Space or A chooses; Left and Right (and a pad's) step the choice while one has the focus, skipping disabled ones, as a native radio group does. The ring's dot grows in with a pop (selection group). Plays select. Parts: `Radio.option[selected][hover|pressed|focused|disabled]`, `Radio.ring[selected]`, `Radio.dot`, `Radio.card[selected]`. Role radiogroup with radio children, checked on the chosen. Probe: `<id>`, `<id>/Options/Option#i`.

Also known as: radio, radio buttons, option group

```ts
og.ui.radioGroup(id: string, desc?: RadioGroupDesc): ChoiceIndexResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `RadioGroupDesc` | yes | Its choices and layout. |

**Returns** `ChoiceIndexResult`.

Binding symbol `og_ui_radio_group` · publish builds: kept

```ogt
fn radioGroup {
  doc "A radio group: one of several choices, as rings with labels or as cards. A press, Enter, Space or A chooses; Left and Right (and a pad's) step the choice while one has the focus, skipping disabled ones, as a native radio group does. The ring's dot grows in with a pop (selection group). Plays select. Parts: `Radio.option[selected][hover|pressed|focused|disabled]`, `Radio.ring[selected]`, `Radio.dot`, `Radio.card[selected]`. Role radiogroup with radio children, checked on the chosen. Probe: `<id>`, `<id>/Options/Option#i`."
  aliases "radio, radio buttons, option group"
  param id String "Its id."
  param desc RadioGroupDesc "Its choices and layout."
  returns ChoiceIndexResult
}
```

<sub>Module `ui` line 3720</sub>
