// Who drives a critter, and what they ask for this tick. Six sources: the four
// pads of og.Gamepad (left stick or d-pad to move, A to dash) and two halves
// of the keyboard (WASD and Space; the arrows and Enter), read from og.Input.
// The camera looks down the island from the south (+z), so up on the screen is
// -z in the world, right is +x.

import type { Gamepad, Input } from "@origingame/engine";
import { Source } from "./components.ts";

export interface Intent {
  /** Where it wants to go, world x and z, length 0 to 1. */
  x: number;
  z: number;
  /** The dash (or join, or ready in the lobby) went down this tick. */
  act: boolean;
  /** The leave key went down (the lobby). */
  leave: boolean;
  /** Left and right went down this tick (the lobby's colour). */
  left: boolean;
  right: boolean;
}

const DEAD = 0.22;

const KEYS = {
  [Source.KeysLeft]: { up: "KeyW", down: "KeyS", left: "KeyA", right: "KeyD", act: "Space", leave: "KeyQ" },
  [Source.KeysRight]: { up: "ArrowUp", down: "ArrowDown", left: "ArrowLeft", right: "ArrowRight", act: "Enter", leave: "Backspace" },
} as const;

/** What a source asks for this tick. */
export function intentOf(source: number, input: Input, pads: Gamepad): Intent {
  if (source >= Source.Pad0 && source <= Source.Pad3) {
    const p = source;
    if (!pads.connected(p)) return { x: 0, z: 0, act: false, leave: false, left: false, right: false };
    let x = pads.axis("leftX", p);
    let z = pads.axis("leftY", p);
    if (Math.hypot(x, z) < DEAD) x = z = 0;
    if (pads.down("left", p)) x = -1;
    if (pads.down("right", p)) x = 1;
    if (pads.down("up", p)) z = -1;
    if (pads.down("down", p)) z = 1;
    const l = Math.hypot(x, z);
    if (l > 1) {
      x /= l;
      z /= l;
    }
    return { x, z, act: pads.pressed("a", p), leave: pads.pressed("b", p), left: pads.pressed("left", p), right: pads.pressed("right", p) };
  }
  if (source === Source.KeysLeft || source === Source.KeysRight) {
    const k = KEYS[source];
    let x = (input.down(k.right) ? 1 : 0) - (input.down(k.left) ? 1 : 0);
    let z = (input.down(k.down) ? 1 : 0) - (input.down(k.up) ? 1 : 0);
    const l = Math.hypot(x, z);
    if (l > 1) {
      x /= l;
      z /= l;
    }
    return { x, z, act: input.pressed(k.act), leave: input.pressed(k.leave), left: input.pressed(k.left), right: input.pressed(k.right) };
  }
  return { x: 0, z: 0, act: false, leave: false, left: false, right: false };
}

/** The string key naming a source in the lobby ("Pad 2", "Keys: WASD + Space"). */
export function sourceKey(source: number): string {
  return source === Source.KeysLeft ? "source.keysLeft" : source === Source.KeysRight ? "source.keysRight" : "source.pad";
}
