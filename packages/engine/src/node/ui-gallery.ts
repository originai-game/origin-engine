// The UI gallery: every built-in component and HUD element on one development page, from the
// review sheets (packages/engine/sheets/<category>), with the style pack, scheme, page and
// viewport switchable and every edit to the built-in UI shown live.
//
//   bun packages/engine/src/node/ui-gallery.ts [--port P] [--host H] [--og <og binary>]
//
// Each sheet is copied to target/ui-gallery/<sheet> and exported once with `og build` (the
// development build: every probe member) when it is first opened. The page runs the engine in
// the browser (WebGPU), so the serving machine needs no GPU.
//
// While the page is open, one long-running `og cook --watch` per open sheet watches the sheet's
// copy, the built-in documents (packages/engine/src/ui), the style packs (templates/styles), the
// design tokens (packages/design/tokens) and the default CSS (crates/og-ui/default.css), and
// cooks only what an edit changed. The changed cooked files go to the page, which applies them
// in place (`__runtime_probe__.devFiles`, no hot reload: the world, the code and the UI stay): a
// document, a pack, a token or default.css edit shows without restarting anything. A development cook carries the default style as the source
// tree has it (og-cook `ui_components::base_css`, og-ui `set_base_css`); a publish build keeps
// the compiled one. An edit to the default style's generator (crates/og-ui/scripts/theme_*.rs),
// or to a token while that is newer than og, runs gen-theme.sh's generator, so default.css never
// lags the checkout. A change to a sheet's TypeScript rebuilds it and reloads the frame. Only a
// change to the engine's Rust needs a rebuild of og and the wasm (packages/engine/scripts/build.sh).

import { copyFileSync, existsSync, mkdirSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve, sep } from "node:path";

declare const Bun: {
  spawn(cmd: string[], o: { cwd: string; stdout: "pipe"; stderr: "pipe"; env: Record<string, string | undefined> }): { exited: Promise<number>; kill(): void; stdout: ReadableStream<Uint8Array>; stderr: ReadableStream<Uint8Array> };
  file(path: string): Blob;
  serve(o: { port: number; hostname: string; idleTimeout: number; fetch(req: Request): Response | Promise<Response> }): unknown;
};

/** The file's stat, or undefined when there is none. */
function stat(path: string): ReturnType<typeof statSync> | undefined {
  try {
    return statSync(path);
  } catch {
    return undefined;
  }
}

const here = dirname(new URL(import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1"));
const pkg = resolve(here, "../..");
const repo = resolve(pkg, "../..");
const sheetsDir = join(pkg, "sheets");
const work = join(repo, "target", "ui-gallery");

function arg(name: string): string | undefined {
  const i = process.argv.indexOf(name);
  return i >= 0 ? process.argv[i + 1] : undefined;
}
const port = Number(arg("--port") ?? process.env.PORT ?? 47900);
const host = arg("--host") ?? "127.0.0.1";
const exe = process.platform === "win32" ? ".exe" : "";
const og =
  arg("--og") ??
  [join(pkg, "native", `og.${process.platform}-${process.arch}`), join(repo, "target", "debug", `og${exe}`), join(repo, "target", "release", `og${exe}`)].find((p) => existsSync(p)) ??
  "og";

/** One category: a review sheet, its pages and the styles it renders. */
interface Sheet {
  name: string;
  pages: string[];
  styles: string[];
  scheme: boolean;
}

/** The sheets' categories, read from their project.ogt `ui { screen … }` entries. */
function sheets(): Sheet[] {
  const out: Sheet[] = [];
  for (const name of readdirSync(sheetsDir).sort()) {
    const project = join(sheetsDir, name, "project.ogt");
    // `style` is a pack's core-control sheet: it loads an installed pack, not a built-in one.
    if (name === "style" || !existsSync(project)) continue;
    const pages = new Set<string>();
    const styles = new Set<string>();
    for (const m of readFileSync(project, "utf8").matchAll(/^\s*screen\s+"[^"]*"\s+"([^"]*)"/gm)) {
      let page = "";
      for (const step of m[1]!.split(";")) {
        const [k, ...v] = step.split(":");
        if (k === "style") styles.add(v.join(":"));
        if (k === "page") page = v.join(":");
        if (k === "state") page = page ? `${page}:${v.join(":")}` : `state:${v.join(":")}`;
      }
      if (page) pages.add(page);
    }
    const main = join(sheetsDir, name, "src", "main.ts");
    const code = existsSync(main) ? readFileSync(main, "utf8") : "";
    // The styles its `style` command takes (`const STYLES = [...]`), beyond those its screens name.
    for (const m of code.match(/const STYLES = \[([^\]]*)\]/)?.[1]?.matchAll(/"([\w-]+)"/g) ?? []) styles.add(m[1]!);
    out.push({ name, pages: [...pages], styles: styles.size ? [...styles] : ["default"], scheme: /\bscheme\s*:/.test(code) });
  }
  return out;
}

/** The style packs a checkout holds: each directory of templates/styles with CSS directly in it. */
function packs(): string[] {
  const dir = join(repo, "templates", "styles");
  const found = existsSync(dir)
    ? readdirSync(dir).filter((d) => d !== "tests" && statSync(join(dir, d)).isDirectory() && readdirSync(join(dir, d)).some((f) => f.endsWith(".css")))
    : [];
  return ["default", ...found.sort()];
}

/** Every file under `dir` (dot files included: the cooked manifest is `.cook.ogt`) by its size and mtime. */
function stamps(dir: string, out = new Map<string, string>(), base = dir): Map<string, string> {
  let entries: string[];
  try {
    entries = readdirSync(dir);
  } catch {
    return out;
  }
  for (const e of entries) {
    const p = join(dir, e);
    const s = stat(p);
    if (!s) continue;
    if (s.isDirectory()) stamps(p, out, base);
    else out.set(relative(base, p).split(sep).join("/"), `${s.size}:${Math.floor(s.mtimeMs)}`);
  }
  return out;
}

function changed(before: Map<string, string>, now: Map<string, string>): string[] {
  return [...now].filter(([k, v]) => before.get(k) !== v).map(([k]) => k);
}

async function runOg(args: string[]): Promise<{ ok: boolean; out: string; ms: number }> {
  const t0 = performance.now();
  const p = Bun.spawn([og, ...args], { cwd: repo, stdout: "pipe", stderr: "pipe", env: { ...process.env, OG_ENGINE_PACKAGE: pkg } });
  const [out, err] = await Promise.all([new Response(p.stdout).text(), new Response(p.stderr).text()]);
  return { ok: (await p.exited) === 0, out: out + err, ms: performance.now() - t0 };
}

function same(a: Uint8Array, b: Uint8Array): boolean {
  return a.length === b.length && a.every((x, i) => x === b[i]);
}

/** The sheet's copy: its sources, as `og ui sheet` copies them (cooked/, dist/ and .og/ stay). */
function sync(name: string): void {
  const to = join(work, name);
  mkdirSync(to, { recursive: true });
  // Only a file whose bytes differ is copied, so the incremental cook re-cooks only the edit.
  const copy = (from: string, at: string) => {
    if (existsSync(at) && same(readFileSync(from), readFileSync(at))) return;
    mkdirSync(dirname(at), { recursive: true });
    copyFileSync(from, at);
  };
  for (const d of ["assets", "src"]) {
    const [src, dst] = [join(sheetsDir, name, d), join(to, d)];
    const want = stamps(src);
    for (const f of stamps(dst).keys()) if (!want.has(f)) rmSync(join(dst, f), { force: true });
    for (const f of want.keys()) copy(join(src, f), join(dst, f));
  }
  copy(join(sheetsDir, name, "project.ogt"), join(to, "project.ogt"));
}

const built = new Map<string, Promise<string | null>>();
/** Exports the sheet once (`og build`, development): null, or why it failed. */
function build(name: string, again = false): Promise<string | null> {
  if (!again && built.has(name)) return built.get(name)!;
  const p = (async () => {
    sync(name);
    const r = await runOg(["build", join(work, name), "--out", join(work, name, "dist"), "--no-check"]);
    if (!r.ok) return r.out.trim();
    // A gallery frame plays at once and carries no badge.
    const index = join(work, name, "dist", "index.html");
    writeFileSync(index, readFileSync(index, "utf8").replace(/"gate":"tap"/, '"gate":"none"').replace(/"madeWith":true/, '"madeWith":false'));
    console.log(`built ${name} in ${Math.round(r.ms)} ms`);
    return null;
  })();
  built.set(name, p);
  return p;
}

// ── watching ────────────────────────────────────────────────────────────────
//
// The open sheet has one long-running cook (`og cook --watch --json`): it watches the copy's
// assets and the engine's built-in documents, styles, tokens and default CSS itself, and keeps
// what it parsed, so an edit cooks only what it changed. This server copies the sheet's own
// edits into the copy and tells the page which cooked files each cook changed.

interface Cooker {
  name: string;
  proc: { kill(): void; exited: Promise<number> };
  /** The cooked files as the last cook left them. */
  cooked: Map<string, string>;
}
let cooker: Cooker | null = null;
let active: string | null = null;
let seenSheet = new Map<string, string>();
const clients = new Set<ReadableStreamDefaultController<string>>();
function tell(event: unknown): void {
  for (const c of clients) c.enqueue(`data: ${JSON.stringify(event)}\n\n`);
}

function stopCooker(): void {
  cooker?.proc.kill();
  cooker = null;
}

/** The newest modification time of `paths` (absolute, or under the copy's assets/), ms since the epoch. */
function newest(name: string, paths: string[]): number {
  let at = 0;
  for (const p of paths) {
    const s = stat(p.startsWith("/") || /^[A-Za-z]:/.test(p) ? p : join(work, name, "assets", p));
    if (s) at = Math.max(at, s.mtimeMs);
  }
  return at;
}

function startCooker(name: string): void {
  stopCooker();
  const proc = Bun.spawn([og, "cook", join(work, name), "--watch", "--json"], { cwd: repo, stdout: "pipe", stderr: "pipe", env: { ...process.env, OG_ENGINE_PACKAGE: pkg } });
  const me: Cooker = { name, proc, cooked: stamps(join(work, name, "cooked")) };
  cooker = me;
  void (async () => {
    const decoder = new TextDecoder();
    let buffered = "";
    for await (const chunk of proc.stdout as unknown as AsyncIterable<Uint8Array>) {
      buffered += decoder.decode(chunk, { stream: true });
      let nl: number;
      while ((nl = buffered.indexOf("\n")) >= 0) {
        const line = buffered.slice(0, nl);
        buffered = buffered.slice(nl + 1);
        if (cooker !== me) return;
        let e: { event?: string; ok?: boolean; changed?: string[]; ms?: number; errors?: unknown[] };
        try {
          e = JSON.parse(line);
        } catch {
          continue;
        }
        if (e.event !== "cook") continue;
        const now = stamps(join(work, name, "cooked"));
        const files = changed(me.cooked, now);
        me.cooked = now;
        const edits = (e.changed ?? []).map((p) => (p.startsWith(repo) ? relative(repo, p) : p));
        // Save to cooked: from the newest edited file's time (the watch's poll and the cook).
        const saved = newest(name, e.changed ?? []);
        const serverMs = saved ? Math.max(0, Math.round(Date.now() - saved)) : null;
        if (e.ok === false) tell({ type: "error", sheet: name, edits, error: (e.errors ?? []).map(String).join("\n") });
        else tell({ type: "files", sheet: name, edits, files, cookMs: Math.round(e.ms ?? 0), serverMs });
        console.log(`${edits.join(", ")}: cooked ${files.length} file(s) in ${Math.round(e.ms ?? 0)} ms (saved to cooked ${serverMs} ms)`);
      }
    }
  })();
  void (async () => {
    const err = await new Response(proc.stderr).text();
    if (cooker === me && err.trim()) console.error(err.trim());
  })();
}

let polling = false;
/** The sheet's own edits: assets copied for the cook to see, code rebuilt. */
async function poll(): Promise<void> {
  if (polling || !active) return;
  polling = true;
  try {
    const name = active;
    const sheetNow = stamps(join(sheetsDir, name));
    const edits = changed(seenSheet, sheetNow);
    if (!edits.length) return;
    seenSheet = sheetNow;
    // Every other sheet cooks the edit when it is next opened.
    for (const other of built.keys()) if (other !== name) built.delete(other);
    if (edits.some((p) => p.startsWith("src/") || p === "project.ogt")) {
      const t0 = performance.now();
      const shown = edits.map((p) => `packages/engine/sheets/${name}/${p}`);
      tell({ type: "building", sheet: name, edits: shown });
      stopCooker();
      const why = await build(name, true);
      if (active === name) startCooker(name);
      tell(why ? { type: "error", sheet: name, edits: shown, error: why } : { type: "rebuilt", sheet: name, edits: shown, ms: Math.round(performance.now() - t0) });
      return;
    }
    sync(name);
  } finally {
    polling = false;
  }
}
setInterval(() => void poll(), 30);

// The default style's generator (crates/og-ui/scripts/theme_*.rs) is Rust compiled into og: a
// cook regenerates the default style from edited tokens only while the checkout's generator is
// the one og was built with, and otherwise reads default.css as gen-theme.sh last wrote it. So a
// generator edit, or a token edit while the generator is newer than og, runs gen-theme.sh's
// generator (theme.rs, default.css, tokens.json, as the drift check wants them; seconds, since
// cargo rebuilds og-ui for it), and the cook shows the default.css it writes. Any other token
// edit the cook regenerates itself. One run at a time; edits during one run it once more.
const themeDirs = [join(repo, "crates", "og-ui", "scripts"), join(repo, "packages", "design", "tokens")];
const themeStamps = (): Map<string, string> => {
  const out = new Map<string, string>();
  for (const d of themeDirs) for (const [k, v] of stamps(d)) out.set(join(d, k), v);
  return out;
};
let seenTheme = themeStamps();
let theming: Promise<void> | null = null;
let rethemed = false;
function regenerate(edits: string[]): void {
  if (theming) {
    rethemed = true;
    return;
  }
  theming = (async () => {
    do {
      rethemed = false;
      const t0 = performance.now();
      const args = ["packages/design/tokens", "crates/og-ui/src/theme.rs", "crates/og-ui/default.css", "packages/design/tokens.json"];
      const proc = Bun.spawn(["cargo", "run", "-q", "-p", "og-ui", "--example", "gen-theme", "--", ...args], { cwd: repo, stdout: "pipe", stderr: "pipe", env: process.env });
      const [code, err] = await Promise.all([proc.exited, new Response(proc.stderr).text()]).catch((e) => [-1, String(e)] as const);
      const ms = Math.round(performance.now() - t0);
      if (code === 0) console.log(`${edits.join(", ")}: gen-theme wrote default.css in ${ms} ms`);
      else tell({ type: "error", sheet: active, edits, error: `gen-theme failed (the default style stays as default.css has it): ${String(err).trim()}` });
    } while (rethemed);
  })().finally(() => {
    theming = null;
  });
}
setInterval(() => {
  const now = themeStamps();
  const edits = changed(seenTheme, now).filter((p) => /theme_\w+\.rs$|\.css$/.test(p));
  seenTheme = now;
  const built = stat(og)?.mtimeMs ?? Infinity;
  const stale = [...now.keys()].some((p) => /theme_\w+\.rs$/.test(p) && (stat(p)?.mtimeMs ?? 0) > built);
  if (edits.some((p) => p.endsWith(".rs")) || (edits.length && stale)) regenerate(edits.map((p) => relative(repo, p)));
}, 100);

// ── serving ─────────────────────────────────────────────────────────────────

const TYPES: Record<string, string> = { html: "text/html; charset=utf-8", js: "text/javascript", css: "text/css", wasm: "application/wasm", json: "application/json", png: "image/png", svg: "image/svg+xml", map: "application/json" };
function file(path: string): Response {
  const s = stat(path);
  if (!s?.isFile()) return new Response("not found", { status: 404 });
  const ext = path.slice(path.lastIndexOf(".") + 1);
  return new Response(Bun.file(path), { headers: { "content-type": TYPES[ext] ?? "application/octet-stream", "cache-control": "no-store" } });
}

Bun.serve({
  port,
  hostname: host,
  idleTimeout: 0,
  async fetch(req) {
    const url = new URL(req.url);
    const path = decodeURIComponent(url.pathname);
    if (path === "/" || path === "/index.html") return new Response(gallery(), { headers: { "content-type": TYPES.html!, "cache-control": "no-store" } });
    if (path === "/api/sheets") return Response.json({ sheets: sheets(), packs: packs() });
    if (path === "/api/events") {
      let me: ReadableStreamDefaultController<string>;
      return new Response(
        new ReadableStream<string>({
          start: (c) => void clients.add((me = c)),
          cancel: () => void clients.delete(me),
        }),
        { headers: { "content-type": "text/event-stream", "cache-control": "no-store" } },
      );
    }
    const open = path.match(/^\/api\/open\/([\w-]+)$/);
    if (open) {
      const name = open[1]!;
      if (!sheets().some((s) => s.name === name)) return new Response("no such sheet", { status: 404 });
      if (active !== name) {
        active = name;
        seenSheet = stamps(join(sheetsDir, name));
        stopCooker();
      }
      const why = await build(name);
      if (!why && active === name && cooker?.name !== name) startCooker(name);
      return why ? Response.json({ ok: false, error: why }, { status: 500 }) : Response.json({ ok: true });
    }
    const served = path.match(/^\/s\/([\w-]+)\/(.*)$/);
    if (served) {
      const [, name, rest] = served;
      if (rest!.split("/").includes("..")) return new Response("bad path", { status: 400 });
      // The cooked bundle is the copy's own, so an edit's re-cook is what the page fetches.
      if (rest!.startsWith("cooked/")) return file(join(work, name!, rest!));
      return file(join(work, name!, "dist", rest || "index.html"));
    }
    return new Response("not found", { status: 404 });
  },
});
console.log(`UI gallery: http://${host === "0.0.0.0" ? "localhost" : host}:${port}/ (og: ${og})`);

function gallery(): string {
  return readFileSync(join(here, "ui-gallery.html"), "utf8");
}
