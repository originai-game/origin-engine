// The match3 systems, in tick order after src/seats.ts's Seats: Flow (title,
// rounds, the end cards), Pick (each seat's cursor and pick choose a swap on
// its own board), Resolve (swap, clear, fall, cascade, refill, and who won),
// Voices (the sounds Pick and Resolve asked for), Animate (what each row
// shows) and Hud; Frame (versus only) points each peer's camera.
//
// Each system sees the rows through its typed views. A board row
// (Piece.kind === Kind.Board) carries its seat's round and that seat's
// og.PlayerInput; seat 0's board carries the tuning both play to
// (src/components.ts says why). Everything that changes the world reads
// og.PlayerInput, never a device, so a replay and a room play the same.
//
// Versus (two seats): both boards are dealt the same grid (both then refill
// from og.Rng as they clear); the first to reach the target wins, else the higher score
// once both are out of moves.
//
// Motion is tweens, not per-tick code: a swap, a clear and a fall each start
// one tween group with a Tag, and the phase waits (Match.wait) for that
// group's og.TweenCompleted. Tweens advance by scaled time, so a hit stop
// holds the board too, and they are world state, so a replay lands on the
// same bits.

import { og, defineSystem } from "@origingame/engine";
import type { ArchetypeView, Query, Rng, TweenDesc, View, World } from "@origingame/engine";
import { IDLE_CYCLE, clearClip, idleClip } from "./art.ts";
import { adjacent, cellOf, colOf, count, deal, findLines, hasMove, rowOf } from "./board.ts";
import type { Grid } from "./board.ts";
import { Hud, Kind, Match, Phase, Piece, Sfx, Sound, Sounds, Tag, Tuning } from "./components.ts";
import { BOARDS, BOARD_GAP, BOARD_TOP, BOARD_Y, GEM_SIZE, PITCH, SIZE, boardX, cellAt, cellCentre } from "./layout.ts";
import { LOCAL, held, localSeats, pressedNow } from "./seats.ts";

/** Ticks an end card holds before a press starts the next round. */
const END_HOLD = 40;

interface Row {
  readonly a: ArchetypeView;
  readonly i: number;
}

/** The board rows by seat; null unless every seat's board is there. */
function boardRows(q: Query): Row[] | null {
  const out: Row[] = [];
  for (const a of q.archetypes()) {
    const piece = a.column(Piece);
    for (let i = 0; i < a.count; i++) if (piece.kind[i] === Kind.Board) out[piece.seat[i]!] = { a, i };
  }
  for (let s = 0; s < BOARDS; s++) if (!out[s]) return null;
  return out;
}

/** Every tile row of a seat's board, in query order. */
function tileRows(q: Query, seat: number): Row[] {
  const out: Row[] = [];
  for (const a of q.archetypes()) {
    const piece = a.column(Piece);
    for (let i = 0; i < a.count; i++) if (piece.kind[i] === Kind.Tile && piece.seat[i] === seat) out.push({ a, i });
  }
  return out;
}

/** Whether a board's round is over (its end card shows). */
const ended = (phase: number | undefined) => phase === Phase.Won || phase === Phase.Lost;

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

/** A new round on every board: the moves refilled, one fresh deal from the dice placed on each board's tiles, popping in from the top. */
function newRound(q: Query, boards: readonly Row[], rng: Rng, world: World): void {
  const tuning = boards[0]!;
  const tune = tuning.a.column(Tuning);
  const grid = deal(rng, tune.colours[tuning.i]!);
  boards.forEach((board, seat) => newBoard(q, board, seat, grid, tune.moves[tuning.i]!, world));
}

function newBoard(q: Query, board: Row, seat: number, grid: Grid, moves: number, world: World): void {
  const m = board.a.column(Match);
  const b = board.i;
  m.phase[b] = Phase.Idle;
  m.phaseTicks[b] = 0;
  m.score[b] = 0;
  m.movesLeft[b] = moves;
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
  tileRows(q, seat).forEach((t, k) => {
    const p = t.a.column(Piece);
    p.col[t.i] = colOf(k);
    p.row[t.i] = rowOf(k);
    p.colour[t.i] = grid[k]!;
    p.marked[t.i] = 0;
    const c = cellCentre(colOf(k), rowOf(k), seat);
    const tr = t.a.column(og.Transform);
    tr.position[t.i * 3] = c.x;
    tr.position[t.i * 3 + 1] = c.y;
    tr.scale[t.i * 3] = 0;
    tr.scale[t.i * 3 + 1] = 0;
    const entity = t.a.entity(t.i);
    og.tween.cancelEntity(world, entity);
    og.tween.to(world, {
      entity,
      field: "scale",
      to: [1, 1, 1],
      duration: 0.28,
      delay: (SIZE - 1 - rowOf(k)) * 0.035 + colOf(k) * 0.02,
      ease: "backOut",
      tag: Tag.Deal,
    });
  });
}

/** The title and the end cards: any seat's pick (after the cards' hold) deals a new round on every board. */
export const Flow = defineSystem(
  "Flow",
  { reads: [og.PlayerInput, Tuning], writes: [og.Rng, Match, Piece, og.Transform, Sounds] },
  (q, _time, world) => {
    const boards = boardRows(q);
    const rng = world.resource(og.Rng);
    const sounds = world.resource(Sounds);
    if (boards === null || !rng.ok) return;
    let waiting = true;
    let press = false;
    for (const board of boards) {
      const m = board.a.column(Match);
      const b = board.i;
      m.phaseTicks[b] = m.phaseTicks[b]! + 1;
      const phase = m.phase[b];
      waiting &&= phase === Phase.Title || (ended(phase) && m.phaseTicks[b]! >= END_HOLD);
      press ||= pressedNow(board.a.column(og.PlayerInput).buttons[board.i]!, m.before[b]!, "pick");
    }
    if (waiting && press) {
      newRound(q, boards, rng.value, world);
      // The press that deals is answered by a sound too (og-taste §6).
      if (sounds.ok) sounds.value.sfx |= Sound.Select;
    }
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

/** A held direction as a step: 0 none, 1 right, 2 left, 3 up, 4 down (the larger axis wins). */
function dirOf(x: number, y: number): number {
  if (Math.max(Math.abs(x), Math.abs(y)) < 0.5) return 0;
  if (Math.abs(x) >= Math.abs(y)) return x > 0 ? 1 : 2;
  return y > 0 ? 3 : 4;
}
const STEP = [[0, 0], [1, 0], [-1, 0], [0, 1], [0, -1]] as const;
/** A held direction steps again after this many ticks, then every REPEAT_EVERY. */
const REPEAT_AFTER = 14;
const REPEAT_EVERY = 6;

/**
 * Each seat on its own board, from its og.PlayerInput: with a pointer (the
 * cursor aimX, aimY moving), tap a tile then a neighbour, or press a tile and
 * drag towards a neighbour; with keys or a pad, a direction moves the hover,
 * pick selects it, and a direction then swaps the selection that way.
 */
export const Pick = defineSystem(
  "Pick",
  { reads: [og.PlayerInput, Piece, og.Camera, og.Lockstep], writes: [Match, Sounds], after: ["Flow"] },
  (q, _time, world) => {
    const boards = boardRows(q);
    if (boards === null) return;
    // Where the world lands on this device's play surface (diagnostics for the
    // playtests, in CSS pixels like the pointer). Not while a room runs: its
    // peers' windows differ, and each started from the authority's snapshot.
    const vp = og.ui.viewport(world);
    const cam = world.resource(og.Camera);
    const lock = world.resource(og.Lockstep);
    const room = lock.ok && lock.value.running;
    if (!room && cam.ok && vp.width > 0 && vp.height > 0) {
      const m = boards[0]!.a.column(Match);
      const b = boards[0]!.i;
      m.pxPerM[b] = vp.height / cam.value.height;
      m.screenX0[b] = vp.width / 2 - cam.value.position.x * m.pxPerM[b]!;
      m.screenY0[b] = vp.height / 2 + cam.value.position.y * m.pxPerM[b]!;
    }
    boards.forEach((board, seat) => pickOn(world, board, seat));
  },
);

function pickOn(world: World, board: Row, seat: number): void {
  const m = board.a.column(Match);
  const b = board.i;
  const input = board.a.column(og.PlayerInput);
  const buttons = input.buttons[b]!;
  const before = m.before[b]!;
  m.before[b] = buttons;
  const x = input.x[b]!;
  const y = input.y[b]!;
  const ax = input.aimX[b]!;
  const ay = input.aimY[b]!;
  // A cursor that moved with no direction held is a pointer's: pick acts on the cell under it.
  const dir = dirOf(x, y);
  if (dir === 0 && (Math.abs(ax - m.aimX[b]!) > 1e-4 || Math.abs(ay - m.aimY[b]!) > 1e-4)) {
    m.pointing[b] = 1;
    const under = cellAt(ax, ay, seat);
    if (under >= 0) m.hover[b] = under;
  }
  m.aimX[b] = ax;
  m.aimY[b] = ay;
  const holding = m.stepTicks[b]!;
  const step = dir !== 0 && (dir !== m.stepDir[b] || (holding + 1 >= REPEAT_AFTER && (holding + 1 - REPEAT_AFTER) % REPEAT_EVERY === 0));
  m.stepTicks[b] = dir !== 0 && dir === m.stepDir[b] ? holding + 1 : 0;
  m.stepDir[b] = dir;
  if (!held(buttons, "pick")) m.dragFrom[b] = -1;
  // The press that dealt the round does not also select; a board out of moves waits for the other.
  if (m.phase[b] !== Phase.Idle || m.phaseTicks[b] === 0 || m.movesLeft[b] === 0) return;
  // A swap asked for by the `swap` command.
  if (m.swapA[b]! >= 0 && adjacent(m.swapA[b]!, m.swapB[b]!)) return startSwap(m, b, m.swapA[b]!, m.swapB[b]!);
  if (step) {
    const [dc, dr] = STEP[dir]!;
    m.pointing[b] = 0;
    const sel = m.selected[b]!;
    const from = sel >= 0 ? sel : m.hover[b]!;
    const col = colOf(from) + dc;
    const row = rowOf(from) + dr;
    if (col < 0 || col >= SIZE || row < 0 || row >= SIZE) return;
    if (sel >= 0) return startSwap(m, b, sel, cellOf(col, row));
    m.hover[b] = cellOf(col, row);
    return;
  }
  if (pressedNow(buttons, before, "pick")) {
    const cell = m.pointing[b] ? cellAt(ax, ay, seat) : m.hover[b]!;
    const sel = m.selected[b]!;
    if (cell < 0) {
      m.selected[b] = -1;
    } else if (sel >= 0 && adjacent(sel, cell)) {
      startSwap(m, b, sel, cell);
    } else {
      m.selected[b] = sel === cell ? -1 : cell;
      if (sel !== cell) ask(world, Sound.Select);
      if (m.pointing[b]) {
        m.dragFrom[b] = cell;
        m.pressX[b] = ax;
        m.pressY[b] = ay;
      }
    }
    return;
  }
  const from = m.dragFrom[b]!;
  if (from < 0) return;
  const dx = ax - m.pressX[b]!;
  const dy = ay - m.pressY[b]!;
  if (Math.max(Math.abs(dx), Math.abs(dy)) < PITCH * 0.45) return;
  const dc = Math.abs(dx) >= Math.abs(dy) ? Math.sign(dx) : 0;
  const dr = Math.abs(dx) >= Math.abs(dy) ? 0 : Math.sign(dy);
  const col = colOf(from) + dc;
  const row = rowOf(from) + dr;
  m.dragFrom[b] = -1;
  if (col < 0 || col >= SIZE || row < 0 || row >= SIZE) return;
  startSwap(m, b, from, cellOf(col, row));
}

/** Start one tween group the phase waits for; with nothing to move, nothing to wait for. Each seat's board tags its groups apart (Tag.* + 8 × seat). */
function waitFor(world: World, m: View<typeof Match>, b: number, tweens: readonly TweenDesc[], tag: number, seat: number): void {
  m.wait[b] = 0;
  if (tweens.length === 0) return;
  const seatTag = tag + 8 * seat;
  const r = og.tween.parallel(world, tweens, { tag: seatTag });
  if (!r.ok) throw new Error(`tween group ${seatTag}: ${r.reason}`);
  m.wait[b] = seatTag;
}

/** The colour grid with cells a and b exchanged. */
function swapped(grid: Grid, a: number, b: number): Grid {
  const g = grid.slice();
  g[a] = grid[b]!;
  g[b] = grid[a]!;
  return g;
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

/** The marked cells in groups: touching cells of one colour (an L or a T is one group). */
function groupsOf(tiles: readonly Row[]): number[][] {
  const { grid, at } = gridOf(tiles);
  const marked = (cell: number) => {
    const t = at[cell];
    return t !== null && t !== undefined && t.a.column(Piece).marked[t.i] === 1;
  };
  const seen = new Uint8Array(SIZE * SIZE);
  const out: number[][] = [];
  for (let start = 0; start < SIZE * SIZE; start++) {
    if (seen[start] || !marked(start)) continue;
    const group: number[] = [];
    const stack = [start];
    seen[start] = 1;
    while (stack.length > 0) {
      const cell = stack.pop()!;
      group.push(cell);
      const c = colOf(cell);
      const r = rowOf(cell);
      for (const [dc, dr] of [[1, 0], [-1, 0], [0, 1], [0, -1]] as const) {
        const nc = c + dc;
        const nr = r + dr;
        if (nc < 0 || nc >= SIZE || nr < 0 || nr >= SIZE) continue;
        const n = cellOf(nc, nr);
        if (!seen[n] && marked(n) && grid[n] === grid[cell]) {
          seen[n] = 1;
          stack.push(n);
        }
      }
    }
    out.push(group.sort((x, y) => x - y));
  }
  return out;
}

/** The floating score's colour per gem (linear): the gem's own light. */
const GEM_LIGHT = [
  { r: 1, g: 0.32, b: 0.38 },
  { r: 0.42, g: 0.68, b: 1 },
  { r: 0.4, g: 1, b: 0.55 },
  { r: 1, g: 0.82, b: 0.25 },
  { r: 0.78, g: 0.5, b: 1 },
  { r: 1, g: 0.55, b: 0.3 },
];
const GOLD_LIN = { r: 1, g: 0.62, b: 0.08 };

/**
 * The lines just marked start clearing: each group's score floats up (a crit
 * on a cascade), the gems flash, play their burst and squash away; four or
 * more punch the camera in, a cascade shakes it, a third cascade stops time
 * for a beat.
 */
function startClear(world: World, tiles: readonly Row[], m: View<typeof Match>, b: number, tileScore: number, clearTicks: number, seat: number): void {
  const chain = m.chain[b]!;
  const font = m.font[b] as never;
  const { grid } = gridOf(tiles);
  let biggest = 0;
  for (const group of groupsOf(tiles)) {
    biggest = Math.max(biggest, group.length);
    let x = 0;
    let y = 0;
    for (const cell of group) {
      const c = cellCentre(colOf(cell), rowOf(cell), seat);
      x += c.x / group.length;
      y += c.y / group.length;
    }
    og.render.floatingText(world, {
      font,
      text: `+${group.length * tileScore * chain}`,
      position: { x, y: y + 0.1, z: 5 },
      size: 0.6,
      color: chain >= 2 ? GOLD_LIN : (GEM_LIGHT[grid[group[0]!]!] ?? GOLD_LIN),
      crit: chain >= 2,
      rise: 1.1,
      spread: 0.15,
      lifetime: 0.9,
    });
  }
  if (biggest >= 4) og.camera.punch(world, Math.min(0.12, 0.05 + 0.02 * (biggest - 4)));
  if (chain >= 2) og.camera.shake(world, Math.min(0.6, 0.16 * chain));
  if (chain >= 3) og.time.hitStop(world, 3);
  ask(world, chain >= 2 ? Sound.Cascade : Sound.Clear);
  const seconds = Math.max(1, clearTicks) / 60;
  const tweens: TweenDesc[] = [];
  for (const t of tiles) {
    if (!t.a.column(Piece).marked[t.i]) continue;
    const entity = t.a.entity(t.i);
    og.tween.flash(world, { entity, amount: 0.85, duration: 0.12 });
    tweens.push({ entity, field: "scale", to: [0, 0, 1], duration: seconds, ease: "backIn" });
    tweens.push({ entity, component: og.Sprite, field: "opacity", to: [0], duration: seconds, ease: "quartIn" });
  }
  waitFor(world, m, b, tweens, Tag.Clear, seat);
}

/** bounceOut first touches the ground this far into its duration. */
const BOUNCE_CONTACT = 1 / 2.75;

/**
 * Cleared tiles leave their columns: the rest fall, and the cleared ones come
 * back above the board with new colours and drop in. Each column starts a
 * little after the one to its left; every gem squashes as it lands.
 */
function collapse(world: World, tiles: readonly Row[], rng: Rng, colours: number, fallSpeed: number, seat: number): TweenDesc[] {
  const tweens: TweenDesc[] = [];
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
      const tr = t.a.column(og.Transform);
      tr.position[t.i * 3 + 1] = cellCentre(c, SIZE + k, seat).y;
      tr.scale[t.i * 3] = 1;
      tr.scale[t.i * 3 + 1] = 1;
    });
    let order = 0;
    for (const t of column) {
      const p = t.a.column(Piece);
      const goal = cellCentre(c, p.row[t.i]!, seat).y;
      const from = t.a.column(og.Transform).position[t.i * 3 + 1]!;
      const drop = from - goal;
      if (drop <= 1e-4) continue;
      const entity = t.a.entity(t.i);
      const delay = c * 0.025 + order * 0.03;
      const duration = 0.12 + drop / Math.max(0.1, fallSpeed * PITCH);
      order += 1;
      tweens.push({ entity, field: "position.y", to: [goal], duration, delay, ease: "bounceOut" });
      og.tween.to(world, { entity, field: "scale", from: [1.16, 0.84, 1], to: [1, 1, 1], duration: 0.2, delay: delay + duration * BOUNCE_CONTACT, ease: "backOut" });
    }
  }
  return tweens;
}

/** Ask for a sound this tick (Voices plays it). */
function ask(world: World, bit: number): void {
  const sounds = world.resource(Sounds);
  if (sounds.ok) sounds.value.sfx |= bit;
}

/** Swap, clear, fall and cascade on each board; then who won, or the next move. */
export const Resolve = defineSystem(
  "Resolve",
  { reads: [Tuning, og.TweenCompleted], writes: [og.Rng, Match, Piece, og.Transform, Sounds], after: ["Pick"] },
  (q, _time, world) => {
    const boards = boardRows(q);
    const rng = world.resource(og.Rng);
    if (boards === null || !rng.ok) return;
    const tuning = boards[0]!;
    const tune = tuning.a.column(Tuning);
    const t = tuning.i;
    const target = tune.target[t]!;
    const done = [...world.events(og.TweenCompleted)];
    boards.forEach((board, seat) => {
      const m = board.a.column(Match);
      for (const d of done) if (d.tag === m.wait[board.i]) m.wait[board.i] = 0;
      resolveOn(world, q, board, seat, rng.value, {
        target,
        colours: tune.colours[t]!,
        tileScore: tune.tileScore[t]!,
        swapTicks: tune.swapTicks[t]!,
        clearTicks: tune.clearTicks[t]!,
        fallSpeed: tune.fallSpeed[t]!,
      });
    });
    referee(world, boards, target);
  },
);

interface Rules {
  target: number;
  colours: number;
  tileScore: number;
  swapTicks: number;
  clearTicks: number;
  fallSpeed: number;
}

function resolveOn(world: World, q: Query, board: Row, seat: number, rng: Rng, tune: Rules): void {
  const m = board.a.column(Match);
  const b = board.i;
  const phase = m.phase[b];
  if (phase !== Phase.Swap && phase !== Phase.SwapBack && phase !== Phase.Clear && phase !== Phase.Fall) return;
  const tiles = tileRows(q, seat);
  const enter = (next: number) => {
    m.phase[b] = next;
    m.phaseTicks[b] = 0;
    if (next === Phase.Idle) {
      m.swapA[b] = -1;
      m.swapB[b] = -1;
    }
  };
  if (phase === Phase.Swap && m.phaseTicks[b] === 0) {
    // Pick chose the pair this tick: slide them past each other, or there and back when no line comes of it.
    const { grid, at } = gridOf(tiles);
    const a = m.swapA[b]!;
    const c = m.swapB[b]!;
    const ta = at[a];
    const tc = at[c];
    if (!ta || !tc) return enter(Phase.Idle);
    const seconds = Math.max(1, tune.swapTicks) / 60;
    const to = (cell: number) => {
      const p = cellCentre(colOf(cell), rowOf(cell), seat);
      return [p.x, p.y, 0];
    };
    const refused = count(findLines(swapped(grid, a, c))) === 0;
    const slide = (t: Row, cell: number): TweenDesc => ({
      entity: t.a.entity(t.i),
      field: "position",
      to: to(cell),
      duration: seconds,
      ease: "cubicInOut",
      ...(refused ? { loops: 1, yoyo: true } : {}),
    });
    ask(world, refused ? Sound.Refused : Sound.Swap);
    if (refused) {
      // A swap that makes no line goes there and back (one yoyo) and costs nothing.
      waitFor(world, m, b, [slide(ta, c), slide(tc, a)], Tag.SwapBack, seat);
      return enter(Phase.SwapBack);
    }
    exchange(at, a, c);
    waitFor(world, m, b, [slide(ta, c), slide(tc, a)], Tag.Swap, seat);
    return;
  }
  if (m.wait[b] !== Tag.None) return;
  if (phase === Phase.SwapBack) return enter(Phase.Idle);
  if (phase === Phase.Swap) {
    for (const t of tiles) {
      const p = t.a.column(Piece);
      const cell = cellOf(p.col[t.i]!, p.row[t.i]!);
      if (cell === m.swapA[b] || cell === m.swapB[b]) og.tween.squash(world, { entity: t.a.entity(t.i), impact: { x: 1.12, y: 0.9 }, duration: 0.14 });
    }
    mark(tiles);
    m.movesLeft[b] = Math.max(0, m.movesLeft[b]! - 1);
    m.chain[b] = 1;
    m.maxChain[b] = Math.max(m.maxChain[b]!, 1);
    enter(Phase.Clear);
    return startClear(world, tiles, m, b, tune.tileScore, tune.clearTicks, seat);
  }
  if (phase === Phase.Clear) {
    let n = 0;
    for (const t of tiles) n += t.a.column(Piece).marked[t.i]!;
    m.score[b] = m.score[b]! + n * tune.tileScore * m.chain[b]!;
    m.cleared[b] = m.cleared[b]! + n;
    m.lastCleared[b] = n;
    waitFor(world, m, b, collapse(world, tiles, rng, tune.colours, tune.fallSpeed, seat), Tag.Fall, seat);
    return enter(Phase.Fall);
  }
  // Fall: every gem has landed.
  if (mark(tiles) > 0) {
    m.chain[b] = m.chain[b]! + 1;
    m.maxChain[b] = Math.max(m.maxChain[b]!, m.chain[b]!);
    enter(Phase.Clear);
    return startClear(world, tiles, m, b, tune.tileScore, tune.clearTicks, seat);
  }
  m.chain[b] = 0;
  // At the target or out of moves the board settles as it is, and the referee decides.
  if (m.score[b]! < tune.target && m.movesLeft[b]! > 0 && !hasMove(gridOf(tiles).grid)) {
    // No swap makes a line: deal the colours again where the gems stand, with a jolt.
    const grid = deal(rng, tune.colours);
    for (const t of tiles) {
      const p = t.a.column(Piece);
      p.colour[t.i] = grid[cellOf(p.col[t.i]!, p.row[t.i]!)]!;
      og.tween.squash(world, { entity: t.a.entity(t.i), impact: { x: 0.7, y: 1.3 }, duration: 0.3 });
    }
    m.shuffles[b] = m.shuffles[b]! + 1;
  }
  enter(Phase.Idle);
}

/**
 * The end of a round, once a board has settled: alone, the target wins and
 * running out of moves loses. In versus the first board settled at the
 * target wins (both on one tick: the higher score; a tie is a draw, both
 * winning); else, once both are out of moves, the higher score wins.
 */
function referee(world: World, boards: readonly Row[], target: number): void {
  const at = boards.map((r) => ({ m: r.a.column(Match), b: r.i }));
  if (at.some(({ m, b }) => m.phase[b] === Phase.Title || ended(m.phase[b]))) return;
  const settled = at.map(({ m, b }) => m.phase[b] === Phase.Idle);
  const reached = at.filter(({ m, b }, s) => settled[s] && m.score[b]! >= target);
  const out = at.every(({ m, b }, s) => settled[s] && m.movesLeft[b] === 0);
  if (reached.length === 0 && !out) return;
  const contenders = reached.length > 0 ? reached : at;
  const top = Math.max(...contenders.map(({ m, b }) => m.score[b]!));
  for (const r of at) {
    const { m, b } = r;
    const won = contenders.includes(r) && m.score[b] === top && (BOARDS > 1 || m.score[b]! >= target);
    m.best[b] = Math.max(m.best[b]!, m.score[b]!);
    m.phase[b] = won ? Phase.Won : Phase.Lost;
    m.phaseTicks[b] = 0;
    m.selected[b] = -1;
    m.dragFrom[b] = -1;
    ask(world, won ? Sound.Won : Sound.Lost);
  }
}

/** Opacity of a gem by height: it fades in over the top half cell as it drops onto the board. */
function entering(y: number): number {
  return Math.min(1, Math.max(0, (BOARD_TOP - y) / (0.5 * PITCH)));
}

/** Plays each requested sound once: a voice already playing is stopped this tick and restarted the next. */
export const Voices = defineSystem("Voices", { writes: [Sfx, Sounds, og.AudioSource], after: ["Resolve"] }, (q, _time, world) => {
  const sounds = world.resource(Sounds);
  if (!sounds.ok || sounds.value.sfx === 0) return;
  const s = sounds.value;
  let left = s.sfx;
  for (const a of q.archetypes()) {
    const sfx = a.column(Sfx);
    const src = a.column(og.AudioSource);
    for (let i = 0; i < a.count; i++) {
      const b = sfx.bit[i]!;
      if ((s.sfx & b) === 0) continue;
      if (src.playing[i]) {
        src.playing[i] = 0;
      } else {
        src.playing[i] = 1;
        sfx.plays[i] = sfx.plays[i]! + 1;
        left &= ~b;
      }
    }
  }
  s.sfx = left;
});

/**
 * What each row shows: each gem's clip (its colour's idle loop, or the burst
 * while it clears), visibility and fade-in, the selected gem's pulse, the
 * selection frame's cell (the selection, or the hover while keys or a pad
 * move it). Tweens write after the game's systems, so a running tween of a
 * field wins over what this writes.
 */
export const Animate = defineSystem(
  "Animate",
  { reads: [Match, Piece, Tuning], writes: [og.Sprite, og.Flipbook, og.Transform], after: ["Resolve"] },
  (q, time) => {
    const boards = boardRows(q);
    if (boards === null) return;
    const shown = boards.map((board) => {
      const m = board.a.column(Match);
      const b = board.i;
      const phase = m.phase[b];
      const sel = phase === Phase.Idle ? m.selected[b]! : -1;
      const hover = phase === Phase.Idle && !m.pointing[b] && m.movesLeft[b]! > 0 ? m.hover[b]! : -1;
      return { phase, sel, frame: sel >= 0 ? sel : hover, ended: ended(phase) };
    });
    // A triangle wave from the tick: deterministic on every host.
    const pulse = 1.02 + 0.06 * Math.abs(((time.tick % 30) / 30) * 2 - 1);
    for (const a of q.archetypes()) {
      const piece = a.column(Piece);
      const sprite = a.column(og.Sprite);
      const book = a.column(og.Flipbook);
      const t = a.column(og.Transform);
      for (let i = 0; i < a.count; i++) {
        const kind = piece.kind[i];
        const seat = piece.seat[i]!;
        const board = shown[seat];
        if (!board) continue;
        const { phase, sel } = board;
        if (kind === Kind.Tile) {
          const colour = piece.colour[i]!;
          const cell = cellOf(piece.col[i]!, piece.row[i]!);
          // Hidden on the title; dimmed under the end cards.
          sprite.visible[i] = colour >= 0 && phase !== Phase.Title ? 1 : 0;
          if (!piece.marked[i]) sprite.opacity[i] = (board.ended ? 0.05 : 1) * entering(t.position[i * 3 + 1]!);
          const size = GEM_SIZE * PITCH * (cell === sel ? pulse : 1);
          sprite.width[i] = size;
          sprite.height[i] = size;
          if (colour >= 0) {
            const clip = piece.marked[i] && phase === Phase.Clear ? clearClip(colour) : idleClip(colour);
            if (book.clip[i] !== clip) {
              book.clip[i] = clip;
              // Each gem's glint at its own moment: seek the idle loop by the cell.
              if (clip === idleClip(colour)) book.time[i] = (cell * 0.37) % IDLE_CYCLE;
            }
          }
        } else if (kind === Kind.Cursor) {
          const at = board.frame;
          sprite.visible[i] = at >= 0 ? 1 : 0;
          if (at >= 0) {
            const c = cellCentre(colOf(at), rowOf(at), seat);
            t.position[i * 3] = c.x;
            t.position[i * 3 + 1] = c.y;
          }
        }
      }
    }
  },
);

const WHITE = { r: 1, g: 1, b: 1 };
/** The brand ink #14161b, linear: the text's outline. */
const INK_LIN = { r: 0.007, g: 0.008, b: 0.0103 };
const GOLD = { r: 1, g: 0.66, b: 0.05 };
const MINT = { r: 0.3, g: 0.85, b: 0.75 };
const CORAL = { r: 1, g: 0.25, b: 0.18 };
const MUTED = { r: 0.55, g: 0.6, b: 0.75 };

/**
 * The HUD: each board's score, moves, target and best, under the top of
 * this peer's view (og.Camera: drawing only, the world never reads it); the
 * title, and each board's won, lost or waiting card.
 */
export const HudText = defineSystem("Hud", { reads: [Match, Hud, Tuning, Piece, og.Camera], after: ["Animate"] }, (q, _time, world) => {
  const cam = world.resource(og.Camera);
  if (!cam.ok) return;
  const top = cam.value.position.y + cam.value.height / 2;
  const middle = cam.value.position.x;
  let target = 0;
  let moves = 0;
  for (const a of q.archetypes()) {
    const tune = a.column(Tuning);
    const piece = a.column(Piece);
    for (let i = 0; i < a.count; i++) {
      if (piece.seat[i] === 0) {
        target = tune.target[i]!;
        moves = tune.moves[i]!;
      }
    }
  }
  for (const a of q.archetypes()) {
    const m = a.column(Match);
    const hud = a.column(Hud);
    const piece = a.column(Piece);
    for (let i = 0; i < a.count; i++) {
      const font = hud.font[i] as never;
      const seat = piece.seat[i]!;
      const x0 = boardX(seat);
      const text = (s: string, x: number, y: number, size: number, color = WHITE, align: "start" | "center" | "end" = "start") =>
        og.render.text(world, { font, text: s, position: { x, y, z: 10 }, size, color, layer: 10, align, outline: 0.5, outlineColor: INK_LIN });
      const centred = (s: string, y: number, size: number, color = WHITE) => text(s, x0, y, size, color, "center");
      const phase = m.phase[i];
      if (phase === Phase.Title) {
        // One title over the whole view, drawn by seat 0's board.
        if (seat !== 0) continue;
        const title = (s: string, y: number, size: number, color = WHITE) => text(s, middle, y, size, color, "center");
        title("TILE SWAP", top - 1.5, 1.1, GOLD);
        title(BOARDS > 1 ? "三消 · VS" : "三消", top - 2.4, 0.6, WHITE);
        if (Math.floor(m.phaseTicks[i]! / 30) % 2 === 0) title("Tap to start · 点击开始游戏", BOARD_TOP - 3.2, 0.42, MINT);
        title(BOARDS > 1 ? `First to ${target} in ${moves} moves wins` : `Reach ${target} in ${moves} moves`, BOARD_TOP - 4.1, 0.38, MUTED);
        // Two lines: on a portrait phone the view is narrower than the pair.
        title("Tap two neighbours, or drag", BOARD_TOP - 4.5, 0.34, MUTED);
        title("点击或拖动交换相邻方块", BOARD_TOP - 5.05, 0.34, MUTED);
        continue;
      }
      const half = 3.5 * PITCH;
      text(`SCORE ${m.score[i]}`, x0 - half, top - 1, 0.5);
      text(`MOVES ${m.movesLeft[i]}`, x0 + half, top - 1, 0.5, m.movesLeft[i]! <= 3 ? CORAL : MINT, "end");
      text(`TARGET ${target}`, x0 - half, top - 1.7, 0.36, MUTED);
      text(`BEST ${m.best[i]}`, x0 + half, top - 1.7, 0.36, MUTED, "end");
      if (BOARDS > 1) centred(`P${seat + 1}`, top - 1.7, 0.45, seat === 0 ? GOLD : MINT);
      if (m.chain[i]! >= 2) centred(`CASCADE x${m.chain[i]}`, BOARD_TOP + 0.6, 0.45, GOLD);
      if (BOARDS > 1 && phase === Phase.Idle && m.movesLeft[i] === 0) {
        centred("OUT OF MOVES", BOARD_TOP - 3, 0.6, CORAL);
        centred(`waiting for P${2 - seat}`, BOARD_TOP - 3.8, 0.38, MUTED);
      }
      if (ended(phase)) {
        const won = phase === Phase.Won;
        let draw = false;
        if (BOARDS > 1) for (let j = 0; j < a.count; j++) draw ||= j !== i && m.phase[j] === Phase.Won && won;
        const head = BOARDS === 1 ? (won ? "YOU WIN" : "OUT OF MOVES") : draw ? "DRAW" : won ? `P${seat + 1} WINS` : "BEATEN";
        centred(head, BOARD_TOP - 2.6, 1, won ? GOLD : CORAL);
        // The font holds the HUD's own CJK characters (assets/fonts/ui.ttf.ogt): a draw has none.
        if (!draw) centred(won ? "胜利" : "失败", BOARD_TOP - 3.5, 0.6, WHITE);
        centred(`Score ${m.score[i]}   Best ${m.best[i]}`, BOARD_TOP - 4.3, 0.5, WHITE);
        if (m.phaseTicks[i]! >= END_HOLD) centred("Tap to play again · 再来一次", BOARD_TOP - 5.1, 0.4, MINT);
      }
    }
  }
});

/** The view keeps at least this much of the world visible around one board. */
export const MIN_VIEW = { width: 8.6, height: 12 } as const;
/** And around both boards of a versus game. */
const BOTH_WIDTH = 2 * BOARD_GAP + MIN_VIEW.width;
/** The view centre: the board's top sits under the HUD. */
export const VIEW_Y = BOARD_Y + 0.9;

/**
 * This peer's view (og.Camera, per peer and never hashed): one board alone;
 * both boards on one device; in a room both while they fit a screen wide
 * enough, else this peer's own board (a portrait phone).
 */
export function frameView(world: World): void {
  const cam = world.resource(og.Camera);
  if (!cam.ok) return;
  // The play surface's shape (CSS px), else the render target's, else 16:9.
  const vp = og.ui.viewport(world);
  const caps = og.render.capabilities(world);
  const aspect = vp.width > 0 && vp.height > 0 ? vp.width / vp.height : caps.width > 0 && caps.height > 0 ? caps.width / caps.height : 16 / 9;
  const one = Math.max(MIN_VIEW.height, MIN_VIEW.width / aspect);
  const both = Math.max(MIN_VIEW.height, BOTH_WIDTH / aspect);
  let x = 0;
  let height = BOARDS > 1 ? both : one;
  if (BOARDS > 1 && !LOCAL && both > one * 1.4) {
    const seat = localSeats(world)[0] ?? 0;
    x = boardX(seat);
    height = one;
  }
  cam.value.height = height;
  cam.value.position = { x, y: VIEW_Y, z: 50 };
}

/** Versus: keeps each peer's view on its boards as the window and the room change. */
export const Frame = defineSystem("Frame", { reads: [og.Lockstep], writes: [og.Camera], after: ["Hud"] }, (_q, _time, world) => frameView(world));
