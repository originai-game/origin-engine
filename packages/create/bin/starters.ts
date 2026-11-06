/**
 * What each creation choice starts: the genre's starter game per view, the
 * view's camera preset, the genre's controls, the UI kit it installs, the
 * engine systems it plays on and which player counts it takes (with the
 * reason where one is refused). Data next to the vocabulary in choices.ts;
 * apply-choices.ts writes it into the game, and every starter reads the
 * part it needs from its src/creation.ts (create.test.ts checks that each
 * starter's committed creation.ts is this data at its own defaults, and
 * that the systems and kit named here are the ones its source uses).
 */
import { CREATION_CHOICES } from "./creation-vocabulary.ts";
import type { CreationChoices } from "./creation-vocabulary.ts";

export type Genre = CreationChoices["genre"];
export type View = CreationChoices["view"];
export type Players = CreationChoices["players"];
export type Input = CreationChoices["input"];

/** A view's camera: an `ArtCamera` block for the game's look (docs/art.md §2.4) and the engine controller that drives og.Camera. */
export interface ViewPreset {
  readonly profile: "2d" | "3d";
  /** The look's ArtCamera fields: the runtime applies them to og.Camera and `og style check` measures frames against them. */
  readonly camera: {
    readonly projection: "perspective" | "orthographic";
    readonly fixedAngle?: boolean;
    readonly pitch?: number;
    readonly yaw?: number;
    readonly fovMin?: number;
    readonly fovMax?: number;
  };
  /** The camera controller component (og.systems.cameras) the starter puts on the local player. */
  readonly rig: "og.Follow2D" | "og.FollowCamera" | "og.FirstPersonCamera";
}

export const VIEW_PRESETS: { readonly [V in View]: ViewPreset } = {
  "side-2d": { profile: "2d", camera: { projection: "orthographic" }, rig: "og.Follow2D" },
  "top-down-2d": { profile: "2d", camera: { projection: "orthographic" }, rig: "og.Follow2D" },
  // True isometric (arctan(1/√2) below the horizon, turned 45°) on an orthographic follow camera.
  "isometric-2.5d": { profile: "3d", camera: { projection: "orthographic", fixedAngle: true, pitch: 35.264, yaw: 45 }, rig: "og.FollowCamera" },
  "third-person-3d": { profile: "3d", camera: { projection: "perspective", fovMin: 45, fovMax: 75 }, rig: "og.FollowCamera" },
  "first-person-3d": { profile: "3d", camera: { projection: "perspective", fovMin: 60, fovMax: 100 }, rig: "og.FirstPersonCamera" },
};

/**
 * One button of a starter's controls. Each local seat gets its own action
 * (`jump` for seat 0, `jump@1` for seat 1) bound to its keys, its own pad
 * (InputBinding.gamepad) and, on touch, its own on-screen button; alone,
 * seat 0 takes both seats' keys.
 */
export interface ButtonControl {
  /** The action's name and its og.PlayerInput.buttons bit (its index in `buttons`). */
  readonly name: string;
  readonly label: string;
  /** Seat 0's keys, then seat 1's. */
  readonly keys: readonly [readonly string[], readonly string[]];
  /** Pad controls (`pad.a`, `pad.rightTrigger`). */
  readonly pad: readonly string[];
  /** Also the mouse's button (`mouse.left`): seat 0 only. */
  readonly mouse?: string;
  /** No on-screen button: the play surface's own taps give it (match, idle, cards). */
  readonly surface?: boolean;
}

/** How a starter is played: what its seats' og.PlayerInput carry. */
export interface Controls {
  /** x, y: a stick (WASD, arrows, the left stick, a touch stick), a d-pad (the same, a touch d-pad) or nothing. */
  readonly move: "stick" | "dpad" | "none";
  /**
   * aimX, aimY: `look` is a turn this tick in degrees (the mouse's movement
   * while locked, the right stick, a drag on the right of a touch screen);
   * `cursor` is a point in world units (the pointer or a tap, or the stick
   * and arrows moving it); `none` leaves them 0.
   */
  readonly aim: "look" | "cursor" | "none";
  readonly buttons: readonly ButtonControl[];
}

/** A starter: a complete game under templates/archetypes/<name>, or the template `dir` names (templates/<dir>). */
export interface Starter {
  readonly dir?: string;
  readonly profile: "2d" | "3d";
  /** The views it draws, its default first. */
  readonly views: readonly View[];
  /** Player counts it plays; a refused one's reason is in `refuse`. */
  readonly players: readonly Players[];
  /** The player count its committed src/creation.ts holds (single when unset). */
  readonly defaultPlayers?: Players;
  readonly refuse?: Partial<Record<Players, string>>;
  /** Seats it plays with local or online (2 for a duel or a pair). */
  readonly seats: number;
  readonly controls: Controls;
  /** A view's own controls where they differ from `controls` (the 2D template's top-down walk has no jump). */
  readonly viewControls?: Partial<Record<View, Controls>>;
  /** An official UI kit (packages/engine/extensions/<kit>) create installs into assets/ui-kits and the starter's HUD uses. */
  readonly kit: string | null;
  /** The engine modules the game plays on (each appears in its source as `og.<module>`). */
  readonly systems: readonly string[];
}

const BOTH_PLAYERS = ["single", "local-multiplayer", "online-lockstep"] as const satisfies readonly Players[];

const k = (zero: string[], one: string[]) => [zero, one] as const;

export const STARTERS = {
  arena: {
    profile: "2d", views: ["top-down-2d"], players: BOTH_PLAYERS, defaultPlayers: "online-lockstep", seats: 2, kit: null,
    systems: ["net", "rng", "render", "audio", "camera"],
    controls: { move: "stick", aim: "none", buttons: [{ name: "dash", label: "Dash", keys: k(["Space", "ShiftLeft"], ["Enter", "ShiftRight"]), pad: ["pad.a"] }] },
  },
  // The 2D profile's template (templates/game-2d, the platform's 2D overlay): the courier's rooftops side-on.
  platformer: {
    dir: "game-2d", profile: "2d", views: ["side-2d"], players: BOTH_PLAYERS, seats: 2, kit: null,
    systems: ["net", "tilemap", "physics", "camera", "tween", "render", "audio", "i18n", "platform", "ui"],
    controls: { move: "dpad", aim: "none", buttons: [
      { name: "jump", label: "Jump", keys: k(["Space", "KeyW"], ["Enter", "ArrowUp"]), pad: ["pad.a"] },
    ] },
  },
  runner: {
    profile: "2d", views: ["side-2d"], players: BOTH_PLAYERS, seats: 2, kit: null,
    systems: ["net", "tilemap", "tween", "camera", "render", "audio", "platform"],
    controls: { move: "none", aim: "none", buttons: [{ name: "jump", label: "Jump", keys: k(["Space", "KeyW"], ["Enter", "ArrowUp"]), pad: ["pad.a"], mouse: "mouse.left" }] },
  },
  topdown: {
    profile: "2d", views: ["top-down-2d"], players: BOTH_PLAYERS, seats: 2, kit: null,
    systems: ["net", "tilemap", "physics", "ai", "nav", "camera", "tween", "time", "render", "audio"],
    controls: { move: "stick", aim: "none", buttons: [
      { name: "attack", label: "Attack", keys: k(["Space", "KeyJ"], ["Enter", "Slash"]), pad: ["pad.a"] },
      { name: "dash", label: "Dash", keys: k(["ShiftLeft", "KeyK"], ["ShiftRight", "Period"]), pad: ["pad.b"] },
    ] },
  },
  match3: {
    profile: "2d", views: ["top-down-2d", "side-2d"], players: BOTH_PLAYERS, seats: 2, kit: "puzzle",
    systems: ["net", "tween", "render", "audio", "ui"],
    controls: { move: "none", aim: "cursor", buttons: [{ name: "pick", label: "Pick", keys: k(["Space"], ["Enter"]), pad: ["pad.a"], mouse: "mouse.left", surface: true }] },
  },
  "tower-defense": {
    profile: "2d", views: ["top-down-2d"], players: BOTH_PLAYERS, seats: 2, kit: "strategy",
    systems: ["net", "nav", "tilemap", "tween", "render", "audio", "platform", "i18n", "ui"],
    controls: { move: "none", aim: "cursor", buttons: [
      { name: "place", label: "Plant", keys: k(["Space"], ["Enter"]), pad: ["pad.a"] },
      { name: "tap", label: "Plant here", keys: k([], []), pad: [], mouse: "mouse.left", surface: true },
      { name: "cycle", label: "Next plant", keys: k(["KeyQ", "Tab"], ["Period"]), pad: ["pad.x", "pad.rightBumper"] },
      { name: "thorn", label: "Thorn", keys: k(["Digit1"], ["Numpad1"]), pad: [] },
      { name: "frost", label: "Frost lily", keys: k(["Digit2"], ["Numpad2"]), pad: [] },
      { name: "hedge", label: "Hedge", keys: k(["Digit3"], ["Numpad3"]), pad: [] },
      { name: "next", label: "Next wave", keys: k(["KeyN"], ["Slash"]), pad: ["pad.y"] },
      { name: "menu", label: "Pause", keys: k(["KeyP"], []), pad: [] },
      { name: "start", label: "Play", keys: k([], []), pad: [] },
      { name: "title", label: "To the title", keys: k([], []), pad: [] },
      { name: "settings", label: "Settings", keys: k([], []), pad: [] },
      { name: "resume", label: "Resume", keys: k([], []), pad: [] },
    ] },
  },
  roguelike: {
    profile: "2d", views: ["top-down-2d"], players: BOTH_PLAYERS, seats: 2, kit: null,
    systems: ["net", "tilemap", "procgen", "rng", "tween", "render", "audio", "platform", "i18n", "ui"],
    controls: { move: "dpad", aim: "none", buttons: [{ name: "wait", label: "Wait", keys: k(["Space", "Period"], ["Enter", "Numpad0"]), pad: ["pad.a"] }] },
  },
  deckbuilder: {
    profile: "2d", views: ["side-2d", "top-down-2d"], players: BOTH_PLAYERS, seats: 2, kit: "rpg",
    systems: ["net", "rng", "tween", "render", "audio", "platform", "i18n", "ui"],
    controls: { move: "dpad", aim: "none", buttons: [
      { name: "play", label: "Play card", keys: k(["Space", "KeyE"], ["Enter", "Slash"]), pad: ["pad.a"], surface: true },
      { name: "end", label: "End turn", keys: k(["KeyR"], ["Backspace"]), pad: ["pad.y"] },
    ] },
  },
  idle: {
    profile: "2d", views: ["side-2d", "top-down-2d"], players: ["single"], seats: 1, kit: null,
    refuse: {
      "local-multiplayer": "an idle game is one player's numbers growing while nobody plays (src/systems.ts catches up from the wall clock between sessions): a second seat would have nothing to do while it grows, and there is no turn to share",
      "online-lockstep": "lockstep runs only while every seat sends input each tick, and an idle game's progress is made while the player is away (the offline catch-up from the save's wall-clock stamp), which no room can run",
    },
    systems: ["rng", "tween", "render", "audio", "platform", "i18n", "ui"],
    controls: { move: "none", aim: "cursor", buttons: [
      { name: "tap", label: "Tap", keys: k(["Space", "Enter"], []), pad: ["pad.a"], surface: true },
      { name: "buy", label: "Buy", keys: k(["KeyB"], []), pad: ["pad.x"], surface: true },
      { name: "prev", label: "Previous", keys: k(["ArrowLeft"], []), pad: ["pad.left"], surface: true },
      { name: "next", label: "Next", keys: k(["ArrowRight"], []), pad: ["pad.right"], surface: true },
      { name: "buy1", label: "Buy Ember", keys: k(["Digit1"], []), pad: [], surface: true },
      { name: "buy2", label: "Buy Storm", keys: k(["Digit2"], []), pad: [], surface: true },
      { name: "buy3", label: "Buy Crystal", keys: k(["Digit3"], []), pad: [], surface: true },
      { name: "buy4", label: "Buy Star", keys: k(["Digit4"], []), pad: [], surface: true },
      { name: "nova", label: "Supernova", keys: k(["KeyN"], []), pad: ["pad.y"], surface: true },
      { name: "menu", label: "Pause", keys: k(["KeyP"], []), pad: [], surface: true },
      { name: "point", label: "Tap the orb", keys: k([], []), pad: [], mouse: "mouse.left", surface: true },
    ] },
  },
  rhythm: {
    profile: "2d", views: ["top-down-2d", "side-2d"], players: BOTH_PLAYERS, seats: 2, kit: null,
    systems: ["net", "audio", "tween", "render", "platform", "i18n", "ui"],
    controls: { move: "none", aim: "none", buttons: [
      { name: "lane0", label: "Left lane", keys: k(["KeyA", "ArrowLeft"], ["KeyJ"]), pad: ["pad.left", "pad.x"] },
      { name: "lane1", label: "Middle lane", keys: k(["KeyS", "ArrowDown"], ["KeyK"]), pad: ["pad.down", "pad.a"] },
      { name: "lane2", label: "Right lane", keys: k(["KeyD", "ArrowRight"], ["KeyL"]), pad: ["pad.right", "pad.b"] },
    ] },
  },
  "third-person": {
    profile: "3d", views: ["third-person-3d"], players: BOTH_PLAYERS, seats: 2, kit: null,
    systems: ["net", "physics", "terrain", "nav", "ai", "anim", "camera", "atmosphere", "render", "audio", "platform", "i18n", "ui"],
    controls: { move: "stick", aim: "look", buttons: [
      { name: "jump", label: "Jump", keys: k(["Space"], ["Enter"]), pad: ["pad.a"] },
      { name: "run", label: "Run", keys: k(["ShiftLeft"], ["ShiftRight"]), pad: ["pad.leftStick", "pad.rightTrigger"] },
      { name: "kindle", label: "Kindle", keys: k(["KeyF"], ["Slash"]), pad: ["pad.x"] },
    ] },
  },
  shooter: {
    profile: "3d", views: ["first-person-3d", "third-person-3d", "isometric-2.5d"], players: BOTH_PLAYERS, seats: 2, kit: "shooter",
    systems: ["net", "physics", "camera", "render", "audio", "tween", "platform", "i18n", "ui"],
    controls: { move: "stick", aim: "look", buttons: [
      { name: "fire", label: "Fire", keys: k(["KeyF"], ["Slash"]), pad: ["pad.rightTrigger"], mouse: "mouse.left" },
      { name: "jump", label: "Jump", keys: k(["Space"], ["Enter"]), pad: ["pad.a"] },
      { name: "reload", label: "Reload", keys: k(["KeyR"], ["Period"]), pad: ["pad.x"] },
    ] },
  },
  rpg: {
    profile: "3d", views: ["isometric-2.5d", "third-person-3d", "first-person-3d"], players: BOTH_PLAYERS, seats: 2, kit: "rpg",
    systems: ["net", "physics", "nav", "camera", "render", "audio", "tween", "platform", "i18n", "ui"],
    controls: { move: "stick", aim: "look", buttons: [
      { name: "attack", label: "Attack", keys: k(["Space", "KeyJ"], ["Enter", "Slash"]), pad: ["pad.a"], mouse: "mouse.left" },
      { name: "talk", label: "Talk", keys: k(["KeyE"], ["Period"]), pad: ["pad.x"] },
      { name: "potion", label: "Potion", keys: k(["KeyQ"], ["Comma"]), pad: ["pad.y"] },
    ] },
  },
  racing: {
    profile: "3d", views: ["third-person-3d", "first-person-3d", "isometric-2.5d"], players: BOTH_PLAYERS, seats: 2, kit: "racing",
    systems: ["net", "camera", "render", "audio", "tween", "platform", "i18n", "ui"],
    controls: { move: "stick", aim: "none", buttons: [
      { name: "throttle", label: "Throttle", keys: k(["KeyW"], ["ArrowUp"]), pad: ["pad.rightTrigger", "pad.a"] },
      { name: "brake", label: "Brake", keys: k(["KeyS"], ["ArrowDown"]), pad: ["pad.leftTrigger", "pad.b"] },
      { name: "boost", label: "Boost", keys: k(["Space"], ["Enter"]), pad: ["pad.x"] },
    ] },
  },
  "open-world": {
    profile: "3d", views: ["third-person-3d", "first-person-3d", "isometric-2.5d"], players: BOTH_PLAYERS, seats: 2, kit: "rpg",
    systems: ["net", "physics", "terrain", "atmosphere", "camera", "render", "audio", "tween", "platform", "i18n", "ui"],
    controls: { move: "stick", aim: "look", buttons: [
      { name: "jump", label: "Jump", keys: k(["Space"], ["Enter"]), pad: ["pad.a"] },
      { name: "run", label: "Run", keys: k(["ShiftLeft"], ["ShiftRight"]), pad: ["pad.leftStick", "pad.rightTrigger"] },
      { name: "gather", label: "Gather", keys: k(["KeyE"], ["Slash"]), pad: ["pad.x"] },
    ] },
  },
} as const satisfies Record<string, Starter>;

export type StarterName = keyof typeof STARTERS;

/** A genre: its starter per view (its default view first) and, where a view has no starter, why. */
export interface GenreStart {
  readonly starters: Partial<Record<View, StarterName>>;
  readonly refuseView?: Partial<Record<View, string>>;
}

export const GENRES: { readonly [G in Genre]: GenreStart } = {
  "action-platformer": {
    starters: { "side-2d": "platformer", "third-person-3d": "third-person", "top-down-2d": "topdown" },
    refuseView: {
      "isometric-2.5d": "jumps read by height, which a fixed isometric angle hides (a jump and a step toward the camera look the same): choose side-2d, top-down-2d or third-person-3d",
      "first-person-3d": "platforming needs the feet and the edge in view, which a first-person camera hides: choose side-2d or third-person-3d",
    },
  },
  shooter: {
    starters: { "first-person-3d": "shooter", "third-person-3d": "shooter", "isometric-2.5d": "shooter" },
    refuseView: {
      "side-2d": "the shooter starter aims in 3D (yaw and pitch); a side-on run-and-gun starts from the platformer (genre action-platformer)",
      "top-down-2d": "the shooter starter is 3D: a top-down twin-stick plays in it at isometric-2.5d",
    },
  },
  rpg: {
    starters: { "isometric-2.5d": "rpg", "third-person-3d": "rpg", "first-person-3d": "rpg" },
    refuseView: {
      "side-2d": "the rpg starter is a 3D keep (navmesh foes, a party moving in depth): choose isometric-2.5d, third-person-3d or first-person-3d",
      "top-down-2d": "the rpg starter is 3D: its top-down reading is isometric-2.5d (a grid crawl in 2D is the roguelike, genre roguelike-card at top-down-2d)",
    },
  },
  "puzzle-match": {
    starters: { "top-down-2d": "match3", "side-2d": "match3" },
    refuseView: {
      "isometric-2.5d": "a match board is read whole and flat; a 2.5D angle foreshortens the far rows: choose top-down-2d",
      "third-person-3d": "a match board has no avatar to follow: choose top-down-2d",
      "first-person-3d": "a match board has no avatar to look through: choose top-down-2d",
    },
  },
  "strategy-tower-defense": {
    starters: { "top-down-2d": "tower-defense" },
    refuseView: {
      "side-2d": "towers cover the lawn in both directions, which a side view flattens: choose top-down-2d",
      "isometric-2.5d": "the tower-defense starter's lawn is a 2D grid with 2D paths: choose top-down-2d",
      "third-person-3d": "tower defence is played from above the whole field, not behind one unit: choose top-down-2d",
      "first-person-3d": "tower defence is played from above the whole field, not through one unit's eyes: choose top-down-2d",
    },
  },
  racing: {
    starters: { "third-person-3d": "racing", "first-person-3d": "racing", "isometric-2.5d": "racing" },
    refuseView: {
      "side-2d": "the racing starter drives a 3D circuit; a side-on race is the runner (genre runner)",
      "top-down-2d": "the racing starter is 3D: its top-down reading is isometric-2.5d",
    },
  },
  runner: {
    starters: { "side-2d": "runner" },
    refuseView: {
      "top-down-2d": "the runner reads the next obstacle by its height on a side-on line: choose side-2d",
      "isometric-2.5d": "the runner is side-on: choose side-2d",
      "third-person-3d": "the runner is a 2D side-scroller: a 3D lane runner is not a starter yet, choose side-2d",
      "first-person-3d": "the runner is a 2D side-scroller: choose side-2d",
    },
  },
  "roguelike-card": {
    starters: { "side-2d": "deckbuilder", "top-down-2d": "roguelike" },
    refuseView: {
      "isometric-2.5d": "the card and dungeon starters are 2D: choose side-2d (cards) or top-down-2d (a dungeon crawl)",
      "third-person-3d": "the card and dungeon starters are 2D: choose side-2d (cards) or top-down-2d (a dungeon crawl)",
      "first-person-3d": "the card and dungeon starters are 2D: choose side-2d (cards) or top-down-2d (a dungeon crawl)",
    },
  },
  idle: {
    starters: { "side-2d": "idle", "top-down-2d": "idle" },
    refuseView: {
      "isometric-2.5d": "the idle starter is a 2D orb and its cards: choose side-2d",
      "third-person-3d": "an idle game has no avatar to follow: choose side-2d",
      "first-person-3d": "an idle game has no avatar to look through: choose side-2d",
    },
  },
  rhythm: {
    starters: { "top-down-2d": "rhythm", "side-2d": "rhythm" },
    refuseView: {
      "isometric-2.5d": "notes are judged where they cross the line, which a 2.5D angle foreshortens: choose top-down-2d",
      "third-person-3d": "the rhythm starter's lanes are 2D: choose top-down-2d",
      "first-person-3d": "the rhythm starter's lanes are 2D: choose top-down-2d",
    },
  },
  arena: {
    starters: { "top-down-2d": "arena", "third-person-3d": "shooter", "first-person-3d": "shooter", "isometric-2.5d": "shooter" },
    refuseView: {
      "side-2d": "the arena is a top-down floor both players cross; a side-on duel starts from the platformer (genre action-platformer, players local-multiplayer)",
    },
  },
  "open-world": {
    starters: { "third-person-3d": "open-world", "first-person-3d": "open-world", "isometric-2.5d": "open-world" },
    refuseView: {
      "side-2d": "the open-world starter is a 3D island on a terrain: choose third-person-3d, first-person-3d or isometric-2.5d",
      "top-down-2d": "the open-world starter is a 3D island on a terrain: its overhead reading is isometric-2.5d",
    },
  },
};

/** The profile a view is made in: the platform's overlay for it (2d: templates/game-2d, 3d: templates/game). */
export function viewProfile(view: View): "2d" | "3d" {
  return VIEW_PRESETS[view].profile;
}

/** The genre a view alone asks for: the one whose starter draws it best. */
export const VIEW_GENRE: Record<View, Genre> = {
  "side-2d": "action-platformer",
  "top-down-2d": "action-platformer",
  "isometric-2.5d": "rpg",
  "third-person-3d": "action-platformer",
  "first-person-3d": "shooter",
};

/**
 * The player counts each genre plays, and why it refuses the others: a
 * count any of its starters plays stands, so a refusal is the genre's own
 * (an idle game's progress is made while nobody plays).
 */
export const GENRE_PLAYERS: { readonly [G in Genre]: { readonly players: readonly Players[]; readonly refuse: Partial<Record<Players, string>> } } = Object.fromEntries(
  CREATION_CHOICES.genre.map((g) => {
    const starters = [...new Set(Object.values(GENRES[g].starters))].map((n) => STARTERS[n] as Starter);
    const players = CREATION_CHOICES.players.filter((p) => starters.some((s) => s.players.includes(p)));
    const refuse = Object.fromEntries(CREATION_CHOICES.players.filter((p) => !players.includes(p)).map((p) => [p, starters.map((s) => s.refuse?.[p]).find(Boolean) ?? "not supported"]));
    return [g, { players, refuse }];
  }),
) as never;

/** Why a genre refuses a player count, or null when it plays it. */
export function genrePlayersRefused(genre: Genre, players: Players): string | null {
  const rule = GENRE_PLAYERS[genre];
  return rule.players.includes(players) ? null : `${genre} with ${players}: ${rule.refuse[players]}`;
}

/** The genre's default view: its first starter's. */
export function defaultView(genre: Genre): View {
  return Object.keys(GENRES[genre].starters)[0] as View;
}

/** A genre's view choice with its starter, or why the pair has none. */
export function starterFor(genre: Genre, view: View): { starter: StarterName } | { refused: string } {
  const starter = GENRES[genre].starters[view];
  if (starter) return { starter };
  return { refused: `${genre} at ${view}: ${GENRES[genre].refuseView?.[view] ?? "no starter"}` };
}

/** Why a starter refuses a player count, or null when it plays it. */
export function playersRefused(starter: StarterName, players: Players): string | null {
  const s: Starter = STARTERS[starter];
  if (s.players.includes(players)) return null;
  return `${starter} with ${players}: ${s.refuse?.[players] ?? "not supported"}`;
}

/** The selection the starter's src/creation.ts carries: the choices, its seats, controls and camera. */
export function creationData(starter: StarterName, choices: CreationChoices) {
  const s: Starter = STARTERS[starter];
  const preset = VIEW_PRESETS[choices.view];
  return {
    ...choices,
    starter,
    seats: choices.players === "single" ? 1 : s.seats,
    controls: s.viewControls?.[choices.view] ?? s.controls,
    camera: preset.camera,
    rig: preset.rig,
    kit: s.kit,
  };
}

/** The defaults a starter's committed src/creation.ts holds: its default view, alone, at the desk. */
export function starterDefaults(starter: StarterName): CreationChoices {
  const s: Starter = STARTERS[starter];
  // A profile template no genre names (the 3D template) starts as the genre its default view asks for.
  const genre = (Object.keys(GENRES) as Genre[]).find((g) => Object.values(GENRES[g].starters).includes(starter)) ?? VIEW_GENRE[s.views[0]!];
  return {
    genre, view: s.views[0]!, look: s.profile === "2d" ? "16bit-pixel" : "faceted-low-poly", mood: "bright", ui: s.profile === "2d" ? "pixel" : "storybook",
    platform: "desktop", input: "keyboard-mouse", players: s.defaultPlayers ?? "single", lang: ["en"], music: "ambient",
  };
}

/** The file every starter reads its selection from. */
export function creationModule(starter: StarterName, choices: CreationChoices): string {
  return `// The creation choices this game was made with (project.ogt's create block),
// with what they start: the seats, the controls and the camera. Written by
// og new from packages/create/bin/starters.ts; src/seats.ts and the game
// read it. Edit the create block and run og new's choices again, or edit
// here by hand: nothing else reads the block at run time.
export const creation = ${JSON.stringify(creationData(starter, choices), null, 2)} as const;
`;
}

/** Every view each genre takes, for the platform's form (a view a genre refuses is left out). */
export function genreViews(): Record<Genre, View[]> {
  return Object.fromEntries(CREATION_CHOICES.genre.map((g) => [g, Object.keys(GENRES[g].starters) as View[]])) as Record<Genre, View[]>;
}
