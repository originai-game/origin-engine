# rune-a11y

The AccessKit tree of rune (https://github.com/anomalyco/rune, commit
`6e200e637f4feed611741a77c8f894b9b7d61b4c`, MIT, licence in `LICENSE`),
ported as logic, not vendored, to og-ui-next's arena
(`crates/og-ui-next/src/a11y.rs`):

- an update that read nothing changed costs one stamp comparison
  (`crates/rune/src/a11y/tree.rs`, `Stamp`);
- otherwise one walk finds the tree's nodes and a fingerprint of everything
  each node shows (its own state, its box, its children, the text it is
  named by); only the nodes whose fingerprint moved are built and sent, and
  nodes the walk no longer finds leave with their parent's new children
  (`a11y_walk`, `a11y_build`, the `sent` map and its passes);
- behind an open modal the tree holds only the modal and what contains it;
- roles from what an element is when it names none: a pressable box is a
  button, a box that scrolls a scroll view, a plain layout box is left out
  and its children join its parent (`roles.rs`, `map`, `group`);
- the content of a button, checkbox, radio, switch, tab, option, link or
  menu item is one leaf named by its text (`leaf_control`,
  `a11y_repeats_control`).
