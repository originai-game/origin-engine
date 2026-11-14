// What the shell (shell.ts) carries out on the page for a frame's answer:
// the UI's pointer shape, the pointer lock (pointer-lock.ts's state machine
// decides every step), fullscreen, and the power preference kept for the
// next load.

import type { CursorImage } from "../../runtime.ts";
import { PointerLockMachine, type LockAction } from "../pointer-lock.ts";
import { safeStorage } from "./viewport.ts";

/** The PNG data URL of the last cursor image drawn, by the image it was drawn from. */
let cursorUrl: [CursorImage, string] | null = null;

/** CSS `cursor` for the UI's shape: its `url()` image (RGBA from the engine) as a PNG at its hotspot, then the keyword. */
function cursorCss(keyword: string, image: CursorImage | null): string {
  if (!image) return keyword;
  let url = cursorUrl?.[0] === image ? cursorUrl[1] : "";
  if (!url) {
    const canvas = document.createElement("canvas");
    canvas.width = image.width;
    canvas.height = image.height;
    const bytes = Uint8ClampedArray.from(atob(image.rgba), (ch) => ch.charCodeAt(0));
    canvas.getContext("2d")?.putImageData(new ImageData(bytes, image.width, image.height), 0, 0);
    url = canvas.toDataURL("image/png");
    cursorUrl = [image, url];
  }
  return `url(${url}) ${image.hotspot[0]} ${image.hotspot[1]}, ${keyword}`;
}

export class Presentation {
  readonly #canvas: HTMLCanvasElement;
  /** A message to the engine Worker. */
  readonly #post: (message: Record<string, unknown>) => void;
  readonly #displayKey: string;
  /** The pointer lock's state machine (pointer-lock.ts): og.PointerLock.wanted and og.input's raw input in, requests out. */
  readonly #lock = new PointerLockMachine();
  /** What the last frame asked of the page: fullscreen and the power preference for the next load. */
  #fullWanted = false;
  /** A fullscreen request waits for the next gesture (the browser refused one without it). */
  #fullArmed = false;
  #mouseSent = "";
  #lowPowerKept: boolean | null = null;

  /** `canvas` is the pointer lock's element; the power preference is kept under `displayKey`. */
  constructor(canvas: HTMLCanvasElement, post: (message: Record<string, unknown>) => void, displayKey: string) {
    this.#canvas = canvas;
    this.#post = post;
    this.#displayKey = displayKey;
  }

  /**
   * The UI's pointer shape over what the pointer is on (og.ui.frameCursor):
   * its url() image at its hotspot, the keyword after it as CSS falls back.
   */
  cursor(keyword: string, image: CursorImage | null): void {
    this.#canvas.style.cursor = cursorCss(keyword, image);
  }

  /** og.PointerLock.wanted and og.input's raw mouse as a frame's answer said them. */
  want(wanted: boolean, raw: boolean): void {
    // Releasing needs no gesture: the lock ends at once. Taking it waits for the next press.
    this.#act(this.#lock.want(wanted, raw));
  }

  /** A frame's page asks: the power preference for the next load, and fullscreen. */
  page(ask: { fullscreen?: boolean; lowPower?: boolean }): void {
    if (typeof ask.lowPower === "boolean") this.#keepLowPower(ask.lowPower);
    if (typeof ask.fullscreen === "boolean") this.#fullscreen(ask.fullscreen);
  }

  /** A press, the gesture browsers grant the pointer lock to: a lock the game wants is asked for. */
  press(): void {
    this.#act(this.#lock.press(performance.now()));
  }

  /** A gesture (a press or a key): a fullscreen the browser refused without one is asked again. */
  gesture(): void {
    if (this.#fullArmed) this.#enterFullscreen();
  }

  /** The window lost focus: the browser drops the lock (and a request still out) with it. */
  blurred(): void {
    this.#act(this.#lock.blurred());
  }

  /** pointerlockchange: whether the canvas holds the lock now. */
  lockChanged(locked: boolean): void {
    this.#act(this.#lock.changed(locked));
    this.#reportMouse();
  }

  /** pointerlockerror: the request was refused; armed for the next press. */
  lockRefused(): void {
    this.#act(this.#lock.refused(performance.now(), false));
    this.#reportMouse();
  }

  /** fullscreenchange: whether the page is fullscreen now. */
  fullscreenChanged(on: boolean): void {
    // Left by the player (Escape, F11): the setting follows, and nothing asks again.
    if (!on) {
      this.#fullWanted = false;
      this.#fullArmed = false;
    }
    this.#post({ type: "fullscreen", on });
  }

  /** Do what the pointer lock's machine says: request the lock (raw or not) or release it. */
  #act(actions: readonly LockAction[]): void {
    const doc = globalThis.document;
    const canvas = this.#canvas;
    for (const a of actions) {
      if (a.kind === "exit") {
        if (doc?.pointerLockElement && doc.pointerLockElement === canvas) doc.exitPointerLock?.();
        continue;
      }
      let asked: unknown;
      try {
        asked = (canvas.requestPointerLock as (o?: unknown) => unknown).call(canvas, a.raw ? { unadjustedMovement: true } : undefined);
      } catch {
        // Refused at once (no gesture, a sandboxed frame): armed for the next press.
        this.#act(this.#lock.refused(performance.now(), false));
        this.#reportMouse();
        continue;
      }
      if (asked instanceof Promise) {
        asked.then(
          () => {
            this.#lock.answered(true);
            this.#reportMouse();
          },
          (e: unknown) => {
            this.#act(this.#lock.refused(performance.now(), (e as { name?: string } | null)?.name === "NotSupportedError"));
            this.#reportMouse();
          },
        );
      } else {
        this.#lock.answered(false);
        this.#reportMouse();
      }
    }
  }

  /** og.input.mouse's rawActive and rawUnsupported, when they changed. */
  #reportMouse(): void {
    const report = { active: this.#lock.rawActive, unsupported: this.#lock.rawUnsupported };
    const text = JSON.stringify(report);
    if (text === this.#mouseSent) return;
    this.#mouseSent = text;
    this.#post({ type: "mouse-report", ...report });
  }

  /** The power preference for the next load, kept in the page's storage. */
  #keepLowPower(lowPower: boolean): void {
    if (lowPower === this.#lowPowerKept) return;
    this.#lowPowerKept = lowPower;
    try {
      safeStorage()?.setItem(this.#displayKey, JSON.stringify({ lowPower }));
    } catch {
      /* storage full or closed: the next load asks for high performance */
    }
  }

  /** Enter or leave fullscreen as the last frame asks; a refused entry waits for the next gesture. */
  #fullscreen(wanted: boolean): void {
    if (wanted === this.#fullWanted) return;
    this.#fullWanted = wanted;
    this.#fullArmed = false;
    const doc = globalThis.document;
    if (!doc) return;
    if (!wanted) {
      if (doc.fullscreenElement) void doc.exitFullscreen?.().catch(() => {});
      return;
    }
    this.#enterFullscreen();
  }

  #enterFullscreen(): void {
    const doc = globalThis.document;
    if (!doc || doc.fullscreenElement || !this.#fullWanted) return;
    const gesture = (globalThis.navigator as { userActivation?: { isActive?: boolean } } | undefined)?.userActivation?.isActive;
    const asked = doc.documentElement?.requestFullscreen?.({ navigationUI: "hide" });
    const refused = () => {
      // Refused inside a gesture (a frame without allow="fullscreen"): the page is not
      // fullscreen and the setting says so. Without one: the next press or key asks again.
      if (gesture === true || this.#fullArmed) {
        this.#fullArmed = false;
        this.#fullWanted = false;
        this.#post({ type: "fullscreen", on: false });
      } else this.#fullArmed = true;
    };
    if (asked instanceof Promise) asked.catch(refused);
    else if (!asked) refused();
  }
}
