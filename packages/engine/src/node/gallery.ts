// The UI gallery: every built-in component (src/ui/components, screens, controls) in every
// fixture it exports, under every style pack, generated from the components themselves, and the
// packs' asset sheets (src/node/gallery-sheets: the icon set at its sizes), which ship nothing.
//
//   bun packages/engine/src/node/gallery.ts [--out DIR] [--packs a,b] [--schemes dark,light] [--only TEXT] [--size WxH] [--no-art] [--og <og>]
//
// It writes one project (target/ui-gallery/project), cooks it once and runs it headless once per
// pack and scheme: the project mounts one fixture per page on the stage (the coast key art, or the pack backdrop with --no-art) (a screen stretched to the
// stage, as mount() gives a game's screen the surface; a piece of a panel, whose module says
// `export const ground = "panel"`, inside a panel's frame; any other component at its own size over
// the scene), the run switches the pack
// and the scheme with og.ui.configure and screenshots every page. The frames land in <out>/<pack>/
// (default target/ui-gallery/out; the light scheme's in <out>/<pack>-light/) with index.html, one
// row per fixture and one column per pack and scheme. `--schemes` is dark alone unless given. A run
// draws on the GPU og finds; without one, run it on a GPU host. `--only` keeps the fixtures
// whose `<dir>/<file> · <fixture>` contains TEXT (or any of TEXT,TEXT,…). The stage under every page is
// the coast key art in the pack's Backdrop (a game's frame stands in for a picture); `--no-art` leaves
// it and every fixture's art slot empty, so each page stands on its pack's fallback backdrop.

import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, renameSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve, sep } from "node:path";
import { pathToFileURL } from "node:url";
import { inflateSync } from "node:zlib";
import { readStringTables } from "./project.ts";

declare const Bun: {
  spawn(cmd: string[], o: { cwd: string; stdout: "pipe"; stderr: "pipe"; env: Record<string, string | undefined> }): { exited: Promise<number>; stdout: ReadableStream<Uint8Array>; stderr: ReadableStream<Uint8Array> };
};

const here = dirname(new URL(import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1"));
const pkg = resolve(here, "../..");
/** The checkout whose templates/ the gallery draws from: OG_REPO when the package runs outside it (a release's previews). */
const repo = process.env.OG_REPO ? resolve(process.env.OG_REPO) : resolve(pkg, "../..");
const ui = join(pkg, "src", "ui");
const sheets = join(here, "gallery-sheets");

/** One fixture of one component: its page in the gallery. */
export interface Page {
  /** The module, as the package exports it: `ui/components/hud/ability-bar.tsx`; an asset sheet's, as the gallery project holds it: `sheets/icons.tsx`. */
  module: string;
  /** Where it sits: `hud/ability-bar`, or `screens/title`. */
  name: string;
  component: string;
  fixture: string;
  /** Where the stage mounts it: a screen stretched to the stage, a piece of a panel (`export const ground = "panel"` in its module) inside a panel's frame, anything else at its own size over the scene. */
  place: "screen" | "panel" | "scene";
}

/** Every fixture of every built-in component, in path order. A component without fixtures has no page. */
export function pages(): Page[] {
  const out: Page[] = [];
  const walk = (dir: string): string[] =>
    readdirSync(dir, { withFileTypes: true }).flatMap((e) => (e.isDirectory() ? walk(join(dir, e.name)) : e.name.endsWith(".tsx") ? [join(dir, e.name)] : []));
  for (const area of ["components", "controls", "screens", "sheets"]) {
    const root = area === "sheets" ? sheets : join(ui, area);
    for (const file of walk(root).sort()) {
      const code = readFileSync(file, "utf8");
      const component = code.match(/^export function ([A-Z]\w*)/m)?.[1];
      const block = code.match(/^export const fixtures[^=]*=\s*\{([\s\S]*?)^\};/m)?.[1];
      if (!component || block === undefined) continue;
      const rel = area === "sheets" ? `sheets/${relative(sheets, file).split(sep).join("/")}` : relative(ui, file).split(sep).join("/");
      const name = rel.replace(/^components\//, "").replace(/\.tsx$/, "");
      const place = /^export const ground = "panel"/m.test(code) ? "panel" : area === "screens" || area === "sheets" ? "screen" : "scene";
      for (const m of block.matchAll(/^ {2}(?:"([^"]+)"|([A-Za-z_$][\w$]*)):/gm)) out.push({ module: area === "sheets" ? rel : `ui/${rel}`, name, component, fixture: m[1] ?? m[2]!, place });
    }
  }
  return out;
}

/** The engine's checked theme declarations. */
export function themes(): string[] {
  const dir = join(repo, "templates", "themes");
  return readdirSync(dir).filter((d) => existsSync(join(dir, d, "theme.ts"))).sort();
}

/** The default layer and explicit built-in packs. */
export function packs(): string[] {
  const dir = join(repo, "templates", "styles");
  return ["default", ...new Set([...themes(), ...readdirSync(dir).filter((d) => existsSync(join(dir, d, "pack.ts"))).sort()])];
}

/** Every key the built-in components' string tables hold. */
export function tableKeys(og?: string): Set<string> {
  return new Set(readStringTables(repo, true, og).flatMap(t => Object.keys(t.entries)));
}

/** The string keys a UI dump's drawn text or names show as written: the table's key, not its words. */
export function rawKeys(dump: { elements?: Array<{ text?: string; name?: string }> }, keys: Set<string>): string[] {
  const found = new Set<string>();
  for (const e of dump.elements ?? []) {
    for (const s of [e.text ?? "", e.name ?? ""]) for (const m of s.matchAll(/[a-z][\w-]*(?:\.[\w-]+)+/g)) if (keys.has(m[0])) found.add(m[0]);
  }
  return [...found].sort();
}

/** The words a UI dump's texts break across lines where UAX #14 has no break (`end|start`). */
export function brokenWords(dump: { elements?: Array<{ brokenWords?: string[] }> }): string[] {
  return [...new Set((dump.elements ?? []).flatMap((e) => e.brokenWords ?? []))];
}

const PROJECT = `import {defineProject} from '@origingame/engine/author';
export default defineProject({id:'ui-gallery',entry:'src/main.tsx',create:{form:'2d'},tick:60,seed:1});
`;

const LOOK = (pack: string) => `import { file } from '@origingame/engine/author';
// Names the pack or theme so the cook bundles it; the gallery switches packs itself.
export default file.look(${JSON.stringify({name: `gallery ${pack}`, ...(pack === "default" ? {} : {ui: `${themes().includes(pack) ? '@themes' : '@styles'}/${pack}`})})});
`;

/**
 * The art the gallery lays into a fixture (templates/gallery-art, pinned in the template store), by
 * `name · fixture`, then by prop key: each texture prop of that key still 0 takes the next image,
 * in the props' order; once the list runs out the rest keep 0 and show their fallback (an icon).
 * Every other fixture keeps its 0s.
 */
const PORTRAITS = ["portrait-warrior.png", "portrait-cultist.png", "portrait-ranger.png", "portrait-mage.png"];
const CARDS = ["card-ember-strike.png", "card-iron-guard.png"];
export const ART: Record<string, Record<string, string[]>> = {
  "arena/versus · default": { portrait: PORTRAITS },
  "arena/versus · teams": { portrait: PORTRAITS },
  "card/face · default": { art: CARDS },
  "card/hand · default": { art: CARDS },
  "card/hand · overflow": { art: CARDS },
  "screens/title · default": { keyArt: ["banner-coast.png"] },
  "screens/main-menu · default": { keyArt: ["banner-coast.png"] },
  "screens/loading · default": { keyArt: ["banner-coast.png"] },
  "screens/loading · ready": { keyArt: ["banner-coast.png"] },
  "screens/credits · default": { keyArt: ["banner-coast.png"] },
};

/**
 * The gallery's game: page:<n> mounts fixture n on the stage with its caption, pack:<name>
 * dresses everything in that pack (default: the engine's own), scheme:<light|dark> picks the
 * scheme.
 */
const MAIN = (list: Page[], noArt: boolean) => `// Generated by packages/engine/src/node/gallery.ts; do not edit.
import { og, type World } from "@origingame/engine";
import { Text, View, mount, packClass, type UiChild } from "@origingame/engine/ui";
import { Backdrop } from "@origingame/engine/ui/controls/backdrop";
${[...new Set(list.map((p) => p.module))].map((m, i) => `import { ${list.find((p) => p.module === m)!.component} as C${i}, fixtures as F${i} } from "${m.startsWith("sheets/") ? `./${m}` : `@origingame/engine/${m}`}";`).join("\n")}

type Any = (props: any) => UiChild;
/** The engine themes the cook compiled (a look names each), loaded as \`@themes/<name>\`. */
const THEMES: string[] = ${JSON.stringify(themes())};
const MODULES: [Any, Record<string, object>][] = [${[...new Set(list.map((p) => p.module))].map((_, i) => `[C${i}, F${i}]`).join(", ")}];
const PAGES: [number, string, string, "screen" | "panel" | "scene", Record<string, string[]>][] = ${JSON.stringify(list.map((p) => [[...new Set(list.map((q) => q.module))].indexOf(p.module), p.name, p.fixture, p.place, ART[`${p.name} · ${p.fixture}`] ?? {}]))};

/** \`props\` with each 0 under a key of \`art\` replaced by the next texture of that key's list. */
function withArt(world: World, props: unknown, art: Record<string, string[]>): unknown {
  if (NO_ART) return props;
  const next = Object.fromEntries(Object.entries(art).map(([k, files]) => [k, [...files]]));
  const fill = (v: unknown): unknown => {
    if (Array.isArray(v)) return v.map(fill);
    if (!v || typeof v !== "object") return v;
    return Object.fromEntries(Object.entries(v).map(([k, x]) => {
      const file = x === 0 ? next[k]?.shift() : undefined;
      if (file === undefined) return [k, fill(x)];
      const id = \`base/gallery-art/\${file.replace(/\\.png$/, "")}\`;
      const texture = og.load.texture(world, id);
      if (!texture.ok) throw new Error(\`\${id}: \${texture.reason}\`);
      return [k, texture.value];
    }));
  };
  return fill(props);
}

/** \`--no-art\`: every art slot stays empty, so each screen stands on its pack's fallback backdrop. */
const NO_ART = ${noArt};
let shown: (() => void) | null = null;
let stage = false;
function page(world: World, n: number): void {
  shown?.();
  // The stage's ground is mounted once, under every page, as a game's scene
  // stays while its screens change: a page leaving plays its exit over it.
  if (!stage) {
    stage = true;
    mount(world, () => (
      <Backdrop art={NO_ART ? 0 : (() => { const t = og.load.texture(world, "base/gallery-art/banner-coast"); if (!t.ok) throw new Error(t.reason); return t.value; })()} />
    ), {}, { layer: "hud" });
  }
  const [m, name, fixture, place, art] = PAGES[n] ?? PAGES[0]!;
  const [Component, fixtures] = MODULES[m]!;
  // The marker names the page, so the gallery lists the fixture that threw.
  try {
    shown = mount(world, () => (
      <View class="absolute inset-0">
        {/* A fixture that points at an element by its name (a tutorial step's target) gets one to point at: the game's button it would light; a fixture named \`…Missing\` gets none (a target not on screen). */}
        {(() => { const target = ({ ...fixtures.default, ...fixtures[fixture] } as { target?: unknown }).target; return typeof target === "string" && target !== "" && !fixture.endsWith("Missing") ? <View name={target} class="absolute left-1/4 top-1/5 size-24 rounded-full border-2 border-rim-strong bg-raised shadow-float" /> : null; })()}
        <View class={place === "screen" ? "absolute inset-0 p-6 grid grid-cols-1 grid-rows-1" : place === "panel" ? \`absolute inset-6 p-4 border border-rim rounded-panel bg-glass text-fg \${packClass("Panel", "frame")}\` : "absolute inset-0 p-6"}>{Component(withArt(world, { ...fixtures.default, ...fixtures[fixture] }, art))}</View>
        <Text class="absolute left-2 bottom-1 text-xs text-white/70 font-mono">{\`\${name} · \${fixture}\`}</Text>
      </View>
    ), {}, { layer: "hud" });
  } catch (e) {
    throw new Error(\`gallery page \${n}: \${e instanceof Error ? e.message : String(e)}\`);
  }
}

function pack(world: World, name: string): void {
  if (name === "default") return void og.ui.configure(world, { style: null });
  const id = THEMES.includes(name) ? \`@themes/\${name}\` : \`@styles/\${name}\`;
  const style = og.load.style(world, id);
  if (!style.ok) throw new Error(\`\${id}: \${style.reason}\`);
  og.ui.configure(world, { style: style.value });
}

export default og.game({
  tick: 60,
  seed: 1,
  // A camera that sees the origin, where the world kit's fixtures anchor (a nameplate, a loot
  // label, a speech bubble): at the default camera's own place they are behind nothing and on no
  // screen, and only an edge-clamped one draws.
  start: (world) => {
    const camera = world.resource(og.Camera);
    if (!camera.ok) throw new Error(\`og.Camera: \${camera.reason}\`);
    camera.value.position = { x: 0, y: 1, z: 8 };
  },
  // Pages mount by command only, so one that throws is a listed defect, not a failed start.
  ui: () => {},
  commands: {
    page: (world, args) => page(world, Number(args[0] ?? 0)),
    pack: (world, args) => pack(world, args[0] ?? "default"),
    scheme: (world, args) => og.ui.configure(world, { scheme: args[0] === "light" ? "light" : "dark" }),
  },
  commandHelp: {
    page: { usage: "<n>", doc: \`Mounts fixture n of \${PAGES.length} on the stage.\` },
    pack: { usage: "<name>", doc: "The style pack everything is dressed in; default is the engine's own." },
    scheme: { usage: "<light|dark>", doc: "The colour scheme." },
  },
});
`;

/** Writes the gallery project for `list` under `dir`. */
export async function generate(dir: string, list: Page[], packNames: string[], noArt = false): Promise<void> {
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(join(dir, "src"), { recursive: true });
  mkdirSync(join(dir, "content", "looks"), { recursive: true });
  const engine = JSON.parse(readFileSync(join(pkg, "package.json"), "utf8")).version;
  writeFileSync(join(dir, "og-project.json"), JSON.stringify({version:1,engine,name:"UI gallery"}) + "\n");
  writeFileSync(join(dir, "project.ts"), PROJECT);
  for (const p of packNames.filter((p) => p !== "default")) writeFileSync(join(dir, "content", "looks", `${p}.ts`), LOOK(p));
  writeFileSync(join(dir, "src", "main.tsx"), MAIN(list, noArt));
  // The asset sheets, and the names of the engine's icon set they show (toned icons first).
  cpSync(sheets, join(dir, "src", "sheets"), { recursive: true });
  const set = join(repo, "templates", "styles", "shared", "icons");
  const { icons: pairs } = await import(pathToFileURL(join(set, "game-names.ts")).href);
  const icons: string[] = pairs.map(([name]: readonly [string, string]) => name);
  const toned = icons.filter((n) => readFileSync(join(set, "game", `${n}.svg`), "utf8").includes("var(--color-"));
  writeFileSync(join(dir, "src", "sheets", "names.ts"), `// Generated by packages/engine/src/node/gallery.ts from templates/styles/shared/icons/game.ts.\nexport const TONED: string[] = ${JSON.stringify(toned)};\nexport const INK: string[] = ${JSON.stringify(icons.filter((n) => !toned.includes(n)))};\n`);
  // The fixtures' art, from the template store (bun scripts/assets.ts ensure templates).
  cpSync(join(repo, "templates", "gallery-art"), join(dir, "assets", "gallery-art"), { recursive: true });
  mkdirSync(join(dir, "content", "gallery-art"), { recursive: true });
  for (const f of readdirSync(join(dir, "assets", "gallery-art")).filter((f) => f.endsWith(".png"))) {
    writeFileSync(join(dir, "content", "gallery-art", f.replace(/\.png$/, ".ts")), `import { file } from '@origingame/engine/author';\nexport default file.import(${JSON.stringify({ kind: "texture", source: `assets/gallery-art/${f}` })});\n`);
  }
  writeFileSync(join(dir, "tsconfig.json"), JSON.stringify({ extends: "@origingame/engine/tsconfig.json", include: ["src"] }, null, 2) + "\n");
}

/** The index page: one row per fixture, one column per pack. */
/** A PNG's pixels: 8-bit RGB or RGBA, not interlaced (what `og run`'s screenshots are), as RGBA. */
export function decodePng(bytes: Uint8Array): { width: number; height: number; rgba: Uint8Array } {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  let at = 8, width = 0, height = 0, channels = 4;
  const idat: Uint8Array[] = [];
  while (at < bytes.length) {
    const len = view.getUint32(at), type = String.fromCharCode(...bytes.subarray(at + 4, at + 8));
    const data = bytes.subarray(at + 8, at + 8 + len);
    if (type === "IHDR") {
      width = view.getUint32(at + 8);
      height = view.getUint32(at + 12);
      const [depth, colour, , , interlace] = data.subarray(8, 13);
      if (depth !== 8 || (colour !== 2 && colour !== 6) || interlace !== 0) throw new Error(`PNG: depth ${depth}, colour type ${colour}, interlace ${interlace} unsupported`);
      channels = colour === 6 ? 4 : 3;
    } else if (type === "IDAT") idat.push(data);
    at += 12 + len;
  }
  const packed = new Uint8Array(idat.reduce((n, d) => n + d.length, 0));
  idat.reduce((o, d) => (packed.set(d, o), o + d.length), 0);
  const raw = inflateSync(packed);
  const stride = width * channels, rgba = new Uint8Array(width * height * 4);
  let prev = new Uint8Array(stride), line = new Uint8Array(stride);
  for (let y = 0; y < height; y++) {
    const filter = raw[y * (stride + 1)]!, src = raw.subarray(y * (stride + 1) + 1, (y + 1) * (stride + 1));
    for (let i = 0; i < stride; i++) {
      const a = i >= channels ? line[i - channels]! : 0, b = prev[i]!, c = i >= channels ? prev[i - channels]! : 0;
      const p = a + b - c, pa = Math.abs(p - a), pb = Math.abs(p - b), pc = Math.abs(p - c);
      const pred = filter === 1 ? a : filter === 2 ? b : filter === 3 ? (a + b) >> 1 : filter === 4 ? (pa <= pb && pa <= pc ? a : pb <= pc ? b : c) : 0;
      line[i] = (src[i]! + pred) & 255;
    }
    for (let x = 0; x < width; x++) {
      const o = (y * width + x) * 4;
      rgba[o] = line[x * channels]!; rgba[o + 1] = line[x * channels + 1]!; rgba[o + 2] = line[x * channels + 2]!;
      rgba[o + 3] = channels === 4 ? line[x * channels + 3]! : 255;
    }
    [prev, line] = [line, prev];
  }
  return { width, height, rgba };
}

/** WCAG 2's contrast ratio of two sRGB colours (0-255). */
export function contrast(a: readonly number[], b: readonly number[]): number {
  const lum = (c: readonly number[]) => [0.2126, 0.7152, 0.0722].reduce((s, k, i) => {
    const v = c[i]! / 255;
    return s + k * (v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4);
  }, 0);
  const [x, y] = [lum(a), lum(b)];
  return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05);
}

interface TextElement { id?: string; parent?: string; text?: string; disabled?: boolean; ink?: number[]; fontSize?: number; textLines?: number[][]; textTransform?: number[]; clip?: number[] }

/**
 * The texts of a UI dump below WCAG AA contrast (4.5:1, 3:1 over 24 px;
 * a disabled control's text is exempt) on the ground the frame shows under
 * their lines. The ground is the pixels of each line's box, padded a fifth of its height all round,
 * least like the ink (the glyphs are the rest: bold figures fill more than half the bare box), so
 * art, a translucent plate over the stage and a halo all count as drawn; the worse fifth
 * of them is what is read. A transformed text (a card fanned in a hand) is read inside each line's
 * quad, its box through `textTransform`, not the upright box round it. Only what its ancestors' overflow
 * leaves on screen is read (`clip`, the screen box a scroller or overflow cuts it to): a line cut away
 * entirely is not drawn and not read. `scale`: frame px per CSS px.
 */
export function lowContrast(dump: { elements?: TextElement[] }, frame: { width: number; height: number; rgba: Uint8Array }, scale: number): string[] {
  const els = dump.elements ?? [];
  const byId = new Map(els.map((e) => [e.id ?? "", e]));
  const disabled = (e: TextElement | undefined): boolean => !!e && (e.disabled === true || (!!e.parent && disabled(byId.get(e.parent))));
  const out: string[] = [];
  for (const e of els) {
    const ink = e.ink, lines = e.textLines;
    if (!ink || !lines?.length || ink[3]! < 0.05 || disabled(e)) continue;
    const ratios: number[] = [];
    let ground = [0, 0, 0];
    // A point on screen (CSS px) back in the lines' own space, and the screen box round a line.
    const [a, b, c, d, tx, ty] = e.textTransform ?? [1, 0, 0, 1, 0, 0];
    const det = a! * d! - b! * c!;
    const local = (x: number, y: number) => [(d! * (x - tx!) - c! * (y - ty!)) / det, (a! * (y - ty!) - b! * (x - tx!)) / det] as const;
    for (const [bx, by, bw, bh] of lines) {
      if (Math.abs(det) < 1e-6) break;
      const pad = bh! * 0.2;
      const [lx, ly, lw, lh] = [bx! - pad, by! - pad, bw! + 2 * pad, bh! + 2 * pad];
      const corners = [[lx!, ly!], [lx! + lw!, ly!], [lx!, ly! + lh!], [lx! + lw!, ly! + lh!]].map(([x, y]) => [a! * x! + c! * y! + tx!, b! * x! + d! * y! + ty!]);
      // The screen box round the line, cut to what its ancestors leave on screen; a line whose
      // glyphs lie wholly outside that cut is not drawn.
      const [cx, cy, cw, ch] = e.clip ?? [0, 0, Infinity, Infinity];
      const glyphs = [[bx!, by!], [bx! + bw!, by!], [bx!, by! + bh!], [bx! + bw!, by! + bh!]].map(([x, y]) => [a! * x! + c! * y! + tx!, b! * x! + d! * y! + ty!]);
      if (Math.max(...glyphs.map((p) => p[0]!)) <= cx! || Math.min(...glyphs.map((p) => p[0]!)) >= cx! + cw! || Math.max(...glyphs.map((p) => p[1]!)) <= cy! || Math.min(...glyphs.map((p) => p[1]!)) >= cy! + ch!) continue;
      const x0 = Math.max(0, Math.floor(Math.max(cx!, Math.min(...corners.map((p) => p[0]!))) * scale)), y0 = Math.max(0, Math.floor(Math.max(cy!, Math.min(...corners.map((p) => p[1]!))) * scale));
      const x1 = Math.min(frame.width, Math.ceil(Math.min(cx! + cw!, Math.max(...corners.map((p) => p[0]!))) * scale)), y1 = Math.min(frame.height, Math.ceil(Math.min(cy! + ch!, Math.max(...corners.map((p) => p[1]!))) * scale));
      const px: number[][] = [];
      for (let y = y0; y < y1; y++) for (let x = x0; x < x1; x++) {
        const [u, v] = local((x + 0.5) / scale, (y + 0.5) / scale);
        if (u < lx! || u > lx! + lw! || v < ly! || v > ly! + lh!) continue;
        const o = (y * frame.width + x) * 4;
        px.push([frame.rgba[o]!, frame.rgba[o + 1]!, frame.rgba[o + 2]!]);
      }
      if (px.length < 4) continue;
      const far = (p: number[]) => (p[0]! - ink[0]!) ** 2 + (p[1]! - ink[1]!) ** 2 + (p[2]! - ink[2]!) ** 2;
      px.sort((p, q) => far(q) - far(p));
      for (const g of px.slice(0, Math.ceil(px.length * 0.4))) {
        const drawn = [0, 1, 2].map((i) => ink[i]! * ink[3]! + g[i]! * (1 - ink[3]!));
        ratios.push(contrast(drawn, g));
      }
      ground = px[Math.floor(px.length * 0.2)]!;
    }
    if (!ratios.length) continue;
    ratios.sort((a, b) => a - b);
    const read = ratios[Math.floor(ratios.length * 0.2)]!;
    const bar = (e.fontSize ?? 0) > 24 ? 3 : 4.5;
    if (read < bar) {
      const hex = (c: number[]) => "#" + c.slice(0, 3).map((v) => Math.round(v).toString(16).padStart(2, "0")).join("");
      out.push(`"${(e.text ?? "").slice(0, 40)}" ${hex(ink)} on ${hex(ground)} ${read.toFixed(2)}:1 (under ${bar}:1)`);
    }
  }
  return out;
}

function index(list: Page[], packNames: string[], file: (pack: string, n: number) => string): string {
  const esc = (s: string) => s.replace(/[&<>"]/g, (c) => `&#${c.charCodeAt(0)};`);
  const rows = list.map((p, n) => `<tr><th>${esc(p.name)}<br><small>${esc(p.fixture)}</small></th>${packNames.map((k) => `<td><a href="${file(k, n)}"><img loading="lazy" src="${file(k, n)}"></a></td>`).join("")}</tr>`);
  return `<!doctype html><meta charset="utf-8"><title>UI gallery</title>
<style>body{font:13px system-ui;background:#111;color:#ddd;margin:0}table{border-collapse:collapse}th{position:sticky;left:0;background:#111;text-align:left;padding:6px;min-width:12em;vertical-align:top}thead th{top:0;z-index:1}td{padding:3px}img{width:400px;display:block}</style>
<table><thead><tr><th>${list.length} fixtures</th>${packNames.map((k) => `<th>${esc(k)}</th>`).join("")}</tr></thead><tbody>
${rows.join("\n")}
</tbody></table>`;
}

function arg(name: string): string | undefined {
  const i = process.argv.indexOf(name);
  return i >= 0 ? process.argv[i + 1] : undefined;
}

async function main(): Promise<number> {
  const exe = process.platform === "win32" ? ".exe" : "";
  const og = arg("--og") ?? [join(repo, "target", "debug", `og${exe}`), join(repo, "target", "release", `og${exe}`)].find((p) => existsSync(p)) ?? "og";
  const only = arg("--only");
  const list = pages().filter((p) => !only || only.split(",").some((t) => `${p.name} · ${p.fixture}`.includes(t)));
  const all = packs();
  const packNames = arg("--packs")?.split(",") ?? all;
  for (const p of packNames) if (!all.includes(p)) throw new Error(`no pack ${p} (${all.join(", ")})`);
  const schemes = arg("--schemes")?.split(",") ?? ["dark"];
  for (const s of schemes) if (s !== "dark" && s !== "light") throw new Error(`no scheme ${s} (dark, light)`);
  /** One run each: a pack in a scheme, and the directory and column it is (`cartoon`, `cartoon-light`). */
  const columns = packNames.flatMap((pack) => schemes.map((scheme) => ({ pack, scheme, key: scheme === "dark" ? pack : `${pack}-${scheme}` })));
  const work = join(repo, "target", "ui-gallery");
  const project = join(work, "project");
  const out = resolve(arg("--out") ?? join(work, "out"));
  rmSync(join(out, "defects.txt"), { force: true });
  rmSync(join(out, "index.html"), { force: true });
  for (const { key } of columns) rmSync(join(out, key), { recursive: true, force: true });
  await generate(project, list, all, process.argv.includes("--no-art"));
  const env = { ...process.env, OG_ENGINE_PACKAGE: pkg };
  /** Runs og, its output passed through and kept. */
  const run = async (args: string[]): Promise<{ ok: boolean; log: string }> => {
    const p = Bun.spawn([og, ...args], { cwd: project, stdout: "pipe", stderr: "pipe", env });
    const keep = async (from: ReadableStream<Uint8Array>, to: { write(s: string): unknown }) => {
      let all = "";
      const d = new TextDecoder();
      for await (const c of from as unknown as AsyncIterable<Uint8Array>) { const t = d.decode(c, { stream: true }); all += t; to.write(t); }
      return all;
    };
    const [o, e] = await Promise.all([keep(p.stdout, process.stdout), keep(p.stderr, process.stderr)]);
    return { ok: (await p.exited) === 0, log: o + e };
  };
  const keys = tableKeys(arg("--og"));
  const size = arg("--size") ?? "1280x720";
  const width = Number(size.split("x")[0]);
  /** The hard defects, by fixture: a mount that threw, a character no face draws, a raw string key drawn, a word broken across lines, text under the contrast bar on its drawn ground. */
  const defects: string[] = [];
  if (!(await run(["cook", "."])).ok) return 1;
  // A pack loads its look and fonts over a few frames; a page settles in 12 ticks. A page whose UI is
  // still `moving` then (an entrance, a transition) is shot again once it has had 2.5 s, longer than
  // the longest built-in entrance (the title screen's prompt: a 1.6 s delay, then 0.64 s in), so a
  // frame is judged at rest, never mid-pop.
  const STEP = 15, SETTLE = 12, LEAD = 30, REST = 150;
  const at = (tick: number) => (tick / 60).toFixed(4);
  for (const { pack, scheme, key } of columns) {
    const shots = join(out, key);
    mkdirSync(shots, { recursive: true });
    /** Shoots `pages` (indices into list), each `settle` ticks after it mounts: per page, its frame file, its dump, what threw and the characters no face drew. */
    const shoot = async (pages: number[], settle: number, dir: string) => {
      mkdirSync(dir, { recursive: true });
      const step = Math.max(STEP, settle + 3);
      const steps = [`${at(1)}:pack:${pack}`, `${at(2)}:scheme:${scheme}`];
      pages.forEach((n, i) => {
        const t = LEAD + i * step;
        steps.push(`${at(t)}:page:${n}`, `${at(t + settle)}:screenshot`, `${at(t + settle + 1)}:dump_ui`);
      });
      steps.push(`${at(LEAD + pages.length * step + 1)}:quit`);
      const r = await run(["run", ".", "--headless", "--no-check", "--size", size, "--out", dir, "--commands", steps.join(";")]);
      if (!r.ok) {
        // The run broke (a crash, a lost session): say where, with og's last lines.
        const last = readdirSync(dir).filter((f) => /^screenshot-\d+\.png$/.test(f)).length;
        console.error(`${key}: og run failed after ${last} of ${pages.length} pages (next: ${list[pages[last] ?? -1]?.name ?? "?"} · ${list[pages[last] ?? -1]?.fixture ?? "?"})\n${r.log.split("\n").filter((l) => !/^tick \d+ (screenshot|dump_ui):/.test(l)).slice(-40).join("\n")}`);
        return undefined;
      }
      const threw = new Map<number, string>();
      for (const m of r.log.matchAll(/gallery page (\d+): ([^\n]*)/g)) if (!threw.has(Number(m[1]))) threw.set(Number(m[1]), m[2]!);
      // A character no face draws, as the engine reports it with its tick: the page shown then draws tofu.
      const tofu = new Map<number, string>();
      for (const m of r.log.matchAll(/WARN tick (\d+): a frame drew ([^\n]*?), which font/g)) {
        const n = pages[Math.floor((Number(m[1]) - LEAD) / step)];
        if (n !== undefined && !tofu.has(n)) tofu.set(n, m[2]!);
      }
      const frames = readdirSync(dir).filter((f) => /^screenshot-\d+\.png$/.test(f)).sort();
      const dumps = readdirSync(dir).filter((f) => /^dump_ui-\d+\.json$/.test(f)).sort();
      if (frames.length !== pages.length || dumps.length !== pages.length) {
        console.error(`${key}: ${frames.length} frames and ${dumps.length} dumps for ${pages.length} fixtures`);
        return undefined;
      }
      return new Map(pages.map((n, i) => {
        const dump = JSON.parse(readFileSync(join(dir, dumps[i]!), "utf8"));
        rmSync(join(dir, dumps[i]!));
        return [n, { frame: join(dir, frames[i]!), dump, threw: threw.get(n), tofu: tofu.get(n) }] as const;
      }));
    };
    console.log(`${key}: ${list.length} fixtures`);
    const shot = await shoot(list.map((_, n) => n), SETTLE, shots);
    // 2, not 1: a run that broke is not a lane finding.
    if (!shot) return 2;
    const moving = [...shot].filter(([, s]) => s.dump.moving === true && !s.threw).map(([n]) => n);
    if (moving.length) {
      console.log(`${key}: ${moving.length} fixtures still moving at ${SETTLE} ticks, shot again at ${REST}`);
      const again = await shoot(moving, REST, join(shots, "rest"));
      if (!again) return 2;
      for (const [n, s] of again) {
        rmSync(shot.get(n)!.frame);
        shot.set(n, s);
      }
    }
    for (const [n, s] of shot) {
      const where = `${key} ${list[n]!.name} · ${list[n]!.fixture}`;
      if (s.threw) defects.push(`${where}: threw at mount: ${s.threw}`);
      if (s.tofu) defects.push(`${where}: draws no glyph for ${s.tofu}`);
      if (!s.threw) {
        const dump = s.dump;
        const raw = rawKeys(dump, keys);
        if (raw.length) defects.push(`${where}: draws raw string keys ${raw.join(", ")}`);
        const broken = brokenWords(dump);
        if (broken.length) defects.push(`${where}: breaks inside a word ${broken.join(", ")}`);
        const frame = decodePng(readFileSync(s.frame));
        for (const low of lowContrast(dump, frame, frame.width / width)) defects.push(`${where}: reads under the contrast bar: ${low}`);
      }
      renameSync(s.frame, join(shots, `${String(n).padStart(4, "0")}.png`));
    }
    rmSync(join(shots, "rest"), { recursive: true, force: true });
  }
  writeFileSync(join(out, "index.html"), index(list, columns.map((c) => c.key), (k, n) => `${k}/${String(n).padStart(4, "0")}.png`));
  console.log(`${join(out, "index.html")}: ${list.length} fixtures × ${packNames.length} packs × ${schemes.length} scheme${schemes.length === 1 ? "" : "s"}`);
  writeFileSync(join(out, "defects.txt"), defects.join("\n") + (defects.length ? "\n" : ""));
  if (defects.length) {
    console.error(`${defects.length} defect(s) (${join(out, "defects.txt")}):\n${defects.join("\n")}`);
    return 1;
  }
  return 0;
}

if (import.meta.main) process.exit(await main());
