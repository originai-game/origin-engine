// The general components as a game uses them (docs/ui.md §6.1): built in,
// used by name from the game's own pages with nothing copied or loaded but
// the pages, dressed by the fantasy pack, each member mounted from every
// fixture its document declares and driven through the probe's touch, key
// and pad commands.
//
// Ways it could fail: an intent sent without its row key, or sent for an
// offer the game refused (sold out, owned, unaffordable), a day that is not
// today, a reward not yet earned or a muted chat; a hidden achievement
// leaking its name; a member that only fits the desktop; a fixture that no
// longer mounts; a member not loaded with the page that uses it.
import { afterAll, beforeAll, expect, setDefaultTimeout, test } from "bun:test";

// The tests run on ticks, not the clock; a time limit only guards a hang.
setDefaultTimeout(60_000);
import { mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { defineSystem, og } from "@origingame/engine";

declare const Bun: { spawnSync(cmd: string[], o: { cwd: string; env: Record<string, string | undefined>; stdout: "ignore"; stderr: "pipe" }): { exitCode: number; stderr: Uint8Array } };
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { ogBin } from "./support/og-bin.ts";

type El = { id: string; path: string; text: string; x: number; y: number; width: number; height: number; states?: string[]; disabled?: boolean; name?: string; role?: string };
type Event = { mount: string; path: string; rowKey: string; kind: string; name: string; value?: unknown };

const ENGINE = dirname(dirname(fileURLToPath(import.meta.url)));
const GROUP = join(ENGINE, "src/ui/components/general");
const STYLES = process.env.OG_STYLES_DIR ?? join(ENGINE, "../../templates/styles");
let root = "";

/** Each member's component name, its parts and its fixtures, read from its document as a game's tooling reads them. */
const MEMBERS: Record<string, { component: string; parts: string[]; fixtures: Record<string, Record<string, unknown>> }> = {};

beforeAll(() => {
  loadNativeHost();
  root = mkdtempSync(join(tmpdir(), "og-general-kit-"));
  mkdirSync(join(root, "assets/ui"), { recursive: true });
  writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "General kit"\nid "general-kit"\nentry "main.ts"\n');
  writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});\n');
  // A look naming the pack is what makes the cook bundle it (@styles/fantasy).
  mkdirSync(join(root, "assets/looks"), { recursive: true });
  writeFileSync(join(root, "assets/looks/fantasy.ogt"), 'type look\nversion 1\n---\nname "general kit fantasy"\nui "fantasy"\n');
  for (const f of readdirSync(GROUP).filter((f) => f.endsWith(".svelte"))) {
    const text = readFileSync(join(GROUP, f), "utf8");
    const component = /<meta name="component" content="([^"]+)">/.exec(text)![1]!;
    const parts = /<meta name="parts" content="([^"]+)">/.exec(text)![1]!.split(" ");
    const fixtures = Object.fromEntries([...text.matchAll(/<meta name="fixture:([a-z]+)" content='([^']*)'>/g)].map((m) => [m[1]!, JSON.parse(m[2]!)]));
    const name = f.replace(".svelte", "");
    MEMBERS[name] = { component, parts, fixtures };
    // A game mounts its own page, which uses the member by name: one host page per member, its props from stores.
    // A two-way prop is bound as Svelte binds one (bind:draft); the rest are read.
    const props = Object.keys(fixtures.default!).map((k) => k === "draft" ? `bind:draft={$draft}` : `${k}={$${k}}`).join(" ");
    writeFileSync(join(root, `assets/ui/host-${name}.svelte`), `<meta name="screen" content="Host_${component}" data-layer="menu" data-input="menu">\n<div style="width: stretch; height: stretch; padding: 16px"><${component} ${props} /></div>\n`);
  }
  const cook = Bun.spawnSync([ogBin(), "cook", root], { cwd: root, env: { ...process.env, OG_STYLES_DIR: STYLES }, stdout: "ignore", stderr: "pipe" });
  if (cook.exitCode !== 0) throw new Error(`cooking ${root}: ${new TextDecoder().decode(cook.stderr)}`);
});

afterAll(() => {
  // root stays "" when setup throws before mkdtempSync; rmSync("") would delete the working directory.
  if (root) rmSync(root, { recursive: true, force: true });
});

const DOCUMENTS = () => Object.keys(MEMBERS).map((m) => `ui/host-${m}.svelte`);

type View = { width: number; height: number; dpr: number; safeArea: { left: number; top: number; right: number; bottom: number }; coarsePointer: boolean; breakpoint: string };
const DESKTOP: View = { width: 1280, height: 720, dpr: 1.5, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktopHd" };
const PHONE: View = { width: 390, height: 844, dpr: 2, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" };

/** An engine mounting `component` dressed by the fantasy pack, its document events collected in `events`. */
function engine(component: string, stores: Record<string, unknown>, o: { events?: Event[]; view?: View } = {}): Engine {
  const events = o.events ?? [];
  const Collect = defineSystem("GeneralKitEvents", {}, (_q, _t, world) => {
    for (const e of og.ui.docEvents(world)) events.push(e as unknown as Event);
  });
  const r = createEngine(og.game({
    systems: [Collect],
    start(world) {
      const style = og.load.style(world, "@styles/fantasy");
      if (!style.ok) throw new Error(style.reason);
      og.ui.configure(world, { style: style.value });
      for (const path of DOCUMENTS()) {
        const loaded = og.load.document(world, path);
        if (!loaded.ok) throw new Error(`${path}: ${loaded.reason}`);
      }
    },
    ui(world) {
      const scope = og.ui.scope(world, "t");
      const inputs = Object.fromEntries(Object.entries(stores).map(([k, v]) => [k, Array.isArray(v) ? scope.collection(k, v as { id: string }[], { key: "id" }) : scope.writable(k, v)]));
      const mounted = scope.mount("page", `Host_${component}`, inputs);
      if (!mounted.ok) throw new Error(mounted.reason);
    },
  }) as unknown as GameToken, { root });
  if (!r.ok) throw new Error(r.reason);
  og.ui.previewViewport(r.value.world, (o.view ?? DESKTOP) as never);
  return r.value;
}

const elements = (e: Engine): El[] => (e.probe.dump({ what: "ui" }) as unknown as { elements: El[] }).elements;
const find = (e: Engine, path: RegExp): El | undefined => elements(e).find((x) => path.test(x.path));
const centre = (el: El): string => `${el.x + el.width / 2},${el.y + el.height / 2}`;
const tap = (e: Engine, el: El): void => {
  e.probe.commands(`0:touch_move:${centre(el)};0.02:touch_start;0.06:touch_end`);
  e.advance(8);
};
const intents = (events: Event[], names: string[]) => events.filter((e) => names.includes(e.name)).map((e) => `${e.name}:${e.rowKey}`);
const member = (name: string, fixture = "default", over: Record<string, unknown> = {}, o: Parameters<typeof engine>[2] = {}) =>
  engine(MEMBERS[name]!.component, { ...MEMBERS[name]!.fixtures[fixture]!, ...over }, o);
const run = (e: Engine, f: (e: Engine) => void) => {
  try {
    e.advance(3);
    f(e);
  } finally {
    e.destroy();
  }
};
const key = (e: Engine, k: string): void => {
  e.probe.commands(`0:key_down:${k};0.05:key_up:${k}`);
  e.advance(8);
};
const pad = (e: Engine, spec: string): void => {
  e.probe.commands(`0:gamepad:${spec};0.05:gamepad:-`);
  e.advance(8);
};
const errors = (e: Engine): readonly string[] => og.ui.scope(e.world, "t").takeErrors();
const row = (e: Engine, k: string): El => find(e, new RegExp(`#${k}$`))!;
/** The element with id `id` inside the row keyed `k`. */
const inRow = (e: Engine, k: string, id: string): El => {
  const el = elements(e).find((x) => x.path.includes(`#${k}/`) && new RegExp(`/${id}(#[^/]*)?$`).test(x.path));
  if (!el) throw new Error(`no ${id} in row ${k}`);
  return el;
};
const byId = (e: Engine, id: string): El => find(e, new RegExp(`/${id}$`))!;
const texts = (e: Engine): string[] => elements(e).map((x) => x.text).filter(Boolean);
/** Whether a drawn text reads `w`; the pack sets some labels in capitals (text-transform), so case is not compared. */
const has = (e: Engine, w: string): boolean => texts(e).some((x) => x.toLowerCase() === w.toLowerCase());
const said = (e: Engine, words: string[]): string[] => words.filter((w) => !has(e, w));
/** The button inside the row keyed `k` (a list item wraps it), if it holds one. */
const cell = (e: Engine, k: string): El | undefined => elements(e).find((x) => x.path.includes(`#${k}/`) && x.role === "button");
/** The day tile inside the calendar slot keyed `k`: its first element, a button only while it can be claimed. */
const day = (e: Engine, k: string): El => {
  const el = elements(e).find((x) => x.path.includes(`#${k}/`));
  if (!el) throw new Error(`no day in slot ${k}`);
  return el;
};
/** Whether the row keyed `k` shows an element with id `id`. */
const shows = (e: Engine, k: string, id: string): boolean => elements(e).some((x) => x.path.includes(`#${k}/`) && new RegExp(`/${id}(#[^/]*)?$`).test(x.path));

test("every part a member names in its markup is in its parts list, and every listed part is named", () => {
  for (const [name, m] of Object.entries(MEMBERS)) {
    const text = readFileSync(join(GROUP, `${name}.svelte`), "utf8");
    const named = new Set([...text.matchAll(/part="([A-Za-z]+\.[a-z]+)/g)].map((x) => x[1]!));
    expect(`${name}: ${[...named].sort().join(" ")}`).toBe(`${name}: ${[...m.parts].sort().join(" ")}`);
    expect(m.parts.every((p) => p.startsWith(`${m.component}.`))).toBeTrue();
  }
});

test("Shop: a tile reports buy by key; sold out, owned and unaffordable tiles are marked disabled, silent and say why", () => {
  const events: Event[] = [];
  run(member("shop", "default", {}, { events }), (e) => {
    for (const k of ["cape", "key", "dragon"]) expect(`${k} ${row(e, k).disabled}`).toBe(`${k} true`);
    expect(row(e, "potion").disabled).toBeFalsy();
    // A refused tile names its verdict; an open one is named by its offer alone.
    expect(["cape", "key", "dragon", "potion"].map((k) => row(e, k).name)).toEqual(["Ember Cape, Owned", "Vault Key, Sold out", "Dragon Mount, Not enough", "Potion Pack"]);
    tap(e, row(e, "cape"));
    tap(e, row(e, "key"));
    tap(e, row(e, "dragon"));
    tap(e, row(e, "potion"));
    tap(e, row(e, "items"));
    key(e, "KeyE");
    pad(e, "leftBumper");
    expect(intents(events, ["buy", "choosetab", "tabNext", "tabPrev"])).toEqual(["buy:potion", "choosetab:items", "tabNext:", "tabPrev:"]);
    expect(said(e, ["Sold out", "Owned", "Not enough", "-30%", "12,450", "1,200", "40"])).toEqual([]);
    // A struck price says what it was, for a screen reader.
    expect(elements(e).some((x) => x.name === "Was 1,000")).toBeTrue();
  });
});

test("Shop: the selection follows its offer's key through a reorder", () => {
  run(member("shop"), (e) => {
    expect(row(e, "bundle").states).toContain("selected");
    const offers = og.ui.scope(e.world, "t").collection("offers", MEMBERS.shop!.fixtures.default!.offers as { id: string }[], { key: "id" });
    offers.move("potion", "bundle");
    e.advance(3);
    expect(row(e, "bundle").states).toContain("selected");
    expect(row(e, "potion").states ?? []).not.toContain("selected");
    expect(row(e, "bundle").x).toBeGreaterThan(row(e, "potion").x);
  });
});

test("Shop: the failed catalogue offers Try again; loading and empty say so", () => {
  const events: Event[] = [];
  run(member("shop", "error", {}, { events }), (e) => {
    expect(has(e, "The shop could not be reached.")).toBeTrue();
    tap(e, byId(e, "Retry"));
    expect(intents(events, ["retry"])).toEqual(["retry:"]);
  });
  run(member("shop", "loading"), (e) => expect(has(e, "Loading the shop")).toBeTrue());
  run(member("shop", "empty"), (e) => expect(has(e, "Nothing for sale here right now")).toBeTrue());
});

test("ShopConfirm: Buy confirms; Cancel, Escape and B cancel; Enter presses the focused button, Cancel first; busy and refused purchases send nothing", () => {
  const events: Event[] = [];
  run(member("shop-confirm", "default", {}, { events }), (e) => {
    expect(said(e, ["Left after: 320 → 0", "1,000", "700", "Founder Bundle"])).toEqual([]);
    tap(e, byId(e, "Buy"));
    tap(e, byId(e, "Cancel"));
    key(e, "Escape");
    expect(intents(events, ["confirm", "cancel"])).toEqual(["confirm:", "cancel:", "cancel:"]);
  });
  const padded: Event[] = [];
  run(member("shop-confirm", "default", {}, { events: padded }), (e) => {
    pad(e, "b");
    expect(intents(padded, ["confirm", "cancel"])).toEqual(["cancel:"]);
  });
  const keyed: Event[] = [];
  run(member("shop-confirm", "default", {}, { events: keyed }), (e) => {
    key(e, "Enter");
    expect(intents(keyed, ["confirm", "cancel"])).toEqual(["cancel:"]);
  });
  for (const fixture of ["loading", "disabled"]) {
    const silent: Event[] = [];
    run(member("shop-confirm", fixture, {}, { events: silent }), (e) => {
      expect(byId(e, "Buy").disabled).toBeTrue();
      tap(e, byId(e, "Buy"));
      expect(intents(silent, ["confirm"])).toEqual([]);
    });
  }
  run(member("shop-confirm", "loading"), (e) => expect(has(e, "Purchasing")).toBeTrue());
  run(member("shop-confirm", "disabled"), (e) => expect(has(e, "Not enough")).toBeTrue());
});

test("Achievements: Claim reports its key; a hidden one shows as hidden; states, dates and rarity are words", () => {
  const events: Event[] = [];
  run(member("achievements", "default", {}, { events }), (e) => {
    tap(e, inRow(e, "first", "Claim"));
    expect(intents(events, ["claim"])).toEqual(["claim:first"]);
    expect(has(e, "Hidden achievement")).toBeTrue();
    expect(texts(e).some((w) => /truth|beneath/i.test(w))).toBeFalse();
    // Nor does its accessible name, or any other element's.
    expect(row(e, "secret").name).toBe("Hidden achievement");
    expect(elements(e).some((x) => /truth|beneath/i.test(x.name ?? ""))).toBeFalse();
    // The tier is a word, not a colour alone.
    expect(said(e, ["Unlocked 12 Nov 2026", "Locked", "64 / 100", "18.4% of players", "Keep playing to reveal it.", "Rare", "Legendary"])).toEqual([]);
    // One Claim only: the reward waiting on First Steps.
    expect(elements(e).filter((x) => /\/Claim(#[^/]*)?$/.test(x.path)).length).toBe(1);
  });
  const list = MEMBERS.achievements!.fixtures.default!.achievements as { id: string }[];
  run(member("achievements", "default", { achievements: list.filter((a) => a.id === "pet") }), (e) => {
    expect(has(e, "Claimed")).toBeTrue();
    expect(elements(e).some((x) => /\/Claim(#[^/]*)?$/.test(x.path))).toBeFalse();
  });
  // Claimed wins over claimable, and an unlocked one without a date says Unlocked alone.
  const silent: Event[] = [];
  run(member("achievements", "default", { achievements: list.filter((a) => a.id === "pet").map((a) => ({ ...a, claimable: true, date: "" })) }, { events: silent }), (e) => {
    expect(shows(e, "pet", "Claim")).toBeFalse();
    expect(has(e, "Unlocked")).toBeTrue();
    expect(intents(silent, ["claim"])).toEqual([]);
  });
  run(member("achievements", "empty"), (e) => expect(has(e, "No achievements yet")).toBeTrue());
});

test("DailyQuests: a done quest's Claim reports its key; the refresh time is the game's", () => {
  const events: Event[] = [];
  run(member("daily-quests", "default", {}, { events }), (e) => {
    expect(elements(e).filter((x) => /\/Claim(#[^/]*)?$/.test(x.path)).length).toBe(1);
    tap(e, inRow(e, "gather", "Claim"));
    expect(intents(events, ["claim"])).toEqual(["claim:gather"]);
    expect(said(e, ["Resets in 4h 12m", "Claimed", "2 / 3"])).toEqual([]);
  });
  run(member("daily-quests", "loading"), (e) => expect(has(e, "Loading quests")).toBeTrue());
  run(member("daily-quests", "empty"), (e) => expect(has(e, "No quests today")).toBeTrue());
});

test("DailyRewards: only today's tile is a button and claims, by its key, and only while the game allows it; a day says the state it was given", () => {
  const events: Event[] = [];
  run(member("daily-rewards", "default", {}, { events }), (e) => {
    for (const k of ["d1", "d3", "d5", "d7"]) expect(`${k} ${cell(e, k) === undefined}`).toBe(`${k} true`);
    for (const k of ["d1", "d3", "d5"]) tap(e, day(e, k));
    tap(e, cell(e, "d4")!);
    expect(intents(events, ["claim"])).toEqual(["claim:d4"]);
    // Today keeps its day label beside Claim.
    expect(said(e, ["Claimed", "Missed", "Upcoming", "Day 4", "Claim", "Legend chest"])).toEqual([]);
    expect(["d1", "d3", "d4", "d5"].map((k) => day(e, k).name)).toEqual(["Day 1, x100, Claimed", "Day 3, x20, Missed", "Day 4, x300, Claim", "Day 5, x1, Upcoming"]);
  });
  // Today without a claim is no button and says Today; the component infers nothing from canClaim.
  const waiting: Event[] = [];
  run(member("daily-rewards", "default", { canClaim: false }, { events: waiting }), (e) => {
    expect(cell(e, "d4")).toBeUndefined();
    expect(day(e, "d4").name).toBe("Day 4, x300, Today");
    expect(has(e, "Today")).toBeTrue();
    expect(has(e, "Claim")).toBeFalse();
    tap(e, day(e, "d4"));
    expect(intents(waiting, ["claim"])).toEqual([]);
  });
  // A reward already taken is the state the game sends for that day.
  const silent: Event[] = [];
  run(member("daily-rewards", "disabled", {}, { events: silent }), (e) => {
    expect(elements(e).some((x) => /#d\d+\//.test(x.path) && x.role === "button")).toBeFalse();
    expect(day(e, "d3").name).toBe("Day 3, x20, Claimed");
    tap(e, day(e, "d3"));
    expect(intents(silent, ["claim"])).toEqual([]);
  });
  // Three engines.
}, 180_000);

test("Chat: a channel reports its key, typing writes the draft back through bind:draft, Send sends; a muted chat sends nothing", () => {
  const events: Event[] = [];
  run(member("chat", "default", {}, { events }), (e) => {
    tap(e, row(e, "guild"));
    tap(e, byId(e, "Draft"));
    e.probe.commands("0:type:hello");
    e.advance(20);
    let draft: unknown;
    og.ui.scope(e.world, "t").writable("draft", "").subscribe((v) => { draft = v; })();
    expect(draft).toBe("hello");
    expect(byId(e, "Draft").text).toBe("hello");
    tap(e, byId(e, "Send"));
    // Typing E and Q into the field must not switch channels: the chat binds no tab keys.
    expect(intents(events, ["choosetab", "send", "tabNext", "tabPrev"])).toEqual(["choosetab:guild", "send:"]);
    expect(said(e, ["You", "System", "Aerin", "20:15", "Count me in, I will bring potions."])).toEqual([]);
  });
  const silent: Event[] = [];
  run(member("chat", "disabled", {}, { events: silent }), (e) => {
    expect(byId(e, "Send").disabled).toBeTrue();
    tap(e, byId(e, "Send"));
    expect(intents(silent, ["send"])).toEqual([]);
    expect(has(e, "You are muted for 5 minutes.")).toBeTrue();
  });
  run(member("chat", "empty"), (e) => expect(has(e, "No messages yet. Say hello!")).toBeTrue());
});

test("Chat: an overflowing log shows the newest message at its foot, and a long word breaks inside it", () => {
  run(member("chat", "overflow"), (e) => {
    const log = elements(e).find((x) => x.role === "log")!;
    const last = elements(e).find((x) => x.text === "Hello everyone!")!;
    expect(last.y + last.height).toBeLessThanOrEqual(log.y + log.height + 0.5);
    expect(last.y).toBeGreaterThan(log.y + log.height / 2);
  });
  const word = "Supercalifragilisticexpialidocious-wordthatdoesnotbreaknaturallyanywhereatallsoitmustbreakanywhere";
  run(member("chat", "default", { messages: [{ id: "w", kind: "player", sender: "Max", time: "20:01", text: word }] }, { view: PHONE }), (e) => {
    const log = elements(e).find((x) => x.role === "log")!;
    const text = elements(e).find((x) => x.text.startsWith("Supercalifragilistic"))!;
    expect(text.x + text.width).toBeLessThanOrEqual(log.x + log.width + 0.5);
    expect(text.height).toBeGreaterThan(30);
  });
});

test("Friends: accept, decline, invite, choose and add report by key; a pending invite is disabled; status is in words", () => {
  const events: Event[] = [];
  run(member("friends", "default", {}, { events }), (e) => {
    tap(e, inRow(e, "r1", "Accept"));
    tap(e, inRow(e, "r1", "Decline"));
    tap(e, inRow(e, "lys", "Invite"));
    expect(inRow(e, "maren", "Invite").disabled).toBeTrue();
    // No Invite where the game refuses one.
    expect(["aerin", "lys", "maren", "sela"].map((k) => shows(e, k, "Invite"))).toEqual([false, true, true, false]);
    tap(e, inRow(e, "maren", "Invite"));
    tap(e, row(e, "aerin"));
    tap(e, byId(e, "Add"));
    expect(intents(events, ["accept", "decline", "invite", "choose", "add"])).toEqual(["accept:r1", "decline:r1", "invite:lys", "choose:aerin", "add:"]);
    expect(said(e, ["In game", "Online", "Away", "Offline", "Invited", "Ranked match", "2 mutual friends", "Friend requests"])).toEqual([]);
  });
  run(member("friends", "empty"), (e) => {
    expect(has(e, "No friends yet")).toBeTrue();
    expect(byId(e, "Add")).toBeDefined();
  });
});

test("keyboard focus walks the shop's tiles from the first, Enter and the pad's A buy the focused one, and a refused tile takes the focus but buys nothing", () => {
  const events: Event[] = [];
  run(member("shop", "default", { tabs: [] }, { events }), (e) => {
    const focused = () => elements(e).find((x) => x.states?.includes("focused"))?.path ?? "";
    expect(focused()).toMatch(/#bundle$/);
    key(e, "ArrowRight");
    expect(focused()).toMatch(/#potion$/);
    pad(e, "a");
    key(e, "Enter");
    expect(intents(events, ["buy"])).toEqual(["buy:potion", "buy:potion"]);
    // The owned cape is next: a player can reach it to learn why, and neither key buys it.
    key(e, "ArrowRight");
    expect(focused()).toMatch(/#cape$/);
    pad(e, "a");
    key(e, "Enter");
    expect(intents(events, ["buy"])).toEqual(["buy:potion", "buy:potion"]);
  });
});

for (const [view, viewport] of [["desktop", DESKTOP], ["phone", PHONE]] as const) {
  test(`every member mounts from every fixture on a ${view}, dressed by the pack, inside the screen's width`, () => {
    for (const [name, m] of Object.entries(MEMBERS)) {
      for (const fixture of Object.keys(m.fixtures)) {
        run(member(name, fixture, {}, { view: viewport }), (e) => {
          e.advance(3);
          expect(`${name}/${fixture}: ${errors(e).join("; ")}`).toBe(`${name}/${fixture}: `);
          const box = find(e, /^t:page\/[^/]+$/)!;
          expect(`${name}/${fixture} ${box.x + box.width <= viewport.width + 0.5}`).toBe(`${name}/${fixture} true`);
        });
      }
    }
  }, 300_000);
}
