// The story: Castellan Maren's conversation (assets/story/maren.ogt) and
// the yard's quest (assets/story/seal.ogt), both og.story data. The game
// never walks the graph: Talk by Maren starts the conversation, a seat's
// Talk or Attack moves a line on, its choice1 to choice3 (the number keys,
// a tap on a reply: src/ui.ts presses them) pick a reply, and og.story
// answers what is said next. The quest counts the risen put down
// (src/foes.ts) and the seal taken from the chest (here), and completes
// when Maren is handed the seal; the run is won when that conversation
// closes.
//
// Everything that changes the story happens in a system on a seat's input,
// so a room changes it on one tick on every peer.

import { og, defineSystem } from "@origingame/engine";
import type { Value, Vector3, World } from "@origingame/engine";
import { Ending, Game, Phase, Quest, Sound } from "./components.ts";
import { held } from "./seats.ts";
import { must } from "./world.ts";

type GameV = Value<typeof Game>;

/** The quest's id (story/seal.ogt). */
export const QUEST = "seal";
/** Ticks a line shows before a press answers it: a press meant for the last one never skips the next. */
const BEAT = 12;
/** The replies a press can pick: choice1 to choice3. */
export const CHOICES = 3;

/** Loads the story's data (in start: og.story's world reads are granted at load). */
export function loadStory(world: World): void {
  must(og.story.load(world, "story/seal.ogt"));
  must(og.story.load(world, "story/maren.ogt"));
}

/** A new run or the title: a new playthrough of the story, nothing said, no quest. */
export function resetStory(world: World, g: GameV): void {
  og.story.reset(world);
  g.conversation = 0;
  g.talker = -1;
  g.lineSeq = 0;
  g.lineAt = 0;
  g.quest = Quest.None;
  g.toast = Quest.None;
  g.toastUp = 0;
  g.seal = false;
  g.chestOpen = false;
}

/** Seat `seat`'s hero talks to Maren. */
export function startTalk(world: World, g: GameV, seat: number): void {
  const conv = og.story.start(world, { dialogue: "maren", cast: { maren: g.maren } });
  if (!conv.ok) return;
  g.conversation = conv.value;
  g.talker = seat;
  g.lineSeq = 0;
  g.lineAt = world.tick;
}

/** A seat's presses while the conversation runs: a reply by its number, else Talk or Attack moves a line on. */
export function answer(world: World, g: GameV, _seat: number, pressed: number): void {
  const v = og.story.view(world, g.conversation);
  if (v === null || v.ended) return;
  if (v.seq !== g.lineSeq) {
    g.lineSeq = v.seq;
    g.lineAt = world.tick;
  }
  if (world.tick - g.lineAt < BEAT) return;
  if (v.choices.length > 0) {
    for (let n = 1; n <= CHOICES; n++) {
      const c = v.choices[n - 1];
      if (c === undefined || !c.enabled || !held(pressed, `choice${n}`)) continue;
      must(og.story.choose(world, g.conversation, c.index));
      g.lineAt = world.tick;
      g.sfx |= Sound.Tap;
      return;
    }
    return;
  }
  if (held(pressed, "talk") || held(pressed, "attack")) {
    must(og.story.advance(world, g.conversation));
    g.lineAt = world.tick;
  }
}

/** Seat `seat`'s hero opens the chest: the seal once the yard is clear, else the lock holds. */
export function openChest(world: World, g: GameV, _seat: number, p: Vector3): void {
  if (g.quest !== Quest.Fetch) {
    must(og.render.floatingText(world, { text: og.i18n.t(world, g.quest === Quest.None ? "float.chest.none" : "float.chest.guarded"), position: { x: p.x, y: p.y + 1.4, z: p.z }, size: 0.3, color: { r: 0.92, g: 0.86, b: 0.72 }, outline: 0.12, rise: 0.5, lifetime: 1.4 }));
    g.sfx |= Sound.Clang;
    return;
  }
  g.chestOpen = true;
  g.seal = true;
  g.gold += 25;
  og.story.progress(world, QUEST, "seal");
  g.sfx |= Sound.Chest | Sound.Coin;
  const chest = world.find("chest");
  const at = chest === null ? p : must(world.get(chest, og.Transform)).position;
  must(og.render.floatingText(world, { text: og.i18n.t(world, "float.seal"), position: { x: at.x, y: at.y + 1.3, z: at.z }, size: 0.42, color: { r: 1, g: 0.84, b: 0.4 }, outline: 0.14, rise: 0.9, lifetime: 1.8, pop: 0.4 }));
  const glow = world.find("chest-glow");
  if (glow !== null) must(world.set(glow, og.ParticleSystem, { visible: true, startTick: world.tick }));
}

/** The quest's stage as the yard plays it. */
function stageOf(world: World): number {
  const q = og.story.quest(world, QUEST);
  if (q === null) return Quest.None;
  if (q.status === "completed") return Quest.Done;
  return q.stage === "fetch" ? Quest.Fetch : q.stage === "report" ? Quest.Return : Quest.Clear;
}

/**
 * Per tick: the conversation's end, the quest's stage with its toast and
 * chime, and the win once the seal is handed over and Maren has said her
 * piece. Read from og.story's own state each tick (og.story.view,
 * og.story.quest), which a restore or a replay carries too.
 */
export const Story = defineSystem(
  "Story",
  { writes: [Game], after: ["Heroes"] },
  (_q, _time, world) => {
    const g = must(world.resource(Game));
    if (g.start) return;
    if (g.conversation !== 0) {
      const v = og.story.view(world, g.conversation);
      if (v === null || v.ended) {
        g.conversation = 0;
        g.talker = -1;
      }
    }
    const stage = stageOf(world);
    if (stage !== g.quest) {
      g.quest = stage;
      g.sfx |= Sound.Quest;
      if (g.phase === Phase.Play) g.toast = stage;
    }
    // The stage's toast once nothing is said over it (the box and the toast share the bottom of the screen).
    if (g.toast !== Quest.None && g.conversation === 0) {
      const key = ["", "toast.quest.start", "toast.quest.fetch", "toast.quest.return", "toast.quest.done"][g.toast]!;
      g.toastUp = og.ui.toast(world, og.i18n.t(world, key), { tone: g.toast === Quest.Done ? "success" : "info" }) + 1;
      g.toast = Quest.None;
    }
    if (g.toastUp !== 0 && g.conversation !== 0) {
      og.ui.dismissToast(world, g.toastUp - 1);
      g.toastUp = 0;
    }
    if (g.phase === Phase.Play && g.ending === Ending.None && g.quest === Quest.Done && g.conversation === 0) {
      g.ending = Ending.Won;
      g.sfx |= Sound.Win;
    }
  },
);
