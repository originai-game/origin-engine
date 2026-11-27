import { file } from "@origingame/engine/author";

export default file.table({ record: "ItemRow", rows: [
  { id: "coin", values: { maxStack: 50, name: { text: { key: "items.coin", base: "Coin" } } } },
  { id: "ore", values: { maxStack: 10, name: { text: { key: "items.ore", base: "Iron ore" } } } },
  { id: "ingot", values: { maxStack: 5, name: { text: { key: "items.ingot", base: "Iron ingot" } } } },
  { id: "sword", values: { maxStack: 1, name: { text: { key: "items.sword", base: "Sword" } } } },
] });
