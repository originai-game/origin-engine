// Writes assets/nav/keep.ogt, the navmesh's bake file, from the yard's
// solid boxes (src/layout.ts): the floor and every wall, prop and the
// chest, so the risen walk round what the heroes bump into, and the gate's
// ward as an area they keep out of.
//
//   bun tools/make-nav.ts        # from the archetype directory; then og cook .
import { writeFileSync } from "node:fs";
import { NAV_BOUNDS, NAV_WARD, solidBoxes, WARD, YARD } from "../src/layout.ts";

const n = (v: number) => String(Math.round(v * 1000) / 1000);
const lines = [
  "type navmesh",
  "version 1",
  "---",
  "# Greywatch's yard, baked by og cook (og.nav.load(world, \"nav/keep.ogt\")):",
  "# the floor and every solid box of src/layout.ts, written by",
  "# tools/make-nav.ts. Edit the layout and run it again, never this file.",
  "agent {",
  "  radius 0.45",
  "  height 1.8",
  "  climb 0.3",
  "  slope 40",
  "}",
  "cellSize 0.2",
  "cellHeight 0.1",
  "tileSize 32",
  "bounds {",
  `  min ${NAV_BOUNDS.min.map(n).join(" ")}`,
  `  max ${NAV_BOUNDS.max.map(n).join(" ")}`,
  "}",
  "box {",
  "  center 0 -0.1 0",
  `  size ${n(YARD.maxX - YARD.minX + 4)} 0.2 ${n(YARD.maxZ - YARD.minZ + 4)}`,
  "}",
];
for (const b of solidBoxes()) lines.push("box {", `  center ${n(b.x)} ${n(b.y)} ${n(b.z)}`, `  size ${n(b.w)} ${n(b.h)} ${n(b.d)}`, "}");
// The gate's ward: an area the risen's filter leaves out of their paths.
lines.push("area {", `  id ${NAV_WARD}`, `  center ${n((WARD.minX + WARD.maxX) / 2)} 0 ${n((WARD.minZ + WARD.maxZ) / 2)}`, `  size ${n(WARD.maxX - WARD.minX)} 2 ${n(WARD.maxZ - WARD.minZ)}`, "}");
writeFileSync(new URL("../assets/nav/keep.ogt", import.meta.url), lines.join("\n") + "\n");
console.log("assets/nav/keep.ogt");
