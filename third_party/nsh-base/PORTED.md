# nsh-base

From NSH (https://github.com/longbridge/nsh), `crates/nsh-base` at commit
`589a60f7569c14023c262c126f50279f27ddf470`, Apache-2.0 (`LICENSE-APACHE`).
nsh/base is the unstyled base layer: each component owns its behaviour,
state, focus and accessibility and leaves its look to the app. No source is
vendored here: nsh-base is Rust on NSH's native views, and our components
are TypeScript on og-ui-next's host elements. What is ported is each
component's props, events, states, keyboard model and screen-reader role,
in `packages/engine/src/ui/controls/`, and the core behaviour they rely on,
in `crates/og-ui-next`.

| nsh-base | Ours | Kept | Changed |
|---|---|---|---|
| `button.rs` `Button` | `controls/button.tsx` | `disabled`, `selected`, `busy`, `role`, label and hint (`label`, `description`), press | Its look is the pack's `Button.plate.<variant>` with rune-kit's variants and sizes; `busy` shows the pack's `Button.spinner`; `hitSlop` is not ported (every size is 44 px tall) |
| `checkbox.rs` `Checkbox` | `controls/checkbox.tsx` | `checked`, `indeterminate` over it, `disabled`, `invalid`, the indicator part, the next value a press asks for | The indicator and marks are pack slots, not `indicatorClass`; no terminal form |
| `radio.rs` `Radio`, `RadioGroup` | `controls/radio.tsx` | checked and selected both for screen readers, a checked radio reports nothing, the group's arrows check what they reach | Roving is the tree's `focusGroup` with `focusRoving`; the arrows' check is the radio's `onFocus` |
| `switch.rs` `Switch` | `controls/switch.tsx` | `checked`, `disabled`, the track and thumb parts following its state | Parts are pack slots (`Switch.track`, `Switch.thumb`); the label comes before the track |
| `tabs.rs` `Tab`, `Tabs` | `controls/tabs.tsx` | the tab list's one stop, automatic activation, the indicator on the selected tab | The indicator is a mark inside the selected tab that slides by the pack's `Tab.indicator` glide, not `View::follow` |
| `toggle.rs` `Toggle`, `ToggleGroup` | `controls/toggle.tsx` | `pressed` (`aria-pressed:`), the group's one stop at its first pressed toggle, arrows that press nothing | A ToggleGroup inside a Toolbar is not merged into the toolbar's items |
| `toolbar.rs` `Toolbar`, `ToolbarSeparator` | `controls/toolbar.tsx` | one stop at the item last left, the separator never a stop | No `axis` prop or `ToolbarGroup` yet: the arrows follow the layout |
| `slider.rs` `Slider` | `controls/slider.tsx` | `min`/`max`/`step` (0 to 100 by 1), press, drag, arrows, Page Up/Down by a tenth, Home/End, `onChange`, `onChangeEnd` | One value (no range), linear scale, horizontal only |
| `number_input.rs` `NumberInput` | `controls/number-input.tsx` | `min`, `max`, `step`, the spin button role, decrement and increment buttons, keys | No text entry yet (the steppers and keys only) |
| `progress.rs` `Progress` | `controls/progress.tsx` | `value` 0 to 100, `indeterminate`, the progress bar role | Track and indicator are pack slots (`Progress.track`, `Bar.fill`, `Progress.indeterminate`) |
| `roving.rs` | `crates/og-ui-next/src/tree/input.rs` `rove`, `focusRoving` | a group is one Tab stop: the focused member, else the current one, else the one last left, else the first | It works on the tree's focus groups, so a game's own `focusGroup` may rove too |
| nsh-core's `accessibilityState` class states | `crates/og-ui-next/src/tree/ops.rs` | `checked:`, `indeterminate:`, `selected:`, `aria-pressed:`, `aria-expanded:`, `aria-busy:`, `aria-invalid:` and their `group-*:` forms | Set from the props of the same names |
