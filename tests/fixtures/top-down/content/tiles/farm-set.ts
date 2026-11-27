import { file, ref } from "@origingame/engine/author";

// Grass 0–3, flowers 4, soil 5–6, watered soil 7, path 8–11.
export default file.tileset({
  texture: ref("base/tiles/farm"),
  tileWidth: 16, tileHeight: 16, columns: 8, rows: 2, filter: "nearest",
});
