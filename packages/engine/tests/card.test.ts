// The card components as a game uses them (docs/ui.md §6.1): built in, used
// by name from the game's own pages with nothing copied or loaded but the
// pages, dressed by the fantasy pack, each member mounted from every one of
// its fixtures and driven through the probe's touch, key and pad commands.
//
// Ways it could fail: an intent sent without its row key, or sent for a pile,
// a collection card, a deck step, an offer or a room the game refused, by a
// tap or by the row's action (X, pad Y); the selection following a row's
// index instead of its key; a card the game cannot play offering Play or
// playing from the pad; a member that only fits the desktop; a member
// not loaded with the page that uses it (CardFace and NodeGraph inside the
// others); a named part missing from the drawn tree.
import { afterAll, beforeAll, expect, setDefaultTimeout, test } from "bun:test";

// The tests run on ticks, not the clock; a time limit only guards a hang.
setDefaultTimeout(60_000);
import { cpSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { defineSystem, og } from "@origingame/engine";

declare const Bun: { spawnSync(cmd: string[], o: { cwd: string; env: Record<string, string | undefined>; stdout: "ignore"; stderr: "pipe" }): { exitCode: number; stderr: Uint8Array } };
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { ogBin } from "./support/og-bin.ts";

type El = { id: string; kind: string; path: string; text: string; x: number; y: number; width: number; height: number; states?: string[]; disabled?: boolean; name?: string; role?: string; part?: string };
type Event = { mount: string; path: string; rowKey: string; kind: string; name: string };

const ENGINE = dirname(dirname(fileURLToPath(import.meta.url)));
const GROUP = join(ENGINE, "src/ui/components/card");
const STYLES = join(ENGINE, "../../templates/styles");
let root = "";
let styles = "";

/** Each member's component name, parts and fixtures, read from its document as a game's tooling reads them. */
const MEMBERS: Record<string, { component: string; parts: string[]; fixtures: Record<string, Record<string, unknown>> }> = {};

beforeAll(() => {
  loadNativeHost();
  root = mkdtempSync(join(tmpdir(), "og-card-kit-"));
  styles = mkdtempSync(join(tmpdir(), "og-card-styles-"));
  for (const d of ["fantasy", "shared"]) cpSync(join(STYLES, d), join(styles, d), { recursive: true });
  mkdirSync(join(root, "assets/ui"), { recursive: true });
  writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "Card kit"\nid "card-kit"\nentry "main.ts"\n');
  writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});\n');
  // A look naming the pack is what makes the cook bundle it (@styles/fantasy).
  mkdirSync(join(root, "assets/looks"), { recursive: true });
  writeFileSync(join(root, "assets/looks/fantasy.ogt"), 'type look\nversion 1\n---\nname "card kit fantasy"\nui "fantasy"\n');
  for (const f of readdirSync(GROUP).filter((f) => f.endsWith(".svelte"))) {
    const text = readFileSync(join(GROUP, f), "utf8");
    const component = /<meta name="component" content="([^"]+)">/.exec(text)![1]!;
    const parts = /<meta name="parts" content="([^"]+)">/.exec(text)![1]!.split(" ");
    const fixtures = Object.fromEntries([...text.matchAll(/<meta name="fixture:([\w-]+)" content='([^']*)'>/g)].map((m) => [m[1]!, JSON.parse(m[2]!)]));
    const name = f.replace(".svelte", "");
    MEMBERS[name] = { component, parts, fixtures };
    // A game mounts its own page, which uses the member by name: one host page per member, its props from stores.
    const props = Object.keys(fixtures.default!).map((k) => `${k}={$${k}}`).join(" ");
    writeFileSync(join(root, `assets/ui/host-${name}.svelte`), `<meta name="screen" content="Host_${component}" data-layer="menu" data-input="menu">\n<div style="width: stretch; height: stretch; padding: 16px"><${component} ${props} /></div>\n`);
  }
  const cook = Bun.spawnSync([ogBin(), "cook", root], { cwd: root, env: { ...process.env, OG_STYLES_DIR: styles }, stdout: "ignore", stderr: "pipe" });
  if (cook.exitCode !== 0) throw new Error(`cooking ${root}: ${new TextDecoder().decode(cook.stderr)}`);
});

// Only the directories made above: when beforeAll fails before making one,
// rmSync("") would remove the working directory (the engine package).
afterAll(() => {
  if (root) rmSync(root, { recursive: true, force: true });
  if (styles) rmSync(styles, { recursive: true, force: true });
});

// Only the host page an engine mounts: each load reads the whole style pack again.
const DOCUMENTS = (component: string) => Object.keys(MEMBERS).filter((m) => MEMBERS[m]!.component === component).map((m) => `ui/host-${m}.svelte`);

type View = { width: number; height: number; dpr: number; safeArea: { left: number; top: number; right: number; bottom: number }; coarsePointer: boolean; breakpoint: string };
const DESKTOP: View = { width: 1280, height: 720, dpr: 1.5, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktopHd" };
const PHONE: View = { width: 390, height: 844, dpr: 2, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" };

/** An engine mounting `component` dressed by the fantasy pack, its document events collected in `events`. */
function engine(component: string, stores: Record<string, unknown>, o: { events?: Event[]; view?: View } = {}): Engine {
  const events = o.events ?? [];
  const Collect = defineSystem("CardKitEvents", {}, (_q, _t, world) => {
    for (const e of og.ui.docEvents(world)) events.push(e as unknown as Event);
  });
  const r = createEngine(og.game({
    systems: [Collect],
    start(world) {
      const style = og.load.style(world, "@styles/fantasy");
      if (!style.ok) throw new Error(style.reason);
      og.ui.configure(world, { style: style.value });
      for (const path of DOCUMENTS(component)) {
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
const key = (e: Engine, k: string): void => {
  e.probe.commands(`0:key_down:${k};0.05:key_up:${k}`);
  e.advance(8);
};
const pad = (e: Engine, spec: string): void => {
  e.probe.commands(`0:gamepad:${spec};0.05:gamepad:-`);
  e.advance(8);
};
const intents = (events: Event[], names: string[]) => events.filter((e) => names.includes(e.name)).map((e) => `${e.name}:${e.rowKey}`);
const member = (name: string, over: Record<string, unknown> = {}, o: Parameters<typeof engine>[2] = {}, fixture = "default") =>
  engine(MEMBERS[name]!.component, { ...MEMBERS[name]!.fixtures[fixture], ...over }, o);
const run = (e: Engine, f: (e: Engine) => void) => {
  try {
    e.advance(3);
    f(e);
  } finally {
    e.destroy();
  }
};
const errors = (e: Engine): readonly string[] => og.ui.scope(e.world, "t").takeErrors();
const row = (e: Engine, k: string): El => find(e, new RegExp(`#${k}$`))!;

/** The elements drawn inside the keyed row `k`. */
const within = (e: Engine, k: string): El[] => elements(e).filter((x) => x.path.includes(`#${k}/`) || x.path.endsWith(`#${k}`));
const button = (e: Engine, k: string, part: string): El => within(e, k).find((x) => x.role !== "" && x.part?.includes(part) && !x.path.includes("::part("))!;
const focused = (e: Engine): string => elements(e).find((x) => x.states?.includes("focused") && !x.path.includes("::part("))?.path ?? "";
/** Puts the focus in the keyed row `k`: a tap there may already have; otherwise each arrow in turn walks towards it. */
const focusRow = (e: Engine, k: string, arrows: string[]): void => {
  const there = () => new RegExp(`#${k}(/|$)`).test(focused(e));
  for (const a of arrows) {
    if (there()) break;
    key(e, a);
  }
  expect(`focus in ${k}: ${there()}`).toBe(`focus in ${k}: true`);
};

// One test per member and view: each fixture boots its own engine.
for (const [view, viewport] of [["desktop", DESKTOP], ["phone", PHONE]] as const) {
  for (const name of readdirSync(GROUP).filter((f) => f.endsWith(".svelte")).map((f) => f.replace(".svelte", ""))) {
    test(`${name} mounts from every fixture on a ${view}, dressed by the pack, inside the screen's width`, () => {
      for (const fixture of Object.keys(MEMBERS[name]!.fixtures)) {
        run(member(name, {}, { view: viewport }, fixture), (e) => {
          expect(`${name}/${fixture}: ${errors(e).join("; ")}`).toBe(`${name}/${fixture}: `);
          const box = find(e, /^t:page\/[^/]+$/)!;
          expect(`${name}/${fixture} ${box.x + box.width <= viewport.width + 0.5}`).toBe(`${name}/${fixture} true`);
        });
      }
    }, 120_000);
  }
}

for (const name of readdirSync(GROUP).filter((f) => f.endsWith(".svelte")).map((f) => f.replace(".svelte", ""))) {
  test(`every part ${name} names is drawn by one of its fixtures`, () => {
    const m = MEMBERS[name]!;
    const drawn = new Set<string>();
    for (const fixture of Object.keys(m.fixtures)) {
      run(member(name, {}, {}, fixture), (e) => {
        for (const x of elements(e)) for (const p of m.parts) if (x.part?.includes(`::part(${p})`)) drawn.add(p);
      });
    }
    expect(`${name}: ${m.parts.filter((p) => !drawn.has(p)).join(" ")}`).toBe(`${name}: `);
  }, 120_000);
}

test("CardHand: a tap selects by key, unplayable cards too; Play on the selected card and the pad's Y play by key", () => {
  // Nothing picked up, so no raised card covers its neighbours' edges.
  const picked: Event[] = [];
  run(member("hand", { selected: "" }, { events: picked }), (e) => {
    expect(button(e, "surge", "CardHand.card").name).toBe("Arcane Surge");
    expect(within(e, "surge").some((x) => x.part?.includes("::part(CardHand.badge)"))).toBeTrue();
    expect(within(e, "guard").some((x) => x.part?.includes("::part(CardHand.badge)"))).toBeFalse();
    tap(e, row(e, "guard"));
    tap(e, row(e, "surge"));
    expect(intents(picked, ["select", "play"])).toEqual(["select:guard", "select:surge"]);
  });
  const played: Event[] = [];
  run(member("hand", {}, { events: played }), (e) => {
    tap(e, within(e, "strike").find((x) => x.kind === "button" && x.name === "Play")!);
    const at = focused(e).match(/#(\w+)\//)?.[1] ?? "";
    expect(at).not.toBe("");
    pad(e, "y");
    expect(intents(played, ["select", "play"])).toEqual(["play:strike", `play:${at}`]);
  });
});
test("CardHand: a selected card the game cannot play offers no Play, and X and the pad's Y on it play nothing", () => {
  const events: Event[] = [];
  run(member("hand", {}, { events }, "disabled"), (e) => {
    expect(button(e, "surge", "CardHand.card").states).toContain("selected");
    expect(elements(e).some((x) => x.kind === "button" && x.name === "Play")).toBeFalse();
    // Every card of this hand is unplayable, so whichever has the focus must stay silent.
    tap(e, row(e, "surge"));
    expect(focused(e)).toMatch(/#(strike|surge)(\/|$)/);
    key(e, "KeyX");
    pad(e, "y");
    expect(intents(events, ["select", "play"])).toEqual(["select:surge"]);
  });
});

test("CardHand: the selection follows its key through a reorder, and the arrows walk the hand in order", () => {
  run(member("hand"), (e) => {
    const cards = og.ui.scope(e.world, "t").collection("cards", MEMBERS.hand!.fixtures.default!.cards as { id: string }[], { key: "id" });
    cards.move("strike", "wound");
    e.advance(3);
    expect(button(e, "strike", "CardHand.card").states).toContain("selected");
    expect(button(e, "guard", "CardHand.card").states).not.toContain("selected");
    expect(row(e, "strike").x).toBeGreaterThan(row(e, "surge").x);
  });
  // A flat row (fan 0), so the spatial arrows read the order without the fan's tilt.
  run(member("hand", { selected: "", fan: 0 }), (e) => {
    expect(focused(e)).toMatch(/#strike\//);
    key(e, "ArrowRight");
    expect(focused(e)).toMatch(/#guard\//);
    pad(e, "-:1,0");
    expect(focused(e)).toMatch(/#surge\//);
  });
});

test("CardHand: eleven cards overlap to fit a phone, the last whole", () => {
  // A flat row: a fan turns the end cards by the game's angle, past the row's box.
  run(member("hand", { fan: 0 }, { view: PHONE }, "overflow"), (e) => {
    const frame = elements(e).find((x) => x.part?.includes("::part(CardHand.frame)"))!;
    const first = row(e, "c1");
    const last = button(e, "c11", "CardHand.card");
    expect(first.x).toBeGreaterThanOrEqual(frame.x - 0.5);
    expect(last.x + last.width).toBeLessThanOrEqual(frame.x + frame.width + 0.5);
  });
});

test("CardPiles: a pile reports open by key; one the game will not open is disabled and silent", () => {
  const events: Event[] = [];
  run(member("piles", {}, { events }, "empty"), (e) => {
    tap(e, row(e, "draw"));
    expect(row(e, "exhaust").disabled).toBeTrue();
    tap(e, row(e, "exhaust"));
    expect(intents(events, ["open"])).toEqual(["open:draw"]);
    expect(row(e, "discard").name).toBe("Discard: 0");
  });
});

test("CardCollection: a card adds by key; a capped or locked one stays in reach and reports inspect, never add; the filters choose by key and the bumpers page them", () => {
  const events: Event[] = [];
  run(member("collection", {}, { events }), (e) => {
    tap(e, row(e, "strike"));
    // Not disabled: a refused card can still be focused and read.
    for (const k of ["guard", "crown"]) expect(`${k} disabled: ${row(e, k).disabled === true}`).toBe(`${k} disabled: false`);
    tap(e, row(e, "guard"));
    tap(e, row(e, "crown"));
    tap(e, row(e, "skill"));
    key(e, "KeyE");
    pad(e, "leftBumper");
    expect(intents(events, ["add", "inspect", "choosefilter", "tabNext", "tabPrev"])).toEqual(["add:strike", "inspect:guard", "inspect:crown", "choosefilter:skill", "tabNext:", "tabPrev:"]);
    expect(row(e, "guard").states).toContain("selected");
  });
});

test("CardCollection: X and the pad's Y inspect the focused card by key, one the game will not add too, and add nothing", () => {
  for (const k of ["strike", "guard"]) {
    const events: Event[] = [];
    run(member("collection", {}, { events }), (e) => {
      focusRow(e, k, ["ArrowDown", "ArrowRight"]);
      key(e, "KeyX");
      pad(e, "y");
      expect(`${k}: ${intents(events, ["add", "inspect"])}`).toBe(`${k}: inspect:${k},inspect:${k}`);
    });
  }
  // Two engines.
}, 180_000);

test("CardDeckList: plus and minus report by key, the game's refusals are silent, a row chooses and Confirm reports once enabled", () => {
  const events: Event[] = [];
  run(member("deck-list", {}, { events }), (e) => {
    tap(e, button(e, "quick", ".plus"));
    tap(e, button(e, "strike", ".plus"));
    tap(e, button(e, "surge", ".minus"));
    // A card the deck must keep: its minus is disabled and silent.
    expect(button(e, "oath", ".minus").disabled).toBeTrue();
    tap(e, button(e, "oath", ".minus"));
    tap(e, within(e, "guard").find((x) => x.kind === "button")!);
    tap(e, find(e, /\/Confirm$/)!);
    expect(find(e, /\/Confirm$/)!.disabled).toBeTrue();
    expect(intents(events, ["add", "remove", "choose", "confirm"])).toEqual(["add:quick", "remove:surge", "choose:guard"]);
    expect(elements(e).some((x) => x.text === "12 / 30")).toBeTrue();
  });
  // X and the pad's Y in a row are its minus: by key where the game allows the removal, silent where it refuses.
  for (const [k, want] of [["guard", "remove:guard,remove:guard"], ["oath", ""]] as const) {
    const pressed: Event[] = [];
    run(member("deck-list", {}, { events: pressed }), (e) => {
      tap(e, within(e, k).find((x) => x.kind === "button")!);
      focusRow(e, k, ["ArrowDown", "ArrowDown", "ArrowDown", "ArrowDown", "ArrowDown"]);
      key(e, "KeyX");
      pad(e, "y");
      expect(`${k}: ${intents(pressed, ["remove"])}`).toBe(`${k}: ${want}`);
    });
  }
  const confirmed: Event[] = [];
  run(member("deck-list", { canConfirm: true }, { events: confirmed }), (e) => {
    tap(e, find(e, /\/Confirm$/)!);
    expect(intents(confirmed, ["confirm"])).toEqual(["confirm:"]);
  });
  // Four engines.
}, 180_000);

test("CardReward: an offer picks by key, a refused one is silent, and Skip, Escape and the pad's B skip", () => {
  const events: Event[] = [];
  run(member("reward", {}, { events }), (e) => {
    tap(e, row(e, "mend"));
    tap(e, find(e, /\/Skip$/)!);
    key(e, "Escape");
    expect(intents(events, ["pick", "skip"])).toEqual(["pick:mend", "skip:", "skip:"]);
  });
  const padded: Event[] = [];
  run(member("reward", {}, { events: padded }), (e) => {
    pad(e, "b");
    expect(intents(padded, ["skip"])).toEqual(["skip:"]);
  });
  const relics: Event[] = [];
  run(member("reward", {}, { events: relics }, "relics"), (e) => {
    expect(row(e, "vial").disabled).toBeTrue();
    tap(e, row(e, "vial"));
    tap(e, row(e, "lantern"));
    expect(intents(relics, ["pick"])).toEqual(["pick:lantern"]);
  });
});

test("CardRunMap: a reachable room reports NodeGraph's pick by key; a locked room is disabled and silent", () => {
  const events: Event[] = [];
  run(member("run-map", {}, { events }), (e) => {
    tap(e, row(e, "e1"));
    for (const k of ["alt", "boss"]) {
      expect(`${k} disabled: ${row(e, k).disabled === true}`).toBe(`${k} disabled: true`);
      tap(e, row(e, k));
    }
    expect(intents(events, ["pick"])).toEqual(["pick:e1"]);
  });
});
