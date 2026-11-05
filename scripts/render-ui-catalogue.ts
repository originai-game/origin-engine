// Art-runner entry point. No GPU on --plan. Each rendered batch takes one
// machine-wide GPU slot; never holds a slot while waiting for another batch.
// bun scripts/render-ui-catalogue.ts --plan
// bun scripts/render-ui-catalogue.ts --render [--only kit/rpg] [--out DIR]
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
const scratch = mkdtempSync(join(tmpdir(), "og-ui-catalogue-"));
type Entry = typeof entries[number];
type Cell = { screen: string; frame: string; uiFrame: string; dpr: number; scale: number; breakpoint: string };
type Element = { id: string; x: number; y: number; width: number; height: number; clipped?: boolean };

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

function crop(image: Image, element: Element, dpr: number): Image {
  const x = Math.max(0, Math.floor(element.x * dpr) - 8);
  const y = Math.max(0, Math.floor(element.y * dpr) - 8);
  const width = Math.min(image.width - x, Math.ceil((element.x + element.width) * dpr) + 8 - x);
  const height = Math.min(image.height - y, Math.ceil((element.y + element.height) * dpr) + 8 - y);
  if (width <= 0 || height <= 0) throw new Error(`Offscreen fixture ${element.id}`);
  const rgba = new Uint8Array(width * height * 4);
  for (let row = 0; row < height; row++) rgba.set(image.rgba.subarray(((y + row) * image.width + x) * 4, ((y + row) * image.width + x + width) * 4), row * width * 4);
  return { width, height, rgba };
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
    const element = frame.elements.find(el => !el.clipped && el.width > 0 && el.height > 0 && candidates.some(f => (!f.page || f.page === cell.screen) && (el.id === f.id || el.id.endsWith(`/${f.id}`))));
    if (element) { save(e, crop(image, element, cell.dpr)); return true; }
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
      const captures = join(scratch, "captures", batch);
      if (batch === "isolated") {
        const project = join(scratch, "isolated");
        cpSync(join(root, "scripts/ui-catalogue-fixtures"), project, { recursive: true, filter: p => !["cooked", ".og", "og-out"].includes(p.split("/").at(-1)!) });
        run(["ui", "preview", project, ...items.flatMap(e => ["--screen", `${e.name}=page:${e.name}`]), "--breakpoints", "desktop", "--locales", "game", "--scales", "100", "--out", captures], true);
        const all = cells(captures);
        for (const e of items) {
          const cell = all.find(c => c.screen === e.name);
          if (!cell) { missing.push(e.id); continue; }
          const image = decodePng(readFileSync(join(captures, cell.frame)));
          // Mass UI (overhead bars) draws no ordinary element nodes. Check
          // the actual image against its plain ground, not the tree alone.
          if (!hasInk(image)) { missing.push(e.id); continue; }
          save(e, image);
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
        writeFileSync(join(project, "src/main.ts"), `import {og, defineSystem} from "@origingame/engine";\nconst Ground=defineSystem("Ground",{},()=>og.ui.column("Ground",{width:{kind:"grow"},height:{kind:"grow"},background:{token:"backdrop"}},()=>{}));\nexport default og.game({systems:[Ground], start(world){const style=og.load.style(world,"@styles/scifi/style.ogt");if(!style.ok)throw new Error(style.reason);og.ui.configure(world,{style:style.value});${loads.join("\n")}});\n`);
        run(["ui", "preview", project, ...screens, "--breakpoints", "desktop", "--scales", "100", "--locales", "game", "--out", captures], true);
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
          run(["ui", "preview", project, ...pages.flatMap(p => ["--screen", `${p}=style:scifi;demo:ground:pack;show:${p}`]), "--breakpoints", "desktop", "--locales", "game", "--scales", "100", "--out", captures], true);
        } else run(["ui", "sheet", batch, "--out", captures], true);
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
