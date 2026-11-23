// The motion facts every built-in fixture is held to (docs/ui.md §6.1),
// read from one laid-out frame (fixture-frames.ts): a control a finger
// presses that does not move under the press, and a screen or a modal that
// appears or leaves with no motion.

import type { FrameElement, Host } from "./fixture-frames.ts";
import type { Finding } from "./fixture-layout.ts";

/** Roles a player presses (a slider, a field or a spin button is dragged or typed in, not pressed). */
const PRESSED = new Set(["button", "link", "checkbox", "radio", "switch", "combobox", "option", "tab", "menuitem", "menuitemcheckbox"]);

const name = (e: FrameElement) => `${e.role || "view"} "${(e.name || e.text).slice(0, 28)}"`;

/**
 * Every enabled control a finger presses squashes under the press (the
 * tree gives it the pack's `Press.feedback` unless its own classes move
 * it). A press catcher over a third of the surface (a screen's "tap to
 * start") and a control inside another one are not controls of their own.
 */
export function pressless(els: FrameElement[], host: Host): Finding[] {
  const byId = new Map(els.map((e) => [e.id, e]));
  const inControl = (e: FrameElement) => { for (let a = byId.get(e.parent); a; a = byId.get(a.parent)) if (PRESSED.has(a.role)) return true; return false; };
  return els.flatMap((e) =>
    PRESSED.has(e.role) && !e.disabled && e.width * e.height * 3 <= host.width * host.height && !inControl(e) && !e.press
      ? [{ key: `no press feedback ${name(e)}`, detail: `${Math.round(e.width)}x${Math.round(e.height)} does not move under a press` }]
      : []);
}

/**
 * A screen (a screen page's root that covers the surface in a game's
 * column: `screen` is the gallery's place for it) and a modal enter from a
 * `starting:` record and leave to an `exit:` one: neither appears or goes
 * in one frame.
 */
export function presenceless(els: FrameElement[], host: Host, screen: boolean): Finding[] {
  const root = els.find((e) => e.parent === "");
  const holder = host.column ? els.find((e) => e.parent === root?.id) : root;
  const screens = screen ? els.filter((e) => e.parent === holder?.id && e.x <= 0.5 && e.y <= 0.5 && e.width >= host.width - 1 && e.height >= host.height - 1) : [];
  return [...new Set([...screens, ...els.filter((e) => e.modal)])].flatMap((e) => {
    const missing = (["enter", "exit"] as const).filter((k) => !e.presence?.includes(k));
    return missing.length ? [{ key: `no ${missing.join(" or ")} motion ${e.modal ? "modal" : "screen"} ${name(e)}`, detail: `presence ${JSON.stringify(e.presence ?? [])}` }] : [];
  });
}
