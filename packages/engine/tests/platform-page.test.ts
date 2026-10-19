// og.platform from a top-level engine page over the HTTP transport
// (shell/transport.ts; origingame/docs/player-protocol.md §The top-level
// engine page), against a stand-in play origin that answers
// `/__player/v2/<game>/<release>/{hello,call,report}` as the contract says:
// it redeems a hand-off code, mints a guest the first time a call needs a
// player, replaces a rejected credential, refuses a stale write with the
// revision it is at, and files reports.
//
// The ways this could fail, each asserted below: the hand-off code stays in
// the address bar, travels in a URL or with a bearer; an account credential
// lands in storage, or a stored account is resumed without a code; a page
// mints two guests, or keeps sending a rejected credential; a conflict loses
// its revision; an offline score is lost or answered as delivered, or is
// delivered out of order; a report goes out before start, or with ids the
// platform refuses; a call answers anything but protocol 2's result; the
// keyed store does not reach og.platform with the HTTP hello's capabilities;
// a report exceeds 20 messages or 64 KiB (the end's keepalive included), or
// one the play origin refuses (400) or cannot take (413) is sent again for
// ever and blocks the rest; a ready goes out after a crash.
import { expect, test } from "bun:test";
import { playerPlatform } from "../shell/platform.ts";
import { connectPage, type StorageLike, type TransportEnv } from "../shell/transport.ts";
import { memoryPersistence, type PlatformAnswer, type PlatformCall } from "../src/platform-local.ts";

const GAME = { id: "runner01", releaseId: "release_abc123" };
const UUID7 = /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
const CAPS = ["player.name", "player.info", "player.rename", "leaderboard.submit", "leaderboard.top", "leaderboard.around", "save.get", "save.read", "save.set", "store.get", "store.set", "store.list", "point.award", "ai.mark", "share.link", "room.ticket", "native.admission"];

type Seen = { what: string; url: string; bearer: string | null; body: Record<string, unknown>; keepalive: boolean };

/** A play origin in memory. `offline` makes every request fail like a lost network. */
function playOrigin(opts: { codes?: Record<string, { token: string; guest: boolean }>; reject?: Set<string> } = {}) {
  const seen: Seen[] = [];
  const minted: string[] = [];
  const store = new Map<string, { value: unknown; revision: number }>();
  const reports: Array<Record<string, unknown>> = [];
  const scores: number[] = [];
  const state = { offline: false, hang: false };
  const json = (status: number, body: unknown) => new Response(JSON.stringify(body), { status, headers: { "Content-Type": "application/json" } });
  const fetch = async (url: string, init: RequestInit): Promise<Response> => {
    if (state.offline) throw new TypeError("Failed to fetch");
    if (state.hang) return new Promise((_, reject) => init.signal?.addEventListener("abort", () => reject(new DOMException("aborted", "AbortError"))));
    const what = url.split("/").at(-1)!;
    const auth = (init.headers as Record<string, string>)["Authorization"] ?? null;
    const body = JSON.parse(String(init.body)) as Record<string, unknown>;
    seen.push({ what, url, bearer: auth, body, keepalive: init.keepalive === true });
    expect(url.startsWith(`/__player/v2/${GAME.id}/${GAME.releaseId}/`)).toBeTrue();
    expect(init.method).toBe("POST");
    expect(init.credentials).toBe("omit");
    let bearer = auth?.replace(/^Bearer /, "") ?? null;
    let player: { token: string; guest: boolean } | undefined;
    const need = () => {
      if (bearer && !opts.reject?.has(bearer)) return;
      bearer = `ogp_guest${minted.length + 1}`;
      minted.push(bearer);
      player = { token: bearer, guest: true };
    };
    if (what === "hello") {
      const code = body["handoff"];
      const redeemed = typeof code === "string" ? (opts.codes?.[code] ?? null) : undefined;
      return json(200, { og: "player", protocol: 2, kind: "hello", locale: "en", region: "global", capabilities: CAPS, started: false, paused: false,
        ...(redeemed === undefined ? {} : { player: redeemed }),
        page: { title: "Runner", cover: "https://cdn.test/c.jpg", backdrop: "https://cdn.test/c32.jpg", back: "https://portal.test/games/runner", portal: "https://portal.test" } });
    }
    if (what === "report") {
      need();
      reports.push(...(body["messages"] as Array<Record<string, unknown>>));
      return json(202, { accepted: (body["messages"] as unknown[]).length, rejected: 0, ...(player ? { player } : {}) });
    }
    need();
    const id = body["id"];
    const a = (body["args"] ?? {}) as Record<string, unknown>;
    const ok = (data: unknown) => json(200, { og: "player", protocol: 2, kind: "result", id, ok: true, data, ...(player ? { player } : {}) });
    switch (body["method"]) {
      case "store.get": {
        const s = store.get(String(a["key"]));
        return ok({ key: a["key"], value: s?.value ?? null, revision: s?.revision ?? 0 });
      }
      case "store.set": {
        const key = String(a["key"]);
        const have = store.get(key)?.revision ?? 0;
        if (typeof a["revision"] === "number" && a["revision"] !== have)
          return json(200, { og: "player", protocol: 2, kind: "result", id, ok: false, error: { code: "conflict", message: `${key} is at ${have}`, retryable: false, status: 409, revision: have } });
        store.set(key, { value: a["value"], revision: have + 1 });
        return ok({ key, revision: have + 1 });
      }
      case "leaderboard.submit":
        scores.push(Number(a["score"]));
        return ok({ ok: true, rank: scores.length, board: a["board"] ?? "main", best: Math.max(...scores) });
      default:
        return json(200, { og: "player", protocol: 2, kind: "result", id, ok: false, error: { code: "unavailable", message: "no", retryable: false } });
    }
  };
  return { fetch, seen, minted, reports, scores, state };
}

function memoryStorage(initial: Record<string, string> = {}): StorageLike & { map: Map<string, string> } {
  const map = new Map(Object.entries(initial));
  return { map, getItem: (k) => map.get(k) ?? null, setItem: (k, v) => void map.set(k, v), removeItem: (k) => void map.delete(k) };
}

let seed = 1;
function env(origin: ReturnType<typeof playOrigin>, storage: StorageLike, hash = ""): TransportEnv & { urls: string[] } {
  const urls: string[] = [];
  return {
    urls,
    fetch: origin.fetch,
    storage,
    location: { hash, pathname: `/play/${GAME.id}/${GAME.releaseId}/`, search: "?level=2" },
    replaceUrl: (u) => urls.push(u),
    now: () => 1_760_870_400_000 + seed * 1000,
    clock: () => 812,
    random: (n) => Uint8Array.from({ length: n }, (_, i) => (i * 37 + seed++) & 0xff),
  };
}

test("a hand-off code leaves the address at once, travels only in the hello's body, and an account stays in memory", async () => {
  const origin = playOrigin({ codes: { [`c${"x".repeat(42)}`]: { token: "ogp_account", guest: false } } });
  const storage = memoryStorage({ og_token_runner01: "ogp_old", og_portal_runner01: "1" });
  const e = env(origin, storage, `#og-h=c${"x".repeat(42)}&lvl=3`);
  const page = connectPage(GAME, e);
  // Removed before any request: the rest of the fragment stays the game's.
  expect(e.urls).toEqual([`/play/${GAME.id}/${GAME.releaseId}/?level=2#lvl=3`]);
  expect(await page.hello).toBeTrue();
  const hello = origin.seen[0]!;
  expect(hello.what).toBe("hello");
  expect(hello.body).toEqual({ og: "player", protocol: 2, kind: "hello", substrate: "origin-engine", handoff: `c${"x".repeat(42)}` });
  expect(hello.bearer).toBeNull();
  expect(hello.url).not.toContain("og-h");
  expect(page.capabilities).toEqual(CAPS);
  expect(page.page?.back).toBe("https://portal.test/games/runner");
  // The account is in memory only, behind the Player's marker.
  expect(page.credential).toBe("ogp_account");
  expect(storage.map.has("og_token_runner01")).toBeFalse();
  expect(storage.map.get("og_portal_runner01")).toBe("1");
  await page.request("store.get", { key: "best" });
  expect(origin.seen[1]!.bearer).toBe("Bearer ogp_account");
});

test("without a code a stored account is never resumed, a stored guest is, and none mints exactly one guest", async () => {
  const origin = playOrigin();
  const marked = memoryStorage({ og_token_runner01: "ogp_account", og_portal_runner01: "1" });
  const a = connectPage(GAME, env(origin, marked));
  await a.hello;
  expect(origin.seen[0]!.bearer).toBeNull();
  expect([...marked.map.keys()].filter((k) => k.startsWith("og_token") || k.startsWith("og_portal"))).toEqual([]);
  // Two calls at once with no credential: one guest, adopted and stored, sent by both.
  const [r1, r2] = await Promise.all([a.request("store.get", { key: "x" }), a.request("store.get", { key: "y" })]);
  expect(r1.ok && r2.ok).toBeTrue();
  expect(origin.minted).toEqual(["ogp_guest1"]);
  expect(marked.map.get("og_token_runner01")).toBe("ogp_guest1");
  expect(origin.seen.slice(1).map((s) => s.bearer)).toEqual([null, "Bearer ogp_guest1"]);

  const guestOrigin = playOrigin();
  const guest = memoryStorage({ og_token_runner01: "ogp_kept" });
  const b = connectPage(GAME, env(guestOrigin, guest));
  await b.request("store.get", { key: "x" });
  expect(guestOrigin.seen.map((s) => s.bearer)).toEqual(["Bearer ogp_kept", "Bearer ogp_kept"]);
  expect(guestOrigin.minted).toEqual([]);
});

test("a rejected credential is replaced by the guest the play origin mints, and a conflict keeps its revision", async () => {
  const origin = playOrigin({ reject: new Set(["ogp_stale"]) });
  const storage = memoryStorage({ og_token_runner01: "ogp_stale" });
  const page = connectPage(GAME, env(origin, storage));
  const first = await page.request("store.set", { key: "k", value: 1, revision: 0 });
  expect(first).toEqual({ ok: true, data: { key: "k", revision: 1 } });
  expect(page.credential).toBe("ogp_guest1");
  expect(storage.map.get("og_token_runner01")).toBe("ogp_guest1");
  const stale = await page.request("store.set", { key: "k", value: 2, revision: 0 });
  expect(stale).toEqual({ ok: false, error: { code: "conflict", message: "k is at 1", retryable: false, status: 409, revision: 1 } });
  expect(origin.seen.at(-1)!.bearer).toBe("Bearer ogp_guest1");
});

test("offline, a score is kept in the Player's outbox and answered queued, then delivered in order; a timeout is retryable", async () => {
  const origin = playOrigin();
  const storage = memoryStorage({ og_token_runner01: "ogp_g" });
  const page = connectPage(GAME, env(origin, storage));
  await page.hello;
  origin.state.offline = true;
  expect(await page.request("leaderboard.submit", { score: 10, board: "main", order: "desc" })).toEqual({ ok: true, data: { ok: true, rank: null, queued: true } });
  expect(await page.request("leaderboard.submit", { score: 20, board: "main", order: "desc" })).toEqual({ ok: true, data: { ok: true, rank: null, queued: true } });
  const box = JSON.parse(storage.map.get("og_outbox_runner01")!) as { scores: Array<{ score: number }> };
  expect(box.scores.map((s) => s.score)).toEqual([10, 20]);
  // A keyed-store write is never queued: offline is an offline error.
  const offline = await page.request("store.set", { key: "k", value: 1 });
  expect(!offline.ok && [offline.error.code, offline.error.retryable]).toEqual(["offline", true]);
  origin.state.offline = false;
  // The next success delivers the outbox, in order, after itself.
  expect((await page.request("leaderboard.submit", { score: 30, board: "main" })).ok).toBeTrue();
  for (let i = 0; i < 50 && storage.map.has("og_outbox_runner01"); i++) await new Promise((r) => setTimeout(r, 1));
  expect(origin.scores).toEqual([30, 10, 20]);
  expect(storage.map.has("og_outbox_runner01")).toBeFalse();

  const slow = playOrigin();
  const timers: Array<() => void> = [];
  const e = { ...env(slow, memoryStorage()), setTimer: (cb: () => void) => (timers.push(cb), timers.length), clearTimer: () => {} };
  const p = connectPage(GAME, e);
  slow.state.hang = true;
  const hello = p.hello;
  await new Promise((r) => setTimeout(r, 1));
  timers.shift()!();
  expect(await hello).toBeFalse();
  const call = p.request("store.get", { key: "k" });
  await new Promise((r) => setTimeout(r, 1));
  timers.at(-1)!();
  const r = await call;
  expect(!r.ok && [r.error.code, r.error.retryable]).toEqual(["timeout", true]);
});

test("nothing is reported before start; then start, what was held and the rest, with UUID v7 ids, one session and the device", async () => {
  const origin = playOrigin();
  const storage = memoryStorage({ og_token_runner01: "ogp_g" });
  const page = connectPage(GAME, env(origin, storage));
  page.loading(0.5);
  page.ready();
  page.sample({ renderer: "webgpu", timings: { firstFrame: 700 } });
  await page.flush();
  expect(origin.seen.filter((s) => s.what === "report")).toEqual([]);
  page.start();
  page.event("game.custom.boss.defeated", { boss: 1 });
  await page.flush();
  await page.flush();
  expect(origin.reports.map((m) => m["kind"])).toEqual(["start", "ready", "sample", "event"]);
  for (const m of origin.reports) {
    expect(m["og"]).toBe("player");
    expect(m["protocol"]).toBe(2);
    expect(String(m["id"])).toMatch(UUID7);
    expect(typeof m["at"]).toBe("number");
    expect(m["ms"]).toBe(812);
  }
  expect(new Set(origin.reports.map((m) => m["id"])).size).toBe(4);
  const bodies = origin.seen.filter((s) => s.what === "report").map((s) => s.body);
  expect(bodies.every((b) => /^game_session_[0-9a-f]{32}$/.test(String(b["session"])) && b["session"] === bodies[0]!["session"])).toBeTrue();
  const device = JSON.parse(storage.map.get("og_device_runner01")!) as { id: string; createdAt: number };
  expect(device.id).toMatch(/^device_[0-9a-f]{32}$/);
  expect(bodies.every((b) => b["device"] === device.id)).toBeTrue();
  // The same browser is the same device on the next load; the end goes out with keepalive.
  const next = connectPage(GAME, env(origin, storage));
  next.start();
  next.end({ ready: true, failed: false });
  await new Promise((r) => setTimeout(r, 5));
  const end = origin.seen.filter((s) => s.what === "report").at(-1)!;
  expect(end.keepalive).toBeTrue();
  expect(end.body["device"]).toBe(device.id);
  const last = (end.body["messages"] as Array<Record<string, unknown>>).at(-1)!;
  expect([last["kind"], last["ready"], last["failed"], typeof last["durationMs"]]).toEqual(["end", true, false, "number"]);
});

test("og.platform's keyed store, telemetry and scores through the page transport, as the HTTP hello lists them", async () => {
  const origin = playOrigin();
  const page = connectPage(GAME, env(origin, memoryStorage({ og_token_runner01: "ogp_g" })));
  await page.hello;
  page.start();
  const host = playerPlatform(page, memoryPersistence());
  expect(host.capabilities).toEqual(["store.get", "store.set", "store.list", "telemetry", "leaderboard.submit", "leaderboard.top", "leaderboard.around", "point", "ai.mark"]);
  const got = new Map<number, PlatformAnswer>();
  const calls: PlatformCall[] = [
    { request: 1, method: "store.set", args: { key: "best", value: 7, create: true }, tick: 1 },
    { request: 2, method: "store.get", args: { key: "best" }, tick: 1 },
    { request: 3, method: "telemetry", args: { name: "level.done", attrs: { n: 1 } }, tick: 1 },
    { request: 4, method: "leaderboard.submit", args: { board: "main", score: 7, order: "desc" }, tick: 1 },
  ];
  host.handle(calls, (r, a) => got.set(r, a));
  for (let i = 0; i < 100 && got.size < 3; i++) await new Promise((r) => setTimeout(r, 1));
  expect(got.get(1)).toEqual({ ok: true, data: { key: "best", revision: 1 } });
  expect(got.get(2)).toEqual({ ok: true, data: { key: "best", value: 7, revision: 1 } });
  expect(got.get(4)).toEqual({ ok: true, data: { board: "main", rank: 1, best: 7 } });
  const sent = origin.seen.filter((s) => s.what === "call").map((s) => s.body);
  expect([sent[0]!["og"], sent[0]!["protocol"], sent[0]!["kind"], sent[0]!["method"], sent[0]!["args"]]).toEqual(["player", 2, "call", "store.set", { key: "best", value: 7, revision: 0 }]);
  await page.flush();
  const event = origin.reports.find((m) => m["kind"] === "event")!;
  expect([event["name"], event["attrs"]]).toEqual(["game.custom.level.done", { n: 1 }]);
});

test("reports stay within 20 messages and 64 KiB; one answered 400 or 413 is dropped and the rest still arrive, the end included", async () => {
  const origin = playOrigin();
  const LIMIT = 64 * 1024;
  const sent: Array<{ count: number; bytes: number; status: number; keepalive: boolean; last: unknown }> = [];
  // The play origin's report limits: more than 20 messages or an invalid one is 400, a body over 64 KiB is 413.
  const fetch = async (url: string, init: RequestInit): Promise<Response> => {
    if (!url.endsWith("/report")) return origin.fetch(url, init);
    if (origin.state.offline) throw new TypeError("Failed to fetch");
    const bytes = new TextEncoder().encode(String(init.body)).length;
    const messages = (JSON.parse(String(init.body)) as { messages: Array<Record<string, unknown>> }).messages;
    const refused = bytes > LIMIT ? 413 : messages.length > 20 || messages.some((m) => m["name"] === "game.custom.poison") ? 400 : 0;
    sent.push({ count: messages.length, bytes, status: refused || 202, keepalive: init.keepalive === true, last: messages.at(-1)?.["kind"] });
    if (refused) return new Response(JSON.stringify({ error: { code: refused === 413 ? "too_large" : "invalid" } }), { status: refused });
    return origin.fetch(url, init);
  };
  const pad = "x".repeat(6000);
  const page = connectPage(GAME, { ...env(origin, memoryStorage({ og_token_runner01: "ogp_g" })), fetch });
  page.start();
  await page.flush();
  page.event("game.custom.poison");
  page.event("game.custom.lost", { n: 1 });
  await page.flush();
  page.event("game.custom.huge", { pad: "x".repeat(70_000) });
  for (let n = 0; n < 30; n++) page.event("game.custom.big", { n, pad });
  await page.flush();
  for (let n = 0; n < 25; n++) page.event("game.custom.small", { n });
  await page.flush();
  const names = origin.reports.map((m) => String(m["name"] ?? m["kind"]));
  expect(names).toEqual(["start", ...Array(30).fill("game.custom.big"), ...Array(25).fill("game.custom.small")]);
  expect(origin.reports.filter((m) => m["name"] === "game.custom.big").map((m) => (m["attrs"] as { n: number }).n)).toEqual([...Array(30).keys()]);
  expect(sent.filter((r) => r.status === 400).map((r) => r.count)).toEqual([2]);
  expect(sent.filter((r) => r.status === 413).map((r) => r.count)).toEqual([1]);
  expect(sent.every((r) => r.status !== 202 || (r.count <= 20 && r.bytes <= LIMIT))).toBeTrue();
  const before = sent.length;
  await page.flush();
  expect(sent.length).toBe(before);

  // Unsent when the page goes: the end's keepalive report is the newest messages that fit, the end last.
  const going = connectPage(GAME, { ...env(origin, memoryStorage({ og_token_runner01: "ogp_g" })), fetch });
  going.start();
  for (let n = 0; n < 25; n++) going.event("game.custom.big", { n, pad });
  origin.state.offline = true;
  await going.flush();
  origin.state.offline = false;
  going.end({ ready: true, failed: false });
  await new Promise((r) => setTimeout(r, 5));
  const end = sent.at(-1)!;
  expect([end.keepalive, end.status, end.last]).toEqual([true, 202, "end"]);
  expect(end.count).toBeGreaterThan(1);
  expect(end.bytes <= LIMIT).toBeTrue();
});

test("after a crash the session is over: a late ready is not reported", async () => {
  const origin = playOrigin();
  const page = connectPage(GAME, env(origin, memoryStorage({ og_token_runner01: "ogp_g" })));
  page.start();
  page.crash("EngineCrash", "unreachable executed");
  page.ready();
  await page.flush();
  await page.flush();
  expect(origin.reports.map((m) => m["kind"])).toEqual(["start", "crash"]);
});
