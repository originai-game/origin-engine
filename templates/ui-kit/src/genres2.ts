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
import type { AbilitySlotDesc, ChartSeries, ChatMessage, SelectedUnit, World, GarageCategory, GaragePart, GarageStat, LedgerLine, LiveryLayer, MapOverlay, PickedColor, Size } from "@origingame/engine";

const grow: Size = { kind: "grow" };
const px = (n: number): Size => ({ kind: "fixed", px: n });

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

// ── ledger: a city's budget, and the map's overlays ───────────────────

/** The city's taxes and services as the game holds them: this period's base amount, last period's, the budget. */
const TAXES = [["residential", 52400, 49800], ["commercial", 31200, 33900], ["industrial", 24800, 23100], ["tourism", 8600, 6100]] as const;
const SERVICES = [
  ["police", 14200, 13600, 100], ["fire", 11800, 12100, 75], ["health", 16400, 15200, 105], ["education", 19600, 18800, 110],
  ["transit", 12900, 11400, 120], ["parks", 4100, 4300, 90], ["power", 9800, 9900, 100], ["water", 7300, 7100, 100],
] as const;
const PERIODS: Record<string, number> = { month: 12, quarter: 8, year: 6 };

const ld0 = () => ({ tab: "month", sort: "", descending: false, funding: Object.fromEntries(SERVICES.map(([k, , , f]) => [k, f as number])) as Record<string, number>, overlay: "traffic" });
let ld = ld0();

/** A history the game keeps: income and expenses per period, a slow climb with a wobble, its last period this one's. */
function history(n: number, scale: number, inc: number, exp: number): ChartSeries[] {
  const wave = (i: number, k: number): number => 1 - (n - 1 - i) * 0.035 * scale + Math.sin(i * 1.3 + k) * 0.04 * scale;
  return [
    { label: L("ledger.income"), values: Array.from({ length: n }, (_, i) => Math.round(inc * scale * wave(i, 0))) },
    { label: L("ledger.expenses"), values: Array.from({ length: n }, (_, i) => Math.round(exp * scale * wave(i, 2))) },
  ];
}

/** A management ledger (Cities: Skylines' budget): the treasury's figures, taxes and services with their budgets, the history. */
export function ledgerScreen(): void {
  frame("Ledger", L("ledger.title"), () => {
    const scale = ld.tab === "year" ? 12 : ld.tab === "quarter" ? 3 : 1;
    const lines: LedgerLine[] = [
      ...SERVICES.map(([k, a, p]) => {
        const f = ld.funding[k] ?? 100;
        return {
          key: k,
          section: "expenses",
          name: L(`ledger.line.${k}`),
          image: art(`city/${k}` as ArtName),
          amount: Math.round((a * f) / 100) * scale,
          previous: p * scale,
          funding: f,
          warning: f < 80 ? L(`ledger.warn.${k}`) : "",
          locked: k === "water",
        };
      }),
      ...TAXES.map(([k, a, p]) => ({ key: k, section: "income", name: L(`ledger.line.${k}`), image: art(`city/${k}` as ArtName), amount: a * scale, previous: p * scale })),
    ];
    const inc = lines.filter((l) => l.section === "income").reduce((n, l) => n + (l.amount ?? 0), 0);
    const exp = lines.filter((l) => l.section === "expenses").reduce((n, l) => n + (l.amount ?? 0), 0);
    const incPrev = lines.filter((l) => l.section === "income").reduce((n, l) => n + (l.previous ?? 0), 0);
    const expPrev = lines.filter((l) => l.section === "expenses").reduce((n, l) => n + (l.previous ?? 0), 0);
    const n = PERIODS[ld.tab] ?? 12;
    const r = og.ui.ledger("Budget", {
      tabs: Object.keys(PERIODS).map((k) => ({ key: k, label: L(`ledger.tab.${k}`) })),
      tab: ld.tab,
      figures: [
        { key: "treasury", label: L("ledger.treasury"), value: 1284600, delta: inc - exp, unit: "", image: art("city/commercial") },
        { key: "income", label: L("ledger.income"), value: inc, delta: inc - incPrev },
        { key: "expenses", label: L("ledger.expenses"), value: exp, delta: exp - expPrev, lowerIsBetter: true },
        { key: "population", label: L("ledger.population"), value: 184320, delta: 2140, format: "compact", image: art("city/residential") },
      ],
      // The services first: their budgets are what the player comes to move.
      sections: [{ key: "expenses", name: L("ledger.expenses"), expense: true }, { key: "income", name: L("ledger.income") }],
      lines,
      sort: ld.sort,
      descending: ld.descending,
      fundingMin: 50,
      fundingMax: 150,
      fundingStep: 5,
      periods: Array.from({ length: n }, (_, i) => L(`ledger.period.${ld.tab}`, { n: i + 1 })),
      history: history(n, scale, inc / scale, exp / scale),
      breakdown: "expenses",
      currency: "$",
      nameLabel: L("ledger.col.name"),
      amountLabel: L("ledger.col.amount"),
      changeLabel: L("ledger.col.change"),
      fundingLabel: L("ledger.col.budget"),
      totalLabel: L("ledger.total"),
      netLabel: L("ledger.net"),
      historyLabel: L("ledger.history"),
      breakdownLabel: L("ledger.breakdown"),
      otherLabel: L("ledger.other"),
      lockedLabel: L("ledger.locked"),
    });
    if (r.tab !== null) ld.tab = r.tab;
    if (r.sort !== null) {
      ld.sort = r.sort;
      ld.descending = r.descending;
    }
    if (r.funding !== null) ld.funding[r.funding.key] = r.funding.value;
    // A service pressed shows its overlay on the map, as a city builder's budget links to its info view.
    if (r.selected !== null && OVERLAYS.some((o) => o.key === OVERLAY_OF[r.selected!])) ld.overlay = OVERLAY_OF[r.selected]!;
  }, [{ action: "tabNext", hint: L("ledger.hint.period") }, { action: "back", hint: L("hint.back") }]);
}

const OVERLAYS = [
  { key: "traffic", art: "city/transit", ramp: ["#2ecc71", "#f1c40f", "#e74c3c"] },
  { key: "power", art: "city/power", ramp: ["#1b1f3b", "#3a6df0", "#ffd23f"] },
  { key: "water", art: "city/water", ramp: ["#3b2a1b", "#1f8fd6", "#7fe3ff"] },
  { key: "pollution", art: "city/industrial", ramp: ["#5fd38d", "#b8a24a", "#6b3f2a"] },
  { key: "landValue", art: "city/residential", ramp: ["#6b5a3a", "#d6b24a", "#fff3a8"] },
  { key: "crime", art: "city/police", ramp: ["#2d6cdf", "#9b59b6", "#e74c3c"] },
] as const;
/** The districts an overlay keys its data to, as fractions of the map (centre x, centre y, width): where the city is. */
const DISTRICTS: Array<[number, number, number]> = [
  [0.3, 0.42, 0.16], [0.48, 0.36, 0.14], [0.64, 0.44, 0.18], [0.4, 0.6, 0.15], [0.56, 0.62, 0.17], [0.74, 0.6, 0.12], [0.22, 0.62, 0.11],
];
const OVERLAY_OF: Record<string, string> = { transit: "traffic", power: "power", water: "water", police: "crime", industrial: "pollution", residential: "landValue" };

/** The city's map with its overlays (Cities: Skylines' info views): the picture, the shown overlay's wash, the bar and legend. */
export function cityMapScreen(): void {
  frame("CityMap", "", () => {
    const shown = OVERLAYS.find((o) => o.key === ld.overlay) ?? null;
    og.ui.column("Map", { width: grow, height: grow, align: "end", cross: "center", padding: { left: 12, top: 12, right: 12, bottom: 16 } }, () => {
      const tex = art("scenes/city");
      og.ui.setDraw({ shape: { kind: "rect" }, fills: tex ? [{ kind: "image", texture: tex, fit: "cover", focal: { x: 0.5, y: 0.6 } }] : [{ kind: "solid", color: { token: "backdrop" } }] } as never);
      // The game's overlay over its map, keyed only where its data is: a soft spot per district at 35–45 %, coloured by
      // its value on the ramp, clear between them so the city still reads under it.
      if (shown) {
        // The map's box from the last frame (the viewport's until it has one).
        const vp = og.ui.frameViewport();
        const mb = og.ui.bounds("CityMap/Body/Map") ?? { width: vp.width, height: vp.height };
        const seed = [...shown.key].reduce((n, c) => (n * 31 + c.charCodeAt(0)) >>> 0, 7);
        const rnd = (i: number): number => (((seed ^ (i * 2654435761)) >>> 0) % 1000) / 1000;
        DISTRICTS.forEach(([x, y, size], i) => {
          const v = rnd(i);
          const at = Math.min(shown.ramp.length - 1, Math.floor(v * shown.ramp.length));
          const w = mb.width * size;
          og.ui.column(`Spot${i}`, { width: px(w), height: px(w * 0.62) }, () => {});
          og.ui.setDraw({
            shape: { kind: "ellipse" },
            fills: [{ kind: "gradient", type: "radial", from: { x: 0.5, y: 0.5 }, to: { x: 1, y: 0.5 }, stops: [
              { offset: 0, color: { color: hexRgb(shown.ramp[at]!), alpha: 0.35 + 0.1 * v } },
              { offset: 0.6, color: { color: hexRgb(shown.ramp[at]!), alpha: 0.25 } },
              { offset: 1, color: { color: hexRgb(shown.ramp[at]!), alpha: 0 } },
            ] }],
          } as never);
          og.ui.props({ position: "absolute", insets: { start: mb.width * x - w / 2, top: mb.height * y - w * 0.31 }, inert: true } as never);
        });
      }
      const overlays: MapOverlay[] = OVERLAYS.map((o) => ({ key: o.key, name: L(`ledger.overlay.${o.key}`), image: art(o.art as ArtName), ramp: [...o.ramp], low: L(`ledger.overlay.${o.key}.low`), high: L(`ledger.overlay.${o.key}.high`) }));
      const r = og.ui.overlayBar("Lenses", { overlays, selected: ld.overlay, label: L("ledger.overlays"), offLabel: L("ledger.overlay.off") });
      if (r.selected !== null) ld.overlay = r.selected;
    });
  }, [], true);
}

function hexRgb(s: string): { r: number; g: number; b: number } {
  const n = (k: number): number => parseInt(s.slice(1 + k, 3 + k), 16) / 255;
  return { r: n(0), g: n(2), b: n(4) };
}

// ── chat: channels, links and mentions ─────────────────────────────────

/** Who speaks: invented names and the kit's portraits. */
const SPEAKERS: Record<string, { name: string; art: ArtName }> = {
  lumen: { name: "Lumen", art: "portraits/aria" },
  riven: { name: "Riven", art: "portraits/kael" },
  juno: { name: "Juno", art: "portraits/mira" },
  sable: { name: "Sable", art: "portraits/wren" },
  vesper: { name: "Vesper", art: "portraits/vesper" },
  torvin: { name: "Torvin", art: "portraits/torvin" },
  maren: { name: "Maren", art: "portraits/maren" },
};
const CHAT_CHANNELS = [["all", "", true], ["party", "info"], ["guild", "success"], ["trade", "warn"], ["whisper", "#e58cff"], ["system", "inkMuted"]] as const;
/** The log as the game holds it: who, where, when, the line's words (keyed: chat.msg.<n>) and its links. */
const CHAT_LOG: Array<{ who: string; ch: string; time: string; n: number; links?: Array<[string, string, ArtName | null, string]>; system?: boolean }> = [
  { who: "", ch: "system", time: "20:58", n: 1, system: true },
  { who: "torvin", ch: "guild", time: "20:59", n: 2 },
  { who: "maren", ch: "trade", time: "21:00", n: 3, links: [["ember-ring", "chat.link.ember", "items/ember-ring", "epic"]] },
  { who: "riven", ch: "party", time: "21:01", n: 4, links: [["amberhold", "chat.link.amberhold", null, "none"]] },
  { who: "juno", ch: "party", time: "21:01", n: 5 },
  { who: "vesper", ch: "whisper", time: "21:02", n: 6 },
  { who: "sable", ch: "party", time: "21:03", n: 7, links: [["ashbringer", "chat.link.ashbringer", "items/ashbringer", "legendary"]] },
  { who: "torvin", ch: "guild", time: "21:03", n: 8 },
  { who: "riven", ch: "party", time: "21:04", n: 9 },
];
const ct0 = () => ({ channel: "all", draft: "", sendTo: "party", hud: false, typing: false, sent: [] as Array<{ ch: string; text: string }>, read: new Set<string>(["all"]) });
let ct = ct0();

/** A chat window (World of Warcraft, Final Fantasy XIV): channel tabs, the log with links and mentions, the input. */
export function chatScreen(): void {
  const messages: ChatMessage[] = [
    ...CHAT_LOG.map((m, i) => ({
      key: `m${i + 1}`,
      channel: m.ch,
      author: m.who ? SPEAKERS[m.who]!.name : "",
      authorKey: m.who,
      avatar: m.who ? art(SPEAKERS[m.who]!.art) : null,
      text: L(`chat.msg.${m.n}`),
      links: (m.links ?? []).map(([key, label, , rarity]) => ({ key, label: L(label), kind: rarity === "none" ? "place" : "item", rarity: rarity as never })),
      time: m.time,
      system: !!m.system,
    })),
    ...ct.sent.map((m, i) => ({ key: `own${i}`, channel: m.ch, author: "Lumen", authorKey: "lumen", avatar: art("portraits/aria"), text: m.text, time: "21:05", own: true })),
  ];
  const unread = (ch: string): number => (ct.read.has(ch) ? 0 : messages.filter((m) => m.channel === ch).length);
  const mentions = (ch: string): number => (ct.read.has(ch) ? 0 : messages.filter((m) => m.channel === ch && /@Lumen\b/.test(m.text ?? "")).length);
  const build = (): void => {
    const r = og.ui.chat("Chat", {
      channels: CHAT_CHANNELS.map(([k, c, a]) => ({ key: k, name: L(`chat.ch.${k}`), color: c, all: !!a, readOnly: k === "system", unread: unread(k), mentions: mentions(k) })),
      channel: ct.channel,
      messages,
      me: "Lumen",
      members: Object.entries(SPEAKERS).map(([k, v]) => ({ key: k, name: v.name, avatar: art(v.art) })),
      draft: ct.draft,
      sendTo: ct.sendTo,
      hud: ct.hud,
      typing: ct.typing,
      placeholder: L("chat.placeholder"),
      sendLabel: L("chat.send"),
      newLabel: L("chat.new"),
      channelLabel: L("chat.channel"),
    });
    if (r.channel !== null) {
      ct.channel = r.channel;
      ct.read.add(r.channel);
      if (r.channel !== "all" && r.channel !== "system") ct.sendTo = r.channel;
    }
    if (r.sendTo !== null) ct.sendTo = r.sendTo;
    if (r.draft !== null) ct.draft = r.draft;
    if (r.send !== null) {
      ct.sent.push({ ch: r.send.channel, text: r.send.text });
      ct.draft = "";
    }
    if (r.author !== null && r.author !== "lumen") {
      ct.sendTo = "whisper";
      ct.draft = `@${SPEAKERS[r.author]?.name ?? ""} `;
    }
    if (r.closed) ct.typing = false;
  };
  if (ct.hud) {
    // The HUD's chat over the game at the bottom start, as an MMO keeps it.
    frame("ChatHud", "", () => {
      og.ui.column("Corner", { width: grow, height: grow, align: "end", padding: { left: 16, top: 16, right: 16, bottom: 16 } }, () => {
        og.ui.column("Box", { width: px(Math.min(520, og.ui.frameViewport().width - 32)), height: px(Math.min(320, og.ui.frameViewport().height * 0.45)) }, build);
      });
    }, [], true);
  } else frame("Chats", L("chat.title"), build, [{ action: "confirm", hint: L("chat.send") }, { action: "back", hint: L("hint.back") }]);
}

// ── RTS: the command card's pages, hotkeys and control groups ───────────

/** The grid's keys by physical code (A S D F G, Z C V B), whatever the layout prints on them. */
const GRID = [["cmdA", "KeyA"], ["cmdS", "KeyS"], ["cmdD", "KeyD"], ["cmdF", "KeyF"], ["cmdG", "KeyG"], ["cmdZ", "KeyZ"], ["cmdC", "KeyC"], ["cmdV", "KeyV"], ["cmdB", "KeyB"]] as const;

/** The preview's UI actions: the grid's keys and the control groups' digits (the game's setup defines them). */
export function defineGenres2Actions(world: World): void {
  const act = (name: string, keys: string[], label: string) => og.ui.defineAction(world, { name, keys, buttons: [], trigger: "press", holdMs: 0, toggle: false, label });
  for (const [name, code] of GRID) act(name, [code], code.slice(3));
  for (let n = 1; n <= 5; n++) act(`group${n}`, [`Digit${n}`], String(n));
}

/** The workers' orders and, a page down, what they build: each one's art, its key's action, its cost and state. */
const ORDERS: Array<[string, ArtName, string, string?]> = [["stop", "rts/stop", "cmdA"], ["patrol", "rts/patrol", "cmdS"], ["gather", "rts/gather", "cmdD"], ["repair", "rts/repair", "cmdF"], ["rally", "rts/rally", "cmdG"], ["build", "rts/build", "cmdZ"]];
const BUILDS: Array<[string, ArtName, string, string, number]> = [["barracks", "rts/barracks", "cmdA", "150", 0], ["tower", "rts/tower", "cmdS", "125", 0], ["farm", "rts/farm", "cmdD", "60", 0], ["wall", "rts/wall", "cmdF", "20", 0], ["forge", "rts/forge", "cmdG", "200", 1], ["keep", "rts/keep", "cmdZ", "650", 2]];
const UNITS: Array<[string, string, ArtName, number]> = [["w1", "worker", "portraits/wren", 38], ["w2", "worker", "portraits/maren", 40], ["w3", "worker", "portraits/brann", 22], ["k1", "knight", "portraits/kael", 140], ["k2", "knight", "portraits/torvin", 96], ["a1", "archer", "portraits/vesper", 55]];

const rt0 = () => ({ page: "", targeting: "", gold: 480, groups: [["w1", "w2", "w3"], ["k1", "k2"], [], [], []] as string[][], selected: ["w1", "w2", "w3", "k1", "k2", "a1"], focus: "w1", said: "" });
let rt = rt0();

/** An RTS's console (Age of Empires IV): the selection with its groups, the command card's orders and its Build page. */
export function rtsScreen(): void {
  frame("Rts", "", () => {
    og.ui.column("Field", { width: grow, height: grow }, () => {
      const tex = art("scenes/battlefield");
      og.ui.setDraw({ shape: { kind: "rect" }, fills: tex ? [{ kind: "image", texture: tex, fit: "cover", focal: { x: 0.5, y: 0.45 } }] : [{ kind: "solid", color: { token: "backdrop" } }] } as never);
    });
    const slot = (k: string, a: ArtName, action: string, extra: Partial<AbilitySlotDesc> = {}): AbilitySlotDesc => ({ key: k, name: L(`rts.cmd.${k}`), image: art(a), action, ...extra });
    const units: SelectedUnit[] = UNITS.filter(([k]) => rt.selected.includes(k)).map(([k, kind, a, hp]) => ({ key: k, name: L(`rts.unit.${kind}`), kind: L(`rts.kind.${kind}`), image: art(a), health: hp, healthMax: kind === "knight" ? 140 : kind === "archer" ? 70 : 40 }));
    const r = og.ui.rtsCommands("Console", {
      console: {
        map: { texture: art("scenes/battlefield"), size: 0, shape: "square", markers: [] },
        selection: {
          units,
          focus: rt.focus,
          groups: rt.groups.map((g, i) => ({ key: `g${i + 1}`, label: String(i + 1), count: g.length })),
          stats: [{ key: "attack", label: L("rts.stat.attack"), value: "3" }, { key: "armour", label: L("rts.stat.armour"), value: "0 / 1" }, { key: "speed", label: L("rts.stat.speed"), value: "1.0" }],
        },
        commands: { columns: 5 },
        anchor: "bottom",
      },
      pages: [
        { key: "root", commands: ORDERS.map(([k, a, act]) => slot(k, a, act)) },
        { key: "build", commands: BUILDS.map(([k, a, act, cost, lock]) => slot(k, a, act, { cost, unaffordable: Number(cost) > rt.gold, locked: lock > 1, ...(lock === 1 ? { cooldown: 12, cooldownMax: 30 } : {}) })) },
      ],
      page: rt.page,
      groupActions: ["group1", "group2", "group3", "group4", "group5"],
      targeting: rt.targeting,
      targetLabel: L("rts.target"),
      cancelLabel: L("rts.cancel"),
      queueLabel: L("rts.queue"),
      backLabel: L("rts.back"),
    });
    // The game's side: pages, orders (a targeted one waits for its target), the groups and the selection.
    if (r.page !== null) rt.page = r.page;
    if (r.command !== null) {
      if (["patrol", "gather", "repair", "rally"].includes(r.command) || BUILDS.some(([k]) => k === r.command)) rt.targeting = r.command;
      else rt.targeting = "";
    }
    // A building awaiting its ground keeps the Build page (StarCraft's), so Back cancels the placement and stays there.
    if (r.cancelTarget) rt.targeting = "";
    const gi = (k: string | null): number => (k ? Number(k.slice(1)) - 1 : -1);
    if (r.recall !== null || r.focus !== null) {
      const g = rt.groups[gi(r.recall ?? r.focus)] ?? [];
      if (g.length > 0) {
        rt.selected = [...g];
        rt.focus = g[0]!;
      }
    }
    if (r.assign !== null) rt.groups[gi(r.assign)] = [...rt.selected];
    if (r.add !== null) rt.groups[gi(r.add)] = [...new Set([...(rt.groups[gi(r.add)] ?? []), ...rt.selected])];
    if (r.clicked !== null) rt.focus = r.clicked;
    if (r.selectType !== null) {
      const kind = UNITS.find(([k]) => k === r.selectType)?.[1];
      rt.selected = rt.selected.filter((k) => UNITS.find(([u]) => u === k)?.[1] === kind);
      rt.focus = r.selectType;
    }
    if (r.deselect !== null && rt.selected.length > 1) {
      rt.selected = rt.selected.filter((k) => k !== r.deselect);
      if (rt.focus === r.deselect) rt.focus = rt.selected[0]!;
    }
  }, [], true);
}

/** The previews' commands: `pat:<garage|ledger|chat|rts>:<key>:<value>`; false when the area is not one of these. */
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
  } else if (area === "ledger") {
    if (key === "tab") ld.tab = value;
    else if (key === "sort") {
      ld.sort = value;
      ld.descending = value !== "name";
    } else if (key === "fund") {
      const [k, v] = value.split("=");
      ld.funding[k ?? ""] = Number(v) || 0;
    } else if (key === "overlay") ld.overlay = value;
    else if (key === "reset") ld = ld0();
    else return false;
  } else if (area === "chat") {
    if (key === "channel") {
      ct.channel = value;
      ct.read.add(value);
    } else if (key === "draft") ct.draft = value.replace(/_/g, " ");
    else if (key === "hud") ct.hud = value !== "off";
    else if (key === "typing") ct.typing = value !== "off";
    else if (key === "reset") ct = ct0();
    else return false;
  } else if (area === "rts") {
    if (key === "page") rt.page = value;
    else if (key === "target") rt.targeting = value;
    else if (key === "gold") rt.gold = Number(value) || 0;
    else if (key === "reset") rt = rt0();
    else return false;
  } else return false;
  return true;
}

/** Every preview of this file back to its start (pat:reset). */
export function resetGenres2(): void {
  ga = ga0();
  ld = ld0();
  ct = ct0();
  rt = rt0();
}
