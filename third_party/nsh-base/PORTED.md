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
| `alert_dialog.rs`, the core's modal overlay | `controls/dialog.tsx` `Dialog` | modal for focus and screen readers, `back` and a scrim press close, `alertdialog` when it holds only buttons | We draw it ourselves (no platform alert); it fills its parent on the `modal` layer |
| `sheet.rs` `Sheet` | `controls/sheet.tsx` | at the bottom edge over a scrim, the grabber, a swipe down closes, a footer | One detent; no drag to resize |
| `popover.rs` `Popover` | `controls/popover.tsx` | anchored to its trigger, the trigger's expanded state, `back` closes | Placement `bottom` or `top`; it follows the trigger by the tree's node `anchor` |
| `menu.rs` `Menu`, `ContextMenu`, `MenuItem`, `MenuSeparator` | `controls/menu.tsx` `Menu`, `ContextMenu`, `MenuItem`, `MenuSeparator`, `MenuLabel` | the trigger's expanded state, the popup modal for focus (the first item focused, the trigger focused again on close), one wrapping focus group, choosing an item, a press outside and `back` close it, `menuitemcheckbox`/`menuitemradio` with `checked`, `closeOnSelect` | A popup on every device (no sheet on phones), shifted inside the screen; no `Submenu` or type-ahead yet; `ContextMenu` opens under its trigger, not at the pointer |
| `tooltip.rs` `Tooltip` | `controls/tooltip.tsx` | shows on hover and keyboard focus, describes its trigger | No delay or grace period yet |
| Toast, `ToastStack` | `controls/toast.tsx` `ToastStack` | stacked at the top, announced as status or alert, swipe to dismiss | The game owns the list and each toast's time |
| `select.rs` `Select` | `controls/select.tsx` | combobox role, arrows step the value while closed, the list a focus group with the chosen option selected | The list is under the trigger on every device; no type-ahead yet |
| `collapsible.rs`, `accordion.rs` | `controls/collapsible.tsx` `Collapsible`, `Accordion` | the header's expanded state, one or `multiple` open | The panel enters by the pack's `Collapsible.panel`, not a springing clipped height |

| `scrollbar.rs` `Scrollbar` (built-in bars) | every `Scroll`'s own bar (`Scrollbar.track`, `Scrollbar.thumb`), the tree's `thumb` and `pinned` props | the thumb's length and place follow the content, viewport and offset; a drag on the thumb scrolls, a press on the track brings the thumb's middle there; `scrollbar={false}` hides it | Always shown while something runs over (no hover or scroll-only mode, no fade timing); no placed `Scrollbar` component yet: a game sets `thumb="<name>"` on its own node |

Not ported yet: `Combobox`.
