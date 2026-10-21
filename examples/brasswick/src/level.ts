// The vault: the three levels' maps placed side by side (level n's top-left
// corner at x = n * LEVEL_PITCH, y = 0), and every thing in them spawned
// once in `start` from the maps' objects (og.load.tilemap's TileObject:
// kind, name, rect, props). All three levels exist at once; a level is
// played inside its og.CameraZone, and the systems reset its things when
// it starts again (Game.reset, src/puzzle.ts).

import { og, BODY_DYNAMIC, BODY_KINEMATIC, JOINT_PRISMATIC, MOTOR_POSITION, SHAPE_BOX } from "@origingame/engine";
import type { Entity, FlipbookInfo, TileObject, Value, World } from "@origingame/engine";
import { Bits, Crate, Door, Draw, Exit, FxKind, FxSlot, Game, Gate, Gear, GEARS, Hazard, Hero, LEVELS, Lamp, Lift, Member, Plate, Tick, Tuning, Zone } from "./components.ts";

export const LEVEL_PITCH = 100;
/** The view: 360 logical lines of 16-pixel cells (looks/brasswick.ogt), 22.5 cells tall. */
export const VIEW_H = 22.5;

export function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

type GameV = Value<typeof Game>;

/** Every sheet and texture the levels draw, loaded in start in one order (a save keeps handles as numbers). */
export interface Art {
  wick: FlipbookInfo;
  tick: FlipbookInfo;
  gear: FlipbookInfo;
  plate: FlipbookInfo;
  lamp: FlipbookInfo;
  hatch: FlipbookInfo;
  crate: ReturnType<typeof tex>;
  door: ReturnType<typeof tex>;
  gate: ReturnType<typeof tex>;
  lift: ReturnType<typeof tex>;
  puff: ReturnType<typeof tex>;
}

function tex(world: World, path: string) {
  return must(og.load.texture(world, path));
}

export function loadArt(world: World): Art {
  const fb = (p: string) => must(og.load.flipbook(world, p));
  return {
    wick: fb("sprites/wick.ogt"),
    tick: fb("sprites/tick.ogt"),
    gear: fb("sprites/gear.ogt"),
    plate: fb("sprites/plate.ogt"),
    lamp: fb("sprites/lamp.ogt"),
    hatch: fb("sprites/hatch.ogt"),
    crate: tex(world, "sprites/crate.png"),
    door: tex(world, "sprites/door.png"),
    gate: tex(world, "sprites/gate.png"),
    lift: tex(world, "sprites/lift.png"),
    puff: tex(world, "fx/puff.png"),
  };
}

export function clip(info: FlipbookInfo, name: string): number {
  return og.anim.clip(info, name);
}

function spawn(world: World, name: string, x: number, y: number): Entity {
  return must(world.spawn({ name, position: { x, y, z: 0 } }));
}

function num(o: TileObject, key: string, fallback: number): number {
  const v = o.properties[key];
  const n = v === undefined ? NaN : Number(v);
  return Number.isFinite(n) ? n : fallback;
}

/** The world point of an object's corner or centre, for a map whose top-left corner is at (ox, 0). */
function worldOf(o: TileObject, ox: number) {
  return {
    cx: ox + o.center.x,
    cy: o.center.y,
    left: ox + o.position.x,
    top: o.position.y,
    bottom: o.position.y - o.size.y,
    w: o.size.x,
    h: o.size.y,
  };
}

/** Wick: a PlatformerController on a box, a flipbook, his flame's light, the camera's target. */
export function spawnWick(world: World, art: Art, x: number, y: number): Entity {
  const t = must(world.resource(Tuning));
  const e = spawn(world, "wick", x, y);
  must(world.insert(e, og.Collider, { shape: SHAPE_BOX, size: { x: 0.8, y: 1.5, z: 1 }, offset: { x: 0, y: 0.75, z: 0 }, layers: Bits.Wick, mask: Bits.World | Bits.Grate | Bits.Crate | Bits.Door | Bits.Lift }));
  must(world.insert(e, og.PlatformerController, {
    maxSpeed: t.maxSpeed,
    jumpHeight: t.jumpHeight,
    timeToApex: t.timeToApex,
    coyoteTime: t.coyoteTime,
    jumpBuffer: t.jumpBuffer,
    wallJumps: true,
    wallSlideSpeed: t.wallSlideSpeed,
    wallJump: { x: t.wallJumpX, y: t.wallJumpY },
    oneWayLayers: Bits.Grate,
    maxFallSpeed: 18,
  }));
  must(world.insert(e, og.Sprite, { texture: art.wick.texture, width: 1.5, height: 2, anchor: { x: 0.5, y: 0 }, layer: Draw.wick }));
  must(world.insert(e, og.Flipbook, { sheet: art.wick.sheet, clip: clip(art.wick, "idle") }));
  must(world.insert(e, og.Light2D, { color: { r: 1, g: 0.78, b: 0.45 }, intensity: 1.1, radius: 4.5, falloff: 1.4, sourceRadius: 0.2 }));
  must(world.insert(e, og.Follow2D, { deadZone: { x: 0.8, y: 0.6 }, smoothTime: 0.15, smoothTimeY: 0.3, lookAhead: 0.2, lookAheadMax: { x: 3, y: 1 } }));
  must(world.insert(e, Hero));
  return e;
}

/** Places the three maps and spawns every object in them. */
export function buildVault(world: World, art: Art, g: GameV): void {
  const doorIds = new Map<string, number>();
  const plates: { e: Entity; opens: string }[] = [];
  let nextId = 0;
  const gearTex = art.gear;
  const dust = must(og.load.emitter(world, "fx/dust.ogt"));
  const gearFx = must(og.load.emitter(world, "fx/gear.ogt"));
  const stompFx = must(og.load.emitter(world, "fx/stomp.ogt"));
  const steam = must(og.load.emitter(world, "fx/steam.ogt"));
  const exitFx = must(og.load.emitter(world, "fx/exit.ogt"));

  for (let n = 0; n < LEVELS; n++) {
    const info = must(og.load.tilemap(world, `maps/level-${n + 1}.ogt`));
    const ox = n * LEVEL_PITCH;
    const map = spawn(world, `l${n + 1}-map`, ox, 0);
    // The map's layers draw from Draw.map - 4 (far, z 0) to Draw.map + 2 (decor, z 6); z 3 is free for the doors and
    // the gate (Draw.map - 1), which slide up behind the plate terrain (z 4).
    must(world.insert(map, og.Tilemap, { map: info.map, layer: Draw.map - 4, colliderLayers: Bits.World, friction: 0.7 }));
    must(world.insert(map, Member, { level: n }));
    let gears = 0;
    for (const o of info.objects) {
      const p = worldOf(o, ox);
      const member = (e: Entity) => must(world.insert(e, Member, { level: n }));
      switch (o.kind) {
        case "spawn":
          // A marker: where Wick's feet start the level (src/ui.ts startLevel reads it).
          member(spawn(world, o.name, p.cx, p.bottom));
          break;
        case "zone": {
          const e = spawn(world, o.name, p.cx, p.cy);
          const amb = (o.properties["ambient"] ?? "0.5 0.5 0.5").split(" ").map(Number);
          must(world.insert(e, og.CameraZone, { min: { x: p.left, y: p.bottom }, max: { x: p.left + p.w, y: p.top }, scrollTime: 0 }));
          must(world.insert(e, Zone, { par: num(o, "par", 60), ambient: amb[0] ?? 0.5, ambientG: amb[1] ?? 0.5, ambientB: amb[2] ?? 0.5, title: o.properties["title"] ?? "" }));
          member(e);
          break;
        }
        case "crate": {
          const e = spawn(world, o.name, p.cx, p.cy);
          must(world.insert(e, og.Sprite, { texture: art.crate, width: 1, height: 1, layer: Draw.crate, castShadow: true }));
          must(world.insert(e, og.RigidBody, { kind: BODY_DYNAMIC, lock: 32, linearDamping: 0.5 }));
          must(world.insert(e, og.Collider, { shape: SHAPE_BOX, size: { x: 0.98, y: 0.98, z: 1 }, friction: 0.6, density: 2, layers: Bits.Crate, mask: Bits.World | Bits.Grate | Bits.Wick | Bits.Crate | Bits.Door | Bits.Lift }));
          must(world.insert(e, Crate, { homeX: p.cx, homeY: p.cy, checkX: p.cx, checkY: p.cy }));
          member(e);
          break;
        }
        case "plate": {
          const e = spawn(world, o.name, p.cx, p.bottom);
          must(world.insert(e, og.Sprite, { texture: art.plate.texture, width: 1, height: 1, anchor: { x: 0.5, y: 0 }, layer: Draw.plate }));
          must(world.insert(e, og.Flipbook, { sheet: art.plate.sheet, clip: clip(art.plate, "up") }));
          must(world.insert(e, Plate, { x: p.cx, y: p.bottom }));
          plates.push({ e, opens: o.properties["opens"] ?? "" });
          member(e);
          break;
        }
        case "door": {
          const id = nextId++;
          doorIds.set(o.name, id);
          const e = spawn(world, o.name, p.cx, p.cy);
          must(world.insert(e, og.Sprite, { texture: art.door, width: 1, height: 3, layer: Draw.map - 1, castShadow: true }));
          must(world.insert(e, og.RigidBody, { kind: BODY_KINEMATIC }));
          must(world.insert(e, og.Velocity, {}));
          must(world.insert(e, og.Collider, { shape: SHAPE_BOX, size: { x: 1, y: 3, z: 1 }, layers: Bits.Door }));
          must(world.insert(e, Door, { id, closedY: p.cy, openY: p.cy + 3 }));
          member(e);
          break;
        }
        case "gate": {
          // A slider on a prismatic joint to the world (og.Joint, og-game §13): its position motor
          // holds it shut (0) or slides it up its travel when the plate is pressed (src/puzzle.ts).
          const id = nextId++;
          doorIds.set(o.name, id);
          const e = spawn(world, o.name, p.cx, p.cy);
          must(world.insert(e, og.Sprite, { texture: art.gate, width: 1, height: 3, layer: Draw.map - 1, castShadow: true }));
          must(world.insert(e, og.RigidBody, { kind: BODY_DYNAMIC, gravityScale: 0, lock: 32, mass: 40 }));
          must(world.insert(e, og.Collider, { shape: SHAPE_BOX, size: { x: 0.96, y: 3, z: 1 }, layers: Bits.Door, mask: Bits.Wick | Bits.Crate }));
          must(world.insert(e, og.Joint, { kind: JOINT_PRISMATIC, otherAnchor: { x: p.cx, y: p.cy, z: 0 }, axis: { x: 0, y: 1, z: 0 }, limits: true, min: 0, max: 3, motor: MOTOR_POSITION, motorTarget: 0, motorStiffness: 600, motorDamping: 60 }));
          must(world.insert(e, Gate, { id, travel: 3 }));
          member(e);
          break;
        }
        case "gear": {
          const e = spawn(world, o.name, p.cx, p.cy);
          must(world.insert(e, og.Sprite, { texture: gearTex.texture, width: 0.75, height: 0.75, layer: Draw.gear, glow: 0.8 }));
          must(world.insert(e, og.Flipbook, { sheet: gearTex.sheet, clip: clip(gearTex, "spin"), speed: 0.8 + ((gears * 37) % 5) / 10 }));
          must(world.insert(e, Gear, { index: gears, baseY: p.cy, phase: (gears * 0.31) % 1 }));
          gears++;
          member(e);
          break;
        }
        case "lamp": {
          const e = spawn(world, o.name, p.cx, p.bottom);
          must(world.insert(e, og.Sprite, { texture: art.lamp.texture, width: 1, height: 2, anchor: { x: 0.5, y: 0 }, layer: Draw.prop, glow: 0 }));
          must(world.insert(e, og.Flipbook, { sheet: art.lamp.sheet, clip: clip(art.lamp, "dark") }));
          must(world.insert(e, og.Light2D, { color: { r: 1, g: 0.8, b: 0.5 }, intensity: 0, radius: 7, falloff: 1.3, sourceRadius: 0.3 }));
          must(world.insert(e, Lamp, { index: Number(o.name.split("-").pop()) }));
          member(e);
          break;
        }
        case "exit": {
          const e = spawn(world, o.name, p.cx, p.bottom);
          must(world.insert(e, og.Sprite, { texture: art.hatch.texture, width: 2, height: 2, anchor: { x: 0.5, y: 0 }, layer: Draw.hatch, glow: 0.4 }));
          must(world.insert(e, og.Flipbook, { sheet: art.hatch.sheet, clip: clip(art.hatch, "shut") }));
          must(world.insert(e, og.Light2D, { color: { r: 0.45, g: 1, b: 0.85 }, intensity: 0.9, radius: 4, falloff: 1.5, castShadows: false }));
          must(world.insert(e, og.ParticleSystem, { emitter: exitFx, seed: n + 1, startTick: 0 }));
          must(world.insert(e, Exit));
          member(e);
          break;
        }
        case "tick": {
          const e = spawn(world, o.name, p.cx, p.bottom);
          must(world.insert(e, og.Sprite, { texture: art.tick.texture, width: 1.5, height: 1, anchor: { x: 0.5, y: 0 }, layer: Draw.tick }));
          must(world.insert(e, og.Flipbook, { sheet: art.tick.sheet, clip: clip(art.tick, "walk") }));
          must(world.insert(e, Tick, { minX: ox + num(o, "min", 0) + 0.75, maxX: ox + num(o, "max", 0) - 0.75, homeX: p.cx, dir: 1 }));
          member(e);
          break;
        }
        case "light": {
          const e = spawn(world, o.name, p.cx, p.cy);
          must(world.insert(e, og.Sprite, { texture: art.puff, width: 0.6, height: 0.6, layer: Draw.prop, glow: 1.5, color: { r: 1, g: 0.75, b: 0.35 }, blend: "additive" }));
          must(world.insert(e, og.Light2D, { color: { r: 1, g: 0.72, b: 0.4 }, intensity: 1.3, radius: 6.5, falloff: 1.4, sourceRadius: 0.25 }));
          member(e);
          break;
        }
        case "vent": {
          const e = spawn(world, o.name, p.cx, p.bottom + 0.2);
          must(world.insert(e, og.ParticleSystem, { emitter: steam, seed: 11 + n, startTick: 0 }));
          member(e);
          break;
        }
        case "lift": {
          // A kinematic body moved by its og.Velocity (og-game §13); the Lifts system turns it round.
          const e = spawn(world, o.name, p.left + 1.5, p.top - 0.25);
          const to = ox + num(o, "to", 0) + 1.5;
          must(world.insert(e, og.Sprite, { texture: art.lift, width: 3, height: 0.5, layer: Draw.prop }));
          must(world.insert(e, og.RigidBody, { kind: BODY_KINEMATIC }));
          must(world.insert(e, og.Velocity, { linear: { x: must(world.resource(Tuning)).liftSpeed, y: 0, z: 0 } }));
          must(world.insert(e, og.Collider, { shape: SHAPE_BOX, size: { x: 3, y: 0.5, z: 1 }, layers: Bits.Lift }));
          must(world.insert(e, Lift, { minX: p.left + 1.5, maxX: to, homeX: p.left + 1.5 }));
          member(e);
          break;
        }
        case "oneway": {
          // A grate: a thin static collider on the one-way layer at the top of its cells (og.PlatformerController.oneWayLayers).
          // A map's tile colliders all take og.Tilemap.colliderLayers, so a one-way layer of tiles cannot be told apart from the plate (FINDINGS.md).
          const e = spawn(world, o.name, p.cx, p.top - 0.125);
          must(world.insert(e, og.Collider, { shape: SHAPE_BOX, size: { x: p.w, y: 0.25, z: 1 }, layers: Bits.Grate, friction: 0.7 }));
          member(e);
          break;
        }
        case "spikes": {
          const e = spawn(world, o.name, p.cx, p.cy);
          must(world.insert(e, Hazard, { x0: p.left, y0: p.bottom, x1: p.left + p.w, y1: p.top }));
          member(e);
          break;
        }
        default:
          throw new Error(`${o.name}: an object of kind '${o.kind}' this game does not know`);
      }
    }
    if (gears !== GEARS[n]) throw new Error(`level ${n + 1} has ${gears} gears, the game counts ${GEARS[n]}`);
  }
  for (const p of plates) {
    const id = doorIds.get(p.opens);
    if (id === undefined) throw new Error(`a plate opens '${p.opens}', which no door or gate is`);
    must(world.set(p.e, Plate, { target: id }));
  }
  // The effects' emitters, one per kind, moved and restarted where an effect is asked for (src/feel.ts).
  for (const [kind, emitter] of [[FxKind.Dust, dust], [FxKind.Gear, gearFx], [FxKind.Stomp, stompFx]] as const) {
    const e = spawn(world, `fx-${kind}`, 0, 10);
    // Hidden until first asked for: startTick is a u32, so a one-shot spawned for later fires at tick 0 unless hidden (FINDINGS.md).
    must(world.insert(e, og.ParticleSystem, { emitter, seed: 3 + kind, startTick: 0, visible: false }));
    must(world.insert(e, FxSlot, { kind }));
  }
  void g;
}
