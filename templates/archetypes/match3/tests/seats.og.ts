// The seats (src/seats.ts): every swap is a function of og.PlayerInput, so
// keys, a pad and a tap each play. Seat 0's keys step the hover (WASD), pick
// it (Space) and swap the selection by a direction; its pad does the same
// with the stick and A; a tap picks the cell under the cursor. With two
// seats on one device (a local versus scaffold) seat 1's arrows and Enter
// play its own board while seat 0's stays as it was; online, this device
// drives its one seat, as alone.
import { playtest } from "@origingame/engine/playtest";
import { creation } from "../src/creation.ts";
import { CASCADE_BOARD, PHASE, matchOf, tapCell } from "./lib.ts";

/** Two seats on this device. */
const players: string = creation.players;
const LOCAL = players === "local-multiplayer";
const key = (t: number, code: string) => `${t}:key_down:${code};${(t + 0.05).toFixed(2)}:key_up:${code}`;
/** The centre cell, where the hover starts. */
const CENTRE = 3 * 7 + 3;

interface SeatDump {
  phase: number;
  hover: number;
  pointing: boolean;
  selected: number;
  score: number;
  movesLeft: number;
}

export default playtest({
  name: "keys, a pad and a tap each reach og.PlayerInput and swap; seat 1 plays its own board",
  commands: [
    key(0.5, "Space"),
    `0.8:deal:${CASCADE_BOARD}`,
    "0.9:dump_entity:board:dealt.json",
    // Keys: from (3,3) left, down, down to (2,1); pick it; down swaps it with (2,0): a line.
    key(1.0, "KeyA"),
    key(1.1, "KeyS"),
    key(1.2, "KeyS"),
    "1.3:dump_entity:board:stepped.json",
    key(1.4, "Space"),
    "1.5:dump_entity:board:picked.json",
    key(1.6, "KeyS"),
    "1.7:dump_entity:board:swapping.json",
    "3.8:dump_entity:board:keys.json",
    // The pad: the stick right steps the hover, A picks it.
    "3.9:gamepad:-@0:1,0",
    "3.95:gamepad:-@0:0,0",
    "4.0:gamepad:a@0",
    "4.05:gamepad:-@0",
    "4.1:dump_entity:board:pad.json",
    // A tap: the cursor moves onto (5,0) and picks the cell under it.
    tapCell(4.2, 5, 0),
    "4.3:dump_entity:board:tap.json",
    ...(LOCAL
      ? [key(4.4, "ArrowRight"), key(4.5, "ArrowUp"), key(4.6, "Enter"), "4.7:dump_entity:board-1:seat1.json", "4.7:dump_entity:board:seat0.json"]
      : []),
    "4.8:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const at = (file: string) => matchOf(run, file) as unknown as SeatDump;
    t.equal(at("dealt.json").phase, PHASE.idle, "Space (the pick key) dealt a round");
    const stepped = at("stepped.json");
    t.equal([stepped.hover, stepped.pointing], [1 * 7 + 2, false], "A, S, S stepped the hover from (3,3) to (2,1)");
    t.equal(at("picked.json").selected, 1 * 7 + 2, "Space picked the hover");
    t.equal(at("swapping.json").phase, PHASE.swap, "S swapped the selection downwards");
    const keys = at("keys.json");
    t.equal(keys.phase, PHASE.idle, "the board settled");
    t.equal(keys.movesLeft, 19, "the key swap spent a move");
    t.ok(keys.score >= 30, `and its line scored (${keys.score})`);
    const pad = at("pad.json");
    t.equal([pad.hover, pad.selected], [1 * 7 + 3, 1 * 7 + 3], "the pad's stick stepped the hover right and A picked it");
    const tap = at("tap.json");
    t.equal([tap.selected, tap.pointing], [5, true], "a tap picked the cell under the cursor, (5,0)");
    if (LOCAL) {
      const one = at("seat1.json");
      t.equal([one.hover, one.selected], [CENTRE + 8, CENTRE + 8], "seat 1's arrows and Enter stepped and picked on its own board");
      const zero = at("seat0.json");
      t.equal([zero.hover, zero.selected, zero.movesLeft], [tap.hover, tap.selected, tap.movesLeft], "seat 0's board stayed as it was");
    }
  },
});
