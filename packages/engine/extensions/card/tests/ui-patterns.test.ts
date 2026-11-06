import { expect, test } from "bun:test";

import { og } from "@origingame/engine";

import { card, type DeckBuilderResult } from "@origingame/engine/extensions/card/index.ts";

import { hand, REWARDS, rewards, TARGETS } from "./support/patterns.ts";

import { drag, cardEl, byPath, elements, type Element, key, run, tapOn } from "../../../tests/support/patterns.ts";

import "../../../tests/support/patterns.ts";

test("cardHand: a card dragged above the play line plays on release; let go below it, it goes back", () => {
  // Wrong: a release anywhere plays (a misdrag spends the card), or a release never does.
  const h = hand();
  drag(h.engine, cardEl(h.engine, "defend"), "640,640");
  expect(h.played).toEqual([]);
  expect(cardEl(h.engine, "defend")).toBeDefined();
  drag(h.engine, cardEl(h.engine, "defend"), "640,200");
  expect(h.played).toEqual([{ key: "defend", target: null }]);
  expect(byPath(h.engine, "Fan/defend") === undefined).toBe(true);
});

test("cardHand: a card that needs a target is aimed with an arc and plays only on a valid target", () => {
  // Wrong: a targeted card played on release above the line with no target, or on the taunted enemy.
  const h = hand();
  drag(h.engine, cardEl(h.engine, "bash"), "640,200");
  expect(h.played).toEqual([]);
  drag(h.engine, cardEl(h.engine, "bash"), "1060,170");
  expect(h.played).toEqual([]);
  drag(h.engine, cardEl(h.engine, "bash"), "840,170");
  expect(h.played).toEqual([{ key: "bash", target: "cultist" }]);
});

test("cardHand: while aiming, the arc and the result say which target is under the tip", () => {
  const h = hand();
  const s = cardEl(h.engine, "strike");
  const [fx, fy] = [Math.round(s.x + s.width / 2), Math.round(s.y + s.height / 2)];
  h.engine.probe.commands(`0:touch_move:${fx},${fy};0.05:touch_start;0.1:touch_move:${fx},400;0.2:touch_move:1060,170`);
  h.engine.advance(20);
  expect(h.results.at(-1)!.aiming).toMatchObject({ key: "strike", target: "louse" });
  expect(byPath(h.engine, "Hand/Fan/Arc")!.name).toBe("invalid: Louse");
  h.engine.probe.commands("0:touch_move:840,170");
  h.engine.advance(6);
  expect(byPath(h.engine, "Hand/Fan/Arc")!.name).toBe("valid: Cultist");
  // The aimed card stays over the hand, laid out larger than at rest (150 wide; its bounds at rest are turned).
  expect(cardEl(h.engine, "strike").width).toBeGreaterThan(150 * 1.25);
  h.engine.probe.commands("0:touch_end");
  h.engine.advance(6);
  expect(h.played).toEqual([{ key: "strike", target: "cultist" }]);
});

test("cardHand: an unplayable card shakes and is refused; it is said disabled", () => {
  const h = hand();
  expect(cardEl(h.engine, "wound").states).toContain("disabled");
  drag(h.engine, cardEl(h.engine, "wound"), "640,200");
  expect(h.played).toEqual([]);
  expect(h.results.some((r) => r.refused === "wound")).toBe(true);
});

test("cardHand: hovering lifts a card clear of the fan; its neighbours part", () => {
  // Wrong: the lifted card scaled in place (its text drawn small) or covered by its neighbour.
  const h = hand();
  const before = elements(h.engine);
  const at = (es: Element[], k: string) => es.find((e) => e.path.endsWith(`Fan/${k}`))!;
  const b = at(before, "bash");
  h.engine.probe.commands(`0:touch_move:${Math.round(b.x + b.width / 2)},${Math.round(b.y + b.height / 2)}`);
  h.engine.advance(40);
  const after = elements(h.engine);
  expect(at(after, "bash").width).toBeGreaterThan(b.width * 1.2);
  expect(at(after, "bash").y).toBeLessThan(b.y - 40);
  expect(at(after, "bash").states).toContain("selected");
  expect(at(after, "defend").x).toBeLessThan(at(before, "defend").x);
  expect(at(after, "anger").x).toBeGreaterThan(at(before, "anger").x);
  expect(h.results.at(-1)!.hovered).toBe("bash");
});

test("cardHand: a click picks a card and a second click plays it", () => {
  const h = hand();
  const tapAt = (e: Element) => h.engine.probe.commands(`0:touch_move:${Math.round(e.x + e.width / 2)},${Math.round(e.y + e.height / 2)};0.05:touch_start;0.1:touch_end`);
  tapAt(cardEl(h.engine, "anger"));
  h.engine.advance(30);
  expect(h.played).toEqual([]);
  tapAt(cardEl(h.engine, "anger"));
  h.engine.advance(12);
  expect(h.played).toEqual([{ key: "anger", target: null }]);
});

test("cardHand: keys walk the hand, Enter aims a targeted card, the arrows step the targets, Enter plays", () => {
  // Wrong: keys play a targeted card with no target, or step onto targets the card cannot take without saying so.
  const h = hand();
  key(h.engine, "Tab");
  expect(byPath(h.engine, "Hand/Fan")!.states).toContain("focused");
  expect(h.results.at(-1)!.hovered).toBe("strike");
  key(h.engine, "ArrowRight");
  key(h.engine, "ArrowRight");
  expect(h.results.at(-1)!.hovered).toBe("bash");
  key(h.engine, "Enter");
  expect(h.results.at(-1)!.aiming).toMatchObject({ key: "bash", target: "cultist" });
  key(h.engine, "ArrowRight");
  expect(h.results.at(-1)!.aiming).toMatchObject({ key: "bash", target: "louse" });
  key(h.engine, "Enter");
  expect(h.played).toEqual([]);
  key(h.engine, "ArrowRight");
  key(h.engine, "Enter");
  expect(h.played).toEqual([{ key: "bash", target: "cultist" }]);
  // The cursor stays at its place (now Anger); back to Strike, aimed, then Back stops aiming without playing.
  expect(h.results.at(-1)!.hovered).toBe("anger");
  key(h.engine, "ArrowLeft");
  key(h.engine, "ArrowLeft");
  key(h.engine, "Enter");
  expect(h.results.at(-1)!.aiming?.key).toBe("strike");
  key(h.engine, "Escape");
  expect(h.results.at(-1)!.aiming).toBeNull();
  expect(h.played.length).toBe(1);
});

test("cardHand: right to left, the hand fans from the right", () => {
  const h = hand(true);
  expect(cardEl(h.engine, "strike").x).toBeGreaterThan(cardEl(h.engine, "wound").x);
});

test("rewardChoice: cards that outgrow the room scroll between the header and the actions, which stay in it", () => {
  // Wrong: the whole choice laid out past the room (a phone's Skip off the screen), or scrolled with the cards so its
  // actions go out of view under the fade.
  const options = [...REWARDS, ...REWARDS.map((o) => ({ ...o, key: `${o.key}2` }))].map((o) => ({ ...o, locked: false }));
  const phone = { width: 390, height: 844, dpr: 3, coarsePointer: true, breakpoint: "phonePortrait", safeArea: { left: 0, top: 47, right: 0, bottom: 34 } } as const;
  for (const [viewport, h] of [[phone, 691], [{}, 520]] as const) {
    const e = run(() => og.ui.column("Box", { width: { kind: "grow" }, height: { kind: "fixed", px: h } }, () => card.rewardChoice("Rew", { title: "Room cleared", subtitle: "Choose a blessing", options, rerolls: 2, banishes: 1, skipLabel: "Skip · take 25 gold" })), viewport);
    e.advance(30);
    const box = byPath(e, "Box")!;
    const actions = ["Reroll", "Banish", "Skip"].map((k) => byPath(e, `Rew/Actions/${k}`)!);
    for (const a of actions) expect(a.y + a.height).toBeLessThanOrEqual(box.y + box.height + 0.5);
    const cards = byPath(e, "Rew/Cards")!;
    expect(cards.height).toBeLessThan(h);
    for (const a of actions) expect(a.y).toBeGreaterThanOrEqual(cards.y + cards.height - 0.5);
  }
});

test("rewardChoice: with one pick a press takes the reward at once; a locked one shakes and is refused", () => {
  // Wrong: a locked reward taken, or a single pick waiting for a confirm it never shows.
  // Its place as the pointer finds it (og.ui.bounds: where it was drawn, its transform in), read in the system.
  const xs: number[] = [];
  const r = rewards({}, () => xs.push(og.ui.bounds("Rew/Cards/Options/crown")?.x ?? -1));
  r.engine.advance(90);
  const rest = xs.at(-1)!;
  xs.length = 0;
  r.press("crown");
  r.engine.advance(30);
  // Pressed, it stood off its place for a few ticks (the shake), then came back to it.
  expect(xs.some((x) => Math.abs(x - rest) >= 1)).toBe(true);
  expect(Math.abs(xs.at(-1)! - rest)).toBeLessThan(0.5);
  expect(r.picked()).toEqual([]);
  expect(r.results.some((x) => x.refused === "crown")).toBe(true);
  expect(byPath(r.engine, "Rew/Cards/Options/crown")!.states).toContain("disabled");
  r.press("vigor");
  expect(r.picked()).toEqual(["vigor"]);
  expect(byPath(r.engine, "Rew/Actions/Confirm") === undefined).toBe(true);
});

test("rewardChoice: with two picks presses select, a third press does not overflow, Confirm takes them in offer order", () => {
  // Wrong: a press takes at once, the selection grows past picks, or the order follows the presses.
  const r = rewards({ picks: 2 });
  r.press("vigor");
  r.press("strike");
  expect(r.picked()).toEqual([]);
  expect(byPath(r.engine, "Rew/Cards/Options/vigor")!.states).toContain("checked");
  r.press("crown");
  expect(r.results.at(-1)!.selected).toEqual(["strike", "vigor"]);
  r.press("Confirm");
  expect(r.picked()).toEqual(["strike", "vigor"]);
});

test("rewardChoice: Banish arms, every card shows its cross, the next press banishes rather than takes; Back disarms", () => {
  // Wrong: an armed press takes the reward, the arming stays after a banish, or Back leaves it armed.
  const r = rewards({ banishes: 2 });
  r.press("Banish");
  expect(byPath(r.engine, "Rew/Cards/Options/strike/Banish")).toBeDefined();
  r.press("strike");
  expect(r.picked()).toEqual([]);
  expect(r.options().map((o) => o.key)).toEqual(["vigor", "crown"]);
  expect(byPath(r.engine, "Rew/Cards/Options/vigor/Banish") === undefined).toBe(true);
  r.press("Banish");
  key(r.engine, "Escape");
  expect(byPath(r.engine, "Rew/Cards/Options/vigor/Banish") === undefined).toBe(true);
  r.press("vigor");
  expect(r.picked()).toEqual(["vigor"]);
});

test("rewardChoice: Reroll offers new rewards until none are left, then is disabled", () => {
  const r = rewards({ rerolls: 1 });
  r.press("Reroll");
  expect(r.options()[0]!.key).toBe("strike2");
  expect(byPath(r.engine, "Rew/Actions/Reroll")!.disabled).toBe(true);
  r.press("Reroll");
  expect(r.options()[0]!.key).toBe("strike2");
});

test("rewardChoice: keys walk the rewards and Enter takes the focused one", () => {
  // Wrong: the locked reward taken by Enter, or the keys skipping the cards.
  const r = rewards();
  key(r.engine, "Tab");
  expect(r.results.at(-1)!.hovered).toBe("strike");
  key(r.engine, "Tab");
  key(r.engine, "Enter");
  expect(r.picked()).toEqual(["vigor"]);
});

test("cardHand: let go on a target it cannot take, the card is refused there (it shakes) and the result names that target", () => {
  // Wrong: a release on the taunted enemy silently cancels (no shake, nothing to say why), or plays.
  const h = hand();
  const louse = TARGETS[1]!.rect!;
  drag(h.engine, cardEl(h.engine, "bash"), `${louse.x + 80},${louse.y + 90}`);
  expect(h.played).toEqual([]);
  expect(h.results.some((r) => r.refused === "bash" && r.aiming?.target === "louse")).toBe(true);
});

test("cardHand: on a phone, a refused drag leaves no card standing; the cursor's card lifts only for keys", () => {
  // Wrong: the press gave the fan the focus, and the focused fan lifts its cursor's card (the refused Wound) over the screen.
  const h = hand(false, 382, true);
  h.engine.advance(60);
  drag(h.engine, cardEl(h.engine, "wound"), "195,300");
  expect(h.results.some((r) => r.refused === "wound")).toBe(true);
  const standing = elements(h.engine).filter((e) => /Fan\/[a-z]+$/.test(e.path) && e.states.includes("selected"));
  expect(standing.map((e) => e.id)).toEqual([]);
});

// ── deck builder ────────────────────────────────────────────────────────

test("deckBuilder: a card at its copies' limit, one not owned, and any card into a full deck are refused with their reason; a deck row takes a copy out; 7+ holds every dearer card; the search filters by name", () => {
  // Wrong: a third copy added past the limit of two, an unowned card added, a full deck taking more, or a cost-9 card missing under 7+.
  const results: DeckBuilderResult[] = [];
  const st = { counts: { bolt: 2, wall: 0, ghost: 0, titan: 0 } as Record<string, number>, size: 30, cost: -1, search: "" };
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 1232 }, height: { kind: "fixed", px: 672 } }, () => {
      const r = card.deckBuilder("Deck", {
        cards: [
          { key: "bolt", name: "Bolt", cost: 1, kind: "spell", owned: 3, count: st.counts.bolt },
          { key: "wall", name: "Wall", cost: 2, kind: "unit", owned: 2, count: st.counts.wall },
          { key: "ghost", name: "Ghost", cost: 3, kind: "unit", owned: 0, count: 0 },
          { key: "titan", name: "Titan", cost: 9, kind: "unit", owned: 1, count: st.counts.titan },
        ],
        deckSize: st.size,
        copyLimit: 2,
        costFilter: st.cost,
        search: st.search,
      });
      results.push(r);
      if (r.add) st.counts[r.add] = (st.counts[r.add] ?? 0) + 1;
      if (r.remove) st.counts[r.remove] = (st.counts[r.remove] ?? 0) - 1;
      if (r.costFilter !== null) st.cost = r.costFilter;
    });
  });
  engine.advance(10);
  const tile = (k: string) => elements(engine).find((e) => /Grid\/Line#\d+\//.test(e.path) && e.path.endsWith(`/${k}`))!;
  tapOn(engine, tile("bolt"));
  expect(byPath(engine, "Deck/Refusal/Text")?.text).toBe("No more copies allowed");
  tapOn(engine, tile("ghost"));
  expect(byPath(engine, "Deck/Refusal/Text")?.text).toBe("Not owned");
  tapOn(engine, tile("wall"));
  expect(st.counts.wall).toBe(1);
  tapOn(engine, byPath(engine, "Deck/List/bolt")!);
  expect(st.counts.bolt).toBe(1);
  st.size = 2;
  engine.advance(4);
  tapOn(engine, tile("titan"));
  expect(byPath(engine, "Deck/Refusal/Text")?.text).toBe("The deck is full");
  expect(results.filter((r) => r.refused).map((r) => r.refused)).toEqual(["bolt", "ghost", "titan"]);
  expect(results.filter((r) => r.add).map((r) => r.add)).toEqual(["wall"]);
  tapOn(engine, byPath(engine, "Filters/Costs/7")!);
  expect(st.cost).toBe(7);
  expect(elements(engine).filter((e) => /Grid\/Line#\d+\/[a-z]+$/.test(e.path)).map((e) => e.id)).toEqual(["titan"]);
  st.cost = -1;
  st.search = "WA";
  engine.advance(4);
  expect(elements(engine).filter((e) => /Grid\/Line#\d+\/[a-z]+$/.test(e.path)).map((e) => e.id)).toEqual(["wall"]);
});
