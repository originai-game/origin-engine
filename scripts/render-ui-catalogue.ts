// Art-runner entry point. No GPU on --plan. Each rendered batch takes one
// machine-wide GPU slot; never holds a slot while waiting for another batch.
// bun scripts/render-ui-catalogue.ts --plan
// bun scripts/render-ui-catalogue.ts --render [--only kit/rpg] [--out DIR] [--captures DIR]
// --captures keeps the engine's captures in DIR and reuses a batch already
// captured there, so a change to the framing reruns without the GPU.
import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { tmpdir } from "node:os";
import { decodePng, encodePng, type Image } from "../packages/engine/src/node/png.ts";
import catalogue from "../packages/knowledge/ui/catalogue.json";

const root = resolve(import.meta.dir, "..");
const arg = (flag: string): string | undefined => {
  const i = process.argv.indexOf(flag);
  return i < 0 ? undefined : process.argv[i + 1];
};
const out = resolve(arg("--out") ?? join(root, "packages/knowledge/ui"));
const og = resolve(process.env.OG_BIN ?? join(root, "target/debug/og"));
const only = arg("--only");
const entries = catalogue.entries.filter(e => !only || e.id === only);
if (!entries.length) throw new Error(`Unknown catalogue id ${only}`);
const fixtureSource = readFileSync(join(root, "scripts/ui-catalogue-fixtures/src/main.ts"), "utf8");
const isolated = new Set([...fixtureSource.matchAll(/^  (?:"([^"]+)"|(\w+)): (?:\(\)|world) =>/gm)].map(m => m[1] ?? m[2]));
const kept = arg("--captures");
const scratch = mkdtempSync(join(tmpdir(), "og-ui-catalogue-"));
const capturesRoot = kept ? resolve(kept) : join(scratch, "captures");
type Entry = typeof entries[number];
type Cell = { screen: string; frame: string; uiFrame: string; dpr: number; scale: number; breakpoint: string };
type Element = { id: string; path: string; x: number; y: number; width: number; height: number; clipped?: boolean };

function files(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap(e => e.isDirectory() ? files(join(dir, e.name)) : [join(dir, e.name)]);
}

function fixtures(e: Entry): Array<{ id: string; page: string | null }> {
  const dir = e.kit ? join(root, "packages/engine/extensions", e.kit, "sheet/src") : join(root, "templates/ui-kit/src");
  if (!existsSync(dir)) return [];
  const name = e.kit ? e.name : `og.ui.${e.name}`;
  const expression = new RegExp(`${name.replaceAll(".", "\\.")}\\(\\s*"([^"]+)"`, "g");
  return files(dir).filter(p => p.endsWith(".ts")).flatMap(p => {
    const text = readFileSync(p, "utf8");
    return [...text.matchAll(expression)].map(m => {
      const before = text.slice(0, m.index);
      const scope = [...before.matchAll(/(?:export )?function (\w+)Screen\(/g)].at(-1)?.[1];
      const branch = [...before.matchAll(/case "([^"]+)":/g)].at(-1)?.[1];
      return { id: m[1]!, page: e.kit ? null : scope ?? branch ?? null };
    });
  });
}

function run(args: string[], gpu = false): void {
  const command = gpu ? ["bash", "-c", 'set -e; source "$1/scripts/lock.sh"; og_gpu_acquire ui-catalogue; shift; "$@"', "ui-catalogue", root, og, ...args] : [og, ...args];
  const result = Bun.spawnSync(command, { cwd: root, env: { ...process.env, OG_ENGINE_PACKAGE: join(root, "packages/engine") }, stdout: "inherit", stderr: "inherit" });
  if (result.exitCode !== 0) throw new Error(`og ${args.join(" ")} exited ${result.exitCode}`);
}

function save(e: Entry, image: Image): void {
  const target = join(out, e.thumbnail);
  mkdirSync(dirname(target), { recursive: true });
  writeFileSync(target, encodePng(image));
}

function hasInk(image: Image): boolean {
  let different = 0;
  for (let i = 0; i < image.rgba.length; i += 4) {
    if ([0, 1, 2].some(c => Math.abs(image.rgba[i + c]! - image.rgba[c]!) > 8) && ++different > 16) return true;
  }
  return false;
}

// A component's thumbnail: its element and descendants, trimmed to their ink
// against the ground around them and grown to ink just past their edges (a
// label that hangs outside its box), drawn on that ground in a fixed cell at
// its own size, twice its size with nearest filtering when small, or
// box-filtered down when large; never stretched to the cell.
const THUMB = { width: 640, height: 480, pad: 24 };
type Box = { x0: number; y0: number; x1: number; y1: number };
type Rgb = [number, number, number];

function box(element: Element, dpr: number, image: Image): Box {
  return {
    x0: Math.max(0, Math.floor(element.x * dpr)),
    y0: Math.max(0, Math.floor(element.y * dpr)),
    x1: Math.min(image.width, Math.ceil((element.x + element.width) * dpr)),
    y1: Math.min(image.height, Math.ceil((element.y + element.height) * dpr)),
  };
}

function union(a: Box, b: Box): Box {
  return { x0: Math.min(a.x0, b.x0), y0: Math.min(a.y0, b.y0), x1: Math.max(a.x1, b.x1), y1: Math.max(a.y1, b.y1) };
}

/** The commonest colour in a ring just outside `b`: the ground the element sits on. */
function groundOf(image: Image, b: Box): Rgb {
  const counts = new Map<number, number>();
  const add = (x: number, y: number) => {
    if (x < 0 || y < 0 || x >= image.width || y >= image.height) return;
    const i = (y * image.width + x) * 4;
    const key = (image.rgba[i]! << 16) | (image.rgba[i + 1]! << 8) | image.rgba[i + 2]!;
    counts.set(key, (counts.get(key) ?? 0) + 1);
  };
  for (let d = 2; d <= 12; d += 2) {
    for (let x = b.x0 - d; x < b.x1 + d; x++) { add(x, b.y0 - d); add(x, b.y1 - 1 + d); }
    for (let y = b.y0 - d; y < b.y1 + d; y++) { add(b.x0 - d, y); add(b.x1 - 1 + d, y); }
  }
  const key = [...counts].sort((p, q) => q[1] - p[1])[0]?.[0] ?? 0;
  return [(key >> 16) & 255, (key >> 8) & 255, key & 255];
}

function inked(image: Image, ground: Rgb, x: number, y: number): boolean {
  const i = (y * image.width + x) * 4;
  return [0, 1, 2].some(c => Math.abs(image.rgba[i + c]! - ground[c]!) > 12);
}

function inkBox(image: Image, ground: Rgb, b: Box): Box | null {
  let n = 0;
  const ink: Box = { x0: b.x1, y0: b.y1, x1: b.x0, y1: b.y0 };
  for (let y = b.y0; y < b.y1; y++) {
    for (let x = b.x0; x < b.x1; x++) {
      if (!inked(image, ground, x, y)) continue;
      n++;
      ink.x0 = Math.min(ink.x0, x); ink.y0 = Math.min(ink.y0, y);
      ink.x1 = Math.max(ink.x1, x + 1); ink.y1 = Math.max(ink.y1, y + 1);
    }
  }
  return n > 16 ? ink : null;
}

/** Grows `b` to ink within `reach` pixels of an edge, inside `limit`, until no edge moves. */
function grow(image: Image, ground: Rgb, b: Box, limit: Box, reach: number): Box {
  const g = { ...b };
  for (let moved = true; moved;) {
    moved = false;
    const strip = (s: Box) => inkBox(image, ground, { x0: Math.max(limit.x0, s.x0), y0: Math.max(limit.y0, s.y0), x1: Math.min(limit.x1, s.x1), y1: Math.min(limit.y1, s.y1) });
    const top = strip({ x0: g.x0, y0: g.y0 - reach, x1: g.x1, y1: g.y0 });
    if (top && top.y1 > top.y0) { g.y0 = top.y0; moved = true; }
    const bottom = strip({ x0: g.x0, y0: g.y1, x1: g.x1, y1: g.y1 + reach });
    if (bottom && bottom.y1 > bottom.y0) { g.y1 = bottom.y1; moved = true; }
    const left = strip({ x0: g.x0 - reach, y0: g.y0, x1: g.x0, y1: g.y1 });
    if (left && left.x1 > left.x0) { g.x0 = left.x0; moved = true; }
    const right = strip({ x0: g.x1, y0: g.y0, x1: g.x1 + reach, y1: g.y1 });
    if (right && right.x1 > right.x0) { g.x1 = right.x1; moved = true; }
  }
  return g;
}

function compose(image: Image, b: Box, ground: Rgb): Image {
  const { width, height, pad } = THUMB;
  const rgba = new Uint8Array(width * height * 4);
  for (let i = 0; i < rgba.length; i += 4) rgba.set([...ground, 255], i);
  const w = b.x1 - b.x0, h = b.y1 - b.y0;
  const fit = Math.min((width - 2 * pad) / w, (height - 2 * pad) / h);
  const scale = fit >= 1 ? Math.min(2, Math.floor(fit)) : fit;
  const ow = Math.max(1, Math.round(w * scale)), oh = Math.max(1, Math.round(h * scale));
  const ox = Math.floor((width - ow) / 2), oy = Math.floor((height - oh) / 2);
  for (let y = 0; y < oh; y++) {
    for (let x = 0; x < ow; x++) {
      // Nearest at 1× and 2×; the mean of the source pixels it covers when smaller.
      const sx0 = b.x0 + Math.floor(x / scale), sy0 = b.y0 + Math.floor(y / scale);
      const sx1 = scale >= 1 ? sx0 + 1 : Math.max(sx0 + 1, b.x0 + Math.floor((x + 1) / scale));
      const sy1 = scale >= 1 ? sy0 + 1 : Math.max(sy0 + 1, b.y0 + Math.floor((y + 1) / scale));
      const sum = [0, 0, 0, 0];
      let n = 0;
      for (let sy = sy0; sy < Math.min(sy1, b.y1); sy++) {
        for (let sx = sx0; sx < Math.min(sx1, b.x1); sx++) {
          const i = (sy * image.width + sx) * 4;
          for (let c = 0; c < 4; c++) sum[c]! += image.rgba[i + c]!;
          n++;
        }
      }
      if (n) rgba.set(sum.map(v => Math.round(v / n)), ((oy + y) * width + ox + x) * 4);
    }
  }
  return { width, height, rgba };
}

/** A whole frame's thumbnail (an isolated fixture page): its ink against the commonest border colour. */
function frameInk(image: Image): Image | null {
  const all = { x0: 0, y0: 0, x1: image.width, y1: image.height };
  const ground = groundOf(image, { x0: 12, y0: 12, x1: image.width - 12, y1: image.height - 12 });
  const ink = inkBox(image, ground, all);
  return ink && compose(image, ink, ground);
}

function parentOf(frame: { elements: Element[] }, element: Element): Element | undefined {
  const path = element.path.slice(0, Math.max(0, element.path.lastIndexOf("/")));
  return path ? frame.elements.find(el => el.path === path) : undefined;
}

/** The thumbnail of `element` in this frame, or null when it draws nothing. */
function frameElement(image: Image, frame: { elements: Element[] }, element: Element, dpr: number): Image | null {
  const parent = parentOf(frame, element);
  const limit = parent ? box(parent, dpr, image) : { x0: 0, y0: 0, x1: image.width, y1: image.height };
  let b = box(element, dpr, image);
  for (const el of frame.elements) if (el.path.startsWith(`${element.path}/`) && el.width > 0 && el.height > 0) b = union(b, box(el, dpr, image));
  if (b.x1 <= b.x0 || b.y1 <= b.y0) return null;
  const ground = groundOf(image, b);
  const ink = inkBox(image, ground, b);
  if (!ink) return null;
  return compose(image, grow(image, ground, ink, limit, Math.round(14 * dpr)), ground);
}

function cells(dir: string): Cell[] {
  // These files are produced in this invocation by the engine's ui-run renderer.
  const value: { cells: Cell[] } = JSON.parse(readFileSync(join(dir, "index.json"), "utf8"));
  return value.cells.filter(c => c.frame && c.uiFrame && c.breakpoint === "desktop" && c.scale === 100);
}

function capture(e: Entry, dir: string): boolean {
  const candidates = fixtures(e);
  for (const cell of cells(dir)) {
    const image = decodePng(readFileSync(join(dir, cell.frame)));
    if (e.kind === "kit" || e.kind === "style") { save(e, image); return true; }
    const frame: { elements: Element[] } = JSON.parse(readFileSync(join(dir, cell.uiFrame), "utf8"));
    const matches = frame.elements.filter(el => !el.clipped && el.width > 0 && el.height > 0 && candidates.some(f => (!f.page || f.page === cell.screen) && (el.id === f.id || el.path.endsWith(`/${f.id}`))));
    for (const element of matches) {
      // An element that draws nothing of its own (a space) shows in its parent.
      const parent = parentOf(frame, element);
      const thumb = frameElement(image, frame, element, cell.dpr) ?? (parent ? frameElement(image, frame, parent, cell.dpr) : null);
      if (thumb) { save(e, thumb); return true; }
    }
  }
  return false;
}

try {
  if (!process.argv.includes("--render")) {
    process.stdout.write(JSON.stringify(entries.map(e => ({ id: e.id, thumbnail: e.thumbnail, source: e.source, fixtures: isolated.has(e.name) ? [{ id: "Sample", page: e.name }] : fixtures(e), render: isolated.has(e.name) ? `isolated fixture page:${e.name}` : e.kind === "screen" ? "document default fixture" : e.kit ? `og ui sheet ${e.kit}` : e.pack ? `og ui sheet ${e.pack}` : "ui-kit on pack ground" })), null, 2) + "\n");
  } else {
    mkdirSync(out, { recursive: true });
    const rendered: string[] = [];
    const missing: string[] = [];
    const batches = new Map<string, Entry[]>();
    for (const e of entries) {
      const batch = isolated.has(e.name) ? "isolated" : e.kind === "screen" ? "documents" : e.kit ?? e.pack ?? "components";
      batches.set(batch, [...(batches.get(batch) ?? []), e]);
    }
    for (const [batch, items] of batches) {
      const captures = join(capturesRoot, batch);
      const captured = existsSync(join(captures, "index.json"));
      if (batch === "isolated") {
        const project = join(scratch, "isolated");
        cpSync(join(root, "scripts/ui-catalogue-fixtures"), project, { recursive: true, filter: p => !["cooked", ".og", "og-out"].includes(p.split("/").at(-1)!) });
        if (!captured) run(["ui", "preview", project, ...items.flatMap(e => ["--screen", `${e.name}=page:${e.name}`]), "--breakpoints", "desktop", "--locales", "game", "--scales", "100", "--out", captures], true);
        const all = cells(captures);
        for (const e of items) {
          const cell = all.find(c => c.screen === e.name);
          if (!cell) { missing.push(e.id); continue; }
          const image = decodePng(readFileSync(join(captures, cell.frame)));
          // Mass UI (overhead bars) draws no ordinary element nodes. Check
          // the actual image against its plain ground, not the tree alone.
          const thumb = hasInk(image) ? frameInk(image) : null;
          if (!thumb) { missing.push(e.id); continue; }
          save(e, thumb);
          rendered.push(e.id);
        }
      } else if (batch === "documents") {
        const project = join(scratch, "documents");
        mkdirSync(join(project, "src"), { recursive: true });
        mkdirSync(join(project, "assets/looks"), { recursive: true });
        writeFileSync(join(project, "project.ogt"), `type project\nversion 1\n---\nname "Catalogue documents"\nid "ui-catalogue"\nengine "0.7.7"\nentry "src/main.ts"\nprofile "2d"\ntick 60\nseed 1\n`);
        writeFileSync(join(project, "assets/looks/catalogue.ogt"), 'type look\nversion 1\n---\nname "Catalogue"\nui "scifi"\n');
        const loads: string[] = [];
        const screens: string[] = [];
        for (const e of items) {
          run(["ui", "eject", e.name, project]);
          const text = readFileSync(join(root, e.source), "utf8");
          const name = /^name "([^"]+)"/m.exec(text)?.[1];
          if (!name) throw new Error(`Unnamed document ${e.source}`);
          loads.push(`{const r = og.load.document(world, ${JSON.stringify(`ui/${e.name}.ogt`)}); if (!r.ok) throw new Error(r.reason);}`);
          screens.push("--screen", `${name}=ui:push:${name}:default`);
        }
        writeFileSync(join(project, "src/main.ts"), `import {og, defineSystem} from "@origingame/engine";\nconst Ground=defineSystem("Ground",{},()=>og.ui.column("Ground",{width:{kind:"grow"},height:{kind:"grow"},background:{token:"backdrop"}},()=>{}));\nexport default og.game({systems:[Ground], start(world){const style=og.load.style(world,"@styles/scifi/style.ogt");if(!style.ok)throw new Error(style.reason);og.ui.configure(world,{style:style.value});${loads.join("\n")}}});\n`);
        if (!captured) run(["ui", "preview", project, ...screens, "--breakpoints", "desktop", "--scales", "100", "--locales", "game", "--out", captures], true);
        const capturesByName = cells(captures);
        for (const e of items) {
          const name = /^name "([^"]+)"/m.exec(readFileSync(join(root, e.source), "utf8"))?.[1];
          const cell = capturesByName.find(c => c.screen === name);
          if (!cell) { missing.push(e.id); continue; }
          save(e, decodePng(readFileSync(join(captures, cell.frame))));
          rendered.push(e.id);
        }
      } else {
        if (batch === "components") {
          // Copy the template so cooking/generation never dirties its source.
          const project = join(scratch, "components");
          cpSync(join(root, "templates/ui-kit"), project, { recursive: true, filter: p => !["node_modules", "cooked", ".og", "og-out"].includes(p.split("/").at(-1)!) });
          const pages = ["main.ts", "showcases.ts"].flatMap(file => {
            const source = readFileSync(join(project, "src", file), "utf8");
            const array = /export const (?:SCREENS|SHOWCASE_SCREENS) = \[([\s\S]*?)\] as const/.exec(source)?.[1] ?? "";
            return [...array.matchAll(/"([^"]+)"/g)].map(m => m[1]!);
          });
          if (!captured) run(["ui", "preview", project, ...pages.flatMap(p => ["--screen", `${p}=style:scifi;demo:ground:pack;show:${p}`]), "--breakpoints", "desktop", "--locales", "game", "--scales", "100", "--out", captures], true);
        } else if (!captured) run(["ui", "sheet", batch, "--out", captures], true);
        for (const e of items) (capture(e, captures) ? rendered : missing).push(e.id);
      }
    }
    writeFileSync(join(out, "render-report.json"), JSON.stringify({ rendered, missing, guidance: "Read every thumbnail. Missing fixtures are not replaced by generic cards or another component's art." }, null, 2) + "\n");
    process.stdout.write(`${rendered.length} thumbnails; ${missing.length} fixtures need a capture. See ${join(out, "render-report.json")}\n`);
    if (missing.length) process.exitCode = 1;
  }
} finally {
  rmSync(scratch, { recursive: true, force: true });
}
