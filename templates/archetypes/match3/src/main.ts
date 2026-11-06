// Tile Swap, the match3 archetype: swap two neighbouring tiles (tap one, then
// the other, or drag one onto the other) to make lines of three or more of a
// colour; lines clear, the tiles above fall, new tiles drop in and any line
// they make clears again as a cascade worth more. Reach the target score
// before the moves run out.
//
// Players (src/seats.ts, src/creation.ts): alone it is one board; with two
// seats (on one device, or over a room) it is versus, a board per seat side
// by side, both dealt the same grid, each swapped by its own seat's cursor:
// the first to reach the target wins, else the higher score once both are
// out of moves. Every swap is a function of og.PlayerInput (the cursor in
// aimX, aimY, the pick bit, the move axis for keys and pads), so a replay and
// a room play it the same.
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
import { BOARDS, BOARD_GAP, BOARD_TOP, BOARD_Y, GEM_SIZE, PITCH, SIZE, boardX, cellCentre } from "./layout.ts";
import { LOCAL, Seats, bit, setBot, setupSeats, useTouchControls } from "./seats.ts";
import { Animate, Flow, Frame, HudText, Pick, Resolve, Voices, frameView } from "./systems.ts";

/** The brand ink #14161b, linear. */
const INK = { r: 0.007, g: 0.008, b: 0.0103 } as const;

/** Sprite layers: higher draws on top. The checker tilemap sits at -10, floating scores at 100. */
const LAYER = { backdrop: -30, panel: -20, cells: -10, gem: 0, cursor: 1 } as const;

function must<T>(r: { ok: true; value?: T } | { ok: false; reason: string }): void {
  if (!r.ok) throw new Error(r.reason);
}

function start(world: World): void {
  setupSeats(world);
  const art = loadArt(world);
  const font = og.load.font(world, "fonts/ui.ttf");
  if (!font.ok) throw new Error(font.reason);

  const camera = world.resource(og.Camera);
  if (!camera.ok) throw new Error(camera.reason);
  camera.value.orthographic = true;
  camera.value.background = INK;
  frameView(world);
  const settings = world.resource(og.RenderSettings);
  if (settings.ok) {
    settings.value.environmentIntensity = 0;
    settings.value.toneMapped = false;
  }

  // Every row the systems see carries the same seven components (src/components.ts says why).
  const row = (d: { name?: string; kind: number; seat?: number; x: number; y: number; z: number; texture: Texture; w: number; h: number; layer: number; visible?: boolean }): Entity => {
    const s = world.spawn({ name: d.name ?? "", position: { x: d.x, y: d.y, z: d.z } });
    if (!s.ok) throw new Error(s.reason);
    must(world.insert(s.value, og.Sprite, { texture: d.texture, width: d.w, height: d.h, layer: d.layer, visible: d.visible ?? true }));
    must(world.insert(s.value, og.Flipbook));
    must(world.insert(s.value, Piece, { kind: d.kind, seat: d.seat ?? 0 }));
    must(world.insert(s.value, Match));
    must(world.insert(s.value, Tuning));
    must(world.insert(s.value, og.PlayerInput, { player: d.seat ?? 0 }));
    return s.value;
  };

  // A soft lift of light behind the boards; around it the camera's background is the ink itself.
  row({ name: "backdrop", kind: Kind.Backdrop, texture: art.backdrop, x: 0, y: BOARD_Y, z: -3, w: 16 + (BOARDS > 1 ? 2 * BOARD_GAP : 0), h: 13, layer: LAYER.backdrop });

  // The board comes from the scene with its Tuning, which every board plays to; versus adds seat 1's.
  const first = world.find("board");
  if (first === null) throw new Error("scenes/main.ogt has no entity named board");
  for (let seat = 0; seat < BOARDS; seat++) {
    const suffix = seat === 0 ? "" : `-${seat}`;
    // The checker under the gems: a hand-written tilemap (sprites/board.ogt), placed by its top-left corner.
    const cells = world.spawn({ name: `cells${suffix}`, position: { x: boardX(seat) - (SIZE / 2) * PITCH, y: BOARD_TOP, z: -1 } });
    if (!cells.ok) throw new Error(cells.reason);
    must(world.insert(cells.value, og.Tilemap, { map: art.board, layer: LAYER.cells, collision: false }));

    // The gems: dealt on the first press (Flow); until then hidden.
    for (let k = 0; k < SIZE * SIZE; k++) {
      const p = cellCentre(k % SIZE, Math.floor(k / SIZE), seat);
      const name = seat === 0 ? `tile-${k}` : `tile${suffix}-${k}`;
      const t = row({ name, kind: Kind.Tile, seat, texture: art.gemTexture, x: p.x, y: p.y, z: 0, w: GEM_SIZE * PITCH, h: GEM_SIZE * PITCH, layer: LAYER.gem, visible: false });
      must(world.set(t, Piece, { col: k % SIZE, row: Math.floor(k / SIZE) }));
      must(world.set(t, og.Flipbook, { sheet: art.gems, clip: idleClip(0) }));
    }
    const cursor = row({ name: `cursor${suffix}`, kind: Kind.Cursor, seat, texture: art.ring, x: boardX(seat), y: 0, z: 1, w: PITCH * 1.06, h: PITCH * 1.06, layer: LAYER.cursor, visible: false });
    // The selection frame breathes forever: one looping tween, no per-tick code.
    must(og.tween.to(world, { entity: cursor, field: "scale", to: [1.07, 1.07, 1], duration: 0.45, ease: "quadInOut", loops: -1, yoyo: true }));

    // The board: the panel (a nine-slice), the round and the HUD.
    let board = first;
    if (seat > 0) {
      const s = world.spawn({ name: `board${suffix}`, position: { x: boardX(seat), y: BOARD_Y, z: -2 } });
      if (!s.ok) throw new Error(s.reason);
      board = s.value;
      must(world.insert(board, Tuning));
    } else if (BOARDS > 1) {
      must(world.set(board, og.Transform, { position: { x: boardX(0), y: BOARD_Y, z: -2 } }));
    }
    const inset = { x: PANEL_INSET, y: PANEL_INSET };
    must(world.insert(board, og.Sprite, { texture: art.panel, width: SIZE * PITCH + 0.7, height: SIZE * PITCH + 0.7, layer: LAYER.panel, sliceX: inset, sliceY: inset, sliceScale: 0.45 / PANEL_INSET }));
    must(world.insert(board, og.Flipbook));
    must(world.insert(board, Piece, { kind: Kind.Board, seat }));
    must(world.insert(board, Match, { font: font.value as unknown as number }));
    must(world.insert(board, Hud, { font: font.value as unknown as number }));
    must(world.insert(board, og.PlayerInput, { player: seat }));
  }

  // Seat 1 on a shared touch screen: a d-pad moves its hover and a Pick button picks (seat 0 taps its board).
  useTouchControls((seat) => {
    const out = { x: 0, y: 0, lx: 0, ly: 0, buttons: 0 };
    if (!LOCAL || seat !== 1) return out;
    og.ui.anchor("TouchMove1", { anchor: "bottomRight", direction: "column", gap: 12, cross: "center" }, () => {
      if (og.ui.actionButton("pick", "Pick", { size: 64, primary: true }).pressed) out.buttons |= bit("pick");
      const d = og.ui.dpad("Move", { size: 112 });
      out.x = Number(d.right) - Number(d.left);
      out.y = Number(d.up) - Number(d.down);
    });
    return out;
  });

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

/** A seat's board and its Match, outside systems. */
function matchOf(world: World, seat = 0): { board: Entity; match: Record<string, number> } {
  const board = world.find(seat === 0 ? "board" : `board-${seat}`);
  if (board === null) throw new Error(`no board for seat ${seat}`);
  const m = world.get(board, Match);
  if (!m.ok) throw new Error(m.reason);
  return { board, match: m.value as unknown as Record<string, number> };
}

/** Every tile entity of a seat's board with its Piece, outside systems. */
function tiles(world: World, seat: number): Array<{ e: Entity; col: number; row: number }> {
  const out: Array<{ e: Entity; col: number; row: number }> = [];
  for (let k = 0; k < SIZE * SIZE; k++) {
    const e = world.find(seat === 0 ? `tile-${k}` : `tile-${seat}-${k}`);
    if (e === null) continue;
    const p = world.get(e, Piece);
    if (p.ok) out.push({ e, col: p.value.col, row: p.value.row });
  }
  return out;
}

/** `deal:<rows>`: lay out every board, SIZE lines of SIZE colour digits joined by '/', the top row first. Waits for idle boards. */
function dealCommand(world: World, args: readonly string[]): void {
  const tuning = world.get(matchOf(world).board, Tuning);
  if (!tuning.ok) throw new Error(tuning.reason);
  const grid = parseBoard(args.join(":"), tuning.value.colours);
  if (typeof grid === "string") throw new Error(grid);
  for (let seat = 0; seat < BOARDS; seat++) {
    const { board, match } = matchOf(world, seat);
    if (match["phase"] !== Phase.Idle) throw new Error(`deal needs an idle board; it is ${PHASE_NAMES[match["phase"]!]} (tap to start a round first)`);
    for (const t of tiles(world, seat)) must(world.set(t.e, Piece, { colour: grid[cellOf(t.col, t.row)]!, marked: false }));
    must(world.set(board, Match, { selected: -1, dragFrom: -1 }));
  }
}

/** `swap:<col>,<row>:<col>,<row>[:<seat>]`: swap two neighbouring cells of a seat's board (seat 0's by default) on the next tick, as a tap on each would. */
function swapCommand(world: World, args: readonly string[]): void {
  const cell = (s: string | undefined) => {
    const [c, r] = (s ?? "").split(",").map(Number);
    return Number.isInteger(c) && Number.isInteger(r) && c! >= 0 && c! < SIZE && r! >= 0 && r! < SIZE ? cellOf(c!, r!) : -1;
  };
  const a = cell(args[0]);
  const b = cell(args[1]);
  if (!adjacent(a, b)) throw new Error("write swap:<col>,<row>:<col>,<row> with two neighbouring cells (0,0 is the bottom left)");
  const seat = Number(args[2] ?? 0);
  if (!Number.isInteger(seat) || seat < 0 || seat >= BOARDS) throw new Error(`the seat is 0${BOARDS > 1 ? " or 1" : " (one board alone)"}`);
  const { board, match } = matchOf(world, seat);
  if (match["phase"] !== Phase.Idle) throw new Error(`swap needs an idle board; it is ${PHASE_NAMES[match["phase"]!]}`);
  // Pick starts it on the next tick, exactly as two taps would.
  must(world.set(board, Match, { swapA: a, swapB: b, selected: -1, dragFrom: -1 }));
}

/** `viewport:<desktop|2x>`: report the play surface as 640×360 CSS px at 2× (a high-DPR screen; the render target stays 1280×720), or the host's again. */
function viewportCommand(world: World, args: readonly string[]): void {
  const hiDpi = { width: 640, height: 360, dpr: 2, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" } as const;
  og.ui.previewViewport(world, args[0] === "2x" ? hiDpi : null);
}

export default og.game({
  tick: 60,
  seed: 1,
  scenes: { main: "scenes/main.ogt" },
  resources: [Sounds],
  // Seats first: it gives each seat's og.PlayerInput for the tick; Frame points a versus game's camera.
  systems: [Seats, Flow, Pick, Resolve, Voices, Animate, HudText, ...(BOARDS > 1 ? [Frame] : [])],
  render: { antialias: "msaa4" },
  start,
  commands: { deal: dealCommand, swap: swapCommand, viewport: viewportCommand, bot: (_world, args) => setBot(Number(args[0])) },
  commandHelp: {
    deal: { usage: "<rows>", doc: "Lay out every board: 7 lines of 7 colour digits joined by /, the top row first; needs idle boards." },
    swap: { usage: "<col>,<row>:<col>,<row>[:<seat>]", doc: "Swap two neighbouring cells of a seat's board (seat 0's by default) on the next tick, as two taps would." },
    bot: { usage: "<n>", doc: "This device's seats take scripted input by the tick (1, 2, ...); 0 gives them back." },
    viewport: { usage: "<desktop|2x>", doc: "Report the play surface as 640×360 CSS px at 2× (the render target stays 1280×720), or the host's again." },
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
      seats: BOARDS,
      ...(BOARDS > 1 ? { phase2: PHASE_NAMES[matchOf(world, 1).match["phase"]!] ?? "unknown", score2: matchOf(world, 1).match["score"], movesLeft2: matchOf(world, 1).match["movesLeft"] } : {}),
    };
  },
});
