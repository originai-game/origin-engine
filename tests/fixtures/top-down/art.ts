// Deterministic CC0 farm tiles, props and wind-cut trees.
// bun tests/fixtures/top-down/art.ts [fixture-dir]
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join, basename } from "node:path";
import { PixelArt, rgb, mix } from "../../../scripts/pixel-art.ts";
import { Random } from "../../../scripts/random.ts";
const HERE = process.argv[2] ?? import.meta.dir,
  T = 16;
class Canvas extends PixelArt {
  blob(cx: number, cy: number, rx: number, ry: number, light: string, mid: string, dark: string, seed = 0, rough = 0): void {
    const rnd = new Random(seed);
    for (let y = Math.trunc(cy - ry - 1); y < Math.trunc(cy + ry + 2); y++)
      for (let x = Math.trunc(cx - rx - 1); x < Math.trunc(cx + rx + 2); x++) {
        const dx = (x + 0.5 - cx) / rx,
          dy = (y + 0.5 - cy) / ry,
          d = dx * dx + dy * dy;
        if (d > 1 + rough * (rnd.random() - 0.5)) continue;
        const s = dx * 0.6 + dy * 0.8;
        this.put(x, y, s < -0.35 ? light : s < 0.35 ? mid : dark);
      }
  }
}
const GRASS = ["#5f9a3c", "#538a35", "#6eab45", "#467a2e"].map(rgb),
  SOIL = ["#8a5a35", "#76492a", "#9c6a40", "#5f3a22"].map(rgb),
  WET = ["#5e3b25", "#4f301d", "#6b4430", "#3f2717"].map(rgb),
  PATH = ["#b99566", "#a7845a", "#c9a676", "#8e6f4a"].map(rgb);
function tile(seed: number, pal: number[][], kind: string): PixelArt {
  const rnd = new Random(seed),
    c = new Canvas(T, T);
  c.rect(0, 0, T, T, pal[0]!);
  if (kind === "grass") {
    for (let i = 0; i < 18; i++) {
      const x = rnd.randrange(T),
        y = rnd.randrange(1, T);
      c.put(x, y, pal[2]!);
      c.put(x, y - 1, rnd.random() < 0.5 ? pal[2]! : pal[1]!);
    }
    for (let i = 0; i < 10; i++) c.put(rnd.randrange(T), rnd.randrange(T), pal[3]!);
  } else if (kind === "flowers") {
    for (let i = 0; i < 12; i++) c.put(rnd.randrange(T), rnd.randrange(T), pal[2]!);
    for (const col of ["#f2e6a0", "#e98fb0", "#f4f4f4"]) {
      const x = rnd.randrange(1, T - 1),
        y = rnd.randrange(1, T - 1);
      c.put(x, y, col);
      c.put(x + 1, y, mix(rgb(col), pal[0]!, 0.4));
    }
  } else if (kind === "soil") {
    for (let y = 0; y < T; y++)
      for (let x = 0; x < T; x++) {
        if (y % 4 === 3) c.put(x, y, pal[3]!);
        else if (y % 4 === 0 && rnd.random() < 0.6) c.put(x, y, pal[2]!);
        else if (rnd.random() < 0.12) c.put(x, y, pal[1]!);
      }
  } else if (kind === "path") {
    for (let i = 0; i < 26; i++) c.put(rnd.randrange(T), rnd.randrange(T), pal[rnd.choice([1, 2])]!);
    for (let i = 0; i < 3; i++) {
      const x = rnd.randrange(T - 1),
        y = rnd.randrange(T - 1);
      c.rect(x, y, 2, 1, pal[2]!);
      c.rect(x, y + 1, 2, 1, pal[3]!);
    }
  }
  return c;
}
function save(img: PixelArt, ...path: string[]): void {
  img.save(join(HERE, "assets", ...path));
  const declaration = join(HERE, "content", ...path).replace(/\.[^.]+$/, ".ts");
  mkdirSync(dirname(declaration), { recursive: true });
  const settings = {
    kind: "texture",
    source: `assets/${path.join("/")}`,
    ...(["tiles", "trees"].includes(path[0]!) ? { usage: "pixel" } : {}),
    ...(path[0] === "trees" ? { sway: { kind: basename(path.at(-1)!, ".png") === "tree" ? "clumps" : "tiers" } } : {}),
  };
  writeFileSync(
    declaration,
    "import {file} from '@origingame/engine/author';\nexport default file.import(" + JSON.stringify(settings, null, 2) + ");\n",
  );
}
const sheet = new PixelArt(T * 8, T * 2),
  kinds: [string, number[][]][] = [
    ...Array.from({ length: 4 }, () => ["grass", GRASS] as [string, number[][]]),
    ["flowers", GRASS],
    ["soil", SOIL],
    ["soil", SOIL],
    ["soil", WET],
    ...Array.from({ length: 4 }, () => ["path", PATH] as [string, number[][]]),
  ];
kinds.forEach(([kind, pal], i) => sheet.blit(tile(100 + i, pal, kind), (i % 8) * T, Math.floor(i / 8) * T));
save(sheet, "tiles", "farm.png");
const LEAF = ["#7cc04f", "#4f9a3a", "#2f6b2e"] as const;
function crop(stage: number, seed: number, fruit?: [string, string, string, number]): PixelArt {
  const c = new Canvas(14, 16),
    rnd = new Random(seed),
    n = stage + 1;
  for (let i = 0; i < n; i++) {
    const lx = 7 + rnd.randint(-3, 3),
      ly = 14 - i * 2 - rnd.randint(0, 1),
      side = i % 2 ? -1 : 1;
    for (let k = 0; k < 3; k++) c.put(lx + side * k, ly - Math.floor(k / 2), LEAF[k]!);
  }
  c.rect(7, 14 - n * 2, 1, n * 2 + 1, LEAF[1]);
  if (fruit) {
    const [light, mid, dark, r] = fruit;
    c.blob(7, 13 - r, r, r * 0.8, light, mid, dark);
  }
  return c.outlined();
}
function pumpkin(): PixelArt {
  const c = new Canvas(16, 14);
  c.blob(8, 8, 7, 5, "#f6a23c", "#e07a22", "#b4531a");
  for (const x of [5, 11]) for (let y = 4; y < 13; y++) c.put(x, y, "#b4531a");
  c.rect(7, 1, 2, 3, "#5d7a2e");
  return c.outlined();
}
function cauliflower(): PixelArt {
  const c = new Canvas(16, 14);
  c.blob(8, 9, 7, 4, ...LEAF, 3);
  c.blob(8, 7, 4.5, 3.5, "#fbf7e8", "#e9e1c6", "#c9bf9d", 4, 0.3);
  return c.outlined();
}
function wheat(seed: number): PixelArt {
  const c = new Canvas(14, 22),
    rnd = new Random(seed);
  for (let i = 0; i < 5; i++) {
    const x = 2 + i * 2 + rnd.randint(0, 1),
      top = rnd.randint(2, 6);
    c.rect(x, top + 4, 1, 21 - top - 4, "#b9a24a");
    for (let k = 0; k < 4; k++) {
      c.put(x, top + k, k % 2 ? "#f0d878" : "#d8b552");
      c.put(x + 1, top + k + 1, "#c79e3e");
    }
  }
  return c.outlined();
}
function sunflower(): PixelArt {
  const c = new Canvas(14, 30);
  c.rect(6, 10, 2, 19, LEAF[1]);
  c.rect(2, 18, 4, 2, LEAF[0]);
  c.rect(8, 22, 4, 2, LEAF[2]);
  c.blob(7, 6, 6, 6, "#ffe066", "#f3c02c", "#d39a1c");
  c.blob(7, 6, 2.5, 2.5, "#7a4a24", "#5a3418", "#3e2210");
  return c.outlined();
}
function tree(): PixelArt {
  const c = new Canvas(44, 60);
  c.rect(19, 36, 7, 22, "#6b4428");
  c.rect(19, 36, 2, 22, "#8a5a36");
  c.rect(24, 36, 2, 22, "#4f301d");
  c.rect(16, 56, 13, 2, "#4f301d");
  [
    [22, 24, 18],
    [12, 30, 10],
    [32, 30, 10],
    [22, 12, 12],
  ].forEach(([x, y, r], i) => c.blob(x!, y!, r!, r! * 0.85, "#5fae4a", "#3f8a3a", "#2a6332", i, 0.25));
  const rnd = new Random(2);
  for (let i = 0; i < 30; i++) c.put(rnd.randint(8, 36), rnd.randint(6, 30), "#78c25a");
  return c.outlined();
}
function pine(): PixelArt {
  const c = new Canvas(30, 52);
  c.rect(13, 42, 4, 9, "#5a3820");
  for (let i = 0; i < 5; i++) {
    const y = 6 + i * 8,
      r = 4 + i * 2.6;
    for (let yy = y; yy < y + 10; yy++) {
      const half = Math.trunc((r * (yy - y + 2)) / 12);
      for (let x = 15 - half; x <= 15 + half; x++)
        c.put(x, yy, x > 15 + Math.floor(half / 3) ? "#2f6b3e" : x > 15 - Math.floor(half / 2) ? "#3f8a4c" : "#56a65a");
    }
  }
  return c.outlined();
}
function bush(seed: number): PixelArt {
  const c = new Canvas(22, 15);
  c.blob(11, 8, 10, 6, "#6ab24c", "#4a903c", "#2f6b32", seed, 0.3);
  const rnd = new Random(seed);
  for (let i = 0; i < 4; i++) c.put(rnd.randint(4, 18), rnd.randint(4, 10), "#e05a5a");
  return c.outlined();
}
function rock(): PixelArt {
  const c = new Canvas(16, 12);
  c.blob(8, 7, 7, 4.5, "#b6b3ad", "#8e8a84", "#625e59", 5, 0.2);
  return c.outlined();
}
function stump(): PixelArt {
  const c = new Canvas(16, 12);
  c.rect(2, 4, 12, 7, "#6b4428");
  c.blob(8, 4, 6, 2.5, "#d6b07a", "#c09560", "#9c7446");
  return c.outlined();
}
function fenceH(): PixelArt {
  const c = new Canvas(16, 14);
  for (const x of [1, 14]) {
    c.rect(x - 1, 2, 3, 12, "#9c7446");
    c.rect(x - 1, 2, 1, 12, "#c09560");
  }
  c.rect(0, 5, 16, 2, "#b38a58");
  c.rect(0, 9, 16, 2, "#8a6238");
  return c.outlined();
}
function fencePost(): PixelArt {
  const c = new Canvas(5, 16);
  c.rect(1, 1, 3, 15, "#9c7446");
  c.rect(1, 1, 1, 15, "#c09560");
  return c.outlined();
}
function lamp(): PixelArt {
  const c = new Canvas(12, 34);
  c.rect(5, 10, 2, 23, "#3a3a44");
  c.rect(3, 31, 6, 2, "#2a2a32");
  c.rect(2, 3, 8, 7, "#2e2e36");
  c.rect(3, 4, 6, 5, "#ffe7a3");
  c.rect(4, 5, 4, 3, "#fff6d8");
  c.rect(1, 2, 10, 1, "#2e2e36");
  return c.outlined();
}
function crate(): PixelArt {
  const c = new Canvas(16, 15);
  c.rect(1, 1, 14, 13, "#b38a58");
  for (const y of [1, 7, 13]) c.rect(1, y, 14, 1, "#7a5532");
  for (const x of [1, 14]) c.rect(x, 1, 1, 13, "#7a5532");
  for (let k = 0; k < 5; k++) c.put(3 + k * 2, 2 + k, "#8a6238");
  return c.outlined();
}
function barrel(): PixelArt {
  const c = new Canvas(14, 17);
  for (let y = 1; y < 16; y++) {
    const w = [1, 15].includes(y) ? 5 : 6;
    for (let x = 7 - w; x < 7 + w; x++) c.put(x, y, x < 6 ? "#9c6a40" : x < 10 ? "#7a5030" : "#5f3a22");
  }
  for (const y of [4, 12]) c.rect(1, y, 12, 1, "#4a4a52");
  return c.outlined();
}
function chest(): PixelArt {
  const c = new Canvas(16, 13);
  c.rect(1, 3, 14, 9, "#9c6a40");
  c.rect(1, 1, 14, 4, "#b98450");
  c.rect(1, 5, 14, 1, "#5f3a22");
  c.rect(7, 4, 2, 3, "#f0c84a");
  return c.outlined();
}
function scarecrow(): PixelArt {
  const c = new Canvas(20, 30);
  c.rect(9, 8, 2, 21, "#7a5532");
  c.rect(2, 12, 16, 2, "#7a5532");
  c.rect(6, 12, 8, 9, "#5b7fb5");
  c.rect(6, 12, 2, 9, "#7a9cd0");
  c.blob(10, 7, 4, 4, "#f0dca0", "#d8c080", "#b09a5c");
  c.rect(4, 2, 12, 2, "#c79e3e");
  c.rect(6, 0, 8, 3, "#d8b552");
  return c.outlined();
}
function sign(): PixelArt {
  const c = new Canvas(14, 16);
  c.rect(6, 7, 2, 9, "#7a5532");
  c.rect(1, 1, 12, 7, "#b38a58");
  c.rect(3, 3, 8, 1, "#6b4428");
  c.rect(3, 5, 6, 1, "#6b4428");
  return c.outlined();
}
function mailbox(): PixelArt {
  const c = new Canvas(12, 18);
  c.rect(5, 8, 2, 10, "#6b4428");
  c.rect(1, 2, 10, 7, "#4a76b0");
  c.rect(1, 2, 10, 2, "#6b96d0");
  c.rect(9, 0, 2, 4, "#e05a5a");
  return c.outlined();
}
function flowers(seed: number, col: string): PixelArt {
  const c = new Canvas(14, 10),
    rnd = new Random(seed);
  for (let i = 0; i < 6; i++) {
    const x = rnd.randint(2, 11),
      y = rnd.randint(3, 8);
    c.rect(x, y, 1, 10 - y, LEAF[1]);
    c.put(x, y, col);
    c.put(x - 1, y, mix(rgb(col), [255, 255, 255], 0.3));
    c.put(x + 1, y, mix(rgb(col), [0, 0, 0], 0.25));
  }
  return c.outlined();
}
function hay(): PixelArt {
  const c = new Canvas(18, 13);
  c.blob(9, 7, 8, 5, "#f2d27a", "#d8b552", "#ad8a34", 8, 0.3);
  return c.outlined();
}
function well(): PixelArt {
  const c = new Canvas(24, 30);
  c.rect(3, 18, 18, 11, "#8e8a84");
  for (const y of [20, 24]) for (let x = 3; x < 21; x += 4) c.rect(x + (Math.floor(y / 4) % 2) * 2, y, 3, 1, "#625e59");
  c.rect(4, 8, 2, 11, "#6b4428");
  c.rect(18, 8, 2, 11, "#6b4428");
  c.rect(1, 4, 22, 4, "#a8423a");
  c.rect(1, 4, 22, 1, "#c95a4e");
  c.rect(5, 17, 14, 2, "#2a3a5c");
  return c.outlined();
}
const PROPS = {
  parsnip_0: () => crop(0, 1),
  parsnip_1: () => crop(1, 2),
  parsnip_2: () => crop(3, 3, ["#fbf2d0", "#ead9a4", "#c7b27a", 2.5]),
  tomato: () => crop(4, 4, ["#ff7a6a", "#e0453a", "#a8282a", 2.2]),
  pumpkin,
  cauliflower,
  wheat_a: () => wheat(1),
  wheat_b: () => wheat(2),
  sunflower,
  bush_a: () => bush(1),
  bush_b: () => bush(2),
  rock,
  stump,
  fence_h: fenceH,
  fence_post: fencePost,
  lamp,
  crate,
  barrel,
  chest,
  scarecrow,
  sign,
  mailbox,
  flowers_a: () => flowers(1, "#f7e26b"),
  flowers_b: () => flowers(2, "#e98fb0"),
  flowers_c: () => flowers(3, "#a0c4ff"),
  hay,
  well,
};
const sources = [
    "import { file } from '@origingame/engine/author';",
    "export default file.flipbook({",
    "  align: 'feet', crop: 'each', padding: 1, filter: 'nearest',",
    "  sources: [",
  ],
  sizes: string[] = [],
  trees: string[] = [];
for (const [name, fn] of Object.entries(PROPS)) {
  const img = fn();
  save(img, "props", `${name}.png`);
  sources.push(`    { source: 'assets/props/${name}.png' },`);
  const [x0, y0, x1, y1] = img.bounds();
  sizes.push(`  ${name}: { w: ${x1 - x0 + 2}, h: ${y1 - y0 + 2} },`);
}
sources.push("  ],", "});");
writeFileSync(join(HERE, "content/props/props.ts"), sources.join("\n") + "\n");
for (const [name, fn] of Object.entries({ tree, pine })) {
  const raw = fn(),
    [x0, y0, x1, y1] = raw.bounds(),
    img = raw.crop([x0 - 1, y0 - 1, x1 + 1, y1 + 1]);
  let sum = 0,
    count = 0;
  for (let y = 0; y < img.height; y++)
    for (let x = 0; x < img.width; x++)
      if (img.get(x, y)[3]! >= 16) {
        sum += x + 0.5;
        count++;
      }
  const center = sum / count,
    lo = Math.floor(center),
    rounded = center - lo === 0.5 ? lo + (lo % 2) : Math.round(center);
  save(img, "trees", `${name}.png`);
  trees.push(
    `  ${name}: { w: ${img.width}, h: ${img.height}, anchor: { x: ${(rounded / img.width).toFixed(7)}, y: ${(1 / img.height).toFixed(7)} } },`,
  );
}
mkdirSync(join(HERE, "src"), { recursive: true });
writeFileSync(
  join(HERE, "src/art.ts"),
  "// Generated by art.ts: each prop's cell in base/props/props (texels), the trees, and the map's layout.\nexport const PROPS = {\n" +
    sizes.join("\n") +
    "\n} as const;\n// Each tree's texture in assets/trees/ (texels) and where it stands in it, from its bottom left.\nexport const TREES = {\n" +
    trees.join("\n") +
    "\n} as const;\nexport const MAP = { w: 48, h: 30, pathX: [22, 23], pathY: [17, 18], plots: [[8, 5, 10, 8], [27, 5, 12, 8], [8, 21, 10, 6]] } as const;\n",
);
