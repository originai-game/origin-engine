// Makes the 2D preview scene's art (templates/looks/preview/README.md): for
// each look and variant of preview.json's 2D entries and each beat of the
// scene it has no art for yet, the look's art brief (packages/knowledge's
// artBrief, in the variant) is sent to the origin gateway's image generator,
// the answer is journalled and kept whole under target/look-previews/raw/,
// and the picture is cut to its subject and brought down to the size the
// look's contract gives it (four source pixels per logical pixel for the
// pixel looks, 128 per metre for the others, their tree line 64), into the look's art directory.
// The cook then conforms it (docs/art.md §2.3).
//
//   bun templates/looks/preview/2d/tools/art.ts plan [name ...]               what is missing, with each prompt
//   bun templates/looks/preview/2d/tools/art.ts make [--jobs N] [--only beat,beat] [--again] [name ...]
//   bun templates/looks/preview/2d/tools/art.ts prepare [--only beat,beat] [name ...]   again from the kept answers
//
// A name is a look slug or <slug>:<variant>; none is every 2D entry. The key
// is $OG_PLATFORM_DEVELOPMENT_KEY. A call that ends without an image is
// journalled and never retried by itself: run `make` again for what is
// still missing. `--again` makes a beat that already has art (its file is
// replaced).
import { appendFileSync, existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { artBrief, presetLook } from "../../../../../packages/knowledge/scripts/brief.ts";
import type { ArtKind } from "../../../../../packages/knowledge/scripts/brief.ts";
import { beatSize } from "../stage.ts";
import type { Scene2d } from "../stage.ts";

const here = import.meta.dir;
const repo = resolve(here, "../../../../..");
const looks = join(repo, "templates/looks");
const raw = join(repo, "target/look-previews/raw");
const journal = join(here, "gateway-journal.md");
const gateway = process.env.OG_GATEWAY_URL ?? "https://api.origingame.ai";

interface Entry extends Scene2d {
  scene: "2d" | "3d";
}
const table = JSON.parse(readFileSync(join(here, "../../preview.json"), "utf8")) as Record<string, Entry | string>;

/** How a beat's answer becomes its file. */
type Cut = "sprite" | "tiles" | "layer" | "full";
interface Beat {
  kind: ArtKind;
  subject: string;
  /** The generator's canvas. */
  canvas: "square" | "tall" | "wide";
  cut: Cut;
  /** Lines of the brief replaced: its opening, and (tiles) its tiling line. */
  opening?: string;
  tiling?: string;
  transparent?: boolean;
}

const SIDE: Record<string, Beat> = {
  backdrop: {
    kind: "backdrop",
    subject: "a wide valley at dusk: far mountains, a distant village with a bridge over a river, a low sun among banded clouds; the near ground is left out",
    canvas: "wide",
    cut: "full",
  },
  mid: {
    kind: "backdrop",
    subject: "a distant tree line",
    opening:
      "One parallax layer of a side-scrolling 2D game, standing between the far sky and the playfield: a long row of pines and broadleaf trees seen side-on with a ruined stone arch and a small watchtower among them, drawn one step quieter and hazier than the playfield's own art. The row stands on the bottom edge of the image and fills its whole width; it reaches about two thirds of the image's height. Everything above and behind the row is fully transparent: no sky, no clouds, no ground in front.",
    canvas: "wide",
    cut: "layer",
    transparent: true,
  },
  tiles: {
    kind: "tileset",
    subject: "the ground of a side-view platformer",
    opening:
      "A tile set for the ground of a side-view platformer, seen in cross-section from the side: 4 x 4 tiles on one grid, each tile meeting its neighbours edge to edge. The top row of tiles is grass growing along the top of the earth, the grass starting at the very top edge of the image with no sky above it; the three rows below are solid packed earth with a few stones and roots, a little darker towards the bottom. The grid itself is never drawn: no line, border, seam or gap between tiles; the grass and the earth run across them as one continuous surface. The whole image is filled.",
    tiling: "Tiling: the left edge continues the right, with no seam; the lowest row repeats under itself.",
    canvas: "square",
    cut: "tiles",
  },
  platform: {
    kind: "prop",
    subject: "a floating platform",
    opening:
      "A game prop sprite of a floating platform for a side-view platformer: three mossy stone blocks joined in one row, a flat slab three times as wide as it is tall, with a flat top to stand on, seen straight from the side.",
    canvas: "wide",
    cut: "sprite",
  },
  hero: { kind: "character", subject: "a hooded traveller in a cloak with a pack on the back, holding a lit lantern out in one hand, seen from the side and facing right", canvas: "tall", cut: "sprite" },
  tree: { kind: "prop", subject: "a tall pine tree with a short trunk, seen from the side", canvas: "tall", cut: "sprite" },
  crate: { kind: "prop", subject: "a wooden crate with a diagonal brace, seen straight from the front", canvas: "square", cut: "sprite" },
  lantern: { kind: "prop", subject: "a tall iron street lantern on a post, its lamp lit and hanging from a curled arm, seen from the side", canvas: "tall", cut: "sprite" },
  slime: { kind: "sprite", subject: "a small round slime monster, an enemy of a platformer, seen from the side and facing left", canvas: "square", cut: "sprite" },
  gem: { kind: "sprite", subject: "one faceted gem, the pickup a platformer's player collects", canvas: "square", cut: "sprite" },
  bush: { kind: "prop", subject: "a low clump of ferns and grass with a few small flowers, a tuft that stands on the ground in front of the playfield, wider than tall, seen from the side", canvas: "square", cut: "sprite" },
  heart: { kind: "sprite", subject: "one heart, the health marker of the game's HUD, seen flat from the front", canvas: "square", cut: "sprite" },
};

const view = "seen from the three-quarter top-down view of a 16-bit role-playing game (the camera high in front, looking down), standing upright";
const TOP: Record<string, Beat> = {
  grass: {
    kind: "tileset",
    subject: "a meadow floor",
    opening: "A tile set for the floor of a top-down game, seen straight from above: 4 x 4 tiles on one grid, each tile meeting its neighbours edge to edge. Every tile is short meadow grass, a few of them with small flowers or a tuft; no path, no shadow, no object. The whole image is filled.",
    canvas: "square",
    cut: "tiles",
  },
  path: {
    kind: "tileset",
    subject: "a paved path",
    opening: "A tile set for the floor of a top-down game, seen straight from above: 4 x 4 tiles on one grid, each tile meeting its neighbours edge to edge. Every tile is worn paving of fitted cobblestones, lighter and less saturated than grass; no border, no grass, no shadow, no object. The whole image is filled.",
    canvas: "square",
    cut: "tiles",
  },
  wall: {
    kind: "tileset",
    subject: "a cliff face",
    opening: "A tile set for the south face of a raised plateau in a top-down game, seen from the front: 4 x 4 tiles on one grid, each tile meeting its neighbours edge to edge. Every tile is a rock cliff wall of rough stone blocks, darker than the floor, darkest along the bottom row; no grass, no sky, no object. The whole image is filled.",
    tiling: "Tiling: the left edge continues the right, with no seam.",
    canvas: "square",
    cut: "tiles",
  },
  hero: { kind: "character", subject: `a hooded traveller in a cloak with a pack on the back, holding a lit lantern, ${view} and facing the viewer`, canvas: "tall", cut: "sprite" },
  tree: { kind: "prop", subject: `a round broadleaf tree, its crown over a short trunk, ${view}`, canvas: "tall", cut: "sprite" },
  crate: { kind: "prop", subject: `a wooden crate with a diagonal brace, ${view}: its top and its front face both show`, canvas: "square", cut: "sprite" },
  lantern: { kind: "prop", subject: `an iron street lantern on a post, its lamp lit, ${view}`, canvas: "tall", cut: "sprite" },
  slime: { kind: "sprite", subject: `a small round slime monster, an enemy, ${view}`, canvas: "square", cut: "sprite" },
  gem: { kind: "sprite", subject: "one faceted gem, the pickup the player collects", canvas: "square", cut: "sprite" },
  bush: { kind: "prop", subject: `a low round bush with a few small flowers, ${view}`, canvas: "square", cut: "sprite" },
  heart: { kind: "sprite", subject: "one heart, the health marker of the game's HUD, seen flat from the front", canvas: "square", cut: "sprite" },
};

/** What a look's art needs said beyond its brief, by look (watercolour: its playfield's beats; its far layers take FAR_WASH). */
const NOTES: Record<string, string> = {
  "cartoon-2d:watercolor":
    "Pigment: this stands in the playfield, in front of pale distance. Paint it in rich full-strength washes: saturated colour, deep mid and dark values, crisp edges where the pigment pools. The game thins every wash towards the paper and adds the paper's grain, so a pale picture would vanish. Still transparent watercolour: no opaque paint, no black line.",
  "pixel:monochrome":
    "Drawing: two values only. Solid shapes of the darkest colour on the lightest colour (the paper), with small marks of the lightest colour inside a dark shape for its detail; no mid-tone fill, no checker or dither filling a shape, no grey.",
  "cartoon-2d:storybook": "Colour: soft and a little chalky, as gouache on cream paper: sage, ochre, dusty rose and terracotta, nothing saturated or glossy.",
  "cartoon-2d:rubber-hose": "Colour: a faded 1930s print: sepia, ochre, grey-green and dull brick over cream, nothing saturated.",
};

/** What one beat of a look needs said beyond its brief: what the scene asks of it that the brief cannot know. */
const BEAT_NOTES: Record<string, Record<string, string>> = {
  "pixel:topdown": {
    hero: "Proportions: a chibi figure two heads tall: the hooded head is half the figure's height and as wide as the body, with two dark eyes of one block each; the lantern is one bright yellow block in the hand. Drawn in exactly 16 x 24 large square blocks and nothing finer: every shape is at least two blocks wide.",
  },
  "pixel:coarse": {
    hero: "Colour: the cloak is a pale warm ochre and the hood a lighter cream, the brightest figure of a dark blue-black scene; the legs are mid brown, the lantern one bright orange block. No black or navy on the figure.",
    slime: "Colour: one bright saturated lime-green mass with a lighter top edge, a bright accent in a dark blue-black scene; no eyes.",
    mid: "Drawing: the whole row is one flat silhouette in a single muted blue-grey, a step lighter than a night sky, with nothing inside it: no windows, no highlights, no second colour.",
  },
  "pixel:luminous": {
    hero: "Drawing: a simple chunky figure made for a 20 x 32 grid of large square blocks, nothing finer: a big pointed hood and head a third of the figure's height, one plain cloak to the knees, two boots, a round pack, the lantern held out as a three-block square. Large flat shapes at least three blocks wide; no straps, belt, buckle, folds or face detail. Light: the lantern lights the figure: the front half of the hood and cloak is one bright orange, the back half one saturated mid blue with a cyan edge, the lantern bright yellow; nothing on the figure is black or near black.",
    slime: "Light: the slime shines from within: a bright saturated cyan-green body, lightest at its core, with two dark eyes and a white sheen; nothing on it is black.",
    crate: "Light: lit warm from the left by a lantern: the planks are mid amber-brown, the left edges bright orange, the right edges saturated blue; no plank is black or near black.",
  },
  "pixel:detailed": {
    slime: "Reading: unmistakably a living creature, not a rock or a gem: a glossy green jelly dome with two large round white eyes with dark pupils on the side it faces, a small mouth, and one bright sheen on its top.",
  },
};

/** Watercolour's far layers: paler than its playfield, but not faint. */
const FAR_WASH = "Pigment: clear medium-strength washes, the far ranges paler than the near ones and the sky left as light paper. The game thins every wash towards the paper and adds the paper's grain, so a faint picture would vanish. Still transparent watercolour: no opaque paint, no black line.";

/** A look whose brief of a kind asks for more than one picture takes another kind for these beats. */
const KINDS: Record<string, Record<string, ArtKind>> = {
  // Its sprite brief asks for a flipbook sheet of frames; the scene takes one drawing.
  "cartoon-2d:rubber-hose": { slime: "prop", gem: "prop", heart: "prop" },
};

const CANVAS = { square: [1024, 1024], tall: [1024, 1536], wide: [1536, 1024] } as const;

const args = process.argv.slice(2);
const verb = args.shift();
const flag = (k: string) => {
  const i = args.indexOf(k);
  if (i < 0) return undefined;
  const v = args[i + 1]!;
  args.splice(i, 2);
  return v;
};
const jobs = Number(flag("--jobs") ?? 4);
const only = flag("--only")?.split(",");
const again = args.includes("--again");
const names = args.filter((a) => !a.startsWith("--"));
const todo = (names.length ? names : Object.keys(table)).filter((n) => typeof table[n] === "object" && (table[n] as Entry).scene === "2d");

function magick(...a: string[]): string {
  const r = spawnSync("magick", a, { encoding: "utf8" });
  if (r.status !== 0) throw new Error(`magick ${a.join(" ")}: ${r.stderr.trim()}`);
  return r.stdout;
}

const fileOf = (dir: string, beat: string) => (existsSync(dir) ? readdirSync(dir).find((n) => n.startsWith(`${beat}.`) && /\.(png|jpe?g)$/.test(n)) : undefined);

interface Job {
  name: string;
  beat: string;
  spec: Beat;
  entry: Entry;
  prompt: string;
  rawFile: string;
}

function plan(): Job[] {
  const out: Job[] = [];
  for (const name of todo) {
    const entry = table[name] as Entry;
    const [slug, variant] = name.split(":") as [string, string | undefined];
    const look = presetLook(slug, variant ? { variant } : {});
    const beats = entry.plan === "top" ? TOP : SIDE;
    for (const [beat, spec] of Object.entries(beats)) {
      if (only && !only.includes(beat)) continue;
      if (verb !== "prepare" && !again && fileOf(join(looks, entry.art!), beat)) continue;
      const metres = entry.heights?.[beat] ?? entry.widths?.[beat];
      const brief = artBrief(look, KINDS[name]?.[beat] ?? spec.kind, spec.subject, metres && spec.cut === "sprite" ? { metres: Math.max(metres, 0.5) } : {});
      const lines = brief.prompt.split("\n").filter((l) => !l.startsWith("Reference:"));
      if (spec.opening) lines[0] = spec.opening;
      const far = name === "cartoon-2d:watercolor" && (beat === "backdrop" || beat === "mid");
      if (far) lines.splice(1, 0, FAR_WASH);
      else if (NOTES[name]) lines.splice(1, 0, NOTES[name]!);
      const note = BEAT_NOTES[name]?.[beat];
      if (note) lines.splice(1, 0, note);
      const prompt = lines
        .map((l) => (spec.tiling && l.startsWith("Tiling:") ? spec.tiling : l))
        .map((l) => (spec.transparent && l.startsWith("Size:") ? `${l}\nBackground: fully transparent above and behind the layer.` : l))
        .join("\n");
      out.push({ name, beat, spec, entry, prompt, rawFile: join(raw, name.replace(":", "@"), `${beat}.png`) });
    }
  }
  return out;
}

/** An image's pixels, RGBA8. */
function pixels(file: string): { w: number; h: number; px: Buffer } {
  const [w, h] = magick("identify", "-format", "%w %h", file).split(" ").map(Number) as [number, number];
  const r = spawnSync("magick", [file, "-depth", "8", "rgba:-"], { maxBuffer: 1 << 28 });
  if (r.status !== 0) throw new Error(`magick ${file}: ${r.stderr}`);
  return { w, h, px: r.stdout };
}

/**
 * `src` brought down to tw x th logical pixels, each the colour most of its
 * block holds (colours within eight levels a channel counted as one) and
 * clear where under half of it is covered, then written at `scale` source
 * pixels a logical pixel: pixel art with hard cells, as the cook's own
 * dominant-colour box would leave it.
 */
function dominant(src: { w: number; h: number; px: Buffer }, tw: number, th: number, scale: number, out: string, opaque: boolean): void {
  const cells = Buffer.alloc(tw * th * 4);
  for (let y = 0; y < th; y++) {
    for (let x = 0; x < tw; x++) {
      const x0 = Math.floor((x * src.w) / tw), x1 = Math.max(x0 + 1, Math.floor(((x + 1) * src.w) / tw));
      const y0 = Math.floor((y * src.h) / th), y1 = Math.max(y0 + 1, Math.floor(((y + 1) * src.h) / th));
      const seen = new Map<number, [number, number, number, number]>();
      let covered = 0;
      for (let sy = y0; sy < y1; sy++) {
        for (let sx = x0; sx < x1; sx++) {
          const i = (sy * src.w + sx) * 4;
          if (!opaque && src.px[i + 3]! < 128) continue;
          covered++;
          const key = ((src.px[i]! >> 3) << 10) | ((src.px[i + 1]! >> 3) << 5) | (src.px[i + 2]! >> 3);
          const sum = seen.get(key) ?? [0, 0, 0, 0];
          sum[0] += src.px[i]!;
          sum[1] += src.px[i + 1]!;
          sum[2] += src.px[i + 2]!;
          sum[3]++;
          seen.set(key, sum);
        }
      }
      if (covered * 2 < (x1 - x0) * (y1 - y0)) continue;
      let best: [number, number, number, number] = [0, 0, 0, 0];
      for (const sum of seen.values()) if (sum[3] > best[3]) best = sum;
      const o = (y * tw + x) * 4;
      cells[o] = Math.round(best[0] / best[3]);
      cells[o + 1] = Math.round(best[1] / best[3]);
      cells[o + 2] = Math.round(best[2] / best[3]);
      cells[o + 3] = 255;
    }
  }
  const r = spawnSync("magick", ["-size", `${tw}x${th}`, "-depth", "8", "rgba:-", "-filter", "point", "-resize", `${scale * 100}%`, "-strip", out], { input: cells });
  if (r.status !== 0) throw new Error(`magick ${out}: ${r.stderr}`);
}

/** The answer cut to its file in the look's art directory. */
function prepare(job: Job): string {
  const { entry, beat, spec } = job;
  const dir = join(looks, entry.art!);
  mkdirSync(dir, { recursive: true });
  const unit = entry.unit ?? 0;
  // Source pixels per metre: four per logical pixel on a pixel grid.
  const ppm = unit ? unit * 4 : 128;
  // Without a pixel grid the opaque beats are JPEGs and the others 256-colour PNGs, to keep the files small.
  const out = join(dir, `${beat}.${(spec.cut === "full" || spec.cut === "tiles") && !unit ? "jpg" : "png"}`);
  const small = ["-dither", "None", "-colors", "256", "-strip"];
  if (spec.cut === "sprite" || spec.cut === "layer") {
    // To the subject (alpha over a half; a layer keeps the image's width), then to its height.
    const trimmed = `${job.rawFile}.trim.png`;
    magick(job.rawFile, "-channel", "A", "-threshold", "50%", "+channel", "-background", "none", "-alpha", "background", ...(spec.cut === "layer" ? ["-define", "trim:edges=north", "-trim"] : ["-trim"]), "+repage", trimmed);
    const src = pixels(trimmed);
    const at = beatSize(entry, beat, src.w, src.h);
    if (!at) throw new Error(`${job.name}: preview.json gives ${beat} no height or width`);
    if (unit) dominant(src, at.pw, at.ph, 4, out, false);
    else {
      // The tree line is far and hazed: half the playfield's density.
      const density = spec.cut === "layer" ? ppm / 2 : ppm;
      magick(trimmed, "-resize", `${Math.round(at.width * density)}x${Math.round(at.height * density)}!`, ...small, `PNG8:${out}`);
    }
  } else if (spec.cut === "tiles") {
    if (unit) dominant(pixels(job.rawFile), 4 * unit, 4 * unit, 4, out, true);
    else magick(job.rawFile, "-alpha", "off", "-resize", `${4 * ppm}x${4 * ppm}!`, "-strip", "-quality", "90", out);
  } else {
    // The far layer: the view's height, 16:9.
    const lines = unit ? Math.round(entry.view! * unit) : 720;
    const wide = unit ? Math.round((lines * 16) / 9) : 1280;
    const cover = `${job.rawFile}.cover.png`;
    magick(job.rawFile, "-alpha", "off", "-resize", `${wide * (unit ? 4 : 1)}x${lines * (unit ? 4 : 1)}^`, "-gravity", "center", "-extent", `${wide * (unit ? 4 : 1)}x${lines * (unit ? 4 : 1)}`, "-strip", ...(unit ? [] : ["-quality", "86"]), unit ? cover : out);
    if (unit) dominant(pixels(cover), wide, lines, 1, out, true);
  }
  return out;
}

async function make(job: Job): Promise<void> {
  mkdirSync(dirname(job.rawFile), { recursive: true });
  const [w, h] = CANVAS[job.spec.canvas];
  const transparent = job.spec.transparent ?? job.spec.cut === "sprite";
  const body = { model: "gpt-image-2.6-sunburst", prompt: job.prompt, size: `${w}x${h}`, background: transparent ? "transparent" : "opaque" };
  const when = new Date().toISOString().slice(0, 16).replace("T", " ");
  let row: string;
  try {
    const r = await fetch(`${gateway}/v1/images/generations`, {
      method: "POST",
      headers: { Authorization: `Bearer ${process.env.OG_PLATFORM_DEVELOPMENT_KEY}`, "Content-Type": "application/json" },
      body: JSON.stringify(body),
      signal: AbortSignal.timeout(600_000),
    });
    const text = await r.text();
    let answer: { data?: { b64_json?: string; url?: string; generation_id?: string }[]; media_url?: string } = {};
    try {
      answer = JSON.parse(text);
    } catch {}
    const first = answer.data?.[0];
    const id = first?.generation_id ?? "none";
    const url = answer.media_url ?? first?.url;
    if (first?.b64_json) writeFileSync(job.rawFile, Buffer.from(first.b64_json, "base64"));
    else if (url) writeFileSync(job.rawFile, Buffer.from(await (await fetch(url)).arrayBuffer()));
    else {
      writeFileSync(`${job.rawFile}.failed.txt`, text.slice(0, 4000));
      appendFileSync(journal, `| ${when} | generate_image | ${job.name} ${job.beat} | generation ${id} | HTTP ${r.status}, no image: not retried |\n`);
      console.log(`${job.name} ${job.beat}: HTTP ${r.status}, no image`);
      return;
    }
    const sha = new Bun.CryptoHasher("sha256").update(readFileSync(job.rawFile)).digest("hex");
    row = `| ${when} | generate_image | ${job.name} ${job.beat} | generation ${id} | ${w} x ${h}, sha256 ${sha} |\n`;
  } catch (e) {
    appendFileSync(journal, `| ${when} | generate_image | ${job.name} ${job.beat} | none | ${String(e).slice(0, 80)}: outcome unknown, not retried |\n`);
    console.log(`${job.name} ${job.beat}: ${e}`);
    return;
  }
  appendFileSync(journal, row);
  writeFileSync(`${job.rawFile}.prompt.txt`, job.prompt);
  console.log(`${job.name} ${job.beat}: ${prepare(job)}`);
}

const list = plan();
if (verb === "plan") {
  for (const j of list) console.log(`== ${j.name} ${j.beat}\n${j.prompt}\n`);
  console.log(`${list.length} to make`);
} else if (verb === "prepare") {
  for (const j of list) if (existsSync(j.rawFile)) console.log(`${j.name} ${j.beat}: ${prepare(j)}`);
} else if (verb === "make") {
  if (!process.env.OG_PLATFORM_DEVELOPMENT_KEY) throw new Error("set OG_PLATFORM_DEVELOPMENT_KEY");
  let next = 0;
  await Promise.all(
    Array.from({ length: jobs }, async () => {
      while (next < list.length) await make(list[next++]!);
    }),
  );
} else {
  console.error("art.ts plan | make | prepare (see the header)");
  process.exit(2);
}
