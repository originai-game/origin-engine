import { file, ref } from "@origingame/engine/author";

// The far wall above the near one.
export default file.tileset({
  texture: ref("base/bg/walls"),
  tileWidth: 32, tileHeight: 32, columns: 16, rows: 10, filter: "nearest",
});
