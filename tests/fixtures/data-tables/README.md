# data-tables

Two tables that name each other's rows (`assets/items.ogt`,
`assets/materials.ogt`, with an `asset` icon and `text` names), read as the
game's `Item` and `Material` components, and an archer that holds a bow and
a quiver of arrows as TableRow fields (`src/main.ts`).

What it checks (`tests/tables.og.ts`, `og test tests/fixtures/data-tables`):

- every tick, the bow's `ammo` row is the quiver's and the wood it salvages
  into names the bow back, across both tables; the sword's numbers, icon,
  rarity and i18n name read back as written;
- `reload` re-parses items.ogt reordered and renumbered: the held handles
  read the new rows; `bad` parses a row with a string in a number field:
  refused naming line 6, the reloaded table kept; `drop` deletes the arrow
  row: og.table.missing names the one field still holding it;
- two runs end on one hash.

A save restored after a row was deleted, the refusals' exact text and a
real 52-bit handle collision across two tables are
`packages/engine/tests/table.test.ts` (a game cannot keep a save's text in
world state). The cook's refusals are og-cook's `cook::systems::table`
tests.

Budget: under 10 s warm. No frames: nothing here is drawn.
