# data-tables

Two TS tables that name each other's rows (`content/items.ts`,
`content/materials.ts`, with an `asset` icon and `text` names), read as the
game's `Item` and `Material` components, and an archer that holds a bow and
a quiver of arrows as TableRow fields (`src/main.ts`).

What it checks (`tests/tables.og.ts`, `og test tests/fixtures/data-tables`):

- every tick, the bow's `ammo` row is the quiver's and the wood it salvages
  into names the bow back, across both tables; the sword's numbers, icon,
  rarity and i18n name read back as written;
- `reload` replaces base/items with structured reordered and renumbered rows:
  held handles read the new rows; `bad` gives a string in a number field:
  refused naming the field, the reloaded table kept; `drop` deletes the arrow
  row: og.table.missing names the one field still holding it;
- two runs end on one hash.

The pure component definitions in src/schema/items.ts are shared by cook and runtime.
The cook contract also covers invalid author input and cross-table row lookup.

Budget: under 10 s warm. No frames: nothing here is drawn.
