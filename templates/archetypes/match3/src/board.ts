// Pure board logic over a colour grid: line detection, the "is there a move"
// test and the deal. A grid is SIZE * SIZE colour indices, row-major from the
// bottom row (cell = row * SIZE + col); -1 is an empty cell. Nothing here
// touches the world, so systems and commands share it.

import { SIZE } from "./layout.ts";

export type Grid = Int32Array;

export const cellOf = (col: number, row: number) => row * SIZE + col;
export const colOf = (cell: number) => cell % SIZE;
export const rowOf = (cell: number) => Math.floor(cell / SIZE);

/** Whether two cells share an edge. */
export function adjacent(a: number, b: number): boolean {
  if (a < 0 || b < 0) return false;
  const dc = Math.abs(colOf(a) - colOf(b));
  const dr = Math.abs(rowOf(a) - rowOf(b));
  return dc + dr === 1;
}

/** Cells in horizontal or vertical runs of three or more of one colour. */
export function findLines(g: Grid): Uint8Array {
  const marked = new Uint8Array(SIZE * SIZE);
  for (let r = 0; r < SIZE; r++) {
    for (let c = 0; c < SIZE; ) {
      const colour = g[cellOf(c, r)]!;
      let n = 1;
      while (c + n < SIZE && colour >= 0 && g[cellOf(c + n, r)] === colour) n++;
      if (colour >= 0 && n >= 3) for (let k = 0; k < n; k++) marked[cellOf(c + k, r)] = 1;
      c += n;
    }
  }
  for (let c = 0; c < SIZE; c++) {
    for (let r = 0; r < SIZE; ) {
      const colour = g[cellOf(c, r)]!;
      let n = 1;
      while (r + n < SIZE && colour >= 0 && g[cellOf(c, r + n)] === colour) n++;
      if (colour >= 0 && n >= 3) for (let k = 0; k < n; k++) marked[cellOf(c, r + k)] = 1;
      r += n;
    }
  }
  return marked;
}

export function count(marked: Uint8Array): number {
  let n = 0;
  for (const m of marked) n += m;
  return n;
}

/** Whether some swap of neighbours makes a line. */
export function hasMove(g: Grid): boolean {
  const t = Int32Array.from(g);
  for (let cell = 0; cell < SIZE * SIZE; cell++) {
    for (const other of [cell + 1, cell + SIZE]) {
      if (other >= SIZE * SIZE || !adjacent(cell, other)) continue;
      [t[cell], t[other]] = [t[other]!, t[cell]!];
      const lines = count(findLines(t)) > 0;
      [t[cell], t[other]] = [t[other]!, t[cell]!];
      if (lines) return true;
    }
  }
  return false;
}

/** A source of dice: Rng or an RngStream. */
export interface Dice {
  integer(low: number, high: number): number;
}

/**
 * A fresh board from the dice: no ready line, at least one move. A cell's
 * colour is redrawn while it would complete a line with the two cells to its
 * left or below it; the whole board is redrawn while it has no move.
 */
export function deal(dice: Dice, colours: number): Grid {
  const g = new Int32Array(SIZE * SIZE);
  for (let attempt = 0; ; attempt++) {
    for (let r = 0; r < SIZE; r++) {
      for (let c = 0; c < SIZE; c++) {
        let colour = dice.integer(0, colours - 1);
        for (let k = 0; k < 8 && makesLine(g, c, r, colour); k++) colour = dice.integer(0, colours - 1);
        if (makesLine(g, c, r, colour)) colour = (colour + 1) % colours;
        if (makesLine(g, c, r, colour)) colour = (colour + 1) % colours;
        g[cellOf(c, r)] = colour;
      }
    }
    if (hasMove(g) || attempt >= 20) return g;
  }
}

function makesLine(g: Grid, c: number, r: number, colour: number): boolean {
  const left = c >= 2 && g[cellOf(c - 1, r)] === colour && g[cellOf(c - 2, r)] === colour;
  const below = r >= 2 && g[cellOf(c, r - 1)] === colour && g[cellOf(c, r - 2)] === colour;
  return left || below;
}

/** A board written as text: SIZE lines of SIZE digits, the top row first (as it reads on screen), `/` between lines. */
export function parseBoard(text: string, colours: number): Grid | string {
  const lines = text.split("/");
  if (lines.length !== SIZE || lines.some((l) => l.length !== SIZE)) return `write ${SIZE} lines of ${SIZE} digits joined by '/', the top row first`;
  const g = new Int32Array(SIZE * SIZE);
  for (let i = 0; i < SIZE; i++) {
    for (let c = 0; c < SIZE; c++) {
      const d = Number(lines[i]![c]);
      if (!Number.isInteger(d) || d < 0 || d >= colours) return `'${lines[i]![c]}' is not a colour index below ${colours}`;
      g[cellOf(c, SIZE - 1 - i)] = d;
    }
  }
  return g;
}
