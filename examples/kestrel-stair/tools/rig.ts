// The characters: one humanoid clip library and three skinned bodies built
// from boxes, each on its own skeleton (other proportions, another rest
// pose, other bone names), so the clips reach them only by retargeting.
//
// - anims/humanoid.glb: the library. Blender-style names (hips, spine,
//   upper_arm.L, ...), a T-pose, no mesh; clips idle, walk, run, attack,
//   hurt, die, jump, bite and slam, with `hit` markers.
// - models/courier.glb: the hero. Mixamo names (mixamorig:Hips, ...), an
//   A-pose, 1.8 m, a sword in the right hand. Its import settings pull the
//   library in at cook time (`animation { library }`).
// - models/imp.glb: the thorn imp. Unreal names (pelvis, upperarm_l, ...),
//   arms hanging at 60 degrees, 1.1 m, long clawed arms; retargeted at run
//   time (og.anim.retarget, the names detected).
// - models/warden.glb: the stone warden. Names no scheme knows (Waist,
//   Breastplate, L_Pauldron, ...), 2.5 m, arms at 20 degrees; retargeted at
//   run time with a `type rig` file (assets/rigs/warden.ogt).
//
// Every rest rotation is the identity: a bone's direction is its child's
// offset. Clips are authored as rotations in the library's frames (the
// character faces +z, its left is +x).

import { Glb, MeshBuilder, quat, v3 } from "./gltf.ts";
import type { Q, V3 } from "./gltf.ts";

/** The standard bones, in the order every skeleton here lists them. */
export const BONES = [
  "hips", "spine", "chest", "neck", "head",
  "shoulderL", "upperArmL", "lowerArmL", "handL",
  "shoulderR", "upperArmR", "lowerArmR", "handR",
  "upperLegL", "lowerLegL", "footL", "toesL",
  "upperLegR", "lowerLegR", "footR", "toesR",
] as const;
export type Bone = (typeof BONES)[number];

const PARENT: Record<Bone, Bone | null> = {
  hips: null, spine: "hips", chest: "spine", neck: "chest", head: "neck",
  shoulderL: "chest", upperArmL: "shoulderL", lowerArmL: "upperArmL", handL: "lowerArmL",
  shoulderR: "chest", upperArmR: "shoulderR", lowerArmR: "upperArmR", handR: "lowerArmR",
  upperLegL: "hips", lowerLegL: "upperLegL", footL: "lowerLegL", toesL: "footL",
  upperLegR: "hips", lowerLegR: "upperLegR", footR: "lowerLegR", toesR: "footR",
};

/** Each naming scheme's node name for a standard bone. */
const NAMES: Record<string, Record<Bone, string>> = {
  blender: {
    hips: "hips", spine: "spine", chest: "chest", neck: "neck", head: "head",
    shoulderL: "shoulder.L", upperArmL: "upper_arm.L", lowerArmL: "forearm.L", handL: "hand.L",
    shoulderR: "shoulder.R", upperArmR: "upper_arm.R", lowerArmR: "forearm.R", handR: "hand.R",
    upperLegL: "thigh.L", lowerLegL: "shin.L", footL: "foot.L", toesL: "toe.L",
    upperLegR: "thigh.R", lowerLegR: "shin.R", footR: "foot.R", toesR: "toe.R",
  },
  mixamo: {
    hips: "mixamorig:Hips", spine: "mixamorig:Spine", chest: "mixamorig:Spine1", neck: "mixamorig:Neck", head: "mixamorig:Head",
    shoulderL: "mixamorig:LeftShoulder", upperArmL: "mixamorig:LeftArm", lowerArmL: "mixamorig:LeftForeArm", handL: "mixamorig:LeftHand",
    shoulderR: "mixamorig:RightShoulder", upperArmR: "mixamorig:RightArm", lowerArmR: "mixamorig:RightForeArm", handR: "mixamorig:RightHand",
    upperLegL: "mixamorig:LeftUpLeg", lowerLegL: "mixamorig:LeftLeg", footL: "mixamorig:LeftFoot", toesL: "mixamorig:LeftToeBase",
    upperLegR: "mixamorig:RightUpLeg", lowerLegR: "mixamorig:RightLeg", footR: "mixamorig:RightFoot", toesR: "mixamorig:RightToeBase",
  },
  unreal: {
    hips: "pelvis", spine: "spine_01", chest: "spine_02", neck: "neck_01", head: "head",
    shoulderL: "clavicle_l", upperArmL: "upperarm_l", lowerArmL: "lowerarm_l", handL: "hand_l",
    shoulderR: "clavicle_r", upperArmR: "upperarm_r", lowerArmR: "lowerarm_r", handR: "hand_r",
    upperLegL: "thigh_l", lowerLegL: "calf_l", footL: "foot_l", toesL: "ball_l",
    upperLegR: "thigh_r", lowerLegR: "calf_r", footR: "foot_r", toesR: "ball_r",
  },
  // The warden's own names: no detection scheme knows them, so assets/rigs/warden.ogt maps each.
  warden: {
    hips: "Waist", spine: "Belly", chest: "Breastplate", neck: "Gorget", head: "Helm",
    shoulderL: "L_Pauldron", upperArmL: "L_Bracer", lowerArmL: "L_Vambrace", handL: "L_Fist",
    shoulderR: "R_Pauldron", upperArmR: "R_Bracer", lowerArmR: "R_Vambrace", handR: "R_Fist",
    upperLegL: "L_Cuisse", lowerLegL: "L_Greave", footL: "L_Sabaton", toesL: "L_Toecap",
    upperLegR: "R_Cuisse", lowerLegR: "R_Greave", footR: "R_Sabaton", toesR: "R_Toecap",
  },
};

/** The `type rig` lines for the warden: the og.anim humanoid bone names, then the node. */
export const RIG_KEYS: Record<Bone, string> = {
  hips: "hips", spine: "spine", chest: "chest", neck: "neck", head: "head",
  shoulderL: "leftShoulder", upperArmL: "leftUpperArm", lowerArmL: "leftLowerArm", handL: "leftHand",
  shoulderR: "rightShoulder", upperArmR: "rightUpperArm", lowerArmR: "rightLowerArm", handR: "rightHand",
  upperLegL: "leftUpperLeg", lowerLegL: "leftLowerLeg", footL: "leftFoot", toesL: "leftToes",
  upperLegR: "rightUpperLeg", lowerLegR: "rightLowerLeg", footR: "rightFoot", toesR: "rightToes",
};
export function wardenRig(): string {
  const lines = BONES.map((b) => `${RIG_KEYS[b]} "${NAMES.warden![b]}"`);
  return `type rig\nversion 1\n---\n# The stone warden's bones by their own names (models/warden.glb):\n# no naming scheme detects them, so og.anim.retarget is given this map.\n${lines.join("\n")}\n`;
}

/** A body's proportions: where each joint's head is in the rest pose (metres). */
interface Proportions {
  hips: number; // hip joint height
  spine: number; // hips to spine
  chest: number; // spine to chest
  neck: number; // chest to neck
  head: number; // neck to head joint
  headSize: number; // head length
  shoulderX: number; // clavicle start offset from the middle
  shoulderY: number; // below the neck
  clavicle: number; // clavicle length
  upperArm: number;
  lowerArm: number;
  hand: number;
  armDrop: number; // the rest pose's arm angle below horizontal, degrees (0 a T-pose)
  legX: number; // hip joints' offset from the middle
  hipDrop: number; // hip joints below the hips joint
  upperLeg: number;
  lowerLeg: number;
  footHeight: number; // ankle above the ground
  foot: number; // ankle to toes, forward
  toes: number;
}

const LIBRARY: Proportions = {
  hips: 1.0, spine: 0.12, chest: 0.2, neck: 0.2, head: 0.1, headSize: 0.22,
  shoulderX: 0.04, shoulderY: 0.06, clavicle: 0.14, upperArm: 0.28, lowerArm: 0.26, hand: 0.14, armDrop: 0,
  legX: 0.1, hipDrop: 0.02, upperLeg: 0.44, lowerLeg: 0.44, footHeight: 0.08, foot: 0.14, toes: 0.08,
};

/** The rest pose: each bone's head in model space, and its tail (the last bones' ends). */
function restPose(p: Proportions): { head: Record<Bone, V3>; tail: Record<Bone, V3> } {
  const head = {} as Record<Bone, V3>;
  const tail = {} as Record<Bone, V3>;
  head.hips = [0, p.hips, 0];
  head.spine = [0, p.hips + p.spine, 0];
  head.chest = [0, p.hips + p.spine + p.chest, 0];
  head.neck = [0, p.hips + p.spine + p.chest + p.neck, 0];
  head.head = [0, head.neck[1] + p.head, 0];
  tail.head = [0, head.head[1] + p.headSize, 0];
  const drop = (p.armDrop * Math.PI) / 180;
  for (const [s, side] of [["L", 1], ["R", -1]] as const) {
    const sh: V3 = [side * p.shoulderX, head.neck[1] - p.shoulderY, 0];
    const ua: V3 = [sh[0] + side * p.clavicle, sh[1], 0];
    const dir: V3 = [side * Math.cos(drop), -Math.sin(drop), 0];
    const la = v3.add(ua, v3.scale(dir, p.upperArm));
    const ha = v3.add(la, v3.scale(dir, p.lowerArm));
    head[`shoulder${s}`] = sh;
    head[`upperArm${s}`] = ua;
    head[`lowerArm${s}`] = la;
    head[`hand${s}`] = ha;
    tail[`hand${s}`] = v3.add(ha, v3.scale(dir, p.hand));
    const hip: V3 = [side * p.legX, p.hips - p.hipDrop, 0];
    const knee: V3 = [hip[0], hip[1] - p.upperLeg, 0];
    const ankle: V3 = [hip[0], p.footHeight, 0];
    void knee;
    head[`upperLeg${s}`] = hip;
    head[`lowerLeg${s}`] = [hip[0], hip[1] - p.upperLeg, 0];
    head[`foot${s}`] = ankle;
    head[`toes${s}`] = [hip[0], 0.02, p.foot];
    tail[`toes${s}`] = [hip[0], 0.02, p.foot + p.toes];
  }
  // Every other bone's tail is its first child's head.
  for (const b of BONES) {
    if (tail[b]) continue;
    const child = BONES.find((c) => PARENT[c] === b);
    tail[b] = child ? head[child] : v3.add(head[b], [0, 0.1, 0]);
  }
  tail.chest = head.neck;
  tail.hips = head.spine;
  return { head, tail };
}

// ------------------------------------------------------------------ clips

type Pose = Partial<Record<Bone, Q>> & { hipsAt?: V3 };
const X = (deg: number): Q => quat.axisAngle([1, 0, 0], (deg * Math.PI) / 180);
const Yr = (deg: number): Q => quat.axisAngle([0, 1, 0], (deg * Math.PI) / 180);
const Z = (deg: number): Q => quat.axisAngle([0, 0, 1], (deg * Math.PI) / 180);
const M = (...qs: Q[]): Q => qs.reduce((a, b) => quat.mul(a, b), quat.identity());

/** Arms hanging `down` degrees from the T-pose, swung forward by `swing` (negative back), elbows bent by `bend`. */
function arms(downL: number, swingL: number, bendL: number, downR: number, swingR: number, bendR: number): Pose {
  return {
    upperArmL: M(X(-swingL), Z(-downL)),
    lowerArmL: X(-bendL),
    upperArmR: M(X(-swingR), Z(downR)),
    lowerArmR: X(-bendR),
  };
}

/** Legs: thigh swung forward by `swing` degrees, knee bent by `knee`, foot flexed by `ankle`. */
function legs(swingL: number, kneeL: number, ankleL: number, swingR: number, kneeR: number, ankleR: number): Pose {
  return {
    upperLegL: X(-swingL),
    lowerLegL: X(kneeL),
    footL: X(-ankleL),
    upperLegR: X(-swingR),
    lowerLegR: X(kneeR),
    footR: X(-ankleR),
  };
}

interface Clip {
  name: string;
  duration: number;
  loop: boolean;
  pose: (t: number) => Pose;
  markers?: { name: string; time: number }[];
}

const TAU = Math.PI * 2;

export const CLIPS: readonly Clip[] = [
  {
    name: "idle",
    duration: 2,
    loop: true,
    pose: (t) => {
      const b = Math.sin((t / 2) * TAU);
      return { spine: X(1.5 * b), chest: X(1.5 * b), neck: X(-2 * b), ...arms(74, 2 * b, 8, 74, -2 * b, 8), ...legs(0, 2, 0, 0, 2, 0), hipsAt: [0, -0.005 * (1 - b), 0] };
    },
  },
  {
    name: "walk",
    duration: 1,
    loop: true,
    pose: (t) => {
      const s = Math.sin(t * TAU);
      const c = Math.cos(t * TAU);
      const kneeL = 8 + 32 * Math.max(0, -Math.sin(t * TAU + 0.6));
      const kneeR = 8 + 32 * Math.max(0, Math.sin(t * TAU + 0.6));
      return {
        hips: Yr(5 * s),
        spine: X(3),
        chest: Yr(-7 * s),
        ...arms(76, -20 * s, 12 + 6 * Math.max(0, -s), 76, 20 * s, 12 + 6 * Math.max(0, s)),
        ...legs(24 * s, kneeL, 6 * c, -24 * s, kneeR, -6 * c),
        hipsAt: [0, 0.025 * Math.abs(c) - 0.02, 0],
      };
    },
  },
  {
    name: "run",
    duration: 0.7,
    loop: true,
    pose: (t) => {
      const ph = (t / 0.7) * TAU;
      const s = Math.sin(ph);
      const kneeL = 15 + 75 * Math.max(0, -Math.sin(ph + 0.8));
      const kneeR = 15 + 75 * Math.max(0, Math.sin(ph + 0.8));
      return {
        hips: Yr(8 * s),
        spine: X(10),
        chest: M(X(4), Yr(-12 * s)),
        neck: X(-8),
        ...arms(70, -45 * s, 70, 70, 45 * s, 70),
        ...legs(42 * s + 8, kneeL, 10 * s, -42 * s + 8, kneeR, -10 * s),
        hipsAt: [0, 0.05 * Math.abs(Math.cos(ph)) - 0.06, 0],
      };
    },
  },
  {
    name: "attack",
    duration: 0.55,
    loop: false,
    markers: [{ name: "hit", time: 0.16 }],
    pose: (t) => {
      // The sword arm (right) up and back, then a slash across and down; the torso turns with it.
      const k = t < 0.08 ? t / 0.08 : t < 0.22 ? 1 + (t - 0.08) / 0.14 : 2 + Math.min(1, (t - 0.22) / 0.33);
      const lerp = (a: number, b: number, c: number, d: number) => (k < 1 ? a + (b - a) * k : k < 2 ? b + (c - b) * (k - 1) : c + (d - c) * (k - 2));
      const twist = lerp(0, 35, -40, 0);
      return {
        spine: M(X(lerp(0, -4, 10, 0)), Yr(twist * 0.4)),
        chest: Yr(twist * 0.6),
        ...arms(70, 10, 20, lerp(74, 10, 60, 74), lerp(0, 60, 95, 0), lerp(10, 80, 5, 10)),
        ...legs(lerp(0, 12, 22, 0), lerp(3, 12, 25, 3), 0, lerp(0, -10, -18, 0), lerp(3, 8, 6, 3), 0),
        hipsAt: [0, lerp(0, -0.03, -0.08, 0), lerp(0, 0, 0.12, 0)],
      };
    },
  },
  {
    name: "hurt",
    duration: 0.4,
    loop: false,
    pose: (t) => {
      const k = Math.sin(Math.min(1, t / 0.4) * Math.PI);
      return { spine: X(-14 * k), chest: X(-10 * k), neck: X(-12 * k), ...arms(70 - 30 * k, -20 * k, 30 * k, 70 - 30 * k, -20 * k, 30 * k), ...legs(10 * k, 20 * k, 0, -5 * k, 10 * k, 0), hipsAt: [0, -0.04 * k, -0.08 * k] };
    },
  },
  {
    name: "die",
    duration: 1.2,
    loop: false,
    pose: (t) => {
      const k = Math.min(1, t / 0.9);
      const e = 1 - (1 - k) * (1 - k);
      return {
        hips: X(-78 * e),
        spine: X(-10 * e),
        neck: X(20 * e),
        ...arms(74 - 50 * e, -20 * e, 20, 74 - 60 * e, -30 * e, 20),
        ...legs(30 * e, 30 * e, 10, 10 * e, 50 * e, 10),
        hipsAt: [0, -0.82 * e, -0.3 * e],
      };
    },
  },
  {
    name: "jump",
    duration: 0.6,
    loop: true,
    pose: (t) => {
      const b = Math.sin((t / 0.6) * TAU) * 0.5 + 0.5;
      return { spine: X(6), ...arms(30 + 10 * b, 20, 40, 30 + 10 * b, 20, 40), ...legs(35, 70 + 10 * b, 10, 10, 30, 0), hipsAt: [0, -0.05, 0] };
    },
  },
  {
    name: "bite",
    duration: 0.5,
    loop: false,
    markers: [{ name: "hit", time: 0.24 }],
    pose: (t) => {
      // A crouch, then a lunge with both arms raking forward.
      const k = t < 0.18 ? t / 0.18 : t < 0.28 ? 1 + (t - 0.18) / 0.1 : 2 + (t - 0.28) / 0.22;
      const lerp = (a: number, b: number, c: number, d: number) => (k < 1 ? a + (b - a) * k : k < 2 ? b + (c - b) * (k - 1) : c + (d - c) * Math.min(1, k - 2));
      return {
        spine: X(lerp(5, -10, 30, 5)),
        chest: X(lerp(0, -5, 15, 0)),
        ...arms(lerp(74, 20, 60, 74), lerp(0, 60, 80, 0), lerp(10, 90, 10, 10), lerp(74, 20, 60, 74), lerp(0, 60, 80, 0), lerp(10, 90, 10, 10)),
        ...legs(lerp(0, 30, -10, 0), lerp(4, 60, 10, 4), 0, lerp(0, 30, 25, 0), lerp(4, 60, 30, 4), 0),
        hipsAt: [0, lerp(0, -0.15, -0.02, 0), lerp(0, -0.05, 0.25, 0)],
      };
    },
  },
  {
    name: "slam",
    duration: 1.4,
    loop: false,
    markers: [{ name: "hit", time: 0.82 }],
    pose: (t) => {
      // Both fists raised overhead (a long wind-up), a lunge forward and the slam, then a slow recovery.
      // The hips travel 0.9 m forward over the lunge: root motion.
      const raise = smooth01((t - 0.05) / 0.55);
      const strike = smooth01((t - 0.62) / 0.2);
      const back = smooth01((t - 1.0) / 0.4);
      const up = raise * (1 - strike);
      return {
        spine: X(-12 * up + 28 * strike * (1 - back)),
        chest: X(-8 * up + 14 * strike * (1 - back)),
        ...arms(74 - 150 * up - 60 * strike * (1 - back), 160 * up + 70 * strike * (1 - back), 30 * up, 74 - 150 * up - 60 * strike * (1 - back), 160 * up + 70 * strike * (1 - back), 30 * up),
        ...legs(20 * strike * (1 - back), 6 + 30 * strike * (1 - back), 0, -15 * strike * (1 - back), 6 + 10 * strike * (1 - back), 0),
        hipsAt: [0, -0.12 * strike * (1 - back), 0.9 * strike],
      };
    },
  },
];

function smooth01(x: number): number {
  const t = Math.max(0, Math.min(1, x));
  return t * t * (3 - 2 * t);
}

// --------------------------------------------------------------- skeletons

interface Body {
  scheme: keyof typeof NAMES;
  prop: Proportions;
  /** Boxes per bone: half widths (x, z) and colour, with an extra piece or two. */
  build: (mb: MeshBuilder, head: Record<Bone, V3>, tail: Record<Bone, V3>, joint: (b: Bone) => number, mats: { body: number; glow: number }) => void;
}

/** A box along a bone from its head to its tail: `w` wide (x), `d` deep (z), grown by `pad` at both ends. */
function limb(mb: MeshBuilder, mat: number, j: number, a: V3, b: V3, w: number, d: number, color: [number, number, number], pad = 0.02): void {
  const dir = v3.sub(b, a);
  const len = v3.len(dir);
  const y = v3.norm(dir);
  // A frame with y along the bone; z as close to the model's +z as the bone allows.
  let z: V3 = Math.abs(y[2]) > 0.9 ? [0, 1, 0] : [0, 0, 1];
  const x = v3.norm(v3.cross(y, z));
  z = v3.norm(v3.cross(x, y));
  mb.box(mat, [-w / 2, -pad, -d / 2], [w / 2, len + pad, d / 2], { joint: j, color, frame: { o: a, x, y, z } });
}

const STEEL: [number, number, number] = [0.42, 0.44, 0.5];
const LEATHER: [number, number, number] = [0.36, 0.22, 0.13];
const SKIN: [number, number, number] = [0.86, 0.62, 0.47];
const CLOTH: [number, number, number] = [0.62, 0.12, 0.1];

export const BODIES: Record<"courier" | "imp" | "warden", Body> = {
  courier: {
    scheme: "mixamo",
    prop: {
      hips: 1.02, spine: 0.13, chest: 0.21, neck: 0.21, head: 0.1, headSize: 0.24,
      shoulderX: 0.04, shoulderY: 0.06, clavicle: 0.15, upperArm: 0.3, lowerArm: 0.27, hand: 0.15, armDrop: 40,
      legX: 0.11, hipDrop: 0.03, upperLeg: 0.45, lowerLeg: 0.44, footHeight: 0.09, foot: 0.15, toes: 0.08,
    },
    build: (mb, h, t, j, m) => {
      limb(mb, m.body, j("hips"), v3.add(h.hips, [0, -0.1, 0]), h.spine, 0.34, 0.22, LEATHER);
      limb(mb, m.body, j("spine"), h.spine, h.chest, 0.34, 0.22, CLOTH);
      limb(mb, m.body, j("chest"), h.chest, v3.add(h.neck, [0, 0.02, 0]), 0.42, 0.25, STEEL);
      limb(mb, m.body, j("neck"), h.neck, h.head, 0.1, 0.1, SKIN);
      limb(mb, m.body, j("head"), h.head, t.head, 0.2, 0.22, SKIN);
      // A steel cap over the head and a dark visor band.
      limb(mb, m.body, j("head"), v3.add(h.head, [0, 0.11, 0]), v3.add(t.head, [0, 0.03, 0]), 0.23, 0.25, STEEL, 0);
      limb(mb, m.body, j("head"), v3.add(h.head, [0, 0.08, 0.06]), v3.add(h.head, [0, 0.12, 0.06]), 0.21, 0.13, [0.08, 0.08, 0.1], 0);
      for (const s of ["L", "R"] as const) {
        limb(mb, m.body, j(`shoulder${s}`), h[`shoulder${s}`], h[`upperArm${s}`], 0.1, 0.12, STEEL);
        limb(mb, m.body, j(`upperArm${s}`), h[`upperArm${s}`], h[`lowerArm${s}`], 0.11, 0.11, CLOTH);
        limb(mb, m.body, j(`lowerArm${s}`), h[`lowerArm${s}`], h[`hand${s}`], 0.1, 0.1, STEEL);
        limb(mb, m.body, j(`hand${s}`), h[`hand${s}`], t[`hand${s}`], 0.09, 0.07, LEATHER);
        limb(mb, m.body, j(`upperLeg${s}`), h[`upperLeg${s}`], h[`lowerLeg${s}`], 0.15, 0.16, LEATHER);
        limb(mb, m.body, j(`lowerLeg${s}`), h[`lowerLeg${s}`], h[`foot${s}`], 0.13, 0.14, STEEL);
        limb(mb, m.body, j(`foot${s}`), h[`foot${s}`], h[`toes${s}`], 0.11, 0.1, LEATHER);
        limb(mb, m.body, j(`toes${s}`), h[`toes${s}`], t[`toes${s}`], 0.11, 0.06, LEATHER);
      }
      // The sword in the right hand: along the arm's rest direction, its blade past the fist.
      const hand = h.handR;
      const dir = v3.norm(v3.sub(t.handR, h.handR));
      const grip = v3.add(hand, v3.scale(dir, 0.1));
      const fwd: V3 = [0, 0, 1];
      limb(mb, m.body, j("handR"), v3.add(grip, v3.scale(fwd, -0.08)), v3.add(grip, v3.scale(fwd, 0.04)), 0.04, 0.04, LEATHER, 0);
      limb(mb, m.body, j("handR"), v3.add(grip, v3.scale(fwd, 0.04)), v3.add(grip, v3.scale(fwd, 0.06)), 0.24, 0.05, [0.7, 0.58, 0.25], 0);
      limb(mb, m.body, j("handR"), v3.add(grip, v3.scale(fwd, 0.06)), v3.add(grip, v3.scale(fwd, 0.9)), 0.07, 0.02, [0.82, 0.85, 0.9], 0);
    },
  },
  imp: {
    scheme: "unreal",
    prop: {
      hips: 0.52, spine: 0.1, chest: 0.16, neck: 0.12, head: 0.06, headSize: 0.24,
      shoulderX: 0.03, shoulderY: 0.03, clavicle: 0.1, upperArm: 0.26, lowerArm: 0.26, hand: 0.14, armDrop: 60,
      legX: 0.09, hipDrop: 0.02, upperLeg: 0.22, lowerLeg: 0.22, footHeight: 0.06, foot: 0.12, toes: 0.06,
    },
    build: (mb, h, t, j, m) => {
      const BARK: [number, number, number] = [0.3, 0.36, 0.16];
      const THORN: [number, number, number] = [0.55, 0.5, 0.2];
      limb(mb, m.body, j("hips"), v3.add(h.hips, [0, -0.06, 0]), h.spine, 0.24, 0.18, BARK);
      limb(mb, m.body, j("spine"), h.spine, h.chest, 0.26, 0.2, BARK);
      limb(mb, m.body, j("chest"), h.chest, h.neck, 0.32, 0.24, BARK);
      limb(mb, m.body, j("neck"), h.neck, h.head, 0.08, 0.08, BARK);
      limb(mb, m.body, j("head"), h.head, t.head, 0.28, 0.26, BARK);
      // Thorns: a crown of spikes and a ridge down the back; two glowing eyes.
      for (const [x, z] of [[-0.08, -0.05], [0.08, -0.05], [0, -0.1], [-0.12, 0.02], [0.12, 0.02]] as const) {
        limb(mb, m.body, j("head"), v3.add(t.head, [x, -0.04, z]), v3.add(t.head, [x * 1.6, 0.12, z * 1.6]), 0.04, 0.04, THORN, 0);
      }
      for (const y of [0.05, 0.12]) limb(mb, m.body, j("chest"), v3.add(h.chest, [0, y, -0.11]), v3.add(h.chest, [0, y + 0.08, -0.22]), 0.04, 0.04, THORN, 0);
      for (const x of [-0.06, 0.06]) limb(mb, m.glow, j("head"), v3.add(h.head, [x, 0.12, 0.12]), v3.add(h.head, [x, 0.15, 0.135]), 0.04, 0.015, [1, 1, 1], 0);
      for (const s of ["L", "R"] as const) {
        limb(mb, m.body, j(`shoulder${s}`), h[`shoulder${s}`], h[`upperArm${s}`], 0.08, 0.08, BARK);
        limb(mb, m.body, j(`upperArm${s}`), h[`upperArm${s}`], h[`lowerArm${s}`], 0.07, 0.07, BARK);
        limb(mb, m.body, j(`lowerArm${s}`), h[`lowerArm${s}`], h[`hand${s}`], 0.06, 0.06, BARK);
        limb(mb, m.body, j(`hand${s}`), h[`hand${s}`], t[`hand${s}`], 0.1, 0.05, THORN);
        limb(mb, m.body, j(`upperLeg${s}`), h[`upperLeg${s}`], h[`lowerLeg${s}`], 0.09, 0.09, BARK);
        limb(mb, m.body, j(`lowerLeg${s}`), h[`lowerLeg${s}`], h[`foot${s}`], 0.07, 0.07, BARK);
        limb(mb, m.body, j(`foot${s}`), h[`foot${s}`], h[`toes${s}`], 0.08, 0.06, THORN);
        limb(mb, m.body, j(`toes${s}`), h[`toes${s}`], t[`toes${s}`], 0.08, 0.04, THORN);
      }
    },
  },
  warden: {
    scheme: "warden",
    prop: {
      hips: 1.25, spine: 0.2, chest: 0.32, neck: 0.3, head: 0.1, headSize: 0.34,
      shoulderX: 0.08, shoulderY: 0.1, clavicle: 0.3, upperArm: 0.42, lowerArm: 0.4, hand: 0.2, armDrop: 20,
      legX: 0.2, hipDrop: 0.04, upperLeg: 0.55, lowerLeg: 0.52, footHeight: 0.14, foot: 0.2, toes: 0.1,
    },
    build: (mb, h, t, j, m) => {
      const STONE: [number, number, number] = [0.52, 0.52, 0.47];
      const MOSS: [number, number, number] = [0.32, 0.42, 0.24];
      limb(mb, m.body, j("hips"), v3.add(h.hips, [0, -0.16, 0]), h.spine, 0.62, 0.44, STONE);
      limb(mb, m.body, j("spine"), h.spine, h.chest, 0.66, 0.46, MOSS);
      limb(mb, m.body, j("chest"), h.chest, v3.add(h.neck, [0, 0.06, 0]), 0.9, 0.56, STONE);
      limb(mb, m.body, j("neck"), h.neck, h.head, 0.22, 0.22, STONE);
      limb(mb, m.body, j("head"), h.head, t.head, 0.36, 0.36, STONE);
      limb(mb, m.glow, j("head"), v3.add(h.head, [0, 0.16, 0.18]), v3.add(h.head, [0, 0.2, 0.19]), 0.24, 0.02, [1, 1, 1], 0);
      for (const s of ["L", "R"] as const) {
        limb(mb, m.body, j(`shoulder${s}`), h[`shoulder${s}`], h[`upperArm${s}`], 0.32, 0.36, MOSS);
        limb(mb, m.body, j(`upperArm${s}`), h[`upperArm${s}`], h[`lowerArm${s}`], 0.24, 0.24, STONE);
        limb(mb, m.body, j(`lowerArm${s}`), h[`lowerArm${s}`], h[`hand${s}`], 0.26, 0.26, STONE);
        limb(mb, m.body, j(`hand${s}`), h[`hand${s}`], t[`hand${s}`], 0.32, 0.3, STONE);
        limb(mb, m.body, j(`upperLeg${s}`), h[`upperLeg${s}`], h[`lowerLeg${s}`], 0.3, 0.3, STONE);
        limb(mb, m.body, j(`lowerLeg${s}`), h[`lowerLeg${s}`], h[`foot${s}`], 0.28, 0.28, MOSS);
        limb(mb, m.body, j(`foot${s}`), h[`foot${s}`], h[`toes${s}`], 0.28, 0.2, STONE);
        limb(mb, m.body, j(`toes${s}`), h[`toes${s}`], t[`toes${s}`], 0.28, 0.12, STONE);
      }
    },
  },
};

/** Writes the joint nodes under a root node; answers each bone's node index. */
function skeleton(g: Glb, scheme: keyof typeof NAMES, head: Record<Bone, V3>, rootName: string): Record<Bone, number> {
  const nodes = {} as Record<Bone, number>;
  const root = g.node({ name: rootName, children: [] as number[] }, true);
  for (const b of BONES) {
    const parent = PARENT[b];
    const t = parent ? v3.sub(head[b], head[parent]) : head[b];
    nodes[b] = g.node({ name: NAMES[scheme]![b], translation: t.map((v) => Math.round(v * 1e6) / 1e6) });
  }
  for (const b of BONES) {
    const parent = PARENT[b];
    const at = parent ? nodes[parent] : root;
    const n = g.json.nodes[at]!;
    ((n.children ??= []) as number[]).push(nodes[b]);
  }
  return nodes;
}

/** The clip library: a skeleton and every clip, no mesh. */
export function library(): Uint8Array {
  const g = new Glb();
  const { head } = restPose(LIBRARY);
  const nodes = skeleton(g, "blender", head, "Rig");
  const FPS = 30;
  for (const clip of CLIPS) {
    const n = Math.round(clip.duration * FPS) + 1;
    const times = new Float32Array(n);
    for (let i = 0; i < n; i++) times[i] = Math.min(clip.duration, i / FPS);
    const input = g.accessor(times, "SCALAR", undefined, true);
    const samplers: object[] = [];
    const channels: object[] = [];
    const poses = Array.from({ length: n }, (_, i) => clip.pose(clip.loop && i === n - 1 ? 0 : times[i]!));
    for (const b of BONES) {
      const rot = new Float32Array(n * 4);
      let prev: Q = [0, 0, 0, 1];
      let moving = false;
      for (let i = 0; i < n; i++) {
        let q = poses[i]![b] ?? quat.identity();
        q = quat.align(prev, q);
        prev = q;
        rot.set(q, i * 4);
        if (Math.abs(q[3]) < 0.99999) moving = true;
      }
      if (!moving && b !== "hips") continue;
      samplers.push({ input, output: g.accessor(rot, "VEC4"), interpolation: "LINEAR" });
      channels.push({ sampler: samplers.length - 1, target: { node: nodes[b], path: "rotation" } });
    }
    const tr = new Float32Array(n * 3);
    for (let i = 0; i < n; i++) tr.set(v3.add(head.hips, poses[i]!.hipsAt ?? [0, 0, 0]), i * 3);
    samplers.push({ input, output: g.accessor(tr, "VEC3"), interpolation: "LINEAR" });
    channels.push({ sampler: samplers.length - 1, target: { node: nodes.hips, path: "translation" } });
    g.json.animations.push({ name: clip.name, samplers, channels, ...(clip.markers ? { extras: { "og.events": clip.markers } } : {}) });
  }
  return g.bytes();
}

/** A character: its skeleton, its box body skinned rigidly to it, no clips of its own. */
export function character(kind: keyof typeof BODIES): Uint8Array {
  const body = BODIES[kind];
  const g = new Glb();
  const { head, tail } = restPose(body.prop);
  const nodes = skeleton(g, body.scheme, head, kind === "courier" ? "Armature" : "root");
  const jointOf = (b: Bone) => BONES.indexOf(b);
  const mats = {
    body: g.material({ name: `${kind}-body`, pbrMetallicRoughness: { baseColorFactor: [1, 1, 1, 1], metallicFactor: kind === "courier" ? 0.2 : 0, roughnessFactor: 0.75 } }),
    glow: g.material({ name: `${kind}-eyes`, pbrMetallicRoughness: { baseColorFactor: [1, 0.55, 0.2, 1], metallicFactor: 0, roughnessFactor: 1 }, emissiveFactor: kind === "warden" ? [0.4, 0.9, 1] : [1, 0.6, 0.15] }),
  };
  const mb = new MeshBuilder();
  body.build(mb, head, tail, jointOf, mats);
  const prims = [...mb.groups.entries()].map(([material, p]) => ({ pos: p.pos, nor: p.nor, col: p.col, joints: p.joints, weights: p.weights, idx: p.idx, material }));
  const mesh = g.mesh(kind, prims);
  const ibm = new Float32Array(16 * BONES.length);
  BONES.forEach((b, i) => {
    for (let k = 0; k < 4; k++) ibm[i * 16 + k * 5] = 1;
    ibm[i * 16 + 12] = -head[b][0];
    ibm[i * 16 + 13] = -head[b][1];
    ibm[i * 16 + 14] = -head[b][2];
  });
  const skin = g.json.skins.push({ joints: BONES.map((b) => nodes[b]), inverseBindMatrices: g.accessor(ibm, "MAT4"), skeleton: nodes.hips }) - 1;
  g.node({ name: "body", mesh, skin }, true);
  return g.bytes();
}

/** The character's height, its rest hips height and its right hand's joint index (for game code). */
export function measures(kind: keyof typeof BODIES): { height: number; hips: number } {
  const { tail, head } = restPose(BODIES[kind].prop);
  return { height: tail.head[1], hips: head.hips[1] };
}
