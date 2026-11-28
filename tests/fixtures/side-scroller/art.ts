// Deterministic CC0 cave walls, floor and pixel props.
// bun tests/fixtures/side-scroller/art.ts [fixture-dir]
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { PixelArt, rgb } from "../../../scripts/pixel-art.ts";
import { Random } from "../../../scripts/random.ts";
const HERE = process.argv[2] ?? import.meta.dir,
  W = 512,
  H = 160;
function noise1(seed: number, n: number, octaves: number[]): number[] {
  const rnd = new Random(seed),
    out = Array(n).fill(0);
  let amp = 1,
    total = 0;
  for (const period of octaves) {
    const pts = Array.from({ length: Math.floor(n / period) }, () => rnd.random());
    for (let i = 0; i < n; i++) {
      const f = i / period,
        a = pts[Math.trunc(f) % pts.length]!,
        b = pts[(Math.trunc(f) + 1) % pts.length]!,
        t = f - Math.trunc(f);
      out[i] += amp * (a + (b - a) * t * t * (3 - 2 * t));
    }
    total += amp;
    amp *= 0.5;
  }
  return out.map((v) => v / total);
}
function save(img: PixelArt, ...path: string[]): void {
  img.save(join(HERE, "assets", ...path));
  const declaration = join(HERE, "content", ...path).replace(/\.[^.]+$/, ".ts");
  mkdirSync(dirname(declaration), { recursive: true });
  writeFileSync(
    declaration,
    "import {file} from '@origingame/engine/author';\nexport default file.import(" +
      JSON.stringify({ kind: "texture", source: `assets/${path.join("/")}`, ...(path[0] === "bg" ? { usage: "pixel" } : {}) }, null, 2) +
      ");\n",
  );
}
function wall(
  seed: number,
  base: number[],
  rim: number[],
  dark: number[],
  topDepth: number,
  bottomDepth: number,
  columns: number[][],
  holes: number[][],
): PixelArt {
  const img = new PixelArt(W, H),
    rnd = new Random(seed),
    top = noise1(seed, W, [64, 16, 4]),
    bot = noise1(seed + 1, W, [64, 16, 4]),
    spikes = Array(W).fill(0),
    solid = Array.from({ length: H }, () => Array(W).fill(false));
  for (let i = 0; i < Math.floor(W / 18); i++) {
    const c = rnd.randrange(W),
      w = rnd.randint(3, 9),
      d = rnd.randint(10, topDepth);
    for (let dx = -w; dx <= w; dx++) {
      const x = (c + dx + W) % W;
      spikes[x] = Math.max(spikes[x], d * (1 - Math.abs(dx) / (w + 1)) ** 1.6);
    }
  }
  for (let x = 0; x < W; x++) {
    const t = Math.trunc(8 + top[x]! * topDepth * 0.6 + spikes[x]),
      b = Math.trunc(H - 6 - bot[x]! * bottomDepth);
    for (let y = 0; y < H; y++) solid[y]![x] = y < t || y >= b;
  }
  for (const [cx, w] of columns) {
    const prof = noise1(seed + cx!, H, [32, 8]);
    for (let y = 0; y < H; y++) {
      const pinch = 0.55 + 0.45 * Math.abs(y / H - 0.5) * 2,
        half = Math.trunc(w! * pinch * (0.8 + 0.4 * prof[y]!));
      for (let dx = -half; dx <= half; dx++) solid[y]![(cx! + dx + W) % W] = true;
    }
  }
  for (const [hx, hy, hr] of holes)
    for (let y = 0; y < H; y++) for (let x = 0; x < W; x++) if ((x - hx!) ** 2 + ((y - hy!) * 1.4) ** 2 < hr! * hr!) solid[y]![x] = false;
  const shade = noise1(seed + 7, W * 4, [24, 6]);
  for (let y = 0; y < H; y++)
    for (let x = 0; x < W; x++)
      if (solid[y]![x]) {
        const up = y > 0 && !solid[y - 1]![x],
          up2 = y > 1 && !solid[y - 2]![x],
          left = !solid[y]![(x + W - 1) % W],
          band = shade[(x + y * 3) % shade.length]!;
        const c = up
          ? rim
          : up2 || left
            ? rim.map((v, i) => Math.floor((v + base[i]!) / 2))
            : band < 0.42 && (x + y) % 2 === 0
              ? dark
              : base;
        img.put(x, y, c);
      }
  return img;
}
const walls = new PixelArt(W, H * 2);
walls.blit(
  wall(
    3,
    rgb("#1c2440"),
    rgb("#3b4f7a"),
    rgb("#161c33"),
    46,
    40,
    [
      [90, 10],
      [300, 14],
      [430, 8],
    ],
    [
      [200, 20, 26],
      [370, 140, 20],
    ],
  ),
  0,
  0,
);
walls.blit(
  wall(
    11,
    rgb("#121726"),
    rgb("#2b3756"),
    rgb("#0d111c"),
    60,
    52,
    [
      [40, 22],
      [250, 28],
      [400, 18],
    ],
    [],
  ),
  0,
  H,
);
save(walls, "bg", "walls.png");
function ground(): void {
  const h = 96,
    rnd = new Random(23),
    img = new PixelArt(W, h),
    seeds = Array.from({ length: 260 }, () => {
      const y = h * rnd.random() ** 1.3;
      return {
        x: rnd.uniform(0, W),
        y,
        shade: rnd.uniform(0.75, 1.2),
        c: rgb(rnd.choice(["#2c3043", "#272a3a", "#33354b", "#2e2c3a", "#2a3140"])),
      };
    });
  const cells = Array.from({ length: h }, (_, y) =>
    Array.from({ length: W }, (_, x) => {
      let best = 1e9,
        second = 1e9,
        k = 0;
      seeds.forEach((s, i) => {
        let dx = Math.abs(x + 0.5 - s.x);
        dx = Math.min(dx, W - dx);
        const d = dx * dx + (y + 0.5 - s.y) * (y + 0.5 - s.y) * 1.8;
        if (d < best) {
          second = best;
          best = d;
          k = i;
        } else if (d < second) second = d;
      });
      return [k, Math.sqrt(second) - Math.sqrt(best)] as const;
    }),
  );
  const edge = noise1(5, W, [32, 8]);
  for (let y = 0; y < h; y++)
    for (let x = 0; x < W; x++) {
      if (y < Math.trunc(edge[x]! * 4)) continue;
      const [k, gap] = cells[y]![x]!;
      let { shade, c } = seeds[k]!;
      if (gap < 1.6) {
        c = [9, 9, 14];
        shade = 1;
      } else if (y === 0 || cells[y - 1]![x]![0] !== k || cells[y - 1]![x]![1] < 1.6)
        c = c.map((v) => Math.min(255, Math.trunc(v * 1.6 + 16)));
      else if (y + 1 < h && cells[y + 1]![x]![1] < 1.6) c = c.map((v) => Math.trunc(v * 0.6));
      const fade = 0.3 + 0.7 * (1 - y / h) ** 1.4;
      img.put(
        x,
        y,
        c.map((v) => Math.trunc(v * shade * fade)),
      );
    }
  for (let i = 0; i < 9; i++) {
    let x = rnd.randrange(W);
    const end = rnd.randint(18, 40);
    for (let y = 3; y < end; y++) {
      x = (x + rnd.choice([-1, 0, 0, 1]) + W) % W;
      const f = (1 - y / h) ** 1.5;
      img.put(
        x,
        y,
        [64, 46, 34].map((v) => Math.trunc(v * f)),
      );
    }
  }
  save(img, "bg", "ground.png");
}
ground();
function draw(w: number, h: number, fn: (x: number, y: number) => string | undefined): PixelArt {
  const img = new PixelArt(w, h);
  for (let y = 0; y < h; y++)
    for (let x = 0; x < w; x++) {
      const c = fn(x, y);
      if (c) img.put(x, y, c);
    }
  return img;
}
const rnd = new Random(5),
  crack = Array.from({ length: 22 }, () => rnd.randint(0, 6));
const pillar = draw(22, 84, (x, y) => {
  if (y < 4 + crack[x]!) return;
  if (y >= 78 || (y >= 8 + crack[x]! && y < 14)) return y === 78 || y === 8 + crack[x]! ? "#55607a" : x % 7 ? "#3a4258" : "#2c3245";
  if (x < 2 || x >= 20) return;
  const flute = (x - 2) % 5;
  return flute === 0 ? "#262b3b" : flute === 1 ? "#4d5872" : (Math.floor(y / 9) + x) % 6 ? "#3a4258" : "#333a4f";
});
const torch = draw(9, 20, (x, y) => {
  if (x >= 3 && x <= 5 && y >= 4 && y < 16) return x !== 5 ? "#6b4426" : "#4a2e19";
  if ([8, 9].includes(y) && x >= 1 && x <= 7) return "#3b3d45";
  if (y >= 16 && y < 20 && x >= 2 && x <= 6) return "#2a2c33";
  if (y >= 2 && y < 5 && x >= 2 && x <= 6) return "#1f1208";
});
const flame = draw(9, 14, (x, y) => {
  const cx = 4 + Math.sin(y * 0.9) * 0.6,
    r = (y / 14) ** 0.8 * 4.2,
    d = Math.abs(x - cx);
  if (d > r) return;
  const t = (d / Math.max(r, 0.01)) * 0.6 + (1 - y / 14) * 0.6;
  return t < 0.35 ? "#fff4c8" : t < 0.6 ? "#ffc24a" : t < 0.85 ? "#ff7a1c" : "#c2390e";
});
function tufts(seed: number): PixelArt {
  const rnd = new Random(seed),
    blades = Array.from({ length: 14 }, () => [rnd.randrange(24), rnd.randint(2, 7)]);
  return draw(24, 7, (x, y) => {
    for (const [bx, bh] of blades) if (x === bx && y >= 7 - bh!) return y < 7 - bh! + 2 ? "#3f7a6e" : "#24514c";
    if (y === 6 && rnd.random() < 0.9) return "#1d3d3b";
  });
}
const mushroom = draw(17, 12, (x, y) => {
  for (const [cx, cy, r] of [
    [3, 6, 3],
    [9, 3, 4],
    [14, 7, 2],
  ]) {
    if (y >= cy! && y < cy! + 2 && Math.abs(x - cx!) <= r!) return y === cy && Math.abs(x - cx!) < r! ? "#9ff4ff" : "#45c6dd";
    if (x === cx && y >= cy! + 2 && y < 12) return "#c9d6d8";
  }
});
const sword = draw(7, 26, (x, y) => {
  if (x === 3 && y < 3) return "#8a6a3a";
  if (y === 3 && x <= 6) return "#6d717d";
  if (x >= 2 && x <= 4 && y >= 4 && y < 26) return x === 2 ? "#c4c9d4" : x === 3 ? "#8d93a1" : "#5d6270";
});
const chain = draw(3, 60, (x, y) => {
  const k = y % 6;
  if ((k < 3 && [0, 2].includes(x) && k > 0) || ([0, 3].includes(k) && x === 1) || (k >= 3 && x === 1))
    return k !== 4 ? "#4b4e58" : "#6c707c";
});
const moon = draw(27, 27, (x, y) => {
  const d = Math.hypot(x - 13, y - 13);
  if (d > 13) return;
  const crater = Math.hypot(x - 8, y - 9) < 3 || Math.hypot(x - 17, y - 16) < 2;
  return crater ? "#c7d4ef" : d < 11.5 ? "#eef3ff" : "#a9b9df";
});
const sheet = [
    "import { file } from '@origingame/engine/author';",
    "export default file.flipbook({",
    "  align: 'feet', crop: 'each', padding: 1, filter: 'nearest',",
    "  sources: [",
  ],
  sizes: string[] = [];
for (const [name, img] of [
  ["pillar", pillar],
  ["torch", torch],
  ["flame", flame],
  ["tufts_a", tufts(1)],
  ["tufts_b", tufts(2)],
  ["mushroom", mushroom],
  ["sword", sword],
  ["chain", chain],
  ["moon", moon],
] as const) {
  save(img, "props", `${name}.png`);
  sheet.push(`    { source: 'assets/props/${name}.png' },`);
  const [x0, y0, x1, y1] = img.bounds();
  sizes.push(`  ${name}: { w: ${x1 - x0 + 2}, h: ${y1 - y0 + 2} },`);
}
sheet.push("  ],", "});");
writeFileSync(join(HERE, "content/props/props.ts"), sheet.join("\n") + "\n");
mkdirSync(join(HERE, "src"), { recursive: true });
writeFileSync(
  join(HERE, "src/art.ts"),
  "// Generated by art.ts: each prop's cell in base/props/props, in texels.\nexport const PROPS = {\n" +
    sizes.join("\n") +
    "\n} as const;\n",
);
