import { file, ref } from "@origingame/engine/author";

export default file.table({ record: "Material", rows: [
  { id: "iron", values: { hardness: 5, makes: { table: ref("base/items"), row: "sword" } } },
  { id: "wood", values: { hardness: 2, makes: { table: ref("base/items"), row: "bow" } } },
] });
