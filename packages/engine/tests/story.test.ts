// og.story on the native host over tests/fixtures/story (a smith's
// dialogue, a quest, en and ja string tables), cooked by the og this
// package built: lines and choices resolve through og.i18n and re-resolve
// on a locale switch, the story's events arrive the tick after, a
// conversation saved mid-line restores to the same line and hash, an edit
// that renames the node a save stands on warns naming it and falls back
// to the start, and a world that never loads a story keeps its hash.
import { expect, test } from "bun:test";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { og } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: { spawnSync(cmd: string[]): { exitCode: number; stderr: Uint8Array } };

const fixture = join(dirname(fileURLToPath(import.meta.url)), "fixtures", "story");

{
  loadNativeHost();
  const cooked = Bun.spawnSync([ogBin(), "cook", fixture]);
  if (cooked.exitCode !== 0) throw new Error(new TextDecoder().decode(cooked.stderr));
  if (!existsSync(join(fixture, "cooked"))) throw new Error(`og cook ${fixture} wrote no cooked/`);
}

type R<T> = { ok: true; value: T } | { ok: false; reason: string };
const must = <T>(r: unknown): T => {
  const x = r as R<T>;
  if (!x.ok) throw new Error(x.reason);
  return x.value;
};

type Choice = { index: number; id: string; text: string; enabled: boolean; reason: string; reasonKey: string; reasonArgs: Record<string, unknown>; visited: boolean; kind: string; check: { stat: string; target: number; chance: number; kind: string } | null };
type View = {
  conversation: number; node: string; seq: number; seen: boolean; key: string; text: string;
  speaker: { id: string; name: string; portrait: string; side: string; entity: number };
  choices: Choice[]; timer: { ticksLeft: number; ticksTotal: number; defaultChoice: number } | null;
  ended: boolean;
};

function start(locale = "en"): Engine {
  const r = createEngine(og.game({ seed: 42 }) as GameToken, { root: fixture, locale });
  if (!r.ok) throw new Error(r.reason);
  const w = r.value.world;
  for (const l of ["en", "ja"]) must(og.i18n.load(w, `strings/${l}.ogt`));
  return r.value;
}

function load(e: Engine): void {
  must(og.story.load(e.world, "story/smith.ogt"));
  must(og.story.load(e.world, "story/rescue.ogt"));
}

test("og.story: lines and choices resolve through og.i18n, re-resolve on a locale switch, and send their events the tick after", () => {
  const e = start();
  const w = e.world;
  load(e);
  expect(must<boolean>(og.story.startQuest(w, "rescue"))).toBeTrue();
  const smith = w.spawn({ name: "smith" });
  const c = must<number>(og.story.start(w, { dialogue: "smith", cast: { smith: must<number>(smith) } }));
  let v = og.story.view(w, c) as unknown as View;
  expect([v.node, v.key, v.text, v.seq, v.seen]).toEqual(["greet", "smith.hello", "Steel's not cheap. You've 10 gold.", 1, false]);
  expect([v.speaker.id, v.speaker.name, v.speaker.portrait, v.speaker.side, v.speaker.entity]).toEqual(["smith", "Brannoc", "art/smith.png", "right", must<number>(smith)]);
  // The secret (charm 3 < 5, hidden) is left out; its index still counts.
  expect(v.choices.map((ch) => [ch.index, ch.text, ch.enabled])).toEqual([
    [0, "Buy the sword (10 gold)", true],
    [1, "Haggle", true],
    [3, "Leave", true],
  ]);
  const haggle = v.choices[1]!.check!;
  expect([haggle.stat, haggle.target, haggle.kind]).toEqual(["charm", 12, "active"]);
  // d20 + 3 >= 12 passes on 9 to 20: 12 of 20 faces.
  expect(haggle.chance).toBeCloseTo(0.6, 6);
  expect(v.timer).toEqual({ ticksLeft: 30, ticksTotal: 30, defaultChoice: 3 });

  // ja mid-line: the same line, re-translated; nothing else moves.
  og.i18n.setLocale(w, "ja");
  const ja = og.story.view(w, c) as unknown as View;
  expect([ja.key, ja.seq, ja.text, ja.speaker.name]).toEqual(["smith.hello", 1, "鋼は安くない。所持金は10ゴールドだな。", "ブラノック"]);
  expect(ja.choices[0]!.text).toBe("剣を買う（10ゴールド）");
  og.i18n.setLocale(w, "en");

  e.advance(1);
  expect(w.events(og.DialogueShown).map((x) => [x.conversation, x.seq])).toEqual([[c, 1]]);
  expect(w.events(og.QuestUpdated).map((x) => [x.quest, x.status])).toEqual([["rescue", "active"]]);
  expect(must<{ lines: number; conversations: number; activeQuests: number }>(w.resource(og.Story))).toEqual({ lines: 1, conversations: 1, activeQuests: 1 });

  expect(og.story.choose(w, c, 0)).toEqual({ ok: true });
  expect(og.story.get(w, "gold")).toBe(0);
  v = og.story.view(w, c) as unknown as View;
  expect([v.key, v.seq, v.choices.length]).toEqual(["smith.thanks", 2, 0]);
  must(og.story.advance(w, c));
  v = og.story.view(w, c) as unknown as View;
  expect(v.ended).toBeTrue();
  e.advance(1);
  expect(w.events(og.StoryEvent).map((x) => [x.name, x.arg])).toEqual([["give_sword", "1"]]);
  expect(w.events(og.DialogueEnded).map((x) => x.conversation)).toEqual([c]);
  const back = og.story.history(w, c) as unknown as { key: string; chosen: { index: number; text: string } | null }[];
  expect(back.map((l) => [l.key, l.chosen])).toEqual([
    ["smith.hello", { index: 0, text: "Buy the sword (10 gold)" }],
    ["smith.thanks", null],
  ]);

  // A second visit: no gold, so buying is disabled with its reason, resolved from a key.
  const c2 = must<number>(og.story.start(w, { dialogue: "smith" }));
  v = og.story.view(w, c2) as unknown as View;
  expect(v.seen).toBeTrue();
  const buy = v.choices[0]!;
  expect([buy.enabled, buy.visited, buy.reason, buy.reasonKey, buy.reasonArgs]).toEqual([false, false, "You need 10 more gold.", "smith.need_gold", { need: 10 }]);
  expect(og.story.choose(w, c2, 0)).toMatchObject({ ok: false });

  type Q = { title: string; stage: string; status: string; tracked: boolean; isNew: boolean; marker: unknown; rewards: unknown[]; objectives: { id: string; text: string; count: number; done: boolean }[] };
  const q = og.story.quest(w, "rescue") as unknown as Q;
  expect([q.title, q.stage, q.status, q.tracked, q.isNew]).toEqual(["The Lost Sword", "hunt", "active", true, true]);
  expect(q.objectives.map((o) => [o.id, o.text, o.count, o.done])).toEqual([
    ["wolves", "Drive off the wolves (0/3)", 0, false],
    ["sword", "Arm yourself", 1, true],
  ]);
  expect(q.marker).toEqual({ entity: 0, position: { x: 12, y: 0, z: 4 } });
  expect(q.rewards).toEqual([{ name: "Gold", amount: 50, image: "", rarity: "common" }]);
  expect(og.story.markSeen(w, "rescue")).toBeTrue();
  expect((og.story.quest(w, "rescue") as unknown as Q).isNew).toBeFalse();
  e.destroy();
});

test("og.story: a conversation saved mid-line restores to the same line and hash; a renamed node warns, names it and falls back", () => {
  const e = start();
  const w = e.world;
  load(e);
  const c = must<number>(og.story.start(w, { dialogue: "smith" }));
  must(og.story.choose(w, c, 0));
  e.advance(2);
  const save = must<string>(w.snapshot());
  const hash = e.hash();
  must(og.story.advance(w, c));
  e.advance(1);
  expect(e.hash()).not.toBe(hash);
  expect(must(w.restore(save))).toBeNull();
  expect(e.hash()).toBe(hash);
  expect((og.story.view(w, c) as unknown as View).key).toBe("smith.thanks");

  // A fresh session restores the save: the story's records join first.
  const f = start();
  load(f);
  expect(must(f.world.restore(save))).toBeNull();
  expect(f.hash()).toBe(hash);
  expect((og.story.view(f.world, c) as unknown as View).key).toBe("smith.thanks");

  // The node it stands on, renamed: one warning naming it, back to the start.
  const edited = smithText().replace("node bought {", "node paid {").replaceAll("goto bought", "goto paid");
  must(og.story.parse(f.world, edited));
  f.takeLog();
  const v = og.story.view(f.world, c) as unknown as View;
  expect([v.node, v.key]).toEqual(["greet", "smith.hello"]);
  const warned = f.takeLog().filter((l) => l.text.includes("og.story"));
  expect(warned.length).toBe(1);
  expect(warned[0]!.text).toContain("'bought' is gone from dialogue 'smith' (renamed or removed); it restarts at 'greet'");
  e.destroy();
  f.destroy();
});

test("og.story: a world that never loads a story keeps its hash; the first load adds og.Story", () => {
  const a = start();
  const b = start();
  a.advance(5);
  b.advance(5);
  expect(a.hash()).toBe(b.hash());
  // Starting before any load names the fix.
  const early = og.story.start(a.world, { dialogue: "smith" }) as unknown as { ok: boolean; reason: string };
  expect(early.ok).toBeFalse();
  expect(early.reason).toContain("og.story.load");
  load(b);
  expect(b.world.resource(og.Story).ok).toBeTrue();
  expect(b.hash()).not.toBe(a.hash());
  a.destroy();
  b.destroy();
});

function smithText(): string {
  // The cooked fixture's own text (tests/fixtures/story/assets/story/smith.ogt).
  return readFileSync(join(fixture, "assets", "story", "smith.ogt"), "utf8");
}
