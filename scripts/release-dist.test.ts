// release-dist's downloadable effect packs (docs/release.md §1): the
// archive's bytes and entries, its place in manifest.json and SHA256SUMS,
// and the check that keeps packages/vfx-packs/ out of the engine archives.
import { afterAll, describe, expect, test } from "bun:test";
import { execFileSync, spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { finish, writeCreation, writeLooks, writeVfxArchives, writeWasmModules, type Slot } from "./release-dist.ts";
import { createHash } from "node:crypto";

const fixture = join(import.meta.dir, "../tests/fixtures/vfx-release");
const made: string[] = [];
const scratch = (prefix: string) => {
  const d = mkdtempSync(join(tmpdir(), prefix));
  made.push(d);
  return d;
};
afterAll(() => {
  for (const d of made) rmSync(d, { recursive: true, force: true });
});
delete process.env.OG_ENGINE_MINISIGN_KEY;

describe("vfx archives", () => {
  test("are named in manifest.json and listed in SHA256SUMS beside the engine archives", () => {
    const dir = scratch("og-vfx-release-");
    const vfx = writeVfxArchives(dir, "1.2.3-dev.4", fixture);
    const symbols = "origingame-engine-wasm-symbols-1.2.3-dev.4.tgz";
    writeFileSync(join(dir, symbols), "matching named modules");
    // A split engine module in the engine package: copied under the platform's
    // wasm route, signed, and its listing told where it is served.
    const engine = scratch("og-wasm-modules-");
    mkdirSync(join(engine, "dist/web-publish/modules"), { recursive: true });
    const module = new Uint8Array([0, 97, 115, 109, 1, 0, 0, 0]);
    const hash = createHash("sha256").update(module).digest("hex");
    writeFileSync(join(engine, `dist/web-publish/modules/physics.${hash.slice(0, 12)}.wasm`), module);
    writeFileSync(join(engine, "dist/web-publish/modules/modules.json"), JSON.stringify({ physics: { file: `modules/physics.${hash.slice(0, 12)}.wasm`, bytes: 8, sha256: hash } }));
    expect(writeWasmModules(engine, dir, "1.2.3-dev.4")).toEqual([`wasm/publish/physics.${hash.slice(0, 12)}.wasm`]);
    expect(JSON.parse(readFileSync(join(engine, "dist/web-publish/modules/modules.json"), "utf8")).physics.base).toBe("https://origingame.ai/downloads/engine/1.2.3-dev.4/wasm/publish/");
    const drawn = drawLooks(dir, "1.2.3-dev.4", "0".repeat(40));
    const defaultPreview = join(dir, "previews/realistic-default.webp");
    rmSync(defaultPreview);
    expect(drawn.looks).toThrow("no previews/realistic-default.webp");
    writeFileSync(defaultPreview, "realistic");
    const { looks, filled } = drawn.looks();
    const creation = writeCreation(dir, "1.2.3-dev.4", { previews: filled });
    finish(dir, "1.2.3-dev.4", "0".repeat(40), "2026-11-07T00:00:00.000Z", vfx, { looks, creation, files: ["looks/catalogue.json", ...SLOTS.map((s) => s.path), "creation.json"] });
    const manifest = JSON.parse(readFileSync(join(dir, "manifest.json"), "utf8"));
    expect(manifest.vfx).toEqual(vfx);
    // The shape a platform reads (AGENTS.md §Ownership, docs/release.md §1): changing it is a break.
    expect(Object.keys(manifest).sort()).toEqual(["channel", "checksums", "creation", "looks", "releasedAt", "revision", "signature", "targets", "version", "vfx"]);
    // Origin reads looks.catalogue and looks.previews.<slug> (docs/release.md §1 The looks).
    expect(manifest.looks).toEqual({
      catalogue: { url: "https://origingame.ai/downloads/engine/1.2.3-dev.4/looks/catalogue.json", sha256: expect.stringMatching(/^[0-9a-f]{64}$/), size: expect.any(Number) },
      previews: {
        neon: { url: "https://origingame.ai/downloads/engine/1.2.3-dev.4/previews/neon.webp", sha256: sha("neon"), size: 4, engine: "1.2.3-dev.4" },
        pixel: { url: "https://origingame.ai/downloads/engine/1.2.3-dev.4/previews/pixel.webp", sha256: sha("pix"), size: 3, engine: "1.2.3-dev.4" },
        "pixel.8bit": { url: "https://origingame.ai/downloads/engine/1.2.3-dev.4/previews/pixel.8bit.webp", sha256: sha("8bit"), size: 4, engine: "1.2.3-dev.4" },
      },
    });
    // Default and UI previews are the creation document's alone, not catalogue presets.
    expect(manifest.creation).toEqual({ url: "https://origingame.ai/downloads/engine/1.2.3-dev.4/creation.json", sha256: sha(readFileSync(join(dir, "creation.json"))), size: statSync(join(dir, "creation.json")).size });
    expect(JSON.parse(readFileSync(join(dir, "creation.json"), "utf8")).previews["previews/ui/default.webp"]).toEqual({ url: "https://origingame.ai/downloads/engine/1.2.3-dev.4/previews/ui/default.webp", sha256: sha("ui"), size: 2, engine: "1.2.3-dev.4" });
    expect(JSON.parse(readFileSync(join(dir, "creation.json"), "utf8")).previews["previews/realistic-default.webp"]).toEqual({ url: "https://origingame.ai/downloads/engine/1.2.3-dev.4/previews/realistic-default.webp", sha256: sha("realistic"), size: 9, engine: "1.2.3-dev.4" });
    expect(manifest).toMatchObject({ version: "1.2.3-dev.4", channel: "dev", checksums: "SHA256SUMS", signature: "SHA256SUMS.minisig" });
    expect(manifest).not.toHaveProperty("symbols");
    const sums = readFileSync(join(dir, "SHA256SUMS"), "utf8");
    expect(sums).not.toContain(symbols);
    expect(sums).toContain(`${vfx["spark-pack"]!.sha256}  vfx/spark-pack.tar.gz\n`);
    expect(sums).toContain(`${hash}  wasm/publish/physics.${hash.slice(0, 12)}.wasm\n`);
    expect(sums).toContain("  manifest.json\n");
    expect(sums).toContain(`${manifest.looks.catalogue.sha256}  looks/catalogue.json\n`);
    expect(sums).toContain(`${sha("pix")}  previews/pixel.webp\n`);
    for (const f of ["previews/pixel.8bit.webp", "previews/ui/default.webp", "previews/realistic-default.webp", "creation.json"]) expect(sums).toContain(`  ${f}\n`);
    // A dev build carries the launcher's installers too: the site's root installer may be older than its name.
    for (const f of ["install.sh", "install.ps1"]) expect(sums).toContain(`${sha(readFileSync(join(dir, f)))}  ${f}\n`);
    // Every line verifies from the version's directory, as publish checks them.
    // Git Bash on Windows has sha256sum, and shasum only as a Perl script.
    const check = process.platform === "win32" ? ["sha256sum", "-c", "SHA256SUMS"] : ["shasum", "-a", "256", "-c", "SHA256SUMS"];
    expect(spawnSync(check[0]!, check.slice(1), { cwd: dir }).status).toBe(0);
  });
});

const sha = (s: string | Buffer) => createHash("sha256").update(s).digest("hex");
const LINUX = { og: sha("og"), addon: sha("addon") };
/** The slots of a version: the engine default, two looks, a variant and a UI pack, with their bytes. */
const SLOTS: (Slot & { bytes: string })[] = [
  { path: "previews/realistic-default.webp", kind: "default", bytes: "realistic" },
  { path: "previews/pixel.webp", kind: "look", look: "pixel", bytes: "pix" },
  { path: "previews/pixel.8bit.webp", kind: "variant", look: "pixel", variant: "8bit", bytes: "8bit" },
  { path: "previews/neon.webp", kind: "look", look: "neon", bytes: "neon" },
  { path: "previews/ui/default.webp", kind: "ui", pack: "default", bytes: "ui" },
];
/** A version directory with its previews as scripts/release.sh previews leaves them, and an engine package with a catalogue. */
function drawLooks(dir: string, version: string, revision: string) {
  const engine = scratch("og-looks-engine-");
  mkdirSync(join(engine, "knowledge/looks"), { recursive: true });
  const catalogue = `${JSON.stringify({ schemaVersion: 3, styles: [{ slug: "pixel", preview: "previews/pixel.webp" }, { slug: "neon", preview: "previews/neon.webp" }] }, null, 2)}\n`;
  writeFileSync(join(engine, "knowledge/looks/catalogue.json"), catalogue);
  for (const s of SLOTS) {
    mkdirSync(join(dir, s.path, ".."), { recursive: true });
    writeFileSync(join(dir, s.path), s.bytes);
  }
  const record = { engine: version, revision, dirty: false, tier: "vulkan", ...LINUX, previews: Object.fromEntries(SLOTS.map((s) => [s.path, { sha256: sha(s.bytes), size: s.bytes.length }])) };
  const write = (r: object) => writeFileSync(join(dir, "looks-render.json"), JSON.stringify(r));
  write(record);
  return { engine, catalogue, record, write, looks: () => writeLooks(dir, version, revision, engine, LINUX, SLOTS) };
}

test("docs/web-export.md's variant table is wasm-crates.txt's feature sets", () => {
  // The gate builds og-host-web's wasm variants with scripts/wasm-crates.txt's
  // `variant` and `lanes` lines (scripts/lib.sh reads it); the doc's Built-with
  // column states each set, so a feature change on either side shows here.
  const repo = join(import.meta.dir, "..");
  const feats = new Map<string, string>();
  for (const line of readFileSync(join(repo, "scripts", "wasm-crates.txt"), "utf8").split("\n")) {
    const w = line.replace(/#.*/, "").trim().split(/\s+/);
    if ((w[0] === "variant" || w[0] === "lanes") && w[2] === "og-host-web") feats.set(w[1]!, w[3]!);
  }
  const doc = readFileSync(join(repo, "docs", "web-export.md"), "utf8");
  const table = doc.slice(doc.indexOf("| Variant | Where |"));
  const cells = [...table.matchAll(/`--no-default-features --features ([a-z0-9,]+)`/g)].map((m) => m[1]!);
  expect([...new Set(cells)].sort()).toEqual([...new Set(feats.values())].sort());
  for (const name of [...feats.keys()]) expect(doc).toContain(`--features ${feats.get(name)}`);
});

test("private staging round-trips all hosts and rejects a wrong commit or corrupt payload", async () => {
  const source = scratch("og-stage-source-");
  const dest = scratch("og-stage-dest-");
  const version = "dev.20261128.9001";
  const revision = execFileSync("git", ["rev-parse", "HEAD"], { cwd: import.meta.dir, encoding: "utf8" }).trim();
  const targets = ["linux-x64", "darwin-arm64", "win32-x64"];
  const objects = new Map<string, Uint8Array>();
  const server = Bun.serve({
    port: 0,
    async fetch(req) {
      if (req.headers.get("authorization") !== "Bearer test-staging-only") return new Response(null, { status: 401 });
      const path = new URL(req.url).pathname;
      if (req.method === "PUT") {
        const bytes = new Uint8Array(await req.arrayBuffer());
        if (sha(Buffer.from(bytes)) !== req.headers.get("x-content-sha256")) return new Response(null, { status: 400 });
        objects.set(path, bytes);
        return new Response(null, { status: 201 });
      }
      return new Response(objects.get(path) ?? null, { status: objects.has(path) ? 200 : 404 });
    },
  });
  const cli = async (verb: string, dir: string, extra: string[] = [], token = "test-staging-only") => {
    const p = Bun.spawn([process.execPath, join(import.meta.dir, "release-stage.ts"), verb, "--version", version,
      "--revision", revision, "--dir", dir, ...extra], {
      cwd: join(import.meta.dir, ".."), stdout: "pipe", stderr: "pipe",
      env: { ...process.env, OG_ENGINE_STAGING_URL: `http://127.0.0.1:${server.port}`, OG_ENGINE_STAGING_TOKEN: token },
    });
    const [exit, stderr] = await Promise.all([p.exited, new Response(p.stderr).text()]);
    return { exit, stderr };
  };
  try {
    const names = targets.map((t) => `origingame-engine-${t}-${version}.tgz`).concat([
      `origingame-engine-${version}.tgz`, `origingame-create-game-${version}.tgz`,
      `origingame-design-${version}.tgz`, `origingame-engine-wasm-symbols-${version}.tgz`, "NOTES.md",
    ]);
    for (const name of names) writeFileSync(join(source, name), `distinct bytes of ${name}`);
    for (const target of targets) {
      expect((await cli("record", source, ["--target", target, "--elapsed", "37", "--dirty"])).exit).toBe(0);
      expect((await cli("push", source, ["--target", target])).exit).toBe(0);
    }
    expect((await cli("pull", dest, [], "wrong-token")).stderr).toContain("401");
    expect((await cli("pull", dest)).exit).toBe(0);
    for (const name of names) expect(readFileSync(join(dest, name))).toEqual(readFileSync(join(source, name)));
    const path = `/${version}/${revision}/darwin-arm64/pack.json`;
    const original = objects.get(path)!;
    const record = JSON.parse(new TextDecoder().decode(original));
    record.revision = "f".repeat(40);
    objects.set(path, new TextEncoder().encode(JSON.stringify(record)));
    expect((await cli("pull", dest)).stderr).toContain("pack identity differs");
    objects.set(path, original);
    objects.set(`/${version}/${revision}/darwin-arm64/origingame-engine-darwin-arm64-${version}.tgz`, new TextEncoder().encode("corrupt"));
    expect((await cli("pull", dest)).stderr).toContain("staging sha256 or size mismatch");
  } finally {
    server.stop(true);
  }
});
