// The genre patterns' preview screens of the systems games (docs/ui.md §11, U6d r41): a garage with its tuning and
// livery editor, each built from its pattern over the kit's fixtures with the game's side done as a game would (a part
// bought or fitted, a layer moved or recoloured). `pat:<area>:<key>:<value>` sets a preview's state so the playtests
// (tests/patterns.ts, genres2Script) can draw each one.
//
// Names (cars, players, places' proper nouns) are invented and stay as they are in every locale; every other word is
// the game's, keyed in assets/strings.

import { og } from "@origingame/engine";
import { art, L, type ArtName } from "./patterns.ts";
import { frame } from "./genres.ts";
import type { GarageCategory, GaragePart, GarageStat, LiveryLayer, PickedColor } from "@origingame/engine";

// ── garage: parts, tuning and the livery ───────────────────────────────

/** The car's figures as fitted: power, handling, braking, launch and top speed, each out of 10. */
const BASE: Record<string, number> = { power: 6.2, handling: 5.4, braking: 5.8, launch: 6.6, speed: 7.1 };
/** What each category's tiers move: the figure and its gain per tier over stock. */
const MOVES: Record<string, [string, number]> = {
  engine: ["power", 0.9], wheels: ["handling", 0.7], suspension: ["handling", 0.6], exhaust: ["launch", 0.5], brakes: ["braking", 0.9], aero: ["speed", 0.6],
};
const CATS = ["engine", "wheels", "suspension", "exhaust", "brakes", "aero", "livery"] as const;
const CAT_ART: Record<string, ArtName> = {
  engine: "garage/turbo", wheels: "garage/wheels", suspension: "garage/suspension", exhaust: "garage/exhaust", brakes: "garage/brakes", aero: "garage/wing", livery: "garage/coupe",
};
/** A category's tiers: stock (fitted), street, sport, race and works; the price of each, and works locked to class S. */
const TIERS = [["stock", 0], ["street", 6400], ["sport", 18500], ["race", 42000], ["works", 68000]] as const;
const RARITY = ["common", "uncommon", "rare", "epic", "legendary"] as const;

const ga0 = () => ({
  category: "engine",
  selected: "engine.sport",
  balance: 24800,
  owned: new Set<string>(CATS.flatMap((c) => [`${c}.stock`, `${c}.street`])),
  fitted: Object.fromEntries(CATS.map((c) => [c, c === "engine" ? `${c}.street` : `${c}.stock`])) as Record<string, string>,
  layers: [
    { key: "base", name: "base", kind: "paint", color: { r: 0.78, g: 0.09, b: 0.12, a: 1 } },
    { key: "stripe", name: "stripe", kind: "vinyl", color: { r: 0.96, g: 0.96, b: 0.94, a: 1 } },
    { key: "number", name: "number", kind: "decal", color: { r: 0.08, g: 0.08, b: 0.1, a: 1 } },
    { key: "sponsor", name: "sponsor", kind: "decal", color: { r: 0.98, g: 0.74, b: 0.16, a: 1 }, locked: true },
  ] as Array<{ key: string; name: string; kind: string; color: PickedColor; visible?: boolean; locked?: boolean }>,
  layer: "stripe",
  added: 0,
});
let ga = ga0();

const tierOf = (key: string): number => TIERS.findIndex(([t]) => key.endsWith(`.${t}`));
const rating = (fits: Record<string, string>): number => Math.round(600 + Object.values(fits).reduce((n, k) => n + Math.max(0, tierOf(k)) * 9, 0));
const klass = (r: number): string => (r >= 800 ? "S" : r >= 700 ? "A" : r >= 600 ? "B" : "C");

/** A garage (Forza Horizon, Gran Turismo): the categories, the category's parts, the car on its turntable, the performance card. */
export function garageScreen(): void {
  frame("Garage", L("garage.title"), () => {
    const cat = ga.category;
    const livery = cat === "livery";
    const categories: GarageCategory[] = CATS.map((c) => ({ key: c, name: L(`garage.cat.${c}`), image: art(CAT_ART[c]), livery: c === "livery", count: c === "livery" ? String(ga.layers.length) : `${TIERS.filter(([t]) => ga.owned.has(`${c}.${t}`)).length}/${TIERS.length}` }));
    const parts: GaragePart[] = livery ? [] : TIERS.map(([t, price], i) => {
      const key = `${cat}.${t}`;
      return {
        key,
        name: L("garage.part", { tier: L(`garage.tier.${t}`), part: L(`garage.cat.${cat}`) }),
        image: art(CAT_ART[cat]),
        rarity: RARITY[i],
        rarityLabel: L(`garage.tier.${t}`),
        tier: i + 1,
        tierMax: TIERS.length,
        price,
        owned: ga.owned.has(key),
        equipped: ga.fitted[cat] === key,
        locked: t === "works",
        lockedHint: t === "works" ? L("garage.needsClass", { cls: "S" }) : "",
        description: L(`garage.desc.${cat}`),
      };
    });
    // The figures with the selected part fitted in place of its category's: the game's own sums.
    const sel = livery ? null : parts.find((p) => p.key === ga.selected) ?? null;
    const preview = sel && !sel.equipped ? { ...ga.fitted, [cat]: sel.key! } : null;
    const figure = (fits: Record<string, string>, f: string): number => {
      let v = BASE[f]!;
      for (const [c, k] of Object.entries(fits)) if (MOVES[c]?.[0] === f) v += Math.max(0, tierOf(k)) * MOVES[c]![1];
      return Math.round(Math.min(10, v) * 10) / 10;
    };
    const stats: GarageStat[] = Object.keys(BASE).map((f) => ({ key: f, label: L(`garage.stat.${f}`), value: figure(ga.fitted, f), preview: preview ? figure(preview, f) : -1, max: 10 }));
    const now = rating(ga.fitted);
    const layers: LiveryLayer[] = ga.layers.map((l) => ({ key: l.key, name: l.name.startsWith("added") ? L("garage.layer.added", { n: l.name.slice(5) }) : L(`garage.layer.${l.name}`), kind: L(`garage.kind.${l.kind}`), color: l.color, visible: l.visible !== false, locked: !!l.locked }));
    const r = og.ui.garage("Garage", {
      balance: ga.balance,
      categories,
      category: cat,
      parts,
      selected: ga.selected,
      vehicle: art("garage/coupe"),
      stats,
      ratingClass: klass(preview ? rating(preview) : now),
      rating: now,
      ratingPreview: preview ? rating(preview) : 0,
      layers,
      layer: ga.layer,
      swatches: ["#c7171f", "#f5f5f0", "#14141a", "#fabd29", "#1a6bdb", "#1fa866", "#8c4dd9", "#999ea8"],
      performanceLabel: L("garage.performance"),
      classLabel: L("garage.class"),
      currentLabel: L("garage.current"),
      previewLabel: L("garage.preview"),
      equipLabel: L("garage.equip"),
      equippedLabel: L("garage.equipped"),
      buyLabel: L("garage.buy"),
      lockedLabel: L("garage.locked"),
      freeLabel: L("garage.free"),
      shortLabel: L("garage.short"),
      layersLabel: L("garage.layers"),
      colorLabel: L("garage.color"),
      addLayerLabel: L("garage.addLayer"),
      deleteLabel: L("garage.delete"),
      upLabel: L("garage.up"),
      downLabel: L("garage.down"),
      visibleLabel: L("garage.visible"),
    });
    // The game's side: a category opens on its fitted part; a purchase is paid and owned; a fitting replaces the category's.
    if (r.category !== null) {
      ga.category = r.category;
      ga.selected = ga.fitted[r.category] ?? "";
    }
    if (r.selected !== null) ga.selected = r.selected;
    const price = (k: string): number => TIERS[tierOf(k)]?.[1] ?? 0;
    if (r.buy !== null && price(r.buy) <= ga.balance) {
      ga.balance -= price(r.buy);
      ga.owned.add(r.buy);
    }
    if (r.equip !== null) ga.fitted[cat] = r.equip;
    if (r.layer !== null) ga.layer = r.layer;
    const li = (k: string | null): number => ga.layers.findIndex((l) => l.key === k);
    if (r.layerColor !== null && li(ga.layer) >= 0) ga.layers[li(ga.layer)]!.color = r.layerColor;
    if (r.layerVisible !== null && li(r.layerVisible) >= 0) {
      const l = ga.layers[li(r.layerVisible)]!;
      l.visible = l.visible === false;
    }
    if (r.layerMove !== null) {
      const i = li(r.layerMove.key);
      const j = i + r.layerMove.by;
      if (i >= 0 && j >= 0 && j < ga.layers.length) [ga.layers[i], ga.layers[j]] = [ga.layers[j]!, ga.layers[i]!];
    }
    if (r.addLayer) {
      ga.added++;
      const key = `added${ga.added}`;
      ga.layers.unshift({ key, name: `added${ga.added}`, kind: "decal", color: { r: 0.1, g: 0.42, b: 0.86, a: 1 } });
      ga.layer = key;
    }
    if (r.deleteLayer !== null && li(r.deleteLayer) >= 0) {
      ga.layers.splice(li(r.deleteLayer), 1);
      ga.layer = ga.layers[0]?.key ?? "";
    }
  }, [{ action: "tabNext", hint: L("garage.cat.next") }, ...(ga.category === "livery" ? [] : [{ action: "confirm", hint: L("garage.buy") }]), { action: "back", hint: L("hint.back") }]);
}

/** The previews' commands: `pat:garage:<key>:<value>`; false when the area is not one of these. */
export function genres2Command(area: string, key: string, value: string): boolean {
  if (area === "garage") {
    if (key === "category") {
      ga.category = value;
      ga.selected = ga.fitted[value] ?? "";
    } else if (key === "select") ga.selected = value;
    else if (key === "layer") ga.layer = value;
    else if (key === "balance") ga.balance = Number(value) || 0;
    else if (key === "reset") ga = ga0();
    else return false;
  } else return false;
  return true;
}

/** Every preview of this file back to its start (pat:reset). */
export function resetGenres2(): void {
  ga = ga0();
}
