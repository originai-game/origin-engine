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

| Control | File | Pack slots | Role / purpose |
|---|---|---|---|
| `Art` | `controls/art.tsx` | pack nine-slice class | Stretched or nine-sliced pack artwork |
| `Button` | `controls/button.tsx` | `Button.plate[.primary\|danger]` | Primary, plain, danger, ghost, quiet actions; hold-to-confirm |
| `Checkbox` | `controls/checkbox.tsx` | `Checkbox.box` | Binary or `"mixed"` checkbox (`role="checkbox"`) |
| `Counter` | `controls/counter.tsx` | — | Animated numeric readout (`tabular-nums`) |
| `Dialog` | `controls/dialog.tsx` | `Modal.scrim`, `Panel.frame`, `Modal.frame` | Modal dialog (`role="dialog"` / `"alertdialog"`) with focus trap |
| `Meter` | `controls/meter.tsx` | `Bar.fill` | Scalar gauge (`role="meter"`) with low/high/optimum regions |
| `Progress` | `controls/progress.tsx` | `Bar.fill` | Determinate progress bar (`role="progressbar"`) |
| `Select` | `controls/select.tsx` | `Select.trigger`, `Select.menu` | Dropdown combobox (`role="combobox"` + `role="listbox"`) |
| `Slider` | `controls/slider.tsx` | `Slider.track`, `Slider.fill`, `Slider.thumb` | Range slider (`role="slider"`) with drag and arrow keys |
| `Switch` | `controls/switch.tsx` | `Switch.track`, `Switch.thumb` | Toggle switch (`role="switch"`) |
| `Tabs` | `controls/tabs.tsx` | `Tab.bar`, `Tab.indicator` | Horizontal tab bar (`role="tablist"` + `role="tab"`) |
| `Tooltip` | `controls/tooltip.tsx` | `Tooltip.frame` | Anchored tooltip (`role="tooltip"`) |
