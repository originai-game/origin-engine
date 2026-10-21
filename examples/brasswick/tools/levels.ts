// Brasswick's three levels as character grids, built by a few rectangles and
// placed things each. tools/make-assets.ts writes assets/maps/level-<n>.ogt
// from these; the game reads every position from the maps' objects
// (src/level.ts), never from here.
//
// One cell is one world unit (16 px). Rows count from the top. The legend:
//
//   #  vault plate: the `plate` terrain (blob47, auto-tiled), solid
//   .  open (the back wall behind it)
//   =  a grate: a one-way plank (drawn on `planks`, its collider an object)
//   ^  spikes on the cell's floor (drawn on `decor`, a `spikes` object)
//   P  Wick's spawn: the feet at the bottom of the cell
//   C  a crate (a dynamic box of one cell)
//   a b j   a pressure plate on the cell's floor (a opens door A, b door B, j the gate J)
//   A B     a door's bottom cell: the door fills it and the open cells above, up to the ceiling
//   J       the gate's bottom cell: a slider on a prismatic joint, three cells tall
//   G  a gear (a collectible)
//   L  a lamp: a checkpoint, lit when Wick passes it
//   E  the exit hatch's bottom-left cell (two by two)
//   T  a tick (a clockwork beetle) walking the floor it stands on
//   *  a wall lamp (an og.Light2D)
//   V  a steam vent (decor and an emitter)
//   M  a lift's left cell (three cells wide, half a cell thick, its top at the cell's top);
//   m  where the lift's left cell turns back

export interface Level {
  id: string;
  /** The string key of its name (assets/strings/*.ogt). */
  name: string;
  /** Lighting2D's ambient, linear: how dark the vault is here. */
  ambient: [number, number, number];
  /** Seconds the third star asks for. */
  par: number;
  rows: string[];
}

class Grid {
  readonly cells: string[][];
  constructor(readonly w: number, readonly h: number) {
    this.cells = Array.from({ length: h }, (_, y) => Array.from({ length: w }, (_, x) => (x === 0 || y === 0 || x === w - 1 || y === h - 1 ? "#" : ".")));
  }
  fill(x0: number, y0: number, x1: number, y1: number, c = "#"): this {
    for (let y = y0; y <= y1; y++) for (let x = x0; x <= x1; x++) this.cells[y]![x] = c;
    return this;
  }
  put(c: string, ...at: [number, number][]): this {
    for (const [x, y] of at) {
      if (this.cells[y]?.[x] === undefined) throw new Error(`(${x}, ${y}) is off the grid`);
      this.cells[y]![x] = c;
    }
    return this;
  }
  /** The rows, with solid rows added on top until the map is `tall` (the view is 22.5 cells: a shorter map shows the void round it). */
  rows(tall = 23): string[] {
    const pad = Array.from({ length: Math.max(0, tall - this.h) }, () => "#".repeat(this.w));
    return [...pad, ...this.cells.map((r) => r.join(""))];
  }
}

/** The Boiler Stair: the spikes, a crate as a step, a crate on a plate, the wall-jump shaft. */
function level1(): string[] {
  const g = new Grid(52, 20);
  g.fill(1, 17, 50, 18); // the floor, top at row 17
  g.fill(11, 17, 13, 17, ".").fill(11, 18, 13, 18, "^"); // the spike pit
  g.fill(23, 13, 50, 18); // the ledge, four cells up: a crate's step
  g.fill(30, 1, 39, 8); // the ceiling over the door's corridor (four cells: Wick hops over a crate under it); its right face is the shaft's left wall
  g.fill(43, 6, 50, 12); // the exit's shelf: the shaft's right wall
  g.fill(15, 13, 17, 13, "="); // a grate a crate's step up: the high gear
  g.put("P", [3, 16]).put("C", [20, 16], [28, 12]).put("a", [31, 12]).put("A", [35, 12]);
  g.put("L", [25, 12]).put("E", [47, 5]);
  g.put("G", [8, 14], [12, 13], [16, 10], [27, 10], [41, 8], [42, 2]);
  g.put("*", [5, 10], [22, 5], [41, 11]);
  return g.rows();
}

/** Counterweights: a crate dropped from a slab, ticks, grates, a crate into a pit. */
function level2(): string[] {
  const g = new Grid(56, 20);
  g.fill(1, 17, 54, 18);
  g.put(".", [41, 17], [42, 17]); // the pit the second crate drops into (two cells: a crate slides over one)
  g.fill(15, 11, 19, 12); // the slab the first crate waits on
  g.fill(22, 1, 26, 13); // over door A, two cells from the slab: room for a crate to tip and fall
  g.fill(44, 1, 48, 13); // over door B
  g.fill(10, 14, 13, 14, "=").fill(31, 14, 34, 14, "=");
  g.put("P", [2, 16]).put("T", [7, 16], [30, 16]).put("C", [17, 10], [36, 16]);
  g.put("a", [21, 16]).put("A", [24, 16]).put("b", [42, 17]).put("B", [46, 16]);
  g.put("L", [28, 16]).put("V", [39, 16]).put("E", [51, 16]);
  g.put("G", [6, 13], [12, 12], [16, 9], [32, 12], [36, 13], [43, 14], [49, 12], [52, 13]);
  g.put("*", [12, 6], [33, 5], [50, 6]);
  return g.rows();
}

/** The Lantern Gate: dark; a lift over spikes, the jointed gate, the long climb. */
function level3(): string[] {
  const g = new Grid(60, 22);
  g.fill(1, 19, 58, 20);
  g.fill(9, 19, 21, 19, ".").fill(9, 20, 21, 20, "^"); // the spike pit the lift crosses
  g.fill(32, 1, 40, 15); // over the gate's corridor; its right face is the climb's left wall
  g.fill(44, 8, 58, 20); // the top shelf: the climb's right wall
  g.put("M", [10, 17]).put("m", [19, 17]);
  g.put("P", [2, 18]).put("L", [6, 18], [37, 18]).put("C", [24, 18]).put("j", [30, 18]).put("J", [33, 18]);
  g.put("T", [50, 7]).put("E", [55, 7]);
  g.put("G", [4, 15], [14, 12], [17, 12], [27, 14], [42, 11], [43, 4], [47, 5], [53, 4]);
  g.put("*", [11, 5], [25, 13], [42, 16], [50, 3]);
  return g.rows();
}

export const LEVELS: Level[] = [
  { id: "level-1", name: "level.1", ambient: [0.62, 0.56, 0.5], par: 45, rows: level1() },
  { id: "level-2", name: "level.2", ambient: [0.42, 0.4, 0.42], par: 70, rows: level2() },
  { id: "level-3", name: "level.3", ambient: [0.1, 0.1, 0.16], par: 90, rows: level3() },
];

/** Whether a cell is the plate terrain. */
export function solid(c: string | undefined): boolean {
  return c === "#";
}
