// Gives the wanderer a cloak: adds a second mesh to the rigged GLB, a
// weathered red cape hung from the collar round the back to the calves,
// skinned to the spine so og.Garment can pose and simulate it.
//
//   bun tools/cloak.ts <in.glb> <out.glb>
//
// The cape's vertex colour alpha is how far each point may leave the
// skinned pose (0 at the collar, 1 at the hem): og.Garment reads it. Its
// cloth is drawn by ImageMagick (`magick`): mottled wool, a dirty hem and
// a pale embroidered vine. README.md records the output's checksum.
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";

const [input, output] = process.argv.slice(2);
if (!input || !output) throw new Error("usage: bun tools/cloak.ts <in.glb> <out.glb>");

const glb = readFileSync(input);
if (glb.readUInt32LE(0) !== 0x46546c67) throw new Error(`${input} is not a GLB`);
const jsonLength = glb.readUInt32LE(12);
const json = JSON.parse(glb.subarray(20, 20 + jsonLength).toString("utf8"));
const bin = glb.subarray(20 + jsonLength + 8, 20 + jsonLength + 8 + glb.readUInt32LE(20 + jsonLength));
if (json.meshes.some((m: any) => m.name === "cloak")) throw new Error(`${input} already wears a cloak`);

// ── the cape, in the model's space: +y up, facing +z, its left at +x ──────
const COLS = 41, ROWS = 30;
const CENTRE = { x: -0.01, z: 0.07 };
const TOP = 1.475, HEM = 0.3;
const SPAN = (118 * Math.PI) / 180; // how far round from the back each edge reaches
const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v));
const smooth = (a: number, b: number, v: number) => { const t = clamp((v - a) / (b - a), 0, 1); return t * t * (3 - 2 * t); };
const mix = (a: number, b: number, t: number) => a + (b - a) * t;
function hash(i: number): number {
  let h = Math.imul(i ^ 0x9e3779b9, 0x85ebca6b) >>> 0;
  h = Math.imul(h ^ (h >>> 13), 0xc2b2ae35) >>> 0;
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}
/** The hem's length at a column: ragged, with a few deep tears. */
function hem(c: number): number {
  const slow = 0.5 * hash(Math.floor(c / 5)) + 0.5 * hash(Math.floor((c + 2) / 3) + 100);
  const tear = hash(c + 300) > 0.86 ? 0.1 : 0;
  return 1 - 0.13 * slow - tear - 0.04 * hash(c + 700);
}

// The skin's joints (their order in the GLB's skin, checked below by name).
const J = { Hips: 0, Spine02: 1, Spine01: 2, Spine: 3, LeftShoulder: 8, RightShoulder: 12 } as const;
const jointY = { Hips: 0.973, Spine02: 1.102, Spine01: 1.23, Spine: 1.363 };
for (const [name, index] of Object.entries(J)) {
  const node = json.nodes[json.skins[0].joints[index]];
  if (node.name !== name) throw new Error(`skin joint ${index} is ${node.name}, not ${name}`);
}

const position: number[] = [], normal: number[] = [], uv: number[] = [], color: number[] = [], joints: number[] = [], weights: number[] = [];
const at = (r: number, c: number): [number, number, number] => {
  const u = c / (COLS - 1), v = r / (ROWS - 1);
  const phi = (u * 2 - 1) * SPAN;
  const y = TOP - (TOP - HEM) * v * mix(1, hem(c), smooth(0.3, 1, v));
  const t = (TOP - y) / (TOP - HEM);
  // Round the neck, out over the shoulders, then a cone widening to the hem.
  const shoulders = smooth(0, 0.075, t), fall = Math.pow(clamp((t - 0.075) / 0.925, 0, 1), 0.8);
  let rx = mix(mix(0.1, 0.275, shoulders), 0.43, fall), rz = mix(mix(0.085, 0.165, shoulders), 0.36, fall);
  // Pleats, deeper towards the hem.
  const pleat = 1 + 0.055 * smooth(0.12, 0.9, t) * Math.sin(phi * 9 + 1.3) + 0.02 * smooth(0.3, 1, t) * Math.sin(phi * 23 + 0.4);
  rx *= pleat; rz *= pleat;
  return [CENTRE.x + rx * Math.sin(phi), y, CENTRE.z - rz * Math.cos(phi)];
};
for (let r = 0; r < ROWS; r++) {
  for (let c = 0; c < COLS; c++) {
    const p = at(r, c);
    position.push(...p);
    // The normal from the neighbours, outwards.
    const a = at(r, Math.min(COLS - 1, c + 1)), b = at(r, Math.max(0, c - 1)), d = at(Math.min(ROWS - 1, r + 1), c), e = at(Math.max(0, r - 1), c);
    const ux = a[0] - b[0], uy = a[1] - b[1], uz = a[2] - b[2], vx = d[0] - e[0], vy = d[1] - e[1], vz = d[2] - e[2];
    let nx = uy * vz - uz * vy, ny = uz * vx - ux * vz, nz = ux * vy - uy * vx;
    if (nx * (p[0] - CENTRE.x) + nz * (p[2] - CENTRE.z) < 0) { nx = -nx; ny = -ny; nz = -nz; }
    const n = Math.hypot(nx, ny, nz) || 1;
    normal.push(nx / n, ny / n, nz / n);
    const u = c / (COLS - 1), v = r / (ROWS - 1);
    uv.push(u, v);
    // Free to move: nothing at the collar, all of it at the hem.
    color.push(1, 1, 1, Math.pow(smooth(0.06, 0.95, v), 1.3));
    // Skinned to the spine by height, the collar's sides to the shoulders.
    const y = p[1];
    const w = [0, 0, 0, 0, 0, 0]; // Hips, Spine02, Spine01, Spine, LeftShoulder, RightShoulder
    if (y >= jointY.Spine) w[3] = 1;
    else if (y >= jointY.Spine01) { const k = (y - jointY.Spine01) / (jointY.Spine - jointY.Spine01); w[3] = k; w[2] = 1 - k; }
    else if (y >= jointY.Spine02) { const k = (y - jointY.Spine02) / (jointY.Spine01 - jointY.Spine02); w[2] = k; w[1] = 1 - k; }
    else if (y >= jointY.Hips) { const k = (y - jointY.Hips) / (jointY.Spine02 - jointY.Hips); w[1] = k; w[0] = 1 - k; }
    else w[0] = 1;
    const side = Math.sin((u * 2 - 1) * SPAN), onShoulder = 0.5 * smooth(0.3, 0.9, Math.abs(side)) * smooth(1.3, 1.4, y);
    for (let i = 0; i < 4; i++) w[i]! *= 1 - onShoulder;
    w[side > 0 ? 4 : 5] = onShoulder;
    const ranked = [J.Hips, J.Spine02, J.Spine01, J.Spine, J.LeftShoulder, J.RightShoulder].map((j, i) => [j, w[i]!] as const).sort((p, q) => q[1] - p[1]).slice(0, 4);
    const sum = ranked.reduce((s, [, x]) => s + x, 0);
    for (const [j, x] of ranked) { joints.push(x > 0 ? j : 0); weights.push(x / sum); }
  }
}
const index: number[] = [];
for (let r = 0; r + 1 < ROWS; r++) {
  for (let c = 0; c + 1 < COLS; c++) {
    const i = r * COLS + c;
    index.push(i, i + 1, i + COLS, i + 1, i + COLS + 1, i + COLS);
  }
}

// ── the cloth ─────────────────────────────────────────────────────────────
const vine: string[] = [];
let line = "";
for (let x = -20; x <= 1044; x += 6) line += `${x},${(806 + 26 * Math.sin(x / 58)).toFixed(1)} `;
vine.push(`polyline ${line}`);
for (let k = 0; k < 9; k++) {
  const cx = 60 + k * 116, up = k % 2 === 0 ? -1 : 1, cy = 806 + up * 58;
  let curl = "";
  for (let a = 0; a <= 3.4 * Math.PI; a += 0.2) curl += `${(cx + (30 - 2.6 * a) * Math.cos(a + k) ).toFixed(1)},${(cy + (30 - 2.6 * a) * Math.sin(a + k) * up).toFixed(1)} `;
  vine.push(`polyline ${curl}`);
}
const cloth = spawnSync("magick", [
  "-seed", "7", "-size", "1024x1024", "xc:#93291f",
  "(", "-size", "1024x1024", "plasma:fractal", "-colorspace", "Gray", "-blur", "0x7", "-auto-level", "-evaluate", "multiply", "0.45", "-evaluate", "add", "55%", ")", "-compose", "Multiply", "-composite",
  "(", "-size", "1024x1024", "xc:gray50", "+noise", "Gaussian", "-colorspace", "Gray", "-motion-blur", "0x3+90", "-auto-level", "-evaluate", "multiply", "0.3", "-evaluate", "add", "70%", ")", "-compose", "Multiply", "-composite",
  "(", "-size", "1024x1024", "gradient:white-#a38d7d", ")", "-compose", "Multiply", "-composite",
  "-fill", "none", "-stroke", "#b7a27a", "-strokewidth", "5", ...vine.flatMap(d => ["-draw", d]),
  "-blur", "0x0.7", "-strip", "-quality", "92", "jpg:-",
], { maxBuffer: 1 << 26 });
if (cloth.status !== 0) throw new Error(`magick: ${cloth.stderr}`);

// ── into the GLB ──────────────────────────────────────────────────────────
const parts: Buffer[] = [bin];
let offset = bin.length;
function view(bytes: Buffer, target?: number): number {
  const pad = (4 - (offset % 4)) % 4;
  if (pad) { parts.push(Buffer.alloc(pad)); offset += pad; }
  json.bufferViews.push({ buffer: 0, byteOffset: offset, byteLength: bytes.length, ...(target ? { target } : {}) });
  parts.push(bytes);
  offset += bytes.length;
  return json.bufferViews.length - 1;
}
function accessor(data: number[], type: string, width: number, componentType: number, bounds = false): number {
  const array = componentType === 5126 ? new Float32Array(data) : componentType === 5123 ? new Uint16Array(data) : new Uint8Array(data);
  const a: any = { bufferView: view(Buffer.from(array.buffer), componentType === 5123 ? 34963 : 34962), componentType, count: data.length / width, type };
  if (bounds) {
    a.min = [0, 1, 2].map(k => Math.min(...data.filter((_, i) => i % 3 === k)));
    a.max = [0, 1, 2].map(k => Math.max(...data.filter((_, i) => i % 3 === k)));
  }
  json.accessors.push(a);
  return json.accessors.length - 1;
}
const attributes = {
  POSITION: accessor(position, "VEC3", 3, 5126, true), NORMAL: accessor(normal, "VEC3", 3, 5126), TEXCOORD_0: accessor(uv, "VEC2", 2, 5126),
  COLOR_0: accessor(color, "VEC4", 4, 5126), JOINTS_0: accessor(joints, "VEC4", 4, 5121), WEIGHTS_0: accessor(weights, "VEC4", 4, 5126),
};
const indices = accessor(index, "SCALAR", 1, 5123);
json.images.push({ bufferView: view(cloth.stdout), mimeType: "image/jpeg", name: "cloak" });
json.textures.push({ source: json.images.length - 1, ...(json.samplers?.length ? { sampler: 0 } : {}) });
json.materials.push({ name: "cloak", doubleSided: true, pbrMetallicRoughness: { baseColorTexture: { index: json.textures.length - 1 }, metallicFactor: 0, roughnessFactor: 0.93 } });
json.meshes.push({ name: "cloak", primitives: [{ attributes, indices, material: json.materials.length - 1 }] });
const body = json.nodes.findIndex((n: any) => n.mesh !== undefined && n.skin !== undefined);
json.nodes.push({ name: "cloak", mesh: json.meshes.length - 1, skin: json.nodes[body].skin });
const parent = json.nodes.find((n: any) => (n.children ?? []).includes(body));
if (parent) parent.children.push(json.nodes.length - 1);
else json.scenes[json.scene ?? 0].nodes.push(json.nodes.length - 1);

const tail = (4 - (offset % 4)) % 4;
if (tail) parts.push(Buffer.alloc(tail));
const data = Buffer.concat(parts);
json.buffers = [{ byteLength: data.length }];
let text = Buffer.from(JSON.stringify(json), "utf8");
if (text.length % 4) text = Buffer.concat([text, Buffer.alloc(4 - (text.length % 4), 0x20)]);
const header = Buffer.alloc(12);
header.writeUInt32LE(0x46546c67, 0); header.writeUInt32LE(2, 4); header.writeUInt32LE(12 + 8 + text.length + 8 + data.length, 8);
const chunk = (length: number, type: number) => { const b = Buffer.alloc(8); b.writeUInt32LE(length, 0); b.writeUInt32LE(type, 4); return b; };
const packed = Buffer.concat([header, chunk(text.length, 0x4e4f534a), text, chunk(data.length, 0x004e4942), data]);
writeFileSync(output, packed);
console.log(`${output}: ${packed.length} bytes sha256 ${createHash("sha256").update(packed).digest("hex")} (cloak: mesh ${json.meshes.length - 1}, ${position.length / 3} points, ${index.length / 3} triangles)`);
