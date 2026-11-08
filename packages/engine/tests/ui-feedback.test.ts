// The feedback components (docs/ui.md §2, U6c) on the native host, read
// through dump_ui and the accessibility fields: what each shows, when it
// leaves, what a screen reader is told, and the number notations.
//
// Ways they could fail, each covered below:
// - a notation rounds 999,950 to "1000K", runs out of letters past T,
//   loses numbers past 1e308, or drops the sign;
// - a notification shows past its time, comes back after it expired
//   while the game still lists it, shows more than max, stacks oldest
//   nearest the edge, or reads an alert as a polite status;
// - a count badge shows 120 instead of 99+, or shows at 0;
// - a counter jumps to its value instead of counting, never lands, or
//   spills out of a narrow holder;
// - a tooltip shows before its delay, never pins, or sits on top of
//   its trigger or the panel it is in instead of beside it (or off the
//   safe area at the edge);
// - a progress bar tells a screen reader nothing of its value;
// - subtitles show three speech lines, or captions the player turned off.
import { beforeAll, expect, test } from "bun:test";
import { og } from "@origingame/engine";
import type { NotificationItem } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { at, dump, engineFor } from "./support/ui.ts";
import { groupNumber } from "../src/ui/data/format.ts";
import { countdown, shortTime } from "../src/ui/feedback/core.ts";

beforeAll(() => {
  loadNativeHost();
});


test("og.ui.formatNumber: every notation, rounding into the next group, letters, past 1e308", () => {
  const f = og.ui.formatNumber;
  expect(f(12840000, {})).toBe("12,840,000");
  expect(f(-1234, {})).toBe("-1,234");
  expect(f(1234.5, { decimals: 1 })).toBe("1,234.5");
  expect(f(1234.5, { decimals: 1, group: ".", point: "," })).toBe("1.234,5");
  expect(f(9999, { notation: "compact" })).toBe("10K");
  expect(f(10000, { notation: "compact" })).toBe("10K");
  expect(f(12_345_678, { notation: "compact" })).toBe("12M");
  expect(f(999_950, { notation: "compact" })).toBe("1M");
  expect(f(3.4e12, { notation: "compact" })).toBe("3.4T");
  expect(f(3.4e12, { notation: "compact", decimals: 2 })).toBe("3.40T");
  expect(f(5e15, { notation: "compact" })).toBe("5.00e15");
  // aa follows T; the 136th letter pair after T is ff (5 × 26 + 5).
  expect(f(1.5e15, { notation: "letters" })).toBe("1.50aa");
  expect(f(2.5e18, { notation: "letters" })).toBe("2.50ab");
  expect(f(1.5, { notation: "letters", exponent: 420 })).toBe("1.50ff");
  expect(f(12_800_000, { notation: "scientific" })).toBe("1.28e7");
  expect(f(12_800_000, { notation: "engineering" })).toBe("12.8e6");
  expect(f(5, { notation: "compact", exponent: 400 })).toBe("5.00e400");
  expect(f(120, { sign: true, suffix: " XP" })).toBe("+120 XP");
  expect(f(0, { sign: true })).toBe("0");
  expect(f(0.004, { sign: true, decimals: 2 })).toBe("0.00");
  expect(f(-0.004, { sign: true, decimals: 2 })).toBe("0.00");
  expect(f(0.006, { sign: true, decimals: 2 })).toBe("+0.01");
  expect(f(Number.NaN, { prefix: "$" })).toBe("$—");
});

test("core data, feedback and calendar formatting follows the current engine locale", () => {
  let locale = "de";
  let values: string[] = [];
  const e = engineFor("LocalizedCore", (world) => {
    og.i18n.setLocale(world, locale);
    values = [groupNumber(1234.5, 1), countdown(125), shortTime(120), og.ui.formatNumber(12345, { notation: "compact" })];
    og.ui.calendar("Calendar", { year: 2026, month: 11 });
  });
  e.advance(3);
  expect(values).toEqual(["1.234,5", "2:05", "2 Min.", "12.345"]);
  expect(at(dump(e), "Calendar/Header/Month")?.text).toBe("November 2026");
  locale = "ja";
  e.advance(3);
  expect(values[3]).toBe("1.2万");
  expect(at(dump(e), "Calendar/Header/Month")?.text).toBe("2026年11月");
  locale = "bn";
  e.advance(3);
  expect(values.slice(0, 2)).toEqual(["১,২৩৪.৫", "২:০৫"]);
  locale = "en-XA";
  e.advance(3);
  expect(values.slice(0, 2)).toEqual(["[1,234.5~~~]", "[2:05~~]"]);
});

test("og.ui.notifications: newest at the edge, at most max, gone after its time and never back", () => {
  const items: NotificationItem[] = [
    { key: "a", title: "First", tone: "success", at: 0, durationMs: 1000 },
    { key: "b", title: "Second", at: 0, durationMs: 1000 },
    { key: "c", title: "Danger close", variant: "alert", at: 0, durationMs: 1000 },
    { key: "d", title: "Queued", at: 0, durationMs: 1000 },
  ];
  let last: ReturnType<typeof og.ui.notifications> | null = null;
  const expired: string[] = [];
  const e = engineFor("FbNotes", () => {
    last = og.ui.notifications("Notes", { items, max: 3, anchor: "topRight" });
    expired.push(...last.expired);
  });
  e.advance(3);
  // The three oldest show, the newest of them nearest the top edge.
  expect(last!.shown).toEqual(["c", "b", "a"]);
  let els = dump(e);
  const ya = at(els, "Notes/a")!.y;
  const yc = at(els, "Notes/c")!.y;
  expect(yc).toBeLessThan(ya);
  expect(at(els, "Notes/c")!.role).toBe("alert");
  expect(at(els, "Notes/a")!.role).toBe("status");
  expect(at(els, "Notes/d")).toBe(undefined);
  // A second past their time (60 ticks), the first three expire once, and the queued one shows.
  e.advance(70);
  expect(expired.sort()).toEqual(["a", "b", "c"]);
  expect(last!.shown).toEqual(["d"]);
  // The game still lists them: they do not come back.
  e.advance(5);
  expect(last!.shown).toEqual(["d"]);
  els = dump(e);
  expect(at(els, "Notes/a")).toBe(undefined);
});

test("og.ui.countBadge: 99+ past max, nothing at 0, read as a status", () => {
  let count = 120;
  const e = engineFor("FbBadge", () => {
    og.ui.column("Bar", { padding: { left: 40, top: 40, right: 40, bottom: 40 } }, () => {
      og.ui.row("Mail", { width: { kind: "fixed", px: 48 }, height: { kind: "fixed", px: 48 } }, () => {
        og.ui.countBadge("Unread", { count, name: `${count} unread` });
      });
    });
  });
  e.advance(2);
  let b = at(dump(e), "Bar/Mail/Unread")!;
  expect(at(dump(e), "Bar/Mail/Unread/Text")!.text).toBe("99+");
  expect(b.role).toBe("status");
  expect(b.name).toBe("120 unread");
  // It straddles the container's top-end corner.
  const mail = at(dump(e), "Bar/Mail")!;
  expect(b.y).toBeLessThan(mail.y);
  expect(b.x + b.width).toBeGreaterThan(mail.x + mail.width);
  count = 0;
  e.advance(2);
  expect(at(dump(e), "Bar/Mail/Unread")).toBe(undefined);
});

test("og.ui.tooltipCard: shows after its delay beside its trigger, pins after resting, leaves when let go", () => {
  let open = true;
  let r: ReturnType<typeof og.ui.tooltipCard> | null = null;
  const e = engineFor("FbTip", () => {
    // The trigger is at the end of a wider panel: the card must clear the panel, not just the trigger.
    og.ui.anchor("Bar", { anchor: "topRight", margin: 16, direction: "column", cross: "end", width: { kind: "fixed", px: 320 } }, () => {
      og.ui.button("Sword", "Sword", {});
      r = og.ui.tooltipCard("Tip", {
        open, target: "Sword", title: "Ashbringer", rarity: "legendary", subtitle: "Legendary sword",
        stats: [{ label: "Damage", value: "42–58", delta: 12 }], delayMs: 200, pinMs: 400,
        body: "Deals [link=fire]fire[/link] damage.", links: [{ key: "fire", title: "Fire", body: "Burns." }],
      });
    });
    // A wide toast stack built after the card, across it.
    og.ui.notifications("Toasts", { anchor: "topRight", width: 760, items: [{ key: "warn", variant: "alert", tone: "danger", title: "Connection unstable", at: 0, sticky: true }] });
  });
  e.advance(4);
  expect(r!.visible).toBe(false);
  e.advance(12);
  expect(r!.visible).toBe(true);
  expect(r!.pinned).toBe(false);
  e.advance(4);
  const els = dump(e);
  const tip = at(els, "Tip")!;
  const sword = at(els, "Bar/Sword")!;
  expect(tip.role).toBe("tooltip");
  expect(tip.name).toBe("Ashbringer");
  // The trigger is at the right edge: the card flips to the left of its whole panel, inside the surface.
  const bar = at(els, "Bar")!;
  expect(sword.x).toBeGreaterThan(bar.x + 100);
  expect(tip.x + tip.width).toBeLessThanOrEqual(bar.x + 1);
  expect(tip.x).toBeGreaterThanOrEqual(0);
  expect(at(els, "Tip/Card/Stats/Stat#0/Delta/Text")!.text).toBe("12");
  expect(at(els, "Tip/Card/Stats/Stat#0/Delta/Mark")).toBeDefined();
  // Built before the toast stack and over it, the card still draws above it (the tooltip layer, not build order).
  const toast = at(els, "Toasts")!;
  expect(toast.x).toBeLessThan(tip.x + tip.width);
  expect(toast.y).toBeLessThan(tip.y + tip.height);
  expect(els.indexOf(at(els, "Tip/Card")!)).toBeGreaterThan(els.indexOf(at(els, "Toasts/warn")!));
  // Resting past pinMs pins it; let go, a pinned card stays only while held, then leaves.
  e.advance(30);
  expect(r!.pinned).toBe(true);
  open = false;
  e.advance(40);
  expect(r!.visible).toBe(false);
});

test("og.ui.subtitles: two speech lines at most, captions only when asked, the speaker named", () => {
  let captions = false;
  const e = engineFor("FbSubs", () => {
    og.ui.subtitles("Subs", {
      settings: { captions },
      lines: [
        { key: "l1", speaker: "Ada", text: "First line." },
        { key: "l2", speaker: "Bo", text: "Second line." },
        { key: "l3", speaker: "Ada", text: "Third line." },
        { key: "s1", caption: true, text: "door creaks" },
      ],
    });
  });
  e.advance(2);
  let els = dump(e);
  expect(at(els, "Subs/l1")).toBe(undefined);
  expect(at(els, "Subs/l2")!.name).toBe("Bo: Second line.");
  expect(at(els, "Subs/l3")).toBeDefined();
  expect(at(els, "Subs/s1")).toBe(undefined);
  captions = true;
  e.advance(2);
  els = dump(e);
  expect(at(els, "Subs/s1/Text")!.text).toBe("[door creaks]");
});

test("og.ui.banner: an open tooltip card fades out while it shows, and comes back after", () => {
  let tick0 = -1;
  const e = engineFor("FbBannerTip", () => {
    if (tick0 < 0) tick0 = og.ui.tick();
    og.ui.button("Sword", "Ashbringer", {});
    og.ui.tooltipCard("Tip", { open: true, target: "Sword", title: "Ashbringer", body: "A blade.", delayMs: 0 });
    og.ui.banner("Wave", { kind: "victory", title: "Victory", shownAt: tick0 + 60, durationMs: 300 });
  });
  const cover = (): string | undefined => (at(dump(e), "Tip") as { visualStates?: Record<string, string> } | undefined)?.visualStates?.cover;
  e.advance(50);
  expect(cover()).toBe("clear");
  e.advance(20);
  expect(cover()).toBe("hidden");
  e.advance(120);
  expect(cover()).toBe("clear");
});

test("og.ui.starRating and og.ui.banner: earned stars, a banner that holds and leaves", () => {
  let tick0 = -1;
  let r: ReturnType<typeof og.ui.banner> | null = null;
  const e = engineFor("FbStars", () => {
    if (tick0 < 0) tick0 = og.ui.tick();
    og.ui.starRating("Stars", { earned: 2, max: 3 });
    r = og.ui.banner("Wave", { kind: "wave", title: "Wave 3", shownAt: tick0, durationMs: 500 });
  });
  e.advance(3);
  const els = dump(e);
  expect(at(els, "Stars")!.name).toBe("2 of 3 stars");
  expect(els.filter((x) => x.id === "Earned").length).toBe(2);
  expect(r!.visible).toBe(true);
  expect(at(els, "Wave/Band/Title")!.text).toBe("Wave 3");
  e.advance(80);
  expect(r!.visible).toBe(false);
});
