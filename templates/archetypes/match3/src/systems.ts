// The match3 systems, in tick order: Flow (title, rounds, the end cards),
// Pick (taps and drags choose a swap), Resolve (swap, clear, fall, cascade,
// refill, the end of a round), Voices (the sounds Pick and Resolve asked
// for), Animate (what each row shows) and Hud.
//
// Each system sees the rows through its typed views. The board row
// (Piece.kind === Kind.Board) carries the round's state and tuning for every
// system (src/components.ts says why).
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
import { BOARD_TOP, GEM_SIZE, PITCH, SIZE, cellAt, cellCentre } from "./layout.ts";

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

/** A new round: the moves refilled, a fresh deal from the dice placed on the tiles, popping in from the top. */
function newRound(q: Query, board: Row, rng: Rng, world: World): void {
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
    if (waiting && anyPress(input.value)) newRound(q, board, rng.value, world);
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
  { reads: [og.Input, Piece], writes: [Match, Sounds], after: ["Flow"] },
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
        if (sel !== cell) ask(world, Sound.Select);
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

/** Start one tween group the phase waits for; with nothing to move, nothing to wait for. */
function waitFor(world: World, m: View<typeof Match>, b: number, tweens: readonly TweenDesc[], tag: number): void {
  m.wait[b] = 0;
  if (tweens.length === 0) return;
  const r = og.tween.parallel(world, tweens, { tag });
  if (!r.ok) throw new Error(`tween group ${tag}: ${r.reason}`);
  m.wait[b] = tag;
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
function startClear(world: World, tiles: readonly Row[], m: View<typeof Match>, tune: View<typeof Tuning>, b: number): void {
  const chain = m.chain[b]!;
  const font = m.font[b] as never;
  const { grid } = gridOf(tiles);
  let biggest = 0;
  for (const group of groupsOf(tiles)) {
    biggest = Math.max(biggest, group.length);
    let x = 0;
    let y = 0;
    for (const cell of group) {
      const c = cellCentre(colOf(cell), rowOf(cell));
      x += c.x / group.length;
      y += c.y / group.length;
    }
    og.render.floatingText(world, {
      font,
      text: `+${group.length * tune.tileScore[b]! * chain}`,
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
  const seconds = Math.max(1, tune.clearTicks[b]!) / 60;
  const tweens: TweenDesc[] = [];
  for (const t of tiles) {
    if (!t.a.column(Piece).marked[t.i]) continue;
    const entity = t.a.entity(t.i);
    og.tween.flash(world, { entity, amount: 0.85, duration: 0.12 });
    tweens.push({ entity, field: "scale", to: [0, 0, 1], duration: seconds, ease: "backIn" });
    tweens.push({ entity, component: og.Sprite, field: "opacity", to: [0], duration: seconds, ease: "quartIn" });
  }
  waitFor(world, m, b, tweens, Tag.Clear);
}

/** bounceOut first touches the ground this far into its duration. */
const BOUNCE_CONTACT = 1 / 2.75;

/**
 * Cleared tiles leave their columns: the rest fall, and the cleared ones come
 * back above the board with new colours and drop in. Each column starts a
 * little after the one to its left; every gem squashes as it lands.
 */
function collapse(world: World, tiles: readonly Row[], rng: Rng, colours: number, fallSpeed: number): TweenDesc[] {
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
      tr.position[t.i * 3 + 1] = cellCentre(c, SIZE + k).y;
      tr.scale[t.i * 3] = 1;
      tr.scale[t.i * 3 + 1] = 1;
    });
    let order = 0;
    for (const t of column) {
      const p = t.a.column(Piece);
      const goal = cellCentre(c, p.row[t.i]!).y;
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

/** Swap, clear, fall and cascade; then the end of the round or the next move. */
export const Resolve = defineSystem(
  "Resolve",
  { reads: [Tuning, og.TweenCompleted], writes: [og.Rng, Match, Piece, og.Transform, Sounds], after: ["Pick"] },
  (q, _time, world) => {
    const board = boardRow(q);
    const rng = world.resource(og.Rng);
    if (board === null || !rng.ok) return;
    const m = board.a.column(Match);
    const tune = board.a.column(Tuning);
    const b = board.i;
    for (const done of world.events(og.TweenCompleted)) if (done.tag === m.wait[b]) m.wait[b] = 0;
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
    if (phase === Phase.Swap && m.phaseTicks[b] === 0) {
      // Pick chose the pair this tick: slide them past each other, or there and back when no line comes of it.
      const { grid, at } = gridOf(tiles);
      const a = m.swapA[b]!;
      const c = m.swapB[b]!;
      const ta = at[a];
      const tc = at[c];
      if (!ta || !tc) return enter(Phase.Idle);
      const seconds = Math.max(1, tune.swapTicks[b]!) / 60;
      const to = (cell: number) => {
        const p = cellCentre(colOf(cell), rowOf(cell));
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
        waitFor(world, m, b, [slide(ta, c), slide(tc, a)], Tag.SwapBack);
        return enter(Phase.SwapBack);
      }
      exchange(at, a, c);
      waitFor(world, m, b, [slide(ta, c), slide(tc, a)], Tag.Swap);
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
      return startClear(world, tiles, m, tune, b);
    }
    if (phase === Phase.Clear) {
      let n = 0;
      for (const t of tiles) n += t.a.column(Piece).marked[t.i]!;
      m.score[b] = m.score[b]! + n * tune.tileScore[b]! * m.chain[b]!;
      m.cleared[b] = m.cleared[b]! + n;
      m.lastCleared[b] = n;
      waitFor(world, m, b, collapse(world, tiles, rng.value, tune.colours[b]!, tune.fallSpeed[b]!), Tag.Fall);
      return enter(Phase.Fall);
    }
    // Fall: every gem has landed.
    if (mark(tiles) > 0) {
      m.chain[b] = m.chain[b]! + 1;
      m.maxChain[b] = Math.max(m.maxChain[b]!, m.chain[b]!);
      enter(Phase.Clear);
      return startClear(world, tiles, m, tune, b);
    }
    m.chain[b] = 0;
    if (m.score[b]! >= tune.target[b]! || m.movesLeft[b] === 0) {
      m.best[b] = Math.max(m.best[b]!, m.score[b]!);
      const won = m.score[b]! >= tune.target[b]!;
      ask(world, won ? Sound.Won : Sound.Lost);
      return enter(won ? Phase.Won : Phase.Lost);
    }
    if (!hasMove(gridOf(tiles).grid)) {
      // No swap makes a line: deal the colours again where the gems stand, with a jolt.
      const grid = deal(rng.value, tune.colours[b]!);
      for (const t of tiles) {
        const p = t.a.column(Piece);
        p.colour[t.i] = grid[cellOf(p.col[t.i]!, p.row[t.i]!)]!;
        og.tween.squash(world, { entity: t.a.entity(t.i), impact: { x: 0.7, y: 1.3 }, duration: 0.3 });
      }
      m.shuffles[b] = m.shuffles[b]! + 1;
    }
    enter(Phase.Idle);
  },
);

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
 * selection frame's cell. Tweens write after the game's systems, so a running
 * tween of a field wins over what this writes.
 */
export const Animate = defineSystem(
  "Animate",
  { reads: [Match, Piece, Tuning], writes: [og.Sprite, og.Flipbook, og.Transform], after: ["Resolve"] },
  (q, time) => {
    const board = boardRow(q);
    if (board === null) return;
    const m = board.a.column(Match);
    const b = board.i;
    const phase = m.phase[b];
    const sel = phase === Phase.Idle ? m.selected[b]! : -1;
    const ended = phase === Phase.Won || phase === Phase.Lost;
    // A triangle wave from the tick: deterministic on every host.
    const pulse = 1.02 + 0.06 * Math.abs(((time.tick % 30) / 30) * 2 - 1);
    for (const a of q.archetypes()) {
      const piece = a.column(Piece);
      const sprite = a.column(og.Sprite);
      const book = a.column(og.Flipbook);
      const t = a.column(og.Transform);
      for (let i = 0; i < a.count; i++) {
        const kind = piece.kind[i];
        if (kind === Kind.Tile) {
          const colour = piece.colour[i]!;
          const cell = cellOf(piece.col[i]!, piece.row[i]!);
          // Hidden on the title; dimmed under the end cards.
          sprite.visible[i] = colour >= 0 && phase !== Phase.Title ? 1 : 0;
          if (!piece.marked[i]) sprite.opacity[i] = (ended ? 0.05 : 1) * entering(t.position[i * 3 + 1]!);
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
          sprite.visible[i] = sel >= 0 ? 1 : 0;
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

const WHITE = { r: 1, g: 1, b: 1 };
/** The brand ink #14161b, linear: the text's outline. */
const INK_LIN = { r: 0.007, g: 0.008, b: 0.0103 };
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
      const text = (s: string, x: number, y: number, size: number, color = WHITE, align: "start" | "center" | "end" = "start") =>
        og.render.text(world, { font, text: s, position: { x, y, z: 10 }, size, color, layer: 10, align, outline: 0.5, outlineColor: INK_LIN });
      const centred = (s: string, y: number, size: number, color = WHITE) => text(s, 0, y, size, color, "center");
      const phase = m.phase[i];
      if (phase === Phase.Title) {
        centred("TILE SWAP", top - 1.5, 1.1, GOLD);
        centred("三消", top - 2.4, 0.6, WHITE);
        if (Math.floor(m.phaseTicks[i]! / 30) % 2 === 0) centred("Tap to start · 点击开始游戏", BOARD_TOP - 3.2, 0.42, MINT);
        centred(`Reach ${tune.target[i]} in ${tune.moves[i]} moves`, BOARD_TOP - 4.1, 0.38, MUTED);
        // Two lines: on a portrait phone the view is narrower than the pair.
        centred("Tap two neighbours, or drag", BOARD_TOP - 4.5, 0.34, MUTED);
        centred("点击或拖动交换相邻方块", BOARD_TOP - 5.05, 0.34, MUTED);
        continue;
      }
      const half = 3.5 * PITCH;
      text(`SCORE ${m.score[i]}`, -half, top - 1, 0.5);
      const moves = `MOVES ${m.movesLeft[i]}`;
      text(moves, half, top - 1, 0.5, m.movesLeft[i]! <= 3 ? CORAL : MINT, "end");
      const target = `TARGET ${tune.target[i]}`;
      text(target, -half, top - 1.7, 0.36, MUTED);
      const best = `BEST ${m.best[i]}`;
      text(best, half, top - 1.7, 0.36, MUTED, "end");
      if (m.chain[i]! >= 2) centred(`CASCADE x${m.chain[i]}`, BOARD_TOP + 0.6, 0.45, GOLD);
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
