// Where the fight stands, in either view (src/creation.ts's `view`):
//   side-2d      the battle side-on on the ash plain (assets/sprites/
//                battlefield.png): the heroes on the left facing right,
//                the foes on the right facing them, all standing on y = 0;
//   top-down-2d  the same battle as a war table seen from above (assets/
//                sprites/table.png): each unit a round token, the heroes'
//                row near the player, the foes' across the felt.
// The rules (src/run.ts) are the same in both; only these spots, the clips
// (`<unit>` or `<unit>-token`) and the camera's frame differ. The Stage
// system dresses the units (their clip, size, shown or not, a fall's fade),
// sizes the backdrop and the camera to the surface, and parks the cards
// that flew.

import { og, defineSystem } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";
import { Foe, Game, Hero, HEROES, Phase } from "./components.ts";
import { creation } from "./creation.ts";
import { FOES, FOE_SLOTS, MAP } from "./foes.ts";
import { clipOf } from "./art.ts";

export const TOP = (creation.view as string) === "top-down-2d";

/** A unit's sprite size in world units (16 px of art a unit: a 48 px cell is 3). */
export function unitSize(kind: "hero" | number): number {
  if (kind === "hero") return 3;
  const d = FOES[kind]!;
  if (TOP) return d.boss ? 3.8 : d.elite ? 3.4 : 3;
  return d.boss ? 4.6 : d.elite ? 4 : 3;
}

/** A hero's spot: where its feet (side-on) or its token's foot (above) stand. */
export function heroSpot(seat: number, party: number): { x: number; y: number } {
  if (TOP) return { x: party > 1 ? (seat === 0 ? -2.2 : 2.2) : 0, y: -3.3 };
  return { x: party > 1 ? (seat === 0 ? -7 : -3.8) : -5, y: 0 };
}

/** A foe slot's spot in a fight of `count` foes. */
export function foeSpot(slot: number, count: number): { x: number; y: number } {
  if (TOP) {
    const xs = count === 1 ? [0] : count === 2 ? [-2.4, 2.4] : [-4, 0, 4];
    return { x: xs[slot] ?? 0, y: -0.2 };
  }
  const xs = count === 1 ? [5] : count === 2 ? [3.6, 7.2] : [2.2, 5.2, 8.2];
  return { x: xs[slot] ?? 5, y: 0 };
}

/** The foes of the fight at this step. */
export function fightFoes(step: number, path: number): readonly number[] {
  return MAP[step]?.options[path]?.foes ?? [];
}

/** Toward the other side: +1 for a hero, -1 for a foe (x side-on, y from above). */
export function lunge(from: { x: number; y: number }, toward: number, by = 0.7): { x: number; y: number } {
  return TOP ? { x: from.x, y: from.y + toward * by } : { x: from.x + toward * by, y: from.y };
}

/**
 * The camera for a surface (this device's own: it reads no rule): 14 units
 * tall where it can (224 px of 16 px art, a pixel-art look's whole grid),
 * taller on a portrait phone so every unit fits across. Side-on the ground
 * stands a little under the middle, over the hand's bar; from above the
 * heroes' row stands over the hand and the foes' across the felt. `fit`
 * may raise the height (to a pixel grid that fills the surface).
 */
export function cameraFor(vw: number, vh: number, party: number, fit: (height: number) => number = (h) => h): { x: number; y: number; height: number } {
  const portrait = vw < vh;
  if (TOP) {
    const height = fit(vw > 0 ? Math.max(14, (12.5 * vh) / vw) : 14);
    return { x: 0, y: -3.3 + (portrait ? 0.1 : 0.12) * height, height };
  }
  const left = party > 1 ? -8.55 : -6.8;
  const right = 9.7;
  const height = fit(vw > 0 ? Math.max(14, ((right - left) * vh) / vw) : 14);
  return { x: (left + right) / 2, y: (portrait ? 0.05 : 0.025) * height, height };
}

/**
 * Under a pixel look (Stylise.pixelHeight, 224 rows for 14 units: the art's
 * 16 px a unit), the grid for a surface: the rows that fill its height at a
 * whole scale, as many units tall as `height` needs at least, so the art
 * keeps its 16 px a unit and no border rings the frame (a 224-row grid
 * leaves 24 px above and below 720, and on a portrait phone would crush the
 * art to a third). Null without a pixel look.
 */
export function pixelGrid(lookRows: number, outHeight: number, height: number): { rows: number; height: number } | null {
  if (lookRows <= 0 || outHeight <= 0) return null;
  const ppu = lookRows / 14;
  const k = Math.max(1, Math.floor(outHeight / Math.ceil(height * ppu)));
  const rows = Math.floor(outHeight / k);
  return { rows, height: rows / ppu };
}

/** A world point on the screen in CSS px, under the camera as it is this tick. */
export function toScreen(world: World, x: number, y: number): { x: number; y: number } {
  const cam = world.resource(og.Camera);
  if (!cam.ok) return { x: 0, y: 0 };
  return cam.value.worldToScreen({ x, y, z: 0 }) ?? { x: 0, y: 0 };
}

let sizedW = 0;
let sizedH = 0;
let sizedParty = 0;
/** The look's own Stylise.pixelHeight, read before the Stage first writes it (0: no pixel look). */
let lookRows = -1;

/** Dresses the units for the view, the backdrop and the camera for the surface. */
export const Stage = defineSystem(
  "Stage",
  { reads: [Game, Hero, Foe], writes: [og.Camera, og.Stylise, og.Sprite, og.Flipbook, og.Transform], after: ["Run"] },
  (_q, time, world) => {
    const game = world.resource(Game);
    if (!game.ok) return;
    const g = game.value;
    const vp = og.ui.frameViewport();
    if (vp.width > 0 && (vp.width !== sizedW || vp.height !== sizedH || g.party !== sizedParty)) {
      sizedW = vp.width;
      sizedH = vp.height;
      sizedParty = g.party;
      const cam = world.resource(og.Camera);
      const look = world.resource(og.Stylise);
      if (lookRows < 0) lookRows = look.ok ? look.value.pixelHeight : 0;
      if (cam.ok) {
        let rows = 0;
        const c = cameraFor(vp.width, vp.height, g.party, (h) => {
          const grid = pixelGrid(lookRows, Math.round(vp.height * vp.dpr), h);
          rows = grid?.rows ?? 0;
          return grid?.height ?? h;
        });
        if (look.ok && rows > 0) look.value.pixelHeight = rows;
        cam.value.height = c.height;
        cam.value.position = { x: c.x, y: c.y, z: 50 };
        // The backdrop covers the frame at its own pixel scale (40 x 44 units,
        // its ground row 24 down on y = 0); only a surface past that, wider
        // than about 2.3:1 or taller than a phone, scales it up.
        const w = (c.height * vp.width) / vp.height;
        const s = Math.max(1, (Math.abs(c.x) + w / 2 + 0.5) / 20, (c.height / 2 - c.y + 0.5) / 20, (c.y + c.height / 2 + 0.5) / 24);
        const back = world.find("backdrop");
        if (back !== null) {
          must(world.set(back, og.Sprite, { width: 40 * s, height: 44 * s }));
          must(world.set(back, og.Transform, { position: { x: 0, y: 2 * s, z: -1 } }));
        }
      }
    }
    const inFight = g.phase === Phase.Battle || g.phase === Phase.Reward;
    for (let seat = 0; seat < HEROES.length; seat++) {
      const e = world.find(seat === 0 ? "hero" : `hero-${seat}`);
      if (e === null) continue;
      const h = world.get(e, Hero);
      if (!h.ok) continue;
      const shown = h.value.active || (g.phase === Phase.Title && seat === 0);
      const down = h.value.active && h.value.hp <= 0 && g.phase !== Phase.Title;
      dress(world, e, shown, down ? 0.35 : 1);
    }
    for (let slot = 0; slot < FOE_SLOTS; slot++) {
      const e = world.find(`foe-${slot}`);
      if (e === null) continue;
      const f = world.get(e, Foe);
      if (!f.ok) continue;
      const v = f.value;
      const shown = inFight && v.active && v.dying < 40;
      const want = clipOf(TOP ? `${FOES[v.kind]!.id}-token` : FOES[v.kind]!.id);
      const fb = world.get(e, og.Flipbook);
      if (fb.ok && fb.value.clip !== want) must(world.set(e, og.Flipbook, { clip: want }));
      const size = unitSize(v.kind);
      const sp = world.get(e, og.Sprite);
      if (sp.ok && sp.value.width !== size) must(world.set(e, og.Sprite, { width: size, height: size }));
      dress(world, e, shown, v.dying > 0 ? Math.max(0, 1 - v.dying / 40) : 1);
    }
    void time;
  },
);

function dress(world: World, e: Entity, visible: boolean, opacity: number): void {
  const sp = world.get(e, og.Sprite);
  if (!sp.ok) return;
  if (sp.value.visible !== visible || Math.abs(sp.value.opacity - opacity) > 0.005) must(world.set(e, og.Sprite, { visible, opacity }));
}

function must(r: { ok: true } | { ok: false; reason: string }): void {
  if (!r.ok) throw new Error(r.reason);
}
