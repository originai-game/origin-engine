# rune-kit

The focus runtime and kit of rune (https://github.com/anomalyco/rune,
commit `a8ca769b944598e179f33cc6843629b90e6ca831`, MIT, licence in
`LICENSE`), the keyboard model ported to og-ui's navigation
(`crates/og-ui/src/interaction/nav.rs`, `apply_list_keys`; the dropdown trigger in
`crates/og-ui/src/widgets/controls/dropdown.rs`):

- Home and End to a focus group's first and last member
  (`crates/rune/src/runtime/input/focus.rs`, `focus_key`);
- PageUp and PageDown to the farthest member still within one view of the
  focused one, one member at least, stopping at the ends (`page_item`);
- type-select: letters typed less than a second apart go to the first
  member whose name starts with them; the same letter again cycles from the
  focused member; an arrow, Home or End clears the letters; only in groups
  of the roles menu, listbox, list, tree and tablist (`type_select`,
  `TYPE_SELECT_ROLES`, `TYPE_SELECT_PAUSE`);
- Up and Down on a closed popup button (role combobox) open it
  (`focus_key`, NSPopUpButton).

Changed: the view for the page keys is the nearest scroll's, else the
screen's; the letters' pause is counted in ticks, so it is deterministic;
names are the members' accessible names from the frame before.

## Kit (token roles and the button matrix)

From rune at commit `337a06f38` (MIT, `LICENSE`): the `Kit` schema's roles
(Palette's `layer_01…04`, `border_muted/base/strong`, `hover`, `pressed`,
`knob`/`knob_on`; Elevation; Motion) are adopted as design tokens in
`packages/design/tokens` (`layer-1…4`, `border-muted`, `hover`, `pressed`,
`knob-on`), and its Button's variants and sizes as the pack slots
`Button.plate.<variant>` (neutral, primary for submit, danger, warning,
outline, contrast, ghost, ghost-muted, ghost-faint) and the sizes sm, md and
lg of `packages/engine/src/ui/controls/button.tsx`. rune-kit's Loading
variant is the button's `busy` state. No source is vendored.
