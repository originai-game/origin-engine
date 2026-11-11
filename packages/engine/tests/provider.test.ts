// The provider contract (src/provider.ts, docs/design.md §2.8 Providers):
// its vocabulary and error codes agree with the IDL's (platform.ogt) and with
// the lists its guide gives a game author (guides/saves.md), and the
// providers in the tree keep it under checkPlatformProvider: the local
// document with and without its fake, no platform, and the Player bridge
// (shell/platform.ts) over a stand-in Player with every method and with
// few. The checker itself catches a provider that answers twice, never, or
// with the wrong shape. The page's choice of providers (pageProviders) is
// checked, and a publish build never fakes a purchase through it.
import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { checkPlatformProvider, CORE_METHODS, ERROR_CODES, HOST_EXTENSIONS, PROVIDER_VERSION, RETRYABLE_CODES, SERVICE_METHODS, unavailable, type PlatformProvider } from "../src/provider.ts";
import { fail, LOCAL_CAPABILITIES, localPlatform, memoryPersistence, noPlatform } from "../src/platform-local.ts";
import { pageProviders, playerPlatform, type PlayerLink } from "../shell/platform.ts";
import type { PlayerResult } from "../shell/player.ts";

const repo = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const idl = readFileSync(join(repo, "crates/og-api/api/platform.ogt"), "utf8");
// The guide wraps its lines: one space for each run of whitespace.
const guide = readFileSync(join(repo, "crates/og-api/guides/saves.md"), "utf8").replace(/\s+/g, " ");

/** The words of an IDL doc sentence: `a, b and c` → [a, b, c]. */
const words = (text: string) => text.split(/,\s*|\s+and\s+|\s+or\s+/).map((w) => w.trim().replace(/\s*\(.*$/, "")).filter(Boolean);

test("the vocabulary is the guide's: PlatformInfo.capabilities lists exactly CORE_METHODS and HOST_EXTENSIONS", () => {
  // A method name has a dot inside it; the list ends at the first one before a space.
  const m = /The methods a host may answer, as PlatformInfo\.capabilities lists them: (.+?)\. /.exec(guide);
  expect(m).not.toBeNull();
  const listed = words(m![1]!).sort();
  expect(listed).toEqual([...CORE_METHODS, ...HOST_EXTENSIONS].sort());
  expect(new Set([...CORE_METHODS, ...HOST_EXTENSIONS]).size).toBe(CORE_METHODS.length + HOST_EXTENSIONS.length);
  expect(SERVICE_METHODS.every((s) => (CORE_METHODS as readonly string[]).includes(s))).toBeTrue();
  expect(PROVIDER_VERSION).toBe(1);
});

test("the error codes are the IDL's and the guide's, the retryable ones the guide's", () => {
  const m = /Error codes: (.+?)\. `retryable` is true for ([^.]+)\./.exec(guide);
  expect(m).not.toBeNull();
  // "cancelled (the player backed out ...) and no_fill (no ad to show now)": the codes before each note.
  const codes = m![1]!.replace(/\([^)]*\)/g, "");
  expect(words(codes).sort()).toEqual([...ERROR_CODES].sort());
  expect(words(m![2]!).sort()).toEqual([...RETRYABLE_CODES].sort());
  const field = /field code String "([^"]+)\."/.exec(idl)!;
  expect(words(field[1]!).sort()).toEqual([...ERROR_CODES].sort());
  // The local platform's failures follow it (no_fill is retryable, as og.platform says).
  expect(fail("no_fill", "x")).toMatchObject({ ok: false, error: { code: "no_fill", retryable: true } });
  expect(unavailable("share")).toEqual({ ok: false, error: { code: "unavailable", message: "this host answers no share", retryable: true } });
});

test("the local platform keeps the contract, without its fake and with it (an ad pauses and resumes)", async () => {
  const plain = localPlatform(memoryPersistence());
  const r = await checkPlatformProvider(plain);
  expect(r.problems).toEqual([]);
  expect(plain.capabilities).toEqual(LOCAL_CAPABILITIES);
  // An unknown method is unavailable, as on every provider.
  expect(r.answers["conformance.unknown"]?.[0]).toMatchObject({ ok: false, error: { code: "unavailable" } });
  expect(r.answers["purchase.buy"]?.[0]).toMatchObject({ ok: false, error: { code: "unavailable" } });
  const faked = await checkPlatformProvider(localPlatform(memoryPersistence(), { unavailable: ["share"] }));
  expect(faked.problems).toEqual([]);
  expect(faked.answers["ad.show"]?.[0]).toMatchObject({ ok: true, data: { outcome: "completed" } });
  expect(faked.answers["share"]?.[0]).toMatchObject({ ok: false, error: { code: "unavailable" } });
  expect((await checkPlatformProvider(noPlatform)).problems).toEqual([]);
});

/** A stand-in Player: every method it lists answers ok with protocol 2's shapes, any other `unavailable`. */
function player(capabilities: readonly string[]): PlayerLink {
  const err = (code: string, message: string): PlayerResult => ({ ok: false, error: { code, message, retryable: code === "unavailable" } });
  const data = (method: string, a: Record<string, unknown>): unknown => {
    switch (method) {
      case "store.get":
        return { key: a["key"], value: null, revision: 0 };
      case "store.set":
        return { key: a["key"], revision: 1 };
      case "store.list":
        return { keys: [] };
      case "leaderboard.submit":
        return { ok: true, rank: 1, board: a["board"], best: a["score"] };
      case "leaderboard.top":
        return { board: a["board"], sort: "desc", top: [], me: null };
      case "leaderboard.around":
        return { board: a["board"], sort: "desc", entries: [], me: null };
      case "point.award":
        return { granted: 0, balance: null };
      default:
        return { method };
    }
  };
  return {
    framed: true,
    hosted: true,
    capabilities,
    time: 1_700_000_000_000,
    async request(method, args) {
      await new Promise((r) => setTimeout(r, 1));
      return capabilities.includes(method) ? { ok: true, data: data(method, args ?? {}), time: 1_700_000_000_500 } : err("unavailable", `no ${method}`);
    },
    event() {},
  };
}

test("the Player bridge keeps the contract over a Player with every method and over one with few", async () => {
  const every = playerPlatform(player(["store.get", "store.set", "store.list", "leaderboard.submit", "leaderboard.top", "leaderboard.around", "point.award", "ai.mark", ...SERVICE_METHODS]), memoryPersistence());
  const r = await checkPlatformProvider(every);
  expect(r.problems).toEqual([]);
  expect([...every.capabilities].sort()).toEqual([...CORE_METHODS, ...HOST_EXTENSIONS].sort());
  expect(r.answers["purchase.buy"]?.[0]).toMatchObject({ ok: true, time: 1_700_000_000_500 });
  // An older Player: the store from the browser's storage, around from top, no services.
  const few = playerPlatform(player(["leaderboard.top"]), memoryPersistence());
  const f = await checkPlatformProvider(few);
  expect(f.problems).toEqual([]);
  expect(f.answers["store.set"]?.[0]).toMatchObject({ ok: true });
  expect(f.answers["leaderboard.around"]?.[0]).toMatchObject({ ok: true });
  expect(f.answers["point"]?.[0]).toMatchObject({ ok: false, error: { code: "unavailable" } });
});

test("the checker catches a provider that answers twice, never, out of shape, an unlisted method, or leaves the game paused", async () => {
  const bad: PlatformProvider = {
    host: "portal",
    capabilities: ["store.get", "store.get", "share", "portal.coins"],
    handle(calls, answer, link) {
      for (const c of calls) {
        if (c.method === "store.get") {
          answer(c.request, { ok: true, data: null });
          answer(c.request, { ok: true, data: null });
        } else if (c.method === "share") link?.pause();
        else if (c.method === "telemetry") answer(c.request, { ok: true, data: null });
        else if (c.method === "store.set") answer(c.request, { ok: false, error: { code: "nope", message: "x", retryable: false } });
        else if (c.method === "store.list") answer(c.request, { ok: true } as never);
        else answer(c.request, { ok: true, data: 1 });
      }
    },
  };
  const r = await checkPlatformProvider(bad, { wait: 50, settle: 10 });
  expect(r.ok).toBeFalse();
  const has = (s: string) => expect(r.problems.some((p) => p.includes(s))).toBeTrue();
  has("capabilities lists store.get twice");
  has("capabilities lists portal.coins, which og.platform never calls");
  has("store.get answered 2 times");
  has("share answered 0 times");
  has("telemetry is one-way");
  has("'nope' is not an error code");
  has("store.list's answer: ok answers carry data");
  has("leaderboard.top is not listed, so it is answered unavailable, not ok");
  has("left paused");
});

test("the page's providers: a relay URL becomes relay rooms, a non-provider is refused with a line, and a publish build never fakes a purchase", async () => {
  const warned: string[] = [];
  expect(pageProviders(undefined, "publish")).toEqual({});
  const p = pageProviders({ platform: { host: "x" }, rooms: "http://nope" }, "dev", (l) => warned.push(l));
  expect(p).toEqual({});
  expect(warned.length).toBe(2);
  expect(warned[1]).toContain("is not a relay URL");

  // A dev page may hand the local fake over; a publish page gets it without the services.
  const dev = pageProviders({ platform: localPlatform(memoryPersistence(), {}) }, "dev").platform!;
  expect(dev.capabilities).toContain("purchase.buy");
  const pub = pageProviders({ platform: localPlatform(memoryPersistence(), {}) }, "publish").platform!;
  expect(pub.capabilities.some((m) => (SERVICE_METHODS as readonly string[]).includes(m))).toBeFalse();
  const r = await checkPlatformProvider(pub);
  expect(r.problems).toEqual([]);
  expect(r.answers["purchase.buy"]?.[0]).toMatchObject({ ok: false, error: { code: "unavailable", message: "a published page answers no purchase.buy" } });
  expect(r.answers["store.set"]?.[0]).toMatchObject({ ok: true });
  // Another provider passes as it is.
  const portal: PlatformProvider = { host: "portal", capabilities: ["share"], handle: () => {} };
  expect(pageProviders({ platform: portal }, "publish").platform).toBe(portal);
});
