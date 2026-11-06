// Where the boards sit, in metres (no engine imports, so playtests read it too).
// Alone, one board at the centre; in versus (two seats) one board per seat,
// seat 0 on the left and seat 1 on the right. Every peer of a room places
// them the same way: the layout is world state, never the window's.

import { creation } from "./creation.ts";

/** The boards in play: one per seat. */
export const BOARDS: number = creation.seats;
/** The board is SIZE x SIZE cells. */
export const SIZE = 7;
/** Cell pitch. */
export const PITCH = 1;
/** A gem's size in its cell. */
export const GEM_SIZE = 0.92;
/** The board's centre height. */
export const BOARD_Y = -0.4;
/** How far each board's centre sits from the middle in versus. */
export const BOARD_GAP = 4.4;

/** The centre x of a seat's board. */
export function boardX(seat: number): number {
  return BOARDS > 1 ? (seat === 0 ? -BOARD_GAP : BOARD_GAP) : 0;
}

/** The centre of a cell of a seat's board. */
export function cellCentre(col: number, row: number, seat = 0): { x: number; y: number } {
  return { x: boardX(seat) + (col - (SIZE - 1) / 2) * PITCH, y: BOARD_Y + (row - (SIZE - 1) / 2) * PITCH };
}

/** The cell of a seat's board under a world point, or -1 off that board. */
export function cellAt(x: number, y: number, seat = 0): number {
  const col = Math.floor((x - boardX(seat)) / PITCH + SIZE / 2);
  const row = Math.floor((y - BOARD_Y) / PITCH + SIZE / 2);
  return col >= 0 && col < SIZE && row >= 0 && row < SIZE ? row * SIZE + col : -1;
}

/** The board's top edge. */
export const BOARD_TOP = BOARD_Y + (SIZE / 2) * PITCH;
