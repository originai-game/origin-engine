// The story patterns' preview screens (docs/ui.md §11, U6d): a conversation
// (cinematic), a visual novel's text box and its backlog, and the quest log
// with the HUD's tracker, each fed by og.story from the kit's own dialogue
// and quests (parsed below, their words in assets/strings), with the game's
// side done as a game does it: og.story.advance and og.story.choose on the
// conversation's answer, og.story.track and og.story.markSeen on the log's.
// `pat:dlg:<key>:<value>` and `pat:quest:<key>:<value>` set a preview's
// state so the playtests (tests/patterns*.og.ts) can draw each.

import { og } from "@origingame/engine";
import type { DialogueView, QuestView, Size, World } from "@origingame/engine";
import { art, besideChrome, chromeRoom, L, titleLine } from "./patterns.ts";
import { rpg } from "@origingame/engine/extensions/rpg/index.ts";
import { story } from "@origingame/engine/extensions/story/index.ts";

const grow: Size = { kind: "grow" };
const px = (n: number): Size => ({ kind: "fixed", px: n });

function phone(): boolean {
  const b = og.ui.frameViewport().breakpoint;
  return b === "phonePortrait" || b === "phoneLandscape";
}

// ── the kit's story ─────────────────────────────────────────────────────

/** At the gate: Mira's question, a check with odds, a passed insight, a toll the purse cannot pay, a timer. */
const GATE = `type dialogue
version 1
---
name kit_gate
start gate
speaker mira {
  name "dlg.mira"
  portrait "textures/art/figures/mira.png"
  side left
}
global charm 3
global insight 4
global credits 40
node gate {
  say mira "dlg.gate.open"
  choice "dlg.gate.ready" {
    goto ready
  }
  choice "dlg.gate.persuade" {
    check active charm 12
    goto secret
    fail refuse
  }
  choice "dlg.gate.read" {
    check passive insight 4
    goto who
  }
  choice "dlg.gate.who" {
    goto who
  }
  choice "dlg.gate.toll" {
    if "credits >= 100"
    reason "dlg.gate.need" {
      arg need "100 - credits"
    }
    kind trade
    goto ready
  }
  choice "dlg.gate.notyet"
  timer 600 dlg.gate.notyet
}
node who {
  say mira "dlg.who"
  goto gate
}
node secret {
  say mira "dlg.secret"
}
node refuse {
  say mira "dlg.refuse"
}
node ready {
  say mira "dlg.ready"
}
`;

/** Lowmarsh: the narrator, then Aria, then a choice: a visual novel's scene. */
const MARSH = `type dialogue
version 1
---
name kit_marsh
start arrive
speaker aria {
  name "dlg.aria"
  portrait "textures/art/figures/aria.png"
  side right
}
node arrive {
  say narrator "vn.n1"
  say aria "vn.a1"
  say aria "vn.a2"
  choice "vn.stay" {
    goto stay
  }
  choice "vn.wander" {
    goto wander
  }
}
node stay {
  say aria "vn.stay.r"
}
node wander {
  say aria "vn.wander.r"
}
`;

/** The quests: the main one with every kind of objective and rewards, a side one, a daily, one completed and one failed. */
const QUESTS = [
  `type quest
version 1
---
id kit_dawn
title "quest.dawn"
summary "quest.dawn.summary"
category main
giver "dlg.mira"
region "region.skyreach"
reward "item.crystal" 1 {
  image "textures/art/boons/crystal.jpg"
  rarity epic
}
reward "item.ring" 1 {
  image "textures/art/items/ember-ring.jpg"
  rarity rare
}
reward "item.gold" 250 {
  image "textures/art/boons/gold.jpg"
  rarity common
}
stage climb {
  objective reach "quest.dawn.reach" {
    marker 12 0 4
  }
  objective charts "quest.dawn.charts" {
    target 3
  }
  objective archivist "quest.dawn.archivist" {
    optional
  }
  objective sealer "quest.dawn.sealer" {
    hidden
  }
}
`,
  `type quest
version 1
---
id kit_mill
title "quest.mill"
summary "quest.mill.summary"
category side
giver "npc.miller"
region "region.mill"
reward "item.potion" 2 {
  image "textures/art/items/healing-draught.jpg"
  rarity uncommon
}
stage hunt {
  objective wolves "quest.mill.wolves" {
    target 5
  }
}
`,
  `type quest
version 1
---
id kit_supply
title "quest.supply"
summary "quest.supply.summary"
category daily
region "region.marsh"
reward "item.gold" 60 {
  image "textures/art/boons/gold.jpg"
  rarity common
}
stage gather {
  objective petals "quest.supply.petals" {
    target 10
  }
}
`,
  `type quest
version 1
---
id kit_letters
title "quest.letters"
summary "quest.letters.summary"
category side
giver "npc.miller"
region "region.marsh"
stage deliver {
  objective letters "quest.letters.deliver"
}
`,
  `type quest
version 1
---
id kit_tide
title "quest.tide"
summary "quest.tide.summary"
category side
region "region.marsh"
stage race {
  objective ferry "quest.tide.ferry"
}
`,
];

/** The kit's story, parsed in the game's setup (the first og.story load registers og.Story, which a system cannot). */
export function loadStory(world: World): void {
  for (const text of [GATE, MARSH, ...QUESTS]) {
    const r = og.story.parse(world, text);
    if (!r.ok) throw new Error(`kit story: ${r.reason}`);
  }
  // Number keys take a choice by its number (the conversation asks for choice1…choice9).
  for (let n = 1; n <= 9; n++) og.ui.defineAction(world, { name: `choice${n}`, keys: [`Digit${n}`, `Numpad${n}`], buttons: [], trigger: "press", holdMs: 0, toggle: false, label: `${n}` });
}

// ── the conversation and the visual novel ───────────────────────────────

const dl = { conv: -1, dialogue: "", auto: false, voice: false, log: false, charm: 3 };

function start(world: World, dialogue: string): void {
  if (dl.conv >= 0) og.story.stop(world, dl.conv);
  og.story.set(world, "charm", dl.charm);
  const r = og.story.start(world, { dialogue });
  dl.conv = r.ok ? (r.value as number) : -1;
  dl.dialogue = dialogue;
}

/** The view, with the preview's voice standing in for a clip (`pat:dlg:voice:on`: the cue drawn as if one played). */
function view(world: World, dialogue: string): DialogueView | null {
  if (dl.dialogue !== dialogue || dl.conv < 0) start(world, dialogue);
  const v = og.story.view(world, dl.conv);
  if (!v) return null;
  return dl.voice && !v.ended ? { ...v, voiceTicks: 240, voiceLeft: 180 } : v;
}

function words(world: World) {
  void world;
  return {
    skipLabel: L("dlg.skip"),
    logLabel: L("dlg.log"),
    autoLabel: L("dlg.auto"),
    continueLabel: L("dlg.continue"),
    unavailableLabel: L("dlg.unavailable"),
    checkLabel: L("dlg.check"),
    passedLabel: L("dlg.passed"),
    failedLabel: L("dlg.failed"),
    statNames: { charm: L("dlg.stat.charm"), insight: L("dlg.stat.insight") },
  };
}

/** A conversation's screen: the pattern over the scene, the game's side of its answer, and the backlog over it when asked. */
function talk(world: World, id: string, dialogue: string, layout: "cinematic" | "box"): void {
  const vp = og.ui.frameViewport();
  const sa = vp.safeArea;
  const small = phone();
  // On a phone the kit's Gallery button takes the top end corner: the conversation's tools stand under it.
  // Under its bottom edge as laid (its margin included), measured from the safe area the anchor stands in.
  const back = og.ui.bounds("Back");
  const top = small ? (back ? back.y + back.height - sa.top : 60) + 8 : 0;
  // On a desktop the Gallery button takes the bottom start corner: the text box (or the cinematic caption, as wide as
  // the screen's middle) stands above it.
  const foot = !small ? (back?.height ?? 40) + 12 : 0;
  const v = view(world, dialogue);
  og.ui.anchor(id, { anchor: "topLeft", margin: 0, offsetY: top, width: px(vp.width - sa.left - sa.right), height: px(vp.height - sa.top - sa.bottom - top - foot), direction: "column" }, () => {
    // Each speaker stands their own figure: the texture loaded for the line's portrait path.
    const figure = v?.speaker.portrait.replace(/^textures\/art\//, "").replace(/\.png$/, "");
    const r = rpg.conversation("Talk", { view: v, portrait: art(figure as Parameters<typeof art>[0]), layout, auto: dl.auto, ...words(world) });
    if (dl.conv >= 0 && !dl.log) {
      if (r.advanced) og.story.advance(world, dl.conv);
      if (r.chosen !== null) og.story.choose(world, dl.conv, r.chosen);
      if (r.log) dl.log = true;
      if (r.autoToggled) dl.auto = !dl.auto;
      if (r.skipped) start(world, dialogue);
    }
  });
  if (dl.log && dl.conv >= 0) {
    og.ui.anchor(`${id}Log`, { anchor: "topLeft", margin: 0, offsetX: -sa.left, offsetY: -sa.top, width: px(vp.width), height: px(vp.height), direction: "column", padding: { left: sa.left, top: sa.top + top, right: sa.right, bottom: sa.bottom } }, () => {
      const b = story.backlog("Backlog", { lines: og.story.history(world, dl.conv), title: L("log.title"), closeLabel: L("log.close"), replayLabel: L("log.replay"), emptyLabel: L("log.empty") });
      if (b.closed) dl.log = false;
    });
  }
  void grow;
}

/** The conversation at the gate (Baldur's Gate 3, Mass Effect): cinematic. */
export function dialogueScreen(world: World): void {
  talk(world, "Dialogue", "kit_gate", "cinematic");
}

/** The visual novel's scene: the text box, its name plate, and the backlog. */
export function vnScreen(world: World): void {
  talk(world, "Novel", "kit_marsh", "box");
}

// ── quests ──────────────────────────────────────────────────────────────

const qs = { selected: "", tab: "main", hud: false, ready: false };

/** The quests started and moved to their states, once (a quest log's fixture, made by og.story as a game would). */
function ensureQuests(world: World): void {
  if (qs.ready) return;
  qs.ready = true;
  for (const id of ["kit_dawn", "kit_mill", "kit_supply", "kit_letters", "kit_tide"]) og.story.startQuest(world, id);
  og.story.progress(world, "kit_dawn", "reach", 1);
  og.story.progress(world, "kit_dawn", "charts", 2);
  og.story.progress(world, "kit_mill", "wolves", 3);
  og.story.progress(world, "kit_supply", "petals", 4);
  og.story.completeQuest(world, "kit_letters");
  og.story.failQuest(world, "kit_tide");
  og.story.track(world, "kit_dawn", true);
  // The side quest has been looked at; the daily is new; the main quest changed since.
  og.story.markSeen(world, "kit_mill");
  og.story.markSeen(world, "kit_letters");
  og.story.markSeen(world, "kit_tide");
}

const REWARD_ART = ["boons/crystal", "items/ember-ring", "boons/gold", "items/healing-draught"] as const;

/** The quest log (The Witcher 3, Genshin Impact) or, with `pat:quest:hud:on`, the HUD's tracker over the scene. */
export function questsScreen(world: World): void {
  ensureQuests(world);
  const t = og.ui.frameTheme();
  const vp = og.ui.frameViewport();
  const sa = vp.safeArea;
  const small = phone();
  const quests: readonly QuestView[] = og.story.quests(world);
  if (qs.hud) {
    rpg.questTracker("Tracker", { quests, from: { x: 0, y: 0, z: 0 }, completeLabel: L("q.complete") });
    return;
  }
  og.ui.anchor("Quests", { anchor: "topLeft", margin: 0, width: px(vp.width - sa.left - sa.right), height: px(vp.height - sa.top - sa.bottom), direction: "column", gap: t.space.md, padding: { left: small ? t.space.sm : t.space.xl, top: small ? t.space.sm : t.space.lg, right: small ? t.space.sm : t.space.xl, bottom: small ? t.space.sm : Math.max(t.space.xl, chromeRoom().bottom) } }, () => {
    const chrome = besideChrome(vp.width - sa.left - sa.right - 2 * t.space.sm, "Quests/Header/Title", L("q.title"), "xl2");
    og.ui.column("Header", { width: grow, height: { kind: "fit", min: chrome.min }, align: "center", padding: { left: 0, top: chrome.top, right: chrome.right, bottom: 0 } }, () => {
      og.ui.label("Title", L("q.title"), { step: small ? "xl2" : "xl4", heading: true, font: "display", ...titleLine(chrome) });
    });
    const r = rpg.questLog("Log", {
      quests,
      selected: qs.selected,
      tab: qs.tab,
      tabs: [{ key: "main", label: L("q.tab.main") }, { key: "side", label: L("q.tab.side") }, { key: "daily", label: L("q.tab.daily") }],
      art: REWARD_ART.flatMap((a) => {
        const tex = art(a);
        return tex ? [{ path: `textures/art/${a}.jpg`, texture: tex }] : [];
      }),
      title: L("q.title"),
      trackLabel: L("q.track"),
      trackedLabel: L("q.tracked"),
      mapLabel: L("q.map"),
      objectivesLabel: L("q.objectives"),
      rewardsLabel: L("q.rewards"),
      optionalLabel: L("q.optional"),
      giverLabel: L("q.giver"),
      newLabel: L("q.new"),
      activeLabel: L("q.active"),
      completedLabel: L("q.completed"),
      failedLabel: L("q.failed"),
      emptyLabel: L("q.empty"),
      backLabel: L("q.back"),
    });
    if (r.selected !== null) {
      qs.selected = r.selected;
      og.story.markSeen(world, r.selected);
    }
    if (r.tab !== null) {
      qs.tab = r.tab;
      qs.selected = "";
    }
    if (r.track !== null) {
      const q = quests.find((x) => x.id === r.track);
      og.story.track(world, r.track, !(q?.tracked ?? false));
    }
  });
}

/** The previews' commands: `pat:dlg:<key>:<value>`, `pat:quest:<key>:<value>`. */
export function storyCommand(world: World, area: string, key: string, value: string): boolean {
  if (area === "dlg") {
    if (key === "reset") {
      dl.auto = false;
      dl.voice = false;
      dl.log = false;
      dl.charm = 3;
      if (dl.dialogue) start(world, dl.dialogue);
    } else if (key === "auto") dl.auto = value !== "off";
    else if (key === "voice") dl.voice = value !== "off";
    else if (key === "log") dl.log = value !== "off";
    else if (key === "charm") {
      dl.charm = Number(value) || 0;
      og.story.set(world, "charm", dl.charm);
    } else if (key === "advance" && dl.conv >= 0) og.story.advance(world, dl.conv);
    else if (key === "choose" && dl.conv >= 0) og.story.choose(world, dl.conv, Number(value) || 0);
    else return false;
    return true;
  }
  if (area === "quest") {
    if (key === "select") qs.selected = value;
    else if (key === "tab") {
      qs.tab = value || "main";
      qs.selected = "";
    } else if (key === "hud") qs.hud = value !== "off";
    else if (key === "progress") {
      const [q, o] = value.split(".");
      if (q && o) og.story.progress(world, q, o, 1);
    } else if (key === "complete") og.story.completeQuest(world, value);
    else return false;
    return true;
  }
  return false;
}
