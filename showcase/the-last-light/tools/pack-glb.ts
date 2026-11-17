// Re-encodes a GLB's images for the repository: a Meshy GLB carries 4k
// textures as a 25 MB PNG normal map and 15 MB of JPEG; this rewrites them
// at `side` texels as JPEG (the normal map with full chroma, since its
// channels are directions) and leaves everything else untouched.
// tools/pack.ts runs it over every model; alone:
//
//   bun tools/pack-glb.ts <in.glb> <out.glb> [side=1024]
//
// Needs ImageMagick (`magick`).
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";

/** The GLB with its images at `side` texels at most, as JPEG. */
export function packImages(glb: Buffer, side: number): Buffer {
  if (glb.readUInt32LE(0) !== 0x46546c67) throw new Error("not a GLB");
  const jsonLength = glb.readUInt32LE(12);
  const json = JSON.parse(glb.subarray(20, 20 + jsonLength).toString("utf8"));
  const bin = glb.subarray(20 + jsonLength + 8, 20 + jsonLength + 8 + glb.readUInt32LE(20 + jsonLength));

  const normals = new Set<number>();
  for (const m of json.materials ?? []) if (m.normalTexture) normals.add(json.textures[m.normalTexture.index].source);
  const replaced = new Map<number, Buffer>();
  for (const [i, image] of (json.images ?? []).entries()) {
    const view = json.bufferViews[image.bufferView];
    const bytes = bin.subarray(view.byteOffset ?? 0, (view.byteOffset ?? 0) + view.byteLength);
    const r = spawnSync("magick", ["-", "-resize", `${side}x${side}>`, "-strip", ...(normals.has(i) ? ["-sampling-factor", "4:4:4", "-quality", "90"] : ["-quality", "88"]), "jpg:-"], { input: bytes, maxBuffer: 1 << 28 });
    if (r.status !== 0) throw new Error(`magick: ${r.stderr}`);
    replaced.set(image.bufferView, r.stdout);
    image.mimeType = "image/jpeg";
  }

  const parts: Buffer[] = [];
  let offset = 0;
  for (const [i, view] of json.bufferViews.entries()) {
    const bytes = replaced.get(i) ?? bin.subarray(view.byteOffset ?? 0, (view.byteOffset ?? 0) + view.byteLength);
    const pad = (4 - (offset % 4)) % 4;
    if (pad) { parts.push(Buffer.alloc(pad)); offset += pad; }
    view.byteOffset = offset;
    view.byteLength = bytes.length;
    parts.push(bytes);
    offset += bytes.length;
  }
  const tail = (4 - (offset % 4)) % 4;
  if (tail) parts.push(Buffer.alloc(tail));
  const body = Buffer.concat(parts);
  json.buffers = [{ byteLength: body.length }];
  let text = Buffer.from(JSON.stringify(json), "utf8");
  if (text.length % 4) text = Buffer.concat([text, Buffer.alloc(4 - (text.length % 4), 0x20)]);
  const header = Buffer.alloc(12);
  header.writeUInt32LE(0x46546c67, 0); header.writeUInt32LE(2, 4); header.writeUInt32LE(12 + 8 + text.length + 8 + body.length, 8);
  const chunk = (length: number, type: number) => { const b = Buffer.alloc(8); b.writeUInt32LE(length, 0); b.writeUInt32LE(type, 4); return b; };
  return Buffer.concat([header, chunk(text.length, 0x4e4f534a), text, chunk(body.length, 0x004e4942), body]);
}

if (import.meta.main) {
  const [input, output, sideArg] = process.argv.slice(2);
  if (!input || !output) throw new Error("usage: bun tools/pack-glb.ts <in.glb> <out.glb> [side]");
  const glb = readFileSync(input);
  const packed = packImages(glb, Number(sideArg ?? 1024));
  mkdirSync(dirname(output), { recursive: true });
  writeFileSync(output, packed);
  const sha = (b: Buffer) => createHash("sha256").update(b).digest("hex");
  console.log(`${output}: ${packed.length} bytes sha256 ${sha(packed)} (from ${glb.length} bytes sha256 ${sha(glb)})`);
}
