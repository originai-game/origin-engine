import { expect, test } from "bun:test";

import { og } from "@origingame/engine";

import { versus, type ChatMessage, type ChatResult, type DraftResult, type VersusResult } from "@origingame/engine/extensions/versus/index.ts";

import { party, selectScreen, ROSTER, chatRun } from "./support/patterns.ts";

import { elements, tapOn, byPath, key, run } from "../../../tests/support/patterns.ts";

import "../../../tests/support/patterns.ts";

test("lobby: the local player's card stands first and larger; a seat answers its place; the toggle asks to flip ready", () => {
  // Wrong: the party drawn in the game's order (the local player second), a seat answering its index among all cards, or ready flipped by the component.
  const p = party();
  const cards = elements(p.engine).filter((e) => /Lineup\/(kael|me|mira)$/.test(e.path)).sort((a, b) => a.x - b.x);
  expect(cards.map((c) => c.path.split("/").at(-1))).toEqual(["me", "kael", "mira"]);
  expect(cards[0]!.height).toBeGreaterThan(cards[1]!.height);
  // The cards rise in one after another: pressed once they stand.
  p.engine.advance(30);
  tapOn(p.engine, byPath(p.engine, "Lineup/Seat#0")!);
  expect(p.results.some((r) => r.invite === 0)).toBeTrue();
  expect(byPath(p.engine, "Bar/Action/Label")!.text).toBe("Ready up");
  tapOn(p.engine, byPath(p.engine, "Bar/Action")!);
  expect(p.results.filter((r) => r.ready).length).toBe(1);
  expect(byPath(p.engine, "Bar/Action/Label")!.text).toBe("Ready");
});

test("lobby: a ready check is accepted by confirm whatever has the focus, once; Back declines only before accepting", () => {
  // Wrong: the check answering only its button (a pad's A does nothing), accept answered every tick, or Back declining after accepting.
  const p = party(true);
  p.state.check = true;
  p.engine.advance(4);
  expect(byPath(p.engine, "Check/Panel/Title")!.text).toBe("Match found");
  key(p.engine, "Enter");
  expect(p.results.filter((r) => r.accept).length).toBe(1);
  expect(byPath(p.engine, "Check/Panel/Actions/Accept/Label")!.text).toBe("Accepted");
  key(p.engine, "Escape");
  expect(p.results.some((r) => r.decline)).toBeFalse();
});

test("characterSelect: a taken or locked tile is refused, an open one picked; the rival's tag stands on its tile; Back unlocks", () => {
  // Wrong: a taken character picked twice, a locked one picked, the rival's cursor not shown, or Back ignored once locked in.
  const s = selectScreen();
  expect(byPath(s.engine, "Roster/Row#0/brann/Tags/Tag#0/Text")!.text).toBe("P2");
  expect(byPath(s.engine, "Stage/Start/Words/Name")!.text).toBe("Kael");
  tapOn(s.engine, byPath(s.engine, "Roster/Row#0/mira")!);
  tapOn(s.engine, byPath(s.engine, "Roster/Row#1/vell") ?? byPath(s.engine, "Roster/Row#0/vell")!);
  expect(s.results.filter((r) => r.refused).map((r) => r.refused)).toEqual(["mira", "vell"]);
  expect(s.results.some((r) => r.picked)).toBeFalse();
  tapOn(s.engine, byPath(s.engine, "Roster/Row#0/aria")!);
  expect(s.results.filter((r) => r.picked).map((r) => r.picked)).toEqual(["aria"]);
  expect(byPath(s.engine, "Foot/Lock/Label")!.text).toBe("Locked in");
  key(s.engine, "Escape");
  expect(s.results.some((r) => r.cancelled)).toBeTrue();
});

test("characterSelect: a cursor the game moves stays where it put it; the focus asks to move it only when it changes tile", () => {
  // Wrong: the focused tile asked for the cursor every tick, so a cursor the game moved by other means (a pad's stick,
  // a random pick) snapped back to the tile the focus had stayed on.
  const s = selectScreen();
  expect(byPath(s.engine, "Stage/Start/Words/Name")!.text).toBe("Kael");
  s.me.entry = "aria";
  s.engine.advance(10);
  expect(s.me.entry).toBe("aria");
  expect(byPath(s.engine, "Stage/Start/Words/Name")!.text).toBe("Aria");
  key(s.engine, "ArrowRight");
  expect(s.me.entry).not.toBe("aria");
});

test("versus: both sides named in one region; confirm skips only once the sides have landed", () => {
  // Wrong: a skip answered the first tick (the press that opened it skips it at once), or a side left unnamed.
  const results: VersusResult[] = [];
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 1232 }, height: { kind: "fixed", px: 672 } }, () => {
      results.push(versus.versus("Vs", { left: { name: "Aria", character: "Kael" }, right: { name: "Mira", character: "Brann", team: 1 }, stage: "Round 1" }));
    });
  });
  expect(elements(engine).some((e) => e.role === "region" && e.name === "Kael VS Brann")).toBeTrue();
  engine.probe.commands("0:key_down:Enter;0.02:key_up:Enter");
  engine.advance(2);
  expect(results.some((r) => r.skipped)).toBeFalse();
  engine.advance(10);
  key(engine, "Enter");
  expect(results.filter((r) => r.skipped).length).toBeGreaterThan(0);
});

// ── draft and ban ───────────────────────────────────────────────────────

test("draft: a banned or picked character is refused; in the local turn a press hovers and the action locks the hovered one in", () => {
  // Wrong: a banned character hoverable, the action live outside the local turn, or the action locking something other than the hover.
  const results: DraftResult[] = [];
  const st = { hovered: "", yourTurn: false };
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 1232 }, height: { kind: "fixed", px: 672 } }, () => {
      const r = versus.draft("Draft", {
        teams: [
          { name: "Blue", team: 0, bans: ["vell"], picks: [{ player: "You", you: true, active: st.yourTurn, entry: st.hovered }, { player: "Kael" }] },
          { name: "Red", team: 1, bans: [""], picks: [{ player: "Nyx", entry: "mira", locked: true }, { player: "Zed" }] },
        ],
        roster: ROSTER.map((e) => ({ ...e, taken: false, locked: false })),
        phase: "pick",
        phaseLabel: "Pick",
        secondsLeft: 20,
        yourTurn: st.yourTurn,
        hovered: st.hovered,
      });
      results.push(r);
      if (r.hovered) st.hovered = r.hovered;
    });
  });
  const tile = (k: string) => elements(engine).find((e) => e.path.endsWith(`/${k}`) && e.path.includes("/Roster/"))!;
  tapOn(engine, tile("aria"));
  expect(results.some((r) => r.hovered)).toBeFalse();
  expect(byPath(engine, "Foot/Action/Label")!.text).toBe("Waiting");
  st.yourTurn = true;
  engine.advance(2);
  tapOn(engine, tile("vell"));
  tapOn(engine, tile("mira"));
  expect(results.filter((r) => r.refused).map((r) => r.refused)).toEqual(["vell", "mira"]);
  tapOn(engine, tile("aria"));
  expect(st.hovered).toBe("aria");
  expect(byPath(engine, "Start/Picks/Slot#0/Words/Character")!.text).toBe("Aria");
  tapOn(engine, byPath(engine, "Foot/Action")!);
  expect(results.filter((r) => r.locked).map((r) => r.locked)).toEqual(["aria"]);
});

test("chat: a player's words show as typed, markup and all; a link and an author answer their keys; @me lights the line", () => {
  // Wrong: a player's [b] read as markup, a link answered as the author (or not at all), a mention of another lit as one of mine.
  const results: ChatResult[] = [];
  const st = { draft: "", sendTo: "", messages: [
    { key: "m1", channel: "party", author: "Riven", authorKey: "riven", text: "[b]loot[/b] {0} for @Lumen", links: [{ key: "blade", label: "Ashbringer", kind: "item", rarity: "legendary" as const }] },
    { key: "m2", channel: "guild", author: "Juno", authorKey: "juno", text: "gg @Riven" },
  ] };
  const engine = chatRun(st, results);
  engine.advance(10);
  const words = (k: string): string => elements(engine).filter((e) => e.path.includes(`/Log/${k}/Text`)).map((e) => e.text).join("");
  expect(words("m1")).toContain("[b]loot[/b]");
  expect(words("m1")).toContain("[Ashbringer]");
  expect(byPath(engine, "Log/m1/Edge") !== undefined && byPath(engine, "Log/m2/Edge") === undefined).toBeTrue();
  tapOn(engine, byPath(engine, "Log/m1/Text/Lblade")!);
  tapOn(engine, byPath(engine, "Log/m2/Text/Ujuno")!);
  expect([results.filter((r) => r.link).map((r) => r.link), results.filter((r) => r.author).map((r) => r.author)]).toEqual([["blade"], ["juno"]]);
});

test("chat: an @word offers the members it starts, a press writes the name; Send posts to the chip's channel, which steps past a read-only one", () => {
  // Wrong: no suggestion, a suggestion that drops the rest of the draft, a send to System, or a send of nothing.
  const results: ChatResult[] = [];
  const st = { draft: "nice one @lu", sendTo: "", messages: [] as ChatMessage[] };
  const engine = chatRun(st, results);
  engine.advance(10);
  expect(byPath(engine, "Suggest/riven") === undefined && byPath(engine, "Suggest/luka") !== undefined).toBeTrue();
  tapOn(engine, byPath(engine, "Suggest/luka")!);
  engine.advance(4);
  expect(st.draft).toBe("nice one @Luka ");
  for (let i = 0; i < 2; i++) {
    tapOn(engine, byPath(engine, "Input/Channel")!);
    engine.advance(4);
  }
  expect(st.sendTo).toBe("party");
  tapOn(engine, byPath(engine, "Input/Send")!);
  expect(results.filter((r) => r.send).map((r) => [r.send!.channel, r.send!.text])).toEqual([["party", "nice one @Luka"]]);
});

test("chat: at its foot the log follows a new message; scrolled up, new ones gather in the pill, whose press brings the view down", () => {
  // Wrong: the view yanked down while the player reads back, or new messages arriving unseen with no pill.
  const results: ChatResult[] = [];
  const say = (i: number): ChatMessage => ({ key: `m${i}`, channel: "party", author: "Riven", text: `line ${i} of the raid plan` });
  const st = { draft: "", sendTo: "", messages: Array.from({ length: 30 }, (_, i) => say(i)) };
  const engine = chatRun(st, results, 300);
  engine.advance(30);
  const shows = (k: string): boolean => {
    const e = byPath(engine, `Log/${k}`);
    const log = byPath(engine, "Chat/Log")!;
    return !!e && e.y + e.height <= log.y + log.height + 2 && e.y >= log.y - 2;
  };
  expect(shows("m29")).toBeTrue();
  st.messages = [...st.messages, say(30)];
  engine.advance(30);
  expect(shows("m30")).toBeTrue();
  const log = byPath(engine, "Chat/Log")!;
  engine.probe.commands(`0:touch_move:${Math.round(log.x + log.width / 2)},${Math.round(log.y + log.height / 2)};0.05:scroll:0,-400`);
  engine.advance(30);
  expect(shows("m30")).toBeFalse();
  st.messages = [...st.messages, say(31), say(32)];
  engine.advance(30);
  expect(shows("m32")).toBeFalse();
  expect(elements(engine).find((e) => e.path.endsWith("Chat/New/Text"))?.text).toBe("2 new");
  tapOn(engine, byPath(engine, "Chat/New")!);
  engine.advance(40);
  expect(shows("m32")).toBeTrue();
  expect(byPath(engine, "Chat/New") === undefined).toBeTrue();
});
