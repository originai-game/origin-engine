// Greywatch Keep's screens on og.ui and the rpg kit, in the style in
// assets/styles/greywatch (the engine's fantasy pack, by way of the
// third-person archetype's ember): the title over the keep's yard and the
// pause menu (the game's screen documents, assets/ui/main-menu.ogt and
// pause.ogt), the HUD (the party's frames top left, the bag's gold and
// draughts and Pause top right, the quest tracker on the right, the last
// struck risen's frame at the top, the Talk prompt by Maren and the chest,
// Maren's conversation as a box at the bottom), the bag (the rpg kit's
// inventory grid and quest log, opened from the pause), the settings
// dialog and the run's summary. Every string is a key of
// assets/strings/*.ogt.
//
// Every choice that changes the world reaches it through a seat's input
// (src/seats.ts press), so a room makes it on one tick on every peer:
// Enter the keep and Again press choice1, Leave the keep and Title press
// Menu (a control with no key), Pause and Resume press Pause, and a reply
// tapped or clicked in the conversation presses its choice1 to choice3 (a
// click or tap on the line, Talk). The touch controls are src/seats.ts's:
// a stick, the look's stick, Attack, Talk and Potion.
//
// The Ui system also holds the world while paused, counts the run's clock,
// shows the summary after a run ends (src/story.ts wins it, src/heroes.ts
// loses it), sets the buses and keeps the settings and the record in
// og.platform (the record written at the revision read, and merged when
// another device wrote it first).

import { og, defineSystem } from "@origingame/engine";
import type { DialogueView, Value, World } from "@origingame/engine";
import { rpg } from "@origingame/engine/extensions/rpg/index.ts";
import type { GridItem, UnitFrameDesc } from "@origingame/engine/extensions/rpg/index.ts";
import { Ending, Foe, Game, Hero, Kind, MAX_LEVEL, Near, Phase, POTIONS, Quest, Sound, Tuning } from "./components.ts";
import { heroOf, lazyTuning, xpFor } from "./heroes.ts";
import { bit, LOCAL, localSeats, press, pressedNow, SEATS, setLookLive, useTouchControls, usesTouch } from "./seats.ts";
import { CHOICES, QUEST, resetStory } from "./story.ts";
import { prefs } from "./device.ts";
import { must } from "./world.ts";

export const LANGUAGES = [
  { code: "en", key: "lang.en" },
  { code: "zh-CN", key: "lang.zh" },
] as const;

/** Ticks between a run's end and its summary: the moment plays out first. */
export const OVER_DELAY = 90;
/** Ticks from a run's end until its summary answers: a third of a second after it shows. */
const SUMMARY_TICKS = OVER_DELAY + 20;

/** The tokens' spacing scale in CSS px. */
const S = { xxs: 2, xs: 4, sm: 8, md: 12, lg: 16 } as const;

type GameV = Value<typeof Game>;
type PrefsV = typeof prefs;

/** A room is running: every peer steps the same world, so no peer's own platform answers (the record) enter it. */
function inRoom(world: World): boolean {
  const lock = world.resource(og.Lockstep);
  return lock.ok && lock.value.running;
}

export function say(world: World, key: string, args?: Record<string, string | number>): string {
  return og.i18n.t(world, key, args);
}

const grow = { kind: "grow" } as const;
const px = (n: number) => ({ kind: "fixed", px: n }) as const;

/** A new run: the party at the gate, the bag as it set out, the story afresh, the risen risen again. */
export function startRun(world: World, g: GameV): void {
  g.phase = Phase.Play;
  g.ending = Ending.None;
  g.paused = false;
  g.bagOpen = false;
  g.time = 0;
  g.slain = 0;
  g.gold = 0;
  g.potions = POTIONS;
  g.hits = 0;
  g.hurts = 0;
  g.drunk = 0;
  g.levelUps = 0;
  g.target = 0;
  g.targetTicks = 0;
  g.overTicks = 0;
  g.newBest = false;
  g.start = true;
  g.runs += 1;
  resetStory(world, g);
  saveRecord(world, g);
}

/** Back to the title: the yard reset behind it. */
export function toTitle(world: World, g: GameV): void {
  g.phase = Phase.Title;
  g.paused = false;
  g.bagOpen = false;
  g.ending = Ending.None;
  g.start = true;
  resetStory(world, g);
}

/**
 * The record ({runs, wins, best}) written at the revision last read or
 * written. One call at a time: while a read or a write is in flight the
 * newer record waits (recordDirty) and goes out when that answers, at the
 * revision the answer gives, so a late answer never stands for a newer
 * local record.
 */
function saveRecord(world: World, g: GameV): void {
  if (inRoom(world)) return;
  if (g.recordRequest !== 0 || g.recordWrite !== 0) {
    g.recordDirty = true;
    return;
  }
  g.recordDirty = false;
  const r = og.platform.storeSet(world, { key: "record", value: { runs: g.runs, wins: g.wins, best: g.best }, revision: g.recordRevision });
  if (!r.ok) return;
  g.recordWrite = r.value;
  g.recordSentRuns = g.runs;
  g.recordSentWins = g.wins;
}

function applyPrefs(world: World, p: PrefsV): void {
  const audio = world.resource(og.Audio);
  if (audio.ok) {
    audio.value.master = p.master;
    audio.value.music = p.music;
    audio.value.sfx = p.effects;
    audio.value.ui = p.effects;
  }
}

export function setLanguage(world: World, p: PrefsV, index: number): void {
  p.language = Math.max(0, Math.min(LANGUAGES.length - 1, index));
  og.i18n.setLocale(world, LANGUAGES[p.language]!.code);
}

function prefsValue(p: PrefsV) {
  return { masterVolume: p.master, musicVolume: p.music, sfxVolume: p.effects, language: LANGUAGES[p.language]!.code, reducedMotion: p.reducedMotion };
}

/**
 * og.platform: the settings and the record asked for on tick 0 and applied
 * when they answer; a record write refused for a stale revision (another
 * device played meanwhile) is read again, merged (the stored counts plus
 * what this device added since it last read or wrote them, the better of
 * the two best times) and written at the new revision.
 */
function platform(world: World, g: GameV, p: PrefsV): void {
  if (world.tick === 0) {
    p.loadRequest = og.platform.loadSettings(world);
    const r = og.platform.storeGet(world, "record");
    g.recordRequest = r.ok ? r.value : 0;
    return;
  }
  if (p.loadRequest !== 0) {
    const r = og.platform.result(world, p.loadRequest);
    if (r !== null) {
      p.loadRequest = 0;
      const s = ((r.ok ? r.data : null) as { value?: Record<string, unknown> } | null)?.value ?? null;
      if (s && typeof s === "object") {
        if (typeof s.masterVolume === "number") p.master = s.masterVolume;
        if (typeof s.musicVolume === "number") p.music = s.musicVolume;
        if (typeof s.sfxVolume === "number") p.effects = s.sfxVolume;
        if (typeof s.reducedMotion === "boolean") p.reducedMotion = s.reducedMotion;
        const at = LANGUAGES.findIndex((l) => l.code === s.language);
        if (at >= 0) setLanguage(world, p, at);
        applyPrefs(world, p);
      }
    }
  }
  // A peer's own answers come back on its own ticks: a running room keeps no record.
  if (inRoom(world)) return;
  if (g.recordRequest !== 0) {
    const r = og.platform.result(world, g.recordRequest);
    if (r !== null) {
      g.recordRequest = 0;
      const d = (r.ok ? r.data : null) as { value?: { runs?: number; wins?: number; best?: number } | null; revision?: number } | null;
      const runs = d?.value?.runs ?? 0;
      const wins = d?.value?.wins ?? 0;
      const best = d?.value?.best ?? 0;
      g.runs = runs + g.runs - g.recordRuns;
      g.wins = wins + g.wins - g.recordWins;
      if (best > 0 && (g.best === 0 || best < g.best)) g.best = best;
      g.recordRuns = runs;
      g.recordWins = wins;
      if (typeof d?.revision === "number") g.recordRevision = d.revision;
      if (g.recordMerge || g.recordDirty) {
        g.recordMerge = false;
        saveRecord(world, g);
      }
    }
  }
  if (g.recordWrite !== 0) {
    const r = og.platform.result(world, g.recordWrite);
    if (r !== null) {
      g.recordWrite = 0;
      if (r.ok) {
        // The store now holds what was sent, at the revision it answers.
        g.recordRevision = (r.data as { revision: number }).revision;
        g.recordRuns = g.recordSentRuns;
        g.recordWins = g.recordSentWins;
        if (g.recordDirty) saveRecord(world, g);
      } else if (r.error?.code === "conflict") {
        // Someone else wrote the record: read it again and write the merge.
        g.recordConflicts += 1;
        g.recordMerge = true;
        const again = og.platform.storeGet(world, "record");
        g.recordRequest = again.ok ? again.value : 0;
      } else if (g.recordDirty) saveRecord(world, g);
    }
  }
}

/** The run's clock, and the summary a beat after it ends (the quest done, or the party fallen). */
function clock(world: World, g: GameV, dt: number): void {
  if (g.phase === Phase.Over) g.overTicks += 1;
  if (g.phase !== Phase.Play) return;
  if (g.ending === Ending.None) {
    g.time += dt;
    return;
  }
  g.overTicks += 1;
  if (g.overTicks === OVER_DELAY) {
    g.phase = Phase.Over;
    g.paused = false;
    g.bagOpen = false;
    if (g.ending === Ending.Won) {
      g.wins += 1;
      g.newBest = g.best === 0 || g.time < g.best;
      if (g.newBest) g.best = g.time;
    }
    saveRecord(world, g);
    must(og.platform.telemetry(world, "run.over", { ending: ["none", "won", "fallen"][g.ending]!, time: Math.round(g.time), slain: g.slain, hurts: g.hurts }));
  }
}

function capsule(id: string, build: () => void): void {
  og.ui.row(id, { gap: S.xs, cross: "center", padding: { left: S.sm, top: S.xs, right: S.md, bottom: S.xs }, frame: "hud", background: { token: "panel" } }, build);
}

/** A hero's frame: its name, level, health and experience (the resource bar). */
function heroFrame(world: World, g: GameV, seat: number): UnitFrameDesc {
  const t = lazyTuning(world);
  const me = must(world.get(heroOf(g, seat), Hero));
  return {
    key: `hero${seat}`,
    name: say(world, seat === 0 ? "hero.knight" : "hero.barbarian"),
    level: `${me.level}`,
    health: me.hp,
    healthMax: me.maxHp,
    resource: me.level >= MAX_LEVEL ? 1 : me.xp / xpFor(t, me.level),
    resourceMax: 1,
    resourceKind: "xp",
    dead: me.dead,
    combat: g.target !== 0,
    leader: seat === 0 && SEATS > 1,
    icon: seat === 0 ? "shield" : "sword",
    size: "medium",
  };
}

/** The HUD: the party, the bag, the quest, the target, the prompts. */
function hud(world: World, g: GameV): void {
  const vp = og.ui.frameViewport();
  const touch = usesTouch();
  const phone = vp.width < 600;
  og.ui.setNavigation(g.conversation !== 0);
  // The party: both heroes' frames, top left (alone, the one).
  const members = Array.from({ length: SEATS }, (_, seat) => heroFrame(world, g, seat));
  if (members.length === 1) rpg.unitFrame("Party", { ...members[0]!, size: phone ? "medium" : "large", anchor: "topLeft" });
  else rpg.partyFrames("Party", { members, anchor: "topLeft" });
  // The purse and Pause, top right (on a phone, the purse under Pause); a conversation's own tools (Skip, Log) stand there while it runs.
  const purse = () => capsule("Gold", () => {
    og.ui.icon("Coin", "coin", { size: 20, tint: { token: "gold" } });
    og.ui.label("Count", `${g.gold}`, { step: "lg", font: "mono", nowrap: true });
    og.ui.icon("Potion", "potion", { size: 20, tint: { token: g.potions > 0 ? "danger" : "controlStrong" } });
    og.ui.label("Potions", `${g.potions}`, { step: "lg", font: "mono", nowrap: true });
    if (g.seal) og.ui.icon("Seal", "key", { size: 20, tint: { token: "primary" } });
  });
  // Pause acts through the seat's input (src/seats.ts), so a room pauses on one tick.
  const pause = () => {
    if (og.ui.actionButton("Pause", say(world, "hud.pause"), { icon: "pause", size: 48 }).clicked) press("pause");
  };
  // The quest: the tracker on the right; on a phone, under the purse, clear of the hero at the centre.
  const quests = og.story.quests(world).filter((q) => q.status === "active");
  const tracker = (anchor: "right" | null) => {
    if (quests.length > 0) rpg.questTracker("Tracker", { quests, max: 1, width: phone ? 220 : 280, anchor, completeLabel: say(world, "quest.complete") });
  };
  if (g.conversation === 0) {
    og.ui.anchor("Purse", { anchor: "topRight", gap: S.sm, cross: phone ? "end" : "center", direction: phone ? "column" : "row" }, () => {
      if (phone) {
        pause();
        purse();
        tracker(null);
      } else {
        purse();
        pause();
      }
    });
    if (!phone) tracker("right");
  }
  // The last struck risen's frame, at the top, while the fight is fresh.
  if (g.target !== 0 && !phone && g.conversation === 0) {
    const f = world.get(g.target as never, Foe);
    if (f.ok && f.value.hp > 0) {
      const captain = f.value.kind === Kind.Captain;
      rpg.unitFrame("Target", { key: "target", name: say(world, captain ? "foe.captain" : "foe.minion"), level: captain ? "3" : "1", health: f.value.hp, healthMax: f.value.maxHp, hostile: true, unitClass: captain ? "elite" : "normal", targeted: true, size: "medium", reverse: true, anchor: "top" });
    }
  }
  if (g.ending !== Ending.None) return;
  if (g.conversation !== 0) {
    conversation(world, g);
    return;
  }
  // Each seat's prompt by Maren or the chest: in its own half of a split screen, or the hero this device plays.
  const pad = must(world.resource(og.Gamepad)).connected();
  for (const seat of LOCAL ? [0, 1] : localSeats(world)) {
    const me = must(world.get(heroOf(g, seat), Hero));
    if (me.near === Near.None || me.dead) continue;
    const offsetX = LOCAL ? (seat === 0 ? -1 : 1) * (vp.width / 4) : 0;
    const key = me.near === Near.Maren ? "hud.talk" : "hud.open";
    const glyph = touch && seat === 0 ? "hud.glyph.touch" : pad ? "hud.glyph.pad" : LOCAL && seat === 1 ? "hud.glyph.keys2" : "hud.glyph.keys";
    og.ui.anchor(seat === 0 || !LOCAL ? "Prompt" : `Prompt${seat}`, { anchor: "bottom", offsetX, offsetY: touch ? -170 : -28, direction: "column", cross: "center", gap: S.xs }, () => {
      capsule("Hint", () => {
        og.ui.label("Glyph", say(world, glyph), { font: "mono", nowrap: true, color: { token: "primary" } });
        og.ui.label("Text", say(world, key), { nowrap: true });
      });
    });
  }
}

/**
 * Maren's conversation as a box at the bottom (the rpg kit): its line, and
 * its replies numbered. A tap or click on a reply, or on the line, is the
 * talker's choice or Talk (press), and the Story system answers it.
 */
function conversation(world: World, g: GameV): void {
  const v: DialogueView | null = og.story.view(world, g.conversation);
  if (v === null) return;
  const r = rpg.conversation("Talk", {
    view: v,
    layout: "box",
    // Whole at once: a press acts on the line the world shows, the same on every peer of a room (a reveal is this device's own).
    charsPerSecond: 0,
    continueLabel: say(world, "talk.continue"),
    unavailableLabel: say(world, "talk.unavailable"),
    skipLabel: say(world, "talk.skip"),
    logLabel: say(world, "talk.log"),
    autoLabel: say(world, "talk.auto"),
  });
  const seat = LOCAL ? Math.max(0, g.talker) : 0;
  if (r.chosen !== null) {
    const n = v.choices.findIndex((c) => c.index === r.chosen);
    if (n >= 0 && n < CHOICES) press(`choice${n + 1}`, seat);
  } else if (r.advanced) press("talk", seat);
}

/** What the touch controls need from the world, kept by Ui each tick (they are drawn by Seats, before Ui). */
const touchHud = { shown: false, world: null as World | null, near: [false, false] };
/** Each local seat's look finger on this device: the pointer, and where it last was. */
const lookFinger = [{ id: -1, x: 0 }, { id: -1, x: 0 }];

/** Degrees the look turns per CSS px a finger drags across the look zone. */
const LOOK_DEG_PER_PX = 0.45;

/**
 * Greywatch's touch controls for a seat, drawn by Seats while touch shows
 * and a run is on screen. Alone: the stick bottom left, Attack large
 * bottom right with Talk and Potion over it (Talk lit by Maren or the
 * chest), and a finger landing on the right half (below the purse, above
 * the buttons) turning the look as it drags sideways (on the isometric
 * view, the hero). Two on one screen: each seat its own corner (seat 0
 * left, seat 1 right), its buttons over its stick, and its own half to
 * drag the look in.
 */
function touchControls(seat: number): { x: number; y: number; lx: number; ly: number; buttons: number } {
  const out = { x: 0, y: 0, lx: 0, ly: 0, buttons: 0 };
  const world = touchHud.world;
  const finger = lookFinger[seat]!;
  if (!touchHud.shown || world === null) {
    finger.id = -1;
    return out;
  }
  const vp = og.ui.frameViewport();
  const corner = LOCAL ? (seat === 0 ? "bottomLeft" : "bottomRight") : "bottomLeft";
  const stick = () => {
    const s = og.ui.joystick("Move", { size: LOCAL ? 116 : 136, deadZone: 0.12 });
    if (s.active) {
      out.x = s.x;
      out.y = -s.y;
    }
  };
  const hands = (big: number, small: number) => {
    const talk = og.ui.actionButton("Talk", say(world, "hud.button.talk"), { icon: "person", size: small, primary: touchHud.near[seat] });
    if (talk.pressed) out.buttons |= bit("talk");
    if (og.ui.actionButton("Potion", say(world, "hud.button.potion"), { icon: "potion", size: small }).pressed) out.buttons |= bit("potion");
    if (og.ui.actionButton("Attack", say(world, "hud.button.attack"), { icon: "sword", size: big, primary: true }).pressed) out.buttons |= bit("attack");
  };
  if (LOCAL) {
    og.ui.anchor(`Hands${seat}`, { anchor: corner, direction: "column", gap: S.md, cross: "center" }, () => {
      og.ui.row("Buttons", { gap: S.sm, cross: "end" }, () => hands(64, 52));
      stick();
    });
  } else {
    og.ui.anchor(`Move${seat}`, { anchor: "bottomLeft" }, stick);
    og.ui.anchor(`Hands${seat}`, { anchor: "bottomRight", direction: "row", gap: S.md, cross: "end" }, () => {
      og.ui.column("Small", { gap: S.md, cross: "center" }, () => hands(84, 60));
    });
  }
  // The look: a finger dragged sideways in this seat's zone, degrees this tick (right turns right).
  const ptr = must(world.resource(og.Pointers));
  if (finger.id >= 0 && !ptr.down(finger.id)) finger.id = -1;
  if (finger.id >= 0) {
    const x = ptr.position(finger.id).x;
    out.lx = (x - finger.x) * LOOK_DEG_PER_PX; // lockstep-ok: this device's input, sent to the room
    finger.x = x;
  } else {
    const left = LOCAL ? (seat === 0 ? 0 : vp.width / 2) : vp.width / 2;
    const right = LOCAL ? (seat === 0 ? vp.width / 2 : vp.width) : vp.width;
    const top = vp.safeArea.top + 110;
    const bottom = vp.height - vp.safeArea.bottom - (LOCAL ? 300 : 240);
    for (let n = 0; n < 5; n++) {
      if (!ptr.pressed(n) || lookFinger.some((f) => f.id === n)) continue;
      const at = ptr.start(n);
      if (at.x < left || at.x >= right || at.y < top || at.y > bottom) continue;
      finger.id = n;
      finger.x = ptr.position(n).x;
      break;
    }
  }
  return out;
}
useTouchControls(touchControls);

/** A seat's control went down this tick (against Hero.before, which the Heroes system moves on after Ui). */
function pressedBy(world: World, g: GameV, name: string): boolean {
  for (let seat = 0; seat < SEATS; seat++) {
    const hero = heroOf(g, seat);
    if (pressedNow(must(world.get(hero, og.PlayerInput)).buttons, must(world.get(hero, Hero)).before, name)) return true;
  }
  return false;
}

/**
 * The seats' choices, from their input: Pause opens and closes the pause
 * menu (under the settings or the bag it closes them); on the title
 * choice1 enters the keep and choice3 opens the settings; paused, Menu
 * leaves it, choice2 opens the bag and choice3 the settings; on the summary choice1
 * goes again and Menu goes back to the title.
 */
function choices(world: World, g: GameV, p: PrefsV): void {
  if (g.settingsOpen) {
    if (pressedBy(world, g, "pause")) closeSettings(world, g, p);
    return;
  }
  if (g.bagOpen) {
    if (pressedBy(world, g, "pause")) g.bagOpen = false;
    return;
  }
  if (pressedBy(world, g, "pause")) {
    if (g.paused) g.paused = false;
    else if (g.phase === Phase.Play && g.ending === Ending.None) g.paused = true;
  }
  if (g.phase === Phase.Title) {
    if (pressedBy(world, g, "choice1")) startRun(world, g);
    else if (pressedBy(world, g, "choice3")) g.settingsOpen = true;
  } else if (g.phase === Phase.Play) {
    if (g.paused && pressedBy(world, g, "menu")) toTitle(world, g);
    else if (g.paused && pressedBy(world, g, "choice2")) {
      g.bagOpen = true;
      og.ui.remember("pause.from", 2);
    } else if (g.paused && pressedBy(world, g, "choice3")) {
      g.settingsOpen = true;
      og.ui.remember("pause.from", 1);
    }
  } else if (g.overTicks >= SUMMARY_TICKS) {
    if (pressedBy(world, g, "choice1")) startRun(world, g);
    else if (pressedBy(world, g, "menu")) toTitle(world, g);
  }
}

function settings(world: World, g: GameV, p: PrefsV): void {
  let done = false;
  const before = JSON.stringify(prefsValue(p));
  // Locked: Back reaches the game as the seat's Pause, which closes them (choices).
  og.ui.modal("Settings", g.settingsOpen, { title: say(world, "settings.title"), locked: true, width: px(Math.min(420, og.ui.frameViewport().width - 32)) }, () => {
    const slider = (id: string, key: string, value: number): number => {
      let out = value;
      og.ui.row(id, { width: grow, gap: S.md, cross: "center" }, () => {
        og.ui.label("Label", say(world, key), { width: px(110), nowrap: true });
        out = og.ui.slider("Slider", value, { step: 0.05 }).value;
        og.ui.label("Value", `${Math.round(out * 100)}`, { width: px(40), align: "end", font: "mono", nowrap: true });
      });
      return out;
    };
    p.master = slider("Master", "settings.master", p.master);
    p.music = slider("Music", "settings.music", p.music);
    p.effects = slider("Effects", "settings.effects", p.effects);
    og.ui.label("LanguageLabel", say(world, "settings.language"), { color: { token: "inkMuted" } });
    const r = og.ui.radio("Language", LANGUAGES.map((l) => say(world, l.key)), p.language, { horizontal: true });
    if (r.selected !== p.language) setLanguage(world, p, r.selected);
    p.reducedMotion = og.ui.toggle("Motion", p.reducedMotion, say(world, "settings.motion")).on;
    og.ui.row("Footer", { width: grow, align: "end" }, () => {
      if (og.ui.button("Done", say(world, "settings.done"), { variant: "primary" }).clicked) done = true;
    });
  });
  if (JSON.stringify(prefsValue(p)) !== before) applyPrefs(world, p);
  // Done acts through the seat's Pause, as Escape does: the settings close on one tick on every peer.
  if (g.settingsOpen && done) press("pause");
}

/** The settings close (Done, or a seat's Pause: Escape, Start) and are saved. */
function closeSettings(world: World, g: GameV, p: PrefsV): void {
  g.settingsOpen = false;
  must(og.platform.saveSettings(world, prefsValue(p)));
  p.saves += 1;
}

/** What the party carries, as the rpg kit's grid shows it: each hero's weapon, the draughts, the gold, the seal. */
function bagItems(world: World, g: GameV): GridItem[] {
  const items: GridItem[] = [
    { key: "sword", name: say(world, "item.sword"), icon: "sword", x: 0, y: 0, w: 1, h: 2, rarity: "uncommon", equipped: true },
    { key: "shield", name: say(world, "item.shield"), icon: "shield", x: 1, y: 0, w: 1, h: 2, rarity: "common", equipped: true },
  ];
  if (SEATS > 1) items.push({ key: "axe", name: say(world, "item.axe"), icon: "sword", x: 2, y: 0, w: 1, h: 2, rarity: "uncommon", equipped: true });
  if (g.potions > 0) items.push({ key: "potion", name: say(world, "item.potion"), icon: "potion", x: 4, y: 0, count: g.potions, maxStack: 9, rarity: "common" });
  if (g.gold > 0) items.push({ key: "gold", name: say(world, "item.gold"), icon: "coin", x: 5, y: 0, count: g.gold, maxStack: 999, rarity: "common" });
  if (g.seal) items.push({ key: "seal", name: say(world, "item.seal"), icon: "key", x: 4, y: 1, rarity: "rare", tag: say(world, "item.tag.quest"), fresh: g.quest === Quest.Return });
  return items;
}

/** The bag over the pause: the party's inventory and the journal (the rpg kit's grid and log). */
function bag(world: World, g: GameV): void {
  const vp = og.ui.frameViewport();
  const phone = vp.width < 600;
  let close = false;
  og.ui.modal("Bag", g.bagOpen, { title: say(world, "bag.title"), locked: true, width: px(Math.min(phone ? 380 : 760, vp.width - 24)) }, () => {
    const body = () => {
      rpg.inventoryGrid("Grid", { columns: 6, rows: 3, items: bagItems(world, g), label: say(world, "bag.items"), capacity: 18, emptyLabel: say(world, "bag.empty"), cellSize: phone ? 48 : 56 });
      const quests = og.story.quests(world);
      if (quests.length > 0) rpg.questLog("Log", { quests, selected: QUEST, title: say(world, "bag.journal"), objectivesLabel: say(world, "quest.objectives"), rewardsLabel: say(world, "quest.rewards"), giverLabel: say(world, "quest.giver"), activeLabel: say(world, "quest.active"), completedLabel: say(world, "quest.completed"), emptyLabel: say(world, "bag.nojournal"), trackLabel: say(world, "quest.track"), trackedLabel: say(world, "quest.tracked") });
      else og.ui.label("NoQuest", say(world, "bag.nojournal"), { color: { token: "inkMuted" }, width: grow });
    };
    // Scrolled inside the screen: the journal's detail runs long on a small one.
    og.ui.scrollArea("Scroll", { width: grow, height: px(Math.max(200, Math.min(phone ? 560 : 470, vp.height - 190))) }, () => {
      if (phone) og.ui.column("Body", { width: grow, gap: S.md }, body);
      else og.ui.row("Body", { width: grow, gap: S.lg }, body);
    });
    og.ui.row("Footer", { width: grow, align: "end" }, () => {
      if (og.ui.button("Close", say(world, "bag.close"), { variant: "primary" }).clicked) close = true;
    });
  });
  if (g.bagOpen && close) press("pause");
}

function seconds(s: number): string {
  const m = Math.floor(s / 60);
  const r = Math.floor(s % 60);
  return `${m}:${r < 10 ? "0" : ""}${r}`;
}

/** The run's summary: how it ended, how long it took, what fell and what was found, the record. */
function summary(world: World, g: GameV): "again" | "menu" | null {
  let chose: "again" | "menu" | null = null;
  const won = g.ending === Ending.Won;
  let level = 1;
  for (let seat = 0; seat < SEATS; seat++) level = Math.max(level, must(world.get(heroOf(g, seat), Hero)).level);
  og.ui.modal("Summary", g.phase === Phase.Over, { title: say(world, won ? "over.won" : "over.fallen"), locked: true, width: px(Math.min(400, og.ui.frameViewport().width - 32)) }, () => {
    og.ui.row("Mark", { width: grow, align: "center" }, () => {
      og.ui.transition("Icon", true, { kind: "pop" }, () => og.ui.icon("Glyph", won ? "crown" : "skull", { size: 48, tint: { token: won ? "primary" : "inkMuted" } }));
      // From a third of a second into the summary, every frame: a burst plays only while it is called.
      if (won && g.overTicks >= SUMMARY_TICKS) og.ui.burst({ count: 14 });
    });
    og.ui.label("Line", say(world, won ? "over.won.line" : "over.fallen.line"), { color: { token: "inkMuted" }, align: "center", width: grow });
    og.ui.column("Stats", { width: grow, gap: 0 }, () => {
      og.ui.keyValue("Time", say(world, "over.time"), seconds(g.time), true);
      og.ui.keyValue("Slain", say(world, "over.slain"), `${g.slain}`, true);
      og.ui.keyValue("Level", say(world, "over.level"), `${level}`, true);
      og.ui.keyValue("Gold", say(world, "over.gold"), `${g.gold}`, true);
      og.ui.keyValue("Best", say(world, "over.best"), g.best > 0 ? seconds(g.best) + (g.newBest ? " ★" : "") : say(world, "over.none"), false);
    });
    og.ui.label("Record", say(world, "over.record", { runs: g.runs, wins: g.wins }), { color: { token: "inkMuted" }, align: "center", width: grow });
    og.ui.row("Actions", { width: grow, gap: S.sm }, () => {
      if (og.ui.button("Again", say(world, "over.again"), { variant: "primary", width: grow }).clicked) chose = "again";
      if (og.ui.button("Menu", say(world, "over.menu"), { width: grow }).clicked) chose = "menu";
    });
  });
  return chose;
}

/**
 * A screen document up while `shown`: og.ui.screen pushes it the first tick
 * and builds it with its viewmodel each tick; the tick it is not shown it is
 * popped (the top layer: the title and the pause never stack). Answers the
 * events it emitted (its buttons' `emit`, Back's).
 */
function layer(world: World, name: string, shown: boolean, vm: object = {}): string[] {
  if (!shown) {
    if (og.ui.layers(world).some((l) => l.screen === name)) og.ui.pop(world);
    return [];
  }
  return og.ui.screen(name, vm).events.filter((e) => e.kind === "emit" || e.kind === "back").map((e) => e.name);
}

/** The screens, the run's start, clock and end, the hold, the buses, the platform's answers. */
export const Ui = defineSystem("Ui", { reads: [Tuning, og.Gamepad, og.Pointers, og.PlayerInput, og.Lockstep, Hero, Foe], writes: [Game, og.Audio] }, (_q, time, world) => {
  const g = must(world.resource(Game));
  const p = prefs;
  platform(world, g, p);
  if (time.tick === 0) applyPrefs(world, p);
  choices(world, g, p);
  const hold = g.phase === Phase.Play && (g.paused || g.settingsOpen || g.bagOpen);
  if (hold !== g.held) {
    must(og.time.setScale(world, hold ? 0 : 1));
    g.held = hold;
  }
  clock(world, g, time.scaledDt);

  const screen = g.phase === Phase.Title ? "title" : g.phase === Phase.Play ? "play" : "over";
  const shown = og.ui.screenTransition("Screens", screen, { kind: p.reducedMotion ? "fade" : "dissolve", durationMs: 600 }).shown;
  // The title and the pause are the game's screen documents (assets/ui/main-menu.ogt, pause.ogt).
  for (const e of layer(world, "main-menu", shown === "title", { touch: usesTouch() })) {
    if (!g.settingsOpen && e === "play" && g.phase === Phase.Title) press("choice1");
    if (e === "settings") press("choice3");
  }
  // Back from its Settings or the bag the pause keeps the focus on that button.
  const from = og.ui.memory("pause.from", 0);
  const pauseUp = shown === "play" && g.paused && !g.settingsOpen && !g.bagOpen;
  for (const e of layer(world, "pause", pauseUp)) {
    if (e === "resume") press("pause");
    // The bag and the settings open through the seat's input (choice2, choice3), so a room opens them on one tick.
    if (e === "settings") press("choice3");
    if (e === "bag") press("choice2");
    if (e === "quit") press("menu");
  }
  if (pauseUp && from !== 0) {
    og.ui.setFocus(from === 1 ? "pause/Pause/Panel/Menu/Settings" : "pause/Pause/Panel/Menu/Bag");
    og.ui.remember("pause.from", 0);
  }
  if (shown === "title") {
    og.ui.setNavigation(true);
  } else if (shown === "play") {
    hud(world, g);
  } else {
    og.ui.setNavigation(true);
    const c = summary(world, g);
    if (c === "again") press("choice1");
    if (c === "menu") press("menu");
  }
  // The mouse turns the view only in a run's fight: in a conversation and on the screens the pointer is the player's.
  // The touch controls (drawn by Seats next tick): in a run, not in a conversation (a tap there is a reply).
  touchHud.shown = shown === "play" && g.conversation === 0 && g.ending === Ending.None;
  touchHud.world = world;
  for (let seat = 0; seat < SEATS; seat++) touchHud.near[seat] = must(world.get(heroOf(g, seat), Hero)).near !== Near.None;
  setLookLive(g.phase === Phase.Play && !g.paused && !g.settingsOpen && !g.bagOpen && g.conversation === 0);
  bag(world, g);
  settings(world, g, p);
});
