// Every piece of game state lives in these reflected components: dumps, the
// probe, Tune and the command channel's `set` read and write them by name.
//
// Why every row carries Match and Tuning: a system sees exactly one query
// (the AND of the components it declares), and inside a system the engine
// reaches no other entity (a game resource it declares, yes). So the systems that
// swap, clear and refill tiles must see the round's state on the same rows
// as the tiles. Every row is spawned with the same six components
// (og.Transform, og.Sprite, og.Flipbook, Piece, Match, Tuning) and the row whose
// Piece.kind is Kind.Board, the entity named "board", is the authority: the
// other rows' Match and Tuning keep their defaults and nothing reads them.
// Tune and `set:board.Tuning.<field>=<value>` write that row.

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
  pressX: f32(0, { unit: "m" }),
  pressY: f32(0, { unit: "m" }),
  viewHeight: f32(12, { unit: "m", doc: "The camera's visible height, set in start." }),
  viewCentreY: f32(0.5, { unit: "m", doc: "The camera's centre height, set in start." }),
  pxPerM: f32(60, { doc: "Pixels of the play surface per metre, worked out by Pick every tick from the render target." }),
  screenX0: f32(640, { doc: "Surface x of the world origin (Pick)." }),
  screenY0: f32(390, { doc: "Surface y of the world origin (Pick)." }),
  wait: u32(0, { doc: "The Tag.* of the tween group the phase waits for; 0 once its og.TweenCompleted arrived." }),
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

/** The HUD's font and where the board sits. On the board row only. */
export const Hud = defineComponent("Hud", {
  font: f64(0, { doc: "The Font handle." }),
  top: f32(6, { unit: "m", doc: "The top of the view." }),
  halfWidth: f32(8, { unit: "m" }),
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
