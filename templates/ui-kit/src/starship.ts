// The starship screens (holo's five, as kit screens styled by the active
// pack; scifi is their showcase): a ship's status, the turn order, a hand
// of ability cards, the sector map and the hangar. Each is built from
// og.ui's components over the kit's starship art (assets/textures/art:
// ships/, space/, scenes/vanguard-orbit.jpg; SOURCE.md), its words in
// assets/strings. Ship and place names are the game's own proper nouns
// (AFS Valiant, Aurora Fleet, Helios Reach) and stay as they are in every
// locale. `pat:ship:<key>:<value>` sets a screen's state so the playtests
// (tests/starship*.og.ts) can draw it.

import { og } from "@origingame/engine";
import type { Size, World } from "@origingame/engine";
import { L } from "./patterns.ts";

const grow: Size = { kind: "grow" };
const px = (n: number): Size => ({ kind: "fixed", px: Math.max(0, Math.round(n)) });
const all = (n: number) => ({ left: n, top: n, right: n, bottom: n });

// ── the art ─────────────────────────────────────────────────────────────

const ART_FILES = [
  "ships/valiant.png", "ships/aegis.png", "ships/kestrel.png", "ships/sentinel.png", "ships/vanguard.png", "ships/carrier.png", "ships/module.png",
  "ships/dreadnought.png", "ships/ravager.png", "ships/marauder.png", "ships/skirmisher.png",
  "space/kinetic-strike.jpg", "space/deflector-screen.jpg", "space/squadron-launch.jpg", "space/repair-drones.jpg", "space/gravity-well.jpg", "space/overcharge.jpg",
  "scenes/vanguard-orbit.jpg",
] as const;
type ArtFile = (typeof ART_FILES)[number];
const ART: Partial<Record<ArtFile, number>> = {};

/** Load the starship art (the game's start). */
export function loadStarshipArt(world: World): void {
  for (const f of ART_FILES) {
    const r = og.load.texture(world, `textures/art/${f}`);
    if (r.ok) ART[f] = r.value as unknown as number;
  }
}

/** A texture of the starship art, or null: the slot then draws the style's placeholder. */
const art = (f: ArtFile): never | null => (ART[f] ? (ART[f] as never) : null);

// ── the screens' frame ──────────────────────────────────────────────────

/** The surface, the style and the room the kit's Gallery button takes. */
function frame() {
  const t = og.ui.frameTheme();
  const vp = og.ui.frameViewport();
  const sa = vp.safeArea;
  const small = vp.breakpoint === "phonePortrait" || vp.breakpoint === "phoneLandscape";
  // A 1080p desktop reads from further away: everything steps up.
  const big = !small && vp.height >= 1000;
  const k = big ? 1.35 : 1;
  const ts = Math.max(1, t.textScale ?? 1);
  const back = og.ui.bounds("Back");
  // On a phone the Gallery button takes the top end corner (the screens start under it); on a desktop the bottom start one.
  const top = small ? (back ? back.y + back.height - sa.top : 60) + t.space.sm : 0;
  const foot = small ? 0 : (back?.height ?? 40) + t.space.md;
  const footW = small ? 0 : (back?.width ?? 120) + t.space.md;
  const m = small ? t.space.sm : t.space.lg;
  const line = (step: string, fallback: number) => t.typeRamp[step]?.lineHeight ?? fallback * ts;
  return { t, vp, sa, small, big, k, ts, top, foot, footW, m, line, innerW: vp.width - sa.left - sa.right, innerH: vp.height - sa.top - sa.bottom };
}

/** A section's label: small caps in the display face, inkMuted. */
function eyebrow(id: string, text: string): void {
  og.ui.label(id, text.toLocaleUpperCase(), { step: "caption", font: "display", color: { token: "inkMuted" }, nowrap: true, ellipsis: true, width: grow });
}

/** The screens' state (a gallery's, never saved). */
const st = { card: 0, cls: 0, zoom: 1 };

// ── ship status ─────────────────────────────────────────────────────────

/**
 * A ship's status (a tactics game's selected unit): its name at the
 * heading step over its class, its silhouette, then Hull (a bar and its
 * percent), Shields (discrete pips) and Reactor (its mark and a bar): three
 * kinds of bar, kept apart on purpose.
 */
export function shipStatusScreen(): void {
  const f = frame();
  const { t, small, k, ts, m } = f;
  const w = small ? f.innerW - 2 * m : Math.min(f.innerW - 2 * m, 420 * k * Math.min(ts, 1.7));
  // At a large text size on a phone each system's name stands over its bar, so neither is cut.
  const stack = small && ts >= 1.5;
  const labelW = Math.round(92 * k * Math.min(ts, 2));
  const valueW = Math.round(56 * k * Math.min(ts, 2));
  og.ui.anchor("ShipStatus", { anchor: "topLeft", margin: m, offsetY: f.top, width: px(w), direction: "column", gap: t.space.md, padding: all(small ? t.space.md : t.space.lg), background: { token: "hud" }, radius: t.radius["panel"] ?? 12, frame: "hud" }, () => {
    og.ui.column("Head", { width: grow, gap: 2 }, () => {
      og.ui.label("Name", "AFS Valiant", { step: small ? "xl2" : f.big ? "xl4" : "xl3", heading: true, font: "display", nowrap: true, ellipsis: true, width: grow });
      og.ui.label("Class", L("ship.class"), { step: small ? "sm" : "base", color: { token: "inkMuted" }, nowrap: true, ellipsis: true, width: grow });
    });
    og.ui.artSlot("Ship", { texture: art("ships/valiant.png"), kind: "unitPortrait", aspect: 400 / 150, fit: "contain", mask: "none", width: grow, label: "AFS Valiant" });
    const row = (id: string, label: string, bar: () => void, value: () => void) => {
      if (stack) {
        og.ui.column(id, { width: grow, gap: t.space.xs }, () => {
          og.ui.row("Head", { width: grow, gap: t.space.sm, cross: "center" }, () => {
            og.ui.label("Label", label, { step: "base", nowrap: true, ellipsis: true, width: grow });
            og.ui.row("Value", { align: "end", cross: "center", gap: 2 }, value);
          });
          og.ui.row("Track", { width: grow, gap: t.space.sm, cross: "center" }, bar);
        });
        return;
      }
      og.ui.row(id, { width: grow, gap: t.space.sm, cross: "center" }, () => {
        og.ui.label("Label", label, { step: "base", nowrap: true, ellipsis: true, width: px(labelW) });
        bar();
        og.ui.row("Value", { width: px(valueW), align: "end", cross: "center", gap: 2 }, value);
      });
    };
    og.ui.column("Systems", { width: grow, gap: t.space.sm }, () => {
      row("Hull", L("ship.hull"), () => og.ui.statBar("Bar", { value: 78, max: 100, kind: "health", ghost: false, width: grow, thickness: Math.round(12 * k), name: L("ship.hull") }), () => {
        og.ui.label("N", "78", { step: "lg", font: "mono", nowrap: true });
        og.ui.label("Unit", "%", { step: "sm", color: { token: "inkMuted" }, nowrap: true });
      });
      // Shields as discrete pips, three of four; the reactor's mark before its slimmer bar.
      row("Shields", L("ship.shields"), () => og.ui.statBar("Bar", { value: 3, max: 4, style: "pips", color: { token: "info" }, width: grow, thickness: Math.round(12 * k), valueText: "3 / 4", name: L("ship.shields") }), () => {});
      row("Reactor", L("ship.reactor"), () => og.ui.statBar("Bar", { value: 64, max: 100, kind: "mana", icon: "bolt", ghost: false, width: grow, thickness: Math.round(8 * k), name: L("ship.reactor") }), () => {});
    });
  });
}

// ── turn order ──────────────────────────────────────────────────────────

const UNITS: ReadonlyArray<{ name: string; art: ArtFile; hostile: boolean }> = [
  { name: "Valiant", art: "ships/valiant.png", hostile: false },
  { name: "Aegis", art: "ships/aegis.png", hostile: false },
  { name: "Kestrel", art: "ships/kestrel.png", hostile: false },
  { name: "Dreadnought", art: "ships/dreadnought.png", hostile: true },
  { name: "Ravager", art: "ships/ravager.png", hostile: true },
  { name: "Marauder", art: "ships/marauder.png", hostile: true },
  { name: "Skirmisher", art: "ships/skirmisher.png", hostile: true },
  { name: "Sentinel", art: "ships/sentinel.png", hostile: false },
];

/**
 * The turn order (a tactics game's initiative): og.ui.turnOrder's list,
 * portrait rows read top to bottom with no numbers, the acting ship first on
 * the selected row's rule, enemies marked by a danger stripe on the start
 * edge (mirrored right to left with the row).
 */
export function turnOrderScreen(): void {
  const f = frame();
  const { t, small, k, ts, m } = f;
  const w = small ? f.innerW - 2 * m : Math.min(f.innerW - 2 * m, 320 * k * Math.min(ts, 1.6));
  const pad = small ? t.space.md : t.space.lg;
  og.ui.anchor("TurnOrder", { anchor: small ? "topLeft" : "topRight", margin: m, offsetY: f.top, width: px(w), direction: "column", gap: t.space.sm, padding: all(pad), background: { token: "hud" }, radius: t.radius["panel"] ?? 12, frame: "hud" }, () => {
    og.ui.label("Title", L("turn.title"), { step: "lg", heading: true, font: "display", nowrap: true, ellipsis: true, width: grow });
    og.ui.turnOrder("Units", {
      style: "list",
      units: UNITS.map((u, i) => ({ key: u.name.toLowerCase(), name: u.name, portrait: art(u.art), team: u.hostile ? 1 : 0, current: i === 0 })),
      size: Math.round(36 * k),
      nowLabel: L("turn.acting"),
      hostileLabel: L("turn.hostile"),
      anchor: null,
    });
  });
}

// ── ability cards ───────────────────────────────────────────────────────

const ABILITIES: ReadonlyArray<{ key: string; art: ArtFile; cost: number; hotkey: string }> = [
  { key: "kinetic", art: "space/kinetic-strike.jpg", cost: 2, hotkey: "Q" },
  { key: "deflector", art: "space/deflector-screen.jpg", cost: 3, hotkey: "W" },
  { key: "squadron", art: "space/squadron-launch.jpg", cost: 4, hotkey: "E" },
  { key: "repair", art: "space/repair-drones.jpg", cost: 3, hotkey: "R" },
  { key: "gravity", art: "space/gravity-well.jpg", cost: 5, hotkey: "T" },
  { key: "overcharge", art: "space/overcharge.jpg", cost: 4, hotkey: "Y" },
];

/** One ability card: its number at the top start, its art, its name in caps, a line of effect and its cost in a disc; its hotkey under it. */
function abilityCard(i: number, cw: number, keys: boolean): void {
  const { t } = frame();
  const a = ABILITIES[i]!;
  og.ui.column(`Slot${i}`, { width: px(cw), gap: t.space.xs, cross: "center" }, () => {
    const r = og.ui.card("Card", { interactive: true, selected: st.card === i, width: px(cw), padding: t.space.sm, media: { texture: art(a.art) ?? 0, aspect: 4 / 3 } as never, number: `${i + 1}`, cost: `${a.cost}` }, () => {
      og.ui.label("Name", L(`ab.${a.key}`).toLocaleUpperCase(), { step: "caption", font: "display", maxLines: 2, ellipsis: true, width: grow });
      og.ui.label("Effect", L(`ab.${a.key}.text`), { step: "sm", color: { token: "inkMuted" }, maxLines: 2, ellipsis: true, width: grow });
    });
    if (r.clicked) st.card = st.card === i ? -1 : i;
    // The hotkey: a quiet cap under the card, never a boxed key on its art (a finger has none).
    if (keys) og.ui.label("Key", a.hotkey, { step: "caption", font: "display", color: { token: "inkMuted" }, nowrap: true });
  });
}

/**
 * A hand of six ability cards (a tactics game's action bar) and End turn,
 * the screen's one filled button, at the bottom end. On a phone the hand
 * is whole rows of a scrolling grid over the button.
 */
export function abilityCardsScreen(): void {
  const f = frame();
  const { t, vp, small, k, ts, m } = f;
  const gap = small ? t.space.sm : t.space.md;
  const keys = !vp.coarsePointer;
  const endW = small ? 0 : Math.round(200 * k * Math.min(ts, 1.6));
  // A card is never narrower than its name in caps and its cost need: where six do not fit in a row, whole rows scroll.
  const minW = Math.round((small ? 112 : 136 * k) * ts);
  const room = f.innerW - 2 * m - f.footW - endW - gap;
  const fit = Math.floor((room + gap) / (minW + gap));
  if (fit >= ABILITIES.length) {
    const cw = Math.floor(Math.min(196 * k * Math.min(ts, 1.4), (room - 5 * gap) / 6));
    og.ui.anchor("Abilities", { anchor: "bottomLeft", margin: m, width: px(f.innerW - 2 * m), direction: "row", gap, cross: "end", padding: { left: f.footW, top: 0, right: 0, bottom: 0 } }, () => {
      og.ui.row("Hand", { width: grow, gap, align: "center", cross: "end" }, () => {
        for (let i = 0; i < ABILITIES.length; i++) abilityCard(i, cw, keys);
      });
      og.ui.semantics({ role: "list", name: L("ab.title") });
      if (og.ui.heroButton("EndTurn", L("ab.endTurn"), { tone: "primary", size: "large", width: px(endW), key: keys ? "Space" : "" }).clicked) st.card = -1;
    });
    return;
  }
  const inner = f.innerW - 2 * m;
  const cols = Math.max(1, Math.min(ABILITIES.length, Math.floor((inner + gap) / (minW + gap))));
  const cw = Math.floor((inner - (cols - 1) * gap) / cols);
  // On a desktop the Gallery button keeps the bottom start corner: End turn stands at the end of the foot beside it.
  og.ui.anchor("Abilities", { anchor: "topLeft", margin: 0, offsetY: f.top, width: px(f.innerW), height: px(f.innerH - f.top + (small ? f.sa.bottom : 0)), direction: "column", gap, cross: "end", padding: { left: m, top: small ? 0 : m, right: m, bottom: small ? f.sa.bottom + m : m } }, () => {
    og.ui.scrollArea("Hand", { width: grow, height: grow, gap, fade: 48, fadeInto: "shadow", label: L("ab.title") }, () => {
      for (let r = 0; r * cols < ABILITIES.length; r++) {
        og.ui.row(`Row${r}`, { width: grow, gap, cross: "start" }, () => {
          for (let i = r * cols; i < Math.min(ABILITIES.length, (r + 1) * cols); i++) abilityCard(i, cw, keys);
        });
      }
    });
    if (og.ui.heroButton("EndTurn", L("ab.endTurn"), { tone: "primary", size: "large", width: small ? grow : px(endW), key: keys ? "Space" : "" }).clicked) st.card = -1;
  });
}

// ── sector map ──────────────────────────────────────────────────────────

/** The sector's systems, in fractions of the field. */
const NODES: ReadonlyArray<{ x: number; y: number; hostile: boolean; current?: boolean }> = [
  { x: 0.12, y: 0.62, hostile: false }, { x: 0.2, y: 0.36, hostile: false }, { x: 0.27, y: 0.5, hostile: false },
  { x: 0.36, y: 0.3, hostile: false }, { x: 0.33, y: 0.68, hostile: false }, { x: 0.45, y: 0.52, hostile: false, current: true },
  { x: 0.42, y: 0.8, hostile: false },
  { x: 0.6, y: 0.24, hostile: true }, { x: 0.72, y: 0.4, hostile: true }, { x: 0.84, y: 0.3, hostile: true },
  { x: 0.66, y: 0.6, hostile: true }, { x: 0.8, y: 0.66, hostile: true },
];
/** The lanes between systems (indices into NODES). */
const LANES: ReadonlyArray<[number, number]> = [[0, 2], [1, 2], [1, 3], [2, 5], [3, 5], [4, 5], [4, 6], [5, 10], [7, 8], [8, 9], [8, 10], [10, 11]];

/**
 * The sector map: og.ui.nodeGraph's systems, allies as triangles and foes as
 * diamonds each in its side's colour (GraphNode.team), the current one
 * ringed (its state), and a caption plate at the foot naming the sector.
 */
export function sectorMapScreen(): void {
  const f = frame();
  const { t, small, k, m } = f;
  const pad = small ? t.space.md : t.space.lg;
  const w = small ? f.innerW - 2 * m : Math.min(f.innerW - 2 * m - 2 * f.footW, 920 * k);
  const fw = w - 2 * pad;
  const head = f.line(small ? "xl" : "xl2", 32);
  const plate = f.line(small ? "base" : "lg", 26) + 2 * t.space.xs;
  // On a desktop the panel stands clear of the Gallery button below it (centred, so the same room above).
  const room = small ? f.innerH - f.top - 2 * m : f.innerH - 2 * (f.foot + m);
  const fh = Math.max(200, Math.min(small ? fw * 1.25 : fw * 0.6, room - 2 * pad - head - plate - 2 * t.space.md));
  // The systems in graph units shaped like the graph's window (its zoom bar, and the room nodeGraph keeps round its
  // nodes and their captions, taken off), so at its fit the sector fills the window instead of a band of it.
  const node = Math.round((small ? 20 : 24) * k);
  const aspect = Math.max(0.3, (fh - (small ? 56 : 52) - 2 * node - 64 - 36 * f.ts) / Math.max(1, fw - node - 2 * t.space.sm));
  const xs = NODES.map((n) => n.x);
  const ys = NODES.map((n) => n.y);
  const x0 = Math.min(...xs);
  const y0 = Math.min(...ys);
  const sx = 1000 / (Math.max(...xs) - x0);
  const sy = (1000 * aspect) / (Math.max(...ys) - y0);
  og.ui.anchor("Sector", { anchor: small ? "topLeft" : "center", margin: small ? m : 0, offsetY: f.top, width: px(w), direction: "column", gap: t.space.md, padding: all(pad), background: { token: "hud" }, radius: t.radius["panel"] ?? 12, frame: "hud" }, () => {
    og.ui.label("Title", L("sector.title"), { step: small ? "xl" : "xl2", heading: true, font: "display", nowrap: true, ellipsis: true, width: grow });
    const r = og.ui.nodeGraph("Map", {
      nodes: NODES.map((n, i) => ({
        id: `s${i}`,
        x: Math.round((n.x - x0) * sx),
        y: Math.round((n.y - y0) * sy),
        // No caption under a system: its label is what a screen reader hears (its side, and whether the fleet is there).
        label: `${L(n.hostile ? "sector.hostile" : "sector.allied")}${n.current ? `, ${L("sector.current")}` : ""}`,
        shape: n.hostile ? "diamond" : "triangle",
        team: n.hostile ? 1 : 0,
        state: n.current ? "current" : "available",
      })),
      links: LANES.map(([a, b]) => ({ from: `s${a}`, to: `s${b}` })),
      curved: false,
      labels: false,
      zoom: st.zoom,
      nodeSize: node,
      width: grow,
      height: px(fh),
      label: L("sector.title"),
    });
    st.zoom = r.zoom;
    // The caption plate at the foot: the sector's name.
    og.ui.row("Foot", { width: grow, align: "end" }, () => {
      og.ui.row("Caption", { gap: t.space.xs, cross: "center", padding: { left: t.space.md, top: t.space.xs, right: t.space.md, bottom: t.space.xs }, background: { token: "panelRaised" }, radius: t.radius["control"] ?? 6 }, () => {
        og.ui.label("Text", "K-317 · Helios Reach", { step: small ? "base" : "lg", font: "display", nowrap: true });
      });
    });
  });
}

// ── hangar ──────────────────────────────────────────────────────────────

const CLASSES: ReadonlyArray<{ key: string; art: ArtFile; hero: string; heroArt: ArtFile; scene: ArtFile | null; stats: [number, number, number, number]; upgrade: number; price: number }> = [
  { key: "frigates", art: "ships/kestrel.png", hero: "Vanguard", heroArt: "ships/vanguard.png", scene: "scenes/vanguard-orbit.jpg", stats: [68, 52, 91, 56], upgrade: 4, price: 72000 },
  { key: "cruisers", art: "ships/aegis.png", hero: "Aegis", heroArt: "ships/aegis.png", scene: null, stats: [84, 76, 58, 72], upgrade: 3, price: 118000 },
  { key: "carriers", art: "ships/carrier.png", hero: "Halcyon", heroArt: "ships/carrier.png", scene: null, stats: [92, 70, 34, 48], upgrade: 2, price: 164000 },
  { key: "modules", art: "ships/module.png", hero: "Bastion", heroArt: "ships/module.png", scene: null, stats: [60, 88, 12, 30], upgrade: 5, price: 38000 },
];

/** The hero's stats as bars with tabular values, and its upgrade pips. */
function heroStats(cls: (typeof CLASSES)[number], stacked: boolean): void {
  const f = frame();
  const { t, k } = f;
  og.ui.panel("Stats", { row: !stacked, width: grow, gap: stacked ? t.space.md : t.space.xl, padding: all(f.small ? t.space.md : t.space.lg), cross: stacked ? "stretch" : "center" }, () => {
    og.ui.statCompare("Bars", {
      rows: (["hull", "shields", "speed", "firepower"] as const).map((s, i) => ({ label: L(`hangar.${s}`), current: cls.stats[i]!, max: 100, icon: ["heart", "shield", "play", "fire"][i]! })),
      width: grow,
      label: L("hangar.stats"),
    });
    og.ui.column("Upgrade", { width: stacked ? grow : px(200 * k * Math.min(f.ts, 1.5)), gap: t.space.xs }, () => {
      eyebrow("Label", L("hangar.upgrade"));
      og.ui.row("Pips", { width: grow, gap: t.space.sm, cross: "center" }, () => {
        og.ui.statBar("Bar", { value: cls.upgrade, max: 6, style: "pips", kind: "primary", width: grow, thickness: Math.round(10 * k), valueText: `${cls.upgrade} / 6`, name: L("hangar.upgrade") });
        og.ui.label("Value", `${cls.upgrade} / 6`, { step: "lg", font: "mono", nowrap: true });
      });
    });
  });
}

/** The detail column: the ship's art, name, flavour, price, Commission (primary) and Compare. */
function heroDetail(world: World, cls: (typeof CLASSES)[number]): void {
  const f = frame();
  const { t, small } = f;
  og.ui.panel("Detail", { width: grow, gap: t.space.sm, padding: all(small ? t.space.md : t.space.lg) }, () => {
    if (cls.scene) og.ui.artSlot("Art", { texture: art(cls.scene), kind: "keyArt", aspect: 3 / 2, width: grow, label: cls.hero });
    else og.ui.artSlot("Art", { texture: art(cls.heroArt), kind: "unitPortrait", aspect: 3 / 2, fit: "contain", width: grow, label: cls.hero });
    og.ui.column("Name", { width: grow, gap: 2 }, () => {
      og.ui.label("Title", cls.hero, { step: small ? "xl2" : "xl3", heading: true, font: "display", nowrap: true, ellipsis: true, width: grow });
      og.ui.label("Class", L(`hangar.${cls.key}.class`), { step: "sm", color: { token: "inkMuted" }, nowrap: true, ellipsis: true, width: grow });
    });
    og.ui.label("Flavour", L(`hangar.${cls.key}.flavour`), { step: "base", width: grow, maxLines: 5, ellipsis: true });
    og.ui.row("Price", { width: grow, gap: t.space.xs, cross: "center" }, () => {
      og.ui.icon("Icon", "gem", { size: Math.round(f.line("xl", 28)), tint: { token: "gold" }, label: "" });
      og.ui.label("Value", og.ui.formatNumber(cls.price), { step: "xl2", font: "mono", nowrap: true });
      og.ui.label("Unit", L("hangar.alloy"), { step: "sm", color: { token: "inkMuted" }, nowrap: true });
    });
    if (og.ui.button("Commission", L("hangar.commission"), { variant: "primary", size: "large", width: grow }).clicked) og.ui.toast(world, L("hangar.commissioned", { name: cls.hero }), { tone: "success" });
    og.ui.button("Compare", L("hangar.compare"), { variant: "secondary", width: grow });
  });
}

/**
 * The hangar (a fleet game's ship select): the classes at the start (art,
 * name and a line each, the selected one in the list's selected row), the
 * selected class's ship in the centre with its stat bars and upgrade pips,
 * and its detail at the end with Commission as the primary action.
 */
export function hangarScreen(world: World): void {
  const f = frame();
  const { t, vp, small, k, ts } = f;
  const cls = CLASSES[st.cls] ?? CLASSES[0]!;
  const stacked = small || ts > 1.4 || vp.width < 960;
  const pad = small ? t.space.sm : t.space.xl;
  const rowH = Math.round(Math.max(76 * k, f.line("lg", 26) + 2 * f.line("sm", 20) + 2 * t.space.sm));
  const classes = (height: Size) => {
    og.ui.column("Classes", { width: grow, height, gap: t.space.sm }, () => {
      eyebrow("Label", L("hangar.classes"));
      const r = og.ui.list("List", { count: CLASSES.length, rowHeight: rowH, gap: t.space.xs, width: grow, height: px(CLASSES.length * (rowH + t.space.xs)), selected: st.cls }, (i) => {
        const c = CLASSES[i]!;
        og.ui.row("Item", { width: grow, gap: t.space.md, cross: "center", padding: { left: t.space.sm, top: 0, right: t.space.sm, bottom: 0 } }, () => {
          og.ui.artSlot("Art", { texture: art(c.art), kind: "unitPortrait", aspect: 16 / 9, fit: "contain", mask: "none", width: px(Math.round(rowH * 1.4)), label: "" });
          og.ui.column("Text", { width: grow, gap: 2 }, () => {
            og.ui.label("Name", L(`hangar.${c.key}`), { step: "lg", font: "display", nowrap: true, ellipsis: true, width: grow });
            og.ui.label("Line", L(`hangar.${c.key}.line`), { step: "sm", color: { token: "inkMuted" }, maxLines: 2, ellipsis: true, width: grow });
          });
        });
      });
      if (r.clicked !== null) st.cls = r.clicked;
    });
  };
  const heroHead = () => {
    og.ui.column("Head", { width: grow, gap: 2 }, () => {
      og.ui.label("Name", cls.hero, { step: small ? "xl3" : f.big ? "xl6" : "xl5", heading: true, font: "display", nowrap: true, ellipsis: true, width: grow });
      og.ui.label("Class", L(`hangar.${cls.key}.class`), { step: small ? "sm" : "base", color: { token: "inkMuted" }, nowrap: true, ellipsis: true, width: grow });
    });
  };
  og.ui.anchor("Hangar", { anchor: "topLeft", margin: 0, offsetY: f.top, width: px(f.innerW), height: px(f.innerH - f.top + (small ? f.sa.bottom : 0)), direction: "column", gap: t.space.md, padding: { left: pad, top: small ? 0 : t.space.lg, right: pad, bottom: small ? 0 : f.foot + t.space.sm } }, () => {
    if (!small) og.ui.label("Title", L("hangar.title"), { step: f.big ? "xl6" : "xl4", heading: true, font: "display", nowrap: true, ellipsis: true, width: grow });
    if (stacked) {
      og.ui.scrollArea("Body", { width: grow, height: grow, gap: t.space.lg, fade: 56, fadeInto: "shadow", padding: { left: 0, top: 0, right: 0, bottom: small ? f.sa.bottom + t.space.md : 0 } }, () => {
        if (small) og.ui.label("Title", L("hangar.title"), { step: "xl2", heading: true, font: "display", nowrap: true, ellipsis: true, width: grow });
        heroHead();
        og.ui.artSlot("Hero", { texture: art(cls.heroArt), kind: "unitPortrait", aspect: 640 / 417, fit: "contain", mask: "none", width: grow, label: cls.hero });
        heroStats(cls, small);
        heroDetail(world, cls);
        classes({ kind: "fit" });
      });
      return;
    }
    const side = Math.round(Math.min(f.innerW * 0.25, 360 * k));
    og.ui.row("Body", { width: grow, height: grow, gap: t.space.xl, cross: "stretch" }, () => {
      og.ui.column("Start", { width: px(side), height: grow }, () => classes(grow));
      og.ui.column("Centre", { width: grow, height: grow, gap: t.space.md }, () => {
        heroHead();
        og.ui.artSlot("Hero", { texture: art(cls.heroArt), kind: "unitPortrait", aspect: 640 / 417, fit: "contain", mask: "none", width: grow, height: grow, label: cls.hero });
        heroStats(cls, false);
      });
      og.ui.column("End", { width: px(side), height: grow }, () => heroDetail(world, cls));
    });
  });
}

/** The screens' commands: `pat:ship:card:<n>` (-1 none), `pat:ship:class:<n>`, `pat:ship:reset`. */
export function starshipCommand(key: string, value: string): boolean {
  if (key === "card") st.card = Number(value);
  else if (key === "class") st.cls = Math.max(0, Math.min(CLASSES.length - 1, Number(value) || 0));
  else if (key === "reset") {
    st.card = 0;
    st.cls = 0;
    st.zoom = 1;
  } else return false;
  return true;
}
