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

// The vocabulary is the IDL's (crates/og-api/api/platform.ogt), generated into
// ./generated/vocabulary.ts by `og api gen` and re-exported here: a method or
// an error code the IDL gains is this module's without a hand edit.

import { CORE_ONLY_METHODS, ERROR_CODES, HOST_EXTENSIONS, RETRYABLE_CODES, SERVICE_METHODS } from "./generated/vocabulary.ts";

/** The methods any host answers from its own storage: the store, the leaderboards, telemetry and the catalog. */
export { CORE_ONLY_METHODS };

/** Purchases, ads, achievements, the player, friends, invites and share (docs/platform-asks.md #19). */
export { SERVICE_METHODS };

/** The methods any store or portal can answer: the core vocabulary — the core-only methods plus the service methods. */
export const CORE_METHODS = [...CORE_ONLY_METHODS, ...SERVICE_METHODS] as const;

/** Methods one host defines: `point` (Origin Point) and `ai.mark` are OriginGame's. */
export { HOST_EXTENSIONS };

export type CoreMethod = (typeof CORE_METHODS)[number];
export type HostExtension = (typeof HOST_EXTENSIONS)[number];
/** Every method og.platform calls (og.platform.info's capabilities name these). */
export type PlatformMethod = CoreMethod | HostExtension;

/** The one-way method: never answered. */
export const ONE_WAY_METHODS: readonly PlatformMethod[] = ["telemetry"];

/** Every error code an answer may carry (crates/og-api/api/platform.ogt). */
export { ERROR_CODES };
export type ErrorCode = (typeof ERROR_CODES)[number];

/** The codes whose call may succeed later (`retryable: true`). */
export { RETRYABLE_CODES };

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
  /**
   * `host`: a message from the page around the game (`{ name, ... }`), whose meaning the game and that page agree.
   * `catalog`: a content catalog may have a new version (`{ catalog, version, manifest }`). Best effort: games poll
   * `catalog.head`, so a provider that cannot push (a top-level page) never sends it.
   */
  readonly kind: "purchase" | "entitlements" | "invite" | "catalog" | "host";
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
  /**
   * Where og.load.data reads the game's content catalogs: `base()` is an https directory (http only on
   * localhost) holding `<catalog>/…`, one for every catalog of the game, read once when the engine starts.
   * Absent: the page's own content base (`og build --content`); absent there too, data loads fail `unavailable`.
   */
  readonly content?: { base(): string };
}

// A store package's provider (packages/engine/src/hermes/platform.ts): a
// module the package bundles with the game, over the store's native library.

/** The provider library, as the factory calls it. */
export interface ShellNative {
  /** Calls `og_provider_call(method, JSON.stringify(args))`; its answer parsed, `null` for none. */
  call(method: string, args?: unknown): unknown;
}

/** What the shell hands a bundled provider. */
export interface ShellProviderContext {
  /** The provider library, `null` when the package has none. */
  readonly native: ShellNative | null;
  /** The package's title (OG_TITLE). */
  readonly game: string;
  /** shell.env's pairs: a provider reads its own keys (a store's app id, say). */
  readonly env: Readonly<Record<string, string | undefined>>;
  /** The local document's provider, for what the store does not answer. */
  readonly local: PlatformProvider;
}

/** A store provider module's default export. */
export type ShellProviderFactory = (context: ShellProviderContext) => PlatformProvider;

/** A successful answer. */
export const ok = (data: unknown): PlatformAnswer => ({ ok: true, data });

/** A failed answer with `code`; `retryable` follows the code. */
export const failure = (code: string, message: string): PlatformAnswer => ({ ok: false, error: { code, message, retryable: retryable(code) } });

/** The answer to a method this provider does not answer. */
export const unavailable = (method: string, host = "this host"): PlatformAnswer => failure("unavailable", `${host} answers no ${method}`);

// ── content catalogs ───────────────────────────────────────────────────

/** `catalog.head`'s answer data: a catalog's current version and its manifest's integrity. */
export interface CatalogHead {
  readonly catalog: string;
  readonly version: string;
  /** The version manifest's integrity, `sha384-<base64>`. */
  readonly manifest: string;
  /** When it was published, UTC ms. */
  readonly published: number;
}

const CATALOG_NAME = /^[A-Za-z0-9._-]{1,64}$/;
const CATALOG_VERSION = /^[A-Za-z0-9._-]{1,128}$/;
const SHA384 = /^sha384-[A-Za-z0-9+/]{64}$/;

/** What is wrong with a `catalog.head` answer's data for `catalog`, or null when it has the contract's shape. */
export function catalogHeadProblem(catalog: string, data: unknown): string | null {
  const d = data as Partial<Record<keyof CatalogHead, unknown>> | null;
  if (!d || typeof d !== "object") return "the head is not an object { catalog, version, manifest, published }";
  if (d.catalog !== catalog) return `the head names catalog '${String(d.catalog)}', not '${catalog}'`;
  if (typeof d.version !== "string" || !CATALOG_VERSION.test(d.version)) return `the head's version '${String(d.version)}' is not 1 to 128 of letters, digits, '.', '_' and '-'`;
  if (typeof d.manifest !== "string" || !SHA384.test(d.manifest)) return "the head's manifest is not an integrity, sha384-<base64>";
  if (typeof d.published !== "number" || !Number.isFinite(d.published)) return "the head's published is not UTC ms, a number";
  return null;
}

/**
 * `catalog.head`'s answer from a `<base>/<catalog>/head.json` file's text, as the static layout serves it:
 * null text is `not_found`, a file without the answer's shape `failed`.
 */
export function catalogHeadAnswer(catalog: string, text: string | null): PlatformAnswer {
  if (text === null) return failure("not_found", `no catalog '${catalog}' is published (no ${catalog}/head.json)`);
  let data: unknown;
  try {
    data = JSON.parse(text);
  } catch (e) {
    return failure("failed", `${catalog}/head.json is not JSON: ${e instanceof Error ? e.message : String(e)}`);
  }
  const problem = catalogHeadProblem(catalog, data);
  if (problem) return failure("failed", `${catalog}/head.json: ${problem}`);
  const h = data as CatalogHead;
  return ok({ catalog: h.catalog, version: h.version, manifest: h.manifest, published: h.published });
}

/** Reads `<base>/<catalog>/head.json`: its text, or null when there is none (404). Throws for a network failure. */
export type CatalogRead = (url: string) => string | null | Promise<string | null>;

/** The browser's read of a head file: never cached, a 404 is none. */
export const fetchCatalog: CatalogRead = async (url) => {
  // Through globalThis: a game's type check has no fetch global.
  const get = (globalThis as unknown as { fetch(url: string, init: { cache: string }): Promise<{ ok: boolean; status: number; text(): Promise<string> }> }).fetch;
  const r = await get(url, { cache: "no-store" });
  if (r.status === 404) return null;
  if (!r.ok) throw new Error(`HTTP ${r.status}`);
  return r.text();
};

/**
 * A provider over the static catalog layout (`<base>/<catalog>/head.json` and the files beside it): it answers
 * `catalog.head` by reading the head file, never cached, names `base` as the content base, and hands every other
 * call to `inner` (none: `unavailable`). For a self-hosted page and for tests; a platform may serve the same layout.
 */
export function staticCatalog(base: string, options: { readonly inner?: PlatformProvider; readonly read?: CatalogRead } = {}): PlatformProvider {
  const root = base.replace(/\/+$/, "");
  const inner = options.inner;
  const read = options.read ?? fetchCatalog;
  const head = (request: number, catalog: string, answer: (request: number, result: PlatformAnswer) => void) => {
    const settle = (text: string | null) => answer(request, catalogHeadAnswer(catalog, text));
    const lost = (e: unknown) => answer(request, failure("offline", `${catalog}/head.json: ${e instanceof Error ? e.message : String(e)}`));
    try {
      const text = read(`${root}/${catalog}/head.json`);
      if (text instanceof Promise) text.then(settle, lost);
      else settle(text);
    } catch (e) {
      lost(e);
    }
  };
  return {
    host: inner?.host ?? "static",
    capabilities: [...(inner?.capabilities ?? []).filter((c) => c !== "catalog.head"), "catalog.head"],
    ...(inner?.time === undefined ? {} : { time: inner.time }),
    content: { base: () => root },
    handle(calls, answer, link) {
      const rest: PlatformCall[] = [];
      for (const c of calls) {
        if (c.method === "catalog.head") head(c.request, String(c.args["catalog"] ?? ""), answer);
        else rest.push(c);
      }
      if (rest.length === 0) return;
      if (inner) inner.handle(rest, answer, link);
      else for (const c of rest) if (c.method !== "telemetry") answer(c.request, unavailable(c.method, "a static catalog"));
    },
    ...(inner?.poll ? { poll: (link: PlatformLink) => inner.poll!(link) } : {}),
  };
}

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

// ── shared-world sessions ──────────────────────────────────────────────

/** What og.shared.join (or a reconnect) asks the session provider for: a connection to this world. */
export interface SessionRequest {
  /** og.SharedJoinDesc.world. */
  readonly world: string;
  /** Whether this is a lost session's reconnect (a token endpoint signs a new token for it). */
  readonly reconnect: boolean;
}

/**
 * The same world server over WebTransport (`og world-server --wt-listen`): its `https://` URL and,
 * for a self-signed certificate (`og world-server cert`), the SHA-256 hashes the browser checks it
 * against (64 hex digits each; such a certificate lasts 14 days at most). Snapshots then cross as
 * datagrams, which a lost packet does not hold up.
 */
export interface SessionWebTransport {
  readonly url: string;
  readonly certHashes?: readonly string[];
}

/**
 * A provider's answer: where the world server is (`ws://` or `wss://`) and the token the engine
 * sends it as it is and never reads (none for a server started `--auth none --dev`), or why
 * not. With `webtransport`, a browser that has WebTransport tries it first and falls back to
 * `url` when it fails to open. A failure with a retryable code (`offline`, `unavailable`,
 * `rate_limited`, `timeout`) is tried again on the reconnect schedule; any other ends the
 * session (og.SharedLeft).
 */
export type SessionAnswer =
  | { readonly ok: true; readonly url: string; readonly token?: string; readonly webtransport?: SessionWebTransport }
  | { readonly ok: false; readonly error: { readonly code: string; readonly message: string } };

/** Whether `wt` is a SessionWebTransport the engine can dial (an https:// URL, hashes of 64 hex digits). */
function validWebTransport(wt: unknown): wt is SessionWebTransport {
  if (typeof wt !== "object" || wt === null) return false;
  const { url, certHashes } = wt as { url?: unknown; certHashes?: unknown };
  return typeof url === "string" && /^https:\/\//.test(url) &&
    (certHashes === undefined || (Array.isArray(certHashes) && certHashes.every((h) => typeof h === "string" && /^[0-9a-fA-F]{64}$/.test(h))));
}

/**
 * What hosts og.shared's sessions (docs/design.md §2.12): asked once per connection, it names
 * the world server and a token for it. The engine opens the connection itself (a WebSocket from
 * its Worker) and speaks the world server's protocol over it; the provider owns every URL and
 * credential, so game code is the same under a self-hosted server and a platform's.
 */
export interface SessionProvider {
  /** og.shared.status's `host`: `static` for a fixed endpoint, `endpoint` for a token endpoint's, a platform's own name. */
  readonly host: string;
  connect(request: SessionRequest): Promise<SessionAnswer>;
}

/** A fixed world server and token (a self-hosted `og world-server`, a test): what `og run --shared` is natively. */
export function staticSession(grant: { readonly url: string; readonly token?: string; readonly webtransport?: SessionWebTransport }): SessionProvider {
  if (!/^wss?:\/\//.test(grant.url)) throw new Error(`staticSession: ${grant.url} is not a ws:// or wss:// URL`);
  if (grant.webtransport !== undefined && !validWebTransport(grant.webtransport)) {
    throw new Error("staticSession: webtransport is not { url: \"https://…\", certHashes?: [64 hex digits] }");
  }
  const answer: SessionAnswer = {
    ok: true,
    url: grant.url,
    ...(grant.token !== undefined ? { token: grant.token } : {}),
    ...(grant.webtransport !== undefined ? { webtransport: grant.webtransport } : {}),
  };
  return { host: "static", connect: () => Promise.resolve(answer) };
}

/** The page's fetch, written out (as relayRooms' SocketFactory): a game's type check has no DOM. */
const pageFetch = (url: string, init: { method: string; credentials: string; headers: Record<string, string>; body: string }) =>
  (globalThis as unknown as { fetch: (u: string, i: typeof init) => Promise<{ readonly status: number; readonly ok: boolean; json(): Promise<unknown> }> }).fetch(url, init);

/**
 * A game's own token endpoint (self-hosting, docs/web-export.md §Providers): each connection
 * POSTs the request as JSON to `url` with the page's cookies and takes `{ url, token }` back
 * (and `webtransport`, as SessionAnswer's, when the server serves it). The
 * endpoint signs a short-lived token for its own player with the world server's secret
 * (`og world-server --auth hs256`); a 401 or 403 ends the session as `unauthorized`, a network
 * failure or a 5xx is tried again.
 */
export function tokenEndpoint(url: string, init: { readonly headers?: Readonly<Record<string, string>> } = {}): SessionProvider {
  return {
    host: "endpoint",
    async connect(request) {
      let res: { readonly status: number; readonly ok: boolean; json(): Promise<unknown> };
      try {
        res = await pageFetch(url, { method: "POST", credentials: "include", headers: { "content-type": "application/json", ...init.headers }, body: JSON.stringify(request) });
      } catch (e) {
        return { ok: false, error: { code: "offline", message: `the token endpoint did not answer: ${e instanceof Error ? e.message : String(e)}` } };
      }
      const code = res.status === 401 || res.status === 403 ? "unauthorized" : res.status === 429 ? "rate_limited" : res.status >= 500 ? "unavailable" : res.ok ? "" : "failed";
      if (code !== "") return { ok: false, error: { code, message: `the token endpoint answered ${res.status}` } };
      const body = (await res.json().catch(() => null)) as { url?: unknown; token?: unknown; webtransport?: unknown } | null;
      if (
        !body || typeof body.url !== "string" || !/^wss?:\/\//.test(body.url) || (body.token !== undefined && typeof body.token !== "string") ||
        (body.webtransport !== undefined && !validWebTransport(body.webtransport))
      ) {
        return { ok: false, error: { code: "invalid", message: "the token endpoint's answer is not { url: \"wss://…\", token, webtransport? }" } };
      }
      return {
        ok: true,
        url: body.url,
        ...(typeof body.token === "string" ? { token: body.token } : {}),
        ...(body.webtransport !== undefined ? { webtransport: body.webtransport as SessionWebTransport } : {}),
      };
    },
  };
}

// ── the page's choice ──────────────────────────────────────────────────

/**
 * The page's providers (`play({ provider })`, or `window.__OG_PROVIDER` before play.js loads):
 * each one named replaces the default for its part. `rooms` may be an `og relay` URL
 * (`wss://relay.example/`), carried by relayRooms; `sessions` may be a world server's URL,
 * a staticSession without a token.
 */
export interface Providers {
  readonly platform?: PlatformProvider;
  readonly rooms?: RoomProvider | string;
  readonly sessions?: SessionProvider | string;
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
  "catalog.head": { catalog: "conformance" },
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
    const head = method === "catalog.head" ? catalogHeadProblem("conformance", r["data"]) : null;
    if (head) out.push(`${where}: ${head}`);
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

/** Resolves after `ms`, through the runtime's timer: a game's type check has no timer globals. */
const sleep = (ms: number) =>
  new Promise<void>((r) => (globalThis as unknown as { setTimeout(f: () => void, ms: number): unknown }).setTimeout(r, ms));

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
      if (!["purchase", "entitlements", "invite", "catalog", "host"].includes(n?.kind)) problems.push(`a notice's kind '${String(n?.kind)}' is not purchase, entitlements, invite, catalog or host`);
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
    await sleep(step);
    tick++;
    provider.poll?.(link);
  }
  // Late duplicates show within the settle time.
  for (let t = 0; t < (options.settle ?? 50); t += step) {
    await sleep(step);
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
