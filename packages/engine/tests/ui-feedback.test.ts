// The feedback components (docs/ui.md §11, U6c) on the native host, read
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

beforeAll(() => {
  loadNativeHost();
});


test("og.ui.formatNumber: every notation, rounding into the next group, letters, past 1e308", () => {
  const f = og.ui.formatNumber;
  expect(f(12840000, {})).toBe("12,840,000");
  expect(f(-1234, {})).toBe("-1,234");
  expect(f(1234.5, { decimals: 1 })).toBe("1,234.5");
  expect(f(1234.5, { decimals: 1, group: ".", point: "," })).toBe("1.234,5");
  expect(f(9999, { notation: "compact" })).toBe("9,999");
  expect(f(10000, { notation: "compact" })).toBe("10.0K");
  expect(f(12_345_678, { notation: "compact" })).toBe("12.3M");
  expect(f(999_950, { notation: "compact" })).toBe("1.00M");
  expect(f(3.4e12, { notation: "compact" })).toBe("3.40T");
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
  expect(f(Number.NaN, { prefix: "$" })).toBe("$—");
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

test("og.ui.numberCounter: counts through the values between, lands on the value, shows its rise beside it, keeps the final width", () => {
  let value = 100;
  const shown: number[] = [];
  const e = engineFor("FbCounter", () => {
    shown.push(og.ui.numberCounter("Gold", value, { label: "Gold", durationMs: 500 }));
  });
  e.advance(2);
  expect(shown.at(-1)).toBe(100);
  value = 1100;
  e.advance(10);
  const mid = shown.at(-1)!;
  expect(mid).toBeGreaterThan(100);
  expect(mid).toBeLessThan(1100);
  let els = dump(e);
  expect(at(els, "Gold/Delta")!.text).toBe("+1,000");
  // The rise has its own place beside the number, and the number's box is already the final value's width.
  const body = at(els, "Gold/Body")!;
  expect(at(els, "Gold/Delta")!.x).toBeGreaterThanOrEqual(body.x + body.width);
  const midWidth = body.width;
  e.advance(40);
  expect(shown.at(-1)).toBe(1100);
  els = dump(e);
  expect(at(els, "Gold/Body/Value")!.text).toBe("1,100");
  expect(at(els, "Gold/Body")!.width).toBe(midWidth);
  expect(at(els, "Gold/Body/Value")!.width).toBeLessThanOrEqual(midWidth + 0.5);
  expect(at(els, "Gold")!.value).toBe("1,100");

  // A count wider than what holds it steps its numerals down instead of spilling out: digits never wrap.
  const narrow = engineFor("FbCounterNarrow", () => {
    og.ui.anchor("Box", { anchor: "topLeft", width: { kind: "fixed", px: 120 } }, () => {
      og.ui.numberCounter("Gold", 1_284_330, { label: "Gold", step: "xl4" });
    });
  });
  narrow.advance(4);
  els = dump(narrow);
  const box = at(els, "Box")!;
  const big = at(els, "Box/Gold/Body/Value")!;
  expect(big.text).toBe("1,284,330");
  expect(big.x + big.width).toBeLessThanOrEqual(box.x + box.width + 0.5);
});

test("og.ui.tooltipCard: shows after its delay beside its trigger, pins after resting, leaves when let go", () => {
  let open = true;
  let r: ReturnType<typeof og.ui.tooltipCard> | null = null;
  const e = engineFor("FbTip", () => {
    // The trigger is at the end of a wider panel: the card must clear the panel, not just the trigger.
    og.ui.anchor("Bar", { anchor: "right", margin: 16, direction: "column", cross: "end", width: { kind: "fixed", px: 320 } }, () => {
      og.ui.button("Sword", "Sword", {});
      r = og.ui.tooltipCard("Tip", {
        open, target: "Sword", title: "Ashbringer", rarity: "legendary", subtitle: "Legendary sword",
        stats: [{ label: "Damage", value: "42–58", delta: 12 }], delayMs: 200, pinMs: 400,
        body: "Deals [link=fire]fire[/link] damage.", links: [{ key: "fire", title: "Fire", body: "Burns." }],
      });
    });
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
  // Resting past pinMs pins it; let go, a pinned card stays only while held, then leaves.
  e.advance(30);
  expect(r!.pinned).toBe(true);
  open = false;
  e.advance(40);
  expect(r!.visible).toBe(false);
});

test("og.ui.progressBar and progressRing: a screen reader hears the value, a spinner is busy", () => {
  const e = engineFor("FbProgress", () => {
    og.ui.progressBar("Download", { value: 0.42, label: "Downloading", showValue: true, width: { kind: "fixed", px: 300 } });
    og.ui.progressRing("Spin", { name: "Loading" });
    og.ui.progressBar("Stages", { value: 0.5, style: "steps", steps: 4, width: { kind: "fixed", px: 300 } });
  });
  e.advance(3);
  const els = dump(e);
  const bar = at(els, "Download")!;
  expect(bar.role).toBe("progressbar");
  expect(bar.value).toBe("42%");
  expect(at(els, "Download/Head/Value")!.text).toBe("42%");
  expect(at(els, "Spin")!.role).toBe("progressbar");
  expect(els.filter((x) => x.parent === "Track" && x.path.startsWith("Stages/Track/Step#")).length).toBe(4);
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
