import { file, ref } from "@origingame/engine/author";

// Two walls repeating along x at their own parallax; the far one stays unlit.
export default file.tilemap({
  width: 16, height: 5, tileWidth: 32, tileHeight: 32, unit: 2,
  tilesets: [ref("base/bg/walls-set")],
  layers: [
    { name: "far", tileset: 0, z: -30, parallax: [0.3, 0.6], repeatX: true, lit: false,
      cells: Uint32Array.from({ length: 80 }, (_, i) => i + 1) },
    { name: "near", tileset: 0, z: -20, parallax: [0.6, 0.8], repeatX: true, lit: true,
      cells: Uint32Array.from({ length: 80 }, (_, i) => i + 81) },
  ],
});
