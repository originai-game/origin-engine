// The other families' play screens (showcases.ts): MOBA, RPG, MMO,
// roguelike, tower defence, rhythm, fighting, survival, horror and open
// world, each over its gameplay scene (assets/textures/art/scenes/genre-*),
// built from the HUD components and the world UI as huds.ts builds the
// first five. Every word is keyed in assets/strings (sc.*); names stay as
// they are (invented heroes, places and foes).

import { og } from "@origingame/engine";
import type { AbilitySlotDesc, Paint, Size, StatusEffect, World } from "@origingame/engine";
import { L } from "./patterns.ts";
import { at, frame, gameScene, held, hudArt, pauseButton } from "./huds.ts";

const grow: Size = { kind: "grow" };
const px = (n: number): Size => ({ kind: "fixed", px: Math.max(0, Math.round(n)) });
const rgb = (r: number, g: number, b: number): Paint => ({ color: { r, g, b } });

/** A HUD chip: a row on the style's HUD frame (the score, the clock, a counter). */
function chip(id: string, body: () => void, opts: { anchor?: Parameters<typeof og.ui.anchor>[1]["anchor"]; offsetY?: number } = {}): void {
  const { t, m } = frame();
  og.ui.anchor(id, { anchor: opts.anchor ?? "top", margin: m, offsetY: opts.offsetY ?? 0, direction: "row", gap: t.space.sm, cross: "center", padding: { left: t.space.md, top: t.space.xs, right: t.space.md, bottom: t.space.xs }, background: { token: "hud" }, radius: 999, frame: "hud" }, body);
}

/** The thumbs' controls on a touch screen: the d-pad at the start, the buttons (primary last, largest) at the end. */
function thumbs(buttons: ReadonlyArray<{ id: string; label: string; icon: string; primary?: boolean }>): void {
  const { t, m } = frame();
  og.ui.anchor("Move", { anchor: "bottomLeft", margin: m }, () => og.ui.dpad("Dpad", {} as never));
  // The secondaries in a row over the primary, so the column stays clear of the d-pad on a 390 px phone.
  og.ui.anchor("Buttons", { anchor: "bottomRight", margin: m, direction: "column", gap: t.space.md, cross: "end" }, () => {
    og.ui.row("More", { gap: t.space.md, cross: "end" }, () => {
      for (const b of buttons) if (!b.primary) og.ui.actionButton(b.id, b.label, { icon: b.icon, size: 60 });
    });
    for (const b of buttons) if (b.primary) og.ui.actionButton(b.id, b.label, { primary: true, icon: b.icon, size: 80 });
  });
}

// ── MOBA and arena ───────────────────────────────────────────────────────

/**
 * A MOBA's HUD (League of Legends, Mobile Legends): the teams' kills and
 * the clock top centre, the hero's frame and the four abilities with the
 * ultimate along the foot, the lane map in the end corner, the heroes'
 * plates over them in the world; on a touch screen the stick and the
 * ability buttons round the attack.
 */
export function mobaHud(world: World): void {
  const { t, small, touch, k, m } = frame();
  og.ui.setNavigation(false);
  void world;
  gameScene(world, "scenes/genre-moba", { x: 0.5, y: 0.5 });
  const tick = og.ui.tick();
  og.ui.nameplate("Foe", { target: { point: at(0.66, 0.42) }, name: "Morrow", level: "11", reaction: "hostile", health: 0.42, targeted: true });
  og.ui.nameplate("Ally", { target: { point: at(0.34, 0.5) }, name: "Ilsa", level: "12", reaction: "friendly", health: 0.78 });
  og.ui.nameplate("Me", { target: { point: at(0.48, 0.58) }, name: L("sc.you"), level: "12", reaction: "self", health: 0.64 });
  // On a phone the map takes the start corner and the score the end one, lowered under the map where the two would meet
  // (large text: both as laid last tick); the hero's own plate carries its health.
  const vp0 = og.ui.frameViewport();
  const mapB = og.ui.bounds("MapHud");
  const scoreB = og.ui.bounds("Score");
  const lower = small && mapB !== null && held("mobaScore", `${vp0.width}x${vp0.height}|${t.textScale ?? 1}|${t.style}`, scoreB !== null && scoreB.x < mapB.x + mapB.width + t.space.sm && scoreB.y < mapB.y + mapB.height);
  chip("Score", () => {
    og.ui.label("Blue", "14", { step: "xl", font: "display", color: { token: "info" }, nowrap: true });
    og.ui.icon("Swords", "sword", { size: Math.round(18 * k) } as never);
    og.ui.label("Red", "9", { step: "xl", font: "display", color: { token: "danger" }, nowrap: true });
    const secs = 14 * 60 + 32 + Math.floor(tick / 60);
    og.ui.label("Clock", `${Math.floor(secs / 60)}:${String(secs % 60).padStart(2, "0")}`, { step: "base", font: "mono", color: { token: "inkMuted" }, nowrap: true });
  }, { anchor: small ? "topRight" : "top", offsetY: lower && mapB ? Math.round(mapB.y + mapB.height + t.space.sm - (vp0.safeArea.top + m)) : 0 });
  og.ui.minimapView("Map", {
    texture: hudArt("maps/world"), size: Math.round((small ? 112 : 176) * k), shape: "square", north: "",
    markers: [{ key: "me", kind: "player", x: 0.46, y: 0.56, heading: 40 }, { key: "ilsa", kind: "ally", x: 0.38, y: 0.62 }, { key: "morrow", kind: "enemy", x: 0.6, y: 0.4 }, { key: "baron", kind: "objective", x: 0.7, y: 0.3 }],
    anchor: touch ? "topLeft" : "bottomRight",
  });
  if (!touch) og.ui.unitFrame("Hero", { name: "Cassia", level: "12", portrait: hudArt("portraits/mira"), health: 0.64, resource: 0.48, resourceKind: "mana", size: "large", anchor: "bottomLeft" });
  const cd = (n: number, max: number) => Math.max(0, n - (tick % (max * 60)) / 60);
  if (touch) {
    thumbs([{ id: "Q", label: L("sc.moba.dash"), icon: "bolt" }, { id: "W", label: L("sc.moba.ward"), icon: "shield" }, { id: "R", label: L("sc.moba.ult"), icon: "fire" }, { id: "Attack", label: L("sc.moba.attack"), icon: "sword", primary: true }]);
  } else {
    const slots: AbilitySlotDesc[] = [
      { key: "q", image: hudArt("boons/thunder-strike"), name: L("sc.moba.dash"), hotkey: "Q", cooldown: cd(6, 6), cooldownMax: 6, level: 4, maxLevel: 5 },
      { key: "w", image: hudArt("boons/rune-shield"), name: L("sc.moba.ward"), hotkey: "W", level: 3, maxLevel: 5 },
      { key: "e", image: hudArt("boons/frost"), name: L("sc.moba.frost"), hotkey: "E", cost: "60", unaffordable: true, level: 3, maxLevel: 5 },
      { key: "r", image: hudArt("boons/flame-blade"), name: L("sc.moba.ult"), hotkey: "R", ultimate: Math.min(1, 0.7 + (tick % 600) / 2000), level: 2, maxLevel: 3 },
    ];
    og.ui.abilityBar("Abilities", { slots, size: Math.round(60 * k), gap: t.space.sm, anchor: "bottom" });
    // The gold stands beside the abilities' end (both as laid last tick, so at large text it moves out with them).
    const bar = og.ui.bounds("AbilitiesHud");
    const gold = og.ui.bounds("Gold");
    const goldX = bar && gold ? Math.round(bar.width / 2 + t.space.md + gold.width / 2) : Math.round(220 * k);
    og.ui.anchor("Gold", { anchor: "bottom", margin: m, offsetX: goldX, direction: "row", gap: t.space.xs, cross: "center" }, () => {
      og.ui.icon("Coin", "coin", { size: Math.round(20 * k), tint: { token: "gold" } } as never);
      og.ui.counter("Amount", 2_340 + Math.floor(tick / 30) * 2, { step: "lg", font: "rounded" });
    });
  }
}

// ── RPG and ARPG ─────────────────────────────────────────────────────────

/**
 * An action RPG's HUD (Diablo, Path of Exile): health and mana as orbs at
 * the foot's ends with the skills and the potions between them and the XP
 * under them, the quest tracker on the end edge, the minimap top end, an
 * elite's plate over it and the hits' numbers.
 */
export function rpgHud(world: World): void {
  const { t, small, portrait, touch, k, m } = frame();
  og.ui.setNavigation(false);
  void world;
  gameScene(world, "scenes/genre-rpg", { x: 0.5, y: 0.55 });
  const tick = og.ui.tick();
  const foe = at(portrait ? 0.62 : 0.6, 0.42);
  og.ui.nameplate("Elite", { target: { point: foe }, name: "Gorvane the Hollow", title: L("sc.rpg.elite"), level: "24", reaction: "hostile", unitClass: "elite", health: 0.58 - 0.1 * ((tick % 600) / 600), targeted: true });
  if (tick % 90 === 20) og.ui.damageNumbers("Hits", { hits: [{ position: foe, amount: 1_240 + (tick % 7) * 31, kind: (tick / 90) % 3 === 2 ? "critical" : "normal", seed: tick }] });
  og.ui.minimapView("Map", { texture: hudArt("maps/world"), size: Math.round((small ? 104 : 164) * k), shape: "circle", zone: L("sc.rpg.zone"), north: "", markers: [{ key: "me", kind: "player", x: 0.5, y: 0.5, heading: 30 }, { key: "q", kind: "objective", x: 0.62, y: 0.38 }, { key: "f", kind: "enemy", x: 0.56, y: 0.44 }], anchor: "topRight" });
  if (!touch) {
    // Centred on the end edge, and lowered under the minimap's zone plate where it would reach it (large text): both
    // as laid last tick; the tracker's height does not depend on where it stands.
    const vp = og.ui.frameViewport();
    const map = og.ui.bounds("MapHud");
    const tracker = og.ui.bounds("QuestsHud");
    const down = map && tracker ? Math.max(0, Math.round(map.y + map.height + t.space.sm - (vp.height - tracker.height) / 2)) : 0;
    og.ui.anchor("QuestsHud", { anchor: "right", margin: m, offsetY: down }, () => og.ui.objectiveTracker("Quests", {
      quests: [
        { key: "crypt", title: L("sc.rpg.quest"), icon: "skull", focused: true, steps: [{ key: "seal", text: L("sc.rpg.seals"), count: 2, target: 3 }, { key: "boss", text: L("sc.rpg.boss") }] },
        { key: "herbs", title: L("sc.rpg.side"), icon: "leaf", steps: [{ key: "moss", text: L("sc.rpg.moss"), count: 4, target: 6 }] },
      ],
      // At large text the focused quest alone, so the tracker fits under the minimap.
      width: Math.round(260 * k * Math.min(1.5, t.textScale ?? 1)), max: (t.textScale ?? 1) >= 1.5 ? 1 : 0, maxSteps: (t.textScale ?? 1) >= 1.5 ? 1 : 0, anchor: null,
    }));
  }
  if (touch) {
    og.ui.anchor("Bars", { anchor: "topLeft", margin: m, direction: "column", gap: t.space.xs }, () => {
      og.ui.statBar("Health", { value: 640, max: 900, kind: "health", ghost: true, showValue: true, width: px(170 * k), thickness: 14, name: L("sc.health"), anchor: null });
      og.ui.statBar("Mana", { value: 210, max: 400, kind: "mana", width: px(130 * k), thickness: 8, name: L("sc.mana"), anchor: null });
    });
    thumbs([{ id: "Potion", label: L("sc.rpg.potion"), icon: "potion" }, { id: "Skill", label: L("sc.rpg.whirl"), icon: "fire" }, { id: "Attack", label: L("sc.action.attack"), icon: "sword", primary: true }]);
    return;
  }
  const orb = Math.round(118 * k);
  og.ui.orbMeter("Health", { value: 640, max: 900, kind: "health", size: orb, showValue: true, label: L("sc.health"), name: L("sc.health"), anchor: "bottomLeft" });
  og.ui.orbMeter("Mana", { value: 210, max: 400, kind: "mana", size: orb, showValue: true, label: L("sc.mana"), name: L("sc.mana"), anchor: "bottomRight" });
  og.ui.anchor("Belt", { anchor: "bottom", margin: m, direction: "column", gap: t.space.xs, cross: "center" }, () => {
    const slots: AbilitySlotDesc[] = [
      { key: "potion", image: hudArt("items/healing-draught"), name: L("sc.rpg.potion"), hotkey: "1", charges: 3, maxCharges: 5 },
      { key: "mana", image: hudArt("items/mana-vial"), name: L("sc.rpg.vial"), hotkey: "2", charges: 2, maxCharges: 5 },
      { key: "whirl", image: hudArt("boons/flame-blade"), name: L("sc.rpg.whirl"), action: "ability1", cooldown: Math.max(0, 4 - (tick % 240) / 60), cooldownMax: 4 },
      { key: "ward", image: hudArt("boons/rune-shield"), name: L("sc.rpg.ward"), action: "ability2" },
      { key: "nova", image: hudArt("boons/frost"), name: L("sc.rpg.nova"), action: "ability3", cost: "80", unaffordable: true },
      { key: "storm", image: hudArt("boons/thunder-strike"), name: L("sc.rpg.storm"), action: "ultimate", ultimate: 0.82 },
    ];
    og.ui.abilityBar("Skills", { slots, size: Math.round(54 * k), gap: t.space.xs, anchor: null });
    og.ui.statBar("Xp", { value: 0.62, max: 1, kind: "xp", width: px(420 * k), thickness: 6, label: L("sc.rpg.level"), name: L("sc.rpg.xp"), anchor: null });
  });
}

// ── MMO and raid ─────────────────────────────────────────────────────────

/**
 * An MMO's raid HUD (World of Warcraft, Final Fantasy XIV): the player's
 * and the target's frames top start with the boss's cast under them, the
 * party's frames on the start edge, the action bars along the foot, the
 * minimap and the buffs top end, the encounter's objective on the end
 * edge.
 */
/** Whether the rhythm HUD's score stands at the foot, for the look it was measured in (rhythmHud). */
let rhythmScore = { look: "", foot: false };

/** Whether the MMO's touch buttons stand as a column, for the look it was measured in (mmoHud). */
let mmoButtons = { look: "", column: false };

export function mmoHud(world: World): void {
  const { t, small, portrait, touch, k, m } = frame();
  og.ui.setNavigation(false);
  void world;
  gameScene(world, "scenes/genre-mmo", { x: 0.5, y: 0.5 });
  const tick = og.ui.tick();
  const buffs: StatusEffect[] = [
    { key: "might", image: hudArt("boons/titan-heart"), name: L("sc.mmo.might"), remaining: 1_740 - tick / 60, duration: 3_600, mine: true },
    { key: "ward", image: hudArt("boons/rune-shield"), name: L("sc.mmo.ward"), remaining: 12 - (tick % 720) / 60, duration: 12, stacks: 2, mine: true },
    { key: "burn", image: hudArt("boons/curse"), name: L("sc.mmo.burn"), remaining: 6 - (tick % 360) / 60, duration: 6, debuff: true, type: "magic" },
  ];
  const cast = (tick % 240) / 180;
  og.ui.anchor("Frames", { anchor: "topLeft", margin: m, direction: touch ? "column" : "row", gap: t.space.sm, cross: "start" }, () => {
    og.ui.unitFrame("Player", { name: "Thessaly", level: "60", portrait: hudArt("portraits/maren"), health: 0.81, resource: 0.56, resourceKind: "mana", effects: buffs, combat: true, leader: true, size: small ? "small" : "large", anchor: null });
    og.ui.unitFrame("Target", { name: "Vorthul", title: L("sc.mmo.title"), level: "??", portrait: hudArt("portraits/cultist"), health: 0.37, hostile: true, unitClass: "boss", castName: cast < 1 ? L("sc.mmo.cast") : "", castProgress: Math.min(1, cast), targeted: true, size: small ? "small" : "large", anchor: null });
  });
  if (!touch) {
    og.ui.partyFrames("Party", {
      members: [
        { key: "brann", name: "Brann", level: "60", portrait: hudArt("portraits/brann"), health: 0.92, resource: 0.4, resourceKind: "primary", size: "small" },
        { key: "wren", name: "Wren", level: "60", portrait: hudArt("portraits/wren"), health: 0.34, resource: 0.7, resourceKind: "mana", size: "small", effects: [buffs[2]!] },
        { key: "torvin", name: "Torvin", level: "59", portrait: hudArt("portraits/torvin"), health: 0, dead: true, size: "small" },
        { key: "vesper", name: "Vesper", level: "60", portrait: hudArt("portraits/vesper"), health: 0.66, resource: 0.2, resourceKind: "mana", size: "small" },
      ],
      gap: t.space.xs,
      anchor: "left",
    });
    og.ui.objectiveTracker("Encounter", { quests: [{ key: "vault", title: L("sc.mmo.raid"), icon: "skull", focused: true, steps: [{ key: "adds", text: L("sc.mmo.adds"), count: 3, target: 4 }, { key: "boss", text: L("sc.mmo.boss"), progress: 0.63 }] }], width: Math.round(240 * k), anchor: "right" });
  }
  og.ui.minimapView("Map", { texture: hudArt("maps/world"), size: Math.round((small ? 96 : 150) * k), shape: "circle", zone: L("sc.mmo.zone"), north: "", markers: [{ key: "me", kind: "player", x: 0.5, y: 0.5, heading: 300 }, { key: "boss", kind: "enemy", x: 0.56, y: 0.4 }], anchor: portrait ? "bottomLeft" : "topRight" });
  if (touch) {
    // A row beside the minimap in the bottom start corner; a column where the row would reach the map (large text
    // widens its zone plate): both as laid last tick, held for the look.
    const map = og.ui.bounds("MapHud");
    const row = og.ui.bounds("Buttons");
    const look = `${og.ui.frameViewport().width}|${t.textScale ?? 1}|${t.style}`;
    if (mmoButtons.look !== look) mmoButtons = { look, column: false };
    if (map && row && !mmoButtons.column && row.x < map.x + map.width + t.space.sm) mmoButtons.column = true;
    og.ui.anchor("Buttons", { anchor: "bottomRight", margin: m, gap: t.space.md, cross: "end", ...(mmoButtons.column ? { direction: "column" as const } : {}) }, () => {
      og.ui.actionButton("Heal", L("sc.mmo.heal"), { icon: "plus", size: 60 });
      og.ui.actionButton("Shield", L("sc.mmo.ward"), { icon: "shield", size: 60 });
      og.ui.actionButton("Strike", L("sc.mmo.strike"), { primary: true, icon: "sword", size: 80 });
    });
    return;
  }
  const bar = (keys: string[], arts: Array<Parameters<typeof hudArt>[0]>, names: string[]): AbilitySlotDesc[] => keys.map((hotkey, i) => ({ key: `s${hotkey}`, hotkey, image: hudArt(arts[i % arts.length]!), name: L(names[i % names.length]!), cooldown: i === 2 ? Math.max(0, 9 - (tick % 540) / 60) : 0, cooldownMax: 9 }));
  // The consumables' bar over the main one: five slots, so the whole HUD stays inside the frame's element budget.
  og.ui.anchor("Bars", { anchor: "bottom", margin: m, direction: "column", gap: t.space.xs, cross: "center" }, () => {
    og.ui.abilityBar("Bar2", { slots: bar(["S1", "S2", "S3", "S4", "S5"], ["items/healing-draught", "items/mana-vial", "boons/star", "boons/apple", "items/iron-key"], ["sc.mmo.heal", "sc.rpg.vial", "sc.mmo.might", "sc.mmo.food", "sc.mmo.key"]), size: Math.round(40 * k), gap: 4, anchor: null });
    og.ui.abilityBar("Bar1", { slots: bar(["1", "2", "3", "4", "5", "6", "7", "8"], ["boons/flame-blade", "boons/thunder-strike", "boons/frost", "boons/rune-shield", "boons/titan-heart", "boons/poison", "boons/crystal", "boons/broken-chain"], ["sc.mmo.strike", "sc.rpg.storm", "sc.rpg.nova", "sc.mmo.ward", "sc.mmo.might", "sc.mmo.venom", "sc.mmo.blink", "sc.mmo.free"]), size: Math.round(48 * k), gap: 4, anchor: null });
  });
}

// ── roguelike ────────────────────────────────────────────────────────────

/**
 * A roguelike's run HUD (Hades, Dead Cells): health and the boons taken
 * top start, the chamber's number and its reward top centre, the run's
 * gold and keys and the four moves bottom end, a cursed foe's plate.
 */
export function rogueHud(world: World): void {
  const { t, small, touch, k, m } = frame();
  og.ui.setNavigation(false);
  void world;
  gameScene(world, "scenes/genre-roguelike", { x: 0.5, y: 0.55 });
  const tick = og.ui.tick();
  og.ui.nameplate("Foe", { target: { point: at(0.62, 0.46) }, name: "Bone Warden", reaction: "hostile", unitClass: "elite", health: 0.7, targeted: true });
  og.ui.anchor("Player", { anchor: "topLeft", margin: m, direction: "column", gap: t.space.sm }, () => {
    og.ui.statBar("Health", { value: 54, max: 80, kind: "health", ghost: true, showValue: true, width: px((small ? 170 : 240) * k), thickness: Math.round(14 * k), name: L("sc.health"), anchor: null });
    og.ui.statusGrid("Boons", {
      effects: [
        { key: "flame", image: hudArt("boons/flame-blade"), name: L("sc.rogue.flame"), stacks: 2 },
        { key: "frost", image: hudArt("boons/frost"), name: L("sc.rogue.frost") },
        { key: "thunder", image: hudArt("boons/thunder-strike"), name: L("sc.rogue.thunder") },
        { key: "heart", image: hudArt("boons/titan-heart"), name: L("sc.rogue.heart"), stacks: 3 },
        { key: "chain", image: hudArt("boons/broken-chain"), name: L("sc.rogue.curse"), debuff: true },
      ],
      columns: 5, size: Math.round((small ? 30 : 36) * k), timers: false, anchor: null,
    });
  });
  // Under a finger the chamber stands top end, or under the player's column where the two would meet (large text): from
  // both as laid last tick.
  const playerB = touch ? og.ui.bounds("Player") : null;
  const chamberB = touch ? og.ui.bounds("Chamber") : null;
  const chamberY = playerB && chamberB && playerB.x + playerB.width + t.space.sm > chamberB.x ? Math.round(playerB.y + playerB.height + t.space.sm - (og.ui.frameViewport().safeArea.top + m)) : 0;
  chip("Chamber", () => {
    og.ui.label("Room", L("sc.rogue.room", { n: 14 }), { step: "base", font: "display", nowrap: true });
    og.ui.icon("Reward", "gem", { size: Math.round(18 * k), tint: { token: "info" } } as never);
  }, { anchor: touch ? "topRight" : "top", offsetY: chamberY });
  // Under a finger the purse stands under the chamber's chip (as laid last tick, at any text size).
  const chamber = touch ? og.ui.bounds("Chamber") : null;
  const purseY = !touch ? 0 : chamber ? Math.round(chamber.y + chamber.height + t.space.sm - (og.ui.frameViewport().safeArea.top + m)) : Math.round(48 * k);
  og.ui.anchor("Purse", { anchor: touch ? "topRight" : "bottomLeft", margin: m, offsetY: purseY, direction: "row", gap: t.space.md, cross: "center" }, () => {
    og.ui.row("Gold", { gap: t.space.xs, cross: "center" }, () => {
      og.ui.icon("Coin", "coin", { size: Math.round(20 * k), tint: { token: "gold" } } as never);
      og.ui.counter("Amount", 312 + Math.floor(tick / 120) * 5, { step: "lg", font: "rounded" });
    });
    og.ui.row("Keys", { gap: t.space.xs, cross: "center" }, () => {
      og.ui.icon("Key", "key", { size: Math.round(20 * k) } as never);
      og.ui.counter("Amount", 3, { step: "lg", font: "rounded" });
    });
  });
  if (touch) {
    thumbs([{ id: "Dash", label: L("sc.rogue.dash"), icon: "bolt" }, { id: "Special", label: L("sc.rogue.special"), icon: "fire" }, { id: "Attack", label: L("sc.action.attack"), icon: "sword", primary: true }]);
    return;
  }
  og.ui.abilityBar("Moves", {
    slots: [
      { key: "attack", image: hudArt("items/iron-sword"), name: L("sc.action.attack"), action: "attack" },
      { key: "special", image: hudArt("boons/flame-blade"), name: L("sc.rogue.special"), action: "ability1" },
      { key: "cast", image: hudArt("boons/crystal"), name: L("sc.rogue.cast"), action: "ability2", charges: 1, maxCharges: 3 },
      { key: "dash", image: hudArt("boons/thunder-strike"), name: L("sc.rogue.dash"), action: "jump", cooldown: Math.max(0, 1 - (tick % 90) / 60), cooldownMax: 1 },
    ],
    size: Math.round(56 * k), anchor: "bottomRight",
  });
}

// ── tower defence ────────────────────────────────────────────────────────

/**
 * A tower defence's HUD (Kingdom Rush, Bloons TD): gold and lives top
 * start, the waves to come on the end edge with the next wave's call, the
 * towers to build along the foot with their costs, the chosen tower's
 * range in the world.
 */
export function tdHud(world: World): void {
  const { t, small, portrait, k, m } = frame();
  og.ui.setNavigation(false);
  void world;
  gameScene(world, "scenes/genre-strategy", { x: 0.5, y: 0.5 });
  const tick = og.ui.tick();
  og.ui.resourceBar("Purse", {
    items: [
      { key: "gold", icon: "coin", name: L("sc.td.gold"), amount: 420 + Math.floor(tick / 60) * 5 },
      { key: "lives", icon: "heart", name: L("sc.td.lives"), amount: 17, capacity: 20 },
    ],
    rates: false, anchor: "topLeft",
  });
  const wavesDesc: Parameters<typeof og.ui.wavePreview>[1] = {
    title: L("sc.td.waves"), current: 7, max: small ? 3 : 5,
    waves: [
      { key: "7", number: 7, name: "Wolfkin", image: hudArt("portraits/red-louse"), count: 18 },
      { key: "8", number: 8, name: "Shieldbearers", image: hudArt("portraits/brann"), count: 12 },
      { key: "9", number: 9, name: "Cultists", image: hudArt("portraits/cultist"), count: 20 },
      { key: "10", number: 10, name: "The Ash Tyrant", image: hudArt("portraits/vesper"), count: 1, boss: true },
      { key: "11", number: 11, name: "Wolfkin", image: hudArt("portraits/red-louse"), count: 30 },
      { key: "12", number: 12, name: "Shieldbearers", image: hudArt("portraits/brann"), count: 20 },
    ],
    anchor: portrait ? "topRight" : "right",
  };
  // In portrait the waves stand top end, or under the purse where the two would meet (large text): from both as laid last tick.
  const vp = og.ui.frameViewport();
  const purse = og.ui.bounds("PurseHud");
  const waves = og.ui.bounds("WavesHud");
  if (portrait && purse && waves && purse.x + purse.width + t.space.sm > waves.x) {
    og.ui.anchor("WavesHud", { anchor: "topRight", margin: m, offsetY: Math.round(purse.y + purse.height + t.space.sm - (vp.safeArea.top + m)) }, () => og.ui.wavePreview("Waves", { ...wavesDesc, anchor: null }));
  } else {
    og.ui.wavePreview("Waves", wavesDesc);
  }
  // On a phone the call stands bottom end over the towers' bar (a gap above its top as laid last tick, at any text size).
  const build = small ? og.ui.bounds("BuildHud") : null;
  const call = og.ui.bounds("Call");
  const meets = portrait || (build !== null && call !== null && build.x + build.width + t.space.sm > call.x);
  const callY = !small || !meets ? 0 : build ? Math.min(0, Math.round(build.y - t.space.sm - (vp.height - vp.safeArea.bottom - m))) : -Math.round(96 * k);
  og.ui.anchor("Call", { anchor: small ? "bottomRight" : "topRight", margin: m, offsetY: callY, direction: "row", gap: t.space.sm, cross: "center" }, () => {
    og.ui.hudTimer("Next", { seconds: Math.max(0, 18 - (tick % 1080) / 60), total: 18, mode: "countdown", label: L("sc.td.next"), warnAt: 5, anchor: null });
    og.ui.pushButton("Send", L("sc.td.send"), { variant: "primary", icon: "flag", width: { kind: "fit" } });
    if (!small) pauseButton();
  });
  const towers: AbilitySlotDesc[] = [
    { key: "archer", image: hudArt("rts/tower"), name: L("sc.td.archer"), hotkey: "1", cost: "70", active: true },
    { key: "wall", image: hudArt("rts/wall"), name: L("sc.td.wall"), hotkey: "2", cost: "40" },
    { key: "barracks", image: hudArt("rts/barracks"), name: L("sc.td.barracks"), hotkey: "3", cost: "120" },
    { key: "forge", image: hudArt("rts/forge"), name: L("sc.td.forge"), hotkey: "4", cost: "500", unaffordable: true },
  ];
  og.ui.abilityBar("Build", { slots: towers, size: Math.round((small ? 54 : 62) * k), anchor: "bottom" });
}

// ── rhythm ───────────────────────────────────────────────────────────────

/** The largest boxes the judgement and the combo were laid out in, for one look (the rhythm HUD's plates under them). */
const judgeBoxes: { look: string; boxes: Record<string, { w: number; h: number }> } = { look: "", boxes: {} };

const LANE_HUES: Paint[] = [rgb(0.95, 0.3, 0.45), rgb(0.25, 0.75, 0.95), rgb(0.25, 0.75, 0.95), rgb(0.95, 0.3, 0.45)];

/**
 * A rhythm game's play screen (osu!mania, DJMAX, Project Sekai): the
 * highway of four lanes with notes falling to the hit line, the lane keys
 * under it, the judgement and the combo over it, the score and the
 * accuracy top end, the song's progress and the life gauge top start.
 */
export function rhythmHud(world: World): void {
  const { t, vp, small, k, m } = frame();
  og.ui.setNavigation(false);
  void world;
  gameScene(world, "scenes/genre-rhythm", { x: 0.5, y: 0.5 });
  const tick = og.ui.tick();
  const portrait = vp.breakpoint === "phonePortrait";
  // On a phone on its side the highway narrows so the kit's chrome in the bottom start corner stays clear of it.
  const lane = Math.round((portrait ? 76 : small ? 56 : 84) * k);
  const width = lane * 4;
  const left = Math.round((vp.width - width) / 2);
  // The keys under the hit line keep a gap over the kit's chrome in the bottom start corner on a phone (both as laid last
  // tick: at large text the keys and the chrome grow into each other).
  // The score in the bottom end corner (a phone at large text, below) is kept clear the same way.
  const keys = og.ui.bounds("KeysHud");
  const lastHit = og.ui.memory("rhythm~hitY", 0);
  let clear = Infinity;
  for (const b of [small ? og.ui.bounds("Back") : null, rhythmScore.foot ? og.ui.bounds("Score") : null]) {
    if (b && keys && lastHit > 0 && b.y > vp.height / 2 && b.x + b.width > keys.x && b.x < keys.x + keys.width) clear = Math.min(clear, b.y - t.space.sm - (keys.y + keys.height - lastHit));
  }
  const hitY = Math.round(Math.min(vp.height - (portrait ? 200 : small ? 90 : 140) * k, clear));
  og.ui.remember("rhythm~hitY", hitY);
  // The highway: four lanes on a dark well, the notes falling at 0.6 of the height a second. The well is the game's
  // own night-blue in every pack (a light pack's backdrop would lay a pale slab down the lane), its rules a faint light.
  og.ui.anchor("Highway", { anchor: "topLeft", margin: 0, offsetX: left - vp.safeArea.left, offsetY: -vp.safeArea.top, width: px(width), height: px(hitY + 8) }, () => {
    og.ui.column("Well", { width: grow, height: grow }, () => {});
    og.ui.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: { color: { r: 0.03, g: 0.035, b: 0.07 }, alpha: 0.82 } }] } as never);
    og.ui.props({ position: "absolute", insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true } as never);
    for (let i = 1; i < 4; i++) {
      og.ui.column(`Rule${i}`, { width: px(1), height: grow }, () => {});
      og.ui.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: { color: { r: 1, g: 1, b: 1 }, alpha: 0.14 } }] } as never);
      og.ui.props({ position: "absolute", insets: { start: i * lane, top: 0, bottom: 0 }, inert: true } as never);
    }
    og.ui.column("HitLine", { width: grow, height: px(3) }, () => {});
    og.ui.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: { token: "highlight" } }] } as never);
    og.ui.props({ position: "absolute", insets: { start: 0, end: 0, top: hitY - 2 }, inert: true } as never);
    const speed = hitY * 0.6 / 60;
    for (let n = 0; n < 14; n++) {
      const l = (n * 7 + (n >> 1)) % 4;
      const due = n * 26 + 40;
      const y = Math.round(hitY - (due - (tick % 400)) * speed);
      if (y < -20 || y > hitY + 4) continue;
      og.ui.column(`Note${n}`, { width: px(lane - 12), height: px(14) }, () => {});
      og.ui.setDraw({ shape: { kind: "rect", radii: [6, 6, 6, 6] }, fills: [{ kind: "solid", color: LANE_HUES[l]!.color }] } as never);
      og.ui.props({ position: "absolute", insets: { start: l * lane + 6, top: y - 7 }, inert: true } as never);
    }
  });
  const pressed = ["d", "f", "j", "k"].filter((_, i) => (tick + i * 13) % 26 < 6);
  og.ui.laneKeys("Keys", { lanes: ["d", "f", "j", "k"].map((key, i) => ({ key, hotkey: key.toUpperCase(), x: left + lane * i + lane / 2, color: LANE_HUES[i]! })), y: hitY, width: lane - 12, pressed, hit: pressed });
  // The judgement and the combo stand over the highway and the stage: a plate of the style's HUD ground under each
  // holds it, so a pack whose tiers' inks are meant for light panels still reads over the dark well and the lights.
  // Each plate is tight to its text, so the notes stay in view around it.
  const hudGround = { token: "hud" } as const;
  const look = `${vp.width}x${vp.height}|${t.textScale ?? 1}|${t.palette["hud"] ?? ""}`;
  if (judgeBoxes.look !== look) Object.assign(judgeBoxes, { look, boxes: {} });
  // Each plate is centred on its widget's place (the static box the word and the count animate in) and as large as the
  // largest its text has been drawn.
  const plates: Array<[string, string, string[]]> = [["JudgePlate", "JudgeHud", ["JudgeHud/Judge/Word"]], ["ComboPlate", "ComboHud", ["ComboHud/Combo/Count", "ComboHud/Combo/Under"]]];
  let plateFloor = -Infinity;
  for (const [id, home, paths] of plates) {
    const box = (judgeBoxes.boxes[id] ??= { w: 0, h: 0 });
    const at = og.ui.bounds(home);
    let top = Infinity;
    let bottom = -Infinity;
    for (const path of paths) {
      const b = og.ui.bounds(path);
      if (!b || b.width <= 0) continue;
      box.w = Math.max(box.w, b.width);
      top = Math.min(top, b.y);
      bottom = Math.max(bottom, b.y + b.height);
    }
    if (bottom > top) box.h = Math.max(box.h, bottom - top);
    if (!at || box.w <= 0) continue;
    const w = box.w + 2 * t.space.sm;
    let h = Math.max(at.height, box.h) + 2 * t.space.xs;
    // A plate starts under the one above it where their margins would meet (large text brings the two together).
    let y = Math.round(at.y + at.height / 2 - h / 2);
    if (y < plateFloor) {
      h -= plateFloor - y;
      y = plateFloor;
    }
    plateFloor = y + Math.round(h);
    og.ui.anchor(id, { anchor: "topLeft", margin: 0, offsetX: Math.round(at.x + at.width / 2 - w / 2 - vp.safeArea.left), offsetY: y - vp.safeArea.top, width: px(w), height: px(Math.round(h)), background: hudGround, radius: t.radius["panel"] ?? 12, frame: "hud" }, () => {});
    og.ui.props({ inert: true, overlaps: [home] } as never);
  }
  const tiers = ["sc.rhythm.perfect", "sc.rhythm.great", "sc.rhythm.perfect", "sc.rhythm.good"];
  const beat = Math.floor(tick / 26);
  og.ui.judgement("Judge", { last: { text: L(tiers[beat % 4]!), tier: [0, 1, 0, 2][beat % 4]!, offsetMs: [0, -18, 0, 26][beat % 4]!, at: beat * 26 }, earlyLabel: L("sc.rhythm.early"), lateLabel: L("sc.rhythm.late"), anchor: "center", offsetY: -Math.round(40 * k) });
  og.ui.comboCounter("Combo", { count: 212 + beat, label: L("sc.combo"), breakLabel: L("sc.comboBreak"), milestones: [250, 500], anchor: "center" });
  const plate = { background: hudGround, radius: t.radius["panel"] ?? 12, frame: "hud" as const, padding: { left: t.space.sm, top: t.space.xs, right: t.space.sm, bottom: t.space.sm } };
  // Beside the song's plate where both fit the width (as laid last tick); where they would meet (a phone at large
  // text) the score takes the bottom end corner, clear of the judgement and the combo in the middle (the keys rise
  // over it, above). Held for the look.
  const songB = og.ui.bounds("Song");
  const scoreB = og.ui.bounds("Score");
  const scoreLook = `${vp.width}|${t.textScale ?? 1}|${t.style}`;
  if (rhythmScore.look !== scoreLook) rhythmScore = { look: scoreLook, foot: false };
  if (songB && scoreB && !rhythmScore.foot && songB.x + songB.width + m > scoreB.x && scoreB.y < songB.y + songB.height) rhythmScore.foot = true;
  og.ui.anchor("Score", { anchor: rhythmScore.foot ? "bottomRight" : "topRight", margin: m, direction: "column", cross: "end", gap: 2, ...plate }, () => {
    og.ui.counter("Points", 812_400 + beat * 1_250, { step: "xl2", font: "display" });
    og.ui.label("Accuracy", L("sc.rhythm.accuracy", { pct: "98.42" }), { step: "sm", font: "mono", color: { token: "inkMuted" }, nowrap: true });
  });
  og.ui.anchor("Song", { anchor: "topLeft", margin: m, direction: "column", gap: t.space.xs, ...plate }, () => {
    og.ui.label("Title", "Stormbreak", { step: "lg", font: "display", nowrap: true });
    og.ui.statBar("Progress", { value: 0.38 + (tick % 3600) / 9000, max: 1, kind: "primary", width: px((small ? 140 : 220) * k), thickness: 4, name: L("sc.rhythm.progress"), anchor: null });
    og.ui.statBar("Life", { value: 82, max: 100, kind: "health", width: px((small ? 140 : 220) * k), thickness: 8, name: L("sc.rhythm.life"), anchor: null });
  });
}

// ── fighting and party ───────────────────────────────────────────────────

/**
 * A fighting game's HUD (Street Fighter, Tekken): the two fighters' health
 * from the centre out with their names and portraits, the round's clock
 * and the rounds won between them, the super gauges at the foot's ends,
 * the combo on the attacker's side; on a touch screen the stick and the
 * four buttons.
 */
export function fightHud(world: World): void {
  const { t, small, touch, k, m } = frame();
  og.ui.setNavigation(false);
  void world;
  gameScene(world, "scenes/genre-fighting", { x: 0.5, y: 0.5 });
  const tick = og.ui.tick();
  const bar = px((small ? 150 : 460) * k);
  og.ui.anchor("Top", { anchor: "top", margin: m, direction: "row", gap: t.space.sm, cross: "start" }, () => {
    og.ui.column("P1", { gap: t.space.xs, cross: "start" }, () => {
      og.ui.statBar("Health", { value: 68 - ((tick / 60) % 6), max: 100, kind: "health", ghost: true, width: bar, thickness: Math.round((small ? 12 : 20) * k), name: "Kael", anchor: null });
      og.ui.row("Name", { gap: t.space.xs, cross: "center" }, () => {
        og.ui.label("Name", "Kael", { step: small ? "sm" : "lg", font: "display", nowrap: true });
        for (let i = 0; i < 2; i++) og.ui.icon(`Round${i}`, "star", { size: Math.round(14 * k), tint: { token: i < 1 ? "gold" : "inkMuted" } } as never);
      });
    });
    og.ui.column("Clock", { cross: "center", padding: { left: t.space.sm, top: 0, right: t.space.sm, bottom: 0 }, background: { token: "hud" }, radius: t.radius["panel"] ?? 8, frame: "hud" }, () => {
      og.ui.label("Seconds", String(Math.max(0, 62 - Math.floor(tick / 60))), { step: small ? "xl" : "xl3", font: "display", nowrap: true });
    });
    og.ui.column("P2", { gap: t.space.xs, cross: "end" }, () => {
      og.ui.statBar("Health", { value: 41, max: 100, kind: "health", ghost: true, reverse: true, width: bar, thickness: Math.round((small ? 12 : 20) * k), name: "Mira", anchor: null });
      og.ui.row("Name", { gap: t.space.xs, cross: "center" }, () => {
        for (let i = 0; i < 2; i++) og.ui.icon(`Round${i}`, "star", { size: Math.round(14 * k), tint: { token: "inkMuted" } } as never);
        og.ui.label("Name", "Mira", { step: small ? "sm" : "lg", font: "display", nowrap: true });
      });
    });
  });
  if (!small) og.ui.comboCounter("Combo", { count: 4 + Math.floor((tick % 240) / 40), label: L("sc.fight.hits"), breakLabel: L("sc.comboBreak"), milestones: [10], anchor: "left" });
  // The super gauges sit where the thumbs go: on a touch screen the buttons' own fill shows the super.
  if (!touch) {
    og.ui.anchor("Super1", { anchor: "bottomLeft", margin: m, direction: "column", gap: 2 }, () => {
      og.ui.label("Label", L("sc.fight.super"), { step: "sm", font: "display", color: { token: "inkMuted" }, nowrap: true });
      og.ui.statBar("Gauge", { value: 2.4, max: 3, kind: "primary", segment: 1, width: px(280 * k), thickness: 10, name: L("sc.fight.super"), anchor: null });
    });
    og.ui.anchor("Super2", { anchor: "bottomRight", margin: m, direction: "column", gap: 2, cross: "end" }, () => {
      og.ui.label("Label", L("sc.fight.super"), { step: "sm", font: "display", color: { token: "inkMuted" }, nowrap: true });
      og.ui.statBar("Gauge", { value: 1.1, max: 3, kind: "primary", segment: 1, reverse: true, width: px(280 * k), thickness: 10, name: L("sc.fight.super"), anchor: null });
    });
  }
  if (touch) thumbs([{ id: "Kick", label: L("sc.fight.kick"), icon: "bolt" }, { id: "Special", label: L("sc.fight.special"), icon: "fire" }, { id: "Punch", label: L("sc.fight.punch"), icon: "sword", primary: true }]);
}

// ── survival ─────────────────────────────────────────────────────────────

/**
 * A survival game's HUD (Valheim, The Forest, Subnautica): health, food,
 * water and stamina bottom start with the statuses over them, the compass
 * along the top, the day and its weather top end, the hotbar along the
 * foot, a resource's gather prompt in the world.
 */
export function survivalHud(world: World): void {
  const { t, vp, small, portrait, touch, k, m } = frame();
  og.ui.setNavigation(false);
  gameScene(world, "scenes/genre-survival", { x: 0.5, y: 0.55 });
  const tick = og.ui.tick();
  og.ui.interactionPrompt(world, "Gather", { target: { point: at(portrait ? 0.62 : 0.58, 0.55) }, action: "interact", verb: L("sc.survival.gather"), name: L("sc.survival.birch"), hold: (tick % 180) / 180 });
  if (!portrait) og.ui.compass("Compass", { heading: 128 + 6 * Math.sin(tick / 120), width: Math.min(Math.round(small ? 280 : 480 * k), vp.width - 360), pois: [{ key: "camp", bearing: 160, distance: 220, kind: "objective", label: L("sc.survival.camp") }, { key: "cave", bearing: 96, distance: 640, kind: "poi" }], units: "m", anchor: "top" });
  chip("Day", () => {
    og.ui.icon("Weather", "drop", { size: Math.round(18 * k), tint: { token: "info" } } as never);
    og.ui.label("Date", L("sc.survival.day", { n: 23 }), { step: "base", nowrap: true });
  }, { anchor: "topRight" });
  og.ui.anchor("Vitals", { anchor: touch ? "topLeft" : "bottomLeft", margin: m, direction: "column", gap: t.space.xs }, () => {
    if (portrait || !small) og.ui.statusGrid("Status", { effects: [{ key: "wet", icon: "drop", name: L("sc.survival.wet"), remaining: 40, duration: 60, debuff: true }, { key: "rested", icon: "heart", name: L("sc.survival.rested"), remaining: 410, duration: 600 }], columns: 4, size: Math.round(28 * k), timers: true, anchor: null });
    const w = px((small ? 150 : 210) * k);
    og.ui.statBar("Health", { value: 74, max: 100, kind: "health", icon: "heart", width: w, thickness: 10, name: L("sc.health"), anchor: null });
    og.ui.statBar("Food", { value: 38, max: 100, color: rgb(0.92, 0.6, 0.2), icon: "apple", low: 40, width: w, thickness: 10, name: L("sc.survival.food"), anchor: null });
    og.ui.statBar("Water", { value: 61, max: 100, color: rgb(0.3, 0.62, 0.95), icon: "drop", width: w, thickness: 10, name: L("sc.survival.water"), anchor: null });
  });
  og.ui.radialMeter("Stamina", { value: 0.35 + 0.6 * ((tick % 300) / 300), size: Math.round(40 * k), hideWhenFull: false, name: L("sc.survival.stamina"), anchor: "center" });
  if (touch) {
    thumbs([{ id: "Use", label: L("sc.survival.gather"), icon: "bag" }, { id: "Swing", label: L("sc.survival.swing"), icon: "sword", primary: true }]);
    return;
  }
  og.ui.abilityBar("Hotbar", {
    slots: [
      { key: "axe", image: hudArt("items/iron-sword"), name: L("sc.survival.axe"), hotkey: "1", active: true },
      { key: "bow", image: hudArt("items/hunters-bow"), name: L("sc.survival.bow"), hotkey: "2", charges: 24 },
      { key: "torch", icon: "fire", name: L("sc.survival.torch"), hotkey: "3" },
      { key: "berries", image: hudArt("boons/apple"), name: L("sc.survival.berries"), hotkey: "4", charges: 9 },
      { key: "draught", image: hudArt("items/healing-draught"), name: L("sc.rpg.potion"), hotkey: "5", charges: 2 },
      { key: "shield", image: hudArt("items/oak-shield"), name: L("sc.survival.shield"), hotkey: "6" },
    ],
    size: Math.round(50 * k), gap: 4, anchor: "bottom",
  });
}

// ── horror ───────────────────────────────────────────────────────────────

/**
 * A horror game's HUD (Amnesia, Outlast, Resident Evil): almost nothing,
 * on purpose. The light's battery bottom start, a hide prompt at the
 * cupboard, the objective whispered top start for a few seconds, the
 * stamina only while it is spent, the vignette as health falls.
 */
export function horrorHud(world: World): void {
  const { t, small, touch, k, m } = frame();
  og.ui.setNavigation(false);
  gameScene(world, "scenes/genre-horror", { x: 0.5, y: 0.5 });
  const tick = og.ui.tick();
  og.ui.lowHealthVignette("Vignette", { health: 0.3, threshold: 0.4, hitAt: null });
  og.ui.interactionPrompt(world, "Hide", { target: { point: at(0.3, 0.52) }, action: "interact", verb: L("sc.horror.hide"), name: L("sc.horror.wardrobe") });
  // On the HUD's plate, so a pack whose ink is meant for light panels still reads over the dark world.
  og.ui.anchor("Objective", { anchor: "topLeft", margin: m, direction: "column", gap: 2, padding: { left: t.space.sm, top: t.space.xs, right: t.space.sm, bottom: t.space.xs }, background: { token: "hud" }, radius: t.radius["panel"] ?? 8, frame: "hud" }, () => {
    og.ui.label("Caption", L("sc.horror.objective"), { step: "sm", font: "display", color: { token: "inkMuted" }, nowrap: true });
    og.ui.label("Text", L("sc.horror.find"), { step: small ? "base" : "lg", nowrap: true });
  });
  og.ui.anchor("Light", { anchor: "bottomLeft", margin: m, direction: "row", gap: t.space.xs, cross: "center" }, () => {
    og.ui.icon("Icon", "bolt", { size: Math.round(18 * k), tint: { token: "warn" } } as never);
    og.ui.statBar("Battery", { value: 2, max: 5, kind: "primary", segment: 1, low: 1, width: px(90 * k), thickness: 8, name: L("sc.horror.battery"), anchor: null });
  });
  og.ui.radialMeter("Stamina", { value: 0.2 + 0.7 * ((tick % 420) / 420), size: Math.round(34 * k), hideWhenFull: true, exhausted: (tick % 420) < 60, name: L("sc.survival.stamina"), anchor: "center" });
  if (touch) thumbs([{ id: "Hide", label: L("sc.horror.hide"), icon: "lock" }, { id: "Run", label: L("sc.horror.run"), icon: "bolt", primary: true }]);
}

// ── open world ───────────────────────────────────────────────────────────

/**
 * An open world's HUD (Breath of the Wild, Horizon, Skyrim): the compass
 * with the quest's marker along the top, the tracked quest on the end
 * edge, the minimap bottom start with health and stamina above it, the
 * time and weather top end, a waypoint and an NPC's talk prompt in the
 * world.
 */
export function openHud(world: World): void {
  const { t, vp, small, portrait, touch, k, m } = frame();
  og.ui.setNavigation(false);
  gameScene(world, "scenes/genre-openworld", { x: 0.5, y: 0.55 });
  const tick = og.ui.tick();
  og.ui.waypoint("Shrine", { target: { point: at(0.7, 0.36) }, kind: "objective", label: L("sc.open.shrine"), units: "m" });
  og.ui.interactionPrompt(world, "Talk", { target: { point: at(portrait ? 0.4 : 0.36, 0.56) }, action: "interact", verb: L("sc.open.talk"), name: "Odile" });
  const mapSize = Math.round((small ? 110 : 160) * k);
  if (!portrait) og.ui.compass("Compass", { heading: 62 + 8 * Math.sin(tick / 150), width: Math.min(Math.round(small ? 280 : 520 * k), vp.width - 2 * Math.max(180, mapSize + 2 * m)), pois: [{ key: "shrine", bearing: 80, distance: 418, kind: "objective", label: L("sc.open.shrine") }, { key: "town", bearing: 20, distance: 1_200, kind: "poi" }, { key: "foe", bearing: 110, distance: 60, kind: "enemy" }], units: "m", anchor: "top" });
  chip("Time", () => {
    og.ui.icon("Sun", "star", { size: Math.round(18 * k), tint: { token: "gold" } } as never);
    og.ui.label("Clock", "16:40", { step: "base", font: "mono", nowrap: true });
  }, { anchor: "topRight" });
  if (!touch) og.ui.objectiveTracker("Quest", { quests: [{ key: "bells", title: L("sc.open.quest"), icon: "flag", focused: true, distance: 418, steps: [{ key: "shrine", text: L("sc.open.step"), done: false }, { key: "map", text: L("sc.open.tower"), done: true }] }], width: Math.round(260 * k), units: "m", anchor: "right" });
  og.ui.anchor("Vitals", { anchor: touch ? "topLeft" : "bottomLeft", margin: m, direction: "column", gap: t.space.xs }, () => {
    og.ui.statBar("Health", { value: 9, max: 12, kind: "health", segment: 1, icon: "heart", width: px((small ? 150 : 200) * k), thickness: 10, name: L("sc.health"), anchor: null });
    if (!portrait) og.ui.minimapView("Map", { texture: hudArt("maps/world"), size: mapSize, shape: "circle", north: "", markers: [{ key: "me", kind: "player", x: 0.48, y: 0.52, heading: 62 }, { key: "shrine", kind: "objective", x: 0.62, y: 0.4 }], anchor: null });
  });
  og.ui.radialMeter("Stamina", { value: 0.4 + 0.5 * ((tick % 360) / 360), wheels: 1, size: Math.round(40 * k), hideWhenFull: false, name: L("sc.survival.stamina"), anchor: "center" });
  if (touch) thumbs([{ id: "Glide", label: L("sc.open.glide"), icon: "leaf" }, { id: "Attack", label: L("sc.action.attack"), icon: "sword", primary: true }]);
}
