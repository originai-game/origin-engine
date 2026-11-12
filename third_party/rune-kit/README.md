# rune-kit

The focus runtime and kit of rune (https://github.com/anomalyco/rune,
commit `a8ca769b944598e179f33cc6843629b90e6ca831`, MIT, licence in
`LICENSE`), the keyboard model ported to og-ui's navigation
(`crates/og-ui-core/src/nav.rs`, `apply_list_keys`; the dropdown trigger in
`crates/og-ui-core/src/widgets/controls/dropdown.rs`):

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
