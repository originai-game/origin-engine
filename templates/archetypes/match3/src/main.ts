// Tile Swap, the match3 archetype: swap two neighbouring tiles (tap one, then
// the other, or drag one onto the other) to make lines of three or more of a
// colour; lines clear, the tiles above fall, new tiles drop in and any line
// they make clears again as a cascade worth more. Reach the target score
// before the moves run out.
//
// scenes/main.ogt places the board (with its Tuning); `start` below sets up
// the orthographic camera and spawns the backdrop, the cells, the tiles and
// the selection frame. The board logic is src/board.ts, the systems
// src/systems.ts, the components src/components.ts.

import { og } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";
import { loadSheet, TILE_SPRITES } from "./art.ts";
import type { SpriteName } from "./art.ts";
import { adjacent, cellOf, parseBoard } from "./board.ts";
import { Hud, Kind, Match, PHASE_NAMES, Phase, Piece, Tuning } from "./components.ts";
import { BOARD_Y, PITCH, SIZE, cellCentre } from "./layout.ts";
import { Animate, Flow, HudText, Pick, Resolve } from "./systems.ts";

/** The view keeps at least this much of the world visible. */
const MIN_VIEW = { width: 8.6, height: 12 } as const;

function must(r: { ok: true } | { ok: false; reason: string }): void {
  if (!r.ok) throw new Error(r.reason);
}

function start(world: World): void {
  const sheet = loadSheet(world);
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
  camera.value.background = { r: 0.0118, g: 0.0137, b: 0.0273 };
  const settings = world.resource(og.RenderSettings);
  if (settings.ok) {
    settings.value.environmentIntensity = 0;
    settings.value.toneMapped = false;
  }
  const sun = world.spawn({ name: "sun" });
  if (sun.ok) must(world.insert(sun.value, og.DirectionalLight, { intensity: 0, castShadow: false }));

  const row = (d: { name?: string; kind: number; sprite: SpriteName; x: number; y: number; z: number; w: number; h: number; visible?: boolean; colour?: number }): Entity => {
    const s = world.spawn({ name: d.name ?? "", position: { x: d.x, y: d.y, z: d.z }, scale: { x: d.w, y: d.h, z: 1 } });
    if (!s.ok) throw new Error(s.reason);
    const sprite = sheet[d.sprite];
    must(world.insert(s.value, og.Mesh, { geometry: sprite.geometry, material: sprite.material, visible: d.visible ?? true }));
    must(
      world.insert(s.value, Piece, {
        kind: d.kind,
        colour: d.colour ?? -1,
        look0: sheet[TILE_SPRITES[0]].material,
        look1: sheet[TILE_SPRITES[1]].material,
        look2: sheet[TILE_SPRITES[2]].material,
        look3: sheet[TILE_SPRITES[3]].material,
        look4: sheet[TILE_SPRITES[4]].material,
        look5: sheet[TILE_SPRITES[5]].material,
      }),
    );
    must(world.insert(s.value, Match));
    must(world.insert(s.value, Tuning));
    return s.value;
  };

  row({ name: "backdrop", kind: Kind.Backdrop, sprite: "backdrop", x: 0, y: centreY, z: -20, w: Math.max(60, halfWidth * 2 + 1), h: Math.max(30, height + 1) });
  for (let r = 0; r < SIZE; r++) {
    for (let c = 0; c < SIZE; c++) {
      const p = cellCentre(c, r);
      row({ kind: Kind.Cell, sprite: "cell", x: p.x, y: p.y, z: -1, w: PITCH * 0.96, h: PITCH * 0.96 });
    }
  }
  // The tiles: dealt on the first press (Flow); until then hidden.
  for (let k = 0; k < SIZE * SIZE; k++) {
    const p = cellCentre(k % SIZE, Math.floor(k / SIZE));
    const t = row({ name: `tile-${k}`, kind: Kind.Tile, sprite: "tile_ruby", x: p.x, y: p.y, z: 0, w: PITCH * 0.92, h: PITCH * 0.92, visible: false });
    must(world.set(t, Piece, { col: k % SIZE, row: Math.floor(k / SIZE) }));
  }
  row({ name: "cursor", kind: Kind.Cursor, sprite: "ring", x: 0, y: 0, z: 1, w: PITCH * 1.04, h: PITCH * 1.04, visible: false });

  // The board comes from the scene with its Tuning; give it the sprite, the round (with the view Pick maps the pointer through) and the HUD.
  const board = world.find("board");
  if (board === null) throw new Error("scenes/main.ogt has no entity named board");
  must(world.insert(board, og.Mesh, { geometry: sheet.board.geometry, material: sheet.board.material }));
  must(world.insert(board, Piece, { kind: Kind.Board }));
  must(world.insert(board, Match, { viewHeight: height, viewCentreY: centreY }));
  must(world.insert(board, Hud, { font: font.value as unknown as number, top: centreY + height / 2, halfWidth }));
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
  systems: [Flow, Pick, Resolve, Animate, HudText],
  render: { antialias: "msaa4" },
  start,
  commands: { deal: dealCommand, swap: swapCommand },
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
