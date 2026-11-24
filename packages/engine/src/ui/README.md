# The built-in UI library

TSX components styled with Tailwind utility classes and theme pack slots
(`crates/og-ui-next/CONVENTIONS.md`): `controls/` (reusable interactive
primitives), `components/` (HUD, feedback and pattern pieces, and the genre
groups) and `screens/`. `og-ui-next` provides layout, text, input, animation
and accessibility; none of a component's look is hardcoded in Rust.
`docs/ui.md` §1–§4 holds the contract, `components/README.md` and
`screens/README.md` the catalogues.

## Controls

Import reusable controls from `@origingame/engine/ui/components` (or from
`@origingame/engine/ui/controls/<name>`). `og ui eject controls/<name>` copies
any control into `assets/ui/controls/<name>.tsx` to customize in a game.
It copies locale tables and rewrites matching imports in game code, not
engine-internal imports; new uses import the copy explicitly. Screens use
the `screens/<name>` eject path and components their functional path
(`panels/backpack`). Named stores follow `svelte/store` semantics, bridged
to reactive accessors by `useStore`; this does not imply Svelte markup,
CSS documents or automatic component-name resolution.

| Control | File | Pack slots | Role / purpose |
|---|---|---|---|
| `Art` | `controls/art.tsx` | pack nine-slice class | Stretched or nine-sliced pack artwork |
| `Button` | `controls/button.tsx` | `Button.plate[.primary\|danger]` | Primary, plain, danger, ghost, quiet actions; hold-to-confirm |
| `Checkbox` | `controls/checkbox.tsx` | `Checkbox.indicator`, `Checkbox.mark`, `Checkbox.dash` | Binary or `"mixed"` checkbox (`role="checkbox"`) |
| `Counter` | `controls/counter.tsx` | `Count.roll`, `Counter.value` | Animated numeric readout |
| `Dialog` | `controls/dialog.tsx` | `Modal.scrim`, `Dialog.frame`, `Modal.frame`, `Dialog.title`, `Dialog.body`, `Dialog.actions` | Modal dialog (`role="dialog"` / `"alertdialog"`) with focus trap |
| `Meter` | `controls/meter.tsx` | `Bar.fill` | Scalar gauge (`role="meter"`) with low/high/optimum regions |
| `Progress` | `controls/progress.tsx` | `Bar.fill` | Determinate progress bar (`role="progressbar"`) |
| `Select` | `controls/select.tsx` | `Select.trigger`, `Select.list`, `Select.option` | Dropdown combobox (`role="combobox"` + `role="listbox"`) |
| `Slider` | `controls/slider.tsx` | `Slider.track`, `Slider.range`, `Slider.thumb` | Range slider (`role="slider"`) with drag and arrow keys |
| `Switch` | `controls/switch.tsx` | `Switch.track`, `Switch.thumb` | Toggle switch (`role="switch"`) |
| `Tabs`, `Tab` | `controls/tabs.tsx` | `Tab.list`, `Tab.item`, `Tab.mark`, `Tab.indicator` | Horizontal tab bar (`role="tablist"` + `role="tab"`) |
| `Tooltip` | `controls/tooltip.tsx` | `Tooltip.frame` | Anchored tooltip (`role="tooltip"`) |

This table is a starting set, not the complete export list. The component
barrel and generated `components` guide also include fields, numeric
input, radio/toggle groups, sheets, popovers, menus, toolbars, toasts,
rich text and other reusable pieces. Their `.tsx` props and fixtures are
the authoritative inputs and states; pack slots replace old CSS parts.
