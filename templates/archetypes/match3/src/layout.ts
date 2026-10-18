// Where the board sits, in metres (no engine imports, so playtests read it too).

/** The board is SIZE x SIZE cells. */
export const SIZE = 7;
/** Cell pitch. */
export const PITCH = 1;
/** A gem's size in its cell. */
export const GEM_SIZE = 0.92;
/** The board's centre height. */
export const BOARD_Y = -0.4;

/** The centre of a cell. */
export function cellCentre(col: number, row: number): { x: number; y: number } {
  return { x: (col - (SIZE - 1) / 2) * PITCH, y: BOARD_Y + (row - (SIZE - 1) / 2) * PITCH };
}

/** The cell under a world point, or -1 off the board. */
export function cellAt(x: number, y: number): number {
  const col = Math.floor(x / PITCH + SIZE / 2);
  const row = Math.floor((y - BOARD_Y) / PITCH + SIZE / 2);
  return col >= 0 && col < SIZE && row >= 0 && row < SIZE ? row * SIZE + col : -1;
}

/** The board's top edge. */
export const BOARD_TOP = BOARD_Y + (SIZE / 2) * PITCH;
