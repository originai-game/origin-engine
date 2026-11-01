// The U5 sample (docs/ui.md §14, §5.5): an arena whose UI lives in the
// world. tests/evidence/u5/NOTES.md has the commands and what each frame
// shows.
//
// - 100 pillars, each with a nameplate (an anchored element: a name and a
//   health bar) that stacks over the nameplates it overlaps, nearer and
//   higher priority keeping their place, fading behind the scene;
// - damage numbers: a mass element template spawned 34 a tick with a
//   60-tick life (about 2,040 live), its pop and rise evaluated on the GPU;
// - a waypoint behind the camera, clamped to the bottom edge of the safe
//   area with an arrow toward it and its distance;
// - a terminal: a world panel drawn into a render target on a plane,
//   whose button a press on the plane reaches through the pointer's ray;
// - a loadout menu with a character viewport (an og.CameraView of a
//   turntable on its own layer, turned by a drag) and a Claim button that
//   throws confetti (og.ui.particles);
// - a low-health vignette pulsing over the scene (og.ui.meta).

import { og, defineComponent, defineSystem, f64, u32 } from "@origingame/engine";
import type { Entity, Geometry, Material, Texture, World } from "@origingame/engine";

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};
const fixed = (px: number) => ({ kind: "fixed", px }) as const;
const full = { kind: "percent", fraction: 1 } as const;
const rgb = (r: number, g: number, b: number, alpha = 1) => ({ color: { r, g, b }, alpha }) as never;

/** Where the main camera stands and looks (the waypoint is behind it). */
const EYE = { x: 0, y: 9, z: 16 };
const LOOK = { x: 0, y: 0, z: -4 };
/** The terminal's centre; it faces the camera. */
const TERMINAL = { x: -5.5, y: 3.4, z: 7 };
/** The turntable, far from the arena and on layer 1 (mask 2) only. */
const STAGE = { x: 200, y: 0, z: 0 };
const WAYPOINT = { x: 3, y: 1, z: 34 };
const PILLARS = 100;
const PER_TICK = 34;

/** What must replay: on the entity "arena". */
const Arena = defineComponent("Arena", {
  /** Ticks since the start. */
  tick: u32(0),
  /** Presses on the terminal's button. */
  presses: u32(0),
  /** Claims in the menu. */
  claims: u32(0),
  /** The turntable's yaw. */
  yaw: f64(0),
});

const NAMES = ["Warden", "Rook", "Vex", "Moth", "Sable", "Ira", "Quill", "Brask", "Nyx", "Odo"];

/** The damage number: a pop, a rise and a fade, evaluated on the GPU. */
const DAMAGE = {
  kind: "number",
  space: "world",
  size: { x: 22, y: 0 },
  pivot: { x: 0.5, y: 1 },
  lifetime: 60,
  keys: [
    { t: 0, scale: 0.4, alpha: 1 },
    { t: 0.12, scale: 1.15, offsetY: -18, ease: "back" },
    { t: 0.7, scale: 1, offsetY: -46, alpha: 1, ease: "out" },
    { t: 1, scale: 0.9, offsetY: -64, alpha: 0, ease: "in" },
  ],
  drift: 22,
  reference: 14,
  minScale: 0.5,
  maxScale: 1.6,
  occlusion: "fade",
  occludedAlpha: 0.2,
  outline: rgb(0.05, 0.02, 0.02, 0.9),
  outlineWidth: 2,
} as never;

let dmg = 0;
let pillars: Entity[] = [];
let terminalTexture: Texture | null = null;
let terminal: Entity | null = null;
let stageTexture: Texture | null = null;
let turntable: Entity | null = null;

/** A hash that a replay repeats (no Math.random). */
function hash(n: number): number {
  let x = (n ^ 0x9e3779b9) >>> 0;
  x = Math.imul(x ^ (x >>> 16), 0x85ebca6b) >>> 0;
  x = Math.imul(x ^ (x >>> 13), 0xc2b2ae35) >>> 0;
  return (x ^ (x >>> 16)) >>> 0;
}

/** Nameplates drawn last tick (declutter hides the ones it cannot stack). */
let shown = 0;

function nameplates(): void {
  let n = 0;
  for (let i = 0; i < pillars.length; i++) {
    const e = pillars[i]!;
    // The front rows rank higher: they keep their place, the rows behind stack over them.
    const info = og.ui.openWorldAnchor(`Plate${i}`, {
      target: { entity: e },
      offset: { x: 0, y: 1.9, z: 0 },
      pivot: "bottom",
      direction: "column",
      cross: "center",
      gap: 2,
      padding: { left: 6, top: 3, right: 6, bottom: 4 },
      background: rgb(0.04, 0.05, 0.08, 0.72),
      radius: 5,
      occlusion: "fade",
      occludedAlpha: 0.3,
      lod: [22],
      maxDistance: 45,
      declutter: "stack",
      priority: Math.floor(i / 10),
      fadeTicks: 8,
    } as never);
    og.ui.label(`Plate${i}Name`, `${NAMES[i % NAMES.length]} ${i + 1}`, { step: "caption", color: rgb(0.95, 0.93, 0.86), nowrap: true } as never);
    if (info.alpha > 0.5) n += 1;
    // Level 1 (past 22 m) is the name alone.
    if (info.lod === 0) {
      const hp = 0.25 + (hash(i) % 75) / 100;
      og.ui.open(`Plate${i}Track`, { width: fixed(64), height: fixed(5), background: rgb(0, 0, 0, 0.8), radius: 2 } as never);
      og.ui.open(`Plate${i}Fill`, { width: fixed(64 * hp), height: fixed(5), background: hp > 0.5 ? rgb(0.35, 0.9, 0.4) : rgb(0.95, 0.35, 0.25), radius: 2 } as never);
      og.ui.close();
      og.ui.close();
    }
    og.ui.close();
  }
  shown = n;
}

function damageNumbers(world: World, tick: number): void {
  const batch = [];
  for (let k = 0; k < PER_TICK; k++) {
    const n = tick * PER_TICK + k;
    const h = hash(n);
    const p = must(world.get(pillars[h % PILLARS]!, og.Transform)).position;
    const crit = h % 9 === 0;
    batch.push({
      position: { x: p.x + ((h >>> 8) % 100) / 100 - 0.5, y: 1.2 + ((h >>> 16) % 60) / 100, z: p.z },
      value: 10 + ((h >>> 4) % 990),
      color: crit ? rgb(1, 0.82, 0.2) : rgb(1, 1, 1),
      scale: crit ? 1.6 : 1,
      seed: h % 1000,
    });
  }
  og.ui.spawnMass(dmg, batch as never);
}

function waypoint(): void {
  const info = og.ui.openWorldAnchor("Waypoint", {
    target: { point: WAYPOINT },
    pivot: "center",
    direction: "column",
    cross: "center",
    padding: { left: 10, top: 6, right: 10, bottom: 6 },
    background: rgb(0.08, 0.06, 0.02, 0.85),
    radius: 8,
    clamp: { margin: 28, arrow: rgb(1, 0.78, 0.25), arrowSize: 16 },
  } as never);
  og.ui.label("WaypointName", "Extraction", { step: "sm", color: rgb(1, 0.85, 0.4), nowrap: true } as never);
  og.ui.label("WaypointDistance", `${Math.round(info.distance)} m`, { step: "caption", color: rgb(0.95, 0.92, 0.85), nowrap: true } as never);
  og.ui.close();
}

function terminalPanel(arena: { presses: number }): number {
  let presses = arena.presses;
  og.ui.openWorldPanel("Terminal", {
    target: terminalTexture,
    width: 480,
    height: 320,
    quad: { entity: terminal },
    quadWidth: 3.6,
    quadHeight: 2.4,
    direction: "column",
    gap: 14,
    padding: { left: 24, top: 20, right: 24, bottom: 24 },
    background: rgb(0.02, 0.08, 0.06),
  } as never);
  og.ui.label("TerminalTitle", "TERMINAL 07 // GATE CONTROL", { step: "lg", color: rgb(0.3, 1, 0.6), nowrap: true } as never);
  og.ui.label("TerminalState", presses % 2 === 1 ? "Gate: OPEN" : "Gate: SEALED", { step: "base", color: presses % 2 === 1 ? rgb(0.4, 1, 0.5) : rgb(1, 0.5, 0.35) } as never);
  og.ui.label("TerminalCount", `Presses: ${presses}`, { step: "sm", color: rgb(0.6, 0.85, 0.7) } as never);
  if (og.ui.button("TerminalToggle", "Toggle gate", { variant: "primary", size: "lg", width: full } as never).clicked) presses += 1;
  og.ui.close();
  return presses;
}

function hud(): void {
  og.ui.open("Hud", { direction: "column", gap: 8, padding: { left: 16, top: 12, right: 16, bottom: 14 }, background: rgb(0.03, 0.03, 0.05, 0.7), radius: 10 } as never);
  og.ui.label("HudHp", "HP 23 / 100", { step: "lg", color: rgb(1, 0.45, 0.35) } as never);
  og.ui.progress("HudHpBar", 0.23, { width: fixed(220) } as never);
  og.ui.label("HudCounts", `Nameplates ${shown} / ${PILLARS} · Damage numbers ${og.ui.massCount(dmg)}`, { step: "caption", color: rgb(0.85, 0.85, 0.9), nowrap: true } as never);
  og.ui.close();
}

function menu(arena: { claims: number; yaw: number }): { claims: number; yaw: number } {
  let { claims, yaw } = arena;
  og.ui.open("Loadout", { direction: "column", gap: 12, cross: "center", padding: { left: 18, top: 16, right: 18, bottom: 18 }, background: rgb(0.06, 0.05, 0.1, 0.88), radius: 14 } as never);
  og.ui.label("LoadoutTitle", "LOADOUT", { step: "xl", color: rgb(0.95, 0.9, 1), heading: true } as never);
  const view = og.ui.sceneViewport("Character", {
    texture: stageTexture,
    width: fixed(300),
    height: fixed(300),
    cameraPosition: { x: STAGE.x, y: 1.5, z: 4.2 },
    cameraTarget: { x: STAGE.x, y: 1.2, z: 0 },
    fov: 35,
    background: rgb(0.12, 0.1, 0.2),
    radius: 12,
    label: "Character preview",
  } as never);
  // Turned by a drag; at rest it turns slowly.
  yaw += view.pressed ? view.drag.x * 0.012 : 0.01;
  og.ui.label("LoadoutHint", "Drag to turn", { step: "caption", color: rgb(0.7, 0.68, 0.8) } as never);
  const claim = og.ui.button("Claim", claims > 0 ? `Claimed ×${claims}` : "Claim reward", { variant: "primary", size: "lg", width: fixed(300) } as never);
  og.ui.particles({
    burst: 90,
    fire: claim.clicked,
    lifetimeMs: 1400,
    shape: "square",
    size: { x: 9, y: 5 },
    colors: [rgb(1, 0.82, 0.2), rgb(0.95, 0.3, 0.5), rgb(0.3, 0.8, 1), rgb(0.5, 1, 0.5), rgb(0.85, 0.5, 1)],
    travelMin: 80,
    travelMax: 220,
    direction: -Math.PI / 2,
    spread: Math.PI * 1.2,
    gravity: 180,
    spin: 3,
    from: "area",
    endScale: 0.8,
    clip: false,
  } as never);
  if (claim.clicked) claims += 1;
  og.ui.close();
  return { claims, yaw };
}

export const Ui = defineSystem("Ui", { reads: [og.Input], writes: [Arena, og.Transform] }, (_q, _time, world) => {
  const e = world.find("arena");
  if (e === null) return;
  const a = must(world.get(e, Arena));
  // og.ui calls build inside a system; declaring again keeps the handle.
  dmg = og.ui.defineMass("damage", DAMAGE);

  // Over the scene, under the screen: the low-health pulse.
  og.ui.meta("LowHealth", { kind: "vignette", color: rgb(0.7, 0.02, 0.02), inner: 0.5, outer: 1, intensity: 0.75, pulseHz: 1.2, pulseDepth: 0.55 } as never);

  nameplates();
  damageNumbers(world, a.tick);
  waypoint();
  const presses = terminalPanel(a);

  og.ui.open("Root", { width: full, height: full, direction: "row", align: "spaceBetween", cross: "start", padding: { left: 24, top: 24, right: 24, bottom: 24 } } as never);
  hud();
  const m = menu(a);
  og.ui.close();

  world.set(e, Arena, { tick: a.tick + 1, presses, claims: m.claims, yaw: m.yaw });
  if (turntable !== null) world.set(turntable, og.Transform, { quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, m.yaw) });
});

function mesh(world: World, name: string, geometry: Geometry, material: Material, position: { x: number; y: number; z: number }, scale: { x: number; y: number; z: number }, extra: { layers?: number; parent?: Entity; castShadow?: boolean } = {}): Entity {
  const e = must(world.spawn({ name, position, ...(extra.parent !== undefined ? { parent: extra.parent } : {}) }));
  world.set(e, og.Transform, { scale });
  world.insert(e, og.Mesh, { geometry, material, castShadow: extra.castShadow ?? true, receiveShadow: true, layers: extra.layers ?? 1 });
  return e;
}

export default og.game({
  tick: 60,
  seed: 1,
  systems: [Ui],
  start(world: World) {
    const font = must(og.load.font(world, "fonts/Inter-Display.ttf"));
    og.ui.configure(world, { font, displayFont: font } as never);

    const cam = must(world.resource(og.Camera));
    cam.position = EYE;
    cam.quaternion = og.transform.lookAt(EYE, LOOK);
    cam.fov = 55;
    cam.background = { r: 0.05, g: 0.06, b: 0.09 };

    const sun = must(world.spawn({ name: "sun", quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: -0.4, y: -0.8, z: -0.45 }) }));
    world.insert(sun, og.DirectionalLight, { intensity: 3 });

    const box = must(og.render.boxGeometry(world, {}));
    const sphere = must(og.render.sphereGeometry(world, {}));
    const mat = (color: { r: number; g: number; b: number }, roughness = 0.8) => must(og.render.standardMaterial(world, { color, roughness }));

    // The arena: a floor and 100 pillars in ten rows going away from the camera.
    mesh(world, "floor", box, mat({ r: 0.3, g: 0.29, b: 0.27 }, 0.95), { x: 0, y: -0.1, z: -8 }, { x: 40, y: 0.2, z: 40 }, { castShadow: false });
    const stone = mat({ r: 0.55, g: 0.5, b: 0.44 });
    pillars = [];
    for (let row = 0; row < 10; row++) {
      for (let col = 0; col < 10; col++) {
        const x = (col - 4.5) * 2.2 + (row % 2) * 0.6;
        const z = 2 - row * 2.4;
        pillars.push(mesh(world, `pillar-${row * 10 + col}`, box, stone, { x, y: 0.75, z }, { x: 0.6, y: 1.5, z: 0.6 }));
      }
    }
    // A wall across the back rows' left: the nameplates behind it fade.
    mesh(world, "wall", box, mat({ r: 0.4, g: 0.36, b: 0.33 }), { x: -6, y: 1.6, z: -12.5 }, { x: 8, y: 3.2, z: 0.5 });


    // The terminal: a plane facing the camera showing the panel's target (unlit: emissive).
    const panelTarget = must(og.render.renderTarget(world, { width: 960, height: 640, format: "hdr" }));
    terminalTexture = panelTarget.texture;
    const away = { x: 2 * TERMINAL.x - EYE.x, y: 2 * TERMINAL.y - EYE.y, z: 2 * TERMINAL.z - EYE.z };
    const facing = og.transform.lookAt(TERMINAL, away);
    terminal = must(world.spawn({ name: "terminal", position: TERMINAL, quaternion: facing }));
    const screen = must(og.render.planeGeometry(world, { width: 3.6, height: 2.4 }));
    world.insert(terminal, og.Mesh, { geometry: screen, material: must(og.render.standardMaterial(world, { color: { r: 0, g: 0, b: 0 }, emissive: { r: 1, g: 1, b: 1 }, emissiveMap: panelTarget.texture, emissiveIntensity: 1 })) });
    mesh(world, "terminal-bezel", box, mat({ r: 0.05, g: 0.05, b: 0.06 }, 0.4), { x: 0, y: 0, z: -0.06 }, { x: 3.85, y: 2.65, z: 0.1 }, { parent: terminal });
    mesh(world, "terminal-stand", box, mat({ r: 0.12, g: 0.12, b: 0.13 }, 0.5), { x: TERMINAL.x, y: 1.1, z: TERMINAL.z - 0.2 }, { x: 0.3, y: 2.2, z: 0.3 });

    // The turntable: a figure on layer 1 only (mask 2), drawn by its own camera into the viewport's target.
    const stageTarget = must(og.render.renderTarget(world, { width: 600, height: 600, format: "hdr" }));
    stageTexture = stageTarget.texture;
    turntable = must(world.spawn({ name: "turntable", position: STAGE }));
    const skin = mat({ r: 0.9, g: 0.72, b: 0.58 }, 0.6);
    const cloth = mat({ r: 0.25, g: 0.35, b: 0.85 }, 0.7);
    const metal = must(og.render.standardMaterial(world, { color: { r: 0.85, g: 0.7, b: 0.3 }, roughness: 0.3, metalness: 1 }));
    const part = { layers: 2, parent: turntable, castShadow: false };
    mesh(world, "fig-plinth", box, mat({ r: 0.2, g: 0.18, b: 0.25 }), { x: 0, y: 0.1, z: 0 }, { x: 1.6, y: 0.2, z: 1.6 }, part);
    mesh(world, "fig-legs", box, cloth, { x: 0, y: 0.75, z: 0 }, { x: 0.55, y: 1.1, z: 0.35 }, part);
    mesh(world, "fig-body", box, cloth, { x: 0, y: 1.6, z: 0 }, { x: 0.8, y: 0.8, z: 0.45 }, part);
    mesh(world, "fig-belt", box, metal, { x: 0, y: 1.22, z: 0 }, { x: 0.82, y: 0.1, z: 0.47 }, part);
    mesh(world, "fig-arm-l", box, cloth, { x: -0.55, y: 1.5, z: 0 }, { x: 0.22, y: 0.9, z: 0.25 }, part);
    mesh(world, "fig-arm-r", box, cloth, { x: 0.55, y: 1.5, z: 0 }, { x: 0.22, y: 0.9, z: 0.25 }, part);
    mesh(world, "fig-head", sphere, skin, { x: 0, y: 2.25, z: 0 }, { x: 0.5, y: 0.5, z: 0.5 }, part);
    mesh(world, "fig-visor", box, metal, { x: 0, y: 2.28, z: 0.22 }, { x: 0.4, y: 0.1, z: 0.1 }, part);
    const stageEye = { x: STAGE.x, y: 1.5, z: 4.2 };
    const stageCam = must(world.spawn({ name: "stage-camera", position: stageEye, quaternion: og.transform.lookAt(stageEye, { x: STAGE.x, y: 1.2, z: 0 }) }));
    world.insert(stageCam, og.CameraView, { target: stageTarget.target, fov: 35, layers: 2, exposure: 1 });

    world.insert(must(world.spawn({ name: "arena" })), Arena);
  },
});
