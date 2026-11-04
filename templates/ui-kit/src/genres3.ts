// The genre patterns' preview screens of the last U6d round (docs/ui.md §11, U6d r42): photo mode, a loadout, a deck builder, a CG gallery, a route chart and the settings tree, each built from its
// pattern over the kit's fixtures with the game's side done as a game would (the camera moved, a filter drawn, the shot
// taken). `pat:<area>:<key>:<value>` sets a preview's state so the playtests (tests/patterns.ts, genres3Script) can
// draw each one.
//
// Names (players, places' proper nouns) are invented and stay as they are in every locale; every other word is the
// game's, keyed in assets/strings.

import { og } from "@origingame/engine";
import { art, chromeRoom, L, phone, type ArtName } from "./patterns.ts";
import { frame } from "./genres.ts";
import type { CgEntry, CollectionCard, PhotoControl, PhotoLook, DisplaySettings, Rarity, RouteScene, SettingEntry, SettingKind, Size, World } from "@origingame/engine";
import { shooter, type LoadoutAttachment, type LoadoutSlot, type LoadoutStat } from "@origingame/engine/extensions/shooter/index.ts";

const grow: Size = { kind: "grow" };
const px = (n: number): Size => ({ kind: "fixed", px: n });

// ── photo mode: the camera, filters and frames over the paused shot ──────

/** The sliders: key, tab, min, max, step, unit, decimals, origin, default. */
const CONTROLS: Array<[string, string, number, number, number, string, number, number | null, number]> = [
  ["fov", "camera", 20, 110, 1, "°", 0, null, 60],
  ["roll", "camera", -30, 30, 1, "°", 0, 0, 0],
  ["height", "camera", -2, 2, 0.1, "m", 1, 0, 0],
  ["focus", "lens", 0.5, 50, 0.5, "m", 1, null, 12],
  ["aperture", "lens", 1.4, 22, 0.1, "", 1, null, 2.8],
  ["exposure", "lens", -2, 2, 0.1, "EV", 1, 0, 0],
  ["vignette", "lens", 0, 1, 0.05, "%", 0, null, 0.3],
];
const FILTERS: Array<[string, ArtName]> = [["none", "photo/shot"], ["vivid", "photo/vivid"], ["noir", "photo/noir"], ["sepia", "photo/sepia"], ["cool", "photo/cool"]];
const FRAMES: Array<[string, ArtName | null, ArtName]> = [
  ["none", null, "photo/shot"], ["cinema", "photo/frame-cinema", "photo/thumb-cinema"], ["polaroid", "photo/frame-polaroid", "photo/thumb-polaroid"], ["film", "photo/frame-film", "photo/thumb-film"],
];

const ph0 = () => ({
  tab: "camera",
  values: Object.fromEntries(CONTROLS.map((c) => [c[0], c[8]])) as Record<string, number>,
  filter: "none",
  strength: 0.8,
  frame: "none",
  grid: false,
  hidden: false,
  shots: 0,
});
let ph = ph0();

/** Photo mode is hidden for a clean shot: the kit's own chrome (its Gallery button) steps aside too. */
export const photoHidden = (): boolean => ph.hidden;

/**
 * Photo mode (Horizon Forbidden West, Ghost of Tsushima): the paused shot under the controls, drawn as the game would
 * through the camera's zoom, roll and exposure, the chosen filter and the frame. True when the player asked to leave.
 */
export function photoScreen(): boolean {
  const vp = og.ui.frameViewport();
  const sa = vp.safeArea;
  const v = ph.values;
  // The game's view: the shot, through the filter (its strength blends the plain shot over it), zoomed by the field of
  // view, turned by the roll, raised by the camera's height; exposure and vignette washed over it.
  const zoom = Math.tan((60 * Math.PI) / 360) / Math.tan(((v.fov ?? 60) * Math.PI) / 360);
  const k = Math.max(1, zoom) * (1 + Math.abs(v.roll ?? 0) / 60);
  const plain = art("photo/shot");
  const filtered = art(FILTERS.find(([f]) => f === ph.filter)?.[1] ?? "photo/shot");
  og.ui.anchor("Shot", { anchor: "topLeft", margin: 0, offsetX: -sa.left, offsetY: -sa.top, width: px(vp.width), height: px(vp.height) }, () => {
    const layer = (id: string, tex: unknown, alpha: number): void => {
      og.ui.column(id, { width: grow, height: grow }, () => {});
      og.ui.setDraw({ shape: { kind: "rect" }, fills: tex ? [{ kind: "image", texture: tex, fit: "cover", focal: { x: 0.5, y: 0.5 } }] : [{ kind: "solid", color: { token: "backdrop" } }] } as never);
      // The blend is the element's opacity, not the draw's: a draw's opacity below 1 composites through a layer.
      og.ui.props({ position: "absolute", insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true, opacity: alpha, transform: { scaleX: k, scaleY: k, rotate: ((v.roll ?? 0) * Math.PI) / 180, y: (v.height ?? 0) * vp.height * 0.06 } } as never);
    };
    layer("Picture", ph.filter === "none" ? plain : filtered, 1);
    if (ph.filter !== "none") layer("Plain", plain, 1 - ph.strength);
    const ev = v.exposure ?? 0;
    og.ui.column("Light", { width: grow, height: grow }, () => {});
    og.ui.setDraw({
      shape: { kind: "rect" },
      fills: [
        ...(ev === 0 ? [] : [{ kind: "solid", color: ev > 0 ? { r: 1, g: 1, b: 1, a: ev * 0.18 } : { r: 0, g: 0, b: 0, a: -ev * 0.3 } }]),
        { kind: "gradient", type: "radial", from: { x: 0.5, y: 0.5 }, to: { x: 1, y: 1 }, stops: [{ offset: 0.35, color: { r: 0, g: 0, b: 0, a: 0 } }, { offset: 1, color: { r: 0, g: 0, b: 0, a: 0.9 * (v.vignette ?? 0) } }] },
      ],
    } as never);
    og.ui.props({ position: "absolute", insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true } as never);
  });
  og.ui.props({ inert: true } as never);

  let exit = false;
  // On a desktop the mode keeps out of the kit's chrome in the bottom start corner (its room as laid).
  og.ui.anchor("Photo", { anchor: "topLeft", margin: 0, width: px(vp.width - sa.left - sa.right), height: px(vp.height - sa.top - sa.bottom), direction: "column", padding: { left: 0, top: 0, right: 0, bottom: phone() ? 0 : chromeRoom().bottom } }, () => {
    const controls: PhotoControl[] = CONTROLS.map(([key, tab, min, max, step, unit, digits, origin]) => ({ key, tab, label: L(`photo.ctl.${key}`), value: v[key] ?? 0, min, max, step, unit, digits, origin }));
    // The game's own render of the shot through each filter and frame for their tiles.
    const filters: PhotoLook[] = FILTERS.map(([key, a]) => ({ key, name: L(`photo.filter.${key}`), image: art(a) }));
    const frames: PhotoLook[] = FRAMES.map(([key, , thumb]) => ({ key, name: L(`photo.frame.${key}`), image: art(thumb) }));
    const r = og.ui.photoMode("Mode", {
      tabs: ["camera", "lens", "filters", "frames"].map((key) => ({ key, label: L(`photo.tab.${key}`) })),
      tab: ph.tab,
      controls,
      filters,
      filter: ph.filter,
      strength: ph.strength,
      frames,
      frame: ph.frame,
      frameImage: art(FRAMES.find(([f]) => f === ph.frame)?.[1] ?? undefined),
      grid: ph.grid,
      hidden: ph.hidden,
      title: L("photo.title"),
      captureLabel: L("photo.capture"),
      hideLabel: L("photo.hide"),
      gridLabel: L("photo.grid"),
      resetLabel: L("photo.reset"),
      exitLabel: L("photo.exit"),
      strengthLabel: L("photo.strength"),
      hiddenHint: L("photo.hint"),
    });
    // The game's side: the camera and the look take what was asked; Capture saves the shot (the album is the game's).
    if (r.tab !== null) ph.tab = r.tab;
    if (r.set !== null) v[r.set.key] = r.set.value;
    if (r.filter !== null) ph.filter = r.filter;
    if (r.strength !== null) ph.strength = r.strength;
    if (r.frame !== null) ph.frame = r.frame;
    if (r.grid) ph.grid = !ph.grid;
    if (r.hide) ph.hidden = !ph.hidden;
    if (r.capture) ph.shots++;
    if (r.reset) ph.values = ph0().values;
    exit = r.exit;
  });
  return exit;
}

// ── loadout: the slots, the weapon and its attachments ─────────────────

/** The slots: key, art, level (0 none), locked. */
const SLOTS: Array<[string, ArtName, number, boolean]> = [
  ["primary", "loadout/raven", 12, false], ["secondary", "loadout/sidearm", 6, false], ["melee", "loadout/knife", 3, false],
  ["lethal", "loadout/frag", 0, false], ["tactical", "loadout/stim", 0, false], ["field", "loadout/drone", 0, true],
];
/** The rifle's attachment points: key, art, where on the art (x, y), its tier, and what it changes. */
const POINTS: Array<[string, ArtName, number, number, Rarity, Record<string, number>]> = [
  ["stock", "loadout/stock", 0.1, 0.4, "uncommon", { control: 4, mobility: -2 }],
  ["optic", "loadout/optic", 0.44, 0.22, "rare", { range: 4 }],
  ["laser", "loadout/laser", 0.74, 0.3, "epic", { mobility: 4 }],
  ["magazine", "loadout/magazine", 0.52, 0.68, "rare", { fireRate: -3, damage: 2 }],
  ["underbarrel", "loadout/grip", 0.67, 0.51, "uncommon", { control: 3 }],
  ["muzzle", "loadout/suppressor", 0.94, 0.37, "rare", { damage: 2, range: 2 }],
];
/** Each slot's figures before its attachments. */
const FIGURES: Record<string, Record<string, number>> = {
  primary: { damage: 28, range: 62, fireRate: 78, mobility: 60, control: 51 },
  secondary: { damage: 22, range: 30, fireRate: 64, mobility: 88, control: 70 },
  melee: { damage: 90, range: 4, fireRate: 40, mobility: 96, control: 80 },
  lethal: { damage: 95, range: 35, fireRate: 10, mobility: 70, control: 40 },
  tactical: { damage: 0, range: 0, fireRate: 20, mobility: 75, control: 60 },
  field: { damage: 0, range: 80, fireRate: 0, mobility: 50, control: 50 },
};
const SUBTITLE: Record<string, string> = { primary: "assault", secondary: "pistol", melee: "blade", lethal: "grenade", tactical: "grenade", field: "drone" };

const lo0 = () => ({ loadout: 0, slot: "primary", attachment: "", fitted: new Set<string>(POINTS.map(([k]) => k)) });
let lo = lo0();

/** A loadout (Call of Duty, Apex Legends): the slots, the selected item large with its attachments called out, its figures. */
export function loadoutScreen(): void {
  frame("Loadout", L("loadout.title"), () => {
    const slots: LoadoutSlot[] = SLOTS.map(([key, a, lvl, locked]) => ({
      key,
      label: L(`loadout.slot.${key}`),
      name: L(`loadout.item.${key}`),
      detail: locked ? L("loadout.unlocks", { n: 20 }) : lvl > 0 ? L("loadout.level", { n: lvl }) : "",
      image: art(a),
      rarity: key === "primary" ? "rare" : "common",
      locked,
    }));
    const primary = lo.slot === "primary";
    const attachments: LoadoutAttachment[] = primary ? POINTS.map(([key, a, x, y, rarity]) => {
      const on = lo.fitted.has(key);
      return { key, label: L(`loadout.point.${key}`), name: on ? L(`loadout.att.${key}`) : "", image: on ? art(a) : null, rarity: on ? rarity : "none", rarityLabel: on ? L(`loadout.rarity.${rarity}`) : "", x, y };
    }) : [];
    // The game's sums: the slot's figures and what each fitted attachment changes.
    const base = FIGURES[lo.slot] ?? {};
    const stats: LoadoutStat[] = Object.keys(base).map((f) => {
      const delta = primary ? POINTS.filter(([k]) => lo.fitted.has(k)).reduce((n, p) => n + (p[5][f] ?? 0), 0) : 0;
      return { key: f, label: L(`loadout.stat.${f}`), value: (base[f] ?? 0) + delta, delta, max: 100 };
    });
    const sel = SLOTS.find(([k]) => k === lo.slot) ?? SLOTS[0]!;
    const r = shooter.loadout("Loadout", {
      loadouts: [1, 2, 3, 4, 5].map((n) => L("loadout.tab", { n })),
      loadout: lo.loadout,
      balance: 1250,
      slots,
      slot: lo.slot,
      image: art(sel[1]),
      aspect: primary ? 1024 / 651 : 0,
      name: L(`loadout.item.${lo.slot}`),
      subtitle: [L(`loadout.class.${SUBTITLE[lo.slot] ?? "assault"}`), ...(primary ? [L("loadout.rarity.rare")] : []), ...(sel[2] > 0 ? [L("loadout.level", { n: sel[2] })] : [])],
      rarity: primary ? "rare" : "common",
      rarityWord: primary ? 1 : -1,
      description: L(`loadout.desc.${lo.slot}`),
      attachments,
      attachment: lo.attachment,
      capacity: primary ? POINTS.length : 0,
      stats,
      statsLabel: L("loadout.stats"),
      attachmentsLabel: L("loadout.attachments"),
      emptyLabel: L("loadout.empty"),
      removeLabel: L("loadout.remove"),
      lockedLabel: L("loadout.locked"),
    });
    // The game's side: a tab switches loadouts, a slot shows its item; Remove takes the attachment off (its point empty).
    if (r.loadout !== null) lo.loadout = r.loadout;
    if (r.slot !== null) {
      lo.slot = r.slot;
      lo.attachment = "";
    }
    if (r.attachment !== null) lo.attachment = r.attachment;
    if (r.remove !== null) lo.fitted.delete(r.remove);
  }, [{ action: "tabNext", hint: L("loadout.next") }, { action: "back", hint: L("hint.back") }]);
}

// ── deck builder: the collection and the deck ──────────────────────────

/** The collection: key, cost, kind, tier, copies owned. Art `cards/<key>`. */
const COLLECTION: Array<[string, number, string, Rarity, number]> = [
  ["strike", 1, "spell", "common", 3], ["defend", 1, "spell", "common", 3], ["bash", 2, "spell", "common", 2], ["anger", 0, "spell", "uncommon", 2],
  ["inflame", 2, "spell", "rare", 1], ["wound", 0, "spell", "common", 0], ["fireball", 4, "spell", "rare", 2], ["bulwark", 3, "unit", "common", 2],
  ["rogue", 2, "unit", "uncommon", 2], ["potion", 1, "relic", "common", 3], ["storm", 6, "spell", "epic", 1], ["serpent", 3, "unit", "uncommon", 2],
  ["ice", 2, "spell", "uncommon", 2], ["horn", 4, "relic", "rare", 1], ["golem", 7, "unit", "legendary", 1], ["owl", 1, "unit", "common", 2],
  ["scroll", 5, "relic", "epic", 0], ["egg", 9, "relic", "legendary", 1],
];
const DECK0: Record<string, number> = { strike: 2, defend: 2, bash: 1, fireball: 2, bulwark: 2, rogue: 2, potion: 2, storm: 1, serpent: 2, ice: 2, horn: 1, golem: 1, owl: 2 };

const db0 = () => ({ deck: { ...DECK0 } as Record<string, number>, cost: -1, rarity: "none" as Rarity, kind: "", search: "", sort: "cost", view: "cards" });
let db = db0();

/** A deck builder (Hearthstone, Marvel Snap): the collection's filters and grid, the deck's list, its curve and counts. */
export function deckScreen(): void {
  frame("Deck", L("deck.title"), () => {
    const cards: CollectionCard[] = COLLECTION.map(([key, cost, kind, rarity, owned]) => ({
      key,
      name: L(`deck.card.${key}`),
      cost,
      kind,
      typeLine: L(`deck.kind.${kind}`),
      text: L(`deck.text.${key}`),
      image: art(`cards/${key}` as ArtName),
      rarity,
      owned,
      count: db.deck[key] ?? 0,
      // The legendaries are one to a deck.
      limit: rarity === "legendary" ? 1 : 0,
    }));
    const r = og.ui.deckBuilder("Builder", {
      cards,
      deckName: L("deck.name"),
      deckImage: art("cards/ice"),
      deckSize: 24,
      copyLimit: 2,
      costFilter: db.cost,
      rarityFilter: db.rarity,
      kindFilter: db.kind,
      search: db.search,
      sort: db.sort,
      rarities: (["common", "uncommon", "rare", "epic", "legendary"] as Rarity[]).map((rarity) => ({ rarity, label: L(`deck.rarity.${rarity}`) })),
      kinds: ["unit", "spell", "relic"].map((key) => ({ key, label: L(`deck.kinds.${key}`) })),
      sorts: ["cost", "name", "rarity", "owned"].map((key) => ({ key, label: L(`deck.sort.${key}`) })),
      view: db.view,
      allLabel: L("deck.all"),
      searchLabel: L("deck.search"),
      sortLabel: L("deck.sortBy"),
      collectionLabel: L("deck.cards"),
      deckLabel: L("deck.mine"),
      curveLabel: L("deck.curve"),
      kindsLabel: L("deck.types"),
      doneLabel: L("deck.done"),
      fullLabel: L("deck.full"),
      limitLabel: L("deck.limit"),
      unownedLabel: L("deck.unowned"),
      emptyLabel: L("deck.none"),
    });
    // The game's side: an add or a removal changes the deck; the filters' state is the game's to keep.
    if (r.add !== null) db.deck[r.add] = (db.deck[r.add] ?? 0) + 1;
    if (r.remove !== null) db.deck[r.remove] = Math.max(0, (db.deck[r.remove] ?? 0) - 1);
    if (r.costFilter !== null) db.cost = r.costFilter;
    if (r.rarityFilter !== null) db.rarity = r.rarityFilter;
    if (r.kindFilter !== null) db.kind = r.kindFilter;
    if (r.search !== null) db.search = r.search;
    if (r.sort !== null) db.sort = r.sort;
    if (r.view !== null) db.view = r.view;
  }, [{ action: "back", hint: L("hint.back") }]);
}

// ── VN gallery: the event pictures and the route chart ─────────────────

/** The pictures: key, chapter, art, its other versions, locked. */
const CGS: Array<[string, string, ArtName | null, ArtName[], boolean]> = [
  ["riverboat", "one", "cg/riverboat", ["cg/riverboat-night"], false], ["lantern", "one", "cg/lantern", [], false],
  ["library", "two", "cg/library", [], false], ["storm", "two", "cg/storm-ferry", [], true],
  ["duel", "three", "cg/duel", ["cg/duel-dusk"], false], ["dawn", "three", "cg/dawn-gate", [], true],
];
const vg0 = () => ({ chapter: "", viewing: "", variant: 0, route: "ferry" });
let vg = vg0();

/** The gallery's viewer is open: its bar takes the foot, so the kit's Gallery button stands at the top. */
export const cgViewing = (): boolean => vg.viewing !== "";

/** A CG gallery (Steins;Gate, Clannad): the chapters, the pictures with the unseen veiled, and the viewer. */
export function cgGalleryScreen(): void {
  const viewing = vg.viewing !== "";
  frame("Gallery", L("vn.gallery.title"), () => {
    const entries: CgEntry[] = CGS.map(([key, chapter, a, vs, locked]) => ({
      key,
      chapter,
      title: L(`vn.cg.${key}`),
      image: art(a ?? undefined),
      variants: vs.map((v) => art(v)).filter((x): x is never => x !== null),
      locked,
      hint: locked ? L(`vn.hint.${key}`) : "",
      fresh: key === "duel",
    }));
    // The gallery seated on the pack's panel, as a game's extras menu is, not straight on the scene.
    let r!: ReturnType<typeof og.ui.cgGallery>;
    const t = og.ui.frameTheme();
    og.ui.column("Seat", { width: grow, height: grow, padding: viewing ? { left: 0, top: 0, right: 0, bottom: 0 } : { left: t.space.md, top: t.space.md, right: t.space.md, bottom: t.space.md }, background: viewing ? undefined : { token: "panel" }, radius: t.radius.panel, frame: viewing ? undefined : "panel" } as never, () => {
    r = og.ui.cgGallery("Cg", {
      title: L("vn.gallery.title"),
      chapters: [{ key: "", label: L("vn.all") }, ...["one", "two", "three"].map((key) => ({ key, label: L(`vn.chapter.${key}`) }))],
      chapter: vg.chapter,
      entries,
      viewing: vg.viewing,
      variant: vg.variant,
      countLabel: L("vn.seen"),
      lockedLabel: L("vn.locked"),
      newLabel: L("vn.new"),
      closeLabel: L("vn.close"),
      prevLabel: L("vn.prev"),
      nextLabel: L("vn.next"),
    });
    });
    // The game's side: the viewer opens on the picture's first version; another picture or version replaces it.
    if (r.chapter !== null) vg.chapter = r.chapter;
    if (r.open !== null || r.step !== null) {
      vg.viewing = (r.open ?? r.step)!;
      vg.variant = 0;
    }
    if (r.variant !== null) vg.variant = r.variant;
    if (r.close) vg.viewing = "";
  }, viewing ? [] : [{ action: "tabNext", hint: L("vn.nextChapter") }, { action: "back", hint: L("hint.back") }], viewing);
}

/** The story's scenes as the save knows them: key, column, lane, art, seen, locked, the ending (good or bad). */
const SCENES: Array<[string, number, number, ArtName | null, boolean, boolean, "" | "good" | "bad"]> = [
  ["arrival", 0, 0, "cg/riverboat", true, false, ""], ["lanterns", 1, 0, "cg/lantern", true, false, ""], ["tower", 1, 1, "cg/library", true, false, ""],
  ["ferry", 2, 0, "cg/storm-ferry", true, false, ""], ["rooftop", 2, 1, "cg/duel", true, false, ""], ["dawn", 3, 0, "cg/dawn-gate", false, false, "good"],
  ["drowned", 3, 1, null, true, false, "bad"], ["oath", 3, 2, null, false, true, "good"],
];
const BRANCHES: Array<[string, string, string, boolean]> = [
  ["arrival", "lanterns", "follow", true], ["arrival", "tower", "study", true], ["lanterns", "ferry", "", true], ["tower", "rooftop", "", true],
  ["ferry", "dawn", "trust", false], ["ferry", "drowned", "storm", true], ["rooftop", "oath", "", false],
];

/** A route flowchart (Zero Escape, Detroit: Become Human): the scenes by chapter and route, the choices, the endings. */
export function routeChartScreen(): void {
  frame("Routes", L("vn.routes.title"), () => {
    const scenes: RouteScene[] = SCENES.map(([key, column, lane, a, seen, locked, ending]) => ({
      key,
      column,
      lane,
      title: L(`vn.scene.${key}`),
      image: art(a ?? undefined),
      seen,
      locked,
      current: key === "ferry",
      ending: ending ? L(`vn.ending.${ending}`) : "",
      good: ending === "good",
    }));
    const r = og.ui.routeChart("Chart", {
      title: L("vn.routes.chart"),
      chapters: ["one", "two", "three", "four"].map((key) => ({ key, label: L(`vn.chapter.${key}`) })),
      scenes,
      links: BRANCHES.map(([from, to, choice, taken]) => ({ from, to, label: choice ? L(`vn.choice.${choice}`) : "", taken })),
      selected: vg.route,
      lockedLabel: L("vn.locked"),
      jumpLabel: L("vn.jump"),
      currentLabel: L("vn.now"),
      seenLabel: L("vn.scenes"),
    });
    if (r.selected !== null) vg.route = r.selected;
    // A jump loads the scene from the save (og.story, loaded in setup): the preview stays on the chart.
  }, [{ action: "back", hint: L("hint.back") }]);
}

// ── settings: the tree as data ─────────────────────────────────────────

/** The keyboard layouts the controls page offers: each key's label by its physical code (the bindings keep the code). */
const LAYOUTS: Record<string, Array<[string, string]>> = {
  qwerty: [],
  azerty: [["KeyQ", "A"], ["KeyA", "Q"], ["KeyW", "Z"], ["KeyZ", "W"], ["KeyM", ","], ["Semicolon", "M"]],
  qwertz: [["KeyY", "Z"], ["KeyZ", "Y"]],
};
const QUALITY = ["low", "medium", "high"] as const;
const CAPS = ["display", "fps30", "fps60", "fps120"] as const;
const ASPECTS = ["16:9", "16:10", "21:9", "32:9"] as const;
const LANGS = ["en", "ja", "ko", "zh-CN", "ar"] as const;
/** Each language named in its own script whatever the locale (as src/main.ts's LANGUAGES, whose characters every locale's subset carries). */
const ENDONYMS: Record<string, string> = { en: "English", ja: "日本語", ko: "한국어", "zh-CN": "简体中文", ar: "العربية" };
/** The values as applied, and the pending ones on top of them. */
type Values = Record<string, number>;
const DEFAULTS: Values = {
  quality: 1, scale: 1, cap: 0, vsync: 1, fullscreen: 0, aspect: 0, safe: 0.9,
  master: 1, music: 0.8, effects: 1, voice: 0.9, subtitles: 1,
  layout: 0, sensitivity: 0.5, invert: 0, raw: 1,
  textSize: 0, contrast: 0, reduceMotion: 0, colorblind: 0,
  language: 0,
};
const se0 = () => ({ page: "graphics", applied: { ...DEFAULTS } as Values, pending: {} as Values, confirm: 0, before: null as Values | null });
let se = se0();

/** What Apply keeps (og.settings, by src/main.ts): the tree's values, the display they make and the language's code. */
export type TreeSettings = { values: Values; display: DisplaySettings; language: string };

/** Puts the tree's values back from og.settings (a relaunch): every key the tree knows, the rest left at their defaults. */
export function restoreSettingsTree(values: unknown): void {
  if (!values || typeof values !== "object") return;
  for (const [k, n] of Object.entries(values as Record<string, unknown>)) if (k in DEFAULTS && typeof n === "number" && Number.isFinite(n)) se.applied[k] = n;
}

/**
 * The settings tree (B-settings, roadmap G1-7): every page's entries as data; display changes confirmed or reverted.
 * Returns the settings Apply made (for the game to save), else null.
 */
export function settingsTreeScreen(world: World): TreeSettings | null {
  let applied: TreeSettings | null = null;
  const v = (k: string): number => se.pending[k] ?? se.applied[k] ?? 0;
  const dirty = (k: string): boolean => k in se.pending && se.pending[k] !== se.applied[k];
  const E = (key: string, kind: SettingKind, more: Partial<SettingEntry> = {}): SettingEntry => ({
    key, kind, label: L(`set.${key}`), description: L(`set.${key}.about`), dirty: dirty(key),
    on: v(key) > 0, value: v(key), index: v(key), ...more,
  });
  const words = (pre: string, list: readonly string[]) => list.map((w) => L(`${pre}.${w}`));
  const sec = (k: string) => L(`set.section.${k}`);
  // The display's state counts down in the preview's own ticks (a game's clock is its own).
  if (se.confirm > 0) se.confirm = Math.max(0, se.confirm - 1 / 60);
  const pages = [
    { key: "graphics", label: L("set.page.graphics"), entries: [
      E("quality", "choice", { options: words("set.quality", QUALITY), section: sec("display"), display: true, preview: art("photo/vivid") }),
      E("scale", "slider", { min: 0.5, max: 1, step: 0.05, unit: "%", display: true, preview: art("photo/shot") }),
      E("cap", "choice", { options: words("set.cap", CAPS), display: true }),
      E("vsync", "toggle", { display: true }),
      E("fullscreen", "toggle", { display: true }),
      E("aspect", "choice", { options: [...ASPECTS], section: sec("screen"), preview: art("scenes/jade") }),
      E("safe", "slider", { min: 0.8, max: 1, step: 0.01, unit: "%" }),
    ] },
    { key: "audio", label: L("set.page.audio"), entries: [
      E("master", "slider", { unit: "%", step: 0.05 }), E("music", "slider", { unit: "%", step: 0.05 }), E("effects", "slider", { unit: "%", step: 0.05 }),
      E("voice", "slider", { unit: "%", step: 0.05 }), E("subtitles", "toggle"),
    ] },
    { key: "controls", label: L("set.page.controls"), entries: [
      E("layout", "choice", { options: words("set.layout", ["qwerty", "azerty", "qwertz"]), section: sec("keyboard") }),
      E("moveUp", "binding", { action: "moveUp" }), E("moveLeft", "binding", { action: "moveLeft" }), E("jumpKey", "binding", { action: "jump" }),
      E("sensitivity", "slider", { step: 0.05, unit: "%", section: sec("mouse") }), E("invert", "toggle"), E("raw", "toggle"),
      E("resetBindings", "action"),
    ] },
    { key: "access", label: L("set.page.access"), entries: [
      E("textSize", "choice", { options: ["100%", "125%", "150%"] }), E("contrast", "toggle"), E("reduceMotion", "toggle"),
      E("colorblind", "choice", { options: words("set.colorblind", ["off", "protan", "deutan", "tritan"]) }),
    ] },
    { key: "language", label: L("set.page.language"), entries: [E("language", "choice", { options: LANGS.map((l) => ENDONYMS[l]!) })] },
    { key: "hud", label: L("set.page.hud"), hud: true },
  ];
  frame("Settings", L("set.title"), () => {
    const r = og.ui.settingsTree(world, "Tree", {
      title: L("set.title"),
      pages,
      page: se.page,
      pending: Object.keys(se.pending).some(dirty),
      confirmSeconds: se.confirm,
      confirmTotal: 15,
      applyLabel: L("set.apply"),
      revertLabel: L("set.revert"),
      defaultsLabel: L("set.defaults"),
      confirmTitle: L("set.confirm"),
      keepLabel: L("set.keep"),
      revertingLabel: L("set.reverting"),
      hudNote: L("set.hudNote"),
      editHudLabel: L("set.editHud"),
      changedLabel: L("set.changed"),
    }, () => {});
    // The game's side: changes wait as pending; Apply makes them the settings (the display through og.render) and a
    // display change asks to be kept; Revert drops them; the countdown at 0, or its Revert, puts the display back.
    if (r.page !== null) se.page = r.page;
    if (r.change !== null) {
      const k = r.change.key;
      const e = pages.flatMap((p) => p.entries ?? []).find((x) => x.key === k);
      se.pending[k] = e?.kind === "toggle" ? (r.change.on ? 1 : 0) : e?.kind === "choice" ? r.change.index : r.change.value;
    }
    const display = (vals: Values): DisplaySettings => ({ quality: QUALITY[vals.quality ?? 1], renderScale: vals.scale ?? 1, frameCap: CAPS[vals.cap ?? 0], fullscreen: (vals.fullscreen ?? 0) > 0 });
    if (r.apply) {
      const displayKeys = pages[0]!.entries!.filter((e) => e.display).map((e) => e.key!);
      const before = { ...se.applied };
      se.applied = { ...se.applied, ...se.pending };
      se.pending = {};
      og.render.setDisplay(world, display(se.applied));
      og.input.setKeyboardLayout((LAYOUTS[["qwerty", "azerty", "qwertz"][se.applied.layout ?? 0]!] ?? []).map(([code, label]) => ({ code, label })));
      applied = { values: { ...se.applied }, display: display(se.applied), language: LANGS[se.applied.language ?? 0] ?? "en" };
      if (displayKeys.some((k) => before[k] !== se.applied[k])) {
        se.before = before;
        se.confirm = 15;
      }
    }
    if (r.revert) se.pending = {};
    if (r.defaults) for (const e of pages.find((p) => p.key === se.page)?.entries ?? []) if (e.key! in DEFAULTS) se.pending[e.key!] = DEFAULTS[e.key!]!;
    if (r.keep) {
      se.confirm = 0;
      se.before = null;
    }
    if ((r.confirmRevert || (se.before && se.confirm === 0 && !r.keep)) && se.before) {
      se.applied = se.before;
      se.before = null;
      se.confirm = 0;
      og.render.setDisplay(world, display(se.applied));
    }
  }, [{ action: "tabNext", hint: L("set.nextPage") }, { action: "back", hint: L("hint.back") }]);
  return applied;
}

/** The settings preview's input actions, defined in setup: the controls page rebinds them. */
export function defineGenres3Actions(world: World): void {
  og.input.defineAction(world, { name: "moveUp", label: L("set.moveUp"), bindings: [{ control: "KeyW" }, { control: "ArrowUp" }] });
  og.input.defineAction(world, { name: "moveLeft", label: L("set.moveLeft"), bindings: [{ control: "KeyA" }, { control: "ArrowLeft" }] });
  if (!og.input.actions(world).some((a) => a.name === "jump")) og.input.defineAction(world, { name: "jump", label: L("set.jumpKey"), bindings: [{ control: "Space" }] });
}

/** The previews' commands: `pat:<photo|loadout|deck|vn|set>:<key>:<value>`; false when the area is not one of these. */
export function genres3Command(area: string, key: string, value: string): boolean {
  if (area === "photo") {
    if (key === "tab") ph.tab = value;
    else if (key === "set") {
      const [k, n] = value.split("=");
      ph.values[k ?? ""] = Number(n) || 0;
    } else if (key === "filter") ph.filter = value;
    else if (key === "frame") ph.frame = value;
    else if (key === "grid") ph.grid = value !== "off";
    else if (key === "hidden") ph.hidden = value !== "off";
    else if (key === "reset") ph = ph0();
    else return false;
  } else if (area === "loadout") {
    if (key === "slot") lo.slot = value;
    else if (key === "tab") lo.loadout = Number(value) || 0;
    else if (key === "attachment") lo.attachment = value;
    else if (key === "remove") lo.fitted.delete(value);
    else if (key === "reset") lo = lo0();
    else return false;
  } else if (area === "deck") {
    if (key === "cost") db.cost = Number(value);
    else if (key === "rarity") db.rarity = value as Rarity;
    else if (key === "kind") db.kind = value;
    else if (key === "search") db.search = value.replace(/_/g, " ");
    else if (key === "view") db.view = value;
    else if (key === "full") db.deck = { ...DECK0, anger: 1, egg: 1 };
    else if (key === "reset") db = db0();
    else return false;
  } else if (area === "vn") {
    if (key === "chapter") vg.chapter = value;
    else if (key === "view") {
      vg.viewing = value;
      vg.variant = 0;
    } else if (key === "variant") vg.variant = Number(value) || 0;
    else if (key === "route") vg.route = value;
    else if (key === "reset") vg = vg0();
    else return false;
  } else if (area === "set") {
    if (key === "page") se.page = value;
    else if (key === "pending") {
      const [k, n] = value.split("=");
      se.pending[k ?? ""] = Number(n) || 0;
    } else if (key === "confirm") {
      se.before = { ...se.applied };
      se.confirm = Number(value) || 0;
    } else if (key === "reset") se = se0();
    else return false;
  } else return false;
  return true;
}

/** Every preview of this file back to its start (pat:reset). */
export function resetGenres3(): void {
  ph = ph0();
  lo = lo0();
  db = db0();
  vg = vg0();
  se = se0();
}
