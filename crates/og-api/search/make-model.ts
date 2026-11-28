// Quantizes the pinned potion-base-8M embeddings for `og api find`.
//   bun crates/og-api/search/make-model.ts <model dir> <words.bin> [dim]
// Safetensors is an eight-byte header length, JSON tensor descriptors and
// little-endian tensor bytes; no model runtime or Python dependency is needed.
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
const SHA256 = {
  "model.safetensors": "f65d0f325faadc1e121c319e2faa41170d3fa07d8c89abd48ca5358d9a223de2",
  "tokenizer.json": "e67e803f624fb4d67dea1c730d06e1067e1b14d830e2c2202569e3ef0f70bb50",
};
const [src, out, d] = process.argv.slice(2),
  dim = Number(d ?? 128);
for (const [name, want] of Object.entries(SHA256)) {
  const got = createHash("sha256")
    .update(readFileSync(join(src!, name)))
    .digest("hex");
  if (got !== want) throw new Error(`${name}: sha256 ${got}, expected ${want}`);
}
const tok = JSON.parse(readFileSync(join(src!, "tokenizer.json"), "utf8")),
  vocab: Record<string, number> = tok.model.vocab;
const words: string[] = [];
for (const [w, i] of Object.entries(vocab)) words[i] = w;
const tensor = readFileSync(join(src!, "model.safetensors")),
  headerLen = Number(tensor.readBigUInt64LE());
const desc = JSON.parse(tensor.subarray(8, 8 + headerLen).toString()).embeddings;
if (desc.dtype !== "F32" || desc.shape[0] !== words.length || !Number.isInteger(dim) || dim < 1 || dim > desc.shape[1])
  throw new Error("invalid embeddings shape or dimension");
const scales = Buffer.alloc(words.length * 4),
  quantized = Buffer.alloc(words.length * dim),
  encoded: Buffer[] = [];
for (let i = 0; i < words.length; i++) {
  const word = Buffer.from(words[i]!);
  if (word.length > 255) throw new Error("token exceeds u8 length");
  encoded.push(Buffer.from([word.length]), word);
  const offset = 8 + headerLen + desc.data_offsets[0] + i * desc.shape[1] * 4;
  let max = 0;
  for (let j = 0; j < dim; j++) max = Math.max(max, Math.abs(tensor.readFloatLE(offset + j * 4)));
  const scale = Math.fround(max / 127) || 1;
  scales.writeFloatLE(scale, i * 4);
  for (let j = 0; j < dim; j++) {
    const v = Math.fround(tensor.readFloatLE(offset + j * 4) / scale),
      lo = Math.floor(v);
    const q = v - lo === 0.5 ? lo + (lo % 2 === 0 ? 0 : 1) : Math.round(v);
    quantized.writeInt8(Math.min(127, Math.max(-127, q)), i * dim + j);
  }
}
const hdr = Buffer.alloc(20);
hdr.write("OGWORDS1");
hdr.writeUInt32LE(words.length, 8);
hdr.writeUInt32LE(dim, 12);
hdr.writeUInt32LE(vocab[tok.model.unk_token]!, 16);
writeFileSync(out!, Buffer.concat([hdr, ...encoded, scales, quantized]));
console.log(`${out}: ${words.length} tokens x ${dim}`);
