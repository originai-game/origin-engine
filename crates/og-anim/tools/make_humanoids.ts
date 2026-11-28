// CC0 retargeting fixtures: Blender clip library, Mixamo strider, Unreal
// stout and a side-on puppet. bun crates/og-anim/tools/make_humanoids.ts [root]
import { join } from "node:path";
import { Gltf, FACES, cross, bounds } from "../../../scripts/gltf.ts";
const ROOT = process.argv[2] ?? "tests/fixtures/motion",
  TAU = 2 * Math.PI;
type V = number[];
const add = (a: V, b: V) => a.map((x, i) => x + b[i]!);
const sub = (a: V, b: V) => a.map((x, i) => x - b[i]!);
const scale = (a: V, s: number) => a.map((x) => x * s);
const dot = (a: V, b: V) => a.reduce((s, x, i) => s + x * b[i]!, 0);
const norm = (a: V) => scale(a, 1 / Math.sqrt(dot(a, a)));
const qconj = (q: V) => [-q[0]!, -q[1]!, -q[2]!, q[3]!];
function qmul(a: V, b: V): V {
  const [ax, ay, az, aw] = a as [number, number, number, number],
    [bx, by, bz, bw] = b as [number, number, number, number];
  return [
    aw * bx + ax * bw + ay * bz - az * by,
    aw * by - ax * bz + ay * bw + az * bx,
    aw * bz + ax * by - ay * bx + az * bw,
    aw * bw - ax * bx - ay * by - az * bz,
  ];
}
const qrot = (q: V, v: V) => qmul(qmul(q, [...v, 0]), qconj(q)).slice(0, 3);
const qaxis = (axis: V, angle: number) => [...scale(axis, Math.sin(angle / 2)), Math.cos(angle / 2)];
function qarc(a: V, b: V): V {
  a = norm(a);
  b = norm(b);
  const d = dot(a, b);
  return d < -0.999999 ? [...norm(cross(a, Math.abs(a[2]!) < 0.9 ? [0, 0, 1] : [1, 0, 0])), 0] : norm([...cross(a, b), 1 + d]);
}
const r6 = (v: V) => v.map((x) => (Math.abs(x) < 5e-7 ? 0 : Number(x.toFixed(6))));
class Rig {
  names: string[] = [];
  parents: (number | null)[] = [];
  pos: V[] = [];
  rot: V[] = [];
  constructor(readonly rootScale: number) {}
  node(name: string, parent: number | null, pos: V, rot = [0, 0, 0, 1]): number {
    this.names.push(name);
    this.parents.push(parent);
    this.pos.push(pos);
    this.rot.push(rot);
    return this.names.length - 1;
  }
  local(i: number): [V, V] {
    const p = this.parents[i];
    if (p == null)
      return [
        [0, 0, 0],
        [0, 0, 0, 1],
      ];
    if (this.parents[p] == null) return [scale(this.pos[i]!, 1 / this.rootScale), this.rot[i]!];
    const inv = qconj(this.rot[p]!);
    return [scale(qrot(inv, sub(this.pos[i]!, this.pos[p]!)), 1 / this.rootScale), qmul(inv, this.rot[i]!)];
  }
  inverseBind(i: number): V {
    const s = this.rootScale,
      r = qconj(this.rot[i]!),
      [x, y, z, w] = r as [number, number, number, number];
    const m = [
      [1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
      [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
      [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)],
    ];
    return r6([
      ...Array.from({ length: 3 }, (_, c) => [m[0]![c]! / s, m[1]![c]! / s, m[2]![c]! / s, 0]).flat(),
      ...scale(qrot(r, this.pos[i]!), -1 / s),
      1,
    ]);
  }
}
type Bones = Record<string, number>;
type Positions = Record<string, V>;
type Links = Record<string, string>;
function humanoid(
  r: Rig,
  style: string,
  hipsY: number,
  thigh: number,
  shin: number,
  ankle: number,
  hipW: number,
  spine: number,
  chest: number,
  neck: number,
  head: number,
  shoulderW: number,
  upperArm: number,
  forearm: number,
  hand: number,
  armDown: number,
  names: Links,
  rootName: string,
): [Bones, Positions, Links] {
  const frame = (d: V) => (style === "blender" ? [0, 0, 0, 1] : qarc(style === "mixamo" ? [0, 1, 0] : [1, 0, 0], d));
  const root = r.node(rootName, null, [0, 0, 0]),
    pos: Positions = {
      hips: [0, hipsY, 0],
      spine: [0, hipsY + spine, 0],
      chest: [0, hipsY + spine + chest, 0],
      neck: [0, hipsY + spine + chest + neck, 0],
      head: [0, hipsY + spine + chest + neck + head * 0.3, 0],
    };
  const down = (armDown * Math.PI) / 180;
  for (const [side, sx] of [
    ["L", 1],
    ["R", -1],
  ] as const) {
    const sh = [sx * shoulderW * 0.35, pos.neck![1]! - 0.04, 0],
      ua = [sx * shoulderW, sh[1]!, 0],
      d = [sx * Math.cos(down), -Math.sin(down), 0];
    Object.assign(pos, {
      [`shoulder.${side}`]: sh,
      [`upper_arm.${side}`]: ua,
      [`forearm.${side}`]: add(ua, scale(d, upperArm)),
      [`hand.${side}`]: add(ua, scale(d, upperArm + forearm)),
      [`hand_end.${side}`]: add(ua, scale(d, upperArm + forearm + hand)),
      [`thigh.${side}`]: [sx * hipW, hipsY, 0],
      [`shin.${side}`]: [sx * hipW, hipsY - thigh, 0],
      [`foot.${side}`]: [sx * hipW, hipsY - thigh - shin, 0],
      [`toe.${side}`]: [sx * hipW, ankle * 0.25, ankle * 1.1],
      [`toe_end.${side}`]: [sx * hipW, ankle * 0.25, ankle * 1.8],
    });
  }
  const child: Links = { hips: "spine", spine: "chest", chest: "neck", neck: "head" },
    parent: Record<string, string | null> = { hips: null, spine: "hips", chest: "spine", neck: "chest", head: "neck" };
  for (const s of "LR") {
    for (const chain of [
      ["shoulder", "upper_arm", "forearm", "hand", "hand_end"],
      ["thigh", "shin", "foot", "toe", "toe_end"],
    ])
      for (let i = 0; i + 1 < chain.length; i++) child[`${chain[i]}.${s}`] = `${chain[i + 1]}.${s}`;
    for (const chain of [
      ["chest", `shoulder.${s}`, `upper_arm.${s}`, `forearm.${s}`, `hand.${s}`],
      ["hips", `thigh.${s}`, `shin.${s}`, `foot.${s}`, `toe.${s}`],
    ])
      for (let i = 1; i < chain.length; i++) parent[chain[i]!] = chain[i - 1]!;
  }
  const order = [
      "hips",
      "spine",
      "chest",
      "neck",
      "head",
      ...[..."LR"].flatMap((s) => ["shoulder", "upper_arm", "forearm", "hand"].map((b) => `${b}.${s}`)),
      ...[..."LR"].flatMap((s) => ["thigh", "shin", "foot", "toe"].map((b) => `${b}.${s}`)),
    ],
    index: Bones = {};
  for (const b of order)
    index[b] = r.node(
      names[b]!,
      parent[b] == null ? root : index[parent[b]!]!,
      pos[b]!,
      b === "hips" ? [0, 0, 0, 1] : frame(b === "head" ? [0, 1, 0] : sub(pos[child[b]!]!, pos[b]!)),
    );
  return [index, pos, child];
}
const BLENDER: Links = Object.fromEntries(["hips", "spine", "chest", "neck", "head"].map((b) => [b, b]));
const MIXAMO: Links = {
    hips: "mixamorig:Hips",
    spine: "mixamorig:Spine",
    chest: "mixamorig:Spine1",
    neck: "mixamorig:Neck",
    head: "mixamorig:Head",
  },
  UE: Links = { hips: "pelvis", spine: "spine_01", chest: "spine_02", neck: "neck_01", head: "head" };
for (const [s, side] of [
  ["L", "Left"],
  ["R", "Right"],
]) {
  for (const [b, mix, ue] of [
    ["shoulder", "Shoulder", "clavicle"],
    ["upper_arm", "Arm", "upperarm"],
    ["forearm", "ForeArm", "lowerarm"],
    ["hand", "Hand", "hand"],
    ["thigh", "UpLeg", "thigh"],
    ["shin", "Leg", "calf"],
    ["foot", "Foot", "foot"],
    ["toe", "ToeBase", "ball"],
  ]) {
    BLENDER[`${b}.${s}`] = `${b}.${s}`;
    MIXAMO[`${b}.${s}`] = `mixamorig:${side}${mix}`;
    UE[`${b}.${s}`] = `${ue}_${s!.toLowerCase()}`;
  }
}
const asset = { version: "2.0", generator: "og-anim make_humanoids.ts", copyright: "CC0 1.0, Origin Engine authors" };
function nodesOf(r: Rig, extra: Record<number, number[]> = {}): any[] {
  return r.names.map((name, i) => {
    let [t, q] = r.local(i);
    t = r6(t);
    q = r6(q);
    const children = [...r.parents.flatMap((p, j) => (p === i ? [j] : [])), ...(extra[i] ?? [])];
    return {
      name,
      ...(t.some(Boolean) ? { translation: t } : {}),
      ...(q.some((v, k) => v !== [0, 0, 0, 1][k]) ? { rotation: q } : {}),
      ...(r.parents[i] == null && r.rootScale !== 1 ? { scale: Array(3).fill(r.rootScale) } : {}),
      ...(children.length ? { children } : {}),
    };
  });
}
class Mesh {
  P: V = [];
  N: V = [];
  J: V = [];
  W: V = [];
  I: V = [];
  box(center: V, half: V, rot: V, joint: number): void {
    for (const [n, u, v] of FACES) {
      const base = this.P.length / 3;
      for (const [su, sv] of [
        [-1, -1],
        [1, -1],
        [1, 1],
        [-1, 1],
      ]) {
        const local = half.map((h, k) => h * (n![k]! + su! * u![k]! + sv! * v![k]!));
        this.P.push(...r6(add(center, qrot(rot, local))));
        this.N.push(...r6(qrot(rot, n!)));
        this.J.push(joint, 0, 0, 0);
        this.W.push(1, 0, 0, 0);
      }
      const [a, b, c, d] = [base, base + 1, base + 2, base + 3];
      this.I.push(...(dot(cross(u!, v!), n!) > 0 ? [a!, b!, c!, a!, c!, d!] : [a!, c!, b!, a!, d!, c!]));
    }
  }
  primitive(g: Gltf, material = 0): any {
    const nv = this.P.length / 3;
    return {
      attributes: {
        POSITION: g.add(this.P, "f", nv, 5126, "VEC3", bounds(this.P), 34962),
        NORMAL: g.add(this.N, "f", nv, 5126, "VEC3", null, 34962),
        JOINTS_0: g.add(this.J, "H", nv, 5123, "VEC4", null, 34962),
        WEIGHTS_0: g.add(this.W, "f", nv, 5126, "VEC4", null, 34962),
      },
      indices: g.add(this.I, "I", this.I.length, 5125, "SCALAR", null, 34963),
      material,
    };
  }
}
const material = (name: string, color: V, roughnessFactor = 0.7) => ({
  name,
  pbrMetallicRoughness: { baseColorFactor: color, metallicFactor: 0, roughnessFactor },
});
function character(
  path: string,
  r: Rig,
  index: Bones,
  pos: Positions,
  child: Links,
  color: V,
  widths: Record<string, number>,
  title: string,
): void {
  const g = new Gltf(),
    joints = r.names.slice(1).map((_, i) => i + 1),
    mesh = new Mesh();
  for (const [b, node] of Object.entries(index)) {
    if (b === "head") {
      mesh.box(add(pos.head!, [0, widths.head! * 0.9, 0]), Array(3).fill(widths.head), [0, 0, 0, 1], node - 1);
      continue;
    }
    if (!child[b]) continue;
    const a = pos[b]!,
      d = sub(pos[child[b]!]!, a),
      length = Math.sqrt(dot(d, d)),
      w = widths[b.split(".")[0]!] ?? 0.05;
    mesh.box(add(a, scale(d, 0.5)), [w, length * 0.5, w * (b.includes("foot") ? 1.2 : 0.8)], qarc([0, 1, 0], d), node - 1);
  }
  const primitive = mesh.primitive(g),
    ibm = g.add(
      joints.flatMap((n) => r.inverseBind(n)),
      "f",
      joints.length,
      5126,
      "MAT4",
    );
  const size = g.write(path, {
    asset,
    scene: 0,
    scenes: [{ name: title, nodes: [0] }],
    nodes: [...nodesOf(r, { 0: [r.names.length] }), { name: "body", mesh: 0, skin: 0 }],
    meshes: [{ name: title, primitives: [primitive] }],
    materials: [material(title, color)],
    skins: [{ name: title, joints, inverseBindMatrices: ibm, skeleton: joints[0] }],
  });
  console.log(`${path}: ${size} bytes, ${r.names.length} nodes, ${mesh.P.length / 3} vertices`);
}
type Chan = [number, string, (t: number) => V];
function clip(g: Gltf, name: string, duration: number, rate: number, chans: Chan[], events?: [string, number][]): any {
  const samplers: any[] = [],
    channels: any[] = [],
    n = Math.round(duration * rate) + 1,
    times = Array.from({ length: n }, (_, i) => i / rate);
  for (const [node, path, f] of chans) {
    const vals = times.flatMap((t) => r6(f(t))),
      ti = g.add(times, "f", n, 5126, "SCALAR", [[Math.min(...times)], [Math.max(...times)]]),
      vi = g.add(vals, "f", n, 5126, f(0).length === 3 ? "VEC3" : "VEC4");
    samplers.push({ input: ti, output: vi, interpolation: "LINEAR" });
    channels.push({ sampler: samplers.length - 1, target: { node, path } });
  }
  return { name, samplers, channels, ...(events ? { extras: { "og.events": events.map(([name, time]) => ({ name, time })) } } : {}) };
}
function library(path: string): void {
  const r = new Rig(1),
    [index] = humanoid(r, "blender", 1, 0.45, 0.45, 0.1, 0.15, 0.25, 0.2, 0.18, 0.2, 0.22, 0.28, 0.26, 0.08, 0, BLENDER, "Rig"),
    g = new Gltf(),
    animations: any[] = [],
    X = [1, 0, 0],
    Y = [0, 1, 0],
    Z = [0, 0, 1];
  function gait(
    name: string,
    duration: number,
    rate: number,
    stride: number,
    stance: number,
    lift: number,
    height: number,
    bob: number,
    arm: number,
    events: [string, number][],
  ): void {
    const hipsY = (t: number) => height + bob * Math.cos(4 * Math.PI * (t / duration - stance / 2));
    function leg(t: number, offset: number): V[] {
      const u = (((t / duration + offset) % 1) + 1) % 1,
        cycleStart = t - u * duration,
        planted = (stride * (cycleStart + (stance / 2) * duration)) / duration,
        s = (u - stance) / (1 - stance),
        e = s * s * (3 - 2 * s);
      const fz = u < stance ? planted : planted + stride * e,
        fy = u < stance ? 0.1 : 0.1 + lift * Math.sin(Math.PI * s),
        vz = fz - (stride * t) / duration,
        vy = fy - hipsY(t),
        d = Math.min(Math.hypot(vz, vy), 0.9 - 1e-5),
        psi = Math.atan2(vz, -vy);
      const beta = Math.acos(Math.max(-1, Math.min(1, (0.45 * 0.45 + d * d - 0.45 * 0.45) / (2 * 0.45 * d)))),
        gamma = Math.acos(Math.max(-1, Math.min(1, (0.45 * 0.45 + 0.45 * 0.45 - d * d) / (2 * 0.45 * 0.45)))),
        thigh = psi + beta,
        knee = Math.PI - gamma;
      return [qaxis(X, -thigh), qaxis(X, knee), qaxis(X, thigh - knee)];
    }
    const ph = (t: number) => (t / duration) * TAU,
      chans: Chan[] = [
        [index.hips!, "translation", (t) => [0, hipsY(t), (stride * t) / duration]],
        [index.chest!, "rotation", (t) => qaxis(Y, -0.1 * Math.sin(ph(t)))],
        [index["upper_arm.L"]!, "rotation", (t) => qmul(qaxis(X, arm * Math.sin(ph(t))), qaxis(Z, -1.3))],
        [index["upper_arm.R"]!, "rotation", (t) => qmul(qaxis(X, -arm * Math.sin(ph(t))), qaxis(Z, 1.3))],
        [index["forearm.L"]!, "rotation", () => qaxis(Y, -0.3)],
        [index["forearm.R"]!, "rotation", () => qaxis(Y, 0.3)],
      ];
    for (const [side, offset] of [
      ["L", 0],
      ["R", 0.5],
    ] as const)
      ["thigh", "shin", "foot"].forEach((part, k) => chans.push([index[`${part}.${side}`]!, "rotation", (t) => leg(t, offset)[k]!]));
    animations.push(clip(g, name, duration, rate, chans, events));
  }
  gait("walk", 1, 30, 1.2, 0.6, 0.12, 0.915, 0.015, 0.35, [
    ["footL", 0],
    ["footR", 0.5],
  ]);
  gait("run", 0.66, 30, 1.8, 0.45, 0.2, 0.88, 0.02, 0.7, [
    ["footL", 0],
    ["footR", 0.33],
  ]);
  animations.push(
    clip(g, "idle", 2, 15, [
      [index.hips!, "translation", (t) => [0, 1 - 0.01 * Math.sin((t * TAU) / 2), 0]],
      [index.chest!, "rotation", (t) => qaxis(X, 0.03 * Math.sin((t * TAU) / 2))],
      [index["upper_arm.L"]!, "rotation", () => qaxis(Z, -1.3)],
      [index["upper_arm.R"]!, "rotation", () => qaxis(Z, 1.3)],
    ]),
  );
  console.log(
    `${path}: ${g.write(path, { asset, scene: 0, scenes: [{ name: "humanoid-clips", nodes: [0] }], nodes: nodesOf(r), animations })} bytes, ${r.names.length} nodes, ${animations.length} clips`,
  );
}
library(join(ROOT, "assets/anims/humanoid-clips.glb"));
const strider = new Rig(0.01),
  [si, sp, sc] = humanoid(
    strider,
    "mixamo",
    1.3,
    0.7,
    0.44,
    0.16,
    0.12,
    0.2,
    0.15,
    0.2,
    0.24,
    0.24,
    0.32,
    0.28,
    0.1,
    45,
    MIXAMO,
    "Armature",
  );
character(
  join(ROOT, "assets/models/strider.glb"),
  strider,
  si,
  sp,
  sc,
  [0.22, 0.45, 0.8, 1],
  {
    hips: 0.12,
    spine: 0.13,
    chest: 0.15,
    neck: 0.05,
    head: 0.12,
    shoulder: 0.05,
    upper_arm: 0.05,
    forearm: 0.045,
    hand: 0.04,
    thigh: 0.07,
    shin: 0.055,
    foot: 0.05,
    toe: 0.045,
  },
  "strider",
);
const stout = new Rig(1),
  [ti, tp, tc] = humanoid(stout, "ue", 0.72, 0.3, 0.32, 0.1, 0.2, 0.22, 0.2, 0.12, 0.26, 0.3, 0.24, 0.22, 0.09, 10, UE, "root");
character(
  join(ROOT, "assets/models/stout.glb"),
  stout,
  ti,
  tp,
  tc,
  [0.85, 0.5, 0.15, 1],
  {
    hips: 0.2,
    spine: 0.22,
    chest: 0.24,
    neck: 0.07,
    head: 0.16,
    shoulder: 0.07,
    upper_arm: 0.075,
    forearm: 0.07,
    hand: 0.06,
    thigh: 0.1,
    shin: 0.085,
    foot: 0.07,
    toe: 0.06,
  },
  "stout",
);
function puppet(path: string): void {
  const r = new Rig(1),
    root = r.node("Puppet", null, [0, 0, 0]),
    pos: Positions = { hips: [0, 0.95, 0], spine: [0, 1.2, 0], head: [0, 1.55, 0] },
    parent: Record<string, string | null> = { hips: null, spine: "hips", head: "spine" };
  for (const [s, z] of [
    ["L", 0.06],
    ["R", -0.06],
  ] as const) {
    Object.assign(pos, {
      [`upper_arm.${s}`]: [0, 1.5, z],
      [`forearm.${s}`]: [0, 1.22, z],
      [`hand.${s}`]: [0, 0.97, z],
      [`thigh.${s}`]: [0, 0.92, z * 0.6],
      [`shin.${s}`]: [0, 0.5, z * 0.6],
      [`foot.${s}`]: [0, 0.1, z * 0.6],
    });
    Object.assign(parent, {
      [`upper_arm.${s}`]: "spine",
      [`forearm.${s}`]: `upper_arm.${s}`,
      [`hand.${s}`]: `forearm.${s}`,
      [`thigh.${s}`]: "hips",
      [`shin.${s}`]: `thigh.${s}`,
      [`foot.${s}`]: `shin.${s}`,
    });
  }
  const order = [
      "hips",
      "spine",
      "head",
      ...[..."LR"].flatMap((s) => ["upper_arm", "forearm", "hand"].map((b) => `${b}.${s}`)),
      ...[..."LR"].flatMap((s) => ["thigh", "shin", "foot"].map((b) => `${b}.${s}`)),
    ],
    index: Bones = {};
  for (const b of order) index[b] = r.node(b, parent[b] == null ? root : index[parent[b]!]!, pos[b]!);
  const g = new Gltf(),
    joints = r.names.slice(1).map((_, i) => i + 1),
    ends: Record<string, [string, string, number]> = { hips: ["hips", "spine", 0.13], spine: ["spine", "head", 0.12] };
  for (const s of "LR")
    Object.assign(ends, {
      [`upper_arm.${s}`]: [`upper_arm.${s}`, `forearm.${s}`, 0.05],
      [`forearm.${s}`]: [`forearm.${s}`, `hand.${s}`, 0.045],
      [`thigh.${s}`]: [`thigh.${s}`, `shin.${s}`, 0.07],
      [`shin.${s}`]: [`shin.${s}`, `foot.${s}`, 0.055],
    });
  const prims = [true, false].map((near) => {
    const mesh = new Mesh(),
      quad = (cx: number, cy: number, cz: number, hx: number, hy: number, joint: number) =>
        mesh.box([cx, cy, cz], [hx, hy, 0.01], [0, 0, 0, 1], joint);
    for (const [bone, [a, b, w]] of Object.entries(ends)) {
      if (bone.endsWith(".R") === near) continue;
      const pa = pos[a]!,
        pb = pos[b]!;
      quad(pa[0]!, (pa[1]! + pb[1]!) / 2, pa[2]!, w, Math.abs(pa[1]! - pb[1]!) / 2 + 0.02, index[bone]! - 1);
    }
    if (near) quad(0.02, 1.68, 0, 0.13, 0.14, index.head! - 1);
    const s = near ? "L" : "R",
      h = pos[`hand.${s}`]!,
      f = pos[`foot.${s}`]!;
    quad(0, h[1]! - 0.05, h[2]!, 0.045, 0.06, index[`hand.${s}`]! - 1);
    quad(0.07, f[1]! - 0.05, f[2]!, 0.12, 0.045, index[`foot.${s}`]! - 1);
    return mesh.primitive(g, near ? 0 : 1);
  });
  const ibm = g.add(
      joints.flatMap((n) => r.inverseBind(n)),
      "f",
      joints.length,
      5126,
      "MAT4",
    ),
    Z = [0, 0, 1],
    ph = (t: number) => t * TAU;
  const chans: Chan[] = [
    [index.hips!, "translation", (t) => [0, 0.95 + 0.02 * Math.cos(2 * ph(t)), 0]],
    [index.spine!, "rotation", (t) => qaxis(Z, -0.06 + 0.03 * Math.cos(2 * ph(t)))],
  ];
  for (const [s, o] of [
    ["L", 0],
    ["R", Math.PI],
  ] as const)
    chans.push(
      [index[`thigh.${s}`]!, "rotation", (t) => qaxis(Z, 0.45 * Math.sin(ph(t) + o))],
      [index[`shin.${s}`]!, "rotation", (t) => qaxis(Z, -0.6 * Math.max(0, Math.cos(ph(t) + o)) ** 2)],
      [index[`upper_arm.${s}`]!, "rotation", (t) => qaxis(Z, -0.4 * Math.sin(ph(t) + o))],
      [index[`forearm.${s}`]!, "rotation", (t) => qaxis(Z, 0.35 + 0.15 * Math.sin(ph(t) + o))],
    );
  const animations = [clip(g, "walk", 1, 30, chans)],
    size = g.write(path, {
      asset,
      scene: 0,
      scenes: [{ name: "puppet", nodes: [0] }],
      nodes: [...nodesOf(r, { 0: [r.names.length] }), { name: "body", mesh: 0, skin: 0 }],
      meshes: [{ name: "puppet", primitives: prims }],
      materials: [material("near", [0.95, 0.62, 0.25, 1], 0.8), material("far", [0.55, 0.32, 0.14, 1], 0.8)],
      skins: [{ name: "puppet", joints, inverseBindMatrices: ibm, skeleton: joints[0] }],
      animations,
    });
  console.log(`${path}: ${size} bytes, ${r.names.length} nodes, 1 clip`);
}
puppet(join(ROOT, "assets/models/puppet2d.glb"));
