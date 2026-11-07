// og.story on the native host over tests/fixtures/story (a smith's
// dialogue, a quest, en and ja string tables), cooked by the og this
// package built: lines and choices resolve through og.i18n and re-resolve
// on a locale switch, the story's events arrive the tick after, a
// conversation saved mid-line restores to the same line and hash, an edit
// that removes a saved id refuses to replay commands, and a world that
// never loads a story keeps its hash.
// Batch 2: barks read the world (a game component) and keep priority and
// cooldowns, ambient barkers draw on the story's stream identically per
// seed, a voiced line plays through og.audio on its own entity and the
// view counts down to its end, quest objectives count named og events with
// no game code, and dump_story writes all of it.
import { expect, test } from "bun:test";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { og, defineComponent, f32, type Entity } from "@origingame/engine";
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
  must(og.story.load(e.world, "story/smith.yarn"));
  must(og.story.load(e.world, "story/rescue.ogt"));
}

test("Yarn base text survives table loading, locale switching and a live text edit", () => {
  const e = start();
  const w = e.world;
  const source = `// Authored base text without a separate English entry.
#og-profile:yarn-spinner-3.2.2
#og-name:authored
title: Start
---
Hello from Yarn #line:authored.hello
-> Continue #line:authored.continue
    <<stop>>
===`;
  must(og.story.parse(w, source));
  const c = must<number>(og.story.start(w, { dialogue: "authored" }));
  expect(og.story.view(w, c)?.text).toBe("Hello from Yarn");
  must(og.i18n.load(w, "strings/en.ogt"));
  expect(og.story.view(w, c)?.text).toBe("Hello from Yarn");
  og.i18n.setLocale(w, "ja");
  expect(og.story.view(w, c)?.text).toBe("こんにちは");
  expect(og.story.view(w, c)?.choices[0]?.text).toBe("続ける");
  must(og.story.parse(w, source.replace("Hello from Yarn", "Edited Yarn text")));
  og.i18n.setLocale(w, "en");
  expect(og.story.view(w, c)?.text).toBe("Edited Yarn text");
  e.destroy();
});

test("Yarn saliency is selectable and views expose scalar typed spans", () => {
  const e = start();
  must(og.story.parse(e.world, `#og-profile:yarn-spinner-3.2.2
#og-name:saliency
title: Start
when: always
subtitle: fallback
---
Fallback #line:saliency.fallback
===
title: Start
when: true and true
subtitle: specific
---
Mae: 😀 [wave speed=2 active=true]hello[/wave] #line:saliency.specific
===`));
  must(og.story.setSaliency(e.world, "first"));
  const first = must<number>(og.story.start(e.world, { dialogue: "saliency" }));
  expect(og.story.view(e.world, first)?.key).toBe("saliency.fallback");
  must(og.story.setSaliency(e.world, "best"));
  const best = must<number>(og.story.start(e.world, { dialogue: "saliency" }));
  const view = og.story.view(e.world, best);
  expect(view?.text).toBe("Mae: 😀 hello");
  expect(view?.spans).toEqual([
    { name: "character", position: 0, length: 5, properties: { name: "Mae" } },
    { name: "wave", position: 7, length: 5, properties: { speed: 2, active: true } },
  ]);
  e.destroy();
});

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

  expect<unknown>(og.story.choose(w, c, 0)).toEqual({ ok: true });
  expect(og.story.get(w, "gold")).toBe(0);
  v = og.story.view(w, c) as unknown as View;
  expect([v.key, v.seq, v.choices.length]).toEqual(["smith.thanks", 2, 0]);
  must(og.story.advance(w, c));
  v = og.story.view(w, c) as unknown as View;
  expect(v.ended).toBeTrue();
  e.advance(1);
  expect(w.events(og.StoryEvent).map((x) => [x.name, x.arg])).toEqual([["give_sword", "1"]]);
  expect(w.events(og.DialogueEnded).map((x) => x.conversation)).toEqual([c]);
  const back = og.story.history(w, c);
  expect(back.map((l) => [l.key, l.chosen])).toEqual([
    ["smith.hello", { index: 0, text: "Buy the sword (10 gold)", spans: [] }],
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

test("og.story: a conversation saved mid-line restores to the same line and hash; a removed id refuses to replay", () => {
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

  // Removing an ID must not restart the conversation and replay its commands.
  const edited = smithText().replace("#line:smith.thanks", "#line:smith.paid");
  must(og.story.parse(f.world, edited));
  expect(og.story.advance(f.world, c)).toMatchObject({ ok: false, reason: expect.stringContaining("stable id") });
  expect(og.story.get(f.world, "gold")).toBe(0);
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

const GuardHealth = defineComponent("GuardHealth", { hp: f32() });

type Bark = { entity: Entity; bark: string; seq: number; key: string; text: string; ticksLeft: number; priority: number };
const barks = (w: Engine["world"]) => (og.story.barks(w) as unknown as Bark[]).map((b) => [b.entity, b.key, b.text]);

function guards(e: Engine): [Entity, Entity] {
  const w = e.world;
  must(og.story.load(w, "story/guard.yarn"));
  must(og.story.load(w, "story/watch.ogt"));
  const g = (name: string, hp: number): Entity => {
    const x = must<Entity>(w.spawn({ name }));
    must(w.insert(x, GuardHealth, { hp }));
    return x;
  };
  return [g("guard-a", 100), g("guard-b", 10)];
}

test("og.story barks: a world read picks the higher priority, cooldowns hold, and an objective counts the bark's event", () => {
  const e = start();
  const w = e.world;
  const [a, b] = guards(e);
  expect(must<boolean>(og.story.startQuest(w, "watch"))).toBeTrue();
  must(og.story.bark(w, { entity: a, topic: "idle" }));
  must(og.story.bark(w, { entity: b, topic: "idle" }));
  e.advance(1);
  // a is healthy: the idle yawn. b's GuardHealth.hp 10 < 30: the hurt line, at priority 5, with the hp it read.
  expect(barks(w)).toEqual([
    [a, "guard.yawn", "*yawns*"],
    [b, "guard.hurt", "Only 10 left in me..."],
  ]);
  // Sent the tick after the ask, as the line settles.
  expect(w.events(og.BarkSaid).map((x) => [x.entity, x.bark, x.key])).toEqual([
    [a, "idle", "guard.yawn"],
    [b, "hurt", "guard.hurt"],
  ]);
  e.advance(1);
  // og.BarkSaid where event.bark == 'hurt': counted with no game code.
  type Q = { objectives: { id: string; count: number; done: boolean }[] };
  expect((og.story.quest(w, "watch") as unknown as Q).objectives.map((o) => [o.id, o.count, o.done])).toEqual([
    ["halts", 0, false],
    ["hurts", 1, true],
  ]);

  // Asked again within the cooldowns: a's yawn (120 ticks) says nothing new; past b's 30-tick duration and 60-tick cooldown it may.
  const seqs = () => (og.story.barks(w) as unknown as Bark[]).map((x) => x.seq);
  const before = seqs();
  must(og.story.bark(w, { entity: a, topic: "idle" }));
  e.advance(1);
  expect((og.story.barks(w) as unknown as Bark[]).find((x) => x.entity === a)!.seq).toBe(before[0]);
  e.advance(70);
  must(og.story.bark(w, { entity: b, topic: "idle" }));
  e.advance(1);
  const hurt = (og.story.barks(w) as unknown as Bark[]).find((x) => x.entity === b)!;
  expect([hurt.key, hurt.seq > before[1]!]).toEqual(["guard.hurt", true]);

  // A topic no dialogue has is refused, naming it.
  const r = og.story.bark(w, { entity: a, topic: "sing" }) as unknown as { ok: boolean; reason: string };
  expect(r.ok).toBeFalse();
  expect(r.reason).toContain("sing");
  e.destroy();
});

test("og.story ambient barks draw on the story's stream: the same seed says the same lines at the same ticks", () => {
  const run = (): string[] => {
    const e = start();
    const w = e.world;
    const [a, b] = guards(e);
    must(og.story.ambient(w, { entity: a, topic: "idle", every: 40, jitter: 30 }));
    must(og.story.ambient(w, { entity: b, topic: "idle", every: 40, jitter: 30 }));
    const said: string[] = [];
    for (let t = 0; t < 400; t++) {
      e.advance(1);
      for (const x of w.events(og.BarkSaid)) said.push(`${t}:${x.entity}:${x.key}:${x.seq}`);
    }
    const hash = e.hash();
    e.destroy();
    return [...said, hash];
  };
  const one = run();
  expect(one.length).toBeGreaterThan(6);
  expect(run()).toEqual(one);
});

test("og.story voice: a voiced line plays on its own og.AudioSource entity, the view counts to its end, and dump_story shows it", () => {
  const e = start();
  const w = e.world;
  guards(e);
  must(og.story.startQuest(w, "watch"));
  const c = must<number>(og.story.start(w, { dialogue: "guard" }));
  type V = View & { voice: string; voiceTicks: number; voiceLeft: number };
  let v = og.story.view(w, c) as unknown as V;
  expect([v.key, v.voice]).toEqual(["guard.halt", "vo/guard_halt.wav"]);
  const total = v.voiceTicks;
  expect(total).toBeGreaterThan(1);
  e.advance(1);
  v = og.story.view(w, c) as unknown as V;
  expect(v.voiceLeft).toBeGreaterThan(0);
  expect(v.voiceLeft).toBeLessThanOrEqual(total);

  e.probe.commands("0:dump_story:story.json");
  e.advance(1);
  type Dump = { loaded: boolean; voices: { for: string; seq: number; entity: number }[]; quests: { id: string; countsEvents: unknown }[]; conversations: { line: { voiceLeft: number } }[]; rng: { seed: number } };
  const dump = JSON.parse(e.takeOutputs().find((o) => o.file === "story.json")!.data) as Dump;
  expect(dump.loaded).toBeTrue();
  expect(dump.voices.map((x) => [x.for, x.seq])).toEqual([[`c${c}`, v.seq]]);
  const voice = dump.voices[0]!.entity;
  expect(w.has(voice as never, og.AudioSource)).toBeTrue();
  expect(dump.quests.find((q) => q.id === "watch")!.countsEvents).toEqual([
    { objective: "halts", on: "og.StoryEvent" },
    { objective: "hurts", on: "og.BarkSaid" },
  ]);

  // Played out: the view says so and the voice's entity is gone.
  e.advance(total + 1);
  v = og.story.view(w, c) as unknown as V;
  expect(v.voiceLeft).toBe(0);
  expect(w.alive(voice as never)).toBeFalse();

  // Advancing emits `halted`; og.StoryEvent where event.name == 'halted' counts it, twice over two visits.
  must(og.story.advance(w, c));
  e.advance(2);
  const c2 = must<number>(og.story.start(w, { dialogue: "guard" }));
  must(og.story.advance(w, c2));
  e.advance(2);
  type Q = { objectives: { id: string; count: number; done: boolean }[] };
  expect((og.story.quest(w, "watch") as unknown as Q).objectives[0]).toMatchObject({ id: "halts", count: 2, done: true });
  e.destroy();
});

function smithText(): string {
  return readFileSync(join(fixture, "assets", "story", "smith.yarn"), "utf8");
}
