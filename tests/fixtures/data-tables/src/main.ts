// Two TS tables that name each other's rows, read as the game's components. An
// archer holds a bow and a quiver of arrows by handle; every tick the Check
// system follows the rows they name across both tables. The probe's
// commands replace items with structured edits (`reload`), with a broken row
// (`bad`) and without the arrow row (`drop`).
//
//   og run tests/fixtures/data-tables --headless --commands "2:dump_world;3:quit"
import { TableRow, defineComponent, defineResource, defineSystem, og, u32 } from "@origingame/engine";
import type { Result, World } from "@origingame/engine";
import { Item, Material } from "./schema/items.ts";
export { Item } from "./schema/items.ts";

function must<T>(r: Result<T>): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** What the archer holds: rows of base/items. */
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
  const sword = must(og.table.get(world, must(og.table.row(world, "base/items", "sword")), Item));
  const iron = must(og.table.get(world, must(og.table.row(world, "base/materials", "iron")), Material));
  f.swordDamage = sword.damage;
  f.ironMakes = must(og.table.get(world, iron.makes, Item)).damage;
  f.iconOk = sword.icon === "base/icons/sword" && sword.rarity === "rare" ? 1 : 0;
  og.assert.reachable("the tables were read", {});
});

function edited(world: World, dropArrow = false) {
  const row = (table: string, id: string) => must(og.table.row(world, table, id));
  return { rows: [
    ...(!dropArrow ? [{ id: "arrow", values: { damage: 4, name: "items.arrow" } }] : []),
    { id: "sword", values: { damage: 15, name: "items.sword", icon: "base/icons/sword", rarity: "rare", salvage: [row("base/materials", "iron")] } },
    { id: "bow", values: { damage: 7, name: "items.bow", ammo: dropArrow ? 0 : row("base/items", "arrow"), salvage: [row("base/materials", "wood")] } },
  ] };
}

function load(world: World): void {
  must(og.table.load(world, "base/items", Item));
  must(og.table.load(world, "base/materials", Material));
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
      weapon: must(og.table.row(world, "base/items", "bow")),
      quiver: must(og.table.row(world, "base/items", "arrow")),
    }));
  },
  commands: {
    // The same ids in another order with new numbers: every handle held still names its row.
    reload: (world) => {
      const before = must(og.table.row(world, "base/items", "arrow"));
      must(og.table.replace(world, "base/items", edited(world), Item));
      const f = must(world.resource(Facts));
      f.sameHandle = must(og.table.row(world, "base/items", "arrow")) === before ? 1 : 0;
      f.reloads += 1;
    },
    // Refused whole, naming the field; the table loaded before stays.
    bad: (world) => {
      const r = og.table.replace(world, "base/items", { rows: [{ id: "sword", values: { weight: "heavy" } }] }, Item);
      const f = must(world.resource(Facts));
      if (!r.ok) {
        f.refusals += 1;
        f.refusedLine = r.reason.includes("weight") ? 1 : 0;
      }
    },
    // The arrow row deleted: the quiver's handle answers nothing and og.table.missing names it.
    drop: (world) => {
      must(og.table.replace(world, "base/items", edited(world, true), Item));
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
