// 2D light. The village is in daylight (og.Lighting2D's ambient near 1);
// walking into the upper hollow eases it down to the area's 0.2 and the map
// is tinted with it (src/props.ts MapShade: og.Lighting2D lights sprites
// but not a tilemap's layers, FINDINGS.md #F-tilelight). In a frame of the
// dark cave the pool round Wren's lantern and a torch's are far brighter
// than the rock between; the brighter wick widens Wren's light from 4.5 m
// to 7.
import { playtest } from "@origingame/engine/playtest";
import { join } from "node:path";
import { readFileSync } from "node:fs";
import { decodePng, outDir } from "./frames.ts";
import type { Image } from "./frames.ts";
import { game, script, value } from "./lib.ts";

/** The mean luminance (0..255) of a square of the frame. */
function lum(img: Image, cx: number, cy: number, r: number): number {
  let sum = 0;
  let n = 0;
  for (let y = cy - r; y <= cy + r; y++) {
    for (let x = cx - r; x <= cx + r; x++) {
      const i = (y * img.width + x) * 4;
      sum += 0.2126 * img.rgba[i]! + 0.7152 * img.rgba[i + 1]! + 0.0722 * img.rgba[i + 2]!;
      n++;
    }
  }
  return sum / n;
}

export default playtest({
  name: "daylight in the village, the dark hollow, light pools round the lantern and a torch, the wick's wider light",
  commands: script(
    "0.3:play",
    "1.0:get:Game:village.json",
    "1.0:get:og.Lighting2D:day.json",
    "1.1:warp:30:16",
    "3.5:get:Game:cave.json",
    "3.5:get:og.Lighting2D:dark.json",
    "3.5:get:map.og.Tilemap:tint.json",
    "3.5:get:og.Camera:camera.json",
    "3.6:screenshot:dark.png",
    "3.7:give:wick",
    "6.0:get:Game:wick.json",
    "6.0:get:hero.og.Light2D:light.json",
    "6.1:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const village = game(run, "village.json");
    t.ok(village.ambient > 0.95, `the village is in daylight (ambient ${village.ambient.toFixed(2)})`);
    t.ok(value<{ enabled: boolean }>(run, "day.json").enabled, "2D lighting is on");
    const cave = game(run, "cave.json");
    t.ok(cave.ambient < 0.3, `the upper hollow is dark (ambient ${cave.ambient.toFixed(2)})`);
    const dark = value<{ ambient: [number, number, number] }>(run, "dark.json").ambient;
    t.ok(dark[0] < 0.3 && dark[2] >= dark[0], `og.Lighting2D's ambient follows, cooler (${dark.map((c) => c.toFixed(2)).join(", ")})`);
    const tint = value<{ color: [number, number, number] }>(run, "tint.json").color;
    t.ok(tint[0] < 0.3, `the map is tinted with it (${tint.map((c) => c.toFixed(2)).join(", ")})`);
    // Where things are on the frame: 1280x720, the 14-cell view 3x of 224 lines, centred.
    const img = decodePng(readFileSync(join(outDir(), "dark.png")));
    const cam = value<{ position: [number, number, number] }>(run, "camera.json").position;
    const px = 3 * 16; // screen pixels per world unit
    const at = (x: number, y: number): [number, number] => [Math.round(640 + (x - cam[0]) * px), Math.round(360 - (y - cam[1]) * px)];
    const [hx, hy] = at(cave.heroX, cave.heroY + 0.7);
    const [tx, ty] = at(38.5, 48 - 25.4 - 0.6);
    const [dx, dy] = at(cave.heroX - 6, cave.heroY - 1.5);
    const hero = lum(img, hx + 30, hy, 6);
    const torch = lum(img, tx, ty + 40, 6);
    const rock = lum(img, dx, dy, 6);
    t.ok(hero > rock * 2 && torch > rock * 2, `light pools: by Wren ${hero.toFixed(0)}, under the torch ${torch.toFixed(0)}, the rock between ${rock.toFixed(0)}`);
    const wick = game(run, "wick.json");
    t.ok(wick.wick && value<{ radius: number }>(run, "light.json").radius > 6.5, `the wick widened the light (${value<{ radius: number }>(run, "light.json").radius.toFixed(2)} m)`);
  },
});
