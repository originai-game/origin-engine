// `og dev --web`: a game project's development web build, served with every edit shown live in
// the open page (the web half of `og dev`; the UI gallery's live edits, for any project).
//
//   bun src/node/dev-web.ts --project <dir> --entry <src/main.ts> [--port P] [--host H]
//                           [--og <og binary>] [--settings <json>]
//
// og-cli's `og dev --web` runs it. The page is a development build (`webBuild`, `live`): the
// inspector (F1, gizmos), Tune, Pause and Step and every probe member, the engine running in the
// viewer's browser (WebGPU), so the serving machine needs no GPU. One long-running
// `og cook --watch` cooks only what each asset or UI edit changed; this server tells the page
// which cooked files changed and the page applies them in place (`devFiles`: the world, the code
// and the UI kept). A change under src/ bundles the game's own code alone (build.ts `web-code`,
// tens of milliseconds) and the page swaps it in (`devCode`: snapshot, swap, restore). Only a
// cold restart (project.ogt changed, or the first build) reloads the page.
//
// Every URL the page uses is relative, so it may be served under a prefix (a proxy's ticket
// path). The server's own routes are under `_dev/` (docs/web-export.md §og dev --web):
//   GET  _dev/status   {ready, build, error, last?: {kind, ms, at, steps}}
//   GET  _dev/files?f=cooked/<path>&f=…  a change's cooked files in one framed response
//   WS   _dev/socket   text JSON, each message under 1 MiB (the page fetches file bytes over HTTP):
//     server→page  {type:"hello", build, seq}             on connect; a new build: the page reloads
//                  {type:"files", seq, files:[{path, hash}], edits, steps}
//                  {type:"code", seq, url, hash, edits, steps}
//                  {type:"reload", build}                 a cold restart finished
//                  {type:"error", error, edits}           a cook or bundle failed (null: cleared)
//     page→server  {type:"hello", build, seq}             the build and the last change it applied
//                  {type:"applied", seq, kind, ok, ms, steps, error?}
//   GET  cooked/<path> the project's live cooked bundle; any other path: the build's files.
// Every response is no-store; the build has no service worker.

import { existsSync, mkdirSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { dirname, join, relative, resolve, sep } from "node:path";
import { bundleGame } from "./build.ts";
import { DEFAULT_PUBLISH, settingsOf, webBuild, type ProjectSettings, type PublishSettings } from "./web-build.ts";

interface Socket {
  send(text: string): void;
  data: { seq: number };
}
declare const Bun: {
  spawn(cmd: string[], o: { cwd: string; stdout: "pipe"; stderr: "pipe"; env: Record<string, string | undefined> }): { exited: Promise<number>; kill(): void; stdout: ReadableStream<Uint8Array>; stderr: ReadableStream<Uint8Array> };
  file(path: string): Blob;
  serve(o: {
    port: number;
    hostname: string;
    idleTimeout: number;
    fetch(req: Request, server: { upgrade(req: Request, o: { data: { seq: number } }): boolean }): Response | Promise<Response> | undefined;
    websocket: { open(ws: Socket): void; message(ws: Socket, text: string | Uint8Array): void; close(ws: Socket): void };
  }): { port: number };
};

const here = dirname(new URL(import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1"));
function arg(name: string): string | undefined {
  const i = process.argv.indexOf(name);
  return i >= 0 ? process.argv[i + 1] : undefined;
}
const project = resolve(arg("--project") ?? ".");
const entry = resolve(project, arg("--entry") ?? "src/main.ts");
const port = Number(arg("--port") ?? process.env.PORT ?? 47810);
const host = arg("--host") ?? "127.0.0.1";
const og = arg("--og") ?? "og";
const settings: ProjectSettings = (() => {
  const given = arg("--settings");
  if (!given) return settingsOf(project);
  const s = JSON.parse(given) as Omit<ProjectSettings, "publish"> & { publish?: Partial<PublishSettings> };
  return { ...s, publish: { ...DEFAULT_PUBLISH, ...s.publish } };
})();
const out = join(project, "og-out", "dev-web");
const codeDir = join(out, "_dev");
const cookedDir = join(project, "cooked");

function stat(path: string): ReturnType<typeof statSync> | undefined {
  try {
    return statSync(path);
  } catch {
    return undefined;
  }
}
/** Every file under `dir` (dot files included: the cooked manifest is `.cook.ogt`) by its size and mtime. */
function stamps(dir: string, into = new Map<string, string>(), base = dir): Map<string, string> {
  let entries: string[];
  try {
    entries = readdirSync(dir);
  } catch {
    return into;
  }
  for (const e of entries) {
    const p = join(dir, e);
    const s = stat(p);
    if (!s) continue;
    if (s.isDirectory()) stamps(p, into, base);
    else into.set(relative(base, p).split(sep).join("/"), `${s.size}:${Math.floor(s.mtimeMs)}`);
  }
  return into;
}
const changed = (before: Map<string, string>, now: Map<string, string>) => [...now].filter(([k, v]) => before.get(k) !== v).map(([k]) => k);
const hashOf = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex").slice(0, 16);
const round = (ms: number) => Math.round(ms * 10) / 10;

// ── state ───────────────────────────────────────────────────────────────────

/** The build the page boots: it changes only on a cold restart (the page then reloads). */
let build = "";
let ready = false;
/** The current cook or bundle error, null when the last of each succeeded. */
const errors: { cook: string | null; code: string | null; check: string | null; build: string | null } = { cook: null, code: null, check: null, build: null };
const errorText = () => [errors.build, errors.cook, errors.code, errors.check].filter(Boolean).join("\n") || null;
let last: { kind: "files" | "code" | "reload"; ms: number; at: number; steps?: unknown } | null = null;

/** Every change since the build, numbered: a reconnecting page catches up from its last. */
let seq = 0;
/** Each cooked file's last change (its number and hash). */
const fileSeq = new Map<string, { seq: number; hash: string }>();
/** The latest code bundle. */
let code: { seq: number; url: string; hash: string } | null = null;
const sockets = new Set<Socket>();
function send(ws: Socket, message: unknown): void {
  ws.send(JSON.stringify(message));
}
function tell(message: unknown): void {
  const text = JSON.stringify(message);
  for (const ws of sockets) ws.send(text);
}
function tellError(edits: string[]): void {
  tell({ type: "error", error: errorText(), edits });
}

// ── the cold build ──────────────────────────────────────────────────────────

async function runOg(args: string[]): Promise<{ ok: boolean; out: string }> {
  const p = Bun.spawn([og, ...args], { cwd: project, stdout: "pipe", stderr: "pipe", env: process.env });
  const [o, e] = await Promise.all([new Response(p.stdout).text(), new Response(p.stderr).text()]);
  return { ok: (await p.exited) === 0, out: (o + e).trim() };
}

let codeCount = 0;
/** Cook, export the development build and start watching: the first build and every cold restart. */
async function coldBuild(): Promise<void> {
  const t0 = performance.now();
  ready = false;
  stopCooker();
  const cooked = await runOg(["cook", project]);
  if (!cooked.ok) {
    errors.build = `og cook: ${cooked.out}`;
    console.error(errors.build);
    tellError(["project.ogt"]);
    // The cook watches anyway: the next edit may fix it.
    startCooker();
    return;
  }
  const b = await webBuild(project, entry, out, false, settings, true);
  if (!b.ok) {
    errors.build = `og build: ${b.reason}`;
    console.error(errors.build);
    tellError(["project.ogt"]);
    startCooker();
    return;
  }
  errors.build = errors.cook = errors.code = errors.check = null;
  mkdirSync(codeDir, { recursive: true });
  // The page's live client and the gate: a development page plays at once.
  build = `${Date.now().toString(36)}`;
  const index = join(out, "index.html");
  writeFileSync(index, readFileSync(index, "utf8").replace(/"gate":"tap"/, '"gate":"none"').replace("</body>", `<meta name="og-dev-build" content="${build}">\n<script type="module" src="./_dev/client.js"></script>\n</body>`));
  writeFileSync(join(codeDir, "client.js"), readFileSync(join(here, "dev-web-client.js"), "utf8"));
  seq = 0;
  fileSeq.clear();
  code = null;
  codeCount = 0;
  seenSrc = stamps(join(project, "src"));
  seenProject = stat(join(project, "project.ogt"))?.mtimeMs ?? 0;
  startCooker();
  ready = true;
  const ms = Math.round(performance.now() - t0);
  last = { kind: "reload", ms, at: Date.now() };
  console.log(`og dev --web: built ${relative(process.cwd(), out) || out} in ${ms} ms (build ${build})`);
  tell({ type: "reload", build });
}

// ── assets: one long-running cook ───────────────────────────────────────────

let cooker: { proc: { kill(): void }; cooked: Map<string, string> } | null = null;
function stopCooker(): void {
  cooker?.proc.kill();
  cooker = null;
}
function startCooker(): void {
  stopCooker();
  const proc = Bun.spawn([og, "cook", project, "--watch", "--json"], { cwd: project, stdout: "pipe", stderr: "pipe", env: process.env });
  const me = { proc, cooked: stamps(cookedDir) };
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
        let e: { event?: string; ok?: boolean; changed?: string[]; ms?: number; errors?: unknown[]; detectMs?: number; walkMs?: number; reportMs?: number };
        try {
          e = JSON.parse(line);
        } catch {
          continue;
        }
        if (e.event === "watching") void warmPacks((e as { styles?: string | null }).styles ?? null);
        if (e.event !== "cook") continue;
        const edits = (e.changed ?? []).map((p) => (p.startsWith(project) ? relative(project, p).split(sep).join("/") : /^([A-Za-z]:)?[\\/]/.test(p) ? p : `assets/${p}`));
        // Save to cooked: from the newest edited file's time.
        // (The cook names an asset relative to assets/, any other file by its absolute path.)
        const saved = Math.max(0, ...(e.changed ?? []).map((p) => stat(resolve(project, "assets", p))?.mtimeMs ?? 0));
        if (e.ok === false) {
          errors.cook = (e.errors ?? []).map(String).join("\n") || "og cook failed";
          console.error(`${edits.join(", ")}: ${errors.cook}`);
          tellError(edits);
          continue;
        }
        const st = performance.now();
        const now = stamps(cookedDir);
        const files = changed(me.cooked, now).filter((f) => !/^\.(cook-times|conform|assets)\.ogt$/.test(f));
        me.cooked = now;
        const stampMs = round(performance.now() - st);
        const hadError = errors.cook !== null;
        errors.cook = null;
        if (hadError) tellError(edits);
        if (!files.length) continue;
        seq++;
        const listed = files.map((f) => {
          const hash = hashOf(new Uint8Array(readFileSync(join(cookedDir, f))));
          fileSeq.set(f, { seq, hash });
          return { path: `cooked/${f}`, hash };
        });
        const steps = { saveToCookedMs: saved ? Math.max(0, Date.now() - saved) : null, detectMs: e.detectMs, walkMs: e.walkMs, cookMs: round(e.ms ?? 0), reportMs: e.reportMs, stampMs, savedAt: saved || null };
        tell({ type: "files", seq, files: listed, edits, steps });
        console.log(`${edits.join(", ")}: cooked ${files.length} file(s) in ${steps.cookMs} ms (save to cooked ${steps.saveToCookedMs} ms)`);
      }
    }
  })();
  void (async () => {
    const err = await new Response(proc.stderr).text();
    if (cooker === me && err.trim()) console.error(err.trim());
  })();
}

// ── warming: the other style packs, cooked once into the shared cook cache ──
//
// A look's `ui "<pack>"` edit cooks the new pack: its CSS, art and fonts, a few seconds the first
// time on a machine (a font's atlas is about a second). After the cold build, each built-in pack
// the project does not use is cooked once in the background, at the lowest priority, in a scratch
// project with the project's own project.ogt and look (so the cook's options, and with them the
// shared cache's keys, are the project's) and its `ui` line naming that pack. A later switch to it
// finds its outputs in the cache (og-cook's own-input actions) and cooks in a fraction of a second.

let warming: Promise<void> | null = null;
function warmPacks(stylesDir: string | null): Promise<void> {
  if (!stylesDir || warming) return warming ?? Promise.resolve();
  warming = (async () => {
    const projectText = readFileSync(join(project, "project.ogt"), "utf8");
    const lookRel = /^\s*look\s+"([^"]+)"/m.exec(projectText)?.[1];
    if (!lookRel || lookRel.startsWith("@")) return;
    const lookFile = join(project, "assets", lookRel);
    const lookText = stat(lookFile)?.isFile() ? readFileSync(lookFile, "utf8") : "";
    const current = /^\s*ui\s+"([^"]+)"/m.exec(lookText)?.[1] ?? "default";
    const packs = readdirSync(stylesDir)
      .filter((d) => d !== "shared" && d !== current && stat(join(stylesDir, d))?.isDirectory() && readdirSync(join(stylesDir, d)).some((f) => f.endsWith(".css")))
      .sort();
    const entryRel = relative(project, entry);
    for (const pack of packs) {
      const dir = join(project, "og-out", "dev-web-warm", pack);
      rmSync(dir, { recursive: true, force: true });
      mkdirSync(dirname(join(dir, "assets", lookRel)), { recursive: true });
      mkdirSync(dirname(join(dir, entryRel)), { recursive: true });
      writeFileSync(join(dir, "project.ogt"), projectText);
      const ui = /^\s*ui\s+"[^"]*"/m.test(lookText) ? lookText.replace(/^(\s*)ui\s+"[^"]*"/m, `$1ui "${pack}"`) : `${lookText.trimEnd()}\nui "${pack}"\n`;
      writeFileSync(join(dir, "assets", lookRel), ui);
      writeFileSync(join(dir, entryRel), "export default null;\n");
      const t0 = performance.now();
      const cmd = [og, "cook", dir, "--threads", "2"];
      const p = Bun.spawn(process.platform === "win32" ? cmd : ["nice", "-n", "19", ...cmd], { cwd: project, stdout: "pipe", stderr: "pipe", env: process.env });
      await Promise.all([new Response(p.stdout).text(), new Response(p.stderr).text(), p.exited]);
      console.log(`warmed the cook cache for ui "${pack}" in ${Math.round(performance.now() - t0)} ms`);
      rmSync(dir, { recursive: true, force: true });
    }
  })().catch((e) => console.error(`warming the style packs: ${e}`));
  return warming;
}

// ── code: the game's own bundle on each src/ edit ───────────────────────────

/** The type check of the latest src/ change, beside its bundle (it never holds a swap back); a newer one supersedes it. */
let checking = 0;
async function check(edits: string[]): Promise<void> {
  const n = ++checking;
  const r = await runOg(["check", project]);
  if (n !== checking) return;
  const was = errors.check;
  errors.check = r.ok ? null : `og check: ${r.out.slice(-8000)}`;
  if (errors.check !== was) tellError(edits);
  console.log(`${edits.join(", ")}: ${r.ok ? "check ok" : errors.check}`);
}

let seenSrc = new Map<string, string>();
let seenProject = 0;
let bundling = false;
let again = false;
async function bundleCode(edits: string[]): Promise<void> {
  if (bundling) {
    again = true;
    return;
  }
  bundling = true;
  try {
    do {
      again = false;
      const saved = Math.max(0, ...edits.map((p) => stat(join(project, p))?.mtimeMs ?? 0));
      const n = ++codeCount;
      const file = join(codeDir, `code-${n}.js`);
      const r = await bundleGame(entry, file, "web-code", false, { root: project });
      if (!r.ok) {
        errors.code = `bundle: ${r.reason}`;
        console.error(`${edits.join(", ")}: ${errors.code}`);
        tellError(edits);
        continue;
      }
      // The older bundles' files go (the page has imported them, or never will).
      for (const f of readdirSync(codeDir)) if (/^code-\d+\.js(\.map)?$/.test(f) && f !== `code-${n}.js` && f !== `code-${n}.js.map`) rmSync(join(codeDir, f), { force: true });
      const hadError = errors.code !== null;
      errors.code = null;
      if (hadError) tellError(edits);
      seq++;
      const hash = hashOf(new Uint8Array(readFileSync(file)));
      code = { seq, url: `_dev/code-${n}.js`, hash };
      const steps = { saveToBundledMs: saved ? Math.max(0, Date.now() - saved) : null, bundleMs: round(r.ms), bytes: r.bytes, writtenAt: Date.now() };
      tell({ type: "code", ...code, edits, steps });
      console.log(`${edits.join(", ")}: bundled the game's code (${r.bytes} B) in ${steps.bundleMs} ms`);
    } while (again);
  } finally {
    bundling = false;
  }
}

setInterval(() => {
  if (!build) return;
  const pm = stat(join(project, "project.ogt"))?.mtimeMs ?? 0;
  if (pm !== seenProject) {
    seenProject = pm;
    console.log("project.ogt changed: a cold restart");
    void coldBuild();
    return;
  }
  const now = stamps(join(project, "src"));
  const edits = [...changed(seenSrc, now), ...[...seenSrc.keys()].filter((k) => !now.has(k))];
  if (!edits.length) return;
  seenSrc = now;
  const shown = edits.map((p) => `src/${p}`);
  void bundleCode(shown);
  void check(shown);
}, 30);

// ── serving ─────────────────────────────────────────────────────────────────

const TYPES: Record<string, string> = { html: "text/html; charset=utf-8", js: "text/javascript", css: "text/css", wasm: "application/wasm", json: "application/json", png: "image/png", svg: "image/svg+xml", map: "application/json", webmanifest: "application/manifest+json" };
const NO_STORE = { "cache-control": "no-store" };
function file(path: string): Response {
  const s = stat(path);
  if (!s?.isFile()) return new Response("not found", { status: 404, headers: NO_STORE });
  const ext = path.slice(path.lastIndexOf(".") + 1);
  return new Response(Bun.file(path), { headers: { "content-type": TYPES[ext] ?? "application/octet-stream", ...NO_STORE } });
}

/** A (re)connected page at `build` that applied changes up to `since`: what it missed, or a reload. */
function resync(ws: Socket, pageBuild: string, since: number): void {
  if (!ready) return;
  if (pageBuild !== build) return send(ws, { type: "reload", build });
  const files = [...fileSeq].filter(([, v]) => v.seq > since).map(([f, v]) => ({ path: `cooked/${f}`, hash: v.hash }));
  if (files.length) send(ws, { type: "files", seq, files, edits: [], steps: { resync: true } });
  if (code && code.seq > since) send(ws, { type: "code", ...code, edits: [], steps: { resync: true } });
  if (errorText()) send(ws, { type: "error", error: errorText(), edits: [] });
}

const server = Bun.serve({
  port,
  hostname: host,
  idleTimeout: 0,
  fetch(req, srv) {
    const url = new URL(req.url);
    // Served under any prefix: the route is the path's tail after the last `_dev/` or the page's own files.
    const path = decodeURIComponent(url.pathname);
    const dev = path.match(/(?:^|\/)_dev\/(status|socket)$/);
    if (dev?.[1] === "socket") {
      if (srv.upgrade(req, { data: { seq: 0 } })) return undefined;
      return new Response("expected a WebSocket", { status: 426, headers: NO_STORE });
    }
    // A change's cooked files in one response (the relay carries a few requests at a time, so
    // a pack's dozens of files fetched one by one took several round trips): `?f=cooked/<path>`
    // repeated; each file as a u32 LE path length, the path (UTF-8), a u32 LE byte length and
    // the bytes. A missing file has the byte length 0xffffffff and no bytes.
    if (/(?:^|\/)_dev\/files$/.test(path)) {
      const enc = new TextEncoder();
      const parts: Uint8Array<ArrayBuffer>[] = [];
      for (const f of url.searchParams.getAll("f")) {
        if (!f.startsWith("cooked/") || f.split("/").includes("..")) return new Response("bad path", { status: 400, headers: NO_STORE });
        const name = enc.encode(f) as Uint8Array<ArrayBuffer>;
        const at = join(cookedDir, f.slice("cooked/".length));
        const bytes = stat(at)?.isFile() ? new Uint8Array(readFileSync(at)) as Uint8Array<ArrayBuffer> : null;
        const head = new DataView(new ArrayBuffer(4));
        head.setUint32(0, name.length, true);
        const len = new DataView(new ArrayBuffer(4));
        len.setUint32(0, bytes ? bytes.length : 0xffffffff, true);
        parts.push(new Uint8Array(head.buffer), name, new Uint8Array(len.buffer), ...(bytes ? [bytes] : []));
      }
      return new Response(new Blob(parts), { headers: { "content-type": "application/octet-stream", ...NO_STORE } });
    }
    if (dev?.[1] === "status") return Response.json({ ready, build, error: errorText(), ...(last ? { last } : {}) }, { headers: NO_STORE });
    if (!ready && !existsSync(join(out, "index.html"))) return new Response("og dev --web: building…", { status: 503, headers: { ...NO_STORE, "retry-after": "1" } });
    const rest = path.replace(/^\/+/, "");
    if (rest.split("/").includes("..")) return new Response("bad path", { status: 400, headers: NO_STORE });
    // The live cooked bundle: an edit's re-cook is what the page fetches.
    const cookedAt = rest.indexOf("cooked/");
    if (cookedAt >= 0 && (cookedAt === 0 || rest[cookedAt - 1] === "/")) return file(join(cookedDir, rest.slice(cookedAt + "cooked/".length)));
    // Any prefix (a proxy's ticket path) maps to the build's root: the longest tail that names a file.
    const parts = rest.split("/");
    for (let i = 0; i < parts.length; i++) {
      const tail = parts.slice(i).join("/");
      const at = join(out, tail || "index.html");
      if (stat(at)?.isFile()) return file(at);
    }
    if (rest === "" || rest.endsWith("/")) return file(join(out, "index.html"));
    return new Response("not found", { status: 404, headers: NO_STORE });
  },
  websocket: {
    open(ws) {
      sockets.add(ws);
      send(ws, { type: "hello", build, seq, ready, error: errorText() });
    },
    message(ws, text) {
      let m: { type?: string; build?: string; seq?: number; kind?: "files" | "code"; ok?: boolean; ms?: number; steps?: unknown; error?: string };
      try {
        m = JSON.parse(String(text));
      } catch {
        return;
      }
      if (m.type === "hello") resync(ws, String(m.build ?? ""), Number(m.seq ?? 0));
      if (m.type === "applied" && (m.kind === "files" || m.kind === "code")) {
        last = { kind: m.kind, ms: Number(m.ms ?? 0), at: Date.now(), steps: m.steps };
        console.log(`applied ${m.kind} #${m.seq}: ${m.ok ? "ok" : `refused (${m.error})`} in ${m.ms} ms save to frame ${JSON.stringify(m.steps)}`);
      }
    },
    close(ws) {
      sockets.delete(ws);
    },
  },
});

await coldBuild();
// One JSON line when ready (a supervisor may wait for it instead of polling _dev/status).
process.stdout.write(JSON.stringify({ event: "ready", url: `http://${host === "0.0.0.0" ? "127.0.0.1" : host}:${server.port}/`, build, error: errorText() }) + "\n");
