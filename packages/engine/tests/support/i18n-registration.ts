import { expect } from "bun:test";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { og } from "@origingame/engine";
import { createEngine, type Engine, type GameToken } from "../../src/runtime.ts";

const host = process.argv[2], root = process.argv[3];
if (!root) throw new Error("expected cooked i18n kit root");
if (host === "node") {
  (await import("../../src/node/index.ts")).loadNativeHost();
} else if (host === "web") {
  const { loadWebHost } = await import("../../src/web/host.ts");
  const glue = await import(fileURLToPath(new URL("../../dist/web/origin-engine.js", import.meta.url)));
  await loadWebHost(glue, readFileSync(fileURLToPath(new URL("../../dist/web/origin-engine_bg.wasm", import.meta.url))));
  function preload(directory: string): void {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const path = join(directory, entry.name);
      if (entry.isDirectory()) preload(path);
      else if (entry.isFile()) glue.og_web_put_file(path, readFileSync(path));
    }
  }
  preload(join(root, "cooked"));
} else throw new Error("expected node or web host");

function initialize(world: Engine["world"]): void {
  for (const locale of ["en", "ja"]) {
    const loaded = og.i18n.load(world, `strings/${locale}.ogt`);
    if (!loaded.ok) throw new Error(loaded.reason);
  }
  expect(og.i18n.setLocale(world, "ja-JP").active).toBe("ja");
}

// Check writes first so even the baseline read failure follows verified refusals.
const refused: string[] = [];
for (const write of ["load", "setLocale"] as const) {
  let before!: ReturnType<typeof og.i18n.locale>;
  let attempted = false;
  const deferred = createEngine(og.game({
    start(world) {
      initialize(world);
      before = og.i18n.locale(world);
    },
    ui(world) {
      attempted = true;
      if (write === "load") {
        const result = og.i18n.load(world, "strings/ar.ogt");
        expect(result.ok).toBe(false);
        if (!result.ok) expect(result.reason).toContain("og_i18n_load is not allowed");
      } else og.i18n.setLocale(world, "en");
    },
  }) as GameToken, { root, deferStart: true });
  if (!deferred.ok) throw new Error(`${host}: ${deferred.reason}`);
  const e = deferred.value;
  try {
    const advanced = e.advance(1);
    expect(attempted).toBe(true);
    expect(advanced.ok).toBe(false);
    if (!advanced.ok) expect(advanced.reason).toContain(`og_i18n_${write === "load" ? "load" : "set_locale"} is not allowed`);
    expect(og.i18n.locale(e.world)).toEqual(before);
    expect(og.i18n.locale(e.world).locales).toEqual(["en", "ja"]);
    expect(og.i18n.t(e.world, "menu.play")).toBe("プレイ");
    refused.push(write);
  } finally { e.destroy(); }
}

let registered = false;
const created = createEngine(og.game({
  start: initialize,
  ui(world) {
    const scope = og.ui.scope(world, "localized");
    const title = scope.writable("title", og.i18n.t(world, "menu.play"));
    expect(scope.get(title)).toBe("プレイ");
    expect(og.i18n.locale(world).requested).toBe("ja-JP");
    expect(og.i18n.locale(world).active).toBe("ja");
    expect(og.i18n.has(world, "menu.play")).toBe(true);
    expect(og.i18n.has(world, "no.such.key")).toBe(false);
    expect(og.i18n.t(world, "hud.coins", { n: 1 })).toBe("コイン1枚");
    expect(og.i18n.formatNumber(world, 1234.5, { decimals: 2 })).toBe("1,234.50");
    expect(og.i18n.formatCompact(world, 12345)).toBe("1.2万");
    expect(og.i18n.formatCurrency(world, 1234.6, "JPY")).toBe("￥1,235");
    expect(og.i18n.formatDate(world, 1793631909000, { style: "long" })).toBe("2026年11月2日");
    expect(og.i18n.formatTime(world, 1793631909000, { offsetMinutes: 540 })).toBe("0:05");
    expect(og.i18n.formatDuration(world, 3723)).toBe("1:02:03");
    registered = true;
  },
}) as GameToken, { root });
if (!created.ok) throw new Error(`${host}: ${created.reason}`);
try { expect(registered).toBe(true); }
finally { created.value.destroy(); }
console.log(JSON.stringify({ host, registered, refused }));
