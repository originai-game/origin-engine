// The seats: who plays, on what. The same file in every starter (og new
// copies it; packages/create's test keeps the copies equal); what differs
// is src/creation.ts, the creation choices with the controls they start.
//
// Gameplay reads og.PlayerInput alone, one entity per seat with `player`
// its seat (docs/design.md §2.10). This file is the only reader of the
// keys, the mouse, the pads and the touch controls: its Seats system gives
// each seat this device drives to og.net.submit once a tick, so the same
// systems play
//   alone             seat 0, from every device;
//   local-multiplayer seats 0 and 1 on one device: WASD and the left keys
//                     against the arrows and the right keys, pad 1 against
//                     pad 2, or two sets of touch controls;
//   online-lockstep   this peer's seat (og.Lockstep.localPlayer) over a
//                     room, every peer running the whole world.
// A menu or HUD button that changes the world (Pause, a card, a plant)
// calls press(name) rather than writing the world, and the game acts on
// the button's bit in og.PlayerInput: in a room every peer then sees it on
// the same tick.
//
// og.PlayerInput's fields here: x, y the move (right, up: -1 to 1); aimX,
// aimY by creation.controls.aim: `look` a turn this tick in degrees (right,
// up), `cursor` a world point; buttons one bit per control in
// creation.controls.buttons order, and PAUSE.

import { og, defineSystem } from "@origingame/engine";
import type { InputBinding, World } from "@origingame/engine";
import { creation } from "./creation.ts";

interface Button {
  readonly name: string;
  readonly label: string;
  readonly keys: readonly (readonly string[])[];
  readonly pad: readonly string[];
  readonly mouse?: string;
  readonly surface?: boolean;
}
interface Controls {
  readonly move: "stick" | "dpad" | "none";
  readonly aim: "look" | "cursor" | "none";
  readonly buttons: readonly Button[];
}
const controls = creation.controls as Controls;

const players: string = creation.players;
const input: string = creation.input;
export const LOCAL = players === "local-multiplayer";
export const ONLINE = players === "online-lockstep";
/** The seats in play: 1 alone, the starter's seats with others. */
export const SEATS: number = creation.seats;
/** og.PlayerInput.buttons: the pause toggle, past every control's bit. */
export const PAUSE = 1 << 15;

/** Whether the touch controls show: the player chose touch, or the pointer is a finger. */
export function usesTouch(): boolean {
  return input === "touch" || og.ui.frameViewport().coarsePointer;
}

/** A control's bit in og.PlayerInput.buttons. */
export function bit(name: string): number {
  if (name === "pause") return PAUSE;
  const i = controls.buttons.findIndex((b) => b.name === name);
  if (i < 0) throw new Error(`no control '${name}' in src/creation.ts (${controls.buttons.map((b) => b.name).join(", ")})`);
  return 1 << i;
}

/** Whether a seat's input holds a control this tick. */
export function held(buttons: number, name: string): boolean {
  return (buttons & bit(name)) !== 0;
}

/** Whether a control went down this tick: its bit now and not in `before` (the seat's buttons last tick, kept by the game). */
export function pressedNow(buttons: number, before: number, name: string): boolean {
  const b = bit(name);
  return (buttons & b) !== 0 && (before & b) === 0;
}

/** Bits the HUD and menus pressed this tick, per local seat (press). */
const intents = [0, 0];
/** Holds a control's bit for this tick's input of a seat: a HUD or menu button that acts on the world. */
export function press(name: string, seat = 0): void {
  intents[seat] = (intents[seat] ?? 0) | bit(name);
}

/** A scripted player for playtests and room runs (`bot:<n>`): this peer's seats take input from the tick. */
let bot = 0;
export function setBot(n: number): void {
  if (!Number.isInteger(n) || n < 0) throw new Error("write bot:<n> with a whole number (0 hands the input back)");
  bot = n;
}

/** The seats this device drives now: [0] alone, [0, 1] local, [localPlayer] in a running room. */
export function localSeats(world: World): number[] {
  if (LOCAL) return [0, 1];
  if (ONLINE) {
    const lock = world.resource(og.Lockstep);
    if (lock.ok && lock.value.running) return lock.value.localPlayer < SEATS ? [lock.value.localPlayer] : [];
  }
  return [0];
}

const name = (control: string, seat: number) => `${control}@${seat}`;

/** Call once in start: lockstep for an online game, and every local seat's actions. */
export function setupSeats(world: World): void {
  const lock = world.resource(og.Lockstep);
  if (lock.ok) {
    lock.value.enabled = ONLINE;
    lock.value.players = Math.max(2, SEATS);
    lock.value.inputDelay = 2;
    lock.value.hashEvery = 10;
  }
  for (let seat = 0; seat < (LOCAL ? 2 : 1); seat++) {
    const pad = LOCAL ? seat + 1 : 0;
    // Alone, seat 0 takes both seats' keys.
    const keys = (b: Button) => (LOCAL ? b.keys[seat] ?? [] : [...(b.keys[0] ?? []), ...(b.keys[1] ?? [])]);
    const move: InputBinding[] = [];
    if (!LOCAL || seat === 0) move.push({ up: "KeyW", down: "KeyS", left: "KeyA", right: "KeyD" });
    if (!LOCAL || seat === 1) move.push({ up: "ArrowUp", down: "ArrowDown", left: "ArrowLeft", right: "ArrowRight" });
    move.push({ control: "pad.leftXY", gamepad: pad }, { control: "pad.dpad", gamepad: pad });
    must(og.input.defineAction(world, { name: name("move", seat), kind: "axis2", label: "Move", bindings: move }));
    if (controls.aim === "look") {
      // The mouse is added in Seats while the pointer is locked: unlocked, a hover would turn the view.
      must(og.input.defineAction(world, { name: name("look", seat), kind: "axis2", label: "Look", bindings: [{ control: "pad.rightXY", gamepad: pad, scale: STICK_LOOK }] }));
    }
    for (const b of controls.buttons) {
      const bindings: InputBinding[] = keys(b).map((control) => ({ control }));
      for (const control of b.pad) bindings.push({ control, gamepad: pad });
      if (b.mouse && seat === 0) bindings.push({ control: b.mouse });
      must(og.input.defineAction(world, { name: name(b.name, seat), label: b.label, bindings }));
    }
    must(og.input.defineAction(world, { name: name("pause", seat), label: "Pause", locked: true, bindings: [...(seat === 0 ? [{ control: "Escape" }] : []), { control: "pad.start", gamepad: pad }] }));
  }
}

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const clamp = (v: number) => (v < -1 ? -1 : v > 1 ? 1 : v);

/** The pointer's world point under a 2D (orthographic) camera. */
function cursor(world: World): { x: number; y: number } | null {
  const vp = og.ui.viewport(world);
  const input = world.resource(og.Input);
  const cam = world.resource(og.Camera);
  if (!input.ok || !cam.ok || vp.width <= 0 || vp.height <= 0) return null;
  const k = cam.value.height / vp.height;
  return { x: cam.value.position.x + (input.value.pointer.x - vp.width / 2) * k, y: cam.value.position.y - (input.value.pointer.y - vp.height / 2) * k };
}

/** Where each seat's cursor is (aim `cursor`), and how fast a stick moves it (world units a second). */
const cursors = [{ x: 0, y: 0 }, { x: 0, y: 0 }];
const CURSOR_SPEED = 8;
/** Mouse-look: degrees per CSS pixel while the pointer is locked; the right stick's degrees a tick at full tilt. */
const MOUSE_LOOK = 0.12;
const STICK_LOOK = 150 / 60;

/** What a seat's touch controls hold this tick. */
export interface TouchInput {
  x: number;
  y: number;
  /** The look's turn this tick, degrees (aim `look`). */
  lx: number;
  ly: number;
  buttons: number;
}

/** A game's own touch HUD for a seat, drawn in place of the default controls. */
let ownTouch: ((seat: number) => TouchInput) | null = null;
/**
 * Draw the game's own touch controls instead of the default ones (a
 * game whose phone layout is part of its look): called by Seats each tick
 * for each local seat while touch shows; return what they hold, using
 * bit(name) for the buttons.
 */
export function useTouchControls(draw: (seat: number) => TouchInput): void {
  ownTouch = draw;
}

/**
 * The touch controls of a seat, and what they hold. Alone: the move bottom
 * left, the look and buttons bottom right. Local: each seat one column in
 * its own bottom corner (seat 0 left, seat 1 right), buttons over the move.
 */
function touch(seat: number): TouchInput {
  if (ownTouch) return ownTouch(seat);
  const out = { x: 0, y: 0, lx: 0, ly: 0, buttons: 0 };
  const size = LOCAL ? 112 : 132;
  const move = () => {
    if (controls.move === "dpad") {
      const d = og.ui.dpad("Move", { size });
      out.x = Number(d.right) - Number(d.left);
      out.y = Number(d.up) - Number(d.down);
    } else if (controls.move === "stick") {
      const s = og.ui.joystick("Move", { size });
      out.x = s.x;
      out.y = -s.y;
    }
  };
  const pressable = controls.buttons.filter((b) => !b.surface);
  const actions = () => {
    if (controls.aim === "look") {
      const s = og.ui.joystick("Look", { size: LOCAL ? 96 : 112 });
      out.lx = s.x * 2.5;
      out.ly = -s.y * 2.5;
    }
    pressable.forEach((b, i) => {
      if (og.ui.actionButton(b.name, b.label, { size: i === 0 ? (LOCAL ? 64 : 76) : LOCAL ? 52 : 60, primary: i === 0 }).pressed) out.buttons |= bit(b.name);
    });
  };
  const some = pressable.length > 0 || controls.aim === "look";
  if (LOCAL) {
    og.ui.anchor(`TouchMove${seat}`, { anchor: seat === 0 ? "bottomLeft" : "bottomRight", direction: "column", gap: 12, cross: "center" }, () => {
      if (some) og.ui.row("Actions", { gap: 8 }, actions);
      move();
    });
    return out;
  }
  if (controls.move !== "none") og.ui.anchor(`TouchMove${seat}`, { anchor: "bottomLeft", direction: "column", gap: 12, cross: "center" }, move);
  if (some) og.ui.anchor(`TouchActions${seat}`, { anchor: "bottomRight", direction: "row", gap: 12, cross: "end" }, actions);
  return out;
}

/**
 * Gives each seat this device drives its input for the tick (og.net.submit
 * with `player`), from its actions, its touch controls, the HUD's press()
 * and, with `bot:<n>`, a script. Draws the touch controls where the player
 * chose touch or the pointer is a finger.
 */
export const Seats = defineSystem(
  "Seats",
  { reads: [og.Input, og.Pointers, og.Gamepad, og.Lockstep, og.Camera], writes: [og.PointerLock] },
  (_q, time, world) => {
    const fingers = usesTouch();
    if (controls.aim === "look" && !fingers) {
      const lock = world.resource(og.PointerLock);
      if (lock.ok) lock.value.wanted = true;
    }
    for (const seat of localSeats(world)) {
      const local = LOCAL ? seat : 0;
      let x = 0, y = 0, aimX = 0, aimY = 0, buttons = intents[local] ?? 0;
      intents[local] = 0;
      if (bot > 0) {
        // By the tick and seat alone, so every session gets the same inputs.
        const t = time.tick + 37 * (bot + seat);
        x = t % 240 < 150 ? 1 : -0.6;
        y = (t >> 5) % 3 === 0 ? 0.5 : 0;
        aimX = controls.aim === "look" ? ((t >> 4) % 4 === 0 ? 2 : 0) : 0;
        controls.buttons.forEach((b, i) => {
          if ((t + 11 * i) % (40 + 13 * i) < 6) buttons |= 1 << i;
        });
      } else {
        const mv = og.input.actionVector(world, name("move", local));
        x = mv.x;
        y = mv.y;
        if (controls.aim === "look") {
          const look = og.input.actionVector(world, name("look", local));
          aimX = look.x;
          aimY = look.y;
          const lock = world.resource(og.PointerLock);
          const ptr = world.resource(og.Pointers);
          if (local === 0 && lock.ok && lock.value.locked && ptr.ok) {
            aimX += ptr.value.movement.x * MOUSE_LOOK;
            aimY -= ptr.value.movement.y * MOUSE_LOOK;
          }
        }
        controls.buttons.forEach((b, i) => {
          if (og.input.actionDown(world, name(b.name, local))) buttons |= 1 << i;
        });
        if (og.input.actionPressed(world, name("pause", local))) buttons |= PAUSE;
        if (fingers) {
          const t = touch(local);
          x = clamp(x + t.x);
          y = clamp(y + t.y);
          aimX += t.lx;
          aimY += t.ly;
          buttons |= t.buttons;
        }
        if (controls.aim === "cursor") {
          // The pointer or a tap for seat 0 while it uses one; a cursor the stick moves otherwise.
          const c = cursors[local]!;
          const device = og.input.device(world);
          const at = local === 0 && (device === "keyboardMouse" || device === "touch") ? cursor(world) : null;
          if (at) {
            c.x = at.x;
            c.y = at.y;
          } else {
            c.x += x * CURSOR_SPEED * time.dt;
            c.y += y * CURSOR_SPEED * time.dt;
          }
          aimX = c.x;
          aimY = c.y;
        }
      }
      must(og.net.submit(world, { player: seat, x: clamp(x), y: clamp(y), aimX, aimY, buttons }));
    }
  },
);
