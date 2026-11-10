// The provider contract (docs/design.md §2.8 Providers): what the page that
// hosts a game plugs in to answer og.platform and carry its rooms. The
// OriginGame Player (shell/platform.ts, Player protocol 2) and the local
// document (./platform-local.ts) are providers; so is a portal's, a store's
// or a mini-game platform's adapter written outside the engine. Game code is
// the same under each, and never chooses one: the page does
// (docs/web-export.md §Providers).
//
// The rules every provider keeps: it owns every credential and URL (game
// code names neither), the ids it hands back are opaque and scoped to the
// game, it answers each call once (telemetry never), a method it does not
// list is answered `unavailable`, and a published page never fakes money.

/**
 * The contract's version. It moves only with a break of this module: a
 * member removed or renamed, a type narrowed, an answer's or room event's
 * shape changed, or a rule above changed. Adding a method to the vocabulary,
 * an error code, or an optional member is additive and keeps the version. A
 * provider built against version N works with every engine whose
 * PROVIDER_VERSION is N; the engine's changelog names each bump as a break.
 */
export const PROVIDER_VERSION = 1;

/** Purchases, ads, achievements, the player, friends, invites and share (docs/platform-asks.md #19). */
export const SERVICE_METHODS = [
  "purchase.products",
  "purchase.buy",
  "purchase.entitlements",
  "purchase.consume",
  "ad.ready",
  "ad.show",
  "achievement.unlock",
  "achievement.progress",
  "achievement.list",
  "player.get",
  "friends.list",
  "invite.send",
  "invite.list",
  "share",
] as const;

/** The methods any store or portal can answer: the core vocabulary. */
export const CORE_METHODS = ["store.get", "store.set", "store.list", "leaderboard.submit", "leaderboard.top", "leaderboard.around", "telemetry", ...SERVICE_METHODS] as const;

/** Methods one host defines: `point` (Origin Point) and `ai.mark` are OriginGame's. */
export const HOST_EXTENSIONS = ["point", "ai.mark"] as const;

export type CoreMethod = (typeof CORE_METHODS)[number];
export type HostExtension = (typeof HOST_EXTENSIONS)[number];
/** Every method og.platform calls (og.platform.info's capabilities name these). */
export type PlatformMethod = CoreMethod | HostExtension;

/** The one-way method: never answered. */
export const ONE_WAY_METHODS: readonly PlatformMethod[] = ["telemetry"];

/** Every error code an answer may carry (crates/og-api/api/platform.ogt). */
export const ERROR_CODES = ["offline", "timeout", "invalid", "unauthorized", "not_found", "conflict", "too_large", "rate_limited", "unavailable", "failed", "cancelled", "no_fill"] as const;
export type ErrorCode = (typeof ERROR_CODES)[number];

/** The codes whose call may succeed later (`retryable: true`). */
export const RETRYABLE_CODES = ["offline", "timeout", "unavailable", "rate_limited", "no_fill"] as const satisfies readonly ErrorCode[];

const RETRYABLE: ReadonlySet<string> = new Set(RETRYABLE_CODES);

/** Whether an error with `code` is retryable. */
export const retryable = (code: string): boolean => RETRYABLE.has(code);

/** One platform call, as og_platform_take lists it. */
export interface PlatformCall {
  readonly request: number;
  readonly method: string;
  readonly args: Record<string, unknown>;
  readonly tick: number;
}

/** An answer, as og_platform_answer takes it. */
export type PlatformAnswer = (
  | { readonly ok: true; readonly data: unknown }
  | { readonly ok: false; readonly error: { readonly code: string; readonly message: string; readonly retryable: boolean; readonly revision?: number } }
) & {
  /** The answering host's clock, UTC ms (a Player that sends its time); the engine's host clock fills it in otherwise. */
  readonly time?: number;
};

/** Unasked host input (og_platform_notify): applied at the next tick's start as og.PlatformNotice. */
export interface PlatformNotice {
  /** `host`: a message from the page around the game (`{ name, ... }`), whose meaning the game and that page agree. */
  readonly kind: "purchase" | "entitlements" | "invite" | "host";
  readonly data: unknown;
  readonly time?: number;
}

/** What a platform provider may do to the engine it answers, besides answering. */
export interface PlatformLink {
  /** The engine's tick now (read between ticks). */
  readonly tick: number;
  /** Answer one call (as `handle`'s `answer`). */
  answer(request: number, result: PlatformAnswer): void;
  /** Send a notice. */
  notify(notice: PlatformNotice): void;
  /** Pause the game through the host's lifecycle (no ticks, no audio), as while an ad shows. */
  pause(): void;
  /** Resume it. */
  resume(): void;
}

/** What answers an engine's platform calls. */
export interface PlatformProvider {
  /** og.platform.info's `host`: `player`, `local`, `none`, or the provider's own name. */
  readonly host: string;
  /** The methods it answers (og.platform.supports); any other is answered `unavailable`. */
  readonly capabilities: readonly string[];
  /** The provider's UTC milliseconds, sampled when it attaches (og.platform.info's clock reads `player` then). */
  readonly time?: number | undefined;
  /** Handle calls in order; answer every one but `telemetry` exactly once, now or later. */
  handle(calls: readonly PlatformCall[], answer: (request: number, result: PlatformAnswer) => void, link?: PlatformLink): void;
  /** Called on every pump, with calls or without: a provider that answers later or sends notices on its own. */
  poll?(link: PlatformLink): void;
}

/** A successful answer. */
export const ok = (data: unknown): PlatformAnswer => ({ ok: true, data });

/** A failed answer with `code`; `retryable` follows the code. */
export const failure = (code: string, message: string): PlatformAnswer => ({ ok: false, error: { code, message, retryable: retryable(code) } });

/** The answer to a method this provider does not answer. */
export const unavailable = (method: string, host = "this host"): PlatformAnswer => failure("unavailable", `${host} answers no ${method}`);

// ── rooms ──────────────────────────────────────────────────────────────

/** An op, from the engine to its room provider (crates/og-net/src/protocol.rs). */
export type RoomOp =
  | { readonly op: "create"; readonly name?: string; readonly capacity?: number }
  | { readonly op: "join"; readonly room: string; readonly name?: string }
  | { readonly op: "quick"; readonly mode?: string; readonly players?: number; readonly timeoutMs?: number; readonly name?: string }
  | { readonly op: "leave" }
  | { readonly op: "send"; readonly to?: number; readonly channel?: string; readonly data?: unknown; readonly bytes?: string; readonly unreliable?: boolean };

/** An event, from the room provider to the engine, in order. */
export type RoomEvent =
  | { readonly type: "joined"; readonly room: string; readonly peer: number; readonly peers: ReadonlyArray<{ readonly peer: number; readonly name: string }> }
  | { readonly type: "peer_joined"; readonly peer: number; readonly name: string }
  | { readonly type: "peer_left"; readonly peer: number }
  | { readonly type: "message"; readonly from: number; readonly channel?: string; readonly data?: unknown; readonly bytes?: string }
  | { readonly type: "left"; readonly reason?: string }
  | { readonly type: "error"; readonly op: string; readonly code: string; readonly message: string };

/** The largest message a game sends, encoded (its JSON, or its bytes): og-net's MAX_MESSAGE. */
export const ROOM_MAX_MESSAGE = 64 * 1024;
/** The largest message on an engine channel (`og.`, a world snapshot for a late joiner): og-net's MAX_ENGINE_MESSAGE. */
export const ROOM_MAX_ENGINE_MESSAGE = 16 << 20;

/**
 * What carries og.platform's rooms (docs/design.md §2.8): the engine's ops in, its events out,
 * in order. Per sender, messages arrive in the order sent and a `peer_left` after the last of
 * them (lockstep rests on it); a `send` marked `unreliable` may be dropped or overtake others.
 * A refused op is an `error` event naming it.
 */
export interface RoomProvider {
  /** og.platform.roomInfo's `host`: `player` for the Player's rooms, `relay` for an `og relay`'s. */
  readonly host: string;
  /** Carry `ops` in order; `deliver` takes events, now or later (the latest `deliver` given). */
  handle(ops: readonly Record<string, unknown>[], deliver: (events: unknown[]) => void): void;
  /** Called when the page makes this the rooms' provider: events it hears before any op (a room joined at connect) go to `deliver`. */
  attach?(deliver: (events: unknown[]) => void): void;
}

// ── the page's choice ──────────────────────────────────────────────────

/**
 * The page's providers (`play({ provider })`, or `window.__OG_PROVIDER` before play.js loads):
 * each one named replaces the default for its part. `rooms` may be an `og relay` URL
 * (`wss://relay.example/`), carried by relayRooms.
 */
export interface Providers {
  readonly platform?: PlatformProvider;
  readonly rooms?: RoomProvider | string;
}

export { relayRooms } from "./web/relay-rooms.ts";

// ── conformance ────────────────────────────────────────────────────────

/** Arguments og.platform sends for each method, as the conformance check calls it. */
export const SAMPLE_ARGS: Readonly<Record<PlatformMethod, Record<string, unknown>>> = {
  "store.get": { key: "conformance" },
  "store.set": { key: "conformance", value: { n: 1 }, revision: 0, create: false },
  "store.list": {},
  "leaderboard.submit": { board: "conformance", score: 1, order: "desc" },
  "leaderboard.top": { board: "conformance", count: 10 },
  "leaderboard.around": { board: "conformance", count: 10 },
  telemetry: { name: "conformance", attrs: {} },
  "purchase.products": { ids: [] },
  "purchase.buy": { product: "conformance", quantity: 1 },
  "purchase.entitlements": {},
  "purchase.consume": { purchase: "conformance" },
  "ad.ready": { kind: "rewarded", placement: "conformance" },
  "ad.show": { kind: "rewarded", placement: "conformance" },
  "achievement.unlock": { id: "conformance" },
  "achievement.progress": { id: "conformance", percent: 50 },
  "achievement.list": {},
  "player.get": {},
  "friends.list": { count: 10 },
  "invite.send": { message: "", payload: null },
  "invite.list": {},
  share: { text: "conformance" },
  point: { reason: "conformance", amount: 1 },
  "ai.mark": { kind: "text", label: "conformance" },
};

/** What checkPlatformProvider found: `problems` is empty for a provider that keeps the contract. */
export interface ConformanceReport {
  readonly ok: boolean;
  readonly problems: readonly string[];
  /** Every answer, by method. */
  readonly answers: Readonly<Record<string, readonly PlatformAnswer[]>>;
}

const VOCABULARY: readonly string[] = [...CORE_METHODS, ...HOST_EXTENSIONS];
/** The method no provider answers: it must be `unavailable`. */
const UNKNOWN = "conformance.unknown";

function shapeProblems(method: string, a: unknown): string[] {
  const where = `${method}'s answer`;
  if (!a || typeof a !== "object") return [`${where} is not an object`];
  const r = a as Record<string, unknown>;
  const out: string[] = [];
  if (r["time"] !== undefined && typeof r["time"] !== "number") out.push(`${where}: time is UTC ms, a number`);
  if (r["ok"] === true) {
    if (!("data" in r)) out.push(`${where}: ok answers carry data (null for none)`);
    return out;
  }
  if (r["ok"] !== false) return [...out, `${where}: ok is true or false`];
  const e = r["error"] as Record<string, unknown> | undefined;
  if (!e || typeof e !== "object") return [...out, `${where}: a failure carries error { code, message, retryable }`];
  if (!(ERROR_CODES as readonly unknown[]).includes(e["code"])) out.push(`${where}: '${String(e["code"])}' is not an error code (${ERROR_CODES.join(", ")})`);
  if (typeof e["message"] !== "string") out.push(`${where}: error.message is a string`);
  if (typeof e["retryable"] !== "boolean") out.push(`${where}: error.retryable is a boolean`);
  if (e["revision"] !== undefined && typeof e["revision"] !== "number") out.push(`${where}: error.revision is a number`);
  return out;
}

/**
 * Checks a platform provider against this contract, for its author's tests: it lists methods
 * og.platform calls, without duplicates; every method is called once in one batch (each listed
 * one, each unlisted one, and one no provider knows); each listed method but `telemetry` is
 * answered exactly once, `telemetry` never; each unlisted one and the unknown one are answered
 * `unavailable`; every answer has the contract's shape; and a pause is resumed. The calls are
 * real: give it a provider over a throwaway store. `wait` is how long answers may take (ms).
 */
export async function checkPlatformProvider(provider: PlatformProvider, options: { wait?: number; settle?: number } = {}): Promise<ConformanceReport> {
  const problems: string[] = [];
  if (typeof provider.host !== "string" || provider.host === "") problems.push("host is a non-empty string");
  const caps = Array.isArray(provider.capabilities) ? provider.capabilities : [];
  if (!Array.isArray(provider.capabilities)) problems.push("capabilities is a list of method names");
  const seen = new Set<string>();
  for (const c of caps) {
    if (seen.has(c)) problems.push(`capabilities lists ${c} twice`);
    seen.add(c);
    if (!VOCABULARY.includes(c)) problems.push(`capabilities lists ${c}, which og.platform never calls (CORE_METHODS, HOST_EXTENSIONS)`);
  }
  const methods = [...VOCABULARY, UNKNOWN];
  const calls: PlatformCall[] = methods.map((method, i) => ({ request: i + 1, method, args: { ...(SAMPLE_ARGS[method as PlatformMethod] ?? {}) }, tick: 0 }));
  const byRequest = new Map(calls.map((c) => [c.request, c.method]));
  const answers: Record<string, PlatformAnswer[]> = {};
  let paused = 0;
  let tick = 0;
  const answer = (request: number, result: PlatformAnswer) => {
    const method = byRequest.get(request);
    if (method === undefined) return void problems.push(`answered request ${request}, which was never made`);
    (answers[method] ??= []).push(result);
  };
  const link: PlatformLink = {
    get tick() {
      return tick;
    },
    answer,
    notify(n) {
      if (!["purchase", "entitlements", "invite", "host"].includes(n?.kind)) problems.push(`a notice's kind '${String(n?.kind)}' is not purchase, entitlements, invite or host`);
    },
    pause: () => void paused++,
    resume: () => void paused--,
  };
  const expected = calls.filter((c) => !ONE_WAY_METHODS.includes(c.method as PlatformMethod));
  const done = () => expected.every((c) => (answers[c.method]?.length ?? 0) > 0);
  try {
    provider.handle(calls, answer, link);
  } catch (e) {
    problems.push(`handle threw: ${e instanceof Error ? e.message : String(e)}`);
  }
  const step = 5;
  const until = Date.now() + (options.wait ?? 2000);
  // Pumped as the shell pumps it: poll on every frame, with the engine's tick moving.
  while (!done() && Date.now() < until) {
    await new Promise((r) => setTimeout(r, step));
    tick++;
    provider.poll?.(link);
  }
  // Late duplicates show within the settle time.
  for (let t = 0; t < (options.settle ?? 50); t += step) {
    await new Promise((r) => setTimeout(r, step));
    tick++;
    provider.poll?.(link);
  }
  for (const c of calls) {
    const got = answers[c.method] ?? [];
    const listed = seen.has(c.method);
    if (ONE_WAY_METHODS.includes(c.method as PlatformMethod)) {
      if (got.length > 0) problems.push(`${c.method} is one-way: answered ${got.length} times, never expected`);
      continue;
    }
    if (got.length !== 1) problems.push(`${c.method}${listed ? "" : " (not listed)"} answered ${got.length} times, expected once`);
    for (const a of got) problems.push(...shapeProblems(c.method, a));
    if (!listed && got[0] && !(got[0].ok === false && got[0].error.code === "unavailable")) problems.push(`${c.method} is not listed, so it is answered unavailable, not ${got[0].ok ? "ok" : got[0].error.code}`);
  }
  if (paused !== 0) problems.push(`the game was left paused (${paused} pauses without a resume)`);
  return { ok: problems.length === 0, problems, answers };
}
