// Saltwind Isle's screens on og.ui and the style in assets/styles/saltwind:
// the title over the camp (the game's screen documents,
// assets/ui/main-menu.ogt and pause.ogt), the HUD, the settings dialog and
// the run's summary. Screens change through a dissolve (a fade with
// reduced motion). Every string is a key of assets/strings/*.ogt.
//
// The HUD: the day and its hour, the landmarks claimed and the bag (the
// rpg kit's inventory grid) top-left; a compass strip per seat (the
// landmarks discovered, the next one to claim, the camp, the partner);
// Pause and the island's map (og.ui.minimapView) top-right; the quests
// (the rpg kit's tracker) on the right; what Gather would do and its
// progress at the bottom; on a touch screen a stick, Gather, Jump and Run,
// and a drag that turns the view.
//
// Every choice that changes the world reaches it through a seat's input
// (src/seats.ts press), so a room makes it on one tick on every peer: Set
// out and Again press Jump, Menu and Leave the isle press Menu, Pause and
// Resume press Pause.
//
// The Ui system also starts runs, moves the clock through three days and
// two nights (og.DayNight held by the game, so a pause holds it), ends a
// run at the third nightfall, holds the world while paused, and keeps this
// device's settings and record in og.platform (src/device.ts: never in the
// world).

import { og, defineSystem, math } from "@origingame/engine";
import type { CompassPoi, MapMarkerItem, Texture, Value, World } from "@origingame/engine";
import { rpg } from "@origingame/engine/extensions/rpg/index.ts";
import type { GridItem } from "@origingame/engine/extensions/rpg/index.ts";
import { BRIDGE_COST, DAYS, Ending, Game, Hero, Kind, Node, Phase, Site, SITE_COUNT, Sound, Target, Tuning } from "./components.ts";
import { prefs, record } from "./device.ts";
import { heroOf, hypot, ISO_YAW, lazyTuning, NODES } from "./hero.ts";
import { BRIDGE, CAMP, ISLE_SIZE, SITES } from "./layout.ts";
import { bit, LOCAL, localSeats, ONLINE, press, pressedNow, SEATS, setLookLive, usesTouch, useTouchControls } from "./seats.ts";
import { FIRST_PERSON, ISOMETRIC, SeatLook } from "./view.ts";
import { BRIDGE_SITE, HOURS, must, nodeOf, setHour } from "./world.ts";

export const LANGUAGES = [
  { code: "en", key: "lang.en" },
  { code: "zh-CN", key: "lang.zh" },
] as const;

/** Ticks between a run's end and its summary: the moment plays out first. */
export const OVER_DELAY = 90;
/** Ticks from a run's end until Jump and Menu answer on the summary: a third of a second after it shows. */
const SUMMARY_TICKS = OVER_DELAY + 20;

/** The tokens' spacing scale in CSS px. */
const S = { xxs: 2, xs: 4, sm: 8, md: 12, lg: 16 } as const;

type GameV = Value<typeof Game>;

export function say(world: World, key: string, args?: Record<string, string | number>): string {
  return og.i18n.t(world, key, args);
}

const grow = { kind: "grow" } as const;
const px = (n: number) => ({ kind: "fixed", px: n }) as const;

/** The textures the HUD draws: the bag's items and the island's map (loaded in `start`). */
const art = { wood: null as Texture | null, stone: null as Texture | null, map: null as Texture | null };

/** Loads the HUD's textures, once in `start`. */
export function loadHudArt(world: World): void {
  art.wood = must(og.load.texture(world, "textures/item-wood.png"));
  art.stone = must(og.load.texture(world, "textures/item-stone.png"));
  art.map = must(og.load.texture(world, "maps/isle.png"));
}

/** The clock at `time` seconds played: the day (1 to DAYS), the hour, and whether the sun is down. */
export function clockAt(t: { daySeconds: number; nightSeconds: number }, time: number): { day: number; hour: number; night: boolean } {
  const cycle = t.daySeconds + t.nightSeconds;
  const day = Math.min(DAYS, Math.floor(time / cycle) + 1);
  const u = time - (day - 1) * cycle;
  if (u < t.daySeconds) return { day, hour: HOURS.dawn + (u / t.daySeconds) * (HOURS.dusk - HOURS.dawn), night: false };
  const h = HOURS.dusk + ((u - t.daySeconds) / t.nightSeconds) * (24 - HOURS.dusk + HOURS.dawn);
  return { day, hour: h >= 24 ? h - 24 : h, night: true };
}

/** The seconds a run lasts: DAYS days and the nights between them, to the third nightfall. */
export function runSeconds(t: { daySeconds: number; nightSeconds: number }): number {
  return DAYS * t.daySeconds + (DAYS - 1) * t.nightSeconds;
}

/** A new run: the explorers at the camp, the bag empty, the isle reset, the clock at 7:00 on the first day. */
export function startRun(world: World, g: GameV): void {
  g.phase = Phase.Play;
  g.ending = Ending.None;
  g.paused = false;
  g.time = 0;
  g.day = 1;
  g.hour = HOURS.dawn;
  g.night = false;
  g.claimed = 0;
  g.claimMask = 0;
  g.seenMask = 0;
  g.wood = 0;
  g.stone = 0;
  g.woodTotal = 0;
  g.stoneTotal = 0;
  g.felled = 0;
  g.bridge = false;
  g.building = 0;
  g.builtAt = 0;
  g.washes = 0;
  g.jumps = 0;
  g.overTicks = 0;
  g.start = true;
  setHour(world, g.hour);
}

/** Back to the title: the isle reset behind it, late afternoon over the camp. */
export function toTitle(world: World, g: GameV): void {
  startRun(world, g);
  g.phase = Phase.Title;
  g.hour = HOURS.title;
  setHour(world, g.hour);
}

// ── this device's settings and record (src/device.ts) ──────────────────

function applyPrefs(world: World): void {
  const audio = world.resource(og.Audio);
  if (audio.ok) {
    audio.value.master = prefs.master;
    audio.value.music = prefs.music;
    audio.value.sfx = prefs.effects;
    audio.value.ui = prefs.effects;
  }
}

function setLanguage(world: World, index: number): void {
  prefs.language = Math.max(0, Math.min(LANGUAGES.length - 1, index));
  og.i18n.setLocale(world, LANGUAGES[prefs.language]!.code);
}

function prefsValue() {
  return { masterVolume: prefs.master, musicVolume: prefs.music, sfxVolume: prefs.effects, language: LANGUAGES[prefs.language]!.code, reducedMotion: prefs.reducedMotion };
}

/**
 * The record ({runs, wins, best}) written at the revision last read or
 * written. One call at a time: while a read or a write is in flight the
 * newer record waits (dirty) and goes out when that answers, at the
 * revision the answer gives.
 */
function saveRecord(world: World): void {
  if (record.request !== 0 || record.write !== 0) {
    record.dirty = true;
    return;
  }
  record.dirty = false;
  const r = og.platform.storeSet(world, { key: "record", value: { runs: record.runs, wins: record.wins, best: record.best }, revision: record.revision });
  if (!r.ok) return;
  record.write = r.value;
  record.sentRuns = record.runs;
  record.sentWins = record.wins;
}

/**
 * og.platform: the settings and the record asked for on tick 0 and applied
 * when they answer; a record write refused for a stale revision (another
 * device played meanwhile) is read again, merged (the stored counts plus
 * what this device added since, the better best time) and written at the
 * new revision.
 */
function platform(world: World): void {
  if (world.tick === 0) {
    prefs.loadRequest = og.platform.loadSettings(world);
    const r = og.platform.storeGet(world, "record");
    record.request = r.ok ? r.value : 0;
    return;
  }
  if (prefs.loadRequest !== 0) {
    const r = og.platform.result(world, prefs.loadRequest);
    if (r !== null) {
      prefs.loadRequest = 0;
      const s = ((r.ok ? r.data : null) as { value?: Record<string, unknown> } | null)?.value ?? null;
      if (s && typeof s === "object") {
        if (typeof s.masterVolume === "number") prefs.master = s.masterVolume;
        if (typeof s.musicVolume === "number") prefs.music = s.musicVolume;
        if (typeof s.sfxVolume === "number") prefs.effects = s.sfxVolume;
        if (typeof s.reducedMotion === "boolean") prefs.reducedMotion = s.reducedMotion;
        const at = LANGUAGES.findIndex((l) => l.code === s.language);
        if (at >= 0) setLanguage(world, at);
        prefs.fresh = true;
      }
    }
  }
  if (record.request !== 0) {
    const r = og.platform.result(world, record.request);
    if (r !== null) {
      record.request = 0;
      const d = (r.ok ? r.data : null) as { value?: { runs?: number; wins?: number; best?: number } | null; revision?: number } | null;
      const runs = d?.value?.runs ?? 0;
      const wins = d?.value?.wins ?? 0;
      const best = d?.value?.best ?? 0;
      record.runs = runs + record.runs - record.storedRuns;
      record.wins = wins + record.wins - record.storedWins;
      if (best > 0 && (record.best === 0 || best < record.best)) record.best = best;
      record.storedRuns = runs;
      record.storedWins = wins;
      if (typeof d?.revision === "number") record.revision = d.revision;
      if (record.merge || record.dirty) {
        record.merge = false;
        saveRecord(world);
      }
    }
  }
  if (record.write !== 0) {
    const r = og.platform.result(world, record.write);
    if (r !== null) {
      record.write = 0;
      if (r.ok) {
        record.revision = (r.data as { revision: number }).revision;
        record.storedRuns = record.sentRuns;
        record.storedWins = record.sentWins;
        if (record.dirty) saveRecord(world);
      } else if (r.error?.code === "conflict") {
        record.conflicts += 1;
        record.merge = true;
        const again = og.platform.storeGet(world, "record");
        record.request = again.ok ? again.value : 0;
      } else if (record.dirty) saveRecord(world);
    }
  }
}

// ── the clock and the end ──────────────────────────────────────────────

/** The clock (three days, two nights between) and the endings: the third nightfall, or (src/work.ts) the fifth landmark claimed. */
function clock(world: World, g: GameV, dt: number): void {
  if (g.phase === Phase.Over) g.overTicks += 1;
  if (g.phase !== Phase.Play) return;
  const t = lazyTuning(world);
  if (g.ending === Ending.None) {
    g.time = Math.min(runSeconds(t), g.time + dt);
    const c = clockAt(t, g.time);
    if (c.night && !g.night) g.sfx |= Sound.Night;
    if (!c.night && g.night) {
      g.dawn = true;
      g.sfx |= Sound.Dawn;
    }
    g.day = c.day;
    g.hour = c.hour;
    g.night = c.night;
    if (g.time >= runSeconds(t)) {
      g.ending = Ending.Night;
      g.night = true;
    }
  }
  setHour(world, g.hour);
  if (g.ending === Ending.None) return;
  g.overTicks += 1;
  if (g.overTicks === OVER_DELAY) {
    g.phase = Phase.Over;
    if (g.ending === Ending.Lit) {
      record.wins += 1;
      record.newBest = record.best === 0 || g.time < record.best;
      if (record.newBest) record.best = g.time;
    } else record.newBest = false;
    saveRecord(world);
    must(og.platform.telemetry(world, "run.over", { ending: ["none", "lit", "night"][g.ending]!, time: Math.round(g.time), claimed: g.claimed, bridge: g.bridge, washes: g.washes }));
  }
}

// ── the HUD ────────────────────────────────────────────────────────────

function capsule(id: string, build: () => void): void {
  og.ui.row(id, { gap: S.xs, cross: "center", padding: { left: S.sm, top: S.xs, right: S.md, bottom: S.xs }, frame: "hud", background: { token: "panel" } }, build);
}

export function hourText(hour: number): string {
  const h = Math.floor(hour);
  const m = Math.floor((hour - h) * 60);
  return `${h}:${m < 10 ? "0" : ""}${m}`;
}

/** South of this z is the main island, north of it the north isle (the strait's middle). */
const NORTH_ISLE = (BRIDGE.south + BRIDGE.north) / 2;

/** The isle's map coordinates (0 to 1, north up) of a point. */
const mapX = (x: number) => x / ISLE_SIZE + 0.5;
const mapY = (z: number) => z / ISLE_SIZE + 0.5;

/** A seat's heading, clockwise from north, as its camera looks (the fixed angle in isometric). */
function headingOf(world: World, g: GameV, seat: number): number {
  if (ISOMETRIC) return -ISO_YAW;
  return -must(world.get(heroOf(g, seat), SeatLook)).yaw;
}

/** The landmark the lights quest points at (src/work.ts markNext), or -1. */
function nextSite(world: World): number {
  const q = og.story.quest(world, "lights");
  const m = q?.marker;
  if (!m) return -1;
  return SITES.findIndex((s) => Math.abs(s.x - m.position.x) < 0.5 && Math.abs(s.z - m.position.z) < 0.5);
}

/** The strip's points for a seat: the landmarks discovered (the next one an objective), the camp, the bridge, the partner. */
function compassPois(world: World, g: GameV, seat: number, next: number): CompassPoi[] {
  const p = must(world.get(heroOf(g, seat), og.Transform)).position;
  const at = (x: number, z: number) => ({ bearing: math.atan2(x - p.x, -(z - p.z)), distance: Math.round(hypot(x - p.x, z - p.z)) });
  const pois: CompassPoi[] = [];
  SITES.forEach((s, i) => {
    const seen = (g.seenMask & (1 << i)) !== 0;
    const claimed = (g.claimMask & (1 << i)) !== 0;
    if (!seen && i !== next) return;
    pois.push({ key: `Site${i}`, ...at(s.x, s.z), icon: claimed ? "check" : "flag", kind: i === next ? "objective" : "poi", label: say(world, `site.${s.name}`) });
  });
  pois.push({ key: "Camp", ...at(CAMP.x, CAMP.z), icon: "home", kind: "poi", label: say(world, "map.camp") });
  if (!g.bridge) pois.push({ key: "Bridge", ...at(BRIDGE_SITE.x, BRIDGE_SITE.z), icon: "plus", kind: "poi", label: say(world, "map.bridge") });
  if (SEATS > 1) {
    const o = must(world.get(heroOf(g, 1 - seat), og.Transform)).position;
    pois.push({ key: "Partner", ...at(o.x, o.z), kind: "ally", label: say(world, "map.partner") });
  }
  return pois;
}

/** The map's markers: the landmarks (claimed, discovered, the next), the camp, the bridge, the explorers. */
function mapMarkers(world: World, g: GameV, next: number): MapMarkerItem[] {
  const out: MapMarkerItem[] = [];
  SITES.forEach((s, i) => {
    const seen = (g.seenMask & (1 << i)) !== 0;
    if (!seen && i !== next) return;
    const claimed = (g.claimMask & (1 << i)) !== 0;
    out.push({ key: `Site${i}`, kind: i === next ? "objective" : "poi", x: mapX(s.x), y: mapY(s.z), icon: claimed ? "check" : "flag", label: say(world, `site.${s.name}`) });
  });
  out.push({ key: "Camp", kind: "poi", x: mapX(CAMP.x), y: mapY(CAMP.z), icon: "home", label: say(world, "map.camp") });
  if (!g.bridge) out.push({ key: "Bridge", kind: "poi", x: mapX(BRIDGE_SITE.x), y: mapY(BRIDGE_SITE.z), icon: "plus", label: say(world, "map.bridge") });
  for (let seat = 0; seat < SEATS; seat++) {
    const p = must(world.get(heroOf(g, seat), og.Transform)).position;
    const me = localSeats(world).includes(seat) && (!LOCAL || seat === 0);
    out.push({ key: `Seat${seat}`, kind: me ? "player" : "ally", x: mapX(p.x), y: mapY(p.z), heading: headingOf(world, g, seat), label: say(world, seat === 0 ? "map.seat0" : "map.seat1") });
  }
  return out;
}

/** The bag's items as the kit's grid shows them: the wood and the stone, each one stack. */
function bagItems(world: World, g: GameV): GridItem[] {
  const items: GridItem[] = [];
  if (g.wood > 0) items.push({ key: "Wood", name: say(world, "item.wood"), image: art.wood, x: 0, y: 0, count: g.wood, maxStack: 99, tag: "wood", rarity: "common" });
  if (g.stone > 0) items.push({ key: "Stone", name: say(world, "item.stone"), image: art.stone, x: 1, y: 0, count: g.stone, maxStack: 99, tag: "stone", rarity: "common" });
  return items;
}

/** What Gather does for a seat now, and how far along it is (null: nothing in reach). */
function promptOf(world: World, g: GameV, seat: number): { key: string; args: Record<string, string | number>; progress: number } | null {
  const me = must(world.get(heroOf(g, seat), Hero));
  const t = lazyTuning(world);
  const touch = usesTouch() && (seat === 0 || LOCAL);
  const pad = must(world.resource(og.Gamepad)).connected();
  const key = say(world, touch ? "key.touch" : pad ? "key.pad" : LOCAL && seat === 1 ? "key.seat1" : "key.seat0");
  if (me.target === Target.Bridge) {
    if (g.wood < BRIDGE_COST.wood || g.stone < BRIDGE_COST.stone) return { key: "hud.need", args: { wood: BRIDGE_COST.wood, stone: BRIDGE_COST.stone, haveWood: g.wood, haveStone: g.stone }, progress: 0 };
    return { key: "hud.build", args: { key }, progress: g.building / t.buildSeconds };
  }
  if (me.target === Target.Node && me.node >= 0) {
    const n = must(world.get(nodeOf(world, me.node), Node));
    const tree = NODES[me.node]!.kind === Kind.Tree;
    const fresh = n.workedTick + 1 >= world.tick;
    return { key: tree ? "hud.fell" : "hud.break", args: { key }, progress: fresh ? n.progress / (tree ? t.chopSeconds : t.breakSeconds) : 0 };
  }
  return null;
}

/** Alone on a phone: the compass's row along the top, which the rest of the HUD sits under. */
const COMPASS_ROW = 80;
/** Where this device's toasts stack now (UiDesc.toastAnchor). */
let toastsAt: "center" | null = null;

function hud(world: World, g: GameV): void {
  const vp = og.ui.frameViewport();
  const phone = vp.width < 600;
  // Alone on a phone in portrait the compass runs along the top beside Pause, and the rest sits under it.
  const tall = phone && !LOCAL;
  const below = tall ? COMPASS_ROW : 0;
  // On a phone the touch controls take the foot, so the toasts stack in the middle.
  const toasts = phone ? "center" : null;
  if (toasts !== toastsAt) {
    toastsAt = toasts;
    must(og.ui.configure(world, { toastAnchor: toasts }));
  }
  og.ui.setNavigation(false);
  og.ui.anchor("Status", { anchor: "topLeft", offsetY: below, direction: "column", gap: S.sm }, () => {
    capsule("Clock", () => {
      og.ui.icon("Glyph", "clock", { size: 20, tint: { token: g.night ? "info" : g.day === DAYS && g.hour > HOURS.dusk - 2 ? "danger" : "ink" } });
      og.ui.label("Day", say(world, "hud.day", { n: g.day, total: DAYS }), { nowrap: true, font: "display" });
      og.ui.label("Hour", hourText(g.hour), { font: "mono", nowrap: true, width: px(44) });
    });
    capsule("Lights", () => {
      og.ui.icon("Flag", "flag", { size: 20, tint: { token: "primary" } });
      og.ui.label("Count", say(world, "hud.lights", { n: g.claimed, total: SITE_COUNT }), { step: "lg", font: "mono", nowrap: true });
    });
    rpg.inventoryGrid("Bag", { columns: 2, rows: 1, cellSize: phone ? 44 : 48, items: bagItems(world, g), label: say(world, "hud.bag"), emptyLabel: say(world, "hud.empty") });
  });
  const next = nextSite(world);
  // Pause acts through the seat's input (src/seats.ts), so a room pauses on one tick.
  const pause = () => {
    if (og.ui.actionButton("Pause", say(world, "hud.pause"), { icon: "pause", size: 48 }).clicked) press("pause");
  };
  if (tall) og.ui.anchor("PauseHud", { anchor: "topRight" }, pause);
  og.ui.anchor("Corner", { anchor: "topRight", offsetY: below, direction: "column", gap: S.sm, cross: "end" }, () => {
    if (!tall) pause();
    const me = localSeats(world)[0] ?? 0;
    const p = must(world.get(heroOf(g, me), og.Transform)).position;
    og.ui.minimapView("Map", {
      texture: art.map,
      ...(LOCAL ? { centerX: 0.5, centerY: 0.5, zoom: 1 } : { centerX: mapX(p.x), centerY: mapY(p.z), zoom: 0.42 }),
      rotate: false,
      heading: headingOf(world, g, me),
      size: phone ? 112 : 180,
      markers: mapMarkers(world, g, next),
      zone: say(world, p.z < NORTH_ISLE ? "zone.north" : "zone.south"),
      north: say(world, "map.north"),
      anchor: null,
    });
  });
  // The quests, from the kit: the lights and the bridge, under the map (a phone shows the first; two on one phone have no room for them).
  const cramped = LOCAL && phone;
  if (!cramped) {
    // Alone on a phone, under the status on the left (the map takes the right).
    og.ui.anchor("QuestsHud", tall ? { anchor: "topLeft", offsetY: below + 130 } : { anchor: "topRight", offsetY: 300 }, () => {
      rpg.questTracker("Quests", { quests: og.story.quests(world), from: must(world.get(heroOf(g, localSeats(world)[0] ?? 0), og.Transform)).position, width: phone ? 220 : 260, max: phone ? 1 : 0, completeLabel: say(world, "quest.complete"), anchor: null });
    });
  }
  if (g.ending !== Ending.None) return;
  const seats = LOCAL ? [0, 1] : localSeats(world);
  const points = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"].map((k) => say(world, `compass.${k}`));
  for (const seat of seats) {
    const offsetX = LOCAL ? (seat === 0 ? -1 : 1) * (vp.width / 4) : 0;
    const suffix = seat === 0 || !LOCAL ? "" : `${seat}`;
    // The compass: along the top alone (on a phone left of Pause), at the foot of each half on a shared
    // screen (on a phone under the top bar, the touch controls taking the foot).
    const top = !LOCAL || phone;
    og.ui.anchor(`Heading${suffix}`, { anchor: top ? "top" : "bottom", offsetX: tall ? -28 : offsetX, offsetY: !top ? -8 : LOCAL && phone ? 244 : 0 }, () => {
      og.ui.compass("Compass", { heading: headingOf(world, g, seat), width: LOCAL ? Math.min(360, vp.width / 2 - 24) : phone ? vp.width - 88 : 520, pois: compassPois(world, g, seat, next), points, units: say(world, "compass.units"), anchor: null });
    });
    const prompt = promptOf(world, g, seat);
    if (prompt === null) continue;
    const touchUp = usesTouch() ? (LOCAL ? 300 : 200) : LOCAL ? 80 : 32;
    // A shared screen's prompt wraps inside its own half.
    const room = LOCAL ? vp.width / 2 - 24 : Math.min(520, vp.width - 32);
    og.ui.anchor(`Prompt${suffix}`, { anchor: "bottom", offsetX, offsetY: -touchUp, direction: "column", cross: "center", gap: S.xs }, () => {
      capsule("Hint", () => {
        og.ui.icon("Glyph", prompt.key === "hud.break" ? "gem" : prompt.key === "hud.fell" ? "leaf" : "plus", { size: 18, tint: { token: prompt.key === "hud.need" ? "warn" : "primary" } });
        og.ui.label("Text", say(world, prompt.key, prompt.args), { width: { kind: "fit", max: room - 48 } });
      });
      if (prompt.progress > 0) og.ui.progress("Work", Math.min(1, prompt.progress), { width: px(Math.min(200, room)) });
    });
  }
}

// ── feedback from the world's changes (this device's toasts) ───────────

/** What this device last showed, to toast what changed since. */
const shown = { seen: 0, claimed: 0, day: 1, night: false, washes: 0, bridge: false, lastNight: false };

function toasts(world: World, g: GameV): void {
  if (g.start || g.phase !== Phase.Play) {
    Object.assign(shown, { seen: g.seenMask, claimed: g.claimMask, day: g.day, night: g.night, washes: g.washes, bridge: g.bridge, lastNight: false });
    return;
  }
  for (let i = 0; i < SITE_COUNT; i++) {
    const b = 1 << i;
    const name = say(world, `site.${SITES[i]!.name}`);
    if ((g.claimMask & b) !== 0 && (shown.claimed & b) === 0 && g.claimed < SITE_COUNT) og.ui.toast(world, say(world, "toast.claimed", { name, n: g.claimed, total: SITE_COUNT }), { tone: "success" });
    else if ((g.seenMask & b) !== 0 && (shown.seen & b) === 0 && (g.claimMask & b) === 0) og.ui.toast(world, say(world, "toast.discovered", { name }), { tone: "info" });
  }
  if (g.bridge && !shown.bridge) og.ui.toast(world, say(world, "toast.bridge"), { tone: "success" });
  if (g.washes > shown.washes) og.ui.toast(world, say(world, "toast.washed"), { tone: "warn" });
  if (g.night && !shown.night && g.ending === Ending.None) og.ui.toast(world, say(world, "toast.night", { n: g.day }), { tone: "info" });
  if (!g.night && shown.night) og.ui.toast(world, say(world, "toast.dawn", { n: g.day }), { tone: "primary" });
  const t = lazyTuning(world);
  if (!shown.lastNight && g.day === DAYS && !g.night && clockAt(t, g.time).hour > HOURS.dusk - 2.5 && g.ending === Ending.None) {
    shown.lastNight = true;
    og.ui.toast(world, say(world, "toast.last"), { tone: "danger" });
  }
  Object.assign(shown, { seen: g.seenMask, claimed: g.claimMask, day: g.day, night: g.night, washes: g.washes, bridge: g.bridge });
}

// ── the touch HUD ──────────────────────────────────────────────────────

/** What the touch HUD needs from the world, kept by Ui each tick (it is drawn by Seats, before Ui). */
const touchHud = { shown: false, gather: [false, false], world: null as World | null };
/** Each seat's look finger on this device, and the taps that answered on the ui bus. */
const looks = [{ id: -1, x: 0, y: 0 }, { id: -1, x: 0, y: 0 }];
const touchState = { taps: 0, held: [0, 0] };

/** Degrees the view turns per CSS px a finger drags across the look zone. */
const LOOK_DEG_PER_PX = 0.4;

/**
 * Saltwind Isle's touch HUD, drawn by Seats for each seat this device plays
 * while touch shows and a run is on screen. Alone: the stick bottom left
 * (pushed all the way it runs), Gather, Jump and Run bottom right, and a
 * finger that lands on the right half (below the map, above the buttons)
 * turning the view as it drags. Two on one device: each seat its own
 * column in its own bottom corner, and its own half to drag the view in.
 */
function touchControls(seat: number): { x: number; y: number; lx: number; ly: number; buttons: number } {
  const out = { x: 0, y: 0, lx: 0, ly: 0, buttons: 0 };
  const world = touchHud.world;
  const look = looks[seat]!;
  if (!touchHud.shown || world === null) {
    look.id = -1;
    return out;
  }
  const vp = og.ui.frameViewport();
  const stick = () => {
    const s = og.ui.joystick("Stick", { size: LOCAL ? 116 : 140, deadZone: 0.12 });
    const held = og.ui.memory("Held", 0);
    if (s.active && held === 0) touchState.taps += 1;
    if ((s.active ? 1 : 0) !== held) og.ui.remember("Held", s.active ? 1 : 0);
    if (s.active) {
      out.x = s.x;
      out.y = -s.y;
      if (hypot(s.x, s.y) > 0.9) out.buttons |= bit("run");
    }
  };
  const hands = () => {
    const gather = og.ui.actionButton("Gather", say(world, "hud.gather"), { icon: "leaf", size: LOCAL ? 60 : 72, primary: touchHud.gather[seat] ?? false }).pressed;
    if (gather && (touchState.held[seat] ?? 0) === 0) touchState.taps += 1;
    touchState.held[seat] = gather ? 1 : 0;
    if (gather) out.buttons |= bit("gather");
    og.ui.column("Pair", { gap: S.sm, cross: "center" }, () => {
      if (og.ui.actionButton("Jump", say(world, "hud.jump"), { icon: "play", size: LOCAL ? 52 : 60 }).clicked) {
        out.buttons |= bit("jump");
        touchState.taps += 1;
      }
      if (og.ui.actionButton("Run", say(world, "hud.run"), { icon: "bolt", size: LOCAL ? 48 : 56 }).pressed) out.buttons |= bit("run");
    });
  };
  if (LOCAL) {
    og.ui.anchor(`TouchSeat${seat}`, { anchor: seat === 0 ? "bottomLeft" : "bottomRight", direction: "column", gap: S.md, cross: "center" }, () => {
      og.ui.row("Hands", { gap: S.sm, cross: "end" }, hands);
      stick();
    });
  } else {
    og.ui.anchor("Move", { anchor: "bottomLeft" }, stick);
    og.ui.anchor("Hands", { anchor: "bottomRight", gap: S.md, cross: "end" }, hands);
  }
  // The look: degrees of the view's turn this tick (right turns it right, up tips it up).
  const ptr = must(world.resource(og.Pointers));
  if (look.id >= 0 && !ptr.down(look.id)) look.id = -1;
  if (look.id >= 0) {
    const at = ptr.position(look.id);
    out.lx = (at.x - look.x) * LOOK_DEG_PER_PX; // lockstep-ok: this device's input, sent to the room
    out.ly = -(at.y - look.y) * LOOK_DEG_PER_PX * 0.6;
    look.x = at.x;
    look.y = at.y;
  } else {
    const top = vp.safeArea.top + (vp.width < 600 ? 160 : 230);
    const bottom = vp.height - vp.safeArea.bottom - (LOCAL ? 260 : 230);
    const left = LOCAL ? (seat === 0 ? 0 : vp.width / 2) : vp.width / 2;
    const right = LOCAL ? (seat === 0 ? vp.width / 2 : vp.width) : vp.width;
    // Every pointer id (0 to 4): a finger's id is not below the count of those held.
    for (let n = 0; n < 5; n++) {
      if (!ptr.pressed(n) || looks.some((l) => l.id === n)) continue;
      const at = ptr.start(n);
      if (at.x < left || at.x >= right || at.y < top || at.y > bottom) continue;
      look.id = n;
      look.x = ptr.position(n).x;
      look.y = ptr.position(n).y;
      break;
    }
  }
  return out;
}
useTouchControls(touchControls);

// ── the seats' choices, the settings, the summary ──────────────────────

/** A seat's control went down this tick (against Hero.before, which the Hero system moves on after Ui). */
function pressedBy(world: World, g: GameV, name: string): boolean {
  for (let seat = 0; seat < SEATS; seat++) {
    const hero = heroOf(g, seat);
    if (pressedNow(must(world.get(hero, og.PlayerInput)).buttons, must(world.get(hero, Hero)).before, name)) return true;
  }
  return false;
}

/**
 * The seats' choices, from their input: Pause opens and closes the pause
 * menu (under the settings it closes them); on the title Jump sets out;
 * paused, Menu leaves the isle; on the summary Jump goes again and Menu
 * goes back to the title.
 */
function choices(world: World, g: GameV): void {
  if (g.settingsOpen) {
    if (pressedBy(world, g, "pause")) closeSettings(world, g);
    return;
  }
  if (pressedBy(world, g, "pause")) {
    if (g.paused) g.paused = false;
    else if (g.phase === Phase.Play && g.ending === Ending.None) g.paused = true;
  }
  if (g.phase === Phase.Title) {
    if (pressedBy(world, g, "jump")) begin(world, g);
  } else if (g.phase === Phase.Play) {
    if (g.paused && pressedBy(world, g, "menu")) toTitle(world, g);
  } else if (g.overTicks >= SUMMARY_TICKS) {
    if (pressedBy(world, g, "jump")) begin(world, g);
    else if (pressedBy(world, g, "menu")) toTitle(world, g);
  }
}

/** A run starts and this device counts it in its record. */
export function begin(world: World, g: GameV): void {
  startRun(world, g);
  record.runs += 1;
  saveRecord(world);
}

function settings(world: World, g: GameV): void {
  let done = false;
  const before = JSON.stringify(prefsValue());
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
    prefs.master = slider("Master", "settings.master", prefs.master);
    prefs.music = slider("Music", "settings.music", prefs.music);
    prefs.effects = slider("Effects", "settings.effects", prefs.effects);
    og.ui.label("LanguageLabel", say(world, "settings.language"), { color: { token: "inkMuted" } });
    const r = og.ui.radio("Language", LANGUAGES.map((l) => say(world, l.key)), prefs.language, { horizontal: true });
    if (r.selected !== prefs.language) setLanguage(world, r.selected);
    prefs.reducedMotion = og.ui.toggle("Motion", prefs.reducedMotion, say(world, "settings.motion")).on;
    og.ui.row("Footer", { width: grow, align: "end" }, () => {
      if (og.ui.button("Done", say(world, "settings.done"), { variant: "primary" }).clicked) done = true;
    });
  });
  if (JSON.stringify(prefsValue()) !== before) applyPrefs(world);
  if (g.settingsOpen && done) closeSettings(world, g);
}

/** The settings close (Done, or a seat's Pause: Escape, Start) and are saved. */
function closeSettings(world: World, g: GameV): void {
  g.settingsOpen = false;
  must(og.platform.saveSettings(world, prefsValue()));
  prefs.saves += 1;
}

/** A time of play as the day and the hour it reached. */
function when(world: World, g: GameV): string {
  const c = clockAt(lazyTuning(world), g.time);
  return say(world, "over.when", { day: c.day, hour: hourText(c.hour) });
}

function seconds(s: number): string {
  const m = Math.floor(s / 60);
  const r = Math.floor(s % 60);
  return `${m}:${r < 10 ? "0" : ""}${r}`;
}

/** The run's summary: how it ended, when, the landmarks, the gathering, the record. */
function summary(world: World, g: GameV): "again" | "menu" | null {
  let chose: "again" | "menu" | null = null;
  const won = g.ending === Ending.Lit;
  og.ui.modal("Summary", g.phase === Phase.Over, { title: say(world, won ? "over.won" : "over.night"), locked: true, width: px(Math.min(400, og.ui.frameViewport().width - 32)) }, () => {
    og.ui.row("Mark", { width: grow, align: "center" }, () => {
      og.ui.transition("Icon", true, { kind: "pop" }, () => og.ui.icon("Glyph", won ? "trophy" : "clock", { size: 48, tint: { token: won ? "primary" : "inkMuted" } }));
      if (won && g.overTicks >= OVER_DELAY + 20) og.ui.burst({ count: 14 });
    });
    og.ui.label("Line", say(world, won ? "over.won.line" : "over.night.line", { n: g.claimed, total: SITE_COUNT }), { color: { token: "inkMuted" }, align: "center", width: grow });
    og.ui.column("Stats", { width: grow, gap: 0 }, () => {
      og.ui.keyValue("When", say(world, "over.time"), when(world, g), true);
      og.ui.keyValue("Lights", say(world, "over.lights"), `${g.claimed}/${SITE_COUNT}`, true);
      og.ui.keyValue("Gathered", say(world, "over.gathered"), say(world, "over.gathered.value", { wood: g.woodTotal, stone: g.stoneTotal }), true);
      og.ui.keyValue("Bridge", say(world, "over.bridge"), say(world, g.bridge ? "over.bridge.built" : "over.bridge.none"), true);
      og.ui.keyValue("Washes", say(world, "over.washes"), `${g.washes}`, true);
      og.ui.keyValue("Best", say(world, "over.best"), record.best > 0 ? (record.newBest && won ? say(world, "over.newBest", { time: seconds(record.best) }) : seconds(record.best)) : say(world, "over.none"), false);
    });
    og.ui.label("Record", say(world, "over.record", { runs: record.runs, wins: record.wins }), { color: { token: "inkMuted" }, align: "center", width: grow });
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
 * popped. Answers the events it emitted (its buttons' `emit`, Back's).
 */
function layer(world: World, name: string, up: boolean, vm: object = {}): string[] {
  if (!up) {
    if (og.ui.layers(world).some((l) => l.screen === name)) og.ui.pop(world);
    return [];
  }
  return og.ui.screen(name, vm).events.filter((e) => e.kind === "emit" || e.kind === "back").map((e) => e.name);
}

/** The title's slow look round the camp: every seat's view turns (third and first person; the isometric angle is fixed). */
function titleOrbit(world: World, g: GameV, dt: number): void {
  if (g.phase !== Phase.Title || ISOMETRIC || dt === 0) return;
  for (let seat = 0; seat < SEATS; seat++) {
    const hero = heroOf(g, seat);
    const look = must(world.get(hero, SeatLook));
    must(world.set(hero, SeatLook, { yaw: look.yaw + (FIRST_PERSON ? 0.06 : 0.09) * dt }));
  }
}

/**
 * This device's platform answers (the settings, the record): a system that
 * writes no shared state, so a lockstep session lets it hear them (they are
 * this peer's alone, and stay out of the world).
 */
export const Device = defineSystem("Device", { before: ["Ui"] }, (_q, _time, world) => {
  platform(world);
});

/** The screens, the run's start, clock and end, the hold, the buses. */
export const Ui = defineSystem("Ui", { reads: [Tuning, og.Gamepad, og.Pointers, og.PlayerInput, og.Lockstep, Hero, Node, Site, og.Transform], writes: [Game, SeatLook, og.Audio, og.DayNight, og.Fog] }, (_q, time, world) => {
  const g = must(world.resource(Game));
  if (time.tick === 0 || prefs.fresh) {
    prefs.fresh = false;
    applyPrefs(world);
  }
  choices(world, g);
  const hold = g.phase === Phase.Play && (g.paused || g.settingsOpen);
  if (hold !== g.held) {
    must(og.time.setScale(world, hold ? 0 : 1));
    g.held = hold;
  }
  clock(world, g, time.scaledDt);
  titleOrbit(world, g, time.dt);
  toasts(world, g);

  const screen = g.phase === Phase.Title ? "title" : g.phase === Phase.Play ? "play" : "over";
  const up = og.ui.screenTransition("Screens", screen, { kind: prefs.reducedMotion ? "fade" : "dissolve", durationMs: 600 }).shown;
  // The title and the pause are the game's screen documents (assets/ui/main-menu.ogt, pause.ogt).
  for (const e of layer(world, "main-menu", up === "title", { touch: usesTouch() })) {
    if (!g.settingsOpen && e === "play" && g.phase === Phase.Title) press("jump");
    if (e === "settings") g.settingsOpen = true;
  }
  const fromSettings = og.ui.memory("pause.fromSettings", 0);
  const pauseUp = up === "play" && g.paused && !g.settingsOpen;
  for (const e of layer(world, "pause", pauseUp)) {
    if (e === "resume") press("pause");
    if (e === "settings") {
      g.settingsOpen = true;
      og.ui.remember("pause.fromSettings", 1);
    }
    if (e === "quit") press("menu");
  }
  if (pauseUp && fromSettings === 1) {
    og.ui.setFocus("pause/Pause/Panel/Menu/Settings");
    og.ui.remember("pause.fromSettings", 0);
  }
  if (up === "title") {
    og.ui.setNavigation(true);
  } else if (up === "play") {
    hud(world, g);
  } else {
    og.ui.setNavigation(true);
    const c = summary(world, g);
    if (c === "again") press("jump");
    if (c === "menu") press("menu");
  }
  // The touch HUD (drawn by Seats next tick) and its taps on the ui bus. A
  // tap is this device's own: a room plays none, which would part the peers.
  touchHud.shown = up === "play" && g.ending === Ending.None;
  for (let seat = 0; seat < SEATS; seat++) touchHud.gather[seat] = must(world.get(heroOf(g, seat), Hero)).target !== Target.None;
  touchHud.world = world;
  if (touchState.taps > 0 && !ONLINE) g.sfx |= Sound.Tap;
  touchState.taps = 0;
  // The mouse turns the view only in a run: on the screens the pointer is the player's.
  setLookLive(g.phase === Phase.Play && !g.paused && !g.settingsOpen);
  settings(world, g);
});
