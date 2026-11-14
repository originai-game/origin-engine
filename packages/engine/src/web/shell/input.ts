// The page's input to the engine (shell.ts): keys, pointers of every type
// and the wheel on the canvas, the UI's text fields through a hidden
// textarea (text-input.ts), the accessibility mirror over the canvas
// (a11y.ts), the keyboard layout's printed keys, the reader's preferences
// as they change, and the device pixel ratio as it changes.

import { A11yMirror } from "../a11y.ts";
import type { InjectedInput } from "../input.ts";
import { LayoutLearner, readLayoutMap, type KeyLabel } from "../keyboard-layout.ts";
import { TextEntry } from "../text-input.ts";
import type { Presentation } from "./presentation.ts";
import { PAGE_PREFERENCES, pagePreferences, watchPixelRatio } from "./viewport.ts";

/** The modifier keys' codes: held through other keys, released on their own keyup. */
const MODIFIER_KEYS = new Set(["ShiftLeft", "ShiftRight", "ControlLeft", "ControlRight", "AltLeft", "AltRight", "MetaLeft", "MetaRight", "CapsLock"]);

/** Keys the page would act on itself (scroll, focus); the shell keeps them for the game. */
const KEPT_KEYS = new Set(["Space", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "Tab", "PageUp", "PageDown", "Home", "End", "Backspace"]);

/** The key ups that release every key in `held` (the codes sent down and not yet up), which it empties: what the shell sends when the window loses focus. */
export function releaseHeld(held: Set<string>, time?: number): InjectedInput[] {
  const out: InjectedInput[] = [...held].map((key) => ({ kind: "key", key, action: "up", ...(time !== undefined ? { time } : {}) }));
  held.clear();
  return out;
}

const BUTTONS = ["left", "middle", "right"] as const;

/** Where the page's input goes, and what it reads of the shell. */
export interface InputSink {
  /** One event to the engine (the input ring when isolated, else a message). */
  send(event: InjectedInput): void;
  /** A message to the engine Worker. */
  post(message: Record<string, unknown>): void;
  /** The viewport last sent to the Worker (the page's preferences are re-read into it when they change). */
  viewport(): Record<string, unknown>;
  /** Whether the engine stopped answering. */
  stopped(): boolean;
  /** The device pixel ratio changed: the canvas and the UI's viewport again, before the next frame. */
  resized(): void;
  /** The pointer lock and fullscreen, which a press or a key may ask for. */
  readonly presentation: Presentation;
  /** What removes each listener wired here (Shell.dispose). */
  readonly listeners: Array<() => void>;
}

/** Wires the page's input on `canvas` to `sink`; answers the text entry and the accessibility mirror it made. */
export function wireInput(canvas: HTMLCanvasElement, sink: InputSink): { text: TextEntry; mirror: A11yMirror } {
  /** Engine pointer ids (0 to 4) of the page pointers held down, by PointerEvent.pointerId. */
  const pointers = new Map<number, number>();
  /** The keyboard layout's printed keys, sent when they change (og.input.setKeyboardLayout). */
  let layoutSent = "";
  const sendLayout = (labels: readonly KeyLabel[]) => {
    const text = JSON.stringify(labels);
    if (text === layoutSent) return;
    layoutSent = text;
    sink.post({ type: "keyboard-layout", labels });
  };
  const listen = <K extends keyof HTMLElementEventMap>(target: EventTarget, type: K, fn: (e: HTMLElementEventMap[K]) => void) => {
    target.addEventListener(type, fn as EventListener, { passive: false });
    sink.listeners.push(() => target.removeEventListener(type, fn as EventListener));
  };
  // When an event happened on a clock the Worker shares (its `at` within the tick, input.ts stampAt).
  const stamp = (e: Event) => performance.timeOrigin + e.timeStamp;
  // The UI's text fields take the page's text through a hidden textarea
  // (text-input.ts); its keys still reach the engine, except an input
  // method's while it composes.
  // The accessibility tree over the canvas; what a screen reader does there reaches the UI as input.
  const mirror = new A11yMirror(canvas, (e) => sink.send(e));
  // The textarea is named after the field's node in the mirror (text-input.ts).
  const text = new TextEntry(canvas, (e) => sink.send(e), mirror);
  sink.listeners.push(() => text.dispose());
  sink.listeners.push(() => mirror.dispose());
  // The page's motion, contrast and transparency preferences, followed as the reader changes them.
  for (const [, q] of PAGE_PREFERENCES) {
    const media = globalThis.matchMedia?.(q);
    if (!media?.addEventListener) continue;
    const changed = () => sink.post({ type: "viewport", viewport: { ...sink.viewport(), ...pagePreferences() } });
    media.addEventListener("change", changed);
    sink.listeners.push(() => media.removeEventListener("change", changed));
  }
  const typing = (e: KeyboardEvent) => {
    if (text.owns(e.target)) return e.isComposing || text.composing || e.keyCode === 229;
    // The mirror's elements (a slider's native range among them) pass their keys to the game.
    if (mirror.owns(e.target)) return false;
    const t = e.target as { tagName?: string; isContentEditable?: boolean } | null;
    return !!t && (t.tagName === "INPUT" || t.tagName === "TEXTAREA" || t.tagName === "SELECT" || t.isContentEditable === true);
  };
  // Keys that went down while Command was held: a Mac's browser sends no
  // keyup for them, so they are released when Command is.
  const underCommand = new Set<string>();
  // Every key sent down and not yet up: released when the window loses focus.
  const heldKeys = new Set<string>();
  // The layout's printed keys: the layout map where the browser has one, else learnt from the keys pressed.
  let mapped = false;
  const learner = new LayoutLearner();
  const readLayout = () =>
    void readLayoutMap().then((labels) => {
      if (!labels || sink.stopped()) return;
      mapped = true;
      sendLayout(labels);
    });
  readLayout();
  listen(globalThis as unknown as EventTarget, "focus", readLayout);
  const keyboard = (globalThis.navigator as { keyboard?: EventTarget } | undefined)?.keyboard;
  if (keyboard?.addEventListener) listen(keyboard, "layoutchange" as keyof HTMLElementEventMap, readLayout);
  listen(globalThis as unknown as EventTarget, "keydown", (e) => {
    if (typing(e) || !e.code) return;
    // In the text target Space types and shortcuts copy, cut and paste.
    const shortcut = text.owns(e.target) && (e.ctrlKey || e.metaKey || e.code === "Space");
    if (KEPT_KEYS.has(e.code) && !shortcut) e.preventDefault();
    if (e.metaKey && !MODIFIER_KEYS.has(e.code)) underCommand.add(e.code);
    if (!mapped && learner.learn(e)) sendLayout(learner.labels());
    // A key is a gesture: a fullscreen the browser refused without one is asked again.
    sink.presentation.gesture();
    heldKeys.add(e.code);
    if (!e.repeat) sink.send({ kind: "key", key: e.code, action: "down", time: stamp(e) });
  });
  const releaseAll = () => {
    underCommand.clear();
    for (const e of releaseHeld(heldKeys, performance.timeOrigin + performance.now())) sink.send(e);
    // The browser drops the lock (and a request still out) with the focus.
    sink.presentation.blurred();
  };
  listen(globalThis as unknown as EventTarget, "blur", releaseAll);
  if (globalThis.document) {
    listen(globalThis.document as unknown as EventTarget, "visibilitychange" as keyof HTMLElementEventMap, () => {
      if (globalThis.document?.visibilityState === "hidden") releaseAll();
    });
  }
  listen(globalThis as unknown as EventTarget, "keyup", (e) => {
    if (typing(e) || !e.code) return;
    underCommand.delete(e.code);
    heldKeys.delete(e.code);
    sink.send({ kind: "key", key: e.code, action: "up", time: stamp(e) });
    if ((e.code === "MetaLeft" || e.code === "MetaRight") && !e.metaKey) {
      for (const key of underCommand) {
        heldKeys.delete(key);
        sink.send({ kind: "key", key, action: "up", time: stamp(e) });
      }
      underCommand.clear();
    }
  });
  // The pointer lock: whether the canvas holds it, and the position it began at (held while locked).
  let locked = false;
  let held = { x: 0, y: 0 };
  const doc = globalThis.document as Document | undefined;
  if (doc) {
    listen(doc as unknown as EventTarget, "pointerlockchange" as keyof HTMLElementEventMap, () => {
      const now = doc.pointerLockElement === canvas;
      if (now === locked) return;
      locked = now;
      sink.send({ kind: "pointerLock", locked: now });
      sink.presentation.lockChanged(now);
    });
    listen(doc as unknown as EventTarget, "pointerlockerror" as keyof HTMLElementEventMap, () => {
      sink.presentation.lockRefused();
    });
    listen(doc as unknown as EventTarget, "fullscreenchange" as keyof HTMLElementEventMap, () => {
      sink.presentation.fullscreenChanged(!!doc.fullscreenElement);
    });
  }
  // A window moved to a monitor of another DPR: the canvas and the UI's viewport again, before the next frame.
  const stopDpr = watchPixelRatio(() => sink.resized());
  sink.listeners.push(stopDpr);
  // Relative to the canvas's CSS box, whichever element the event landed on; while locked, where the lock began.
  const at = (e: PointerEvent | WheelEvent) => {
    if (locked) return held;
    const r = canvas.getBoundingClientRect();
    return (held = { x: e.clientX - r.left, y: e.clientY - r.top });
  };
  const point = (e: PointerEvent, action: "move" | "down" | "up", id: number) => {
    const p = at(e);
    // The mouse's relative movement (touches and pens report none worth reading).
    const mx = e.pointerType === "mouse" ? e.movementX || 0 : 0;
    const my = e.pointerType === "mouse" ? e.movementY || 0 : 0;
    sink.send({
      kind: "pointer",
      action,
      x: p.x,
      y: p.y,
      button: action === "move" ? "none" : (BUTTONS[e.button] ?? "left"),
      ...(id > 0 ? { id } : {}),
      ...(mx !== 0 || my !== 0 ? { movementX: mx, movementY: my } : {}),
      time: stamp(e),
    });
  };
  listen(canvas, "pointerdown", (e) => {
    // The game wants the lock: this press is the gesture browsers grant it to.
    if (e.pointerType === "mouse") sink.presentation.press();
    sink.presentation.gesture();
    let id = pointers.get(e.pointerId);
    if (id === undefined) {
      const taken = new Set(pointers.values());
      id = [0, 1, 2, 3, 4].find((i) => !taken.has(i));
      if (id === undefined) return; // a sixth finger: ignored
      pointers.set(e.pointerId, id);
    }
    try {
      canvas.setPointerCapture(e.pointerId);
    } catch {
      /* a synthetic event has no active pointer */
    }
    if (e.pointerType !== "mouse") e.preventDefault();
    point(e, "down", id);
  });
  listen(canvas, "pointermove", (e) => {
    const id = pointers.get(e.pointerId);
    if (id !== undefined) point(e, "move", id);
    // A hovering mouse moves the primary pointer while nothing is held (and, locked, only its movement).
    else if ((pointers.size === 0 || locked) && e.pointerType === "mouse") point(e, "move", 0);
  });
  const end = (e: PointerEvent) => {
    const id = pointers.get(e.pointerId);
    if (id === undefined) return;
    pointers.delete(e.pointerId);
    point(e, "up", id);
  };
  listen(canvas, "pointerup", end);
  listen(canvas, "pointercancel", end);
  listen(canvas, "contextmenu", (e) => e.preventDefault());
  listen(canvas, "wheel", (e) => {
    e.preventDefault();
    const p = at(e);
    sink.send({ kind: "wheel", x: p.x, y: p.y, deltaX: e.deltaX, deltaY: e.deltaY });
  });
  return { text, mirror };
}
