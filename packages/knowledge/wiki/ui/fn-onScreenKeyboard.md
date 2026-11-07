<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-onScreenKeyboard"></a>

### `og.ui.onScreenKeyboard`

An on-screen keyboard for pads and touch (docs/ui.md §2: search and names with a pad): a panel at the screen's bottom with the text being typed and its caret, a candidate bar while composing, and keys a pad walks with focus wrapping by rows (Y erases, LB and RB move the caret, Back closes) and a finger presses (a key pops its letter above the finger on a press). Letters (shift for one, double tap for caps), symbols, a number pad and the Japanese kana table, with composition for an IME: typed kana (or letters with compose) stay underlined in KeyboardResult.composition until a candidate (the game's, plus the katakana) or Enter commits them. Plays press, select and back sounds. Parts: `Keyboard.panel`, `Keyboard.key[wide][action][active][primary][hover|pressed|focused|disabled]` (primary: the enter key; disabled: a gap in the kana table), `Keyboard.preview`, `Keyboard.candidates`, `Keyboard.candidate[hover|pressed|focused]`, `Keyboard.popup`. A dialog named by its title; keys are buttons named by what they type. Probe: `<id>/Panel/Content/Keys/Row#r/Key#k`, `<id>/Panel/Content/Preview`, `<id>/Panel/Content/Candidates/Candidate#i`.

```ts
og.ui.onScreenKeyboard(id: string, open: boolean, desc?: KeyboardDesc): KeyboardResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `open` | `boolean` | no | Whether it shows. |
| `desc` | `KeyboardDesc` | yes | What it edits and how it types. |

**Returns** `KeyboardResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn onScreenKeyboard {
  doc "An on-screen keyboard for pads and touch (docs/ui.md §2: search and names with a pad): a panel at the screen's bottom with the text being typed and its caret, a candidate bar while composing, and keys a pad walks with focus wrapping by rows (Y erases, LB and RB move the caret, Back closes) and a finger presses (a key pops its letter above the finger on a press). Letters (shift for one, double tap for caps), symbols, a number pad and the Japanese kana table, with composition for an IME: typed kana (or letters with compose) stay underlined in KeyboardResult.composition until a candidate (the game's, plus the katakana) or Enter commits them. Plays press, select and back sounds. Parts: `Keyboard.panel`, `Keyboard.key[wide][action][active][primary][hover|pressed|focused|disabled]` (primary: the enter key; disabled: a gap in the kana table), `Keyboard.preview`, `Keyboard.candidates`, `Keyboard.candidate[hover|pressed|focused]`, `Keyboard.popup`. A dialog named by its title; keys are buttons named by what they type. Probe: `<id>/Panel/Content/Keys/Row#r/Key#k`, `<id>/Panel/Content/Preview`, `<id>/Panel/Content/Candidates/Candidate#i`."
  host false
  param id String "Its id."
  param open bool "Whether it shows."
  param desc KeyboardDesc "What it edits and how it types."
  returns KeyboardResult
}
```

<sub>Module `ui` line 4141</sub>
