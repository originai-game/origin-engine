// The showcases' HUDs (showcases.ts): each a game's play screen over its
// gameplay scene (assets/textures/art/scenes/genre-*.jpg), built from the
// HUD components (packages/engine/src/ui/hud) and the world UI
// (packages/engine/src/ui/world) with the game's side done as a game would
// (a timer running, hits landing, a lap's split). Every word is keyed in
// assets/strings (sc.*); names stay as they are. `sc:hud:<key>:<value>`
// sets a HUD's preview state for the playtests.

import { og } from "@origingame/engine";
import type { AbilitySlotDesc, Anchor, KillfeedEntry, Paint, MapMarkerItem, Size, StandingsEntry, Vector3, World } from "@origingame/engine";
import { L } from "./patterns.ts";

const grow: Size = { kind: "grow" };
const px = (n: number): Size => ({ kind: "fixed", px: Math.max(0, Math.round(n)) });

// ── art ─────────────────────────────────────────────────────────────────

const ART_FILES = [
  "scenes/genre-action", "scenes/genre-shooter", "scenes/genre-management", "scenes/genre-racing", "scenes/genre-sports", "scenes/title",
  "scenes/genre-moba", "scenes/genre-rpg", "scenes/genre-mmo", "scenes/genre-roguelike", "scenes/genre-strategy", "scenes/genre-rhythm",
  "scenes/genre-fighting", "scenes/genre-survival", "scenes/genre-horror", "scenes/genre-openworld",
  "boons/gold", "boons/crystal", "boons/thunder-strike", "boons/flame-blade", "boons/frost", "boons/rune-shield", "boons/titan-heart",
  "loadout/raven", "loadout/sidearm", "loadout/frag", "loadout/stim", "loadout/drone", "loadout/shield",
  "city/residential", "city/commercial", "city/industrial", "city/parks", "city/power", "city/water", "city/transit", "city/police",
  "portraits/aria", "portraits/kael", "portraits/mira", "maps/world",
  "portraits/maren", "portraits/cultist", "portraits/brann", "portraits/wren", "portraits/torvin", "portraits/vesper", "portraits/red-louse",
  "items/healing-draught", "items/mana-vial", "items/iron-sword", "items/iron-key", "items/hunters-bow", "items/oak-shield",
  "boons/curse", "boons/star", "boons/apple", "boons/bomb", "boons/poison", "boons/broken-chain",
  "rts/tower", "rts/wall", "rts/barracks", "rts/forge",
] as const;
export type HudArt = (typeof ART_FILES)[number];
const CUT_OUTS: ReadonlySet<string> = new Set(["loadout/raven"]);
const ART: Partial<Record<HudArt, number>> = {};

/** Load the HUDs' art (the game's start). A file that is missing leaves its slot on the style's placeholder. */
export function loadHudArt(world: World): void {
  for (const f of ART_FILES) {
    const r = og.load.texture(world, `textures/art/${f}.${CUT_OUTS.has(f) ? "png" : "jpg"}`);
    if (r.ok) ART[f] = r.value as unknown as number;
  }
}

/** A texture of the HUDs' art, or null. */
export function hudArt(name: HudArt): never | null {
  const h = ART[name];
  return h ? (h as never) : null;
}

/**
 * The play screens' actions (og.ui.defineAction): their keys and the pad's
 * buttons, whose glyphs the hints, prompts and ability slots draw for the
 * device used last. Clear of the UI's own (Enter, Space as confirm,
 * Escape, Q, E, X, R).
 */
export function defineHudActions(world: World): void {
  const act = (name: string, keys: string[], buttons: string[], label: string) => og.ui.defineAction(world, { name, keys, buttons, trigger: "press", holdMs: 0, toggle: false, label });
  act("jump", ["KeyK"], ["a"], "Jump");
  act("attack", ["KeyJ"], ["x"], "Attack");
  act("interact", ["KeyF"], ["y"], "Interact");
  act("pause", ["KeyP"], ["start"], "Pause");
  act("ability1", ["KeyC"], ["leftBumper"], "Ability 1");
  act("ability2", ["KeyG"], ["rightBumper"], "Ability 2");
  act("ability3", ["KeyH"], ["down"], "Ability 3");
  act("ultimate", ["KeyZ"], ["rightStick"], "Ultimate");
}

// ── the frame ───────────────────────────────────────────────────────────

export function frame() {
  const t = og.ui.frameTheme();
  const vp = og.ui.frameViewport();
  const small = vp.breakpoint === "phonePortrait" || vp.breakpoint === "phoneLandscape";
  const portrait = vp.breakpoint === "phonePortrait";
  const touch = small || vp.breakpoint === "tablet" || vp.coarsePointer;
  const big = !small && vp.height >= 1000;
  const m = small ? t.space.sm : t.space.lg;
  return { t, vp, small, portrait, touch, big, k: big ? 1.35 : 1, m };
}

/**
 * The game behind a HUD: its gameplay scene over the whole surface (under
 * the notch too), sharp: it is the game, not a menu's backdrop. Without
 * the art, the style's backdrop.
 */
export function gameScene(name: HudArt, focal = { x: 0.5, y: 0.5 }): void {
  const vp = og.ui.frameViewport();
  const sa = vp.safeArea;
  const tex = hudArt(name);
  og.ui.anchor("Game", { anchor: "topLeft", margin: 0, offsetX: -sa.left, offsetY: -sa.top, width: px(vp.width), height: px(vp.height) }, () => {
    og.ui.column("Picture", { width: grow, height: grow }, () => {});
    og.ui.setDraw({ shape: { kind: "rect" }, fills: tex ? [{ kind: "image", texture: tex, fit: "cover", focal }] : [{ kind: "solid", color: { token: "backdrop" } }] } as never);
    og.ui.props({ position: "absolute", insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true } as never);
  });
}

/**
 * A point on the play surface as the world sees it (the kit's camera is
 * orthographic, 360 units tall, at the origin): x and y as fractions of the
 * surface, from its top left.
 */
export function at(fx: number, fy: number): Vector3 {
  const vp = og.ui.frameViewport();
  const unit = 360 / vp.height;
  return { x: (fx - 0.5) * vp.width * unit, y: (0.5 - fy) * 360, z: 0 };
}

/** The pause button every play screen has in its top end corner: true when pressed (or Escape, the pad's Start). */
export function pauseButton(id = "Pause"): boolean {
  return og.ui.iconButton(id, "pause", { label: L("sc.pause"), variant: "secondary", round: true }).clicked || og.ui.action("pause");
}

// ── preview state ───────────────────────────────────────────────────────

const st0 = () => ({ hp: 72, coins: 86, score: 48210, combo: 14, hitAt: -1000, crit: false, ammo: 18, reloading: false, kills: 7, zone: 95, speed: 1 });
let st = st0();

export function resetHuds(): void {
  st = st0();
}

/** `sc:hud:<key>:<value>`: a HUD's preview state (hp, coins, combo, ammo, reload, hit, speed). */
export function hudCommand(area: string, key: string, value: string): boolean {
  if (area !== "hud") return false;
  const n = Number(value);
  if (key === "hp") st.hp = n;
  else if (key === "coins") st.coins = n;
  else if (key === "combo") st.combo = n;
  else if (key === "ammo") st.ammo = n;
  else if (key === "reload") st.reloading = value !== "off";
  else if (key === "hit") {
    st.hitAt = og.ui.tick();
    st.crit = value === "crit";
  } else if (key === "speed") st.speed = n;
  else return false;
  return true;
}

// ── action and platformer ───────────────────────────────────────────────

/**
 * A platformer's and character action game's HUD (Hollow Knight, Ori,
 * Devil May Cry): health in segments with its ghost and the energy under
 * it top start, coins and the stage's countdown top end by the pause,
 * the combo on the end edge, a prompt at the door and the hits' numbers
 * where they land; on a touch screen the d-pad and the Jump and Attack
 * buttons in the bottom corners, else the keys' hints along the foot.
 * True when the pause is asked for.
 */
export function actionHud(world: World, under = false): boolean {
  const { t, small, portrait, touch: hasTouch, k, m } = frame();
  // Played with the game's actions (and touches), not a moving focus.
  og.ui.setNavigation(false);
  // Under a menu (the pause, a tutorial's callout) the HUD shows, without its controls.
  const touch = hasTouch && !under;
  gameScene("scenes/genre-action", { x: 0.45, y: 0.55 });
  const tick = og.ui.tick();
  // A hit lands every two seconds on the foe ahead: its number thrown up from it, the combo climbing.
  const foe = at(portrait ? 0.68 : 0.62, portrait ? 0.56 : 0.6);
  if (tick % 120 === 30) {
    const crit = (tick / 120) % 3 === 2;
    og.ui.damageNumbers("Hits", { hits: [{ position: foe, amount: crit ? 248 : 96 + ((tick / 120) % 4) * 7, kind: crit ? "critical" : "normal", seed: tick }] });
  }
  if (st.hitAt === tick) og.ui.damageNumbers("Hits", { hits: [{ position: foe, amount: st.crit ? 312 : 120, kind: st.crit ? "critical" : "normal", seed: tick }] });
  if (!under) og.ui.interactionPrompt(world, "Door", { target: { point: at(portrait ? 0.3 : 0.24, portrait ? 0.5 : 0.52) }, action: "interact", verb: L("sc.action.open"), name: L("sc.action.door") });

  og.ui.anchor("Player", { anchor: "topLeft", margin: m, direction: "column", gap: t.space.xs }, () => {
    og.ui.statBar("Health", { value: st.hp, max: 100, kind: "health", segment: 20, ghost: true, icon: "heart", width: px((small ? 180 : 260) * k), thickness: Math.round(16 * k), name: L("sc.health"), anchor: null });
    og.ui.statBar("Energy", { value: 60, max: 100, kind: "mana", icon: "bolt", width: px((small ? 130 : 190) * k), thickness: Math.round(8 * k), name: L("sc.energy"), anchor: null });
  });
  let paused = false;
  og.ui.anchor("Run", { anchor: "topRight", margin: m, direction: "column", cross: "end", gap: t.space.sm }, () => {
    og.ui.row("Top", { gap: t.space.sm, cross: "center" }, () => {
      og.ui.row("Coins", { gap: t.space.xs, cross: "center", padding: { left: t.space.sm, top: t.space.xs, right: t.space.md, bottom: t.space.xs }, background: { token: "hud" }, radius: 999, frame: "hud" }, () => {
        og.ui.icon("Icon", "coin", { size: Math.round(22 * k), tint: { token: "gold" } } as never);
        og.ui.counter("Amount", st.coins, { step: "xl", font: "rounded" });
      });
      if (!under) paused = pauseButton();
    });
    og.ui.hudTimer("Timer", { seconds: Math.max(0, 300 - tick / 60), total: 300, mode: "countdown", label: L("sc.time"), warnAt: 30, anchor: null });
  });
  og.ui.comboCounter("Combo", { count: st.combo + Math.floor(tick / 120), multiplier: 2, decay: 1 - ((tick % 120) / 120), label: L("sc.combo"), breakLabel: L("sc.comboBreak"), milestones: [25, 50, 100], anchor: "right" });

  if (touch) {
    // The thumbs' corners: the d-pad at the start, Jump (primary) and Attack at the end.
    og.ui.anchor("Move", { anchor: "bottomLeft", margin: m }, () => og.ui.dpad("Dpad", { customize: { key: "action.dpad" } } as never));
    og.ui.anchor("Buttons", { anchor: "bottomRight", margin: m, gap: t.space.md, cross: "end" }, () => {
      if (og.ui.actionButton("Attack", L("sc.action.attack"), { icon: "sword", size: 64 }).clicked) st.combo += 1;
      og.ui.actionButton("Jump", L("sc.action.jump"), { primary: true, icon: "play", size: 84 });
    });
  } else if (!under) {
    og.ui.actionHints("Hints", { hints: [{ action: "jump", hint: L("sc.action.jump") }, { action: "attack", hint: L("sc.action.attack") }, { action: "interact", hint: L("sc.action.open") }, { action: "pause", hint: L("sc.pause") }] });
  }
  return paused;
}

// ── shooter and battle royale ───────────────────────────────────────────

const FEED: KillfeedEntry[] = [
  { key: "k1", killer: "Vex", killerTeam: 0, victim: "Orrin", victimTeam: 1, weaponText: "Raven", headshot: true },
  { key: "k2", killer: "Sable", killerTeam: 1, victim: "Juno", victimTeam: 0, weaponText: "Frag" },
  { key: "k3", killer: "You", killerTeam: 0, victim: "Brask", victimTeam: 1, weaponText: "Raven", local: true, assist: "Vex" },
];

/**
 * A shooter's and battle royale's HUD (Apex Legends, Valorant, Call of
 * Duty): the compass along the top, the minimap and the zone's timer top
 * start with the killfeed under them, the crosshair with its hit marker,
 * health with shield cells and the squad's plates bottom start, the
 * abilities bottom centre, the ammunition bottom end, a ping in the world,
 * the hits' directions round the centre and the low-health vignette.
 */
export function shooterHud(world: World): void {
  const { t, small, portrait, touch, k, m } = frame();
  og.ui.setNavigation(false);
  void world;
  gameScene("scenes/genre-shooter", { x: 0.5, y: 0.5 });
  const tick = og.ui.tick();
  og.ui.lowHealthVignette("Vignette", { health: st.hp / 100, threshold: 0.35, hitAt: st.hitAt >= 0 ? st.hitAt : null });
  const hits = [{ angle: 0.9, at: tick - (tick % 150), strength: 0.8 }, { angle: -2.2, at: tick - ((tick + 60) % 150), strength: 0.45 }];
  og.ui.damageDirection("Hits", { hits, durationMs: 1400 });
  og.ui.waypoint("Ping", { target: { point: at(0.64, 0.42) }, kind: "ping", label: L("sc.shooter.enemy"), by: "Vex", at: tick - (tick % 300), durationMs: 4000, units: "m" });
  og.ui.waypoint("Objective", { target: { point: at(0.38, 0.47) }, kind: "objective", label: L("sc.shooter.supply") });
  og.ui.crosshair("Crosshair", { style: "cross", spread: 0.2 + 0.15 * Math.abs(Math.sin(tick / 20)), dot: true, hitAt: tick - (tick % 90) + 10, hit: (tick / 90) % 4 === 3 ? "headshot" : "hit", onTarget: (tick % 90) < 30 });
  if (!portrait) og.ui.compass("Compass", { heading: 214 + 10 * Math.sin(tick / 90), width: Math.round(small ? 300 : 520 * k), pois: [{ key: "ping", bearing: 236, distance: 84, kind: "ping" }, { key: "drop", bearing: 190, distance: 312, kind: "objective", label: L("sc.shooter.supply") }, { key: "squad", bearing: 260, distance: 22, kind: "ally" }], units: "m", anchor: "top" });

  const markers: MapMarkerItem[] = [
    { key: "me", kind: "player", x: 0.5, y: 0.5, heading: 214 },
    { key: "vex", kind: "ally", x: 0.46, y: 0.42, heading: 180, label: "Vex" },
    { key: "juno", kind: "ally", x: 0.55, y: 0.56, heading: 250, label: "Juno" },
    { key: "ping", kind: "ping", x: 0.62, y: 0.4 },
    { key: "enemy", kind: "enemy", x: 0.7, y: 0.3 },
    { key: "drop", kind: "objective", x: 0.36, y: 0.66 },
  ];
  og.ui.minimapView("Map", { texture: hudArt("maps/world"), zoom: 2.2, size: Math.round((small ? 116 : 168) * k), shape: "circle", markers, zone: L("sc.shooter.zone"), north: "", anchor: "topLeft" });
  og.ui.hudTimer("Zone", { seconds: Math.max(0, st.zone - tick / 60), total: 120, mode: "countdown", label: L("sc.shooter.ring"), warnAt: 20, anchor: "topLeft" });
  if (!portrait) og.ui.killfeed("Feed", { entries: FEED.map((e, i) => ({ ...e, killer: e.local ? L("sc.you") : e.killer ?? "", at: tick - 60 * (i + 1) })), max: 4, durationMs: 60_000, anchor: "topRight" });
  og.ui.anchor("Squad", { anchor: "bottomLeft", margin: m, direction: "column", gap: t.space.xs }, () => {
    og.ui.statBar("Health", { value: st.hp, max: 100, kind: "health", shield: 75, shieldMax: 100, shieldCell: 25, shieldTier: "epic", ghost: true, width: px((small ? 170 : 280) * k), thickness: Math.round(14 * k), name: L("sc.health"), label: L("sc.you"), anchor: null });
    if (!small) {
      for (const [name, hp] of [["Vex", 88], ["Juno", 31]] as const) og.ui.statBar(name, { value: hp, max: 100, kind: "health", ghost: true, width: px(180 * k), thickness: 6, label: name, name, anchor: null });
    }
  });
  const slots: AbilitySlotDesc[] = [
    { key: "tactical", image: hudArt("loadout/drone"), name: L("sc.shooter.drone"), action: "ability1", cooldown: Math.max(0, 8 - (tick % 600) / 60), cooldownMax: 8 },
    { key: "grenade", image: hudArt("loadout/frag"), name: L("sc.shooter.frag"), action: "ability2", charges: 2, maxCharges: 3 },
    { key: "heal", image: hudArt("loadout/stim"), name: L("sc.shooter.stim"), action: "ability3", charges: 4, maxCharges: 4 },
    { key: "ultimate", image: hudArt("loadout/shield"), name: L("sc.shooter.ultimate"), action: "ultimate", ultimate: Math.min(1, 0.4 + (tick % 900) / 1500) },
  ];
  og.ui.abilityBar("Abilities", { slots: touch ? slots.slice(0, 3) : slots, size: Math.round((portrait ? 44 : small ? 48 : 56) * k), anchor: "bottom" });
  og.ui.ammoCounter("Ammo", { magazine: st.ammo, magazineMax: 30, reserve: 120, weapon: "Raven", fireMode: L("sc.shooter.auto"), reloading: st.reloading ? ((tick % 90) / 90) : null, reloadLabel: L("sc.shooter.reloading"), reloadHint: L("sc.shooter.reload"), anchor: "bottomRight" });
  if (touch) {
    og.ui.anchor("Fire", { anchor: "right", margin: m }, () => og.ui.actionButton("Fire", L("sc.shooter.fire"), { primary: true, icon: "bolt", size: 76 }));
  }
}

// ── management: city, idle, life ────────────────────────────────────────

const BUILD: ReadonlyArray<[string, HudArt]> = [
  ["residential", "city/residential"], ["commercial", "city/commercial"], ["industrial", "city/industrial"], ["parks", "city/parks"],
  ["power", "city/power"], ["water", "city/water"], ["transit", "city/transit"], ["police", "city/police"],
];

/**
 * A management game's HUD (Cities: Skylines, Stardew Valley, Anno): the
 * resources along the top with their rates, the date and the game's speed
 * top end, the goals on the end edge, the build palette along the foot
 * with the tool chosen, and the hovered plot's card.
 */
export function manageHud(world: World): void {
  const { t, small, portrait, k, m } = frame();
  og.ui.setNavigation(false);
  void world;
  gameScene("scenes/genre-management", { x: 0.5, y: 0.5 });
  const tick = og.ui.tick();
  og.ui.resourceBar("Resources", {
    items: [
      { key: "gold", icon: "coin", name: L("sc.manage.gold"), amount: 18_420 + Math.floor(tick / 6) * 3, rate: 184 },
      { key: "wood", icon: "leaf", name: L("sc.manage.wood"), amount: 640, rate: 12, capacity: 800 },
      { key: "food", icon: "apple", name: L("sc.manage.food"), amount: 212, rate: -4 },
      { key: "people", icon: "person", name: L("sc.manage.people"), amount: 1_204, capacity: 1_250 },
    ].slice(0, portrait ? 3 : 4),
    rates: true,
    anchor: "topLeft",
  });
  og.ui.anchor("Clock", { anchor: portrait ? "topLeft" : "topRight", margin: m, offsetY: portrait ? Math.round(60 * k) : 0, direction: "row", gap: t.space.sm, cross: "center", padding: { left: t.space.md, top: t.space.xs, right: t.space.xs, bottom: t.space.xs }, background: { token: "hud" }, radius: 999, frame: "hud" }, () => {
    og.ui.label("Date", L("sc.manage.date"), { step: "base", nowrap: true });
    const speed = og.ui.segmented("Speed", { items: [{ icon: "pause", label: L("sc.pause") }, { icon: "play", label: L("sc.manage.play") }, { icon: "bolt", label: L("sc.manage.fast") }], selected: st.speed, size: "small" });
    if (speed.selected !== st.speed) st.speed = speed.selected;
  });
  if (!small) {
    og.ui.objectiveTracker("Goals", {
      quests: [
        { key: "grow", title: L("sc.manage.goal.grow"), icon: "person", focused: true, steps: [{ key: "people", text: L("sc.manage.goal.people"), count: 1_204, target: 1_500 }, { key: "park", text: L("sc.manage.goal.park"), done: true }] },
        { key: "trade", title: L("sc.manage.goal.trade"), icon: "coin", steps: [{ key: "market", text: L("sc.manage.goal.market"), progress: 0.4 }] },
      ],
      width: Math.round(280 * k),
      anchor: "right",
    });
  }
  // The plot under the cursor: what the chosen tool would build there, its cost and upkeep.
  og.ui.anchor("Plot", { anchor: small ? "top" : "center", margin: m, offsetX: small ? 0 : Math.round(170 * k), offsetY: small ? Math.round(120 * k) : Math.round(-60 * k), direction: "column", gap: 2, padding: { left: t.space.md, top: t.space.sm, right: t.space.md, bottom: t.space.sm }, background: { token: "hud" }, radius: t.radius["panel"] ?? 12, frame: "hud", width: px(220 * k) }, () => {
    og.ui.label("Name", L("sc.manage.plot"), { step: "lg", font: "display", nowrap: true });
    og.ui.label("Cost", L("sc.manage.cost", { cost: og.ui.formatNumber(1200, {}), upkeep: 18 }), { step: "sm", color: { token: "inkMuted" }, width: grow });
  });
  const slots: AbilitySlotDesc[] = BUILD.map(([key, art], i) => ({ key, image: hudArt(art), name: L(`ledger.line.${key}`), hotkey: String(i + 1), active: i === 0, unaffordable: key === "power" }));
  og.ui.abilityBar("Build", { slots: portrait ? slots.slice(0, 4) : slots, size: Math.round((small ? 52 : 60) * k), anchor: "bottom" });
}

// ── racing and sports ───────────────────────────────────────────────────

const RACERS: ReadonlyArray<[string, number]> = [["Nyx", 0], ["Ferro", 1.204], ["You", 2.861], ["Lumen", 3.402], ["Quill", 5.119], ["Sable", 7.88], ["Orrin", 9.02], ["Brask", 12.4]];
const TEAM_HUES: Paint[] = [
  { color: { r: 0.86, g: 0.16, b: 0.18 } }, { color: { r: 0.12, g: 0.42, b: 0.86 } }, { color: { r: 0.98, g: 0.72, b: 0.12 } }, { color: { r: 0.18, g: 0.7, b: 0.42 } },
  { color: { r: 0.6, g: 0.3, b: 0.86 } }, { color: { r: 0.96, g: 0.46, b: 0.14 } }, { color: { r: 0.5, g: 0.52, b: 0.58 } }, { color: { r: 0.1, g: 0.72, b: 0.8 } },
];

/**
 * A racing game's HUD (Forza Horizon, Gran Turismo, F1): the place and
 * lap with the running times and the split top start, the standings tower
 * on the start edge, the minimap top end, the speedometer bottom end; on a
 * touch screen the steering and the pedals in the bottom corners.
 */
export function raceHud(world: World): void {
  const { small, portrait, touch, k, m, t } = frame();
  og.ui.setNavigation(false);
  void world;
  gameScene("scenes/genre-racing", { x: 0.5, y: 0.6 });
  const tick = og.ui.tick();
  const lapMs = 41_200 + tick * (1000 / 60);
  og.ui.lapTimer("Lap", {
    position: 3, positionTotal: 8, lap: 2, lapTotal: 3, currentMs: lapMs % 90_000, bestMs: 81_904, lastMs: 82_377, deltaMs: -312 + ((tick / 60) % 5) * 90,
    positionLabel: L("sc.race.pos"), lapLabel: L("sc.race.lap"), finalLabel: L("sc.race.final"), currentLabel: L("sc.race.current"), bestLabel: L("sc.race.best"), lastLabel: L("sc.race.last"), splitLabel: L("sc.race.split"),
    anchor: "topLeft",
  });
  if (!portrait) {
    const entries: StandingsEntry[] = RACERS.map(([name, gap], i) => ({ key: name, name: name === "You" ? L("sc.you") : name, color: TEAM_HUES[i], gap, player: name === "You", fastest: name === "Ferro", change: name === "Lumen" ? -1 : 0 }));
    og.ui.standingsTower("Standings", { entries, rows: small ? 5 : 8, leaderLabel: L("sc.race.leader"), pitLabel: L("sc.race.pit"), outLabel: L("sc.race.out"), fastestLabel: L("sc.race.fastest"), anchor: "left" });
  }
  og.ui.minimapView("Track", {
    size: Math.round((small ? 120 : 176) * k), shape: "square",
    track: [{ x: 0.2, y: 0.7 }, { x: 0.3, y: 0.25 }, { x: 0.6, y: 0.18 }, { x: 0.82, y: 0.38 }, { x: 0.74, y: 0.74 }, { x: 0.45, y: 0.84 }],
    markers: [{ key: "me", kind: "player", x: 0.62, y: 0.19, heading: 80 }, { key: "a", kind: "enemy", x: 0.72, y: 0.27 }, { key: "b", kind: "enemy", x: 0.53, y: 0.17 }],
    north: "",
    anchor: "topRight",
  });
  const speed = 212 + 24 * Math.sin(tick / 40);
  og.ui.speedometer("Speed", { speed, maxSpeed: 320, unit: L("sc.race.kmh"), rpm: 0.55 + 0.4 * ((tick % 90) / 90), redline: 0.88, gear: String(5 + Math.floor(((tick % 90) / 90) * 1.5)), boost: 0.62, size: Math.round((small ? 150 : 210) * k), anchor: "bottomRight" });
  if (touch) {
    og.ui.anchor("Steer", { anchor: "bottomLeft", margin: m, gap: t.space.md }, () => {
      og.ui.actionButton("Left", L("sc.race.left"), { icon: "back", size: 64 });
      og.ui.actionButton("Right", L("sc.race.right"), { icon: "play", size: 64 });
    });
    og.ui.anchor("Pedals", { anchor: "bottomRight", margin: m, offsetY: -Math.round((small ? 170 : 230) * k), gap: t.space.md }, () => {
      og.ui.actionButton("Brake", L("sc.race.brake"), { size: 56 });
      og.ui.actionButton("Boost", L("sc.race.boost"), { primary: true, icon: "bolt", size: 64 });
    });
  }
}

const HOME = { abbr: "RVR", name: "Riverside", score: 2, color: { color: { r: 0.12, g: 0.36, b: 0.82 } } };
const AWAY = { abbr: "HLD", name: "Highland", score: 1, color: { color: { r: 0.86, g: 0.2, b: 0.2 } } };

/**
 * A football game's broadcast HUD (EA FC, eFootball): the scorebug top
 * start with the clock and added time, the controlled player's plate with
 * his stamina bottom centre, the radar of the pitch bottom end, a
 * substitution's lower third.
 */
export function sportsHud(world: World): void {
  const { small, portrait, k } = frame();
  og.ui.setNavigation(false);
  void world;
  gameScene("scenes/genre-sports", { x: 0.5, y: 0.5 });
  const tick = og.ui.tick();
  const secs = 67 * 60 + 12 + Math.floor(tick / 60);
  og.ui.scorebug("Bug", { home: { ...HOME, possession: true }, away: AWAY, clock: `${Math.floor(secs / 60)}:${String(secs % 60).padStart(2, "0")}`, period: L("sc.sports.half2"), situation: tick % 600 < 300 ? L("sc.sports.corner") : "", anchor: "topLeft" });
  og.ui.playerPlate("Player", { number: "10", name: "D. Arlo", color: HOME.color, stamina: 0.64, segments: 10, anchor: "bottom" });
  og.ui.minimapView("Radar", {
    size: Math.round((small ? 110 : 180) * k), shape: "square", north: "",
    markers: [
      { key: "ball", kind: "objective", x: 0.71, y: 0.38 },
      ...[[0.7, 0.4], [0.6, 0.55], [0.48, 0.3], [0.52, 0.7], [0.35, 0.5]].map(([x, y], i) => ({ key: `h${i}`, kind: "ally" as const, x: x!, y: y! })),
      ...[[0.78, 0.45], [0.82, 0.32], [0.66, 0.66], [0.88, 0.55], [0.74, 0.22]].map(([x, y], i) => ({ key: `a${i}`, kind: "enemy" as const, x: x!, y: y! })),
    ],
    anchor: "bottomRight",
  });
  if (!portrait) og.ui.substitution("Sub", { caption: L("sc.sports.sub"), off: { number: "7", name: "K. Moreau" }, on: { number: "19", name: "T. Ilves" }, offLabel: L("sc.sports.off"), onLabel: L("sc.sports.on"), anchor: "bottomLeft" });
}

/**
 * The full-time board (EA FC's match facts): the scorers by side against a
 * centre rule and the stats face to face, over the stadium.
 */
export function matchFacts(world: World): void {
  const { t, small, k, m } = frame();
  void world;
  gameScene("scenes/genre-sports", { x: 0.5, y: 0.5 });
  og.ui.anchor("Facts", { anchor: small ? "top" : "center", margin: m, direction: "column", gap: t.space.md, width: small ? grow : px(640 * k), padding: { left: t.space.lg, top: t.space.lg, right: t.space.lg, bottom: t.space.lg }, background: { token: "hud" }, radius: t.radius["panel"] ?? 12, frame: "hud" }, () => {
    og.ui.scorebug("Bug", { home: HOME, away: AWAY, clock: "90:00", period: "", final: L("sc.sports.ft"), anchor: null });
    og.ui.goalTimeline("Goals", { events: [{ key: "g1", team: 0, name: "D. Arlo", minute: "23'", icon: "ball" }, { key: "g2", team: 1, name: "R. Kade", minute: "51'", icon: "ball" }, { key: "g3", team: 0, name: "T. Ilves", minute: "88'", icon: "ball", note: L("sc.sports.pen") }], anchor: null });
    og.ui.matchStats("Stats", {
      home: HOME, away: AWAY, header: false,
      stats: [
        { key: "pos", label: L("sc.sports.possession"), home: 58, away: 42, homeText: "58%", awayText: "42%" },
        { key: "shots", label: L("sc.sports.shots"), home: 14, away: 9 },
        { key: "target", label: L("sc.sports.onTarget"), home: 6, away: 3 },
        { key: "pass", label: L("sc.sports.passes"), home: 512, away: 388 },
        { key: "fouls", label: L("sc.sports.fouls"), home: 11, away: 14, worse: true },
      ],
      anchor: null,
    });
  });
}

export type { Anchor };
