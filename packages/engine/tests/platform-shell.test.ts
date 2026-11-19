// og.platform in a store package (src/hermes/platform.ts): the local
// document kept in the shell's data directory across runs, a full disk that
// costs the save and not the game, and a bundled store provider reaching its
// native library beside the shell.
import { expect, test } from "bun:test";
import type { PlatformAnswer, PlatformCall, PlatformProvider } from "../src/provider.ts";
import { shellPlatform, type ShellServices } from "../src/hermes/platform.ts";

function services(disk: Map<string, string>, extra: Partial<ShellServices> = {}): ShellServices & { warnings: string[] } {
  const warnings: string[] = [];
  return {
    load: (game) => disk.get(game) ?? null,
    save: (game, doc) => (disk.set(game, doc), true),
    open: () => ({ ok: false, reason: "no library" }),
    call: () => null,
    warn: (m) => warnings.push(m),
    warnings,
    ...extra,
  };
}

function ask(p: PlatformProvider, method: string, args: Record<string, unknown>): PlatformAnswer {
  let got: PlatformAnswer | undefined;
  const call: PlatformCall = { request: 1, method, args, tick: 0 };
  p.handle([call], (_, a) => (got = a));
  return got!;
}

test("the local document is the game's and survives the next run", () => {
  const disk = new Map<string, string>();
  const first = shellPlatform({ OG_TITLE: "night-shift" }, services(disk));
  expect(first.host).toBe("local");
  expect(ask(first, "store.set", { key: "save", value: { level: 3 } })).toMatchObject({ ok: true });
  // Another game's document is its own.
  expect(ask(shellPlatform({ OG_TITLE: "first-light" }, services(disk)), "store.get", { key: "save" })).toMatchObject({ ok: true, data: { value: null } });
  const next = shellPlatform({ OG_TITLE: "night-shift" }, services(disk));
  expect(ask(next, "store.get", { key: "save" })).toMatchObject({ ok: true, data: { value: { level: 3 }, revision: 1 } });
});

test("a save the disk refuses warns and the game goes on", () => {
  const s = services(new Map(), { save: () => false });
  const p = shellPlatform({ OG_TITLE: "night-shift" }, s);
  expect(ask(p, "store.set", { key: "save", value: 1 })).toMatchObject({ ok: true });
  expect(ask(p, "store.get", { key: "save" })).toMatchObject({ ok: true, data: { value: 1, revision: 1 } });
  expect(s.warnings).toEqual(["og.platform: could not save night-shift's document in the data directory"]);
});

test("a bundled provider calls its library beside the shell, with the local document behind it", () => {
  const opened: string[] = [];
  const calls: Array<[string, string]> = [];
  const s = services(new Map(), {
    open: (path) => (opened.push(path), { ok: true }),
    call: (method, json) => (calls.push([method, json]), method === "user" ? '{"name":"Ada"}' : null),
  });
  const env = { OG_TITLE: "night-shift", OG_BIN: "/pkg/night-shift.app/Contents/MacOS", OG_PROVIDER_LIB: "libstore.dylib", STORE_APP: "480" };
  const p = shellPlatform(env, s, ({ native, game, env, local }) => ({
    host: "store",
    capabilities: ["player.get", ...local.capabilities],
    handle(list, answer, link) {
      const mine = list.filter((c) => c.method === "player.get");
      for (const c of mine) answer(c.request, { ok: true, data: { game, app: env.STORE_APP, user: native!.call("user", { id: c.args["id"] }), none: native!.call("poll") } });
      local.handle(list.filter((c) => c.method !== "player.get"), answer, link);
    },
  }));
  expect(opened).toEqual(["/pkg/night-shift.app/Contents/MacOS/libstore.dylib"]);
  expect(ask(p, "player.get", { id: 7 })).toEqual({ ok: true, data: { game: "night-shift", app: "480", user: { name: "Ada" }, none: null } });
  expect(calls).toEqual([["user", '{"id":7}'], ["poll", "null"]]);
  expect(ask(p, "store.set", { key: "k", value: 2 })).toMatchObject({ ok: true });
});

test("a provider library that does not load stops the package with its reason", () => {
  const env = { OG_TITLE: "night-shift", OG_BIN: "/pkg", OG_PROVIDER_LIB: "libstore.so" };
  expect(() => shellPlatform(env, services(new Map()), ({ local }) => local)).toThrow("the provider library: no library");
});
