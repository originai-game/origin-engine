// Every piece of game state lives in these reflected components: dumps, the
// probe, Tune and the command channel's `set` read and write them by name.
//
// Why every row carries Match, Tuning and og.PlayerInput: a system sees exactly one query
// (the AND of the components it declares), and inside a system the engine
// reaches no other entity (a game resource it declares, yes). So the systems that
// swap, clear and refill tiles must see the round's state on the same rows
// as the tiles, and the seat's input (og.PlayerInput, its `player` the row's
// seat) on the board rows. Every row is spawned with the same seven
// components (og.Transform, og.Sprite, og.Flipbook, Piece, Match, Tuning,
// og.PlayerInput) and the row whose Piece.kind is Kind.Board is the authority
// for its seat's board: the entity named "board" for seat 0, "board-1" for
// seat 1 in versus. The other rows' Match and Tuning keep their defaults and
// nothing reads them. Tuning is read from "board" alone, so both boards of a
// versus game play to the same target and move budget: Tune and
// `set:board.Tuning.<field>=<value>` write that row.

import { bool, defineComponent, defineResource, f32, f64, i32, u32 } from "@origingame/engine";

import { SIZE } from "./layout.ts";

/** What a row is. */
export const Kind = { Backdrop: 1, Board: 2, Cell: 3, Tile: 4, Cursor: 5 } as const;

/** Round phases. */
export const Phase = { Title: 0, Idle: 1, Swap: 2, SwapBack: 3, Clear: 4, Fall: 5, Won: 6, Lost: 7 } as const;
export const PHASE_NAMES = ["title", "idle", "swap", "swapBack", "clear", "fall", "won", "lost"] as const;

/** One row: a tile (its cell and colour), or the board, a cell, the cursor or the backdrop. */
export const Piece = defineComponent("Piece", {
  kind: u32(0, { doc: "Kind.* of src/components.ts." }),
  seat: u32(0, { doc: "The seat whose board the row belongs to: 0 alone, 0 or 1 in versus." }),
  col: i32(0, { min: 0, max: SIZE - 1, doc: "The tile's column, 0 at the left." }),
  row: i32(0, { min: 0, max: SIZE - 1, doc: "The tile's row, 0 at the bottom." }),
  colour: i32(-1, { min: -1, max: 5, doc: "The tile's colour index (art.ts GEMS); -1 is an empty cell." }),
  marked: bool(false, { doc: "Part of a line being cleared this step." }),
});

/** og.TweenCompleted tags: the tween group a phase waits for (Match.wait). */
export const Tag = { None: 0, Swap: 1, SwapBack: 2, Clear: 3, Fall: 4, Deal: 5 } as const;

/** The round. Authoritative on the board row. Cells are indexed row * SIZE + col; -1 is none. */
export const Match = defineComponent("Match", {
  phase: u32(Phase.Title, { doc: "PHASE_NAMES[phase]." }),
  phaseTicks: u32(0, { unit: "ticks" }),
  score: u32(0),
  best: u32(0, { doc: "The best score since the game started." }),
  movesLeft: u32(0, { doc: "Moves left this round; a swap that makes no line costs none." }),
  chain: u32(0, { doc: "The cascade level of the current clear: 1 for the swap's own lines." }),
  maxChain: u32(0, { doc: "The deepest cascade this round." }),
  cleared: u32(0, { doc: "Tiles cleared this round." }),
  lastCleared: u32(0, { doc: "Tiles the last clear step removed." }),
  rounds: u32(0),
  shuffles: u32(0, { doc: "Boards dealt again because no swap made a line." }),
  selected: i32(-1, { doc: "The selected cell (tap-tap swaps)." }),
  swapA: i32(-1),
  swapB: i32(-1),
  dragFrom: i32(-1, { doc: "The cell a held press started on (drag swaps)." }),
  pressX: f32(0, { unit: "m", doc: "Where the cursor (og.PlayerInput aimX, aimY) was when the drag's press went down." }),
  pressY: f32(0, { unit: "m" }),
  hover: i32(24, { doc: "The cell keys and pads move over: pick selects it, a direction then swaps the selection that way." }),
  pointing: bool(true, { doc: "The seat last moved its cursor by pointer (pick acts on the cell under it) rather than by direction steps (pick acts on hover)." }),
  aimX: f32(0, { unit: "m", doc: "The seat's cursor last tick, to tell a pointer that moved." }),
  aimY: f32(0, { unit: "m" }),
  before: u32(0, { doc: "The seat's og.PlayerInput.buttons last tick, for press edges." }),
  stepDir: u32(0, { doc: "The direction held last tick: 0 none, 1 right, 2 left, 3 up, 4 down." }),
  stepTicks: u32(0, { unit: "ticks", doc: "How long that direction has been held (it repeats)." }),
  pxPerM: f32(60, { doc: "CSS pixels of the play surface per metre, worked out by Pick every tick from og.ui.viewport and og.Camera (not while a room runs: its peers' windows differ)." }),
  screenX0: f32(640, { doc: "Surface x of the world origin (Pick)." }),
  screenY0: f32(390, { doc: "Surface y of the world origin (Pick)." }),
  wait: u32(0, { doc: "The Tag.* (+ 8 × seat) of the tween group the phase waits for; 0 once its og.TweenCompleted arrived." }),
  font: f64(0, { doc: "The Font handle, for the floating scores Resolve spawns." }),
});

/** Tune's parameters. Authoritative on the board row. */
export const Tuning = defineComponent("Tuning", {
  moves: u32(20, { min: 1, max: 60, step: 1, tunable: true, label: "Moves", doc: "The move budget of a round." }),
  target: u32(1500, { min: 10, max: 10000, step: 50, tunable: true, label: "Target score", doc: "Reach it before the moves run out to win." }),
  colours: u32(5, { min: 3, max: 6, step: 1, tunable: true, label: "Colours", doc: "Tile colours dealt; fewer colours, more lines." }),
  tileScore: u32(10, { min: 1, max: 100, step: 1, tunable: true, label: "Points per tile", doc: "Each cleared tile scores this times the cascade level." }),
  swapTicks: u32(8, { min: 1, max: 30, step: 1, unit: "ticks", tunable: true, label: "Swap time" }),
  clearTicks: u32(12, { min: 1, max: 40, step: 1, unit: "ticks", tunable: true, label: "Clear time" }),
  fallSpeed: f32(14, { min: 2, max: 60, step: 1, unit: "cells/s", tunable: true, label: "Fall speed" }),
});

/** The HUD's font. On the board rows only; the HUD sits under the top of each peer's own view (og.Camera). */
export const Hud = defineComponent("Hud", {
  font: f64(0, { doc: "The Font handle." }),
});

/** Sound requests: bits of Sounds.sfx that the Voices system plays once each. */
export const Sound = { Select: 1, Swap: 2, Refused: 4, Clear: 8, Cascade: 16, Won: 32, Lost: 64 } as const;

/** The sounds asked for this tick (a resource: the systems that ask see no voice rows). */
export const Sounds = defineResource("Sounds", {
  sfx: u32(0, { doc: "Sound.* bits waiting to play." }),
});

/** One voice: plays its sound once when Sounds.sfx asks for its bit. */
export const Sfx = defineComponent("Sfx", {
  bit: u32(0, { doc: "Sound.* of src/components.ts." }),
  plays: u32(0, { doc: "Times Voices has started it." }),
});
