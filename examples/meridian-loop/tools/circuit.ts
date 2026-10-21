// The circuit's models, from src/track.ts's samples:
//
// - models/track.glb: what the car touches. Mesh 0 the road (one
//   primitive, the asphalt), mesh 1 the walls (one primitive): src/world.ts
//   gives each primitive its own triangle-mesh collider, the road gripping
//   and the walls slippery.
// - models/scenery.glb: what it never touches: the embankments under a
//   raised or banked road, the neon tubes along the wall tops (magenta in
//   sector 1, cyan in 2, amber in 3), the start gantry and the two split
//   gates, the skyline and the far ridges.
//
// The ground (a grid plane) is a texture on a plane in src/world.ts.

import { Glb, MeshBuilder, v3 } from "./gltf.ts";
import type { V3 } from "./gltf.ts";
import { draw, fbm, hash2, png, rng } from "./media.ts";
import { SAMPLES, SPLITS, START, WALL_HEIGHT, WALL_THICK, WIDTH, fromStart } from "../src/track.ts";
import type { Sample } from "../src/track.ts";

const N = SAMPLES.length;

const add = v3.add;
const sc = v3.scale;

function centre(s: Sample): V3 {
  return [s.x, s.y, s.z];
}
function right(s: Sample): V3 {
  return [s.rx, s.ry, s.rz];
}
function up(s: Sample): V3 {
  return [s.nx, s.ny, s.nz];
}

/** The road's edge at a sample: side -1 left, 1 right. */
function edge(s: Sample, side: number): V3 {
  return add(centre(s), sc(right(s), (side * WIDTH) / 2));
}

/** The asphalt: dark, a little grain, white edge lines and a dashed centre line (u across the road, v along). */
function asphalt(): Uint8Array {
  return draw(256, (u, v, x, y) => {
    const g = 26 + fbm(x / 9, y / 9, 3, 3, 256 / 9) * 18 + (hash2(x, y, 7) - 0.5) * 10;
    let r = g * 0.92;
    let gg = g * 0.9;
    let b = g * 1.12;
    const line = u < 0.035 || u > 0.965;
    const dash = Math.abs(u - 0.5) < 0.012 && v % 0.5 < 0.25;
    if (line || dash) {
      r = 210;
      gg = 214;
      b = 226;
    }
    return [r, gg, b, 255];
  });
}

/** The wall's face: dark panels with a faint seam every 4 m. */
function panel(): Uint8Array {
  return draw(128, (u, v) => {
    const seam = u < 0.03 || v < 0.04;
    const c = seam ? 50 : 28 + (v > 0.5 ? 6 : 0);
    return [c * 0.8, c * 0.75, c * 1.2, 255];
  });
}

/** Windows on a tower: a dark face with lit cells in pink, cyan and amber. */
function windows(): Uint8Array {
  const r = rng(91);
  const lit: number[] = [];
  for (let i = 0; i < 32 * 32; i++) lit.push(r());
  return draw(256, (u, v) => {
    const cx = Math.floor(u * 32);
    const cy = Math.floor(v * 32);
    const fx = (u * 32) % 1;
    const fy = (v * 32) % 1;
    const inCell = fx > 0.22 && fx < 0.78 && fy > 0.3 && fy < 0.75;
    const p = lit[cy * 32 + cx]!;
    if (inCell && p > 0.55) {
      if (p > 0.92) return [255, 120, 220, 255];
      if (p > 0.8) return [110, 230, 255, 255];
      return [255, 190, 110, 255];
    }
    return [10, 8, 22, 255];
  });
}

/** The gantry's sign: a checker band. */
function checker(): Uint8Array {
  return draw(64, (u, v) => ((Math.floor(u * 8) + Math.floor(v * 2)) % 2 === 0 ? [255, 255, 255, 255] : [12, 10, 20, 255]));
}

export function trackModel(): Uint8Array {
  const glb = new Glb();
  const road = glb.material({ name: "asphalt", pbrMetallicRoughness: { baseColorTexture: { index: glb.texture(asphalt()) }, metallicFactor: 0, roughnessFactor: 0.85 } });
  const wall = glb.material({ name: "wall", pbrMetallicRoughness: { baseColorTexture: { index: glb.texture(panel()) }, metallicFactor: 0.4, roughnessFactor: 0.5 } });

  // The road: a strip between the edges, u across (0 left .. 1 right), v along (a tile every 8 m).
  const pos: number[] = [];
  const nor: number[] = [];
  const uv: number[] = [];
  const idx: number[] = [];
  for (let i = 0; i < N; i++) {
    const s = SAMPLES[i]!;
    const l = edge(s, -1);
    const r = edge(s, 1);
    pos.push(...l, ...r);
    nor.push(s.nx, s.ny, s.nz, s.nx, s.ny, s.nz);
    const v = s.s / 8;
    uv.push(0, v, 1, v);
  }
  for (let i = 0; i < N; i++) {
    const j = (i + 1) % N;
    if (SAMPLES[i]!.gap || SAMPLES[j]!.gap) continue;
    // The last strip wraps to the first sample: its v would run backwards; its own quad repeats the sample with v continuing.
    const a = i * 2;
    const b = j * 2;
    if (j === 0) {
      const base = pos.length / 3;
      const s = SAMPLES[0]!;
      pos.push(...edge(s, -1), ...edge(s, 1));
      nor.push(s.nx, s.ny, s.nz, s.nx, s.ny, s.nz);
      const v = (SAMPLES[N - 1]!.s + (SAMPLES[1]!.s - SAMPLES[0]!.s)) / 8;
      uv.push(0, v, 1, v);
      idx.push(a, a + 1, base, a + 1, base + 1, base);
      continue;
    }
    // Counter-clockwise seen from above: left_i, right_i, left_j.
    idx.push(a, a + 1, b, a + 1, b + 1, b);
  }
  const roadMesh = glb.mesh("road", [{ pos, nor, uv, idx, material: road }]);

  // The walls: a slab on each edge, standing along the road's normal, outward by WALL_THICK.
  const walls = new MeshBuilder();
  for (let i = 0; i < N; i++) {
    const j = (i + 1) % N;
    const a = SAMPLES[i]!;
    const b = SAMPLES[j]!;
    for (const side of [-1, 1]) {
      const a0 = edge(a, side);
      const b0 = edge(b, side);
      const a1 = add(a0, sc(up(a), WALL_HEIGHT));
      const b1 = add(b0, sc(up(b), WALL_HEIGHT));
      const ao = sc(right(a), side * WALL_THICK);
      const bo = sc(right(b), side * WALL_THICK);
      const a2 = add(a1, ao);
      const b2 = add(b1, bo);
      const a3 = add(a0, ao);
      const b3 = add(b0, bo);
      // Faces wound to face the road (inner), up (top) and away (outer).
      if (side > 0) {
        walls.quad(wall, a0, a1, b1, b0, { tile: 4 });
        walls.quad(wall, a1, a2, b2, b1, { tile: 4 });
        walls.quad(wall, a2, a3, b3, b2, { tile: 4 });
      } else {
        walls.quad(wall, a0, b0, b1, a1, { tile: 4 });
        walls.quad(wall, a1, b1, b2, a2, { tile: 4 });
        walls.quad(wall, a2, b2, b3, a3, { tile: 4 });
      }
    }
  }
  const wg = walls.groups.get(wall)!;
  const wallMesh = glb.mesh("walls", [{ pos: wg.pos, nor: wg.nor, uv: wg.uv, idx: wg.idx, material: wall }]);
  glb.node({ name: "road", mesh: roadMesh }, true);
  glb.node({ name: "walls", mesh: wallMesh }, true);
  return glb.bytes();
}

/** The neon tube colour of the sector a sample lies in. */
function sectorColour(i: number): [number, number, number] {
  const k = fromStart(i);
  if (k < SPLITS[0]!) return [1, 0.12, 0.75];
  if (k < SPLITS[1]!) return [0.15, 0.9, 1];
  return [1, 0.62, 0.12];
}

export function sceneryModel(): Uint8Array {
  const glb = new Glb();
  const skirt = glb.material({ name: "skirt", pbrMetallicRoughness: { baseColorFactor: [0.035, 0.03, 0.06, 1], metallicFactor: 0.2, roughnessFactor: 0.8 } });
  // One emissive material per tube colour (vertex colours tint the base colour, never the emission).
  const tubes = new Map<string, number>();
  const tubeOf = (c: [number, number, number]): number => {
    const key = c.join(",");
    let t = tubes.get(key);
    if (t === undefined) {
      t = glb.material({ name: `tube-${tubes.size}`, pbrMetallicRoughness: { baseColorFactor: [c[0], c[1], c[2], 1], metallicFactor: 0, roughnessFactor: 0.4 }, emissiveFactor: c, extensions: { KHR_materials_emissive_strength: { emissiveStrength: 4 } } });
      tubes.set(key, t);
    }
    return t;
  };
  const dark = glb.material({ name: "steel", pbrMetallicRoughness: { baseColorFactor: [0.06, 0.05, 0.1, 1], metallicFactor: 0.8, roughnessFactor: 0.35 } });
  const sign = glb.material({ name: "sign", pbrMetallicRoughness: { baseColorTexture: { index: glb.texture(checker()) }, metallicFactor: 0, roughnessFactor: 0.6 }, emissiveTexture: { index: 0 }, emissiveFactor: [1, 1, 1] });
  const tower = glb.material({ name: "tower", pbrMetallicRoughness: { baseColorFactor: [0.05, 0.04, 0.09, 1], metallicFactor: 0.3, roughnessFactor: 0.6 }, emissiveTexture: { index: glb.texture(windows()) }, emissiveFactor: [1.6, 1.6, 1.6] });
  const ridge = glb.material({ name: "ridge", pbrMetallicRoughness: { baseColorFactor: [0.02, 0.012, 0.04, 1], metallicFactor: 0, roughnessFactor: 1 } });
  glb.json.extensionsUsed.push("KHR_materials_emissive_strength");
  const m = new MeshBuilder();
  const floor = -0.02;
  for (let i = 0; i < N; i++) {
    const j = (i + 1) % N;
    const a = SAMPLES[i]!;
    const b = SAMPLES[j]!;
    for (const side of [-1, 1]) {
      // Under the wall's outer face down to the ground: the embankment.
      const ao = add(edge(a, side), sc(right(a), side * WALL_THICK));
      const bo = add(edge(b, side), sc(right(b), side * WALL_THICK));
      if (ao[1] > 0.05 || bo[1] > 0.05) {
        const ag: V3 = [ao[0], floor, ao[2]];
        const bg: V3 = [bo[0], floor, bo[2]];
        if (side > 0) m.quad(skirt, ao, ag, bg, bo, { tile: 4 });
        else m.quad(skirt, ao, bo, bg, ag, { tile: 4 });
      }
      // The neon tube along the wall's top, outer edge.
      const at = add(add(edge(a, side), sc(up(a), WALL_HEIGHT + 0.01)), sc(right(a), side * WALL_THICK * 0.5));
      const bt = add(add(edge(b, side), sc(up(b), WALL_HEIGHT + 0.01)), sc(right(b), side * WALL_THICK * 0.5));
      const w = sc(right(a), 0.09);
      const wb = sc(right(b), 0.09);
      m.quad(tubeOf(sectorColour(i)), add(at, w), add(bt, wb), add(bt, sc(wb, -1)), add(at, sc(w, -1)));
    }
    // The road's underside where it is raised (the jump's ramps and banks), seen from the pit.
    if ((a.y > 0.3 || b.y > 0.3) && !a.gap && !b.gap) {
      const la = edge(a, -1);
      const lb = edge(b, -1);
      const ra = edge(a, 1);
      const rb = edge(b, 1);
      const d = -0.05;
      m.quad(skirt, add(lb, [0, d, 0]), add(rb, [0, d, 0]), add(ra, [0, d, 0]), add(la, [0, d, 0]), { tile: 4 });
    }
  }
  // The jump's lip and the landing ramp's face: the ends of the road over the gap, down to the ground.
  for (let i = 0; i < N; i++) {
    const a = SAMPLES[i]!;
    const b = SAMPLES[(i + 1) % N]!;
    if (a.gap === b.gap) continue;
    const s = a.gap ? b : a;
    const l = edge(s, -1);
    const r = edge(s, 1);
    m.quad(skirt, l, r, [r[0], floor, r[2]], [l[0], floor, l[2]], { tile: 4 });
    m.quad(skirt, r, l, [l[0], floor, l[2]], [r[0], floor, r[2]], { tile: 4 });
  }

  // The start gantry and the split gates: two posts outside the walls and a beam.
  const gate = (i: number, beam: number, colour: [number, number, number], checkered: boolean) => {
    const s = SAMPLES[i]!;
    const t: V3 = [s.tx, 0, s.tz];
    // Left across the road, so (x, y, z) = (left, up, ahead) is right-handed and the boxes face out.
    const r: V3 = v3.norm([-s.rx, 0, -s.rz]);
    const u: V3 = [0, 1, 0];
    const frame = (o: V3) => ({ o, x: r, y: u, z: v3.norm(t) });
    const half = WIDTH / 2 + WALL_THICK + 0.8;
    const base = Math.min(...[-1, 1].map((sd) => edge(s, sd)[1]));
    for (const sd of [-1, 1]) {
      const o = add([s.x, base - 0.02, s.z], sc(r, sd * half));
      m.box(dark, [-0.3, 0, -0.3], [0.3, beam + 0.9, 0.3], { frame: frame(o), tile: 2 });
      m.box(tubeOf(colour), [-0.32, 0.4, -0.32], [-0.26, beam + 0.6, -0.26], { frame: frame(o) });
    }
    const o = add([s.x, base + beam, s.z], [0, 0, 0]);
    m.box(dark, [-half, 0, -0.35], [half, 0.9, 0.35], { frame: frame(o), tile: 2 });
    if (checkered) m.box(sign, [-half + 0.5, 0.12, -0.38], [half - 0.5, 0.78, -0.36], { frame: frame(o), tile: 2 });
    m.box(tubeOf(colour), [-half, -0.06, -0.37], [half, 0.0, 0.37], { frame: frame(o) });
  };
  gate(START, 6.5, [1, 1, 1], true);
  gate((START + SPLITS[0]!) % N, 6, [0.15, 0.9, 1], false);
  gate((START + SPLITS[1]!) % N, 6, [1, 0.62, 0.12], false);

  // The skyline: towers in a broken ring 380 to 520 m out, and the ridges beyond.
  const r = rng(404);
  for (let k = 0; k < 64; k++) {
    const ang = (k / 64) * Math.PI * 2 + r() * 0.05;
    if (r() < 0.25) continue;
    const dist = 380 + r() * 140;
    const cx = 100 + Math.cos(ang) * dist;
    const cz = Math.sin(ang) * dist;
    const w = 14 + r() * 26;
    const h = 30 + r() * r() * 150;
    m.box(tower, [cx - w / 2, -0.02, cz - w / 2], [cx + w / 2, h, cz + w / 2], { tile: 64, skip: "-y" });
    if (r() < 0.35) m.box(tubeOf([1, 0.12, 0.75]), [cx - w / 2 - 0.2, h, cz - w / 2 - 0.2], [cx + w / 2 + 0.2, h + 0.4, cz + w / 2 + 0.2], { skip: "-y" });
  }
  for (let k = 0; k < 48; k++) {
    const a0 = (k / 48) * Math.PI * 2;
    const a1 = ((k + 1) / 48) * Math.PI * 2;
    const d = 760;
    const h0 = 40 + fbm(k * 0.6, 3, 11, 3, 48 * 0.6) * 110;
    const h1 = 40 + fbm((k + 1) * 0.6, 3, 11, 3, 48 * 0.6) * 110;
    const p0: V3 = [100 + Math.cos(a0) * d, -0.02, Math.sin(a0) * d];
    const p1: V3 = [100 + Math.cos(a1) * d, -0.02, Math.sin(a1) * d];
    m.quad(ridge, p0, p1, [p1[0], h1, p1[2]], [p0[0], h0, p0[2]], { tile: 64 });
  }
  const prims = [...m.groups.entries()].map(([material, g]) => ({ pos: g.pos, nor: g.nor, uv: g.uv, col: g.col, idx: g.idx, material }));
  glb.node({ name: "scenery", mesh: glb.mesh("scenery", prims) }, true);
  return glb.bytes();
}

/** The ground: a dark plane with a glowing grid every 10 m (tiled by the material). */
export function gridTexture(): Uint8Array {
  return draw(256, (u, v) => {
    const du = Math.min(u, 1 - u) * 256;
    const dv = Math.min(v, 1 - v) * 256;
    const d = Math.min(du, dv);
    const glow = Math.max(0, 1 - d / 3) + Math.max(0, 1 - d / 14) * 0.25;
    return [12 + glow * 200, 6 + glow * 40, 26 + glow * 210, 255];
  });
}

/** The mini-map: the road as a light ribbon on transparent, north up, the start line marked. Its bounds are MAP in src/track.ts's terms. */
export function minimap(bounds: { minX: number; minZ: number; size: number }): Uint8Array {
  const size = 256;
  const px = new Float32Array(size * size * 4);
  const toPx = (x: number, z: number): [number, number] => [((x - bounds.minX) / bounds.size) * size, ((bounds.minZ + bounds.size - z) / bounds.size) * size];
  for (let i = 0; i < N; i++) {
    const a = SAMPLES[i]!;
    const b = SAMPLES[(i + 1) % N]!;
    const [x0, y0] = toPx(a.x, a.z);
    const [x1, y1] = toPx(b.x, b.z);
    const steps = 8;
    for (let k = 0; k <= steps; k++) {
      const x = x0 + ((x1 - x0) * k) / steps;
      const y = y0 + ((y1 - y0) * k) / steps;
      for (let dy = -5; dy <= 5; dy++) {
        for (let dx = -5; dx <= 5; dx++) {
          const d = Math.hypot(dx, dy);
          if (d > 4.5) continue;
          const xi = Math.floor(x + dx);
          const yi = Math.floor(y + dy);
          if (xi < 0 || yi < 0 || xi >= size || yi >= size) continue;
          const o = (yi * size + xi) * 4;
          const core = d < 2.4;
          const c = core ? sectorColour(i).map((v) => 120 + v * 135) : [40, 24, 70];
          const aOld = px[o + 3]!;
          const aNew = core ? 255 : 200;
          if (aNew >= aOld) {
            px[o] = c[0]!;
            px[o + 1] = c[1]!;
            px[o + 2] = c[2]!;
            px[o + 3] = aNew;
          }
        }
      }
    }
  }
  return pngOf(size, px);
}

function pngOf(size: number, px: Float32Array): Uint8Array {
  return png(size, size, px, 4);
}

/** The ground: a 1.6 km square of the grid texture, repeating every 10 m (UVs past 1 and a repeating sampler: a plane geometry has no repeat). */
export function groundModel(): Uint8Array {
  const glb = new Glb();
  const grid = glb.material({ name: "grid", pbrMetallicRoughness: { baseColorTexture: { index: glb.texture(gridTexture()) }, metallicFactor: 0, roughnessFactor: 0.9 }, emissiveTexture: { index: 0 }, emissiveFactor: [0.8, 0.8, 0.8] });
  const pos: number[] = [];
  const nor: number[] = [];
  const uv: number[] = [];
  const idx: number[] = [];
  const n = 16;
  const half = 800;
  const cx = 100;
  const y = -0.03;
  for (let j = 0; j <= n; j++) {
    for (let i = 0; i <= n; i++) {
      const x = cx - half + (i / n) * half * 2;
      const z = -half + (j / n) * half * 2;
      pos.push(x, y, z);
      nor.push(0, 1, 0);
      uv.push(x / 10, z / 10);
    }
  }
  for (let j = 0; j < n; j++) {
    for (let i = 0; i < n; i++) {
      const a = j * (n + 1) + i;
      const b = a + 1;
      const c = a + n + 1;
      const d = c + 1;
      // Counter-clockwise from above (+y): a, c, b and b, c, d.
      idx.push(a, c, b, b, c, d);
    }
  }
  glb.node({ name: "ground", mesh: glb.mesh("ground", [{ pos, nor, uv, idx, material: grid }]) }, true);
  return glb.bytes();
}
