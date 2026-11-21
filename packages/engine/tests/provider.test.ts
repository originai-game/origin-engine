// The provider contract (src/provider.ts, docs/design.md §2.8 Providers):
// its vocabulary is the IDL's — platform.ogt's list constants, generated into
// src/generated/vocabulary.ts by `og api gen` (`og api check` holds the
// generated files to the IDL) and re-exported by provider.ts. The providers
// in the tree keep the contract under checkPlatformProvider: the local
// document with and without its fake, no platform, and the Player bridge
// (shell/platform.ts) over a stand-in Player with every method and with
// few. The checker itself catches a provider that answers twice, never, or
// with the wrong shape. The page's choice of providers (pageProviders) is
// checked, and a publish build never fakes a purchase through it.
import { expect, test } from "bun:test";
import { catalogHeadAnswer, checkPlatformProvider, CORE_METHODS, CORE_ONLY_METHODS, staticCatalog, ERROR_CODES, HOST_EXTENSIONS, PROVIDER_VERSION, RETRYABLE_CODES, SERVICE_METHODS, unavailable, type PlatformProvider } from "../src/provider.ts";
import * as vocabulary from "../src/generated/vocabulary.ts";
import { fail, LOCAL_CAPABILITIES, localPlatform, memoryPersistence, noPlatform } from "../src/platform-local.ts";
import { pageProviders, playerPlatform, type PlayerLink } from "../shell/platform.ts";
import { contentBase } from "../shell/page.ts";
import type { PlayerResult } from "../shell/player.ts";

test("the vocabulary is generated from the IDL: provider re-exports platform.ogt's list constants", () => {
  // Re-exports, not copies: the same arrays the generator wrote.
  expect(SERVICE_METHODS).toBe(vocabulary.SERVICE_METHODS);
  expect(HOST_EXTENSIONS).toBe(vocabulary.HOST_EXTENSIONS);
  expect(ERROR_CODES).toBe(vocabulary.ERROR_CODES);
  expect(RETRYABLE_CODES).toBe(vocabulary.RETRYABLE_CODES);
  // The partitions make the vocabulary: CORE_METHODS is each partition once, services within the core, extensions beside it.
  expect([...CORE_METHODS]).toEqual([...CORE_ONLY_METHODS, ...SERVICE_METHODS]);
  expect(SERVICE_METHODS.every((s) => (CORE_METHODS as readonly string[]).includes(s))).toBeTrue();
  expect(HOST_EXTENSIONS.every((h) => !(CORE_METHODS as readonly string[]).includes(h))).toBeTrue();
  // The local platform answers the core vocabulary but `catalog.head` (only a content base serves it).
  expect(new Set([...LOCAL_CAPABILITIES, "catalog.head"])).toEqual(new Set([...CORE_ONLY_METHODS, ...HOST_EXTENSIONS]));
  expect(PROVIDER_VERSION).toBe(1);
});

test("the retryable error codes are among all of them, and the local failures follow", () => {
  expect(RETRYABLE_CODES.every((c) => (ERROR_CODES as readonly string[]).includes(c))).toBeTrue();
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

const HEAD = { catalog: "world", version: "v12", manifest: `sha384-${"A".repeat(64)}`, published: 1_767_225_600_000 };

test("catalog heads: staticCatalog reads <base>/<catalog>/head.json and passes the rest on; the local platform answers one only with a content base", async () => {
  const files: Record<string, string> = { "https://cdn.example/g/world/head.json": JSON.stringify(HEAD), "https://cdn.example/g/bad/head.json": JSON.stringify({ ...HEAD, catalog: "bad", manifest: "sha256-x" }) };
  const asked: string[] = [];
  const read = async (url: string) => (asked.push(url), files[url] ?? null);
  const local = localPlatform(memoryPersistence());
  const p = staticCatalog("https://cdn.example/g/", { inner: local, read });
  expect(p.content?.base()).toBe("https://cdn.example/g");
  expect([...p.capabilities].sort()).toEqual([...LOCAL_CAPABILITIES, "catalog.head"].sort());
  const got: Record<number, unknown> = {};
  p.handle(
    [
      { request: 1, method: "catalog.head", args: { catalog: "world" }, tick: 0 },
      { request: 2, method: "catalog.head", args: { catalog: "none" }, tick: 0 },
      { request: 3, method: "catalog.head", args: { catalog: "bad" }, tick: 0 },
      { request: 4, method: "store.get", args: { key: "k" }, tick: 0 },
    ],
    (r, a) => void (got[r] = a),
  );
  await new Promise((r) => setTimeout(r, 5));
  expect(got[1]).toEqual({ ok: true, data: HEAD });
  expect(got[2]).toMatchObject({ ok: false, error: { code: "not_found" } });
  expect(got[3]).toMatchObject({ ok: false, error: { code: "failed" } });
  expect(got[4]).toMatchObject({ ok: true, data: { key: "k", value: null } });
  expect(asked).toEqual(["https://cdn.example/g/world/head.json", "https://cdn.example/g/none/head.json", "https://cdn.example/g/bad/head.json"]);
  // A network failure is `offline`, never a hang.
  const lost = staticCatalog("https://cdn.example/g", { read: () => Promise.reject(new TypeError("Failed to fetch")) });
  const once: unknown[] = [];
  lost.handle([{ request: 1, method: "catalog.head", args: { catalog: "world" }, tick: 0 }], (_r, a) => void once.push(a));
  await new Promise((r) => setTimeout(r, 5));
  expect(once).toMatchObject([{ ok: false, error: { code: "offline", retryable: true } }]);
  // Both keep the contract; a provider whose head has the wrong shape does not.
  expect((await checkPlatformProvider(p)).problems).toEqual([]);
  const withHead = localPlatform(memoryPersistence(), undefined, { base: "/content", read: (url) => (url === "/content/conformance/head.json" ? JSON.stringify({ ...HEAD, catalog: "conformance" }) : null) });
  const w = await checkPlatformProvider(withHead);
  expect(w.problems).toEqual([]);
  expect(w.answers["catalog.head"]).toMatchObject([{ ok: true, data: { catalog: "conformance", version: "v12" } }]);
  expect(withHead.content?.base()).toBe("/content");
  expect(local.capabilities).not.toContain("catalog.head");
  expect((await checkPlatformProvider(local)).answers["catalog.head"]).toMatchObject([{ ok: false, error: { code: "unavailable" } }]);
  const wrong: PlatformProvider = { host: "wrong", capabilities: ["catalog.head"], handle: (calls, answer) => calls.forEach((c) => c.method !== "telemetry" && answer(c.request, c.method === "catalog.head" ? { ok: true, data: { catalog: "conformance", version: "v1" } } : unavailable(c.method))) };
  expect((await checkPlatformProvider(wrong)).problems.join("\n")).toContain("catalog.head's answer: the head's manifest is not an integrity");
  expect(catalogHeadAnswer("world", "{")).toMatchObject({ ok: false, error: { code: "failed" } });
});

test("the content base: a Player's hello names it over the build's; a provider's that is not an https directory falls back with a line", () => {
  const named = playerPlatform({ ...player([]), content: "https://api.example/world/v1/content/iw-dev/" }, memoryPersistence());
  const lines: string[] = [];
  const warn = (l: string) => void lines.push(l);
  expect(contentBase(named, "https://build.example/c", warn)).toBe("https://api.example/world/v1/content/iw-dev");
  // A Player whose hello names none: the build's.
  expect(playerPlatform(player([]), memoryPersistence()).content).toBeUndefined();
  expect(contentBase(playerPlatform(player([]), memoryPersistence()), "https://build.example/c", warn)).toBe("https://build.example/c");
  expect(contentBase(undefined, undefined, warn)).toBeUndefined();
  expect(lines).toEqual([]);
  for (const bad of ["http://evil.example/c", "https://u:p@cdn.example/c", "https://cdn.example/c?x=1", "/relative"]) {
    expect(contentBase(staticCatalog(bad), "https://build.example/c", warn)).toBe("https://build.example/c");
  }
  expect(lines.length).toBe(4);
  expect(contentBase(staticCatalog("http://localhost:8731/"), undefined, warn)).toBe("http://localhost:8731");
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
      case "catalog.head":
        return { catalog: a["catalog"], version: "v1", manifest: HEAD.manifest, published: 1 };
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
  const every = playerPlatform(player(["store.get", "store.set", "store.list", "leaderboard.submit", "leaderboard.top", "leaderboard.around", "point.award", "ai.mark", "catalog.head", ...SERVICE_METHODS]), memoryPersistence());
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
