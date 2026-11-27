import { file, ref } from "@origingame/engine/author";

export default file.table({ record: "Item", rows: [
  { id: "sword", values: {
    damage: 12, weight: 3.5, rarity: "rare", icon: { asset: ref("base/icons/sword") },
    name: { text: { key: "items.sword", base: "Sword" } },
    salvage: [{ table: ref("base/materials"), row: "iron" }, { table: ref("base/materials"), row: "wood" }],
  } },
  { id: "bow", values: {
    damage: 7, weight: 1.5, name: { text: { key: "items.bow", base: "Bow" } },
    ammo: { row: "arrow" }, salvage: [{ table: ref("base/materials"), row: "wood" }],
  } },
  { id: "arrow", values: { damage: 3, weight: 0.1, name: { text: { key: "items.arrow", base: "Arrow" } } } },
] });
