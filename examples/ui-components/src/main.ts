// The component library's feedback, HUD and world components (docs/ui.md
// §11 and §14, roadmap U6c) on one stage, a screen per group, each showing
// its components in their states at once: idle, active, low and critical,
// cooling down, stacked, empty, overflowing numbers and edge-clamped.
//
// Commands:
//   show:<feedback|meters|abilities|frames|shooter|strategy|racing|flight|world>
//   viewport:<desktop|ultrawide|tablet|phone|landscape>  (og.ui.previewViewport; render at the matching --size)
//   lang:<en|ar|pseudo>     ar lays out right to left; pseudo is 40 % longer, accented and bracketed
//   style:<prototype|scifi|fantasy|arcade|tactical|broadcast>
//   text:<percent>          the player's text size, 100 to 200 (§15.2 holds every screen to 200)
//   act:<hit|crit|heal|ready|empty|reload|interrupt|ping|say|complete|combo|score|break|place>
//   reset   every act forgotten, the clock of the screen started again
//
//   og run examples/ui-components --size 1920x1080 --out film --commands "$(cat film-desktop.txt)"
//
// film-desktop.txt (1920×1080), film-ultrawide.txt (3440×1440),
// film-tablet.txt (2360×1640), film-phone.txt (1170×2532) and
// film-landscape.txt (2532×1170) take every screen through its acts; the
// desktop film also runs each screen in Arabic and under the pseudo-locale.

import { og, defineSystem, SHAPE_BOX } from "@origingame/engine";
import type { DamageHitDesc, KillfeedEntry, Texture, UnitFrameDesc, Viewport, World } from "@origingame/engine";

const SCREENS = ["feedback", "meters", "abilities", "frames", "shooter", "strategy", "racing", "flight", "world"] as const;
type Screen = (typeof SCREENS)[number];
const STYLES = ["prototype", "scifi", "fantasy", "arcade", "tactical", "broadcast"] as const;

const VIEWPORTS: Record<string, Viewport> = {
  desktop: { width: 1920, height: 1080, dpr: 1, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" },
  ultrawide: { width: 3440, height: 1440, dpr: 1, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" },
  tablet: { width: 1180, height: 820, dpr: 2, safeArea: { left: 0, top: 24, right: 0, bottom: 20 }, coarsePointer: true, breakpoint: "tablet" },
  phone: { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" },
  landscape: { width: 844, height: 390, dpr: 3, safeArea: { left: 47, top: 0, right: 47, bottom: 21 }, coarsePointer: true, breakpoint: "phoneLandscape" },
};

/** The stage's state: the screen, the locale and style, and when each act last happened (og.ui.tick()). */
const state = {
  screen: "meters" as Screen,
  locale: "en" as "en" | "ar" | "pseudo",
  style: "prototype" as (typeof STYLES)[number],
  t0: 0,
  acts: {} as Record<string, number>,
  hits: 0,
  kills: [] as KillfeedEntry[],
  fonts: { body: 0, icons: 0 },
  /** The abilities' painted art (assets/abilities), by name; empty before it loads. */
  art: {} as Record<string, Texture>,
  /**
   * What commands asked for since the last frame. A command runs between
   * ticks, where og.ui.tick() answers 0 (it reads the frame being built),
   * so the stage applies them at the start of its next frame, stamped
   * with that frame's tick.
   */
  queue: [] as Array<() => void>,
  /** The player's text size (UiDesc.textScale), 1 to 2. */
  textScale: 1,
};

/** The pseudo-locale: 40 % longer, accented and bracketed (the checks' own), numbers as they are. */
function L(s: string): string {
  if (state.locale !== "pseudo" || !s) return s;
  const map: Record<string, string> = { a: "á", e: "é", i: "í", o: "ö", u: "ü", c: "ç", n: "ñ", A: "Å", E: "É", I: "Î", O: "Ø", U: "Û", C: "Ç", N: "Ñ", y: "ý", s: "š", z: "ž" };
  // Rich text (a string with tags): the tags stay as written and the pseudo-locale's own bracket is escaped, so neither forms a tag.
  const rich = s.includes("[");
  const body = s.split(/(\[[^\]]*\])/).map((part, i) => (rich && i % 2 === 1 ? part : [...part].map((ch) => map[ch] ?? ch).join(""))).join("");
  const pad = "·".repeat(Math.ceil(s.length * 0.4));
  return `${rich ? "[[" : "["}${body}${pad}]`;
}

/** Ticks since an act, or a large number before it ever happened. */
function since(name: string): number {
  const at = state.acts[name];
  return at === undefined ? 1e9 : og.ui.tick() - at;
}
const did = (name: string): boolean => state.acts[name] !== undefined;
/** The tick of the latest hit or crit, null before either. */
const lastHit = (): number | null => {
  const t = Math.max(state.acts.hit ?? -1, state.acts.crit ?? -1);
  return t < 0 ? null : t;
};
const seconds = (): number => (og.ui.tick() - state.t0) / 60;
/** A phone in either orientation: the screens show what a mobile HUD would, not everything at once. */
const small = (): boolean => {
  const b = og.ui.frameViewport().breakpoint;
  return b === "phonePortrait" || b === "phoneLandscape";
};

/** Room for the extras: 1180×820 is desktop class (1024 wide or more) but has no height to spare. */
const tall = (): boolean => og.ui.frameViewport().height >= 960;

// ── feedback ──────────────────────────────────────────────────────────────

function feedbackScreen(): void {
  const t = og.ui.tick();
  og.ui.notifications("Notes", {
    anchor: "topRight",
    max: small() ? 2 : 4,
    items: [
      { key: "ach", variant: "achievement", title: L("Dragonslayer"), body: L("Defeat the ancient wyrm"), points: "50", rarity: "legendary", at: state.t0, sticky: true },
      { key: "loot", variant: "reward", title: L("Ashbringer"), eyebrow: L("New item"), rarity: "epic", count: 3, at: state.t0 + 6, sticky: true },
      { key: "warn", variant: "alert", tone: "danger", title: L("Connection unstable"), body: L("Reconnecting…"), at: state.t0 + 12, sticky: true },
      { key: "dl", tone: "info", title: L("Downloading update"), progress: 0.62, progressText: "1.2 / 1.9 GB", at: state.t0 + 18, sticky: true },
    ],
  });
  const tk = og.ui.frameTheme().space;
  // A phone held upright opens the column under the toast stack that shares its top and has no room for the rings;
  // held sideways the subtitles take the lower half, so it keeps the badges, one bar and the counters; at a large
  // text size it keeps the badges and bars only (the rest shows on every other surface).
  const crowded = small() && state.textScale >= 1.5;
  const sideways = og.ui.frameViewport().breakpoint === "phoneLandscape";
  const under = og.ui.frameViewport().breakpoint === "phonePortrait" ? (og.ui.bounds("Notes")?.height ?? 0) + 8 : 0;
  og.ui.anchor("Left", { anchor: "topLeft", direction: "column", gap: tk.lg ?? 16, width: { kind: "fixed", px: small() ? 320 : 420 }, offsetY: under }, () => {
    // Room above for the badges, which sit half over their icons' top corners.
    og.ui.row("Badges", { gap: 28, padding: { top: 12, bottom: 0, left: 0, right: 0 } }, () => {
      for (const [k, n] of [["Mail", 7], ["Quests", 120], ["Friends", 0]] as const) {
        og.ui.row(k, { width: { kind: "fixed", px: 56 }, height: { kind: "fixed", px: 56 } }, () => {
          og.ui.icon("Icon", k === "Mail" ? "chat-bubble" : k === "Quests" ? "map" : "group", { size: 40 });
          og.ui.countBadge("Badge", { count: n, max: 99, name: `${n} ${k}` });
        });
      }
      og.ui.row("New", { width: { kind: "fixed", px: 56 }, height: { kind: "fixed", px: 56 } }, () => {
        og.ui.icon("Icon", "gift", { size: 40 });
        og.ui.countBadge("Badge", { kind: "label", text: L("NEW"), tone: "danger", pulse: true });
      });
    });
    og.ui.progressBar("Download", { value: Math.min(1, 0.2 + seconds() * 0.15), buffer: 0.85, label: L("Installing"), showValue: true });
    // Held sideways the subtitles' band starts halfway down: the bars stop at the first, and at a large text size
    // so does the column.
    if (crowded && sideways) return;
    if (!sideways) {
      og.ui.progressBar("Steps", { value: 0.6, style: "steps", steps: 5, label: L("Chapter progress"), showValue: true, tone: "success" });
      // An upright phone at a large text size has room over the captions for two bars, not three.
      if (!crowded) og.ui.progressBar("Busy", { style: "indeterminate", label: L("Matchmaking") });
    }
    if (crowded) return;
    // Large text on a short screen: the rings give their room to the counters.
    if (!small() && (tall() || state.textScale < 1.5)) og.ui.row("Rings", { gap: 24, cross: "center" }, () => {
      og.ui.progressRing("Spin", { size: 48 });
      og.ui.progressRing("Ring", { value: 0.72, size: 72 });
      og.ui.starRating("Stars", { earned: 2, max: 3, size: 40, revealAt: state.t0 + 10 });
    });
    if (tall() && state.textScale < 1.5) og.ui.skeletonGroup("Skeleton", { shape: "row", count: 2 });
    // Overflowing numbers: grouped, compact, letters past T, scientific past letters.
    og.ui.row("Numbers", { gap: 24, wrap: true, width: { kind: "grow" } }, () => {
      og.ui.numberCounter("Gold", did("score") ? 1_284_330 : 12_840, { icon: "coin", label: L("Gold"), format: { notation: "grouped" } });
      og.ui.numberCounter("Idle", 3.4e19 * (1 + seconds()), { label: L("Cookies"), format: { notation: "letters" } });
    });
    if (small()) return;
    og.ui.button("Sword", L("Ashbringer"), {});
    og.ui.tooltipCard("Tip", {
      open: true, target: "Left/Sword", title: L("Ashbringer"), subtitle: L("Legendary two-handed sword"), rarity: "legendary",
      stats: [{ label: L("Damage"), value: "142–198", delta: 24 }, { label: L("Speed"), value: "1.10", delta: -0.1, better: false }, { label: L("Strength"), value: "+48" }],
      compareTitle: L("Equipped: Iron Greatsword"), compareStats: [{ label: L("Damage"), value: "118–174" }, { label: L("Speed"), value: "1.20" }],
      body: L("Deals bonus [link=fire]fire[/link] damage to undead."), links: [{ key: "fire", title: L("Fire"), body: L("Burns for 6 seconds.") }],
      hints: [L("Shift: compare")], delayMs: 0, pinMs: 600,
    });
  });
  // A store card: the ribbon wraps its corner, the card clips the band.
  // At 200 % text the toasts' wrapped titles fill the right column: the store card waits for a larger screen.
  if (tall() && state.textScale < 1.5) og.ui.anchor("Offer", { anchor: "bottomRight", direction: "column", width: { kind: "fixed", px: 280 }, gap: 4, padding: { top: 16, bottom: 16, left: 72, right: 72 } }, () => {
    og.ui.setDraw({ shape: { kind: "rect", radii: [12, 12, 12, 12] }, fills: [{ kind: "solid", color: { token: "panelRaised" } }], strokes: [], effects: [], clip: "shape" });
    og.ui.label("Name", L("Starter bundle"), { step: "lg" });
    og.ui.label("Price", "4.99", { step: "base", color: { token: "inkMuted" } });
    og.ui.ribbon("Sale", { text: L("-50%"), corner: "topLeft", tone: "danger", shownAt: state.t0 + 30 });
  });
  // One celebration at a time on the stage: the level-up follows the victory banner rather than land on it.
  if (did("complete")) og.ui.banner("Wave", { kind: "victory", eyebrow: L("Stage clear"), title: L("Victory"), subtitle: L("3 stars · 12,480 points"), shownAt: state.acts.complete!, durationMs: 1800 });
  if (did("complete")) og.ui.celebration("Party", { kind: "levelUp", at: state.acts.complete! + 180, title: L("Level up!"), value: "24" });
  if (!small() || !did("complete")) og.ui.subtitles("Subs", {
    lines: [
      { key: "a", speaker: L("Captain Reyes"), text: L("Hold this position until the convoy is through."), color: { token: "info" } },
      { key: "b", text: L("distant explosion"), caption: true, direction: -0.8 },
    ],
    settings: { enabled: true, captions: true, speakerNames: true, speakerColors: true, directions: true },
  });
  void t;
}

// ── meters ────────────────────────────────────────────────────────────────

function hp(): number {
  // Each hit takes 18; a heal gives back 30.
  return Math.max(0, Math.min(100, 92 - state.hits * 18 + (did("heal") ? 30 : 0)));
}

function metersScreen(): void {
  const h = hp();
  og.ui.anchor("Bars", { anchor: "topLeft", direction: "column", gap: 18 }, () => {
    og.ui.statBar("Health", { value: h, max: 100, label: L("HP"), showValue: true, segment: 25, shield: 50, shieldMax: 100, shieldCell: 25, shieldTier: "epic", heal: did("heal") ? 0 : 12, anchor: null });
    og.ui.statBar("Low", { value: 24, max: 100, label: L("HP"), showValue: true, anchor: null });
    og.ui.statBar("Critical", { value: 9, max: 100, label: L("HP"), showValue: true, recoverable: 22, anchor: null });
    og.ui.statBar("Empty", { value: 0, max: 100, label: L("HP"), showValue: true, anchor: null });
    if (small()) return;
    og.ui.statBar("Mana", { value: 430, max: 600, kind: "mana", label: L("MP"), showValue: true, style: "pill", anchor: null });
    og.ui.statBar("Xp", { value: 7340, max: 12000, kind: "xp", label: L("XP"), showValue: true, valueText: "61%", thickness: 8, anchor: null });
    og.ui.statBar("Rage", { value: 80, max: 100, kind: "primary", style: "slanted", label: L("Rage"), anchor: null });
  });
  const vw = og.ui.frameViewport().width;
  og.ui.anchor("Fighters", { anchor: "bottom", direction: "row", gap: vw * 0.06, width: { kind: "fixed", px: Math.min(900, vw * 0.5) }, offsetY: small() ? -150 : -40 }, () => {
    og.ui.statBar("P1", { value: h, max: 100, style: "slanted", thickness: 22, width: { kind: "grow" }, recoverable: Math.min(100, h + 14), name: "Ryu", anchor: null });
    og.ui.statBar("P2", { value: 64, max: 100, style: "slanted", thickness: 22, width: { kind: "grow" }, reverse: true, name: "Ken", anchor: null });
  });
  og.ui.orbMeter("HealthOrb", { value: h, max: 100, kind: "health", anchor: "bottomLeft", label: L("Life") });
  og.ui.orbMeter("ManaOrb", { value: 14, max: 100, kind: "mana", anchor: "bottomRight", label: L("Mana") });
  const landscape = og.ui.frameViewport().breakpoint === "phoneLandscape";
  og.ui.anchor("Wheels", { anchor: landscape ? "topRight" : "right", direction: landscape ? "row" : "column", gap: 24, cross: "center" }, () => {
    og.ui.radialMeter("Stamina", { value: Math.max(0, 1.6 - seconds() * 0.3), wheels: 2, hideWhenFull: false, anchor: null });
    og.ui.radialMeter("Tired", { value: 0.35, exhausted: true, anchor: null });
    og.ui.radialMeter("Full", { value: 1, hideWhenFull: false, anchor: null });
  });
}

// ── abilities, auras and casts ────────────────────────────────────────────

function abilitiesScreen(): void {
  const s = seconds();
  const ready = did("ready");
  const art = (k: string): Texture | null => state.art[k] ?? null;
  og.ui.abilityBar("Abilities", {
    slots: [
      { key: "q", name: L("Fireball"), image: art("fireball"), icon: "fire", hotkey: "Q", cooldown: ready ? 0 : Math.max(0, 6.4 - s), cooldownMax: 8, cost: "40" },
      { key: "w", name: L("Blink"), image: art("blink"), icon: "flash", hotkey: "W", maxCharges: 3, charges: 1, cooldown: Math.max(0, 9 - s), cooldownMax: 12, chargeProgress: Math.min(1, s / 12), outOfRange: true },
      { key: "e", name: L("Frost Nova"), image: art("frost-nova"), icon: "sparks", hotkey: "E", maxCharges: 2, charges: 0, cooldown: 3.2, cooldownMax: 10 },
      { key: "r", name: L("Meteor"), image: art("meteor"), icon: "rocket", hotkey: "R", ultimate: Math.min(1, 0.55 + s * 0.05), level: 2, maxLevel: 3 },
      { key: "d", name: L("Shield Wall"), image: art("shield-wall"), icon: "shield", hotkey: "D", active: true },
      { key: "f", name: L("Pyroblast"), image: art("pyroblast"), icon: "fire", hotkey: "F", unaffordable: true, cost: "120" },
      { key: "g", name: L("Hex"), image: art("hex"), icon: "magic-wand", hotkey: "G", disabled: true },
      { key: "h", name: L("Rune of Power"), image: art("rune-of-power"), icon: "star", hotkey: "H", locked: true },
    ],
  });
  og.ui.statusGrid("Auras", {
    anchor: "topRight",
    // A phone held sideways at 200 % text keeps one row of buffs (the rest fold into +n) over its cast bar.
    max: og.ui.frameViewport().breakpoint === "phoneLandscape" && state.textScale >= 1.5 ? 4 : 8,
    effects: [
      { key: "haste", name: L("Haste"), icon: "flash", image: art("blink"), remaining: Math.max(0.1, 14 - s), duration: 20, mine: true },
      { key: "might", name: L("Might"), icon: "fire", image: art("fireball"), remaining: 0 },
      { key: "regen", name: L("Regeneration"), icon: "heart", remaining: 1820, duration: 3600 },
      { key: "ward", name: L("Ward"), icon: "shield", image: art("shield-wall"), remaining: Math.max(0.1, 4.6 - s), duration: 12, stacks: 3 },
      ...Array.from({ length: 7 }, (_, i) => ({ key: `buff${i}`, name: `${L("Blessing")} ${i + 1}`, icon: "star", remaining: 300 + i * 40, duration: 600 })),
      { key: "poison", name: L("Deadly Poison"), icon: "flask", image: art("hex"), debuff: true, type: "poison", stacks: 5, remaining: 8.2, duration: 12 },
      { key: "curse", name: L("Curse of Agony"), icon: "skull", debuff: true, type: "curse", remaining: 2.4, duration: 24 },
      { key: "bleed", name: L("Rend"), icon: "drop", debuff: true, type: "bleed", stacks: 12, remaining: 15, duration: 15 },
    ],
  });
  const interrupted = did("interrupt");
  // Built after the ability bar at the same anchor: they stack above it.
  og.ui.castBar("Cast", { name: L("Pyroblast"), icon: "fire", progress: Math.min(0.97, (s % 3.5) / 3.5), remaining: 3.5 - (s % 3.5), latency: 0.08, state: interrupted ? "interrupted" : "casting", stateAt: state.acts.interrupt ?? 0, anchor: "bottom", width: { kind: "fixed", px: 340 } });
  // A phone held sideways has room over its ability bar for two bars, not three.
  if (og.ui.frameViewport().breakpoint !== "phoneLandscape") og.ui.castBar("Channel", { name: L("Arcane Missiles"), icon: "sparks", channel: true, ticks: 5, progress: Math.min(1, (s % 2.5) / 2.5), remaining: 2.5 - (s % 2.5), anchor: "bottom", width: { kind: "fixed", px: 340 } });
  // At 200 % text the sideways phone's bars reach the aura grid: it keeps the player's own cast.
  if (og.ui.frameViewport().breakpoint !== "phoneLandscape" || state.textScale < 1.5) og.ui.castBar("Boss", { name: L("Cataclysm"), icon: "skull", uninterruptible: true, progress: 0.42, remaining: 4.2, anchor: "bottom", width: { kind: "fixed", px: 340 } });
}

// ── unit, party and raid frames ───────────────────────────────────────────

function framesScreen(): void {
  const s = seconds();
  const h = hp();
  og.ui.unitFrame("Player", { size: small() ? "medium" : "large", name: L("Aria Vale"), level: "60", title: L("Archmage"), health: h * 380, healthMax: 38000, shield: did("hit") ? 0 : 6000, resource: 21400, resourceMax: 30000, leader: true, combat: did("hit"), anchor: "topLeft", effects: [{ key: "a", icon: "flash", remaining: 9, duration: 20 }, { key: "b", icon: "heart", remaining: 0 }] });
  og.ui.unitFrame("Target", {
    name: L("Grull the Devourer"), level: "??", unitClass: "boss", hostile: true, reverse: true, targeted: true, health: 0.64 * 1_250_000, healthMax: 1_250_000,
    castName: L("Earthshatter"), castProgress: Math.min(1, s / 3) % 1, anchor: og.ui.frameViewport().breakpoint === "phonePortrait" ? "topLeft" : small() ? "topRight" : "top", size: small() ? "medium" : "large",
  });
  og.ui.partyFrames("Party", {
    anchor: small() ? "bottomLeft" : "left",
    members: ([
      { key: "tank", name: L("Borin"), level: "60", icon: "shield", health: 82, resource: 40, resourceKind: "primary" },
      { key: "heal", name: L("Seraphine"), level: "60", icon: "plus", health: 34, resource: 12, leader: false, effects: [{ key: "p", debuff: true, type: "magic", remaining: 5, duration: 8 }] },
      { key: "dead", name: L("Kael"), level: "59", health: 0, dead: true },
      { key: "off", name: L("Nyx"), level: "60", health: 100, offline: true },
    ] satisfies UnitFrameDesc[]).slice(0, small() ? 2 : 4),
  });
  if (small()) return;
  og.ui.raidFrames("Raid", {
    anchor: "right",
    members: Array.from({ length: 20 }, (_, i) => ({
      key: `r${i}`,
      name: ["Thrall", "Jaina", "Uther", "Sylvanas", "Anduin", "Varian", "Tyrande", "Malfurion", "Illidan", "Velen"][i % 10]! + (i >= 10 ? "II" : ""),
      health: [100, 76, 34, 12, 90, 0, 55, 100, 67, 21][i % 10]!,
      absorb: i % 4 === 1 ? 20 : 0,
      incomingHeal: i % 5 === 2 ? 25 : 0,
      role: (i % 5 === 0 ? "tank" : i % 5 === 1 ? "healer" : "damage") as "tank" | "healer" | "damage",
      dead: i % 10 === 5,
      offline: i === 17,
      outOfRange: i === 13,
      aggro: i === 0,
      dispel: i === 3 ? "magic" : i === 8 ? "poison" : "",
      selected: i === 2,
      classColor: { color: [{ r: 0.78, g: 0.61, b: 0.43 }, { r: 0.25, g: 0.78, b: 0.92 }, { r: 0.96, g: 0.55, b: 0.73 }, { r: 0.67, g: 0.83, b: 0.45 }, { r: 1, g: 1, b: 1 }][i % 5]! },
    })),
  });
}

// ── shooter ───────────────────────────────────────────────────────────────

function shooterScreen(world: World): void {
  const s = seconds();
  const mag = did("empty") ? 0 : Math.max(0, 30 - state.hits * 4 - Math.floor(s * 2));
  og.ui.crosshair("Aim", { style: "cross", spread: did("hit") && since("hit") < 20 ? 10 : 2, hitAt: lastHit(), hit: did("crit") && (state.acts.crit ?? 0) >= (state.acts.hit ?? 0) ? "headshot" : "hit", dot: true, onTarget: s % 2 < 1 });
  og.ui.ammoCounter("Ammo", { magazine: mag, magazineMax: 30, reserve: 186, weapon: L("R-301 Carbine"), icon: "bolt", fireMode: "AUTO", reloading: did("reload") && since("reload") < 120 ? since("reload") / 120 : null, reloadHint: L("R  Reload") });
  og.ui.damageDirection("Dir", { hits: Object.entries(state.acts).filter(([k]) => k === "hit" || k === "crit").map(([, at], i) => ({ angle: [2.2, -0.9][i % 2]!, at, strength: 0.8 })) });
  og.ui.lowHealthVignette("Low", { health: hp() / 100, hitAt: lastHit() });
  og.ui.compass("Compass", {
    heading: s * 0.25,
    pois: [
      { key: "a", bearing: 0.6, kind: "objective", distance: 238, icon: "star", label: "Objective A" },
      { key: "b", bearing: -1.1, kind: "enemy", distance: 41 },
      { key: "ping", bearing: 3.0, kind: "ping", distance: 120, label: "Ping" },
    ],
  });
  // An upright phone at 200 % text has no room for the map: the top stack (compass, timer, killfeed) reaches its middle.
  if (og.ui.frameViewport().breakpoint !== "phonePortrait" || state.textScale < 1.5) og.ui.minimapView("Map", {
    anchor: og.ui.frameViewport().breakpoint === "phonePortrait" ? "left" : small() ? "bottomLeft" : "topRight",
    zone: L("Kings Canyon"),
    heading: s * 0.25,
    centerX: 0.5 + s * 0.004,
    centerY: 0.5,
    zoom: 0.25,
    markers: [
      { key: "me", kind: "player", x: 0.5 + s * 0.004, y: 0.5 },
      { key: "ally", kind: "ally", x: 0.56, y: 0.44, heading: 1.2, label: "Ally" },
      { key: "foe", kind: "enemy", x: 0.42, y: 0.58, label: "Enemy" },
      { key: "far", kind: "enemy", x: 0.95, y: 0.95 },
      { key: "obj", kind: "objective", x: 0.92, y: 0.1, label: "Objective" },
      { key: "ping", kind: "ping", x: 0.47, y: 0.47, label: "Ping" },
      { key: "loot", kind: "loot", x: 0.53, y: 0.55, rarity: "epic", label: "Loot" },
    ],
  });
  og.ui.hudTimer("Round", { seconds: Math.max(0, 14.6 - s), total: 90, label: L("Round 4") });
  og.ui.killfeed("Feed", { entries: state.kills, max: 5 });
  if (small()) return;
  og.ui.anchor("Score", { anchor: "bottomLeft", direction: "column", gap: 8 }, () => {
    og.ui.scorebug("Bug", {
      home: { abbr: "LAL", score: did("score") ? 101 : 98, timeouts: 3, timeoutsMax: 7, possession: true },
      away: { abbr: "BOS", score: 99, timeouts: 5, timeoutsMax: 7 },
      clock: "2:41", period: "Q4", shotClock: "14", situation: L("Bonus"), anchor: null,
    });
  });
  void world;
}

// ── strategy, rhythm and tactics ─────────────────────────────────────────

function strategyScreen(): void {
  const s = seconds();
  og.ui.resourceBar("Res", {
    items: [
      { key: "min", icon: "gem", name: L("Minerals"), amount: 1250 + s * 12.5, rate: 12.5 },
      { key: "gas", icon: "drop", name: L("Vespene"), amount: 384, rate: -3 },
      { key: "gold", icon: "coin", name: L("Gold"), amount: 4.2e12, rate: 1.8e9, format: { notation: "letters" } },
      { key: "sup", icon: "user", name: L("Supply"), amount: did("place") ? 200 : 186, capacity: 200 },
    ],
  });
  og.ui.objectiveTracker("Quests", {
    // Narrow on a phone, but at a large text size wide enough that its lines do not stack up.
    width: small() && state.textScale < 1.5 ? 220 : 0,
    quests: [
      {
        key: "main", title: L("The Siege of Lordaeron"), icon: "star", focused: true, distance: 312, timer: Math.max(0, 88 - s),
        completedAt: state.acts.complete ?? null,
        steps: [
          { key: "a", text: L("Destroy the siege engines"), count: Math.min(4, 2 + state.hits), target: 4, done: 2 + state.hits >= 4 },
          { key: "b", text: L("Rescue the prisoners"), progress: Math.min(1, 0.35 + s * 0.05) },
          { key: "c", text: L("Find the hidden cache"), optional: true },
          { key: "d", text: L("Protect the bridge"), failed: true },
        ],
      },
      { key: "side", title: L("A Hunter's Errand"), icon: "map-pin", steps: [{ key: "a", text: L("Collect wolf pelts"), count: 7, target: 7, done: true }] },
    ],
  });
  og.ui.turnOrder("Turns", {
    max: small() ? 4 : 8,
    // Smaller portraits on a phone held sideways at a large text size, so the combo fits between it and the resources.
    ...(small() && state.textScale >= 1.5 && og.ui.frameViewport().breakpoint === "phoneLandscape" ? { size: 40 } : {}),
    // On a phone the top is the resources' and the right the tracker's: the order sits at the bottom
    // (held sideways, under the combo at the bottom left).
    ...(og.ui.frameViewport().breakpoint === "phonePortrait" ? { anchor: "bottom" as const } : og.ui.frameViewport().breakpoint === "phoneLandscape" ? { anchor: "bottomLeft" as const } : {}),
    units: [
      { key: "hero", name: "Tidus", current: true, atb: 1 },
      { key: "imp", name: "Imp", team: 1, atb: 0.72 },
      { key: "mage", name: "Lulu", atb: Math.min(1, 0.3 + s * 0.1) },
      { key: "ogre", name: "Ogre", team: 1, delayed: true, atb: 0.2 },
      { key: "kimahri", name: "Kimahri", atb: 0.55 },
      { key: "dead", name: "Wakka", dead: true },
    ],
  });
  // A phone held sideways has no height to stack the combo under the tracker: it takes the left side.
  const sideways = og.ui.frameViewport().breakpoint === "phoneLandscape";
  og.ui.comboCounter("Combo", { count: state.hits >= 2 ? state.hits * 5 : did("break") ? 0 : 47, multiplier: 4, decay: 1 - (s % 2) / 2, ...(sideways ? { anchor: "left" as const } : {}) });
  og.ui.judgement("Judge", { last: did("hit") ? { text: "PERFECT", tier: 0, offsetMs: -12, at: state.acts.hit! } : did("break") ? { text: "MISS", tier: 4, at: state.acts.break! } : null, offsetY: 160 });
  if (did("score")) {
    og.ui.scoreboard("Board", {
      title: L("Bind · Competitive"), subtitle: L("Round 14 · 7–6"),
      columns: [{ key: "k", label: "K" }, { key: "d", label: "D" }, { key: "a", label: "A" }, { key: "s", label: L("Score"), width: 72 }],
      teams: [
        { team: 0, name: L("Attackers"), score: "7", rows: [
          { key: "a", name: "Jett", values: ["21", "9", "4", "6,120"], local: true, ping: 24, mvp: true },
          { key: "b", name: "Sova", values: ["14", "11", "12", "4,410"], ping: 118 },
          { key: "c", name: "Sage", values: ["6", "12", "15", "2,380"], dead: true, ping: 260 },
        ] },
        { team: 1, name: L("Defenders"), score: "6", rows: [
          { key: "d", name: "Cypher", values: ["18", "13", "3", "5,040"], ping: 40 },
          { key: "e", name: "Killjoy", values: ["9", "15", "8", "2,990"], ping: 66 },
        ] },
      ],
    });
  }
}

function racingScreen(): void {
  const s = seconds();
  const rpm = 0.45 + 0.5 * Math.abs(Math.sin(s * 0.8));
  og.ui.speedometer("Speedo", { speed: 118 + rpm * 120, maxSpeed: 320, rpm, gear: rpm > 0.85 ? "5" : "4", boost: 0.66 });
  // On a phone held upright the two dials do not fit side by side: the digital one sits on the left side.
  og.ui.speedometer("Digital", { speed: 287, maxSpeed: 320, rpm: 0.93, gear: "6", style: "digital", anchor: og.ui.frameViewport().breakpoint === "phonePortrait" ? "left" : "bottomLeft" });
  og.ui.hudTimer("Lap", { seconds: 83.4 + s, mode: "stopwatch", label: L("Lap 2/3") });
}

function flightScreen(): void {
  const s = seconds();
  og.ui.flightHud("Flight", { pitch: 0.12 + 0.06 * Math.sin(s), roll: 0.35 * Math.sin(s * 0.6), heading: 1.4 + s * 0.05, speed: 412, altitude: 12450 + s * 30, verticalSpeed: 1800, pathX: 0.02, pathY: -0.04 });
}

// ── world ─────────────────────────────────────────────────────────────────

const UNITS = [
  { key: "grull", name: "Grull the Devourer", x: -1.6, z: -2.2, hp: 0.64, reaction: "hostile" as const, cls: "boss" as const },
  { key: "imp1", name: "Ember Imp", x: 1.8, z: -3.2, hp: 0.3, reaction: "hostile" as const, cls: "normal" as const },
  { key: "imp2", name: "Ember Imp", x: 2.3, z: -3.4, hp: 0.9, reaction: "hostile" as const, cls: "normal" as const },
  { key: "imp3", name: "Ember Imp", x: 2.0, z: -3.0, hp: 1, reaction: "hostile" as const, cls: "normal" as const },
  { key: "borin", name: "Borin", x: -3.4, z: 0.6, hp: 0.82, reaction: "friendly" as const, cls: "normal" as const },
  { key: "merchant", name: "Old Tobias", x: 3.6, z: 0.8, hp: 1, reaction: "neutral" as const, cls: "normal" as const },
  { key: "far", name: "Watchtower Guard", x: -6, z: -26, hp: 1, reaction: "friendly" as const, cls: "normal" as const },
];

function worldScreen(world: World): void {
  const s = seconds();
  const t = og.ui.tick();
  for (const u of UNITS) {
    og.ui.nameplate(`Plate-${u.key}`, {
      target: { entity: null, point: { x: u.x, y: 0, z: u.z } },
      name: L(u.name), title: u.key === "grull" ? L("<The Black Horde>") : "", level: u.key === "grull" ? "??" : "12",
      reaction: u.reaction, unitClass: u.cls, health: u.key === "grull" ? Math.max(0.05, u.hp - state.hits * 0.08) : u.hp,
      targeted: u.key === "grull", castName: u.key === "grull" ? L("Earthshatter") : "", castProgress: (s % 3) / 3,
    });
  }
  og.ui.overheadBars("Bars", {
    bars: Array.from({ length: 40 }, (_, i) => ({ key: `u${i}`, position: { x: -9 + (i % 10) * 2, y: 1.1, z: -10 - Math.floor(i / 10) * 2 }, value: ((i * 37) % 100) / 100, reaction: i % 3 === 0 ? "friendly" as const : "hostile" as const })),
    hideFull: true,
  });
  if (since("hit") === 0 || since("crit") === 0 || since("heal") === 0) {
    const hits: DamageHitDesc[] = [];
    const at = { x: -1.6, y: 1.8, z: -2.2 };
    if (since("hit") === 0) hits.push({ position: at, amount: 1284, seed: 0 }, { position: at, amount: 312, kind: "dot", seed: 1 }, { position: at, kind: "miss", word: "Dodge", seed: 2 }, { position: { x: -3.4, y: 1.8, z: 0.6 }, amount: 540, kind: "self", seed: 3 });
    if (since("crit") === 0) hits.push({ position: at, amount: 48_210, kind: "critical", seed: 4 }, { position: at, amount: 1_250_000, kind: "critical", seed: 5 });
    if (since("heal") === 0) hits.push({ position: { x: -3.4, y: 1.8, z: 0.6 }, amount: 2210, kind: "heal", seed: 6 });
    og.ui.damageNumbers("Dmg", { hits });
  }
  og.ui.selectionRing(world, "Sel", { center: { x: -1.6, y: 0.02, z: -2.2 }, radius: 1.1, reaction: "hostile", selectedAt: state.t0, segments: 0 });
  og.ui.selectionRing(world, "Mine", { center: { x: -3.4, y: 0.02, z: 0.6 }, radius: 0.7, reaction: "self" });
  og.ui.selectionRing(world, "Hover", { center: { x: 3.6, y: 0.02, z: 0.8 }, radius: 0.7, reaction: "neutral", hover: true });
  og.ui.telegraph(world, "Cone", { center: { x: -1.6, y: 0.03, z: -2.2 }, shape: "cone", radius: 5, angle: 1.1, direction: Math.PI * 0.85, progress: Math.min(1, (s % 3) / 2.6) });
  og.ui.telegraph(world, "Donut", { center: { x: 7.5, y: 0.03, z: -10 }, shape: "ring", inner: 1.4, radius: 3, progress: 0.4 });
  og.ui.telegraph(world, "Quake", { center: { x: -6.5, y: 0, z: -5 }, shape: "circle", radius: 2, progress: Math.min(1, (s % 2.4) / 2), ground: { colliders: true } });
  og.ui.telegraph(world, "Charge", { center: { x: 5, y: 0.03, z: -6 }, shape: "line", radius: 6, width: 1.6, direction: -2.4, progress: 0.7 });
  og.ui.pathLine(world, "Path", { points: [{ x: -3.4, y: 0.05, z: 0.6 }, { x: -2, y: 0.05, z: 2.6 }, { x: 0.5, y: 0.05, z: 3 }, { x: 3, y: 0.05, z: 2 }], reach: 5.5, label: L("9 m · 2 AP") });
  og.ui.placementGhost(world, "Ghost", { center: { x: 5.5, y: 0.02, z: 2.5 }, width: 3, depth: 2, rotation: 0.3, cell: 1, valid: !did("place"), label: did("place") ? L("Blocked") : "150", range: did("place") ? null : 4 });
  og.ui.interactionPrompt(world, "Chest", { target: { entity: null, point: { x: 3.6, y: 0, z: 0.8 } }, height: 2.6, action: "interact", verb: L("Trade"), name: L("Old Tobias"), hold: did("say") ? Math.min(1, since("say") / 60) : null });
  og.ui.interactionPrompt(world, "Gate", { target: { entity: null, point: { x: 6.5, y: 0, z: -1 } }, verb: L("Open"), name: L("Iron Gate"), disabled: true, reason: L("Locked") });
  og.ui.waypoint("Obj", { target: { entity: null, point: { x: -10, y: 2, z: -30 } }, kind: "objective", label: L("Defend the gate") });
  og.ui.waypoint("Behind", { target: { entity: null, point: { x: 18, y: 0, z: 22 } }, kind: "enemy", label: L("Enemy spotted") });
  og.ui.waypoint("Left", { target: { entity: null, point: { x: -40, y: 0, z: -4 } }, kind: "marker", label: L("Camp") });
  if (did("ping")) og.ui.waypoint("Ping", { target: { entity: null, point: { x: 0.5, y: 0, z: -6 } }, kind: "ping", label: L("Loot here"), by: "Wraith", at: state.acts.ping! });
  og.ui.lootLabel("Loot1", { target: { entity: null, point: { x: 0.2, y: 0, z: 1.2 } }, name: L("Doombringer"), rarity: "legendary" });
  og.ui.lootLabel("Loot2", { target: { entity: null, point: { x: 0.6, y: 0, z: 1.0 } }, name: L("Frostweave Robe"), rarity: "epic" });
  og.ui.lootLabel("Loot3", { target: { entity: null, point: { x: 0.4, y: 0, z: 1.4 } }, name: L("Gold"), rarity: "common", count: 87, icon: "coin" });
  og.ui.lootLabel("Loot4", { target: { entity: null, point: { x: 0.0, y: 0, z: 1.1 } }, name: L("Linen Scraps"), rarity: "uncommon", count: 3 });
  og.ui.speechBubble("Say", { target: { entity: null, point: { x: 3.6, y: 0, z: 0.8 } }, height: 3.4, speaker: L("Old Tobias"), text: L("Fine wares, traveller! Nothing in the realm is cheaper."), at: state.acts.say ?? state.t0 + 20, durationMs: 60000 });
  og.ui.speechBubble("Shout", { target: { entity: null, point: { x: -1.6, y: 0, z: -2.2 } }, height: 4.4, text: L("YOU WILL KNEEL!"), shout: true, at: state.t0 + 40, charsPerSecond: 0, durationMs: 60000 });
  void t;
}

// ── the stage ─────────────────────────────────────────────────────────────

function applyStyle(world: World): void {
  const loaded = state.style === "prototype" ? null : og.load.style(world, `@styles/${state.style}/style.ogt`);
  const style = loaded && loaded.ok ? loaded.value : null;
  og.ui.configure(world, {
    font: state.fonts.body && !style ? (state.fonts.body as never) : null,
    displayFont: state.fonts.body && !style ? (state.fonts.body as never) : null,
    style,
    icons: state.fonts.icons ? { font: state.fonts.icons as never } : null,
    textScale: state.textScale,
  });
}

const Stage = defineSystem("Stage", { reads: [og.Camera] }, (_q, _t, world) => {
  for (const run of state.queue.splice(0)) run();
  switch (state.screen) {
    case "feedback": feedbackScreen(); break;
    case "meters": metersScreen(); break;
    case "abilities": abilitiesScreen(); break;
    case "frames": framesScreen(); break;
    case "shooter": shooterScreen(world); break;
    case "strategy": strategyScreen(); break;
    case "racing": racingScreen(); break;
    case "flight": flightScreen(); break;
    case "world": worldScreen(world); break;
  }
});

/** Every act forgotten, the screen's clock started on this frame. */
function restart(): void {
  state.acts = {};
  state.hits = 0;
  state.kills = [];
  state.t0 = og.ui.tick();
}

/** Something happens, on this frame (called from the stage, never from a command). */
function act(name: string): void {
  const t = og.ui.tick();
  state.acts[name] = t;
  if (name === "hit" || name === "crit") {
    state.hits += 1;
    const n = state.kills.length;
    state.kills = [...state.kills, { key: `k${n}`, killer: n % 2 ? "Wraith" : "Bloodhound", killerTeam: n % 2, victim: ["Octane", "Lifeline", "Caustic", "Mirage", "Pathfinder", "Gibraltar"][n % 6]!, victimTeam: 1 - (n % 2), weapon: "bolt", weaponText: "R-99", headshot: name === "crit", local: n % 3 === 0, at: t }];
  }
}

export default og.game({
  tick: 60,
  seed: 1,
  systems: [Stage],
  render: { antialias: "msaa4" },
  start(world) {
    // The prompt's glyph follows the device used last: F, a pad's X (Square on a PlayStation pad).
    og.input.defineAction(world, { name: "interact", label: "Interact", bindings: [{ control: "KeyF" }, { control: "pad.x" }] });
    const camera = world.resource(og.Camera);
    if (camera.ok) {
      const eye = { x: 0, y: 6.5, z: 9.5 };
      camera.value.position = eye;
      camera.value.quaternion = og.transform.lookAt(eye, { x: 0, y: 0, z: -2.5 });
      camera.value.fov = 50;
      camera.value.background = { r: 0.16, g: 0.2, b: 0.26 };
    }
    const settings = world.resource(og.RenderSettings);
    if (settings.ok) settings.value.sky = true;
    // A lit field with a few units: what the world tags and ground shapes sit on.
    const ground = og.render.planeGeometry(world, { width: 80, height: 80 });
    const grass = og.render.standardMaterial(world, { color: { r: 0.27, g: 0.36, b: 0.22 }, roughness: 0.95 });
    const plane = world.spawn({ name: "ground", quaternion: og.transform.fromAxisAngle({ x: 1, y: 0, z: 0 }, -Math.PI / 2) });
    if (ground.ok && grass.ok && plane.ok) world.insert(plane.value, og.Mesh, { geometry: ground.value, material: grass.value, receiveShadow: true });
    const box = og.render.boxGeometry(world, { width: 0.8, height: 1.6, depth: 0.8 });
    for (const u of UNITS) {
      const tint = u.reaction === "hostile" ? { r: 0.62, g: 0.2, b: 0.16 } : u.reaction === "neutral" ? { r: 0.7, g: 0.55, b: 0.2 } : { r: 0.25, g: 0.45, b: 0.7 };
      const mat = og.render.standardMaterial(world, { color: tint, roughness: 0.6 });
      const e = world.spawn({ name: u.key, position: { x: u.x, y: 0.8, z: u.z } });
      if (box.ok && mat.ok && e.ok) world.insert(e.value, og.Mesh, { geometry: box.value, material: mat.value, castShadow: true });
    }
    // A ramp, solid: the quake telegraph lies on it (GroundDesc.colliders), not under or over it.
    const rampGeo = og.render.boxGeometry(world, { width: 5, height: 0.6, depth: 5 });
    const stone = og.render.standardMaterial(world, { color: { r: 0.42, g: 0.4, b: 0.36 }, roughness: 0.9 });
    const ramp = world.spawn({ name: "ramp", position: { x: -6.5, y: 0.5, z: -5 }, quaternion: og.transform.fromAxisAngle({ x: 1, y: 0, z: 0 }, 0.32) });
    if (rampGeo.ok && stone.ok && ramp.ok) {
      world.insert(ramp.value, og.Mesh, { geometry: rampGeo.value, material: stone.value, castShadow: true, receiveShadow: true });
      world.insert(ramp.value, og.Collider, { shape: SHAPE_BOX, size: { x: 5, y: 0.6, z: 5 } });
    }
    const sun = world.spawn({ name: "sun", quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: 0.4, y: -0.7, z: -0.5 }) });
    if (sun.ok) world.insert(sun.value, og.DirectionalLight, { intensity: 3, castShadow: true });
    for (const l of ["en", "ar"]) {
      const loaded = og.i18n.load(world, `strings/${l}.ogt`);
      if (!loaded.ok) throw new Error(loaded.reason);
    }
    og.i18n.setLocale(world, "en");
    for (const k of ["fireball", "blink", "frost-nova", "meteor", "shield-wall", "pyroblast", "hex", "rune-of-power"]) {
      const tex = og.load.texture(world, `abilities/${k}.jpg`);
      if (tex.ok) state.art[k] = tex.value;
    }
    const body = og.load.font(world, "fonts/Inter-Display.ttf");
    const icons = og.load.font(world, "icons/iconoir.ogt");
    state.fonts = { body: body.ok ? (body.value as unknown as number) : 0, icons: icons.ok ? (icons.value as unknown as number) : 0 };
    applyStyle(world);
  },
  commands: {
    show: (_world, args) => {
      state.queue.push(() => {
        state.screen = (SCREENS as readonly string[]).includes(args[0] ?? "") ? (args[0] as Screen) : "meters";
        restart();
      });
    },
    viewport: (world, args) => og.ui.previewViewport(world, VIEWPORTS[args[0] ?? "desktop"] ?? null),
    lang: (world, args) => {
      state.locale = args[0] === "ar" ? "ar" : args[0] === "pseudo" ? "pseudo" : "en";
      og.i18n.setLocale(world, state.locale === "ar" ? "ar" : "en");
    },
    style: (world, args) => {
      state.style = (STYLES as readonly string[]).includes(args[0] ?? "") ? (args[0] as (typeof STYLES)[number]) : "prototype";
      applyStyle(world);
    },
    text: (world, args) => {
      state.textScale = Math.min(2, Math.max(1, Number(args[0]) / 100 || 1));
      applyStyle(world);
    },
    act: (_world, args) => {
      state.queue.push(() => act(args[0] ?? "hit"));
    },
    reset: () => {
      state.queue.push(restart);
    },
  },
  commandHelp: {
    show: { usage: `<${SCREENS.join("|")}>`, doc: "Open a screen of components; its acts and clock start again." },
    viewport: { usage: "<desktop|ultrawide|tablet|phone|landscape>", doc: "Lay the UI out for that surface (og.ui.previewViewport)." },
    lang: { usage: "<en|ar|pseudo>", doc: "English, Arabic (right to left) or the pseudo-locale (40 % longer, accented, bracketed)." },
    style: { usage: `<${STYLES.join("|")}>`, doc: "Dress the components in a built-in style." },
    text: { usage: "<percent>", doc: "The player's text size, 100 to 200 (og.ui.configure's textScale)." },
    act: { usage: "<hit|crit|heal|ready|empty|reload|interrupt|ping|say|complete|combo|score|break|place>", doc: "Something happens: a hit, a crit, a ping, a cooldown ending, a magazine running dry." },
    reset: { usage: "", doc: "Forget every act and start the screen's clock again." },
  },
  facts: () => ({ screen: state.screen, locale: state.locale, style: state.style, hits: state.hits }),
});
