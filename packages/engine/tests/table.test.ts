// og.table on the native host, with the data-tables fixture's game (an
// archer holding a bow and a quiver of arrows as rows of items.ogt).
//
// Ways it could fail, and the check that catches each:
// - a refusal that names no line, or a half-applied table → the broken
//   row's reason names its line, row and field, and the old rows still read
// - handles from the row's index, not its id → a save restored after the
//   rows were reordered reads the same rows
// - a deleted row read as zeros, or never reported → og.table.missing names
//   the entity, component and field still holding it after the restore
// - the collision check kept per table → two rows of two tables with one
//   handle (a real 52-bit FNV-1a collision) are refused
import { beforeAll, expect, test } from "bun:test";
import { og, type World } from "@origingame/engine";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type GameToken } from "../src/runtime.ts";
import { cookProject } from "./support/og-bin.ts";

const project = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..", "tests", "fixtures", "data-tables");
const fixture = (await import(join(project, "src", "main.ts"))) as typeof import("../../../tests/fixtures/data-tables/src/main.ts");
const { Item, Holds } = fixture;

beforeAll(() => {
  loadNativeHost();
  // The fixture's cooked/ is not tracked: cooked here, as og test cooks it.
  cookProject(project, join(project, "..", "..", ".."));
});

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};


const HEAD = "type table\nversion 1\n---\n";
const rows = (...r: [string, number][]) => HEAD + r.map(([id, d]) => `row "${id}" {\n  damage ${d}\n}\n`).join("");

function tables() {
  const engine = must(createEngine(fixture.default as GameToken, { root: project }));
  must(engine.advance(1));
  return { engine, world: engine.world as World };
}

test("a broken row is refused naming its line, row and field, and the table loaded before stays", () => {
  const { engine, world } = tables();
  const bow = must(og.table.row(world, "items.ogt", "bow"));
  const r = og.table.parse(world, "items.ogt", HEAD + 'row "bow" {\n  damage 9\n}\nrow "sword" {\n  damage "sharp"\n}\n', Item);
  expect(r.ok).toBe(false);
  if (!r.ok) expect(r.reason).toBe("items.ogt: line 8: row 'sword': field 'damage': a u32 field takes one number");
  expect(must(og.table.get(world, bow, Item)).damage).toBe(7);
  // A field the component lacks, and a row reference to a row the text lacks, are refused alike.
  const lacks = og.table.parse(world, "items.ogt", HEAD + 'row "bow" {\n  colour 3\n}\n', Item);
  expect(lacks.ok ? "" : lacks.reason).toBe("items.ogt: line 5: row 'bow': field 'colour': 'Item' has no such field");
  engine.destroy();
});

test("a save restored after its row was deleted reads nothing for it, and og.table.missing names who holds it", () => {
  const { engine, world } = tables();
  const archer = world.find("archer")!;
  const arrow = must(og.table.row(world, "items.ogt", "arrow"));
  const save = must(world.snapshot());

  // The arrow row deleted, the others reordered and renumbered.
  must(og.table.parse(world, "items.ogt", rows(["bow", 8], ["sword", 20]), Item));
  must(world.restore(save));
  expect(must(world.get(archer, Holds)).quiver).toBe(arrow);
  expect(og.table.get(world, arrow, Item).ok).toBe(false);
  expect(must(og.table.get(world, must(world.get(archer, Holds)).weapon, Item)).damage).toBe(8);
  expect(og.table.missing(world)).toEqual([{ entity: archer, record: "Holds", field: "quiver", row: arrow }]);

  // The row back under its id: the handle the save held reads it again.
  must(og.table.parse(world, "items.ogt", rows(["sword", 20], ["arrow", 5], ["bow", 8]), Item));
  expect(og.table.missing(world)).toEqual([]);
  expect(must(og.table.get(world, arrow, Item)).damage).toBe(5);
  expect(must(og.table.name(world, arrow))).toEqual({ table: "items.ogt", id: "arrow", schema: "Item" });
  engine.destroy();
});

test("two rows with one handle are refused across tables, as in one", () => {
  const { engine, world } = tables();
  // FNV-1a's low 52 bits of "a.ogt\0779d5caa6d657a20" and "b.ogt\0752d64ec8b92ba81" are both 1847583106041024.
  must(og.table.parse(world, "a.ogt", rows(["779d5caa6d657a20", 1]), Item));
  const b = og.table.parse(world, "b.ogt", rows(["752d64ec8b92ba81", 2]), Item);
  expect(b.ok ? "" : b.reason).toBe(
    "b.ogt: row '752d64ec8b92ba81' hashes to the handle of row '779d5caa6d657a20' of a.ogt: rename one",
  );
  expect(og.table.rows(world, "b.ogt").ok).toBe(false);
  // Loading a.ogt again replaces its own rows: no collision with itself.
  must(og.table.parse(world, "a.ogt", rows(["779d5caa6d657a20", 3]), Item));
  // "9d92866c2047ccb8" and "df085b7356583154" collide within a.ogt.
  const one = og.table.parse(world, "a.ogt", rows(["9d92866c2047ccb8", 1], ["df085b7356583154", 2]), Item);
  expect(one.ok ? "" : one.reason).toBe("a.ogt: rows '9d92866c2047ccb8' and 'df085b7356583154' hash to one handle: rename one");
  expect(must(og.table.get(world, must(og.table.row(world, "a.ogt", "779d5caa6d657a20")), Item)).damage).toBe(3);
  engine.destroy();
});

// ArchetypeView.handle: a TableRow or Entity column holds two words a row.
// - the words read as one handle each (`row[i]`) → og.table.get refuses it
// - the high word dropped or the words swapped → a 52-bit handle whose high
//   word is not 0 reads as another number than world.get's
// - row i read at word i, not 2i → the second entity reads the first's
test("a view answers a TableRow or Entity field's handle, as world.get does", async () => {
  const { Entity, TableRow, defineComponent, defineSystem, f32 } = await import("@origingame/engine");
  const Enemy = defineComponent("ViewEnemy", { hp: f32() });
  const Foe = defineComponent("ViewFoe", { t: f32(), row: TableRow(), leader: Entity() });
  const read: { hp: number; row: number; leader: number; want: { row: number; leader: number } }[] = [];
  let refused = "";
  const Read = defineSystem("Read", { reads: [Foe] }, (q, _time, world) => {
    for (const a of q.archetypes()) {
      for (let i = 0; i < a.count; i++) {
        const row = a.handle(Foe, "row", i);
        const want = must(world.get(a.entity(i), Foe));
        read.push({ hp: must(og.table.get(world, row, Enemy)).hp, row, leader: a.handle(Foe, "leader", i), want: { row: want.row, leader: want.leader } });
      }
      try {
        // @ts-expect-error `t` is an f32: no handle field.
        a.handle(Foe, "t", 0);
      } catch (e) {
        refused = (e as Error).message;
      }
    }
  });
  const engine = must(
    createEngine(
      og.game({
        systems: [Read],
        start(world: World) {
          must(og.table.parse(world, "enemies.ogt", HEAD + 'row "cross" {\n  hp 2\n}\nrow "ring" {\n  hp 5\n}\n', Enemy));
          const boss = must(world.spawn({ name: "boss" }));
          for (const id of ["cross", "ring"]) {
            must(world.insert(must(world.spawn({})), Foe, { row: must(og.table.row(world, "enemies.ogt", id)), leader: boss }));
          }
        },
      } as never) as GameToken,
      {},
    ),
  );
  must(engine.advance(1));
  expect(read.map((r) => r.hp)).toEqual([2, 5]);
  for (const r of read) expect({ row: r.row, leader: r.leader }).toEqual(r.want);
  // The handles use their high word: one that dropped it would differ.
  expect(read.every((r) => r.row > 2 ** 32)).toBe(true);
  expect(refused).toBe("'ViewFoe.t' holds no single handle: ArchetypeView.handle reads an Entity, a TableRow or an asset field");
  engine.destroy();
});
