// og.i18n on the native host over the UI kit's cooked string tables
// (tests/fixtures/ui-text/assets/strings: en, zh-CN, ja, ko, ar): the host's
// locale picks the table (ja-JP takes ja), t formats placeholders and
// plurals in each language's rules, a key a locale lacks comes from the
// base, a key no table has answers itself with one warning, and the active
// locale sets the UI's font chain and direction (Arabic: right to left).
import { expect, test } from "bun:test";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { og } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: { spawnSync(cmd: string[]): { exitCode: number; stderr: Uint8Array } };

const kit = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..", "tests", "fixtures", "ui-text");

// At module level, not in a hook: a hook's 5 s would not cover a cook on a
// loaded machine (b2532cd's gate).
{
  loadNativeHost();
  // The kit's tables and fonts, cooked (incrementally) by the og this package built.
  const cooked = Bun.spawnSync([ogBin(), "cook", kit]);
  if (cooked.exitCode !== 0) throw new Error(new TextDecoder().decode(cooked.stderr));
  if (!existsSync(join(kit, "cooked"))) throw new Error(`og cook ${kit} wrote no cooked/`);
}

function start(locale?: string): Engine {
  const r = createEngine(og.game({}) as GameToken, { root: kit, ...(locale ? { locale } : {}) });
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

type Info = { requested: string; active: string; direction: string; locales: string[]; fonts: string[] };

test("og.i18n: the host's locale picks the table, and t formats each language's plurals", () => {
  const e = start("ja-JP");
  const w = e.world;
  for (const l of ["en", "zh-CN", "ja", "ko", "ar"]) {
    const r = og.i18n.load(w, `strings/${l}.ogt`);
    if (!r.ok) throw new Error(r.reason);
  }
  const info = og.i18n.locale(w) as unknown as Info;
  expect([info.requested, info.active, info.direction]).toEqual(["ja-JP", "ja", "ltr"]);
  expect(info.locales).toEqual(["ar", "en", "ja", "ko", "zh-CN"]);
  expect(info.fonts[0]).toBe("fonts/NotoSansJP-subset.otf");
  expect(info.fonts.length).toBe(4);
  expect(og.i18n.t(w, "menu.play")).toBe("プレイ");
  expect(og.i18n.t(w, "confirm.message", { slot: 2 })).toBe("スロット[b]2[/b]の進行状況は失われます。[color=danger]元に戻すことはできません。[/color]");
  expect(og.i18n.t(w, "hud.coins", { n: 1 })).toBe("コイン1枚");
  expect(og.i18n.has(w, "menu.play")).toBeTrue();

  const en = og.i18n.setLocale(w, "en-GB") as unknown as Info;
  expect(en.active).toBe("en");
  expect([og.i18n.t(w, "hud.coins", { n: 1 }), og.i18n.t(w, "hud.coins", { n: 1250 })]).toEqual(["1 coin", "1,250 coins"]);

  const ar = og.i18n.setLocale(w, "ar-EG") as unknown as Info;
  expect([ar.active, ar.direction, ar.fonts[0]]).toEqual(["ar", "rtl", "fonts/NotoSansArabic-subset.ttf"]);
  expect([0, 1, 2, 5, 11, 100].map((n) => og.i18n.t(w, "hud.coins", { n }))).toEqual(["لا عملات", "عملة واحدة", "عملتان", "5 عملات", "11 عملة", "100 عملة"]);

  // zh-TW has no table of its own: the base answers.
  expect((og.i18n.setLocale(w, "fr") as unknown as Info).active).toBe("en");
  expect(og.i18n.t(w, "no.such.key")).toBe("no.such.key");
  expect(og.i18n.t(w, "no.such.key")).toBe("no.such.key");
  const warned = e.takeLog().filter((l) => l.text.includes("no table has 'no.such.key'"));
  expect(warned.length).toBe(1);
});

test("og.i18n: refusals name the fix", () => {
  const e = start();
  const r = og.i18n.load(e.world, "strings/none.ogt");
  expect(r.ok).toBeFalse();
  const info = og.i18n.locale(e.world) as unknown as Info;
  expect([info.requested, info.active, info.locales.length]).toEqual(["", "", 0]);
});

test("the kit's zh-CN table is simplified Chinese: every Han character is in GB2312", () => {
  // Wrong: a traditional form (級, 選, 體, 設, 獲) in the simplified table; GB2312 holds the simplified set and none of those.
  const gbk = new TextDecoder("gbk");
  const gb2312 = new Set<string>();
  for (let row = 0xb0; row <= 0xf7; row++) for (let col = 0xa1; col <= 0xfe; col++) gb2312.add(gbk.decode(new Uint8Array([row, col])));
  expect(gb2312.has("级") && !gb2312.has("級")).toBeTrue();
  const table = readFileSync(join(kit, "assets", "strings", "zh-CN.ogt"), "utf8");
  const outside = [...table.matchAll(/^string "([^"]*)" "(.*)"$/gm)].flatMap(([, key, text]) =>
    [...text!].filter((ch) => /\p{Script=Han}/u.test(ch) && !gb2312.has(ch)).map((ch) => `${key}: ${ch}`));
  expect(outside).toEqual([]);
});

test("og.i18n: format functions follow the active locale, else the requested one, with moments the game passes in", () => {
  // No table loaded: the host's request (de-DE) decides.
  const de = start("de-DE").world;
  expect(og.i18n.formatCompact(de, 1234567)).toBe("1,2\u00a0Mio.");
  expect(og.i18n.formatCurrency(de, 1234.5, "EUR")).toBe("1.234,50\u00a0€");
  // 2026-11-02 15:05:09 UTC; Tokyo is 540 minutes ahead.
  const t = 1793631909;
  expect(og.i18n.formatDate(de, t)).toBe("02.11.2026");
  expect(og.i18n.formatDate(de, t, { style: "full" })).toBe("Montag, 2. November 2026");
  expect(og.i18n.formatTime(de, t, { offsetMinutes: 540 })).toBe("00:05");
  expect(og.i18n.formatDuration(de, 3723)).toBe("1:02:03");
  // A loaded table's locale wins over the request.
  const w = start("de-DE").world;
  for (const l of ["en", "ja"]) {
    const r = og.i18n.load(w, `strings/${l}.ogt`);
    if (!r.ok) throw new Error(r.reason);
  }
  og.i18n.setLocale(w, "ja-JP");
  expect(og.i18n.formatCompact(w, 12345)).toBe("1.2万");
  expect(og.i18n.formatCurrency(w, 1234.6, "JPY")).toBe("￥1,235");
  expect(og.i18n.formatDate(w, t, { style: "long" })).toBe("2026年11月2日");
  og.i18n.setLocale(w, "en-US");
  expect(og.i18n.formatTime(w, t, { style: "medium" })).toBe("3:05:09\u202fPM");
});
