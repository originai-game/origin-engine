<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-radioGroup"></a>

### `og.ui.radioGroup`

A radio group: one of several choices, as rings with labels or as cards. A press, Enter, Space or A chooses; Left and Right (and a pad's) step the choice while one has the focus, skipping disabled ones, as a native radio group does. The ring's dot grows in with a pop (selection group). Plays select. Parts: `Radio.option[selected][hover|pressed|focused|disabled]`, `Radio.ring[selected]`, `Radio.dot`, `Radio.card[selected]`. Role radiogroup with radio children, checked on the chosen. Probe: `<id>`, `<id>/Options/Option#i`.

```ts
og.ui.radioGroup(id: string, desc?: RadioGroupDesc): ChoiceIndexResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `RadioGroupDesc` | yes | Its choices and layout. |

**Returns** `ChoiceIndexResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn radioGroup {
  doc "A radio group: one of several choices, as rings with labels or as cards. A press, Enter, Space or A chooses; Left and Right (and a pad's) step the choice while one has the focus, skipping disabled ones, as a native radio group does. The ring's dot grows in with a pop (selection group). Plays select. Parts: `Radio.option[selected][hover|pressed|focused|disabled]`, `Radio.ring[selected]`, `Radio.dot`, `Radio.card[selected]`. Role radiogroup with radio children, checked on the chosen. Probe: `<id>`, `<id>/Options/Option#i`."
  host false
  param id String "Its id."
  param desc RadioGroupDesc "Its choices and layout."
  returns ChoiceIndexResult
}
```

<sub>`api.ogt` line 19474</sub>
