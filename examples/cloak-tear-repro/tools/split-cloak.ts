// Splits the traveller's cloak out of the Meshy model into its own skinned
// mesh, so og.Garment can simulate it (assets/models/traveller.glb):
//
//   bun tools/split-cloak.ts <in.glb> <out.glb>
//
// then compress `out.glb` as SOURCE.md says. Needs ImageMagick (`magick`)
// to decode the base colour texture.
//
// Meshy bakes the whole character into one skinned mesh. The cloak is the
// vermilion of the base colour: its region of the texture (the red, closed
// so the cream hem pattern inside it counts too) picks its triangles,
// which become mesh "Cloak", a second skinned mesh on the same skin; the
// rest stays the body. Each cloak vertex's freedom (og.Garment's red ×
// alpha, painted in the alpha over white so the texture keeps its colour)
// is 0 where the cloak meets the body (the seam, and a band along it by
// distance over the cloak), 0 above the shoulder blades (the hood and
// collar stay on the skin; the hood is where the hand holds it), rising
// to 1 by the hips, so the lower cloak swings while its top stays put and
// never opens a gap to the body.

import { execFileSync } from "node:child_process";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Accessor, Document, NodeIO, Primitive } from "@gltf-transform/core@4";
import { ALL_EXTENSIONS } from "@gltf-transform/extensions@4";
import { dequantize } from "@gltf-transform/functions@4";
import { MeshoptDecoder, MeshoptEncoder } from "meshoptimizer@0.22";
import { MeshoptSimplifier } from "meshoptimizer@1";

const [input, output] = process.argv.slice(2);
// Above this share of the height the cloak stays on the skin (freedom 0).
const PIN_LINE = 0.7;
if (!input || !output) throw new Error("usage: bun tools/split-cloak.ts <in.glb> <out.glb>");

await MeshoptDecoder.ready;
await MeshoptEncoder.ready;
await MeshoptSimplifier.ready;
const io = new NodeIO()
  .registerExtensions(ALL_EXTENSIONS)
  .registerDependencies({ "meshopt.decoder": MeshoptDecoder, "meshopt.encoder": MeshoptEncoder });
const doc: Document = await io.read(input);
await doc.transform(dequantize());
const root = doc.getRoot();
const mesh = root.listMeshes()[0]!;
const prim = mesh.listPrimitives()[0]!;
const node = root.listNodes().find((n) => n.getMesh() === mesh)!;
const material = prim.getMaterial()!;

// The base colour as RGB bytes.
const texture = material.getBaseColorTexture()!;
const dir = mkdtempSync(join(tmpdir(), "cloak-"));
const src = join(dir, "base" + (texture.getMimeType() === "image/png" ? ".png" : ".webp"));
writeFileSync(src, texture.getImage()!);
const [tw, th] = texture.getSize()!;
const rgb = execFileSync("magick", [src, "-depth", "8", "rgb:-"], { maxBuffer: 1 << 28 });

// The vermilion: hue within 20° of red, saturated, not near black. Then
// closed (dilate, erode) so the hem's cream pattern inside the cloak's
// region of the atlas counts as cloak.
let mask = new Uint8Array(tw * th);
for (let i = 0; i < tw * th; i++) {
  const r = rgb[i * 3]! / 255, g = rgb[i * 3 + 1]! / 255, b = rgb[i * 3 + 2]! / 255;
  const max = Math.max(r, g, b), min = Math.min(r, g, b);
  const s = max > 0 ? (max - min) / max : 0;
  let h = 0;
  if (max === r && max > min) h = (60 * ((g - b) / (max - min)) + 360) % 360;
  else if (max === g) h = 120;
  else h = 240;
  const red = (h >= 340 || h <= 14) && s > 0.45 && max > 0.12;
  mask[i] = red ? 1 : 0;
}
const morph = (m: Uint8Array, radius: number, grow: boolean): Uint8Array => {
  // Separable box: a texel is set when any (grow) or all (shrink) of its
  // square neighbourhood is.
  const pass = (a: Uint8Array, dx: number, dy: number) => {
    const out = new Uint8Array(a.length);
    for (let y = 0; y < th; y++) {
      for (let x = 0; x < tw; x++) {
        let v = grow ? 0 : 1;
        for (let k = -radius; k <= radius; k++) {
          const xx = Math.min(tw - 1, Math.max(0, x + k * dx));
          const yy = Math.min(th - 1, Math.max(0, y + k * dy));
          const t = a[yy * tw + xx]!;
          v = grow ? v | t : v & t;
        }
        out[y * tw + x] = v;
      }
    }
    return out;
  };
  return pass(pass(m, 1, 0), 0, 1);
};
mask = morph(morph(mask, 6, true), 6, false);

const pos = prim.getAttribute("POSITION")!;
const uv = prim.getAttribute("TEXCOORD_0")!;
const indices = prim.getIndices()!;
const count = pos.getCount();
const at = (a: Accessor, i: number) => a.getElement(i, []);
const inMask = (i: number) => {
  const [u, v] = at(uv, i) as [number, number];
  const x = Math.min(tw - 1, Math.max(0, Math.floor((u - Math.floor(u)) * tw)));
  const y = Math.min(th - 1, Math.max(0, Math.floor((v - Math.floor(v)) * th)));
  return mask[y * tw + x] === 1;
};

// Triangles: cloak when at least two of its corners are.
const tri = indices.getArray()!;
let cloakTris: number[] = [];
const bodyTris: number[] = [];
for (let t = 0; t < tri.length; t += 3) {
  const n = [tri[t]!, tri[t + 1]!, tri[t + 2]!].filter(inMask).length;
  (n >= 2 ? cloakTris : bodyTris).push(tri[t]!, tri[t + 1]!, tri[t + 2]!);
}

// Vertices at the same position (Meshy splits them along UV seams) are
// one point of the cloth.
const key = (q: number[]) => q.map((v) => v.toFixed(5)).join(",");
const pointOf = new Int32Array(count);
{
  const ids = new Map<string, number>();
  for (let i = 0; i < count; i++) {
    const k = key(at(pos, i));
    if (!ids.has(k)) ids.set(k, ids.size);
    pointOf[i] = ids.get(k)!;
  }
}

// Only the cloak itself: red specks elsewhere (cuffs, trims), islands
// apart from it, go back to the body.
{
  const parent = new Int32Array(count).map((_, i) => i);
  const find = (x: number): number => {
    while (parent[x] !== x) x = parent[x] = parent[parent[x]!]!;
    return x;
  };
  const union = (x: number, y: number) => {
    parent[find(x)] = find(y);
  };
  const firstOfPoint = new Map<number, number>();
  for (const i of cloakTris) {
    const f = firstOfPoint.get(pointOf[i]!);
    if (f === undefined) firstOfPoint.set(pointOf[i]!, i);
    else union(i, f);
  }
  for (let t = 0; t < cloakTris.length; t += 3) {
    union(cloakTris[t]!, cloakTris[t + 1]!);
    union(cloakTris[t]!, cloakTris[t + 2]!);
  }
  const size = new Map<number, number>();
  for (let t = 0; t < cloakTris.length; t += 3) size.set(find(cloakTris[t]!), (size.get(find(cloakTris[t]!)) ?? 0) + 1);
  const keep = cloakTris.length / 3 * 0.02;
  const kept: number[] = [];
  for (let t = 0; t < cloakTris.length; t += 3) {
    const triangle = [cloakTris[t]!, cloakTris[t + 1]!, cloakTris[t + 2]!];
    ((size.get(find(cloakTris[t]!)) ?? 0) >= keep ? kept : bodyTris).push(...triangle);
  }
  cloakTris = kept;
}

// Strips the colour misses along the cloak's edges (the hem's trim, its
// rim) are body triangles that only the cloak holds: islands of the body
// apart from its main piece that touch the cloak. They are the cloak's
// (left on the body they stay rigid while the cloth moves, and their
// points would pin the cloak's whole outline as its seam).
{
  const parent = new Int32Array(count).map((_, i) => i);
  const find = (x: number): number => {
    while (parent[x] !== x) x = parent[x] = parent[parent[x]!]!;
    return x;
  };
  const head = new Map<number, number>();
  const join = (i: number) => {
    const h = head.get(pointOf[i]!);
    if (h === undefined) head.set(pointOf[i]!, i);
    else parent[find(i)] = find(h);
  };
  for (const i of bodyTris) join(i);
  for (let t = 0; t < bodyTris.length; t += 3) {
    parent[find(bodyTris[t + 1]!)] = find(bodyTris[t]!);
    parent[find(bodyTris[t + 2]!)] = find(bodyTris[t]!);
  }
  const size = new Map<number, number>();
  for (let t = 0; t < bodyTris.length; t += 3) size.set(find(bodyTris[t]!), (size.get(find(bodyTris[t]!)) ?? 0) + 1);
  const main = [...size.entries()].sort((a, b) => b[1] - a[1])[0]![0];
  const cloakPoint = new Set(cloakTris.map((i) => pointOf[i]!));
  const touches = new Set<number>();
  for (const i of bodyTris) if (cloakPoint.has(pointOf[i]!)) touches.add(find(i));
  const kept: number[] = [];
  let moved = 0;
  for (let t = 0; t < bodyTris.length; t += 3) {
    const c = find(bodyTris[t]!);
    const triangle = [bodyTris[t]!, bodyTris[t + 1]!, bodyTris[t + 2]!];
    if (c !== main && touches.has(c)) {
      cloakTris.push(...triangle);
      moved++;
    } else kept.push(...triangle);
  }
  bodyTris.length = 0;
  bodyTris.push(...kept);
  console.log(`moved ${moved} rim triangles from the body to the cloak`);
}

// The character's height (the bind pose's y).
let ymin = Infinity, ymax = -Infinity;
for (let i = 0; i < count; i++) {
  const y = at(pos, i)[1]!;
  ymin = Math.min(ymin, y);
  ymax = Math.max(ymax, y);
}
const height = ymax - ymin;

// The cloak's vertices, and where it meets the body: points any body
// triangle uses.
const bodyPoint = new Uint8Array(count);
for (const i of bodyTris) bodyPoint[pointOf[i]!] = 1;
{
  const hist = new Array(10).fill(0);
  const seen = new Set<number>();
  for (const i of cloakTris) if (bodyPoint[pointOf[i]!] && !seen.has(pointOf[i]!)) { seen.add(pointOf[i]!); hist[Math.min(9, Math.floor(((at(pos, i)[1]! - ymin) / height) * 10))]++; }
  console.log("seam points by tenth of height", hist.join(" "));
}
let used = new Map<number, number>();
for (const i of cloakTris) if (!used.has(i)) used.set(i, used.size);
let local = [...used.keys()];

const valence = (tris: number[]) => {
  const e = new Map<string, number>();
  for (let t = 0; t < tris.length; t += 3)
    for (let k = 0; k < 3; k++) {
      const a = pointOf[tris[t + k]!]!, b = pointOf[tris[t + ((k + 1) % 3)]!]!;
      const id = a < b ? `${a}_${b}` : `${b}_${a}`;
      e.set(id, (e.get(id) ?? 0) + 1);
    }
  const h: Record<number, number> = {};
  for (const v of e.values()) h[v] = (h[v] ?? 0) + 1;
  // Open boundary (not the seam to the body): loops by size.
  const adj = new Map<number, number[]>();
  for (const [id, v] of e) if (v === 1) { const [a, b] = id.split("_").map(Number) as [number, number]; if (bodyPoint[a] && bodyPoint[b]) continue; adj.set(a, [...(adj.get(a) ?? []), b]); adj.set(b, [...(adj.get(b) ?? []), a]); }
  const seen = new Set<number>(); const loops: number[] = [];
  for (const s0 of adj.keys()) { if (seen.has(s0)) continue; let c = 0; const st = [s0]; seen.add(s0); while (st.length) { const x = st.pop()!; c++; for (const y of adj.get(x)!) if (!seen.has(y)) { seen.add(y); st.push(y); } } loops.push(c); }
  loops.sort((a, b) => b - a);
  return JSON.stringify(h) + " open loops " + loops.length + " sizes " + loops.slice(0, 12).join(",");
};
console.log("cloak edge valence before simplifying", valence(cloakTris));

// Simplified: the cloth is simulated point by point, so a quarter of
// Meshy's dense remesh (its folds and edges kept; across UV seams too,
// weighing the UVs so the texture stays put), the seam to the body locked
// so no crack opens.
{
  const n0 = local.length;
  const positions = new Float32Array(n0 * 3);
  const attributes = new Float32Array(n0 * 5);
  const lock = new Uint8Array(n0);
  const normal = prim.getAttribute("NORMAL")!;
  local.forEach((i, k) => {
    positions.set(at(pos, i), k * 3);
    attributes.set([...(at(uv, i) as number[]).slice(0, 2), ...(at(normal, i) as number[]).slice(0, 3).map((v) => v * 0.3)], k * 5);
    lock[k] = bodyPoint[pointOf[i]!]!;
  });
  const localTris = new Uint32Array(cloakTris.map((i) => used.get(i)!));
  const target = Math.floor((localTris.length * Number(process.env.SPLIT_CLOAK_KEEP ?? 0.25)) / 3) * 3;
  const [simplified, error] = MeshoptSimplifier.simplifyWithAttributes(
    localTris, positions, 3, attributes, 5, [1, 1, 1, 1, 1], lock, target, 0.01, ["Permissive"],
  );
  console.log(`simplified ${localTris.length / 3} to ${simplified.length / 3} triangles, error ${error.toFixed(4)}`);
  cloakTris = [...simplified].map((k) => local[k]!);
  console.log("cloak edge valence after simplifying", valence(cloakTris));
  used = new Map<number, number>();
  for (const i of cloakTris) if (!used.has(i)) used.set(i, used.size);
  local = [...used.keys()];
}
const n = local.length;
const p = local.map((i) => at(pos, i));

// Distance over the cloak from the seam (Dijkstra over its edges; the
// split vertices of one point joined at no cost).
const edges: Array<Array<[number, number]>> = Array.from({ length: n }, () => []);
const len = (a: number, b: number) => Math.hypot(p[a]![0]! - p[b]![0]!, p[a]![1]! - p[b]![1]!, p[a]![2]! - p[b]![2]!);
for (let t = 0; t < cloakTris.length; t += 3) {
  const [a, b, c] = [used.get(cloakTris[t]!)!, used.get(cloakTris[t + 1]!)!, used.get(cloakTris[t + 2]!)!];
  for (const [x, y] of [[a, b], [b, c], [c, a]] as const) {
    edges[x]!.push([y, len(x, y)]);
    edges[y]!.push([x, len(x, y)]);
  }
}
const byPoint = new Map<number, number[]>();
local.forEach((i, k) => byPoint.set(pointOf[i]!, [...(byPoint.get(pointOf[i]!) ?? []), k]));
for (const group of byPoint.values()) for (const a of group) for (const b of group) if (a !== b) edges[a]!.push([b, 0]);
const dist = new Float64Array(n).fill(Infinity);
const heap: Array<[number, number]> = [];
const push = (d: number, k: number) => {
  heap.push([d, k]);
  let i = heap.length - 1;
  while (i > 0) {
    const up = (i - 1) >> 1;
    if (heap[up]![0] <= heap[i]![0]) break;
    [heap[up], heap[i]] = [heap[i]!, heap[up]!];
    i = up;
  }
};
const pop = (): [number, number] => {
  const top = heap[0]!;
  const last = heap.pop()!;
  if (heap.length) {
    heap[0] = last;
    let i = 0;
    for (;;) {
      const l = 2 * i + 1, r = l + 1;
      let m = i;
      if (l < heap.length && heap[l]![0] < heap[m]![0]) m = l;
      if (r < heap.length && heap[r]![0] < heap[m]![0]) m = r;
      if (m === i) break;
      [heap[m], heap[i]] = [heap[i]!, heap[m]!];
      i = m;
    }
  }
  return top;
};
local.forEach((i, k) => {
  if (bodyPoint[pointOf[i]!]) {
    dist[k] = 0;
    push(0, k);
  }
});
while (heap.length) {
  const [d, k] = pop();
  if (d > dist[k]!) continue;
  for (const [j, w] of edges[k]!) {
    if (d + w < dist[j]!) {
      dist[j] = d + w;
      push(d + w, j);
    }
  }
}
const smooth = (a: number, b: number, x: number) => {
  const t = Math.min(1, Math.max(0, (x - a) / (b - a)));
  return t * t * (3 - 2 * t);
};
const freedom = p.map((q, k) => {
  const h = (q[1]! - ymin) / height;
  // (A part the seam never reaches stays held: Infinity is not free.)
  const fromSeam = Number.isFinite(dist[k]!) ? smooth(0.03 * height, 0.12 * height, dist[k]!) : 0;
  const below = 1 - smooth(0.5, PIN_LINE, h);
  return fromSeam * below;
});

// The cloak's own vertex set and attributes; COLOR_0 white with the freedom in alpha.
const buffer = root.listBuffers()[0]!;
const copy = (name: string): Accessor | null => {
  const a = prim.getAttribute(name);
  if (!a) return null;
  const size = a.getElementSize();
  const Out = a.getArray()!.constructor as { new (n: number): Float32Array };
  const out = new Out(n * size);
  local.forEach((i, k) => out.set(a.getElement(i, []), k * size));
  return doc.createAccessor(name).setType(a.getType()).setArray(out).setNormalized(a.getNormalized()).setBuffer(buffer);
};
const cloakPrim: Primitive = doc.createPrimitive().setMaterial(material);
for (const name of prim.listSemantics()) {
  const a = copy(name);
  if (a) cloakPrim.setAttribute(name, a);
}
const colors = new Float32Array(n * 4);
// (SPLIT_CLOAK_DEBUG=1: the freedom shown in the colour, red free, blue held.)
const debug = process.env.SPLIT_CLOAK_DEBUG === "1";
freedom.forEach((f, k) => colors.set(debug ? [0.2 + 0.8 * f, 0.15, 1 - f, f] : [1, 1, 1, f], k * 4));
cloakPrim.setAttribute("COLOR_0", doc.createAccessor("COLOR_0").setType("VEC4").setArray(colors).setBuffer(buffer));
cloakPrim.setIndices(
  doc
    .createAccessor("cloak indices")
    .setType("SCALAR")
    .setArray(new Uint32Array(cloakTris.map((i) => used.get(i)!)))
    .setBuffer(buffer),
);
prim.setIndices(doc.createAccessor("body indices").setType("SCALAR").setArray(new Uint32Array(bodyTris)).setBuffer(buffer));
mesh.setName("Body");
const cloakMesh = doc.createMesh("Cloak").addPrimitive(cloakPrim);
const cloakNode = doc.createNode("Cloak").setMesh(cloakMesh).setSkin(node.getSkin());
for (const parent of node.listParents()) {
  if (parent.propertyType === "Node" || parent.propertyType === "Scene") (parent as unknown as { addChild(n: unknown): void }).addChild(cloakNode);
}
await io.write(output, doc);

const free = freedom.filter((f) => f > 0.5).length;
console.log(
  `cloak: ${cloakTris.length / 3} triangles, ${n} vertices (${free} more than half free); body: ${bodyTris.length / 3} triangles; height ${height.toFixed(3)}`,
);
const points = new Set<string>();
const cp = cloakPrim.getAttribute("POSITION")!;
for (let i = 0; i < cp.getCount(); i++) points.add(key(cp.getElement(i, [])));
console.log(`cloth points (welded by position): ${points.size}`);
console.log(`meshes: Body (glTF mesh 0), Cloak (glTF mesh ${root.listMeshes().indexOf(cloakMesh)})`);

