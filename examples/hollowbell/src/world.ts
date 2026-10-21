// The hollow, built once in `start` from the Tiled map (assets/maps/hollow.tmj):
// the tilemap itself (the far hills scrolling at their parallax, the back
// wall, the auto-tiled rock whose solid tiles become static colliders, the
// planks, the torches), then one entity per map object: Wren on
// og.PlatformerController, the enemies, the pickups drawn from the items'
// texture array, the checkpoint lanterns, one-way plank colliders, the
// lift, the stones (dynamic bodies), the torches' and crystals' 2D lights,
// rock that blocks the light (og.Occluder2D), the gloam's camera zone and
// the areas. Everything the systems need about the map is on those rows,
// so nothing of the map is kept outside the world.

import { og, BODY_DYNAMIC, BODY_KINEMATIC, SHAPE_BOX, SHAPE_SPHERE, bool, defineComponent, f32, str, u32 } from "@origingame/engine";
import type { Entity, FlipbookInfo, TextureArrayInfo, TileObject, Value, World } from "@origingame/engine";
import { Bell, Bits, Draw, Enemy, Flicker, Fx, FxKind, Game, Hero, Item, Kind, Lantern, Layer, Lift, Mole, Pickup, Stone, Tuning } from "./components.ts";

export function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** The map's size in cells (one cell, 16 px, is one world unit). Its top-left corner sits at (0, MAP_H). */
export const MAP_W = 96;
export const MAP_H = 48;
/** The camera's view: 15 cells tall, the 240-line pixel grid (looks/hollowbell.ogt) at 16 px a cell, 3x at 720p (FINDINGS.md #7, fixed: it was the 224-line preset's 14). */
export const VIEW_H = 15;

/** The hero's body: a box from the feet up. */
export const HERO_BOX = { w: 0.7, h: 1.25 } as const;

/** The flipbooks' clips by name: src/art checks the order at start. */
export const HERO_CLIP = { idle: 0, run: 1, jump: 2, fall: 3, attack: 4, hurt: 5 } as const;
export const BEETLE_CLIP = { walk: 0, wind: 1, dash: 2, hurt: 3 } as const;
export const MOTH_CLIP = { fly: 0, dive: 1, hurt: 2 } as const;
export const MOLE_CLIP = { idle: 0, talk: 1 } as const;
export const BELL_CLIP = { still: 0, ring: 1 } as const;

/** An area of the map: its name's string key and the 2D ambient light inside it. */
export const Area = defineComponent("Area", {
  index: u32(0, { doc: "Its order in the map: the first area holding the hero names it." }),
  minX: f32(0, { unit: "m" }),
  minY: f32(0, { unit: "m" }),
  maxX: f32(0, { unit: "m" }),
  maxY: f32(0, { unit: "m" }),
  ambient: f32(1),
  key: str(32, "", { doc: "The string table key of its name (area.*)." }),
});

/** The sky behind the village: follows the camera. */
export const Sky = defineComponent("Sky", { on: bool(true) });

/** The pit's brambles: Wren below their top within minX..maxX is hurt and put back on the near edge. */
export const Hazard = defineComponent("Hazard", {
  minX: f32(0, { unit: "m" }),
  maxX: f32(0, { unit: "m" }),
  y: f32(0, { unit: "m" }),
});

function checkClips(info: FlipbookInfo, names: Record<string, number>, sheet: string): void {
  for (const [name, index] of Object.entries(names)) {
    const at = og.anim.clip(info, name);
    if (at !== index) throw new Error(`${sheet}: clip ${name} is ${at}, the code expects ${index} (tools/make-assets.ts writes the sheet)`);
  }
}

export interface Art {
  hero: FlipbookInfo;
  beetle: FlipbookInfo;
  moth: FlipbookInfo;
  mole: FlipbookInfo;
  bell: FlipbookInfo;
  items: TextureArrayInfo;
}

export function loadArt(world: World): Art {
  const sheet = (path: string, clips: Record<string, number>) => {
    const info = must(og.load.flipbook(world, path));
    checkClips(info, clips, path);
    return info;
  };
  return {
    hero: sheet("sprites/wren.ogt", HERO_CLIP),
    beetle: sheet("sprites/beetle.ogt", BEETLE_CLIP),
    moth: sheet("sprites/moth.ogt", MOTH_CLIP),
    mole: sheet("sprites/mole.ogt", MOLE_CLIP),
    bell: sheet("sprites/bell.ogt", BELL_CLIP),
    items: must(og.load.textureArray(world, "sprites/items.ogt")),
  };
}

/** A map object's point (or its rectangle's top-left corner) in world units. */
function at(o: TileObject): { x: number; y: number } {
  return { x: o.position.x, y: MAP_H + o.position.y };
}

function num(o: TileObject, key: string, fallback: number): number {
  const v = Number(o.properties[key]);
  return Number.isFinite(v) && o.properties[key] !== undefined ? v : fallback;
}

function spawn(world: World, name: string, x: number, y: number): Entity {
  return must(world.spawn({ name, position: { x, y, z: 0 } }));
}

/** The camera: orthographic, VIEW_H cells tall, over the ink. */
export function viewHalfWidth(world: World): number {
  const caps = og.render.capabilities(world);
  const aspect = caps.width > 0 && caps.height > 0 ? caps.width / caps.height : 16 / 9;
  return (VIEW_H * aspect) / 2;
}

/**
 * `g` is the caller's handle on Game, passed down rather than fetched again
 * here: a second world.resource(Game) handle does not see the first one's
 * writes, nor the first this one's (FINDINGS.md #6).
 */
export function buildHollow(world: World, art: Art, g: Value<typeof Game>): void {
  const t = must(world.resource(Tuning));
  const info = must(og.load.tilemap(world, "maps/hollow.tmj"));
  if (info.width !== MAP_W || info.height !== MAP_H || info.tileWidth !== 1) throw new Error(`maps/hollow.tmj is ${info.width}x${info.height} cells of ${info.tileWidth}: the game expects ${MAP_W}x${MAP_H} of 1`);

  const camera = must(world.resource(og.Camera));
  camera.orthographic = true;
  camera.height = VIEW_H;
  camera.background = { r: 0.0034, g: 0.0030, b: 0.0060 };
  camera.position = { x: 16, y: MAP_H - 8, z: 50 };

  const lighting = must(world.resource(og.Lighting2D));
  lighting.enabled = true;
  lighting.ambient = { r: 1, g: 1, b: 1 };
  // The map, the actors, props and effects take the light; the sky (Draw.sky) is below the lit
  // range. FINDINGS.md #9 (og.Lighting2D never lit a tilemap's layers) is fixed in the engine, so
  // the map's tint (MapShade) and the light-pool sprites it needed are gone.
  lighting.minLayer = Draw.sky + 1;
  lighting.maxLayer = 50;
  lighting.resolution = 0.5;

  g.items = art.items.array as unknown as number;
  const map = spawn(world, "map", 0, MAP_H);
  must(world.insert(map, og.Tilemap, { map: info.map, layer: Draw.map, colliderLayers: Bits.World, friction: 0.6 }));
  g.map = map as unknown as number;

  const sky = spawn(world, "sky", 16, MAP_H - 6);
  must(world.insert(sky, og.Sprite, { texture: must(og.load.texture(world, "sprites/sky.png")), width: 80, height: VIEW_H + 2, layer: Draw.sky }));
  must(world.insert(sky, Sky));

  const tiles = must(og.load.texture(world, "tiles/hollow.png"));
  const spark = must(og.load.texture(world, "fx/spark.png"));
  const embers = must(og.load.emitter(world, "fx/embers.ogt"));
  const motes = must(og.load.emitter(world, "fx/motes.ogt"));
  let enemies = 0;
  let areas = 0;
  for (const o of info.objects) {
    const p = at(o);
    switch (o.kind) {
      case "spawn":
        g.spawnX = p.x;
        g.spawnY = p.y;
        spawnHero(world, g, art, p.x, p.y, t.lightRadius);
        break;
      case "bell": {
        const e = spawn(world, "bell", p.x, p.y);
        must(world.insert(e, og.Sprite, { texture: art.bell.texture, width: 2, height: 3, anchor: { x: 0.5, y: 0 }, layer: Draw.props }));
        must(world.insert(e, og.Flipbook, { sheet: art.bell.sheet, clip: 0 }));
        must(world.insert(e, Bell));
        break;
      }
      case "mole": {
        const e = spawn(world, "mole", p.x, p.y);
        must(world.insert(e, og.Sprite, { texture: art.mole.texture, width: 1, height: 1, anchor: { x: 0.5, y: 0 }, layer: Draw.props, flipX: true }));
        must(world.insert(e, og.Flipbook, { sheet: art.mole.sheet, clip: 0 }));
        must(world.insert(e, Mole));
        // The mole's candle.
        must(world.insert(e, og.Light2D, { color: { r: 1, g: 0.75, b: 0.4 }, intensity: 1.8, radius: 3.5, falloff: 1.3 }));
        break;
      }
      case "chime":
      case "glimmer":
      case "heart": {
        const kind = o.kind === "chime" ? Item.Chime : o.kind === "heart" ? Item.Heart : Item.Glimmer;
        const index = Number(o.name.split("-")[1] ?? 0);
        const size = kind === Item.Chime ? 1 : kind === Item.Heart ? 0.8 : 0.7;
        const e = spawn(world, o.name, p.x, p.y);
        must(world.insert(e, og.Sprite, {
          array: art.items.array,
          arrayLayer: kind === Item.Chime ? Layer.Chime : kind === Item.Heart ? Layer.Heart : Layer.Glimmer,
          width: size,
          height: size,
          layer: Draw.items,
          glow: kind === Item.Chime ? 1.2 : 0.35,
          visible: o.properties["boss"] !== "true",
        }));
        must(world.insert(e, Pickup, { kind, index, baseY: p.y, phase: (index * 0.37) % 1, hidden: o.properties["boss"] === "true" }));
        if (kind === Item.Chime) must(world.insert(e, og.Light2D, { color: { r: 0.55, g: 0.8, b: 1 }, intensity: 0.9, radius: 2.6, falloff: 1.4, castShadows: false }));
        break;
      }
      case "beetle":
      case "moth":
      case "gloam": {
        const kind = o.kind === "beetle" ? Kind.Beetle : o.kind === "moth" ? Kind.Moth : Kind.Gloam;
        spawnEnemy(world, art, o.name, kind, enemies++, p.x, p.y, num(o, "range", 4), t.gloamHp);
        break;
      }
      case "lantern": {
        const index = Number(o.name.split("-")[1] ?? 0);
        const e = spawn(world, o.name, p.x, p.y);
        must(world.insert(e, og.Sprite, { array: art.items.array, arrayLayer: Layer.LanternDark, width: 1, height: 1, anchor: { x: 0.5, y: 0 }, layer: Draw.props }));
        must(world.insert(e, og.Light2D, { color: { r: 1, g: 0.8, b: 0.45 }, intensity: 0, radius: 5, falloff: 1.5 }));
        must(world.insert(e, Lantern, { index }));
        break;
      }
      case "torch":
      case "crystal": {
        const torch = o.kind === "torch";
        const index = Number(o.name.split("-")[1] ?? 0);
        const e = spawn(world, o.name, p.x, p.y);
        must(world.insert(e, og.Light2D, torch
          ? { color: { r: 1, g: 0.62, b: 0.3 }, intensity: 3, radius: 6, falloff: 1.3, sourceRadius: 0.3 }
          : { color: { r: 0.45, g: 0.75, b: 1 }, intensity: 1.4, radius: 3, falloff: 1.5, sourceRadius: 0.15 }));
        must(world.insert(e, Flicker, { base: torch ? 3 : 1.4, rate: torch ? 9 + (index % 4) : 2, phase: index * 1.7 }));
        if (torch) {
          must(world.insert(e, og.ParticleSystem, { emitter: embers, seed: 100 + index }));
          // The flame glows in the 2D glow buffer over the map's flame tile, which its own light now lights (FINDINGS.md #9, fixed).
          const flame = spawn(world, `${o.name}-flame`, p.x, p.y + 0.15);
          must(world.insert(flame, og.Sprite, { texture: spark, width: 0.45, height: 0.6, color: { r: 1, g: 0.75, b: 0.4 }, blend: "additive", glow: 1.4, layer: Draw.props }));
        }
        break;
      }
      case "occluder": {
        const e = spawn(world, o.name, p.x + o.size.x / 2, p.y - o.size.y / 2);
        must(world.insert(e, og.Occluder2D, { width: o.size.x, height: o.size.y }));
        break;
      }
      case "oneway": {
        // A thin box at the top of the plank's row, on the one-way layer (PlatformerController.oneWayLayers).
        const e = spawn(world, o.name, p.x + o.size.x / 2, p.y - 0.125);
        must(world.insert(e, og.Collider, { shape: SHAPE_BOX, size: { x: o.size.x, y: 0.25, z: 1 }, layers: Bits.Plank }));
        break;
      }
      case "lift": {
        const e = spawn(world, "lift", p.x + o.size.x / 2, p.y);
        // The planks of the tileset, drawn as one sprite (tiles 50-52 sit side by side in the atlas).
        must(world.insert(e, og.Sprite, { texture: tiles, uvMin: { x: 32 / 128, y: 96 / 144 }, uvMax: { x: 80 / 128, y: 112 / 144 }, width: 3, height: 1, anchor: { x: 0.5, y: 1 }, layer: Draw.props }));
        must(world.insert(e, og.RigidBody, { kind: BODY_KINEMATIC }));
        must(world.insert(e, og.Collider, { shape: SHAPE_BOX, size: { x: o.size.x, y: 0.25, z: 1 }, offset: { x: 0, y: -0.125, z: 0 }, layers: Bits.Plank }));
        must(world.insert(e, og.Velocity, { linear: { x: num(o, "speed", 2), y: 0, z: 0 } }));
        must(world.insert(e, Lift, { x0: p.x + o.size.x / 2, travel: num(o, "travel", 6), speed: num(o, "speed", 2) }));
        break;
      }
      case "stone": {
        const e = spawn(world, o.name, p.x, p.y + 0.4);
        must(world.insert(e, og.Sprite, { texture: tiles, uvMin: { x: 0, y: 0 }, uvMax: { x: 16 / 128, y: 16 / 144 }, width: 0.8, height: 0.8, layer: Draw.props + 1, castShadow: true }));
        must(world.insert(e, og.RigidBody, { kind: BODY_DYNAMIC, linearDamping: 0.4 }));
        must(world.insert(e, og.Collider, { shape: SHAPE_BOX, size: { x: 0.78, y: 0.78, z: 1 }, layers: Bits.Stone, mask: Bits.World | Bits.Hero | Bits.Stone | Bits.Plank, friction: 0.9, density: 2 }));
        must(world.insert(e, Stone, { x0: p.x, y0: p.y + 0.4 }));
        break;
      }
      case "hazard": {
        const e = spawn(world, o.name, p.x + o.size.x / 2, p.y - o.size.y / 2);
        must(world.insert(e, Hazard, { minX: p.x, maxX: p.x + o.size.x, y: p.y }));
        break;
      }
      case "arena": {
        const e = spawn(world, "arena", p.x + o.size.x / 2, p.y - o.size.y / 2);
        must(world.insert(e, og.CameraZone, { min: { x: p.x, y: p.y - o.size.y }, max: { x: p.x + o.size.x, y: p.y }, priority: 1, scrollTime: 0.5 }));
        break;
      }
      case "area": {
        const e = spawn(world, o.name, p.x + o.size.x / 2, p.y - o.size.y / 2);
        must(world.insert(e, Area, { index: areas++, minX: p.x, maxX: p.x + o.size.x, minY: p.y - o.size.y, maxY: p.y, ambient: num(o, "ambient", 1), key: o.properties["key"] ?? "" }));
        break;
      }
      default:
        throw new Error(`maps/hollow.tmj: an object of kind "${o.kind}" (${o.name}) the game does not place`);
    }
  }
  // One-shot effects the Fx rows play where Game.fx asks (src/props.ts Effects); parked out of sight until then.
  const kinds: [number, string][] = [[FxKind.Dust, "fx/dust.ogt"], [FxKind.Hit, "fx/hit.ogt"], [FxKind.Chime, "fx/chime.ogt"], [FxKind.Burst, "fx/burst.ogt"]];
  for (const [kind, path] of kinds) {
    const e = spawn(world, `fx-${kind}`, -50, -50);
    must(world.insert(e, og.ParticleSystem, { emitter: must(og.load.emitter(world, path)), seed: 31 + kind }));
    must(world.insert(e, Fx, { kind }));
  }
  // Motes drifting in the caves' air: a texture array's layers at random.
  [[40, MAP_H - 27], [62, MAP_H - 40], [22, MAP_H - 40], [80, MAP_H - 27], [5, MAP_H - 24]].forEach(([x, y], i) => {
    const e = spawn(world, `motes-${i}`, x!, y!);
    must(world.insert(e, og.ParticleSystem, { emitter: motes, seed: 7 + i }));
  });
}

function spawnHero(world: World, g: Value<typeof Game>, art: Art, x: number, y: number, light: number): void {
  const e = spawn(world, "hero", x, y);
  must(world.insert(e, og.Sprite, { texture: art.hero.texture, width: 1.5, height: 1.5, anchor: { x: 0.5, y: 0 }, layer: Draw.hero }));
  must(world.insert(e, og.Flipbook, { sheet: art.hero.sheet, clip: HERO_CLIP.idle }));
  must(world.insert(e, og.Collider, { shape: SHAPE_BOX, size: { x: HERO_BOX.w, y: HERO_BOX.h, z: 1 }, offset: { x: 0, y: HERO_BOX.h / 2, z: 0 }, layers: Bits.Hero, mask: Bits.World | Bits.Stone | Bits.Plank }));
  // Its run holds maxSpeed on flat ground with every default skin and snap (FINDINGS.md #16, fixed:
  // it stuttered to 0 m/s every few ticks).
  must(world.insert(e, og.PlatformerController, { oneWayLayers: Bits.Plank, wallJumps: false, coyoteTime: 0.1, jumpBuffer: 0.12, maxFallSpeed: 18 }));
  must(world.insert(e, og.Light2D, { color: { r: 1, g: 0.86, b: 0.6 }, intensity: 2.4, radius: light, falloff: 1.2, sourceRadius: 0.2 }));
  // Steady (rate 0: 0.894 of the base), out in daylight, where it would light the village's hills.
  must(world.insert(e, Flicker, { base: 2.7, rate: 0, phase: 0, dusk: true }));
  const half = viewHalfWidth(world);
  must(world.insert(e, og.Follow2D, {
    offset: { x: 0, y: 1.2 },
    deadZone: { x: 0.8, y: 0.6 },
    smoothTime: 0.12,
    smoothTimeY: 0.25,
    lookAhead: 0.2,
    lookAheadMax: { x: 2.5, y: 1 },
    clamp: true,
    boundsMin: { x: half, y: VIEW_H / 2 },
    boundsMax: { x: MAP_W - half, y: MAP_H - VIEW_H / 2 },
  }));
  must(world.insert(e, Hero));
  g.hero = e as unknown as number;
}

function spawnEnemy(world: World, art: Art, name: string, kind: number, index: number, x: number, y: number, range: number, gloamHp: number): void {
  const e = spawn(world, name, x, y);
  if (kind === Kind.Moth) {
    must(world.insert(e, og.Sprite, { texture: art.moth.texture, width: 1, height: 1, layer: Draw.enemies, castShadow: true }));
    must(world.insert(e, og.Flipbook, { sheet: art.moth.sheet, clip: MOTH_CLIP.fly }));
    must(world.insert(e, og.Collider, { shape: SHAPE_SPHERE, radius: 0.4, sensor: 1, layers: Bits.Enemy }));
  } else {
    const big = kind === Kind.Gloam;
    const w = big ? 3.3 : 1.5;
    const h = big ? 2.2 : 1;
    must(world.insert(e, og.Sprite, { texture: art.beetle.texture, width: w, height: h, anchor: { x: 0.5, y: 0 }, layer: Draw.enemies, castShadow: true, color: big ? { r: 0.85, g: 0.55, b: 1 } : { r: 1, g: 1, b: 1 } }));
    must(world.insert(e, og.Flipbook, { sheet: art.beetle.sheet, clip: BEETLE_CLIP.walk }));
    must(world.insert(e, og.Collider, { shape: SHAPE_BOX, size: { x: w * 0.8, y: h * 0.75, z: 1 }, offset: { x: 0, y: h * 0.375, z: 0 }, sensor: 1, layers: Bits.Enemy }));
  }
  must(world.insert(e, og.RigidBody, { kind: BODY_KINEMATIC }));
  must(world.insert(e, Enemy, { kind, index, hp: kind === Kind.Gloam ? gloamHp : kind === Kind.Beetle ? 2 : 1, homeX: x, homeY: y, range, dir: -1 }));
}
