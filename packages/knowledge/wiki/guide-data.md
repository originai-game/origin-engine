<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `data` guide: Data: tables of items, abilities, recipes and waves

Keep data in `file.table` declarations. Each row has an identity and the fields
of a component the game declares. Cook validates and packs the rows; the world
holds stable TableRow handles, not copies of the data. The engine does not pick
values or balance them. The API is `og api show table`.

<a id="guide-data-1"></a>
### `data §1` Writing a table

Define a pure component in `src/schema/items.ts`, import it from game code, and
list that source in `project.ts`'s `schemas`. Run `og api gen --project .` for
project-bound types and `og check` for source errors. The game entry never runs
during schema discovery.

```ts
import { defineComponent, TableRow, u32, str, list } from "@origingame/engine/schema";
export const Item = defineComponent("Item", {
  damage: u32(), name: str(48), icon: str(128),
  ammo: TableRow(), salvage: list(TableRow(), 4),
});
```

`content/items.ts` exports one declaration:

```ts
import { file, ref } from "@origingame/engine/author";
export default file.table({ record: "Item", locale: "en", rows: [
  { id: "bow", values: {
    damage: 7, name: { text: { key: "items.bow", base: "Bow" } },
    icon: { asset: ref("base/icons/bow") }, ammo: { row: "arrow" },
    salvage: [{ table: ref("base/materials"), row: "wood" }],
  } },
  { id: "arrow", values: { damage: 3 } },
] });
```

- IDs are unique and nonempty. Array order is retained. Omitted fields take
  their declared defaults, including string and list defaults.
- Ordinary fields use the same author shapes as scenes: numbers, booleans,
  strings, enum names, vector tuples, linear RGB tuples or hex colors, and lists.
- A str field may use `{asset: ref('module/content-id')}`. Cook checks the
  reference and stores its actual cooked ID; the game decides when to load it.
- A str field may use `{text: {key, base}}`. It stores the localization key;
  loading adds its base message in `locale` (default `en`). A conflicting base
  message for the same key is refused. `og i18n extract` discovers these keys.
- A TableRow uses `{row: 'id'}` or `{table: ref('module/table'), row: 'id'}`.
  Lists contain these objects. Cook checks that each target row exists. Tables
  may refer to each other; references do not execute the target declaration.
- Typed asset columns use `ref` directly and are loaded through their registered
  columns at runtime. Asset identities are never guessed numeric handles.

Use separate files and modules for large datasets or regions. A row is identified
by its table's canonical content ID and row ID; moving it to another table changes
its identity. Import TS helpers for generation; use explicit input capabilities
for binary data. The same declaration and cache path serves small projects.

<a id="guide-data-2"></a>
### `data §2` Loading and replacing

```ts
og.table.load(world, "base/items", Item);
```

Load during setup. The table must declare this component. A refused load leaves
the previous table installed. Loading again replaces all its rows. A hot reload
runs setup again, so edited values appear without changing surviving row handles.

For data generated during play, use `og.table.replace(world, id, data, Component)`.
`data` is `{locale?, strings?, rows: [{id, values}]}`; `strings` maps localization
keys to base messages. Values are runtime component values,
including vector objects and numeric asset/row handles, not author ref tokens.
The entire replacement validates before installation. An unknown field, wrong
kind, duplicate ID or invalid capacity refuses the operation and names the row
and field. Runtime replacement has no source line; cook errors identify TS source.

<a id="guide-data-3"></a>
### `data §3` Holding rows in the world

Keep a TableRow field on the inventory slot, equipped weapon or next-wave record.
The snapshot stores its 8-byte handle while values stay in the loaded table.

- Handles hash the canonical table ID and row ID to 52 bits, exactly representable
  as a JavaScript number. Reordering or editing values does not change them.
  Two loaded rows that collide are refused rather than silently aliased.
- Deleted rows return no values. After WorldRestored, `og.table.missing` lists
  component and resource fields still holding handles absent from loaded tables.
  The game decides how to respond; the engine does not change them.
- `og.table.name` returns table ID, row ID and component name.
- In a system, TableRow columns hold two u32 words per row. Use
  `a.handle(Wields, 'item', i)` to read the handle.

<a id="guide-data-4"></a>
### `data §4` Recipes

Generated runtime rows and a handle stored on an entity:

```ts recipe
import { TableRow, defineComponent, defineSystem, og, u32 } from "@origingame/engine";
import type { Result } from "@origingame/engine";

function must<T>(r: Result<T>): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}
const Item = defineComponent("Item", { damage: u32(), ammo: TableRow() });
const Wields = defineComponent("Wields", { item: TableRow() });
const Check = defineSystem("Check", { reads: [Wields] }, (q, _time, world) => {
  for (const a of q.archetypes()) for (let i = 0; i < a.count; i++) {
    const bow = must(og.table.get(world, a.handle(Wields, "item", i), Item));
    const arrow = must(og.table.get(world, bow.ammo, Item));
    og.assert.always(bow.damage === 7 && arrow.damage === 3, "a row and its reference read back", {});
    og.assert.reachable("the wielded bow's arrow was read", {});
  }
});
export default og.game({ systems: [Check], start(world) {
  must(og.table.replace(world, "base/items", { rows: [{id: "arrow", values: {damage: 3}}] }, Item));
  const arrow = must(og.table.row(world, "base/items", "arrow"));
  must(og.table.replace(world, "base/items", { rows: [
    {id: "bow", values: {damage: 7, ammo: arrow}},
    {id: "arrow", values: {damage: 3}},
  ] }, Item));
  const archer = must(world.spawn({name: "archer"}));
  must(world.insert(archer, Wields, {item: must(og.table.row(world, "base/items", "bow"))}));
} });
```
