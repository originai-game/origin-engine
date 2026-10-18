// Tile Swap, the match3 archetype: swap two neighbouring tiles (tap one, then
// the other, or drag one onto the other) to make lines of three or more of a
// colour; lines clear, the tiles above fall, new tiles drop in and any line
// they make clears again as a cascade worth more. Reach the target score
// before the moves run out.
//
// scenes/main.ogt places the board (with its Tuning); `start` below sets up
// the orthographic camera and spawns the backdrop, the checker tilemap, the
// 49 gems and the selection frame, every one an og.Sprite (the gems play
// og.Flipbook clips). The board logic is src/board.ts, the systems
// src/systems.ts, the components src/components.ts, the art src/art.ts.

import { og } from "@origingame/engine";
import type { Entity, Texture, World } from "@origingame/engine";
import { PANEL_INSET, idleClip, loadArt } from "./art.ts";
import { adjacent, cellOf, parseBoard } from "./board.ts";
import { Hud, Kind, Match, PHASE_NAMES, Phase, Piece, Sfx, Sound, Sounds, Tuning } from "./components.ts";
import { BOARD_TOP, BOARD_Y, GEM_SIZE, PITCH, SIZE, cellCentre } from "./layout.ts";
import { Animate, Flow, HudText, Pick, Resolve, Voices } from "./systems.ts";

/** The view keeps at least this much of the world visible. */
const MIN_VIEW = { width: 8.6, height: 12 } as const;

/** The brand ink #14161b, linear. */
const INK = { r: 0.007, g: 0.008, b: 0.0103 } as const;

/** Sprite layers: higher draws on top. The checker tilemap sits at -10, floating scores at 100. */
const LAYER = { backdrop: -30, panel: -20, cells: -10, gem: 0, cursor: 1 } as const;

function must<T>(r: { ok: true; value?: T } | { ok: false; reason: string }): void {
  if (!r.ok) throw new Error(r.reason);
}

function start(world: World): void {
  const art = loadArt(world);
  const font = og.load.font(world, "fonts/ui.ttf");
  if (!font.ok) throw new Error(font.reason);

  const caps = og.render.capabilities(world);
  const aspect = caps.width > 0 && caps.height > 0 ? caps.width / caps.height : 16 / 9;
  const height = Math.max(MIN_VIEW.height, MIN_VIEW.width / aspect);
  const halfWidth = (height * aspect) / 2;
  // The board's top sits under the HUD; the view centre is above the board's.
  const centreY = BOARD_Y + 0.9;
  const camera = world.resource(og.Camera);
  if (!camera.ok) throw new Error(camera.reason);
  camera.value.orthographic = true;
  camera.value.height = height;
  camera.value.position = { x: 0, y: centreY, z: 50 };
  camera.value.background = INK;
  const settings = world.resource(og.RenderSettings);
  if (settings.ok) {
    settings.value.environmentIntensity = 0;
    settings.value.toneMapped = false;
  }

  // Every row the systems see carries the same six components (src/components.ts says why).
  const row = (d: { name?: string; kind: number; x: number; y: number; z: number; texture: Texture; w: number; h: number; layer: number; visible?: boolean }): Entity => {
    const s = world.spawn({ name: d.name ?? "", position: { x: d.x, y: d.y, z: d.z } });
    if (!s.ok) throw new Error(s.reason);
    must(world.insert(s.value, og.Sprite, { texture: d.texture, width: d.w, height: d.h, layer: d.layer, visible: d.visible ?? true }));
    must(world.insert(s.value, og.Flipbook));
    must(world.insert(s.value, Piece, { kind: d.kind }));
    must(world.insert(s.value, Match));
    must(world.insert(s.value, Tuning));
    return s.value;
  };

  // A soft lift of light behind the board; around it the camera's background is the ink itself.
  row({ name: "backdrop", kind: Kind.Backdrop, texture: art.backdrop, x: 0, y: BOARD_Y, z: -3, w: 16, h: 13, layer: LAYER.backdrop });

  // The checker under the gems: a hand-written tilemap (sprites/board.ogt), placed by its top-left corner.
  const cells = world.spawn({ name: "cells", position: { x: -(SIZE / 2) * PITCH, y: BOARD_TOP, z: -1 } });
  if (!cells.ok) throw new Error(cells.reason);
  must(world.insert(cells.value, og.Tilemap, { map: art.board, layer: LAYER.cells, collision: false }));

  // The gems: dealt on the first press (Flow); until then hidden.
  for (let k = 0; k < SIZE * SIZE; k++) {
    const p = cellCentre(k % SIZE, Math.floor(k / SIZE));
    const t = row({ name: `tile-${k}`, kind: Kind.Tile, texture: art.gemTexture, x: p.x, y: p.y, z: 0, w: GEM_SIZE * PITCH, h: GEM_SIZE * PITCH, layer: LAYER.gem, visible: false });
    must(world.set(t, Piece, { col: k % SIZE, row: Math.floor(k / SIZE) }));
    must(world.set(t, og.Flipbook, { sheet: art.gems, clip: idleClip(0) }));
  }
  const cursor = row({ name: "cursor", kind: Kind.Cursor, texture: art.ring, x: 0, y: 0, z: 1, w: PITCH * 1.06, h: PITCH * 1.06, layer: LAYER.cursor, visible: false });
  // The selection frame breathes forever: one looping tween, no per-tick code.
  must(og.tween.to(world, { entity: cursor, field: "scale", to: [1.07, 1.07, 1], duration: 0.45, ease: "quadInOut", loops: -1, yoyo: true }));

  // The board comes from the scene with its Tuning; give it the panel (a nine-slice), the round and the HUD.
  const board = world.find("board");
  if (board === null) throw new Error("scenes/main.ogt has no entity named board");
  const inset = { x: PANEL_INSET, y: PANEL_INSET };
  must(world.insert(board, og.Sprite, { texture: art.panel, width: SIZE * PITCH + 0.7, height: SIZE * PITCH + 0.7, layer: LAYER.panel, sliceX: inset, sliceY: inset, sliceScale: 0.45 / PANEL_INSET }));
  must(world.insert(board, og.Flipbook));
  must(world.insert(board, Piece, { kind: Kind.Board }));
  must(world.insert(board, Match, { viewHeight: height, viewCentreY: centreY, font: font.value as unknown as number }));
  must(world.insert(board, Hud, { font: font.value as unknown as number, top: centreY + height / 2, halfWidth }));

  // The voices: one per sound, played by the Voices system.
  const voices: [string, number, number][] = [
    ["audio/select.wav", Sound.Select, 0.4],
    ["audio/swap.wav", Sound.Swap, 0.45],
    ["audio/refused.wav", Sound.Refused, 0.5],
    ["audio/clear.wav", Sound.Clear, 0.55],
    ["audio/cascade.wav", Sound.Cascade, 0.6],
    ["audio/won.wav", Sound.Won, 0.6],
    ["audio/lost.wav", Sound.Lost, 0.6],
  ];
  for (const [path, bit, volume] of voices) {
    const sound = og.load.sound(world, path);
    if (!sound.ok) throw new Error(sound.reason);
    const v = world.spawn({ name: `sfx-${bit}` });
    if (!v.ok) throw new Error(v.reason);
    must(world.insert(v.value, og.AudioSource, { sound: sound.value, playing: false, volume }));
    must(world.insert(v.value, Sfx, { bit }));
  }
}

/** The board's Match, outside systems. */
function matchOf(world: World): { board: Entity; match: Record<string, number> } {
  const board = world.find("board");
  if (board === null) throw new Error("no board");
  const m = world.get(board, Match);
  if (!m.ok) throw new Error(m.reason);
  return { board, match: m.value as unknown as Record<string, number> };
}

/** Every tile entity with its Piece, outside systems. */
function tiles(world: World): Array<{ e: Entity; col: number; row: number }> {
  const out: Array<{ e: Entity; col: number; row: number }> = [];
  for (let k = 0; k < SIZE * SIZE; k++) {
    const e = world.find(`tile-${k}`);
    if (e === null) continue;
    const p = world.get(e, Piece);
    if (p.ok) out.push({ e, col: p.value.col, row: p.value.row });
  }
  return out;
}

/** `deal:<rows>`: lay out the board, SIZE lines of SIZE colour digits joined by '/', the top row first. Waits for an idle board. */
function dealCommand(world: World, args: readonly string[]): void {
  const { board, match } = matchOf(world);
  if (match["phase"] !== Phase.Idle) throw new Error(`deal needs an idle board; it is ${PHASE_NAMES[match["phase"]!]} (tap to start a round first)`);
  const tuning = world.get(board, Tuning);
  if (!tuning.ok) throw new Error(tuning.reason);
  const grid = parseBoard(args.join(":"), tuning.value.colours);
  if (typeof grid === "string") throw new Error(grid);
  for (const t of tiles(world)) must(world.set(t.e, Piece, { colour: grid[cellOf(t.col, t.row)]!, marked: false }));
  must(world.set(board, Match, { selected: -1, dragFrom: -1 }));
}

/** `swap:<col>,<row>:<col>,<row>`: swap two neighbouring cells on the next tick, as a tap on each would. */
function swapCommand(world: World, args: readonly string[]): void {
  const cell = (s: string | undefined) => {
    const [c, r] = (s ?? "").split(",").map(Number);
    return Number.isInteger(c) && Number.isInteger(r) && c! >= 0 && c! < SIZE && r! >= 0 && r! < SIZE ? cellOf(c!, r!) : -1;
  };
  const a = cell(args[0]);
  const b = cell(args[1]);
  if (!adjacent(a, b)) throw new Error("write swap:<col>,<row>:<col>,<row> with two neighbouring cells (0,0 is the bottom left)");
  const { board, match } = matchOf(world);
  if (match["phase"] !== Phase.Idle) throw new Error(`swap needs an idle board; it is ${PHASE_NAMES[match["phase"]!]}`);
  // Pick starts it on the next tick, exactly as two taps would.
  must(world.set(board, Match, { swapA: a, swapB: b, selected: -1, dragFrom: -1 }));
}

export default og.game({
  tick: 60,
  seed: 1,
  scenes: { main: "scenes/main.ogt" },
  resources: [Sounds],
  systems: [Flow, Pick, Resolve, Voices, Animate, HudText],
  render: { antialias: "msaa4" },
  start,
  commands: { deal: dealCommand, swap: swapCommand },
  commandHelp: {
    deal: { usage: "<rows>", doc: "Lay out the board: 7 lines of 7 colour digits joined by /, the top row first; needs an idle board." },
    swap: { usage: "<col>,<row>:<col>,<row>", doc: "Swap two neighbouring cells on the next tick, as two taps would." },
  },
  facts: (world) => {
    const { match: m } = matchOf(world);
    return {
      phase: PHASE_NAMES[m["phase"]!] ?? "unknown",
      score: m["score"],
      movesLeft: m["movesLeft"],
      cleared: m["cleared"],
      maxChain: m["maxChain"],
      best: m["best"],
      rounds: m["rounds"],
      pointer: { pxPerM: m["pxPerM"], x0: m["screenX0"], y0: m["screenY0"] },
    };
  },
});
