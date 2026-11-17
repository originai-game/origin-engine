// Packs every model for the repository from the sources as downloaded
// (tools/fetch.ts and tools/catalogue.ts put them in ../.work/the-last-light,
// which git ignores): the mesh simplified to the triangles its size on
// screen needs, its maps at 1024 texels (2048 for the lighthouse, the
// cover's subject; the art contract cooks the wanderer's at 1024), as JPEG.
// The wanderer then gets its cloak (tools/cloak.ts).
//
//   bun tools/pack.ts [name...]
//
// Prints each file's size and sha256 with its source's, for README.md.
// Needs ImageMagick (`magick`); the simplifier is meshoptimizer's, through
// @gltf-transform/cli (fetched by bunx at the version below). It keeps the
// maps' seams, so a generated mesh, which is thousands of small islands,
// stops well short of a low count: collapsing across the seams (gltfpack
// -sa) reaches it and smears the texture.
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { packImages } from "./pack-glb.ts";

const GLTF_TRANSFORM = "@gltf-transform/cli@4.1.1";
/** The most a simplified surface may leave the source's, as a share of the model's size: under a pixel where each stands in the cover. */
const ERROR = "0.002";
const root = join(import.meta.dir, "..");
const work = join(root, "..", ".work", "the-last-light");

/** Each model: its source, the fewest triangles it is simplified to (within ERROR) and its maps' side. */
const MODELS: Record<string, { source: string; triangles: number; side: number; cloak?: boolean }> = {
  lighthouse: { source: "lighthouse.glb", triangles: 40000, side: 2048 },
  cottage: { source: "cottage.glb", triangles: 20000, side: 1024 },
  ruin: { source: "ruin.glb", triangles: 12000, side: 1024 },
  "rock-cliff": { source: "rock-cliff.glb", triangles: 24000, side: 1024 },
  "sea-stack": { source: "sea-stack.glb", triangles: 10000, side: 1024 },
  boulder: { source: "boulder.glb", triangles: 14000, side: 1024 },
  wanderer: { source: "wanderer-idles.glb", triangles: 44000, side: 1024, cloak: true },
  lantern: { source: "lantern/Lantern_01_1k.gltf", triangles: 6000, side: 256 },
};

function run(command: string, args: string[]): void {
  const r = spawnSync(command, args, { stdio: ["ignore", "ignore", "pipe"], maxBuffer: 1 << 26 });
  if (r.status !== 0) throw new Error(`${command} ${args.join(" ")}: ${r.stderr}`);
}
function triangles(glb: Buffer): number {
  const json = JSON.parse(glb.subarray(20, 20 + glb.readUInt32LE(12)).toString("utf8"));
  let count = 0;
  for (const mesh of json.meshes) for (const p of mesh.primitives) count += json.accessors[p.indices ?? p.attributes.POSITION].count / 3;
  return count;
}
const sha = (b: Buffer) => createHash("sha256").update(b).digest("hex");

const names = process.argv.slice(2);
const scratch = mkdtempSync(join(tmpdir(), "pack-"));
mkdirSync(join(root, "assets", "models"), { recursive: true });
for (const [name, model] of Object.entries(MODELS)) {
  if (names.length && !names.includes(name)) continue;
  const source = join(work, model.source);
  const welded = join(scratch, `${name}-welded.glb`), simple = join(scratch, `${name}-simple.glb`);
  // weld also packs a .gltf with its files into one .glb.
  run("bunx", [GLTF_TRANSFORM, "weld", source, welded]);
  const before = triangles(readFileSync(welded));
  run("bunx", [GLTF_TRANSFORM, "simplify", welded, simple, "--ratio", Math.min(1, model.triangles / before).toFixed(4), "--error", ERROR]);
  const out = join(root, "assets", "models", `${name}.glb`);
  const packed = packImages(readFileSync(simple), model.side);
  if (model.cloak) {
    const bare = join(scratch, `${name}-bare.glb`);
    writeFileSync(bare, packed);
    run("bun", [join(import.meta.dir, "cloak.ts"), bare, out]);
  } else writeFileSync(out, packed);
  const final = readFileSync(out);
  console.log(`assets/models/${name}.glb: ${final.length} bytes sha256 ${sha(final)}; ${triangles(final)} triangles of ${before}, maps at ${model.side}; from ${model.source} sha256 ${sha(readFileSync(source))}`);
}
rmSync(scratch, { recursive: true });
