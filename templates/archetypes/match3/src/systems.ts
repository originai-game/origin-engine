// The match3 systems, in tick order: Flow (title, rounds, the end cards),
// Pick (taps and drags choose a swap), Resolve (swap, clear, fall, cascade,
// refill, the end of a round), Animate (what each row shows) and Hud.
//
// Each system sees the rows through its typed views. The board row
// (Piece.kind === Kind.Board) carries the round's state and tuning for every
// system (src/components.ts says why).

import { og, defineSystem } from "@origingame/engine";
import type { ArchetypeView, Query, Rng, View } from "@origingame/engine";
import { adjacent, cellOf, colOf, count, deal, findLines, hasMove, rowOf } from "./board.ts";
import type { Grid } from "./board.ts";
import { Hud, Kind, Match, Phase, Piece, Tuning } from "./components.ts";
import { BOARD_TOP, PITCH, SIZE, cellAt, cellCentre } from "./layout.ts";

/** Ticks an end card holds before a press starts the next round. */
const END_HOLD = 40;

interface Row {
  readonly a: ArchetypeView;
  readonly i: number;
}

function boardRow(q: Query): Row | null {
  for (const a of q.archetypes()) {
    const kind = a.column(Piece).kind;
    for (let i = 0; i < a.count; i++) if (kind[i] === Kind.Board) return { a, i };
  }
  return null;
}

/** Every tile row, in query order. */
function tileRows(q: Query): Row[] {
  const out: Row[] = [];
  for (const a of q.archetypes()) {
    const kind = a.column(Piece).kind;
    for (let i = 0; i < a.count; i++) if (kind[i] === Kind.Tile) out.push({ a, i });
  }
  return out;
}

/** The colour grid and which tile row sits on each cell. */
function gridOf(tiles: readonly Row[]): { grid: Grid; at: (Row | null)[] } {
  const grid = new Int32Array(SIZE * SIZE).fill(-1);
  const at: (Row | null)[] = new Array(SIZE * SIZE).fill(null);
  for (const t of tiles) {
    const p = t.a.column(Piece);
    const cell = cellOf(p.col[t.i]!, p.row[t.i]!);
    grid[cell] = p.colour[t.i]!;
    at[cell] = t;
  }
  return { grid, at };
}

function anyPress(i: { pressed(k: string): boolean; buttonPressed(b: "left"): boolean }): boolean {
  return i.buttonPressed("left") || i.pressed("Space") || i.pressed("Enter");
}

/** A new round: the moves refilled, a fresh deal from the dice placed on the tiles. */
function newRound(q: Query, board: Row, rng: Rng): void {
  const m = board.a.column(Match);
  const tune = board.a.column(Tuning);
  const b = board.i;
  m.phase[b] = Phase.Idle;
  m.phaseTicks[b] = 0;
  m.score[b] = 0;
  m.movesLeft[b] = tune.moves[b]!;
  m.chain[b] = 0;
  m.maxChain[b] = 0;
  m.cleared[b] = 0;
  m.lastCleared[b] = 0;
  m.shuffles[b] = 0;
  m.rounds[b] = m.rounds[b]! + 1;
  m.selected[b] = -1;
  m.dragFrom[b] = -1;
  m.swapA[b] = -1;
  m.swapB[b] = -1;
  const grid = deal(rng, tune.colours[b]!);
  tileRows(q).forEach((t, k) => {
    const p = t.a.column(Piece);
    p.col[t.i] = colOf(k);
    p.row[t.i] = rowOf(k);
    p.colour[t.i] = grid[k]!;
    p.marked[t.i] = 0;
    const c = cellCentre(colOf(k), rowOf(k));
    const pos = t.a.column(og.Transform).position;
    pos[t.i * 3] = c.x;
    pos[t.i * 3 + 1] = c.y;
  });
}

/** The title and the end cards: a press (after the card's hold) deals a new round. */
export const Flow = defineSystem(
  "Flow",
  { reads: [og.Input, Tuning], writes: [og.Rng, Match, Piece, og.Transform] },
  (q, _time, world) => {
    const board = boardRow(q);
    const input = world.resource(og.Input);
    const rng = world.resource(og.Rng);
    if (board === null || !input.ok || !rng.ok) return;
    const m = board.a.column(Match);
    const b = board.i;
    m.phaseTicks[b] = m.phaseTicks[b]! + 1;
    const phase = m.phase[b];
    const waiting = phase === Phase.Title || ((phase === Phase.Won || phase === Phase.Lost) && m.phaseTicks[b]! >= END_HOLD);
    if (waiting && anyPress(input.value)) newRound(q, board, rng.value);
  },
);

/** Begin a swap of two neighbouring cells (Resolve moves the tiles). */
function startSwap(m: View<typeof Match>, b: number, a: number, c: number): void {
  m.swapA[b] = a;
  m.swapB[b] = c;
  m.selected[b] = -1;
  m.dragFrom[b] = -1;
  m.phase[b] = Phase.Swap;
  m.phaseTicks[b] = 0;
}

/** Taps and drags: tap a tile, then a neighbour; or press a tile and drag towards a neighbour. */
export const Pick = defineSystem(
  "Pick",
  { reads: [og.Input, Piece], writes: [Match], after: ["Flow"] },
  (q, _time, world) => {
    const board = boardRow(q);
    const input = world.resource(og.Input);
    if (board === null || !input.ok) return;
    const m = board.a.column(Match);
    const b = board.i;
    const i = input.value;
    // Where the world lands on the play surface this tick: the orthographic view is
    // viewHeight metres tall over the render target. The target is in device pixels
    // and the pointer in CSS pixels; with no device pixel ratio inside a system yet,
    // this assumes 1 (headless runs and desktop browsers at 100%).
    const caps = og.render.capabilities(world);
    if (caps.width > 0 && caps.height > 0) {
      m.pxPerM[b] = caps.height / m.viewHeight[b]!;
      m.screenX0[b] = caps.width / 2;
      m.screenY0[b] = caps.height / 2 + m.viewCentreY[b]! * m.pxPerM[b]!;
    }
    // buttonsReleased & 1: the left button went up (the runtime lacks Input.buttonReleased yet).
    if ((i.buttonsReleased & 1) !== 0) m.dragFrom[b] = -1;
    // The press that dealt the round does not also select.
    if (m.phase[b] !== Phase.Idle || m.phaseTicks[b] === 0) return;
    // A swap asked for by the `swap` command.
    if (m.swapA[b]! >= 0 && adjacent(m.swapA[b]!, m.swapB[b]!)) return startSwap(m, b, m.swapA[b]!, m.swapB[b]!);
    const wx = (i.pointer.x - m.screenX0[b]!) / m.pxPerM[b]!;
    const wy = (m.screenY0[b]! - i.pointer.y) / m.pxPerM[b]!;
    const cell = cellAt(wx, wy);
    if (i.buttonPressed("left")) {
      const sel = m.selected[b]!;
      if (cell < 0) {
        m.selected[b] = -1;
      } else if (sel >= 0 && adjacent(sel, cell)) {
        startSwap(m, b, sel, cell);
      } else {
        m.selected[b] = sel === cell ? -1 : cell;
        m.dragFrom[b] = cell;
        m.pressX[b] = wx;
        m.pressY[b] = wy;
      }
      return;
    }
    const from = m.dragFrom[b]!;
    if (from < 0 || !i.buttonDown("left")) return;
    const dx = wx - m.pressX[b]!;
    const dy = wy - m.pressY[b]!;
    if (Math.max(Math.abs(dx), Math.abs(dy)) < PITCH * 0.45) return;
    const dc = Math.abs(dx) >= Math.abs(dy) ? Math.sign(dx) : 0;
    const dr = Math.abs(dx) >= Math.abs(dy) ? 0 : Math.sign(dy);
    const col = colOf(from) + dc;
    const row = rowOf(from) + dr;
    m.dragFrom[b] = -1;
    if (col < 0 || col >= SIZE || row < 0 || row >= SIZE) return;
    startSwap(m, b, from, cellOf(col, row));
  },
);

/** Moves every tile towards its cell by at most `step` metres per axis; true when all are there. */
function moveTiles(tiles: readonly Row[], step: number): boolean {
  let settled = true;
  for (const t of tiles) {
    const p = t.a.column(Piece);
    const target = cellCentre(p.col[t.i]!, p.row[t.i]!);
    const pos = t.a.column(og.Transform).position;
    for (const [axis, goal] of [[0, target.x], [1, target.y]] as const) {
      const v = pos[t.i * 3 + axis]!;
      const d = goal - v;
      if (Math.abs(d) <= step) pos[t.i * 3 + axis] = goal;
      else {
        pos[t.i * 3 + axis] = v + Math.sign(d) * step;
        settled = false;
      }
    }
  }
  return settled;
}

/** Exchange the cells of the tiles on cells a and b. */
function exchange(at: (Row | null)[], a: number, b: number): void {
  const ta = at[a];
  const tb = at[b];
  if (!ta || !tb) return;
  const pa = ta.a.column(Piece);
  const pb = tb.a.column(Piece);
  pa.col[ta.i] = colOf(b);
  pa.row[ta.i] = rowOf(b);
  pb.col[tb.i] = colOf(a);
  pb.row[tb.i] = rowOf(a);
}

/** Mark the tiles on the lines; returns how many. */
function mark(tiles: readonly Row[]): number {
  const { grid, at } = gridOf(tiles);
  const lines = findLines(grid);
  lines.forEach((on, cell) => {
    const t = at[cell];
    if (on && t) t.a.column(Piece).marked[t.i] = 1;
  });
  return count(lines);
}

/** Cleared tiles leave their columns: the rest fall, and the cleared rows come back above the board with new colours. */
function collapse(tiles: readonly Row[], rng: Rng, colours: number): void {
  for (let c = 0; c < SIZE; c++) {
    const column = tiles
      .filter((t) => t.a.column(Piece).col[t.i] === c)
      .sort((x, y) => x.a.column(Piece).row[x.i]! - y.a.column(Piece).row[y.i]!);
    const kept = column.filter((t) => !t.a.column(Piece).marked[t.i]);
    const gone = column.filter((t) => t.a.column(Piece).marked[t.i]);
    kept.forEach((t, r) => (t.a.column(Piece).row[t.i] = r));
    gone.forEach((t, k) => {
      const p = t.a.column(Piece);
      p.row[t.i] = kept.length + k;
      p.marked[t.i] = 0;
      p.colour[t.i] = rng.integer(0, colours - 1);
      const pos = t.a.column(og.Transform).position;
      pos[t.i * 3 + 1] = cellCentre(c, SIZE + k).y;
    });
  }
}

/** Swap, clear, fall and cascade; then the end of the round or the next move. */
export const Resolve = defineSystem(
  "Resolve",
  { reads: [Tuning], writes: [og.Rng, Match, Piece, og.Transform], after: ["Pick"] },
  (q, time, world) => {
    const board = boardRow(q);
    const rng = world.resource(og.Rng);
    if (board === null || !rng.ok) return;
    const m = board.a.column(Match);
    const tune = board.a.column(Tuning);
    const b = board.i;
    const phase = m.phase[b];
    if (phase !== Phase.Swap && phase !== Phase.SwapBack && phase !== Phase.Clear && phase !== Phase.Fall) return;
    const tiles = tileRows(q);
    const enter = (next: number) => {
      m.phase[b] = next;
      m.phaseTicks[b] = 0;
      if (next === Phase.Idle) {
        m.swapA[b] = -1;
        m.swapB[b] = -1;
      }
    };
    if (phase === Phase.Swap || phase === Phase.SwapBack) {
      if (phase === Phase.Swap && m.phaseTicks[b] === 0) exchange(gridOf(tiles).at, m.swapA[b]!, m.swapB[b]!);
      if (!moveTiles(tiles, PITCH / Math.max(1, tune.swapTicks[b]!))) return;
      if (phase === Phase.SwapBack) return enter(Phase.Idle);
      if (mark(tiles) === 0) {
        // A swap that makes no line goes back and costs nothing.
        exchange(gridOf(tiles).at, m.swapA[b]!, m.swapB[b]!);
        return enter(Phase.SwapBack);
      }
      m.movesLeft[b] = Math.max(0, m.movesLeft[b]! - 1);
      m.chain[b] = 1;
      m.maxChain[b] = Math.max(m.maxChain[b]!, 1);
      return enter(Phase.Clear);
    }
    if (phase === Phase.Clear) {
      if (m.phaseTicks[b]! < tune.clearTicks[b]!) return;
      let n = 0;
      for (const t of tiles) n += t.a.column(Piece).marked[t.i]!;
      m.score[b] = m.score[b]! + n * tune.tileScore[b]! * m.chain[b]!;
      m.cleared[b] = m.cleared[b]! + n;
      m.lastCleared[b] = n;
      collapse(tiles, rng.value, tune.colours[b]!);
      return enter(Phase.Fall);
    }
    // Fall.
    if (!moveTiles(tiles, tune.fallSpeed[b]! * PITCH * time.dt)) return;
    if (mark(tiles) > 0) {
      m.chain[b] = m.chain[b]! + 1;
      m.maxChain[b] = Math.max(m.maxChain[b]!, m.chain[b]!);
      return enter(Phase.Clear);
    }
    m.chain[b] = 0;
    if (m.score[b]! >= tune.target[b]! || m.movesLeft[b] === 0) {
      m.best[b] = Math.max(m.best[b]!, m.score[b]!);
      return enter(m.score[b]! >= tune.target[b]! ? Phase.Won : Phase.Lost);
    }
    if (!hasMove(gridOf(tiles).grid)) {
      // No swap makes a line: deal the colours again where the tiles stand.
      const grid = deal(rng.value, tune.colours[b]!);
      for (const t of tiles) {
        const p = t.a.column(Piece);
        p.colour[t.i] = grid[cellOf(p.col[t.i]!, p.row[t.i]!)]!;
      }
      m.shuffles[b] = m.shuffles[b]! + 1;
    }
    enter(Phase.Idle);
  },
);

/** What each row shows: tile colours (hidden above the board while they fall in), the clear's shrink, the selection frame. */
export const Animate = defineSystem(
  "Animate",
  { reads: [Match, Piece, Tuning], writes: [og.Mesh, og.Transform], after: ["Resolve"] },
  (q, time) => {
    const board = boardRow(q);
    if (board === null) return;
    const m = board.a.column(Match);
    const b = board.i;
    const phase = m.phase[b];
    const clearing = phase === Phase.Clear ? Math.min(1, m.phaseTicks[b]! / Math.max(1, board.a.column(Tuning).clearTicks[b]!)) : 0;
    const sel = phase === Phase.Idle ? m.selected[b]! : -1;
    // A triangle wave from the tick: deterministic on every host.
    const pulse = 0.9 + 0.08 * Math.abs(((time.tick % 30) / 30) * 2 - 1);
    for (const a of q.archetypes()) {
      const piece = a.column(Piece);
      const mesh = a.column(og.Mesh);
      const t = a.column(og.Transform);
      const looks = [piece.look0, piece.look1, piece.look2, piece.look3, piece.look4, piece.look5];
      for (let i = 0; i < a.count; i++) {
        const kind = piece.kind[i];
        if (kind === Kind.Tile) {
          const colour = piece.colour[i]!;
          // Hidden on the title and the end cards, and above the board while they fall in.
          const shown = colour >= 0 && phase !== Phase.Title && phase !== Phase.Won && phase !== Phase.Lost && t.position[i * 3 + 1]! < BOARD_TOP;
          mesh.visible[i] = shown ? 1 : 0;
          if (colour >= 0) {
            const look = looks[colour]!;
            mesh.material[i * 2] = look[i * 2]!;
            mesh.material[i * 2 + 1] = look[i * 2 + 1]!;
          }
          const cell = cellOf(piece.col[i]!, piece.row[i]!);
          const s = piece.marked[i] ? 0.92 * (1 - clearing) : cell === sel ? pulse : 0.92;
          t.scale[i * 3] = s * PITCH;
          t.scale[i * 3 + 1] = s * PITCH;
        } else if (kind === Kind.Cursor) {
          mesh.visible[i] = sel >= 0 ? 1 : 0;
          if (sel >= 0) {
            const c = cellCentre(colOf(sel), rowOf(sel));
            t.position[i * 3] = c.x;
            t.position[i * 3 + 1] = c.y;
          }
        }
      }
    }
  },
);

/** A rough advance for centring: Latin at 0.56 em, CJK at 1 em (the API has no text measure yet). */
function width(text: string, size: number): number {
  let w = 0;
  for (const c of text) w += c.charCodeAt(0) > 0x2e80 ? 1 : c === " " ? 0.3 : 0.56;
  return w * size;
}

const WHITE = { r: 1, g: 1, b: 1 };
const GOLD = { r: 1, g: 0.66, b: 0.05 };
const MINT = { r: 0.3, g: 0.85, b: 0.75 };
const CORAL = { r: 1, g: 0.25, b: 0.18 };
const MUTED = { r: 0.55, g: 0.6, b: 0.75 };

/** The HUD: score, moves and target; the title, won and lost cards. */
export const HudText = defineSystem("Hud", { reads: [Match, Hud, Tuning], after: ["Animate"] }, (q, _time, world) => {
  for (const a of q.archetypes()) {
    const m = a.column(Match);
    const hud = a.column(Hud);
    const tune = a.column(Tuning);
    for (let i = 0; i < a.count; i++) {
      const font = hud.font[i] as never;
      const top = hud.top[i]!;
      const text = (s: string, x: number, y: number, size: number, color = WHITE) =>
        og.render.text(world, { font, text: s, position: { x, y, z: 10 }, size, color, layer: 10 });
      const centred = (s: string, y: number, size: number, color = WHITE) => text(s, -width(s, size) / 2, y, size, color);
      const phase = m.phase[i];
      if (phase === Phase.Title) {
        centred("TILE SWAP", top - 1.5, 1.1, GOLD);
        centred("三消", top - 2.4, 0.6, WHITE);
        if (Math.floor(m.phaseTicks[i]! / 30) % 2 === 0) centred("Tap to start · 点击开始游戏", BOARD_TOP - 3.2, 0.42, MINT);
        centred(`Reach ${tune.target[i]} in ${tune.moves[i]} moves`, BOARD_TOP - 4.1, 0.38, MUTED);
        centred("Tap two neighbours, or drag · 点击或拖动交换相邻方块", BOARD_TOP - 4.7, 0.34, MUTED);
        continue;
      }
      const half = 3.5 * PITCH;
      text(`SCORE ${m.score[i]}`, -half, top - 1, 0.5);
      const moves = `MOVES ${m.movesLeft[i]}`;
      text(moves, half - width(moves, 0.5), top - 1, 0.5, m.movesLeft[i]! <= 3 ? CORAL : MINT);
      const target = `TARGET ${tune.target[i]}`;
      text(target, -half, top - 1.7, 0.36, MUTED);
      const best = `BEST ${m.best[i]}`;
      text(best, half - width(best, 0.36), top - 1.7, 0.36, MUTED);
      if (m.chain[i]! >= 2) centred(`CASCADE x${m.chain[i]}`, BOARD_TOP + 0.4, 0.45, GOLD);
      if (phase === Phase.Won || phase === Phase.Lost) {
        const won = phase === Phase.Won;
        centred(won ? "YOU WIN" : "OUT OF MOVES", BOARD_TOP - 2.6, 1, won ? GOLD : CORAL);
        centred(won ? "胜利" : "失败", BOARD_TOP - 3.5, 0.6, WHITE);
        centred(`Score ${m.score[i]}   Best ${m.best[i]}`, BOARD_TOP - 4.3, 0.5, WHITE);
        if (m.phaseTicks[i]! >= END_HOLD) centred("Tap to play again · 再来一次", BOARD_TOP - 5.1, 0.4, MINT);
      }
    }
  }
});
