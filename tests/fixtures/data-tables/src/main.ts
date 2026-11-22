// Two tables that name each other's rows (assets/items.ogt,
// assets/materials.ogt), read as the game's components (README.md). An
// archer holds a bow and a quiver of arrows by handle; every tick the Check
// system follows the rows they name across both tables. The probe's
// commands re-parse items.ogt with an edit (`reload`), with a broken row
// (`bad`) and without the arrow row (`drop`).
//
//   og run tests/fixtures/data-tables --headless --commands "2:dump_world;3:quit"
import { TableRow, defineComponent, defineResource, defineSystem, f32, list, og, str, u32 } from "@origingame/engine";
import type { Result, World } from "@origingame/engine";

function must<T>(r: Result<T>): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

export const Item = defineComponent("Item", {
  damage: u32(),
  weight: f32(),
  rarity: str(16),
  icon: str(64),
  name: str(32),
  ammo: TableRow(),
  salvage: list(TableRow(), 4),
});
const Material = defineComponent("Material", { hardness: u32(), makes: TableRow() });

/** What the archer holds: rows of items.ogt. */
export const Holds = defineComponent("Holds", { weapon: TableRow(), quiver: TableRow() });

/** What the game read back from the tables. */
const Facts = defineResource("Facts", {
  swordDamage: u32(),
  arrowDamage: u32(),
  salvage: u32(),
  ironMakes: u32(),
  iconOk: u32(),
  nameOk: u32(),
  sameHandle: u32(),
  reloads: u32(),
  refusedLine: u32(),
  refusals: u32(),
  missing: u32(),
});

const Check = defineSystem("Check", { reads: [Holds], writes: [Facts] }, (q, _time, world) => {
  const f = must(world.resource(Facts));
  f.missing = og.table.missing(world).length;
  for (const a of q.archetypes()) {
    for (let i = 0; i < a.count; i++) {
      const holds = must(world.get(a.entity(i), Holds));
      const bow = must(og.table.get(world, holds.weapon, Item));
      // The bow's ammo row is the one in the archer's quiver: one handle, one row.
      og.assert.always(bow.ammo === holds.quiver || f.missing > 0, "the bow's ammo is the quiver's row", {});
      const arrow = og.table.get(world, holds.quiver, Item);
      if (arrow.ok) f.arrowDamage = arrow.value.damage;
      f.salvage = bow.salvage.length;
      // items -> materials -> items: the wood the bow salvages into makes the bow.
      const wood = must(og.table.get(world, bow.salvage[0]!, Material));
      og.assert.always(wood.makes === holds.weapon, "a row two tables away names back", {});
    }
  }
  const sword = must(og.table.get(world, must(og.table.row(world, "items.ogt", "sword")), Item));
  const iron = must(og.table.get(world, must(og.table.row(world, "materials.ogt", "iron")), Material));
  f.swordDamage = sword.damage;
  f.ironMakes = must(og.table.get(world, iron.makes, Item)).damage;
  f.iconOk = sword.icon === "icons/sword.svg" && sword.rarity === "rare" ? 1 : 0;
  og.assert.reachable("the tables were read", {});
});

const ITEMS_EDITED = `type table
version 1
---
row "arrow" {
  damage 4
  name text "items.arrow" "Arrow"
}
row "sword" {
  damage 15
  name text "items.sword" "Sword"
  icon asset "icons/sword.svg"
  rarity "rare"
  salvage row "materials.ogt" "iron"
}
row "bow" {
  damage 7
  name text "items.bow" "Bow"
  ammo row "arrow"
  salvage row "materials.ogt" "wood"
}
`;

/** Line 6 is the broken field: a string where damage takes a number. */
const ITEMS_BROKEN = `type table
version 1
---
row "sword" {
  damage 12
  weight "heavy"
}
`;

const ITEMS_NO_ARROW = ITEMS_EDITED.replace(/row "arrow" \{[^}]*\}\n/, "").replace(/  ammo row "arrow"\n/, "");

function load(world: World): void {
  must(og.table.load(world, "items.ogt", Item));
  must(og.table.load(world, "materials.ogt", Material));
}

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Facts],
  systems: [Check],
  start(world) {
    load(world);
    const f = must(world.resource(Facts));
    f.nameOk = og.i18n.t(world, "items.sword") === "Sword" ? 1 : 0;
    const archer = must(world.spawn({ name: "archer" }));
    must(world.insert(archer, Holds, {
      weapon: must(og.table.row(world, "items.ogt", "bow")),
      quiver: must(og.table.row(world, "items.ogt", "arrow")),
    }));
  },
  commands: {
    // The same ids in another order with new numbers: every handle held still names its row.
    reload: (world) => {
      const before = must(og.table.row(world, "items.ogt", "arrow"));
      must(og.table.parse(world, "items.ogt", ITEMS_EDITED, Item));
      const f = must(world.resource(Facts));
      f.sameHandle = must(og.table.row(world, "items.ogt", "arrow")) === before ? 1 : 0;
      f.reloads += 1;
    },
    // Refused whole, naming the line; the table loaded before stays.
    bad: (world) => {
      const r = og.table.parse(world, "items.ogt", ITEMS_BROKEN, Item);
      const f = must(world.resource(Facts));
      if (!r.ok) {
        f.refusals += 1;
        const line = /line (\d+)/.exec(r.reason);
        f.refusedLine = line ? Number(line[1]) : 0;
      }
    },
    // The arrow row deleted: the quiver's handle answers nothing and og.table.missing names it.
    drop: (world) => {
      must(og.table.parse(world, "items.ogt", ITEMS_NO_ARROW, Item));
    },
  },
  facts: (world) => {
    const f = world.resource(Facts);
    if (!f.ok) return {};
    const v = f.value;
    return {
      swordDamage: v.swordDamage, arrowDamage: v.arrowDamage, salvage: v.salvage, ironMakes: v.ironMakes,
      iconOk: v.iconOk, nameOk: v.nameOk, sameHandle: v.sameHandle, reloads: v.reloads,
      refusedLine: v.refusedLine, refusals: v.refusals, missing: v.missing,
    };
  },
});
