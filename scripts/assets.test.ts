import { afterAll, expect, test } from "bun:test";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const script = join(import.meta.dir, "assets.ts");
const KEY = "ogp-writer";
const sha = (bytes: Uint8Array | string) => createHash("sha256").update(bytes).digest("hex");

// The engine asset store as Portal answers it (origingame server/src/routes/engine-assets.ts):
// private objects read as absent without the writer key, a put is refused unless its bytes are its address.
const stored = new Map<string, { bytes: Uint8Array; visibility: "public" | "private" }>();
const store = { down: false, corrupt: false, gets: 0 };
const server = Bun.serve({ port: 0, hostname: "127.0.0.1", async fetch(request) {
  if (store.down) return new Response(null, { status: 503 });
  const writer = request.headers.get("authorization") === `Bearer ${KEY}`;
  const url = new URL(request.url);
  const hex = /^\/api\/engine-assets\/sha256\/([a-f0-9]{64})$/.exec(url.pathname)?.[1];
  if (request.method === "POST" && url.pathname === "/api/engine-assets/verify") {
    const { objects } = (await request.json()) as { objects: { sha256: string }[] };
    return Response.json({ objects: objects.map(({ sha256 }) => {
      const o = stored.get(sha256);
      return o && (o.visibility === "public" || writer) ? { sha256, present: true, size: o.bytes.length, visibility: o.visibility } : { sha256, present: false };
    }) });
  }
  if (request.method === "GET" && hex) {
    const o = stored.get(hex);
    if (!o || (o.visibility === "private" && !writer)) return new Response(null, { status: 404 });
    store.gets++;
    return new Response(store.corrupt ? new Uint8Array(o.bytes.length) : o.bytes);
  }
  if (request.method === "PUT" && hex) {
    if (!writer) return new Response(null, { status: 403 });
    const bytes = new Uint8Array(await request.arrayBuffer());
    if (sha(bytes) !== hex) return Response.json({ error: { code: "sha256_mismatch" } }, { status: 422 });
    const visibility = url.searchParams.get("visibility") as "public" | "private";
    const old = stored.get(hex);
    stored.set(hex, { bytes, visibility: old?.visibility === "public" ? "public" : visibility });
    return Response.json({}, { status: old ? 200 : 201 });
  }
  return new Response(null, { status: 404 });
} });
const root = mkdtempSync(join(tmpdir(), "og-assets-"));
afterAll(() => { server.stop(true); rmSync(root, { recursive: true, force: true }); });

let caches = 0;
const freshCache = () => join(root, `cache-${++caches}`);
async function run(args: string[], options: { cache: string; key?: boolean; mirror?: string }) {
  const env: Record<string, string | undefined> = { ...process.env, OG_ASSETS_URL: `http://127.0.0.1:${server.port}`, ORIGIN_ENGINE_CACHE: options.cache };
  delete env.OG_PLATFORM_DEVELOPMENT_KEY;
  delete env.OG_ASSETS_MIRROR;
  if (options.key) env.OG_ASSETS_KEY = KEY; else delete env.OG_ASSETS_KEY;
  if (options.mirror) env.OG_ASSETS_MIRROR = options.mirror;
  const child = Bun.spawn(["bun", script, ...args], { env, stdout: "ignore", stderr: "pipe" });
  return { code: await child.exited, error: await new Response(child.stderr).text() };
}

// A project with a public model and a private generated source, and a .gitignore of its own.
const project = join(root, "showcase", "boatyard");
const files = { "assets/models/boat.glb": "glTF boat bytes", "tools/raw/boat-source.bin": "the raw generated boat" };
const local = (path: string) => join(project, path);
function place() {
  for (const [path, text] of Object.entries(files)) {
    mkdirSync(join(local(path), ".."), { recursive: true });
    writeFileSync(local(path), text);
  }
}
const remove = () => { for (const path of Object.keys(files)) rmSync(local(path), { force: true }); };
const installed = () => Object.fromEntries(Object.keys(files).map((p) => [p, existsSync(local(p)) ? readFileSync(local(p), "utf8") : null]));
mkdirSync(project, { recursive: true });
writeFileSync(join(project, ".gitignore"), "cooked/\n");
place();

test("pinned files push to the store, then install from it, from the cache offline and from a mirror", async () => {
  const cache = freshCache();
  expect((await run(["pin", project, "assets/models/boat.glb"], { cache })).code).toBe(0);
  expect((await run(["pin", project, "--private", "tools/raw/boat-source.bin"], { cache })).code).toBe(0);
  expect(JSON.parse(readFileSync(join(project, "og-assets.json"), "utf8"))).toEqual({ version: 1, files: [
    { path: "assets/models/boat.glb", sha256: sha(files["assets/models/boat.glb"]), size: 15, visibility: "public" },
    { path: "tools/raw/boat-source.bin", sha256: sha(files["tools/raw/boat-source.bin"]), size: 22, visibility: "private" },
  ] });
  // The pinned paths are ignored by Git; the project's own rules stay.
  expect(readFileSync(join(project, ".gitignore"), "utf8")).toMatch(/^cooked\/\n\n# og-assets: .*\n\/assets\/models\/boat\.glb\n\/tools\/raw\/boat-source\.bin\n# og-assets end\n$/);

  expect((await run(["push", project], { cache: freshCache() })).code).toBe(1); // no key
  expect(await run(["push", project], { cache: freshCache(), key: true })).toEqual({ code: 0, error: "" });
  expect([...stored.entries()].map(([h, o]) => [h, o.visibility]).sort()).toEqual(
    [[sha(files["assets/models/boat.glb"]), "public"], [sha(files["tools/raw/boat-source.bin"]), "private"]].sort());
  expect(await run(["verify", project], { cache: freshCache(), key: true })).toEqual({ code: 0, error: "" });

  // A fresh machine: both come from the store, each fetched once into the cache.
  const machine = freshCache();
  remove();
  store.gets = 0;
  expect(await run(["ensure", join(root, "showcase")], { cache: machine, key: true })).toEqual({ code: 0, error: "" });
  expect(installed()).toEqual(files);
  expect(store.gets).toBe(2);
  expect(await run(["status", project], { cache: machine })).toEqual({ code: 0, error: "" });

  // Offline, the same machine installs from its cache alone.
  store.down = true;
  remove();
  expect(await run(["ensure", project], { cache: machine })).toEqual({ code: 0, error: "" });
  expect(installed()).toEqual(files);

  // Offline with no cache, a mirror laid out by address serves it.
  const mirror = join(root, "mirror");
  for (const text of Object.values(files)) {
    mkdirSync(join(mirror, "sha256", sha(text).slice(0, 2)), { recursive: true });
    writeFileSync(join(mirror, "sha256", sha(text).slice(0, 2), sha(text)), text);
  }
  remove();
  expect(await run(["ensure", project], { cache: freshCache(), mirror })).toEqual({ code: 0, error: "" });
  expect(installed()).toEqual(files);
  store.down = false;
});

test("other bytes never install or upload, and every pin that cannot be had is named", async () => {
  // The store answers wrong bytes: nothing is installed, and both files are named.
  store.corrupt = true;
  remove();
  const corrupt = await run(["ensure", project], { cache: freshCache(), key: true });
  expect(corrupt.code).toBe(1);
  expect(corrupt.error).toContain("assets/models/boat.glb: not in the cache; the store holds other bytes");
  expect(corrupt.error).toContain("tools/raw/boat-source.bin: not in the cache; the store holds other bytes");
  expect(installed()).toEqual({ "assets/models/boat.glb": null, "tools/raw/boat-source.bin": null });
  store.corrupt = false;

  // A private pin without a key is named, while the public one installs.
  const keyless = await run(["ensure", project], { cache: freshCache() });
  expect(keyless.code).toBe(1);
  expect(keyless.error).toContain("tools/raw/boat-source.bin: not in the cache; the store: a private object needs OG_ASSETS_KEY");
  expect(installed()["assets/models/boat.glb"]).toBe(files["assets/models/boat.glb"]);

  // A changed file is reported and never pushed over its pin.
  place();
  writeFileSync(local("assets/models/boat.glb"), "a changed boat");
  const before = stored.size;
  const status = await run(["status", project], { cache: freshCache() });
  expect(status.code).toBe(1);
  expect(status.error).toContain("assets/models/boat.glb: modified");
  const push = await run(["push", project], { cache: freshCache(), key: true });
  expect(push.code).toBe(1);
  expect(push.error).toContain("assets/models/boat.glb is not its pin");
  expect(stored.size).toBe(before);

  // A pin the store lacks fails verify, naming it.
  const manifest = JSON.parse(readFileSync(join(project, "og-assets.json"), "utf8"));
  manifest.files.push({ path: "assets/models/raft.glb", sha256: sha("a raft never pushed"), size: 19, visibility: "public" });
  writeFileSync(join(project, "og-assets.json"), JSON.stringify(manifest));
  const verify = await run(["verify", project], { cache: freshCache(), key: true });
  expect(verify.code).toBe(1);
  expect(verify.error).toContain(`assets/models/raft.glb: the store has no ${sha("a raft never pushed")}`);
});

// Binaries live in the asset store (docs/ci.md §The asset store). Git keeps only what a bare cargo build
// compiles in, the knowledge package's previews (mounted by `git archive`), the install test's sample,
// which runs in a sandbox without the store, and vendored third-party data.
const IN_GIT = new Set([
  "crates/og-api/search/words.bin",
  "crates/og-render/data/blue-noise-64.r8",
  "crates/og-render/data/grass-tile-1024.f32",
  "templates/looks/assets/canvas/linen.png",
  "templates/looks/assets/canvas/paper.png",
  "templates/looks/assets/canvas/watercolor.png",
  "tests/fixtures/ui-text/assets/fonts/Inter-Display.ttf",
  "tests/fixtures/ui-text/assets/fonts/NotoSansArabic-subset.ttf",
  "tests/fixtures/ui-text/assets/fonts/NotoSansDevanagari-subset.ttf",
  "tests/install/fixtures/tone.mp3",
]);
test("Git holds no binary outside its list: every other one is pinned in the asset store", () => {
  const git = (...args: string[]) => Bun.spawnSync(["git", ...args], { cwd: join(import.meta.dir, "..") }).stdout.toString().split("\0").filter(Boolean);
  // Git's own reading of the committed bytes as binary, whatever the file's
  // name: every non-empty file in the index that `git grep -I` (threaded,
  // unlike `ls-files --eol`, which reads each blob in turn) does not list as text.
  const EMPTY = "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391";
  const text = new Set(git("grep", "--cached", "-I", "-l", "-z", "-e", ""));
  // A symlink's blob is its target's path (the checkout's node_modules/@origingame/engine).
  const binary = git("ls-files", "-s", "-z")
    .filter((line) => line.split(" ")[1] !== EMPTY && !line.startsWith("120000 "))
    .map((line) => line.slice(line.indexOf("\t") + 1))
    .filter((path) => !text.has(path));
  const stray = binary.filter((path) => !IN_GIT.has(path) && !/^packages\/knowledge\/previews\/[^/]+\.webp$/.test(path) && !path.startsWith("third_party/"));
  expect(stray).toEqual([]);
});
