// The style-pack gallery: every screen of the kit in one style, composed into
// one reference sheet per style and surface, in FRAME_SCREENS order without
// the gallery: on a desktop crates/og-ui/references/<tier>/kit-style-<name>.png
// (7 cells a row, each the 1280×720 frame box-filtered to 640×360), on a phone
// in portrait kit-style-<name>-phone.png (10 cells a row, each the 390×844@3
// frame box-filtered to its 390×844 CSS pixels). Each screen is compared with
// its cell of the sheet under og-tests' rule (frames.ts).
// `OG_UI_CAPTURE=1 og test .` records the sheets instead.

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { deflateSync } from "node:zlib";
import { playtest, type PlaytestChecks, type PlaytestDesc, type PlaytestRun } from "@origingame/engine/playtest";
import { decodePng, difference, outDir, referencesDir, type Image } from "./frames.ts";
import { framesScript, STYLE_SETTLE } from "./script.ts";

/** How a surface's frames become a sheet: the frame size, the box filter, the columns. */
export interface Surface {
  readonly size: string;
  readonly viewport: string;
  readonly factor: number;
  readonly cols: number;
  readonly suffix: string;
}

export const SURFACES: Record<"desktop" | "phone", Surface> = {
  desktop: { size: "1280x720", viewport: "", factor: 2, cols: 7, suffix: "" },
  phone: { size: "1170x2532", viewport: "0.0:viewport:phone;", factor: 3, cols: 10, suffix: "-phone" },
};

/** An n× box filter (1280×720 by 2 to 640×360; 1170×2532 by 3 to 390×844). */
export function shrink(img: Image, n: number): Image {
  const w = Math.floor(img.width / n);
  const h = Math.floor(img.height / n);
  const out = new Uint8Array(w * h * 4);
  const area = n * n;
  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      for (let k = 0; k < 4; k++) {
        let sum = 0;
        for (let dy = 0; dy < n; dy++) for (let dx = 0; dx < n; dx++) sum += img.rgba[((n * y + dy) * img.width + n * x + dx) * 4 + k]!;
        out[(y * w + x) * 4 + k] = Math.floor((sum + (area >> 1)) / area);
      }
    }
  }
  return { width: w, height: h, rgba: out };
}

/** Cell `i` of a sheet with `cols` columns of `cw`×`ch` cells. */
export function cell(sheet: Image, i: number, cols: number, cw: number, ch: number): Image {
  const x0 = (i % cols) * cw;
  const y0 = Math.floor(i / cols) * ch;
  const out = new Uint8Array(cw * ch * 4);
  for (let y = 0; y < ch; y++) {
    out.set(sheet.rgba.subarray(((y0 + y) * sheet.width + x0) * 4, ((y0 + y) * sheet.width + x0 + cw) * 4), y * cw * 4);
  }
  return { width: cw, height: ch, rgba: out };
}

function crc32(bytes: Uint8Array): number {
  let c = ~0;
  for (const b of bytes) {
    c ^= b;
    for (let k = 0; k < 8; k++) c = (c >>> 1) ^ (0xedb88320 & -(c & 1));
  }
  return ~c >>> 0;
}

/** An 8-bit RGB PNG (filter 0). */
export function encodePng(img: Image): Uint8Array {
  const raw = new Uint8Array((img.width * 3 + 1) * img.height);
  for (let y = 0; y < img.height; y++) {
    const row = y * (img.width * 3 + 1);
    for (let x = 0; x < img.width; x++) {
      for (let k = 0; k < 3; k++) raw[row + 1 + x * 3 + k] = img.rgba[(y * img.width + x) * 4 + k]!;
    }
  }
  const chunk = (type: string, data: Uint8Array): Uint8Array => {
    const out = new Uint8Array(12 + data.length);
    const view = new DataView(out.buffer);
    view.setUint32(0, data.length);
    out.set(new TextEncoder().encode(type), 4);
    out.set(data, 8);
    view.setUint32(8 + data.length, crc32(out.subarray(4, 8 + data.length)));
    return out;
  };
  const ihdr = new Uint8Array(13);
  const v = new DataView(ihdr.buffer);
  v.setUint32(0, img.width);
  v.setUint32(4, img.height);
  ihdr.set([8, 2, 0, 0, 0], 8);
  const parts = [new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10]), chunk("IHDR", ihdr), chunk("IDAT", deflateSync(raw, { level: 9 })), chunk("IEND", new Uint8Array())];
  const total = parts.reduce((n, p) => n + p.length, 0);
  const png = new Uint8Array(total);
  let at = 0;
  for (const p of parts) {
    png.set(p, at);
    at += p.length;
  }
  return png;
}

/**
 * Compose `files` (frames in the run's output) into the sheet `name`, each
 * box-filtered by the surface's factor, `surface.cols` a row, and compare each
 * cell with the reference sheet's under og-tests' rule (`label` names them in
 * the asserts). `OG_UI_CAPTURE=1` records the sheet instead.
 */
export function sheetCheck(run: PlaytestRun, t: PlaytestChecks, files: readonly string[], name: string, surface: Surface, label: string): void {
  // The sheet is built from frames: without a renderer each is reported unchecked.
  t.frame(files, () => {
    const cells = files.map((f) => shrink(decodePng(readFileSync(join(outDir(), f))), surface.factor));
    const cw = cells[0]!.width;
    const ch = cells[0]!.height;
    const cols = surface.cols;
    const sheet: Image = { width: cols * cw, height: Math.ceil(cells.length / cols) * ch, rgba: new Uint8Array(0) };
    sheet.rgba = new Uint8Array(sheet.width * sheet.height * 4);
    cells.forEach((c, i) => {
      const x0 = (i % cols) * cw;
      const y0 = Math.floor(i / cols) * ch;
      for (let y = 0; y < ch; y++) sheet.rgba.set(c.rgba.subarray(y * cw * 4, (y + 1) * cw * 4), ((y0 + y) * sheet.width + x0) * 4);
    });
    writeFileSync(join(outDir(), name), encodePng(sheet));
    const refs = referencesDir();
    const own = join(refs, run.tier);
    if (process.env.OG_UI_CAPTURE !== undefined) {
      mkdirSync(own, { recursive: true });
      writeFileSync(join(own, name), encodePng(sheet));
      t.ok(true, `recorded ${join(own, name)}`);
      return;
    }
    const dir = existsSync(join(own, name)) ? own : join(refs, "metal");
    if (dir !== own) process.stdout.write(`  note: no ${run.tier} reference for ${label}; comparing with metal\n`);
    const refPath = join(dir, name);
    if (!existsSync(refPath)) {
      t.ok(false, `${refPath} is missing: record it with OG_UI_CAPTURE=1 og test templates/ui-kit`);
      return;
    }
    const ref = decodePng(readFileSync(refPath));
    t.equal(`${ref.width}x${ref.height}`, `${sheet.width}x${sheet.height}`, "the sheet has every screen");
    files.forEach((f, i) => {
      const d = difference(cells[i]!, cell(ref, i, cols, cw, ch));
      t.ok(d.outliers <= 0.005 && d.rmse <= 0.01, `${label} ${f} matches its cell: ${(d.outliers * 100).toFixed(3)} % outliers (≤ 0.5 %), RMSE ${d.rmse.toFixed(5)} (≤ 0.01)`);
    });
  });
}

/** A capture's UI dump (`<screen>.settle.json`): what is still moving. */
interface Settle {
  motion: { path: string; moving: string[]; leaving: boolean }[];
}

/** What was still entering or leaving when `screen`'s frame was taken (withSettleDumps), joined; "" when settled. */
export function unsettled(run: PlaytestRun, screen: string): string {
  return run
    .json<Settle>(`${screen}.settle.json`)
    .motion.filter((m) => m.leaving || m.moving.includes("presence"))
    .map((m) => `${m.path} (${m.leaving ? "leaving" : m.moving.join(", ")})`)
    .join("; ");
}

/**
 * The script with a dump_ui just before each screenshot, at its tick: the
 * playtest refuses a capture taken while an entrance (a layer's, a
 * transition's, a stagger, the cyber pack's glitch) or an exit is still in
 * flight, since a pack cannot be judged from a frame caught mid-presence.
 */
export function withSettleDumps(commands: string): string {
  return commands
    .split(";")
    .flatMap((step) => {
      const m = /^([\d.]+):screenshot:(.+)\.png$/.exec(step);
      return m ? [`${m[1]}:dump_ui:${m[2]}.settle.json`, step] : [step];
    })
    .join(";");
}

/**
 * States of a pack the art director approved one by one, beyond its screens:
 * each a gear frame after its commands, on the desktop, composed into
 * kit-style-<name>-states.png (a phone's, PHONE_STYLE_STATES, into
 * kit-style-<name>-phone-states.png). Fantasy's and scifi's selected bag cells
 * (2026-11-04): the selection, not the tier rim, is a selected cell's one
 * and brightest edge, for a rare (the helm) and a legendary (the greatsword),
 * at 100 and 150 % text; the idle frame is the unselected side of each pair
 * (pat:reset selects the helm, so idle clears it). `selected` is the bag item
 * the frame's dump must show selected, and the only one ("" none).
 */
export interface StyleState {
  name: string;
  commands: readonly string[];
  /** The bag item the frame's dump must show selected, and the only one ("" none); unset: not a bag state. */
  selected?: string;
}

export const STYLE_STATES: Record<string, readonly StyleState[]> = Object.fromEntries(
  ["fantasy", "scifi"].map((style) => [
    style,
    ["100", "150"].flatMap((scale) => [
      { name: `state-gear-idle-${scale}`, commands: [`demo:text:${scale}`, "show:gear", "pat:reset", "pat:select:"], selected: "" },
      { name: `state-gear-rare-${scale}`, commands: ["pat:select:helm"], selected: "helm" },
      { name: `state-gear-legendary-${scale}`, commands: ["pat:select:greatsword"], selected: "greatsword" },
    ]),
  ]),
);

/**
 * The phone's states, composed into kit-style-<name>-phone-states.png at
 * the phone's own DPR (the surface's 390×844@3): the horror HUD at 100 and
 * 150 % text in scifi, storybook and fantasy (art, 2026-11-04: the world
 * prompt's name over the scene in the scheme's light ink, its gap above the
 * chip, the d-pad clear of the battery), and the race HUD at 100 and 150 % in
 * fantasy and craft (art, 2026-11-04: the tach's speed at the whole text size,
 * its unit and gear readable under the dial). The phone sheet's own cells are
 * at 100 % only.
 */
export const PHONE_STYLE_STATES: Record<string, readonly StyleState[]> = Object.fromEntries(
  [
    ["scifi", ["horror"]],
    ["storybook", ["horror"]],
    ["fantasy", ["horror", "race"]],
    ["craft", ["race"]],
  ].map(([style, screens]) => [
    style,
    (screens as string[]).flatMap((screen) =>
      ["100", "150"].map((scale) => ({ name: `state-${screen}-${scale}`, commands: [`demo:text:${scale}`, `show:${screen}Hud`] })),
    ),
  ]),
);

/** The states' script from `t0` (their frames settle as a screen's): its commands, without a quit, and its files. */
function statesScript(t0: number, states: readonly StyleState[]): { commands: string; files: string[]; end: number } {
  const steps: string[] = [];
  const files: string[] = [];
  let t = t0;
  for (const st of states) {
    st.commands.forEach((c, k) => steps.push(`${(t + 0.05 * k).toFixed(2)}:${c}`));
    // The settle counts from the last command (the selection a state makes), as a screen's from its show.
    t += 0.05 * (st.commands.length - 1) + (STYLE_SETTLE.default ?? 1);
    steps.push(`${t.toFixed(2)}:screenshot:${st.name}.png`);
    files.push(`${st.name}.png`);
    t += 0.1;
  }
  return { commands: steps.join(";"), files, end: t };
}

/** The playtest of one style pack on a surface (desktop by default). */
export function stylePlaytest(style: string, on: "desktop" | "phone" = "desktop"): PlaytestDesc {
  const surface = SURFACES[on];
  const script = framesScript(0.5, STYLE_SETTLE);
  const files = script.files.filter((f) => f !== "gallery.png");
  const name = `kit-style-${style}${surface.suffix}.png`;
  const states = (on === "desktop" ? STYLE_STATES : PHONE_STYLE_STATES)[style] ?? [];
  // The states start once the toasts the screens queued are gone: each stays its 3200 ms (og-ui's ToastDesc
  // default) and leaves on the pack's motion (under a second in every pack); the screens' frames end sooner.
  const toasted = Math.max(0, ...[...script.commands.matchAll(/(?:^|;)([\d.]+):toast:/g)].map((m) => Number(m[1]) + 3.2 + 1.0));
  const extra = statesScript(Math.max(script.end, toasted), states);
  const commands = states.length > 0 ? `${script.commands.replace(/;[\d.]+:quit$/, "")};${extra.commands};${extra.end.toFixed(2)}:quit` : script.commands;
  return playtest({
    name: `every screen in the ${style} style pack${on === "phone" ? " on a phone" : ""} matches its cell of ${name}`,
    size: surface.size,
    commands: `${surface.viewport}0.2:style:${style};${withSettleDumps(commands)}`,
    check(run, t) {
      t.equal(run.warnings.join("; "), "", "the script ran without warnings");
      t.ok(run.quit, "the script reached quit");
      for (const f of [...files, ...extra.files]) {
        const screen = f.replace(/\.png$/, "");
        t.equal(unsettled(run, screen), "", `${style} ${screen} is captured with every entrance and exit settled`);
      }
      sheetCheck(run, t, files, name, surface, style);
      // Each state shows its selection and no other, read from the dump at its frame's tick.
      for (const st of states) {
        if (st.selected === undefined) continue;
        const shown = run
          .json<{ elements: Array<{ path?: string; states?: string[] }> }>(`${st.name}.settle.json`)
          .elements.filter((e) => /\/Bag\/Cells\/[^/]+$/.test(e.path ?? "") && (e.states ?? []).includes("selected"))
          .map((e) => e.path!.split("/").pop()!);
        t.equal(shown.join(","), st.selected, `${style} ${st.name} has ${st.selected || "nothing"} selected in the bag`);
      }
      if (extra.files.length > 0) sheetCheck(run, t, extra.files, `kit-style-${style}${surface.suffix}-states.png`, surface, `${style}${surface.suffix} states`);
    },
  });
}
