// The Player wire, protocol 2 (origingame/docs/player-protocol.md; types in
// origin/packages/contracts/src/player-wire.ts), game side: the shell says
// `hello` at startup and owns its readiness: `loading` with the real bytes
// received, `ready` at the first rendered frame, `fail` before ready,
// `crash` after it (an error name and a fingerprint only), one `sample`
// with the startup milestones and the renderer. It hears `hello`, `start`,
// `gesture`, `pause` and `resume` from the Player, and the `result` of each
// `call` it made (og.platform's calls, through ./platform.ts).
//
// Rules: post only to `window.parent`, naming this page's own origin (the
// Player and its releases share the play origin); accept only messages from
// `window.parent` at this origin with `og: "player", protocol: 2`.
//
// A top-level page with no `window.__OG_GAME` that was told its own origin
// as the Stage host (`window.__OG_STAGE_HOST__`, decided as the og-probe
// wire decides it: ./wire.ts `stageInPage`) has the platform's Stage overlay
// drawn in its own document, and that overlay is its Player: the shell posts
// to its own window at its own origin and hears only its own window at that
// origin. Every post there reaches the shell too, so a `hello` that names a
// `substrate` or lists no `capabilities` (the shell's own) is not the
// Player's. The hello repeats every 250 ms until the overlay answers (or
// HELLO_WAIT_MS passes), in case its listener comes up after the shell.
// Any other unframed page sends nothing and hears nothing.
//
// This is the `frame` transport. A top-level page the platform published
// (`window.__OG_GAME`) speaks the same protocol to its play origin over
// same-origin HTTP instead (`page`, ./transport.ts); og build declares both
// in `<meta name="og-player" content="frame page">`.

import { stageInPage } from "./wire.ts";

export const PLAYER_WIRE = "player";
export const PLAYER_WIRE_PROTOCOL = 2;

export interface PlayerEvents {
  /** The Player answered hello (`started`: play began already; `paused`: it covers the game). */
  hello?(m: { started: boolean; paused: boolean; capabilities: readonly string[] }): void;
  start?(): void;
  gesture?(): void;
  pause?(): void;
  resume?(): void;
}

/** A short fingerprint of an error message: `[a-z0-9-]{1,64}` (FNV-1a of the first line, hex). */
export function fingerprint(message: string): string {
  const line = (message.split("\n")[0] ?? "").replace(/\d+/g, "#");
  let h = 0x811c9dc5;
  for (let i = 0; i < line.length; i++) {
    h ^= line.charCodeAt(i);
    h = Math.imul(h, 0x01000193) >>> 0;
  }
  return `oe-${h.toString(16).padStart(8, "0")}`;
}

/** A `result`: the data, or the Player's error; `time` when the Player stamped it (UTC ms, its clock). */
export type PlayerResult =
  | { readonly ok: true; readonly data: unknown; readonly time?: number }
  | { readonly ok: false; readonly error: { readonly code: string; readonly message: string; readonly retryable: boolean; readonly status?: number; readonly revision?: number }; readonly time?: number };

/** A message's `time` field, when it is a finite number. */
export const timeOf = (m: Record<string, unknown>): { time?: number } =>
  typeof m["time"] === "number" && Number.isFinite(m["time"]) ? { time: m["time"] } : {};

/** How long a call waits for its `result` before it fails with `timeout`. */
export const CALL_TIMEOUT_MS = 15_000;

/** Who the bridge speaks to: the framing Player, the Stage overlay in this document, or no one. */
export type PlayerPeer = "parent" | "self" | null;

/** How long the shell repeats its hello to the Stage overlay in its own document. */
const SELF_HELLO_EVERY_MS = 250;
const SELF_HELLO_TRIES = 6;

export function connectPlayer(events: PlayerEvents) {
  const w = globalThis as unknown as Window & Record<string, unknown>;
  const framed = (() => {
    try {
      return w.parent !== w;
    } catch {
      return true;
    }
  })();
  const peer: PlayerPeer = framed ? "parent" : w["__OG_GAME"] === undefined && stageInPage(w) ? "self" : null;
  const target = (): Window => (peer === "self" ? w : w.parent);
  const origin = w.location.origin;
  let lastValue = -1;
  let readySent = false;
  // After fail or crash the session is over: a late ready is not sent.
  let ended = false;
  let heard = false;
  let started = false;
  let paused = false;
  let capabilities: readonly string[] = [];
  let nextCall = 1;
  const calls = new Map<string, (r: PlayerResult) => void>();

  const post = (kind: string, fields: Record<string, unknown> = {}) => {
    if (!peer || !origin || origin === "null") return;
    try {
      target().postMessage({ og: PLAYER_WIRE, protocol: PLAYER_WIRE_PROTOCOL, kind, ...fields }, origin);
    } catch {
      /* a cross-origin parent is not our Player */
    }
  };

  const onMessage = (e: MessageEvent) => {
    if (!peer || e.source !== target() || e.origin !== origin) return;
    const m = e.data as Record<string, unknown> | null;
    if (!m || typeof m !== "object" || m["og"] !== PLAYER_WIRE || m["protocol"] !== PLAYER_WIRE_PROTOCOL) return;
    switch (m["kind"]) {
      case "hello":
        // In one window the shell hears its own hello: only the Player's lists capabilities.
        if (peer === "self" && ("substrate" in m || !Array.isArray(m["capabilities"]))) break;
        heard = true;
        if (m["started"] === true) started = true;
        paused = m["paused"] === true;
        capabilities = Array.isArray(m["capabilities"]) ? (m["capabilities"] as unknown[]).filter((c): c is string => typeof c === "string") : [];
        events.hello?.({ started: m["started"] === true, paused, capabilities });
        break;
      case "result": {
        const id = m["id"];
        const done = typeof id === "string" ? calls.get(id) : undefined;
        if (!done) break;
        calls.delete(id as string);
        if (m["ok"] === true) done({ ok: true, data: m["data"] ?? null, ...timeOf(m) });
        else {
          const e = (m["error"] ?? {}) as Record<string, unknown>;
          const code = typeof e["code"] === "string" ? e["code"] : "failed";
          done({
            ok: false,
            error: {
              code,
              message: typeof e["message"] === "string" ? e["message"] : "",
              retryable: e["retryable"] === true,
              ...(typeof e["status"] === "number" ? { status: e["status"] } : {}),
              ...(typeof e["revision"] === "number" ? { revision: e["revision"] } : {}),
            },
            ...timeOf(m),
          });
        }
        break;
      }
      case "start":
        heard = true;
        started = true;
        events.start?.();
        break;
      case "gesture":
        events.gesture?.();
        break;
      case "pause":
        paused = true;
        events.pause?.();
        break;
      case "resume":
        paused = false;
        events.resume?.();
        break;
    }
  };
  if (peer) addEventListener("message", onMessage);
  post("hello", { substrate: "origin-engine" });
  if (peer === "self") {
    let tries = 1;
    const again = setInterval(() => {
      if (heard || tries++ >= SELF_HELLO_TRIES) clearInterval(again);
      else post("hello", { substrate: "origin-engine" });
    }, SELF_HELLO_EVERY_MS);
  }

  return {
    /** Whether the page is framed (a Player may be listening). */
    framed,
    /** Who answers: the framing Player (`parent`), the Stage overlay in this document (`self`), or no one. */
    peer,
    /** Whether a Player answered hello (or sent start). */
    get hosted(): boolean {
      return heard;
    },
    /** Whether the Player said play began. */
    get started(): boolean {
      return started;
    },
    /** Whether the Player covers the game now. */
    get paused(): boolean {
      return paused;
    },
    /** The methods the Player's hello said it answers (empty before it, or with no Player). */
    get capabilities(): readonly string[] {
      return capabilities;
    },
    loading(value: number | null, label?: string) {
      if (readySent) return;
      if (value !== null) {
        const v = Math.round(Math.min(1, Math.max(0, value)) * 1000) / 1000;
        if (v <= lastValue) return;
        lastValue = v;
        post("loading", { value: v, ...(label ? { label } : {}) });
      } else post("loading", { value: null });
    },
    ready() {
      if (readySent || ended) return;
      readySent = true;
      post("ready");
    },
    fail(code: string, message: string, retryable: boolean) {
      if (readySent) return;
      ended = true;
      post("fail", { code, message: message.slice(0, 400), retryable });
      lastValue = -1;
    },
    crash(name: string, message: string) {
      ended = true;
      post("crash", { name: /^[A-Za-z][A-Za-z0-9_.-]{0,63}$/.test(name) ? name : "Error", fingerprint: fingerprint(message) });
    },
    sample(fields: { renderer?: string; timings?: Record<string, number> }) {
      post("sample", { substrate: "origin-engine", ...fields });
    },
    event(name: string, attrs?: Record<string, string | number | boolean | null>) {
      post("event", { name, ...(attrs ? { attrs } : {}) });
    },
    /** One `call`, answered by its `result` (or `timeout` after CALL_TIMEOUT_MS; `unavailable` with no Player). */
    request(method: string, args?: Record<string, unknown>, timeoutMs = CALL_TIMEOUT_MS): Promise<PlayerResult> {
      if (!peer) return Promise.resolve({ ok: false, error: { code: "unavailable", message: "no Player frames this page", retryable: false } });
      const id = `oe-${nextCall++}`;
      return new Promise((resolve) => {
        const timer = setTimeout(() => {
          if (calls.delete(id)) resolve({ ok: false, error: { code: "timeout", message: `the Player did not answer ${method} in ${timeoutMs} ms`, retryable: true } });
        }, timeoutMs);
        calls.set(id, (r) => {
          clearTimeout(timer);
          resolve(r);
        });
        post("call", { id, method, ...(args ? { args } : {}) });
      });
    },
  };
}
