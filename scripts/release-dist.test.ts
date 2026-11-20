// release-dist's downloadable effect packs (docs/release.md §1): the
// archive's bytes and entries, its place in manifest.json and SHA256SUMS,
// and the check that keeps packages/vfx-packs/ out of the engine archives.
import { afterAll, describe, expect, test } from "bun:test";
import { spawnSync } from "node:child_process";
import { chmodSync, cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { gunzipSync } from "node:zlib";
import { ENGINE_PUBKEY, verifyMinisign } from "../packages/engine/src/node/vfx-pack.ts";
import { ENGINE_PUBKEY as LAUNCHER_PUBKEY } from "../packages/launcher/og.ts";
import { pinnedKey } from "../packages/launcher/build.ts";
import { checkPacksOutside, deduplicateAssets, finish, writeCreation, writeLooks, writeVfxArchives, writeWasmModules, type Slot } from "./release-dist.ts";
import { createHash } from "node:crypto";
import { splitSymbols } from "../packages/engine/scripts/wasm-symbols.ts";

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

test("shared assets survive tar installation at every original path, with one stored copy and modes preserved", () => {
  const root = scratch("og-assets-");
  const copies = ["engine/styles/shared/font.ttf", "create/styles/shared/font.ttf", "engine/sheets/hud/assets/font.ttf"];
  for (const path of copies) {
    mkdirSync(join(root, path, ".."), { recursive: true });
    writeFileSync(join(root, path), "shared font", { mode: 0o644 });
  }
  const distinct = join(root, "engine/styles/shared/other.ttf");
  writeFileSync(distinct, "different font");
  const executable = join(root, "engine/styles/shared/executable");
  writeFileSync(executable, "shared font");
  chmodSync(executable, 0o755);
  deduplicateAssets(root);
  const out = scratch("og-assets-installed-");
  expect(spawnSync("tar", ["-czf", join(out, "assets.tgz"), "engine", "create"], { cwd: root }).status).toBe(0);
  expect(spawnSync("tar", ["-xzf", "assets.tgz"], { cwd: out }).status).toBe(0);
  for (const path of copies) expect(readFileSync(join(out, path), "utf8")).toBe("shared font");
  expect(new Set(copies.map((path) => statSync(join(out, path)).ino)).size).toBe(1);
  expect(readFileSync(join(out, "engine/styles/shared/other.ttf"), "utf8")).toBe("different font");
  if (process.platform !== "win32") expect(statSync(join(out, "engine/styles/shared/executable")).mode & 0o777).toBe(0o755);
});

test("shipping wasm keeps its exports and other custom sections, with matching named modules outside the install", () => {
  const root = scratch("og-wasm-symbols-");
  const dist = join(root, "dist");
  const symbols = join(root, "symbols");
  mkdirSync(join(dist, "web-publish"), { recursive: true });
  // One exported function returning 42, plus a name section and an unrelated
  // custom section that must survive byte-for-byte (including on a second run).
  const code = Buffer.from([
    0, 97, 115, 109, 1, 0, 0, 0,
    1, 5, 1, 96, 0, 1, 127, 3, 2, 1, 0,
    7, 7, 1, 3, 114, 117, 110, 0, 0,
    10, 6, 1, 4, 0, 65, 42, 11,
    0, 14, 4, 110, 97, 109, 101, 1, 7, 1, 0, 4, 116, 101, 115, 116,
    0, 6, 4, 107, 101, 101, 112, 99,
  ]);
  const path = join(dist, "web-publish/origin-engine_bg.wasm");
  writeFileSync(path, code);
  // A split module (scripts/wasm-split.ts) keeps its names until pack, too.
  const module_ = join(dist, "web-publish/modules/physics.0123456789ab.wasm");
  mkdirSync(join(dist, "web-publish/modules"));
  writeFileSync(module_, code);
  splitSymbols(dist, symbols);
  expect(readFileSync(module_)).toEqual(readFileSync(path));
  expect(readFileSync(join(symbols, "web-publish/modules/physics.0123456789ab.wasm"))).toEqual(code);
  const shipped = readFileSync(path);
  const module = new WebAssembly.Module(shipped);
  expect(WebAssembly.Module.customSections(module, "name")).toHaveLength(0);
  expect(new Uint8Array(WebAssembly.Module.customSections(module, "keep")[0]!)).toEqual(new Uint8Array([99]));
  const run = new WebAssembly.Instance(module).exports.run;
  expect(typeof run).toBe("function");
  if (typeof run !== "function") throw new Error("missing wasm export");
  expect(run()).toBe(42);
  expect(readFileSync(join(symbols, "web-publish/origin-engine_bg.wasm"))).toEqual(code);
  splitSymbols(dist, symbols);
  expect(readFileSync(path)).toEqual(shipped);
  expect(readFileSync(join(symbols, "web-publish/origin-engine_bg.wasm"))).toEqual(code);
});

describe("vfx archives", () => {
  test("are the same bytes on every build, root <name>/, owned by 0:0 at mtime 0, without preview/", () => {
    const a = scratch("og-vfx-a-");
    const b = scratch("og-vfx-b-");
    // A second copy of the pack, made later: its files' times and order on disk differ.
    const copy = scratch("og-vfx-src-");
    cpSync(join(fixture, "spark-pack"), join(copy, "spark-pack"), { recursive: true });
    const first = writeVfxArchives(a, "1.2.3", fixture);
    const second = writeVfxArchives(b, "1.2.3", copy);
    expect(Object.keys(first)).toEqual(["spark-pack"]);
    expect(readFileSync(join(a, "vfx/spark-pack.tar.gz")).equals(readFileSync(join(b, "vfx/spark-pack.tar.gz")))).toBe(true);
    expect(second).toEqual(first);
    expect(first["spark-pack"]).toMatchObject({ url: "https://origingame.ai/downloads/engine/1.2.3/vfx/spark-pack.tar.gz", version: "1.0.0", minEngine: "0.7.0" });
    // Relative to its directory: GNU tar (Git Bash) reads `C:\...` as a remote host.
    const list = spawnSync("tar", ["-tvzf", "spark-pack.tar.gz"], { cwd: join(a, "vfx"), encoding: "utf8" });
    expect(list.status).toBe(0);
    const names = spawnSync("tar", ["-tzf", "spark-pack.tar.gz"], { cwd: join(a, "vfx"), encoding: "utf8" }).stdout.trim().split("\n");
    expect(names).toEqual(["spark-pack/", "spark-pack/burst.ogt", "spark-pack/extension.json", "spark-pack/glint.ogt", "spark-pack/spark.png", "spark-pack/sparks.ogt"]);
    for (const line of list.stdout.trim().split("\n")) expect(line).toMatch(/^(drwxr-xr-x|-rw-r--r--) +(0 root +root|root\/root) /); // bsdtar, GNU tar
    // uid, gid and mtime in every header (at 108, 116, 136) are 0.
    const tar = gunzipSync(readFileSync(join(a, "vfx/spark-pack.tar.gz")));
    for (let at = 0, n = 0; n < names.length; n++) {
      const field = (o: number, len: number) => Number.parseInt(tar.toString("latin1", at + o, at + o + len).replace(/\0.*$/, ""), 8);
      expect([field(108, 8), field(116, 8), field(136, 12)]).toEqual([0, 0, 0]);
      at += 512 + Math.ceil(field(124, 12) / 512) * 512;
    }
  });

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
    const { looks, filled } = drawLooks(dir, "1.2.3-dev.4", "0".repeat(40)).looks();
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
        "neon-vector": { url: "https://origingame.ai/downloads/engine/1.2.3-dev.4/previews/neon-vector.webp", sha256: sha("neon"), size: 4, engine: "1.2.3-dev.4" },
        pixel: { url: "https://origingame.ai/downloads/engine/1.2.3-dev.4/previews/pixel.webp", sha256: sha("pix"), size: 3, engine: "1.2.3-dev.4" },
        "pixel.8bit": { url: "https://origingame.ai/downloads/engine/1.2.3-dev.4/previews/pixel.8bit.webp", sha256: sha("8bit"), size: 4, engine: "1.2.3-dev.4" },
      },
    });
    // ui and mood previews are the creation document's alone, filled like the rest.
    expect(manifest.creation).toEqual({ url: "https://origingame.ai/downloads/engine/1.2.3-dev.4/creation.json", sha256: sha(readFileSync(join(dir, "creation.json"))), size: statSync(join(dir, "creation.json")).size });
    expect(JSON.parse(readFileSync(join(dir, "creation.json"), "utf8")).previews["previews/mood/dark.webp"]).toEqual({ url: "https://origingame.ai/downloads/engine/1.2.3-dev.4/previews/mood/dark.webp", sha256: sha("dark"), size: 4, engine: "1.2.3-dev.4" });
    expect(manifest).toMatchObject({ version: "1.2.3-dev.4", channel: "dev", checksums: "SHA256SUMS", signature: "SHA256SUMS.minisig" });
    expect(manifest).not.toHaveProperty("symbols");
    const sums = readFileSync(join(dir, "SHA256SUMS"), "utf8");
    expect(sums).not.toContain(symbols);
    expect(sums).toContain(`${vfx["spark-pack"]!.sha256}  vfx/spark-pack.tar.gz\n`);
    expect(sums).toContain(`${hash}  wasm/publish/physics.${hash.slice(0, 12)}.wasm\n`);
    expect(sums).toContain("  manifest.json\n");
    expect(sums).toContain(`${manifest.looks.catalogue.sha256}  looks/catalogue.json\n`);
    expect(sums).toContain(`${sha("pix")}  previews/pixel.webp\n`);
    for (const f of ["previews/pixel.8bit.webp", "previews/ui/fantasy.webp", "previews/mood/dark.webp", "creation.json"]) expect(sums).toContain(`  ${f}\n`);
    // Every line verifies from the version's directory, as publish checks them.
    // Git Bash on Windows has sha256sum, and shasum only as a Perl script.
    const check = process.platform === "win32" ? ["sha256sum", "-c", "SHA256SUMS"] : ["shasum", "-a", "256", "-c", "SHA256SUMS"];
    expect(spawnSync(check[0]!, check.slice(1), { cwd: dir }).status).toBe(0);
  });

  test("never ride in an engine archive", () => {
    const root = scratch("og-vfx-dist-");
    mkdirSync(join(root, "engine/extensions/materials"), { recursive: true });
    writeFileSync(join(root, "engine/package.json"), JSON.stringify({ files: ["src", "extensions"] }));
    mkdirSync(join(root, "overlay"));
    expect(() => checkPacksOutside(root, ["spark-pack"])).not.toThrow();
    mkdirSync(join(root, "engine/extensions/spark-pack"));
    expect(() => checkPacksOutside(root, ["spark-pack"])).toThrow(/engine\/extensions\/spark-pack \(spark-pack is a downloadable pack\)/);
    rmSync(join(root, "engine/extensions/spark-pack"), { recursive: true });
    mkdirSync(join(root, "overlay/packages/vfx-packs"), { recursive: true });
    expect(() => checkPacksOutside(root, [])).toThrow(/overlay\/packages\/vfx-packs/);
    rmSync(join(root, "overlay/packages"), { recursive: true });
    writeFileSync(join(root, "engine/package.json"), JSON.stringify({ files: ["src", "../vfx-packs"] }));
    expect(() => checkPacksOutside(root, [])).toThrow(/engine\/package.json files: \.\.\/vfx-packs/);
  });

  test("the checkout's engine package lists nothing that reaches packages/vfx-packs/", () => {
    const pkg = JSON.parse(readFileSync(join(import.meta.dir, "../packages/engine/package.json"), "utf8")) as { files: string[] };
    expect(pkg.files.filter((f) => f.includes("vfx-packs") || f.split("/").includes(".."))).toEqual([]);
  });
});

const sha = (s: string | Buffer) => createHash("sha256").update(s).digest("hex");
const LINUX = { og: sha("og"), addon: sha("addon") };
/** The slots of a version: two looks, a variant, a UI pack and a mood, with their bytes. */
const SLOTS: (Slot & { bytes: string })[] = [
  { path: "previews/pixel.webp", kind: "look", look: "pixel", bytes: "pix" },
  { path: "previews/pixel.8bit.webp", kind: "variant", look: "pixel", variant: "8bit", bytes: "8bit" },
  { path: "previews/neon-vector.webp", kind: "look", look: "neon-vector", bytes: "neon" },
  { path: "previews/ui/fantasy.webp", kind: "ui", pack: "fantasy", bytes: "ui" },
  { path: "previews/mood/dark.webp", kind: "mood", look: "neon-vector", mood: "dark", bytes: "dark" },
];
/** A version directory with its previews as scripts/release.sh previews leaves them, and an engine package with a catalogue. */
function drawLooks(dir: string, version: string, revision: string) {
  const engine = scratch("og-looks-engine-");
  mkdirSync(join(engine, "knowledge/looks"), { recursive: true });
  const catalogue = `${JSON.stringify({ schemaVersion: 3, styles: [{ slug: "pixel", preview: "previews/pixel.webp" }, { slug: "neon-vector", preview: "previews/neon-vector.webp" }] }, null, 2)}\n`;
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

describe("the look previews", () => {
  test("ship the package's catalogue byte for byte", () => {
    const dir = scratch("og-looks-");
    const d = drawLooks(dir, "0.9.1", "a".repeat(40));
    d.looks();
    expect(readFileSync(join(dir, "looks/catalogue.json"), "utf8")).toBe(d.catalogue);
  });

  test("are refused unless this version drew every style's on the GPU host with its own og and addon", () => {
    const dir = scratch("og-looks-");
    const d = drawLooks(dir, "0.9.1", "a".repeat(40));
    const refused = (change: object, why: RegExp) => {
      d.write({ ...d.record, ...change });
      expect(() => d.looks()).toThrow(why);
      d.write(d.record);
    };
    refused({ engine: "0.9.0" }, /drawn by engine 0\.9\.0, not 0\.9\.1/);
    refused({ revision: "b".repeat(40) }, /drawn at b{40}/);
    refused({ tier: "vulkan-cpu" }, /drawn on vulkan-cpu, not vulkan/);
    refused({ tier: "metal" }, /drawn on metal, not vulkan/);
    refused({ tier: null }, /drawn on no renderer/);
    refused({ dirty: true }, /dirty templates\/looks/);
    refused({ og: sha("another og") }, /another og or addon/);
    // An old preview left in place: its bytes are not the ones drawn.
    writeFileSync(join(dir, "previews/pixel.webp"), "old");
    expect(() => d.looks()).toThrow(/previews\/pixel\.webp is not the preview looks-render\.json recorded/);
    writeFileSync(join(dir, "previews/pixel.webp"), "pix");
    rmSync(join(dir, "previews/mood/dark.webp"));
    expect(() => d.looks()).toThrow(/no previews\/mood\/dark\.webp/);
    writeFileSync(join(dir, "previews/mood/dark.webp"), "dark");
    writeFileSync(join(dir, "previews/ui/retired.webp"), "x");
    expect(() => d.looks()).toThrow(/previews\/ui\/retired\.webp: no preview slot names them/);
    rmSync(join(dir, "previews/ui/retired.webp"));
    expect(() => d.looks()).not.toThrow();
    rmSync(join(dir, "looks-render.json"));
    expect(() => d.looks()).toThrow(/no looks-render\.json/);
  });
});

describe("og vfx add's signature check", () => {
  test("pins the key docs/release.md states, the launcher's, and accepts and refuses what minisign signed as the launcher does", () => {
    expect(ENGINE_PUBKEY).toBe(pinnedKey()!);
    expect(ENGINE_PUBKEY).toBe(LAUNCHER_PUBKEY);
    // The launcher's fixtures: SHA256SUMS signed by minisign itself (prehashed and legacy) with test.pub's key.
    const fx = join(import.meta.dir, "../packages/launcher/tests/fixtures");
    const read = (f: string) => readFileSync(join(fx, f));
    const key = read("test.pub").toString().split("\n")[1]!;
    expect(verifyMinisign(read("SHA256SUMS"), read("SHA256SUMS.minisig").toString(), key)).toBe("origin-engine 1.0.0 test");
    expect(verifyMinisign(read("SHA256SUMS"), read("SHA256SUMS.legacy.minisig").toString(), key)).toBe("origin-engine 1.0.0 legacy");
    const changed = Buffer.from(read("SHA256SUMS"));
    changed[0] = changed[0]! ^ 1;
    expect(() => verifyMinisign(changed, read("SHA256SUMS.minisig").toString(), key)).toThrow(/does not verify/);
    expect(() => verifyMinisign(read("SHA256SUMS"), read("SHA256SUMS.minisig").toString().replace("1.0.0 test", "9.9.9 test"), key)).toThrow(/trusted comment does not verify/);
    expect(() => verifyMinisign(read("SHA256SUMS"), read("SHA256SUMS.minisig").toString(), ENGINE_PUBKEY)).toThrow(/not the engine's F6CC11AC12997D89/);
  });
});
