// The genre patterns' preview screens of the last U6d round (docs/ui.md §11, U6d r42): photo mode and a loadout, each built from its
// pattern over the kit's fixtures with the game's side done as a game would (the camera moved, a filter drawn, the shot
// taken). `pat:<area>:<key>:<value>` sets a preview's state so the playtests (tests/patterns.ts, genres3Script) can
// draw each one.
//
// Names (players, places' proper nouns) are invented and stay as they are in every locale; every other word is the
// game's, keyed in assets/strings.

import { og } from "@origingame/engine";
import { art, L, type ArtName } from "./patterns.ts";
import { frame } from "./genres.ts";
import type { LoadoutAttachment, LoadoutSlot, LoadoutStat, PhotoControl, PhotoLook, Rarity, Size } from "@origingame/engine";

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
      og.ui.setDraw({ shape: { kind: "rect" }, fills: tex ? [{ kind: "image", texture: tex, fit: "cover", focal: { x: 0.5, y: 0.5 } }] : [{ kind: "solid", color: { token: "backdrop" } }], opacity: alpha } as never);
      og.ui.props({ position: "absolute", insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true, transform: { scaleX: k, scaleY: k, rotate: ((v.roll ?? 0) * Math.PI) / 180, y: (v.height ?? 0) * vp.height * 0.06 } } as never);
    };
    layer("Picture", ph.filter === "none" ? plain : filtered, 1);
    if (ph.filter !== "none") layer("Plain", plain, 1 - ph.strength);
    const ev = v.exposure ?? 0;
    og.ui.column("Light", { width: grow, height: grow }, () => {});
    og.ui.setDraw({
      shape: { kind: "rect" },
      fills: [
        { kind: "solid", color: ev >= 0 ? { r: 1, g: 1, b: 1, a: ev * 0.18 } : { r: 0, g: 0, b: 0, a: -ev * 0.3 } },
        { kind: "gradient", type: "radial", from: { x: 0.5, y: 0.5 }, to: { x: 1, y: 1 }, stops: [{ offset: 0.35, color: { r: 0, g: 0, b: 0, a: 0 } }, { offset: 1, color: { r: 0, g: 0, b: 0, a: 0.9 * (v.vignette ?? 0) } }] },
      ],
    } as never);
    og.ui.props({ position: "absolute", insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true } as never);
  });
  og.ui.props({ inert: true } as never);

  let exit = false;
  og.ui.anchor("Photo", { anchor: "topLeft", margin: 0, width: px(vp.width - sa.left - sa.right), height: px(vp.height - sa.top - sa.bottom), direction: "column" }, () => {
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
    const r = og.ui.loadout("Loadout", {
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

/** The previews' commands: `pat:<photo|loadout>:<key>:<value>`; false when the area is not one of these. */
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
  } else return false;
  return true;
}

/** Every preview of this file back to its start (pat:reset). */
export function resetGenres3(): void {
  ph = ph0();
  lo = lo0();
}
