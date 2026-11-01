// The pointer lock on the page (mouse-look; og.PointerLock, og.input.setMouse):
// what the shell asks of the browser, decided by one state machine so that no
// sequence of Escape, focus loss, a refused or unanswered request, or the game
// changing its mind leaves the game stuck with a hidden cursor it cannot use or
// a lock it can never get back.
//
// The browser grants the lock only to a gesture (a press on the canvas), so a
// wanted lock is `armed` until the next press, `asking` while the request is
// out, `locked` once pointerlockchange says so. Escape and focus loss release
// it (the browser does), back to `armed`: the next click locks again. A request
// that is refused (Chrome refuses one about a second after the player's
// Escape; a sandboxed frame refuses every one) is armed again, and one that is
// never answered is asked again by a press after ASK_TIMEOUT_MS. A lock
// granted after the game stopped wanting it is released at once.
//
// Raw input (og.input's MouseDesc.raw, pointer lock's unadjustedMovement: no OS
// pointer acceleration) is asked for where the browser has not refused it:
// Chrome and Edge on Windows, macOS and ChromeOS grant it. A browser that
// rejects the option (NotSupportedError: Chrome on Linux) is asked again at
// once without it, within the same gesture; a browser whose requestPointerLock
// returns no promise (Firefox, older Safari) ignores the option. Either way
// the lock is the plain one and `rawUnsupported` says so (og.input.mouse).

/** A request that the browser has not answered after this long is asked again on the next press. */
export const ASK_TIMEOUT_MS = 1000;

export type LockPhase = "free" | "armed" | "asking" | "locked";

/** What the shell does for the machine. */
export type LockAction = { readonly kind: "request"; readonly raw: boolean } | { readonly kind: "exit" };

export class PointerLockMachine {
  /** og.PointerLock.wanted, as the last frame said. */
  wanted = false;
  /** og.input's MouseDesc.raw. */
  raw = true;
  phase: LockPhase = "free";
  /** The lock delivers raw movement now. */
  rawActive = false;
  /** The browser refused unadjustedMovement (or ignores it): every lock from now on is the plain one. */
  rawUnsupported = false;
  #askedAt = 0;
  #askedRaw = false;

  /** The game's ask (each frame's `pointerLock` and `rawMouse`). */
  want(wanted: boolean, raw: boolean): LockAction[] {
    this.raw = raw;
    if (wanted === this.wanted) return [];
    this.wanted = wanted;
    if (wanted) {
      if (this.phase === "free") this.phase = "armed";
      return [];
    }
    const was = this.phase;
    this.phase = "free";
    // Releasing needs no gesture; a request still out is released when it lands (granted, below).
    return was === "locked" ? [{ kind: "exit" }] : [];
  }

  /** A mouse press on the canvas at `now` (ms): the gesture a request needs. */
  press(now: number): LockAction[] {
    if (!this.wanted) return [];
    if (this.phase === "armed" || (this.phase === "asking" && now - this.#askedAt >= ASK_TIMEOUT_MS)) return [this.#ask(now, this.raw && !this.rawUnsupported)];
    return [];
  }

  #ask(now: number, raw: boolean): LockAction {
    this.phase = "asking";
    this.#askedAt = now;
    this.#askedRaw = raw;
    return { kind: "request", raw };
  }

  /** pointerlockchange: the canvas holds the lock now, or lost it (Escape, focus loss, an exit). */
  changed(locked: boolean): LockAction[] {
    if (locked) {
      if (!this.wanted) {
        this.phase = "free";
        return [{ kind: "exit" }];
      }
      this.phase = "locked";
      return [];
    }
    this.rawActive = false;
    this.phase = this.wanted ? "armed" : "free";
    return [];
  }

  /**
   * The request's promise resolved (`raw`: with unadjustedMovement), or the
   * browser returned none (`promise` false), in which case it ignored the option.
   */
  answered(promise: boolean): void {
    if (!promise && this.#askedRaw) this.rawUnsupported = true;
    this.rawActive = promise && this.#askedRaw;
  }

  /**
   * The request was refused (its promise rejected, it threw, or pointerlockerror).
   * `unsupported`: the browser rejected unadjustedMovement itself: asked again
   * at once without it, inside the same gesture.
   */
  refused(now: number, unsupported: boolean): LockAction[] {
    if (unsupported && this.#askedRaw) {
      this.rawUnsupported = true;
      if (this.wanted && this.phase === "asking") return [this.#ask(now, false)];
    }
    if (this.phase === "asking") this.phase = this.wanted ? "armed" : "free";
    return [];
  }

  /** The window lost focus or the page was hidden: the browser drops the lock and any request with it. */
  blurred(): LockAction[] {
    if (this.phase === "asking") this.phase = this.wanted ? "armed" : "free";
    return this.phase === "locked" ? [{ kind: "exit" }] : [];
  }
}
