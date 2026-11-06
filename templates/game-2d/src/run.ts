// What both views of the 2D starter share (src/main.ts plays the rooftops
// side-on, src/courtyard.ts the courtyard from above; src/creation.ts's
// view picks one): the run's state, the coins, the voices, the HUD with its
// pause screen, and Frame, which keeps two couriers in view.

import { og, defineComponent, defineResource, defineSystem, bool, f32, u32 } from "@origingame/engine";
import type { Result, World } from "@origingame/engine";
import { creation } from "./creation.ts";
import { press, SEATS } from "./seats.ts";

const view: string = creation.view;
/** The courtyard seen from above (top-down-2d), not the rooftops side-on (side-2d). */
export const TOP_DOWN = view === "top-down-2d";

export function must<T>(r: Result<T>): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** The camera's height in world units: 14 cells of 16 px, the 16bit-pixel look's 224-pixel frame. */
export const VIEW_H = 14;
/** World units kept clear above and below the two couriers when the camera frames both. */
const MARGIN = 2;
/** And beside them: a sprite's half width and a little, so a portrait screen spends its narrow width on the couriers. */
const MARGIN_X = 1.4;
export const LIVES = 3;

/** A voice: the sound bit (Sound) it plays when Game.sfx asks for it. */
export const Voice = defineComponent("Voice", { bit: u32() });

export const Sound = { Jump: 1, Coin: 2, Dash: 4, Splash: 8 } as const;

/** A coin of the map's `coin` objects; taken once. */
export const Coin = defineComponent("Coin", { taken: bool() });

/** The run: what the HUD shows and the systems share. */
export const Game = defineResource("Game", {
  coins: u32(),
  total: u32(),
  lives: u32(LIVES),
  won: bool(),
  /** The last life was lost: Pickups starts the level again. */
  restart: bool(),
  goalX: f32(0, { unit: "m" }),
  /** Seat 0's courier, and seat 1's when it plays. */
  heroX: f32(0, { unit: "m" }),
  heroY: f32(0, { unit: "m" }),
  hero2X: f32(0, { unit: "m" }),
  hero2Y: f32(0, { unit: "m" }),
  /** Sounds asked for this tick (Sound bits), played by Voices. */
  sfx: u32(),
  /** The pause menu is open (a seat's PAUSE toggles it): the couriers and time hold. */
  paused: bool(),
  /** The keeper's breaths: og.Skeleton2DEvent "inhale" (the idle clip's event at 0). */
  inhales: u32(),
  /** The keeper's footfalls: og.Skeleton2DEvent "step" (the walk clip's, one per foot). */
  steps: u32(),
});

/**
 * Two couriers: Frame drives og.Camera (presentation only, never the
 * world) so both stay in view with a margin, framed into the play rect:
 * the part of the screen between the HUD's top corners and the touch
 * controls (last frame's layout), so on a portrait phone the level fills
 * the room between the two rather than a strip across the middle. It zooms
 * no closer than one courier's view and eases as they part and close up.
 */
export const Frame = defineSystem("Frame", { reads: [Game], writes: [og.Camera], after: ["Control"] }, (_q, time, world) => {
  if (SEATS < 2) return;
  const g = must(world.resource(Game));
  const camera = must(world.resource(og.Camera));
  const vp = og.ui.viewport(world);
  const w = vp.width > 0 ? vp.width : 16;
  const h = vp.height > 0 ? vp.height : 9;
  const rect = playRect(w, h);
  // World units per UI unit: the couriers' bounds fit in the rect, never closer than VIEW_H across the screen.
  const x0 = Math.min(g.heroX, g.hero2X) - MARGIN_X, x1 = Math.max(g.heroX, g.hero2X) + MARGIN_X;
  const y0 = Math.min(g.heroY, g.hero2Y) - MARGIN, y1 = Math.max(g.heroY, g.hero2Y) + LEVEL.figure + MARGIN;
  const s = Math.max(VIEW_H / h, (x1 - x0) / rect.width, (y1 - y0) / rect.height);
  // The bounds' middle sits at the rect's middle (screen y runs down, world y up), kept so the rect shows the level, not past its edges.
  const inside = (c: number, half: number, size: number) => (2 * half >= size ? size / 2 : Math.min(Math.max(c, half), size - half));
  const mx = inside((x0 + x1) / 2, (rect.width * s) / 2, LEVEL.width);
  const my = inside((y0 + y1) / 2, (rect.height * s) / 2, LEVEL.height);
  const cx = mx - (rect.x + rect.width / 2 - w / 2) * s;
  const cy = my + (rect.y + rect.height / 2 - h / 2) * s;
  const k = Math.min(1, time.dt * 6);
  camera.height += (s * h - camera.height) * k;
  camera.position = { x: camera.position.x + (cx - camera.position.x) * k, y: camera.position.y + (cy - camera.position.y) * k, z: camera.position.z };
});

/** The level's size in world units (its bottom-left corner at the origin), set by start, and how tall a courier stands over its feet. */
export const LEVEL = { width: 0, height: 0, figure: 1.9 };

/** The screen's play rect, in og-ui units: below the HUD's top corners and above the touch controls, as last laid out. */
function playRect(w: number, h: number): { x: number; y: number; width: number; height: number } {
  let top = 0, bottom = h;
  for (const id of ["Status", "Purse"]) {
    const b = og.ui.bounds(id);
    if (b !== null && b.y < h / 2) top = Math.max(top, b.y + b.height);
  }
  for (const id of ["TouchMove0", "TouchMove1", "TouchActions0"]) {
    const b = og.ui.bounds(id);
    if (b !== null && b.y > h / 2) bottom = Math.min(bottom, b.y);
  }
  // A sliver (a tiny window, a layout not run yet) frames the whole screen.
  return bottom - top < h / 3 ? { x: 0, y: 0, width: w, height: h } : { x: 0, y: top, width: w, height: bottom - top };
}

/** Plays each sound asked for once: a voice still playing is stopped this tick and played again the next. */
export const Voices = defineSystem("Voices", { reads: [Voice], writes: [Game, og.AudioSource], after: ["Pickups"] }, (q, _time, world) => {
  const g = must(world.resource(Game));
  if (g.sfx === 0) return;
  let left = g.sfx;
  for (const a of q.archetypes()) {
    const voice = a.column(Voice);
    const source = a.column(og.AudioSource);
    for (let i = 0; i < a.count; i++) {
      const bit = voice.bit[i]!;
      if ((g.sfx & bit) === 0) continue;
      if (source.playing[i]) source.playing[i] = 0;
      else {
        source.playing[i] = 1;
        left &= ~bit;
      }
    }
  }
  g.sfx = left;
});

/** A group of the HUD on the style's HUD chip (the city behind can be any colour). */
function chip(id: string, build: () => void): void {
  og.ui.row(id, { gap: 6, cross: "center", padding: { left: 12, top: 4, right: 12, bottom: 4 }, frame: "hud", background: { token: "hud" } }, build);
}

/**
 * The HUD (og.ui's HUD components, docs/ui.md §11): the lives as pips and
 * the level's name top left, the coins, the score and Pause top right (the
 * touch controls are the seats', src/seats.ts). Pause (the button, Escape
 * or a pad's Start) shows the game's pause screen (assets/ui/pause.ogt)
 * while Game.paused holds time, until Resume or Back.
 */
export const Hud = defineSystem("Hud", { reads: [Game] }, (_q, _time, world) => {
  const g = must(world.resource(Game));
  const t = (key: string) => og.i18n.t(world, key);
  // The keys move the courier, not the focus.
  og.ui.setNavigation(false);
  og.ui.anchor("Status", { anchor: "topLeft", direction: "column", gap: 8 }, () => {
    og.ui.statBar("Lives", { value: g.lives, max: LIVES, kind: "health", style: "pips", width: { kind: "fixed", px: 96 }, name: t("hud.lives") });
    chip("LevelBox", () => og.ui.label("Level", t(TOP_DOWN ? (g.won ? "hud.yardClear" : "hud.yard") : g.won ? "hud.clear" : "hud.level"), { step: "meta", color: { token: "inkMuted" }, nowrap: true }));
  });
  og.ui.anchor("Purse", { anchor: "topRight", gap: 8, cross: "center" }, () => {
    chip("Coins", () => {
      if (og.ui.hasIcon("coin")) og.ui.icon("Coin", "coin", { size: 20, tint: { token: "gold" } });
      og.ui.counter("Count", g.coins, { font: "mono" });
    });
    chip("Score", () => og.ui.counter("Points", g.coins * 100, { font: "mono" }));
    // Pause acts through the seat's input (src/seats.ts), so a room pauses on one tick.
    if (og.ui.actionButton("Pause", t("hud.pause"), { icon: "pause", size: 44 }).clicked) press("pause");
  });
  // The pause screen follows Game.paused; its Resume and Back press pause again.
  if (g.paused && !pauseShown) {
    must(og.ui.push(world, "pause"));
    pauseShown = true;
  } else if (!g.paused && pauseShown) {
    og.ui.pop(world);
    pauseShown = false;
  }
  if (pauseShown) {
    for (const e of og.ui.screen("pause").events) {
      if ((e.kind === "emit" || e.kind === "back") && e.name === "resume") press("pause");
    }
  }
});

/** Whether this device shows the pause screen (UI state, not the world's). */
let pauseShown = false;

/** `viewport:<desktop|phone>`: lay the UI out for a phone in portrait, its touch controls showing (playtests). */
export function viewport(world: World, args: readonly string[]): void {
  const phone = { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" } as const;
  og.ui.previewViewport(world, args[0] === "phone" ? phone : null);
}

/** The commands' help both views share. */
export const VIEWPORT_HELP = { usage: "<desktop|phone>", doc: "Lay the UI out as on a portrait phone, with its touch controls (desktop: back)." } as const;
