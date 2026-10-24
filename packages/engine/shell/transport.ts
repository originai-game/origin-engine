// The Player protocol 2 over same-origin HTTP: the transport of a top-level
// engine page on its play origin (origingame/docs/player-protocol.md §The
// top-level engine page). The page is the play page itself, so it does what
// the framed Player did around it: it keeps the game-scoped credential
// (redeeming the Portal's hand-off code from the fragment), sends each call
// as a same-origin request, keeps the offline outbox, and reports the
// session's lifecycle (start, ready, samples, the game's events, heartbeats,
// the end) for the play origin to file.
//
//   POST /__player/v2/<game>/<release>/hello   the protocol-2 hello (+ handoff) → the Player's hello, player, page
//   POST /__player/v2/<game>/<release>/call    a protocol-2 call               → its result (+ player)
//   POST /__player/v2/<game>/<release>/report  { session, device, messages }    → 202 { accepted, rejected } (+ player)
//
// One request at a time (calls and reports share the queue: a page mints at
// most one guest); 15 s without an answer is `timeout`, a failed fetch
// `offline`. The credential travels only as `Authorization: Bearer` on these
// requests; it is kept in memory, and a guest's also under the Player's
// `og_token_<game>` key. Nothing here touches og.platform's own storage.

import { fingerprint, timeOf, type PlayerEvents, type PlayerResult } from "./player.ts";

/** `window.__OG_GAME`, written into the entry at publication. */
export interface PageGame {
  readonly id: string;
  readonly releaseId: string;
}

/** What the Player page used to draw around the game, for the page to draw itself. */
export interface PageInfo {
  readonly title: string | null;
  readonly cover: string | null;
  readonly backdrop: string | null;
  readonly back: string | null;
  readonly portal: string | null;
}

export interface StorageLike {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
  removeItem(key: string): void;
}

export interface TransportEnv {
  readonly fetch: (url: string, init: RequestInit) => Promise<Response>;
  readonly storage: StorageLike | null;
  /** The page's address: the fragment is read, and the hand-off code removed from it. */
  readonly location: { readonly hash: string; readonly pathname: string; readonly search: string };
  readonly replaceUrl: (url: string) => void;
  readonly now: () => number;
  /** ms since the page's navigation start. */
  readonly clock: () => number;
  readonly random: (length: number) => Uint8Array;
  readonly setTimer?: (callback: () => void, ms: number) => unknown;
  readonly clearTimer?: (handle: unknown) => void;
  readonly locale?: string;
}

/** How long a request waits for its answer before it is `timeout`. */
export const PAGE_CALL_TIMEOUT_MS = 15_000;
/** Messages a page keeps unsent at most, and sends per report. */
export const REPORT_KEEP = 200;
export const REPORT_BATCH = 20;
/** A report's body at most, in UTF-8 bytes: the play origin's limit, and the browser's for a keepalive request in flight. */
export const REPORT_MAX_BYTES = 64 * 1024;
/** How long a save or score stays in the outbox. */
const OUTBOX_DAYS = 7;
const RETRYABLE = new Set(["offline", "timeout", "unavailable", "rate_limited"]);

/** The page's `window.__OG_GAME`, when it names a game and release that speak protocol 2. */
export function pageGame(value: unknown): PageGame | null {
  const g = value as { id?: unknown; releaseId?: unknown; protocol?: unknown } | null;
  if (!g || typeof g !== "object" || g.protocol !== 2) return null;
  if (typeof g.id !== "string" || !/^[a-z0-9]{6,32}$/.test(g.id)) return null;
  if (typeof g.releaseId !== "string" || !/^release_[A-Za-z0-9_-]{1,80}$/.test(g.releaseId)) return null;
  return { id: g.id, releaseId: g.releaseId };
}

/** Splits the Portal's one-time hand-off code (`og-h=<code>`) off a fragment; the rest stays the game's. */
export function takeHandoff(hash: string): { code: string | null; hash: string } {
  const parts = hash.replace(/^#/, "").split("&");
  let code: string | null = null;
  const rest = parts.filter((part) => {
    if (!part.startsWith("og-h=")) return true;
    const value = part.slice(5);
    if (/^[A-Za-z0-9_-]{16,256}$/.test(value)) code = value;
    return false;
  });
  if (rest.length === parts.length) return { code: null, hash };
  const joined = rest.filter(Boolean).join("&");
  return { code, hash: joined ? `#${joined}` : "" };
}

/** A failed request's status as protocol 2's error (the platform's `playerWireError` table). */
export function statusError(status: number, message: string, revision?: number): PlayerResult {
  const code =
    status === 0 ? "offline" : status < 0 ? "timeout" : status >= 500 ? "unavailable"
      : ({ 400: "invalid", 422: "invalid", 401: "unauthorized", 403: "unauthorized", 404: "not_found", 409: "conflict", 413: "too_large", 429: "rate_limited" } as Record<number, string>)[status] ?? "failed";
  return { ok: false, error: { code, message, retryable: RETRYABLE.has(code), ...(status > 0 ? { status } : {}), ...(typeof revision === "number" ? { revision } : {}) } };
}

const hex = (bytes: Uint8Array) => Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");

/** A UUID (version 7: time-ordered), the id a report message is filed under. */
export function uuid7(random: (length: number) => Uint8Array, now: number): string {
  const b = random(16);
  for (let i = 5; i >= 0; i--) b[i] = Math.floor(now / 2 ** ((5 - i) * 8)) & 0xff;
  b[6] = (b[6]! & 0x0f) | 0x70;
  b[8] = (b[8]! & 0x3f) | 0x80;
  const h = hex(b);
  return `${h.slice(0, 8)}-${h.slice(8, 12)}-${h.slice(12, 16)}-${h.slice(16, 20)}-${h.slice(20)}`;
}

/** The Player's per-game device id on this browser (`og_device_<game>`, 30 days). */
export function deviceId(storage: StorageLike | null, game: string, now: number, random: (length: number) => Uint8Array): string {
  const key = `og_device_${game}`;
  try {
    const d = JSON.parse(storage?.getItem(key) ?? "null") as { id?: unknown; createdAt?: unknown } | null;
    if (typeof d?.id === "string" && /^device_[0-9a-f]{32}$/.test(d.id) && typeof d.createdAt === "number" && d.createdAt <= now && now - d.createdAt < 30 * 864e5) return d.id;
  } catch {
    /* replaced below */
  }
  const id = `device_${hex(random(16))}`;
  try {
    storage?.setItem(key, JSON.stringify({ id, createdAt: now }));
  } catch {
    /* private mode */
  }
  return id;
}

type Outbox = { save?: { data: unknown }; scores: Array<Record<string, unknown>>; at: number };
type Message = Record<string, unknown> & { kind: string };

export interface PageConnection {
  readonly transport: "page";
  readonly framed: false;
  /** Whether the play origin answered the hello. */
  readonly hosted: boolean;
  readonly started: boolean;
  readonly paused: boolean;
  readonly capabilities: readonly string[];
  readonly time: number | undefined;
  readonly page: PageInfo | null;
  /** Resolves once the hello is answered (true) or failed (false). */
  readonly hello: Promise<boolean>;
  loading(value: number | null, label?: string): void;
  ready(): void;
  fail(code: string, message: string, retryable: boolean): void;
  /** After ready: the game stopped (only an error name and a fingerprint of the message leave the page). */
  crash(name: string, message: string): void;
  sample(fields: Record<string, unknown>): void;
  event(name: string, attrs?: Record<string, string | number | boolean | null>): void;
  request(method: string, args?: Record<string, unknown>): Promise<PlayerResult>;
  /** The person started play (the page's own gate): the session opens. */
  start(): void;
  /** The page is going away: the end report, sent with keepalive. */
  end(state: { ready: boolean; failed: boolean }): void;
  /** Sends what is queued (tests; the page flushes by itself). */
  flush(): Promise<void>;
  /** The credential this page holds now (tests). */
  readonly credential: string | null;
}

/** The HTTP transport for `game`, under its play origin's `/__player/v2/<game>/<release>/`. */
export function connectPage(game: PageGame, env: TransportEnv, events: PlayerEvents = {}): PageConnection {
  const base = `/__player/v2/${game.id}/${game.releaseId}/`;
  const tokenKey = `og_token_${game.id}`;
  const portalKey = `og_portal_${game.id}`;
  const outboxKey = `og_outbox_${game.id}`;
  const store = env.storage;
  const setTimer = env.setTimer ?? ((cb, ms) => setTimeout(cb, ms));
  const clearTimer = env.clearTimer ?? ((h) => clearTimeout(h as ReturnType<typeof setTimeout>));
  const get = (k: string) => {
    try {
      return store?.getItem(k) ?? null;
    } catch {
      return null;
    }
  };
  const set = (k: string, v: string | null) => {
    try {
      if (v === null) store?.removeItem(k);
      else store?.setItem(k, v);
    } catch {
      /* private mode: memory keeps it */
    }
  };

  // ── identity: the fragment's code, else a stored guest, else none ──
  const handoff = takeHandoff(env.location.hash);
  if (handoff.hash !== env.location.hash) env.replaceUrl(env.location.pathname + env.location.search + handoff.hash);
  let token: string | null = null;
  if (handoff.code) {
    // An attempted hand-off never leaves the previous account (or guest) available.
    set(tokenKey, null);
    set(portalKey, null);
  } else if (get(portalKey)) {
    // An account is resumed only by a fresh Portal code.
    set(tokenKey, null);
    set(portalKey, null);
  } else token = get(tokenKey);
  const adopt = (p: unknown) => {
    const player = p as { token?: unknown; guest?: unknown } | null | undefined;
    if (!player || typeof player.token !== "string" || !player.token) return;
    token = player.token;
    if (player.guest === false) {
      set(tokenKey, null);
      set(portalKey, "1");
    } else {
      set(tokenKey, player.token);
      set(portalKey, null);
    }
  };

  // ── one request at a time ──
  let chain: Promise<unknown> = Promise.resolve();
  const serial = <T>(task: () => Promise<T>): Promise<T> => {
    const next = chain.then(task, task);
    chain = next.catch(() => {});
    return next;
  };
  /** One POST: its status and JSON body; status 0 offline, -1 timed out. */
  const post = async (what: string, body: unknown, keepalive = false): Promise<{ status: number; data: Record<string, unknown> | null }> => {
    const controller = typeof AbortController === "function" ? new AbortController() : null;
    let timedOut = false;
    const timer = setTimer(() => {
      timedOut = true;
      controller?.abort();
    }, PAGE_CALL_TIMEOUT_MS);
    try {
      const response = await env.fetch(base + what, {
        method: "POST",
        credentials: "omit",
        cache: "no-store",
        keepalive,
        headers: { "Content-Type": "application/json", ...(token ? { Authorization: `Bearer ${token}` } : {}) },
        body: JSON.stringify(body),
        ...(controller ? { signal: controller.signal } : {}),
      });
      const data = (await response.json().catch(() => null)) as Record<string, unknown> | null;
      if (data && typeof data === "object") adopt(data["player"]);
      return { status: response.status, data };
    } catch {
      return { status: timedOut ? -1 : 0, data: null };
    } finally {
      clearTimer(timer);
    }
  };

  // ── hello ──
  let hosted = false;
  let capabilities: readonly string[] = [];
  let time: number | undefined;
  let page: PageInfo | null = null;
  const str = (v: unknown) => (typeof v === "string" && v ? v : null);
  const hello = serial(async () => {
    const r = await post("hello", { og: "player", protocol: 2, kind: "hello", substrate: "origin-engine", ...(handoff.code ? { handoff: handoff.code } : {}) });
    const m = r.data;
    if (r.status !== 200 || !m || m["kind"] !== "hello") return false;
    hosted = true;
    time = timeOf(m).time;
    capabilities = Array.isArray(m["capabilities"]) ? (m["capabilities"] as unknown[]).filter((c): c is string => typeof c === "string") : [];
    const p = (m["page"] ?? null) as Record<string, unknown> | null;
    page = p && typeof p === "object" ? { title: str(p["title"]), cover: str(p["cover"]), backdrop: str(p["backdrop"]), back: str(p["back"]), portal: str(p["portal"]) } : null;
    events.hello?.({ started: false, paused: false, capabilities });
    return true;
  });

  // ── reports: nothing before start ──
  const session = `game_session_${hex(env.random(16))}`;
  const device = deviceId(store, game.id, env.now(), env.random);
  let started = false;
  let held: Message[] = [];
  let unsent: Message[] = [];
  let flushTimer: unknown = null;
  let readySent = false;
  let failed = false;
  let lastValue = -1;
  let startedAt = 0;
  let heartbeat: unknown = null;
  const stamp = (m: Message): Message => ({ og: "player", protocol: 2, ...m, id: uuid7(env.random, env.now()), at: env.now(), ms: Math.round(env.clock()) });
  const keep = (m: Message) => {
    unsent.push(m);
    if (unsent.length > REPORT_KEEP) unsent.splice(0, unsent.length - REPORT_KEEP);
  };
  const report = (m: Message) => {
    const s = stamp(m);
    if (!started) {
      held.push(s);
      if (held.length > REPORT_KEEP) held.splice(0, held.length - REPORT_KEEP);
      return;
    }
    keep(s);
    schedule();
  };
  const schedule = (ms = 1000) => {
    if (flushTimer !== null) return;
    flushTimer = setTimer(() => {
      flushTimer = null;
      void flush();
    }, ms);
  };
  const encoder = new TextEncoder();
  const bytes = (v: unknown) => encoder.encode(JSON.stringify(v)).length;
  /** At most `count` messages from the front (or the end) whose report fits REPORT_MAX_BYTES; always at least one. */
  const batchOf = (messages: readonly Message[], count: number, fromEnd = false): Message[] => {
    const order = fromEnd ? [...messages].reverse() : messages;
    let size = bytes({ session, device, messages: [] });
    const batch: Message[] = [];
    for (const m of order) {
      if (batch.length >= count) break;
      const more = bytes(m) + (batch.length > 0 ? 1 : 0);
      if (batch.length > 0 && size + more > REPORT_MAX_BYTES) break;
      size += more;
      batch.push(m);
    }
    return fromEnd ? batch.reverse() : batch;
  };
  let flushing: Promise<void> | null = null;
  const flush = (): Promise<void> => {
    if (flushing) return flushing;
    flushing = serial(async () => {
      let count = REPORT_BATCH;
      while (unsent.length) {
        const batch = batchOf(unsent, count);
        const r = await post("report", { session, device, messages: batch });
        // Too large: halve the batch; a single message that still does not fit is dropped below.
        if (r.status === 413 && batch.length > 1) {
          count = Math.ceil(batch.length / 2);
          continue;
        }
        // 400 (invalid) and 413 are final: sending the same batch again fails the same way, so it is dropped.
        if (r.status !== 202 && r.status !== 200 && r.status !== 400 && r.status !== 413) return; // kept for the next report
        unsent = unsent.filter((m) => !batch.includes(m));
      }
    }).finally(() => (flushing = null));
    return flushing;
  };

  // ── the outbox (the Player's key and shape) ──
  const readBox = (): Outbox => {
    try {
      const b = JSON.parse(get(outboxKey) ?? "null") as Outbox | null;
      if (b && Array.isArray(b.scores) && env.now() - b.at < OUTBOX_DAYS * 864e5) return b;
    } catch {
      /* corrupted */
    }
    return { scores: [], at: env.now() };
  };
  const writeBox = (b: Outbox) => set(outboxKey, b.save || b.scores.length ? JSON.stringify({ ...b, at: env.now() }) : null);

  /** One call as its protocol-2 result. */
  let nextCall = 1;
  const call = (method: string, args: Record<string, unknown>): Promise<PlayerResult> =>
    serial(async () => {
      const id = `oe-${nextCall++}`;
      const r = await post("call", { og: "player", protocol: 2, kind: "call", id, method, args });
      const m = r.data;
      if (m && m["kind"] === "result" && m["id"] === id) {
        if (m["ok"] === true) return { ok: true, data: m["data"] ?? null, ...timeOf(m) };
        const e = (m["error"] ?? {}) as Record<string, unknown>;
        const code = typeof e["code"] === "string" ? e["code"] : "failed";
        return {
          ok: false,
          error: {
            code,
            message: typeof e["message"] === "string" ? e["message"] : "",
            retryable: e["retryable"] === true,
            ...(typeof e["status"] === "number" ? { status: e["status"] } : {}),
            ...(typeof e["revision"] === "number" ? { revision: e["revision"] } : {}),
          },
          ...timeOf(m),
        };
      }
      return statusError(r.status, r.status === 0 ? "the network is unreachable" : r.status < 0 ? `no answer to ${method} in ${PAGE_CALL_TIMEOUT_MS} ms` : `the play origin answered ${r.status}`);
    });

  let delivering = false;
  /** Delivers the outbox in order: the latest save, then the scores. */
  const deliver = async (): Promise<void> => {
    if (delivering) return;
    delivering = true;
    try {
      let box = readBox();
      if (box.save) {
        const r = await call("save.set", { data: box.save.data });
        if (!r.ok && r.error.retryable) return;
        box = readBox();
        delete box.save;
        writeBox(box);
      }
      for (box = readBox(); box.scores.length; box = readBox()) {
        const r = await call("leaderboard.submit", { ...box.scores[0] });
        if (!r.ok && r.error.retryable) return;
        box.scores.shift();
        writeBox(box);
      }
    } finally {
      delivering = false;
    }
  };
  void hello.then(() => {
    const b = readBox();
    if (b.save || b.scores.length) void deliver();
  });

  const request = async (method: string, args: Record<string, unknown> = {}): Promise<PlayerResult> => {
    await hello;
    const queueable = method === "leaderboard.submit" || (method === "save.set" && (args["revision"] === undefined || args["revision"] === null));
    if (method === "save.get" || method === "save.read") {
      const pending = readBox().save;
      if (pending && method === "save.get") return { ok: true, data: pending.data };
      if (pending) {
        await deliver();
        const still = readBox().save;
        if (still) return { ok: true, data: { data: still.data, revision: null, queued: true } };
      }
    }
    if (method === "save.set" && queueable) {
      // The latest save wins: a queued older one is dropped first.
      const b = readBox();
      delete b.save;
      writeBox(b);
    }
    const r = await call(method, args);
    if (r.ok) {
      const b = readBox();
      if (b.save || b.scores.length) void deliver();
      return r;
    }
    if (!queueable || !r.error.retryable) return r;
    const b = readBox();
    if (method === "save.set") b.save = { data: args["data"] ?? null };
    else b.scores = [...b.scores, { score: args["score"], ...(args["board"] === undefined ? {} : { board: args["board"] }), ...(args["order"] === undefined ? {} : { order: args["order"] }) }].slice(-20);
    writeBox(b);
    return { ok: true, data: method === "save.set" ? { ok: true, queued: true } : { ok: true, rank: null, queued: true } };
  };

  return {
    transport: "page",
    framed: false,
    get hosted() {
      return hosted;
    },
    get started() {
      return started;
    },
    paused: false,
    get capabilities() {
      return capabilities;
    },
    get time() {
      return time;
    },
    get page() {
      return page;
    },
    hello,
    get credential() {
      return token;
    },
    loading(value) {
      if (readySent || value === null) return;
      const v = Math.round(Math.min(1, Math.max(0, value)) * 1000) / 1000;
      if (v <= lastValue) return;
      lastValue = v;
      // Accepted and not filed by the play origin: not worth a request each.
    },
    ready() {
      if (readySent || failed) return;
      readySent = true;
      report({ kind: "ready" });
    },
    fail(code, message, retryable) {
      if (readySent || failed) return;
      failed = true;
      report({ kind: "fail", code, message: message.slice(0, 400), retryable });
    },
    crash(name, message) {
      failed = true;
      report({ kind: "crash", name: /^[A-Za-z][A-Za-z0-9_.-]{0,63}$/.test(name) ? name : "Error", fingerprint: fingerprint(message) });
    },
    sample(fields) {
      report({ kind: "sample", substrate: "origin-engine", ...fields });
    },
    event(name, attrs) {
      report({ kind: "event", name, ...(attrs ? { attrs } : {}) });
    },
    request,
    start() {
      if (started) return;
      started = true;
      startedAt = env.now();
      const first = stamp({ kind: "start" });
      keep(first);
      for (const m of held.splice(0)) keep(m);
      void flush();
      const beat = () => {
        heartbeat = setTimer(() => {
          if (readySent) {
            keep(stamp({ kind: "heartbeat", durationMs: env.now() - startedAt }));
            void flush();
          }
          beat();
        }, 60_000);
      };
      beat();
    },
    end(state) {
      if (heartbeat !== null) clearTimer(heartbeat);
      heartbeat = null;
      if (!started) return;
      keep(stamp({ kind: "end", durationMs: env.now() - startedAt, ready: state.ready, failed: state.failed }));
      // The page is going: no queue to wait behind, keepalive so the request outlives it.
      const batch = batchOf(unsent, REPORT_BATCH, true);
      unsent = [];
      void post("report", { session, device, messages: batch }, true);
    },
    flush,
  };
}
