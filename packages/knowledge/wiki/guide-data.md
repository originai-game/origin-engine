<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `data` guide: Data: tables of items, abilities, recipes and waves

How a game keeps its data as tables (`type table`): one row per item, ability, recipe, loot entry, wave or level, read as one of the game's own components, and named in the world by a handle that survives reloads and saves. Read before writing a game's content as more than a few constants.

The API: `og api show table`.

<a id="guide-data-1"></a>
### `data §1` Writing a table

A table is an `.ogt` under `assets/`:

```text
type table
version 1
---
locale "en"
row "sword" {
  damage 12
  weight 3.5
  size 0.2 1.1 0.05
  rarity rare
  icon asset "icons/sword.png"
  name text "items.sword" "Sword"
  ammo row "arrow"
  salvage row "materials.ogt" "iron" row "materials.ogt" "wood"
}
row "arrow" {
  damage 3
}
```

- A row is `row "<id>" { ... }`; an id appears once in its table. The
  rows' order is the file's.
- A field line is the field's name and its value as a scene writes it:
  numbers (a vector or colour is its numbers), a string, `true` or
  `false`, an enum's variant by name, or a list's numbers. A field left
  out takes the component's default.
- Three forms name something else, so the cook can check it:
  - `asset "path"` on a `str` field: a file under `assets/`; the field
    holds the path, which the game loads when it wants it.
  - `text "key" "Base text"` on a `str` field: an og.i18n key and its
    text in the table's `locale` (`en` when it names none). The field
    holds the key; `og i18n extract` and the load add the base text, and
    a string table translates it like any other key.
  - `row "id"` (this table) or `row "table" "id"` on a TableRow field,
    once per row on a list of them: the field holds that row's handle.
    Two tables may name each other's rows.
- og cook refuses a table with an id twice, an `asset` path that is not
  there, or a `row` its table lacks, naming the line.

<a id="guide-data-2"></a>
### `data §2` Loading it as a component

Declare the component the rows are read as, and load the table in setup:

```ts
const Item = defineComponent("Item", { damage: u32(), weight: f32(), name: str(48), ammo: TableRow() });
og.table.load(world, "items.ogt", Item);
```

- Every field of every row is checked against the component: a name it
  lacks, a value of the wrong kind, a string past its capacity, an enum's
  unknown variant. One bad field refuses the whole table with the table,
  line, row and field; a table loaded before stays as it was.
- og.table.parse reads text instead of a cooked file, under a path of
  your choosing, with the same reader and checks: content a game
  generates, and tests. og cook checks a file's assets and `row` names
  besides; parse checks the rows only when they are read.
- Loading a path again replaces its rows. A hot reload runs setup again,
  so an edited table is in effect after it, and every handle still names
  its row.
- The component is a schema here, not data: rows are read with
  og.table.get, never stored whole on entities.

<a id="guide-data-3"></a>
### `data §3` Holding rows in the world

Store a row's handle, never its values, in the component that holds it
(an inventory slot, an equipped weapon, the next wave): a TableRow field.
The snapshot then holds an 8-byte handle, the values stay in the table,
and an edited table changes what every holder reads.

- A handle is a hash of the table's asset id and the row's id, kept to
  52 bits so it is exact as a JavaScript number. It is the same in every
  run, on both hosts and after a reload, so a save restores by id: rows
  reordered or added since keep their handles. Two loaded rows that hash
  to one handle (about one chance in 10^9 among a hundred thousand rows)
  are refused at load, so a handle never names two rows.
- A row deleted since a save answers nothing: og.table.get refuses it.
  After og.WorldRestored, og.table.missing lists each component and
  resource field still holding one, so the game can drop, replace or
  report it; nothing is changed for it.
- og.table.name answers a handle's table and id: for a save's own
  migration, a debug line or a UI.
- In a system, a TableRow column (as an Entity column) holds two u32
  words a row, not handles: read row i's handle with
  `a.handle(Wields, "item", i)`, typed as the field's TableRow.

<a id="guide-data-4"></a>
### `data §4` Recipes

A table read into the world: a row's handle on an entity, its values read
where they are needed, and the row a field names followed.

```ts recipe
import { TableRow, defineComponent, defineSystem, f32, og, str, u32 } from "@origingame/engine";
import type { Result } from "@origingame/engine";

function must<T>(r: Result<T>): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const Item = defineComponent("Item", { damage: u32(), weight: f32(), name: str(32), ammo: TableRow() });
/** What an entity wields: a row of items.ogt. */
const Wields = defineComponent("Wields", { item: TableRow() });

const ITEMS = `type table
version 1
---
row "bow" {
  damage 7
  weight 1.5
  name text "items.bow" "Bow"
  ammo row "arrow"
}
row "arrow" {
  damage 3
  name text "items.arrow" "Arrow"
}
`;

const Check = defineSystem("Check", { reads: [Wields] }, (q, _time, world) => {
  for (const a of q.archetypes()) {
    for (let i = 0; i < a.count; i++) {
      // A TableRow column holds two words a row: a.handle answers row i's handle.
      const bow = must(og.table.get(world, a.handle(Wields, "item", i), Item));
      const arrow = must(og.table.get(world, bow.ammo, Item));
      og.assert.always(bow.damage === 7 && arrow.damage === 3, "a row and the row it names read back", {});
      og.assert.reachable("the wielded bow's arrow was read", {});
    }
  }
});

export default og.game({
  systems: [Check],
  start(world) {
    must(og.table.parse(world, "items.ogt", ITEMS, Item));
    const archer = must(world.spawn({ name: "archer" }));
    must(world.insert(archer, Wields, { item: must(og.table.row(world, "items.ogt", "bow")) }));
  },
});
```
