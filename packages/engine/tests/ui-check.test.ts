// `og ui check`'s rules (src/node/ui-check.ts, docs/ui.md §6), each on a
// frame that shows the fact and on the same frame put right: every finding
// names its rule, paths, what was found and the prop that sets it. The
// measures (contrast, text size, touch targets, text scale, primary
// actions, the element count) are numbers held to no bar: each is checked
// for its value, its path and what it leaves out. The rules that wait for
// later phases are listed as such, never dropped.
import { expect, test } from "bun:test";
import { checkFrame, checkLocales, elementPaths, formatText, measureExtremes, measureFrame, measureLayout, measureScale, mergeFindings, neighbour, padFindings, padGraph, RULES, screensFromHelp, screensFromPlaytest, type Rect4, type UiCell, type UiElementDump, type UiFrameSnapshot, type UiMeasure } from "../src/node/ui-check.ts";

const cell: UiCell = { screen: "s", breakpoint: "phonePortrait", locale: "", style: "", scale: 100, highContrast: false };

function el(id: string, parent: string, kind: string, role: string, r: [number, number, number, number], extra: Partial<UiElementDump> = {}): UiElementDump {
  return { id, parent, kind, role, text: "", name: role ? id : "", states: [], disabled: false, clipped: false, x: r[0], y: r[1], width: r[2], height: r[3], ...extra };
}

function frame(over: Partial<UiFrameSnapshot> = {}, vp: Partial<UiFrameSnapshot["viewport"]> = {}): UiFrameSnapshot {
  return {
    viewport: { width: 390, height: 844, dpr: 1, safeArea: [0, 47, 0, 34], coarsePointer: true, highContrast: false, reducedMotion: false, covered: [0, 0, 0, 0], breakpoint: "phonePortrait", ...vp },
    style: "",
    contrast: "standard",
    sounds: { hover: false, press: false, open: false, close: false, success: false, error: false },
    primary: [[0.2, 0.4, 0.9, 1], [0.25, 0.45, 0.95, 1], [0.15, 0.3, 0.8, 1]],
    elements: [],
    overlaps: [],
    stats: { elements: 10, rebuilt: 0, laidOut: 0, keptCommands: 0 },
    draws: 10,
    texts: [],
    fills: [],
    a11y: { focused: "", focusOrder: [], nodes: [] },
    motion: [],
    warnings: [],
    ...over,
  };
}

const rules = (f: UiFrameSnapshot, c: UiCell = cell) => checkFrame(f, c).map((x) => x.rule);
/** One measure's entries of a frame, as [path, value] pairs. */
const measured = (id: string, f: UiFrameSnapshot, c: UiCell = cell, image: { width: number; height: number; rgba: Uint8Array } | null = null) =>
  measureFrame(f, c, { image }).filter((m) => m.measure === id).map((m) => [m.paths.join(" × "), m.value] as const);

test("every §16.5 rule is listed as a finding or a measure, and those waiting for a later phase say so with the reason", () => {
  const kind = new Map(RULES.map((r) => [r.id, r.kind]));
  for (const id of ["overlap", "text-cut", "text-truncated", "pseudo-overflow", "safe-area", "covered", "focus-reach", "focus-trap", "layer-back", "a11y-name", "a11y-locale", "feedback", "motion", "art-slot"]) expect(kind.get(id)).toBe("finding");
  for (const id of ["contrast", "text-size", "touch-target", "text-scale", "primary-actions", "elements"]) expect(kind.get(id)).toBe("measure");
  for (const r of RULES.filter((r) => r.status !== "checked")) expect(r.note ?? "").not.toBe("");
  const text = formatText([]);
  for (const id of ["layer-back", "motion", "art-slot"]) expect(text).toContain(`${id} (not checkable yet)`);
  // No verdict words: a report counts what it found and measured.
  expect(text).not.toMatch(/error|warning/);
});

test("an undeclared overlap names both paths, the area and the prop; nested pairs are one finding", () => {
  const f = frame({
    overlaps: [
      { path: "Hud.top/Timer", other: "Hud.right/Coins", rect: [180, 63, 44, 44], inFlow: false },
      { path: "Hud.top/Timer/Time", other: "Hud.right/Coins/Coin", rect: [192, 75, 20, 20], inFlow: false },
    ],
  });
  const got = checkFrame(f, cell).filter((x) => x.rule === "overlap");
  expect(got.length).toBe(1);
  expect(got[0]!.paths).toEqual(["Hud.top/Timer", "Hud.right/Coins"]);
  expect(got[0]!.measured).toContain("44×44");
  expect(got[0]!.prop).toContain("overlaps");
  expect(rules(frame())).not.toContain("overlap");
});

test("text cut by its clip without an ellipsis is a finding; text that fits is not", () => {
  const els = [el("Panel", "", "panel", "", [0, 100, 100, 40]), el("Name", "Panel", "label", "text", [10, 110, 80, 20], { text: "A very long name" })];
  const cut = frame({ elements: els, texts: [{ text: "A very long name", size: 14, bounds: [10, 110, 130, 20], clip: [0, 100, 100, 40], color: [1, 1, 1, 1] }] });
  expect(rules(cut)).toContain("text-cut");
  const fits = frame({ elements: els, texts: [{ text: "Name", size: 14, bounds: [10, 110, 40, 20], clip: [0, 100, 100, 40], color: [1, 1, 1, 1] }] });
  expect(rules(fits)).not.toContain("text-cut");
});

test("a row a scroll container cuts at its bottom edge is scrolling, not cut text; a line cut top or bottom says by how much", () => {
  // The kit's gallery on a phone: only the top 5 px of row 9's 20 px line show (its visible box is
  // 5 px, so the line's centre lies outside it), and the probe marks the element clipped.
  const line = { text: "inventory", size: 14, bounds: [52, 771, 62.5, 20] as [number, number, number, number], clip: [40, 192, 310, 584] as [number, number, number, number], color: [1, 1, 1, 1] };
  const scrolled = frame({ elements: [el("Row#9", "", "row", "", [40, 660, 310, 52]), el("Name", "Row#9", "label", "text", [52, 771, 62, 5], { text: "inventory", clipped: true })], texts: [line] });
  expect(rules(scrolled)).not.toContain("text-cut");
  // The same cut on a box that is not a scroller's: a finding, measured on the axis it cut.
  const cut = frame({ elements: [el("Name", "", "label", "text", [52, 771, 62, 5], { text: "inventory" })], texts: [line] });
  const got = checkFrame(cut, cell).find((x) => x.rule === "text-cut");
  expect(got?.paths).toEqual(["Name"]);
  expect(got?.measured).toContain("15 of its 20 CSS px height");
  expect(got?.measured).not.toContain(" 0 CSS px");
  // A line cut across (a right-to-left line past its box's start) in a clipped element is still reported.
  const across = frame({ elements: [el("Blurb", "", "label", "text", [40, 219, 298, 18], { text: "Every widget", clipped: true })], texts: [{ ...line, text: "Every widget", bounds: [-83, 219, 421, 18] }] });
  expect(checkFrame(across, cell).find((x) => x.rule === "text-cut")?.paths).toEqual(["Blurb"]);
  // The same cut across inside a sideways scroller (a row of tabs) scrolls into view: no finding.
  const tabs = frame({ sideways: ["Tabs"], elements: [el("Tabs", "", "scroll", "", [18, 121, 354, 52]), el("Trade", "Tabs", "label", "text", [358, 131, 14, 32], { text: "Trade", clipped: true })], texts: [{ ...line, text: "Trade", bounds: [358, 131, 60, 32] }] });
  expect(rules(tabs)).not.toContain("text-cut");
  expect(rules({ ...tabs, sideways: [] })).toContain("text-cut");
});

test("text drawn wider than its own box (sized to English) is a finding", () => {
  const f = frame({ elements: [el("Go", "", "label", "text", [10, 100, 40, 20], { text: "[Gó fórwárd ñów]" })], texts: [{ text: "[Gó fórwárd ñów]", size: 14, bounds: [10, 100, 110, 20], clip: [0, 0, 390, 844], color: [1, 1, 1, 1] }] });
  expect(rules(f, { ...cell, locale: "pseudo" })).toContain("text-cut");
});

test("a line is its own box's though it matches no text as written (uppercased, or shaped as glyph ids) and touches a smaller one", () => {
  const ink = { size: 14, clip: [0, 0, 390, 844] as Rect4, color: [1, 1, 1, 1] as [number, number, number, number] };
  // Arabic drawn as the face's glyph ids, its descenders 4 px into the turn under it.
  const title = el("Title", "", "label", "heading", [100, 100, 66, 34], { text: "القتال" });
  const turn = el("Turn", "", "label", "text", [134, 134, 32, 22], { text: "دورك" });
  const shaped = "\u{F0004}\u{F0065}\u{F000C}ا";
  expect(rules(frame({ elements: [title, turn], texts: [{ ...ink, text: shaped, bounds: [100, 100, 66, 38] }] }))).not.toContain("text-cut");
  // A step's uppercase draws "EASY" for the label's "Easy", over the value beside it.
  const level = el("Level", "", "label", "text", [10, 200, 40, 20], { text: "Easy" });
  const value = el("Value", "", "label", "text", [40, 200, 17, 20], { text: "3" });
  expect(rules(frame({ elements: [level, value], texts: [{ ...ink, text: "EASY", bounds: [10, 200, 34, 20] }] }))).not.toContain("text-cut");
  // Wider than its own box, it is still cut.
  expect(rules(frame({ elements: [title, turn], texts: [{ ...ink, text: shaped, bounds: [100, 100, 120, 38] }] }))).toContain("text-cut");
});

test("text truncated with an ellipsis asks for a tooltip", () => {
  const f = frame({ elements: [el("Name", "", "label", "text", [10, 100, 60, 20], { text: "Excalibur of the North" })], texts: [{ text: "Excalib…", size: 14, bounds: [10, 100, 60, 20], clip: [0, 0, 390, 844], color: [1, 1, 1, 1] }] });
  const got = checkFrame(f, cell).find((x) => x.rule === "text-truncated");
  expect(got?.paths).toEqual(["Name"]);
  expect(got?.prop).toContain("tooltip");
});

test("a control laid out past the screen (og-ui's warning) is a finding with its path", () => {
  const f = frame({ warnings: ["'Settings/Panel/Footer/Done' is laid out 22 px past the 844x390 screen and nothing scrolls to it: fit the screen"] });
  const got = checkFrame(f, cell).find((x) => x.rule === "offscreen");
  expect(got?.paths).toEqual(["Settings/Panel/Footer/Done"]);
  expect(got?.measured).toContain("22");
});

test("an element under the notch is outside the safe area; one inside is not", () => {
  expect(rules(frame({ elements: [el("Score", "", "label", "text", [10, 20, 60, 20], { text: "Score" })] }))).toContain("safe-area");
  expect(rules(frame({ elements: [el("Score", "", "label", "text", [10, 60, 60, 20], { text: "Score" })] }))).not.toContain("safe-area");
});

test("an element under the shell's covered region is a finding", () => {
  const f = (x: number) => frame({ elements: [el("Lives", "", "label", "text", [x, 60, 60, 20], { text: "Lives" })] }, { covered: [12, 52, 40, 40] });
  expect(rules(f(20))).toContain("covered");
  expect(rules(f(80))).not.toContain("covered");
});

test("touch targets: each control's shorter side on a coarse pointer, whatever its size; none on a mouse", () => {
  const els = [el("Close", "", "button", "button", [300, 100, 30, 44]), el("Play", "", "button", "button", [20, 100, 200, 48])];
  // The shorter side (30, not 44), and the 48 px button too: no bar decides which are listed.
  expect(measured("touch-target", frame({ elements: els }))).toEqual([["Close", 30], ["Play", 48]]);
  expect(measured("touch-target", frame({ elements: els }, { coarsePointer: false, width: 1280, height: 720, safeArea: [0, 0, 0, 0] }))).toEqual([]);
  // No finding names a target's size.
  expect(rules(frame({ elements: els }))).not.toContain("touch-target");
});

test("a link inside a sentence is measured and marked inline; a standalone link is not marked", () => {
  const para = el("P", "", "richtext", "text", [20, 100, 300, 20], { text: "Read the pact first." });
  const detail = (f: UiFrameSnapshot) => measureFrame(f, cell).filter((m) => m.measure === "touch-target").map((m) => [m.paths[0], m.detail.includes("inline")]);
  expect(detail(frame({ elements: [para, el("pact", "P", "link", "link", [90, 100, 30, 20], { text: "pact" })] }))).toEqual([["P/pact", true]]);
  expect(detail(frame({ elements: [el("More", "", "link", "link", [90, 300, 30, 20], { text: "More" })] }))).toEqual([["More", false]]);
});

/** A 120×40 image: a ground of `bg` with a block of "glyph" strokes of `ink`. */
function textImage(bg: number, ink: number) {
  const width = 120, height = 40, rgba = new Uint8Array(width * height * 4);
  for (let y = 0; y < height; y++) for (let x = 0; x < width; x++) {
    const glyph = y >= 12 && y < 28 && x >= 10 && x < 110 && (x % 6 < 2 || y % 8 < 2);
    const v = glyph ? ink : bg;
    rgba.set([v, v, v, 255], (y * width + x) * 4);
  }
  return { width, height, rgba };
}

test("contrast is measured on pixels for live text, as a number with no bar; dimmed and over-the-scene text is not measured", () => {
  const f = frame({ elements: [el("Hint", "", "label", "text", [0, 0, 120, 40], { text: "Hint text" })] }, { width: 120, height: 40, safeArea: [0, 0, 0, 0] });
  const ratio = (fr: typeof f, img: ReturnType<typeof textImage>, c = cell) => measured("contrast", fr, c, img).find(([p]) => p === "Hint")?.[1];
  // Ground 90 under ink 150 is about 2.3:1; black under white about 21:1; 20 under 108 about 3.5:1. Each is reported,
  // high or low: the ordering of the three is the measure's, never a pass or a fail.
  const low = ratio(f, textImage(90, 150))!, high = ratio(f, textImage(0, 255))!, mid = ratio(f, textImage(20, 108))!;
  expect(low).toBeGreaterThan(1.5);
  expect(low).toBeLessThan(3);
  expect(high).toBeGreaterThan(15);
  expect(mid).toBeGreaterThan(low);
  expect(mid).toBeLessThan(4.5);
  // High contrast changes nothing in the number, only says it in the detail.
  const hc = measureFrame(f, { ...cell, highContrast: true }, { image: textImage(20, 150) }).find((m) => m.measure === "contrast")!;
  expect(hc.value).toBe(ratio(f, textImage(20, 150))!);
  expect(hc.detail).toContain("high contrast");
  // No finding names contrast.
  expect(checkFrame(f, cell).map((x) => x.rule)).not.toContain("contrast");
  // Drawn in the style's dimmed ink, or over no UI ground (HUD over the scene): not measured.
  const drawn = (color: number[]) => [{ text: "Hint text", size: 16, bounds: [10, 12, 100, 16] as [number, number, number, number], clip: [0, 0, 120, 40] as [number, number, number, number], color }];
  const muted = [0.58, 0.58, 0.61, 1];
  const vp = { width: 120, height: 40, safeArea: [0, 0, 0, 0] as [number, number, number, number] };
  const dim = frame({ elements: f.elements, texts: drawn(muted), dimInks: [muted] }, vp);
  expect(ratio(dim, textImage(90, 150))).not.toBeDefined();
  const live = frame({ elements: f.elements, texts: drawn([1, 1, 1, 1]), dimInks: [muted], grounds: [[0, 0, 120, 40]] }, vp);
  expect(ratio(live, textImage(90, 150))).toBe(low);
  const overScene = frame({ elements: f.elements, texts: drawn([1, 1, 1, 1]), dimInks: [muted], grounds: [[200, 0, 50, 50]] }, vp);
  expect(ratio(overScene, textImage(90, 150))).not.toBeDefined();
  // Without an image there is nothing to measure contrast on.
  expect(measured("contrast", f)).toEqual([]);
});

test("text the layout draws at full ink that shows none reads 1:1; text faded out, or a sliver a scroll cuts, is left out", () => {
  const vp = { width: 120, height: 40, safeArea: [0, 0, 0, 0] as [number, number, number, number] };
  type R4 = [number, number, number, number];
  const run = (alpha: number, clip: R4 = [0, 0, 120, 40]) => [{ text: "Inventory", size: 16, bounds: [10, 12, 100, 16] as R4, clip, color: [0.1, 0.1, 0.2, alpha] }];
  const at = (r: R4, clipped = false) => [el("Title", "", "label", "text", r, { text: "Inventory", clipped })];
  const ratio = (fr: ReturnType<typeof frame>, img: ReturnType<typeof textImage>) => measured("contrast", fr, cell, img)[0]?.[1];
  // Navy on indigo, as the pixels show it: the same value as the ground, no ink at all.
  const blank = textImage(60, 60);
  expect(ratio(frame({ elements: at([0, 0, 120, 40]), texts: run(1), grounds: [[0, 0, 120, 40]] }, vp), blank)).toBe(1);
  // Faded out (a transition, opacity 0; a world prompt past its maxDistance): not drawn, not measured, even over
  // pixels that read as ink (the scene's art behind it).
  expect(ratio(frame({ elements: at([0, 0, 120, 40]), texts: run(0), grounds: [[0, 0, 120, 40]] }, vp), blank)).not.toBeDefined();
  expect(ratio(frame({ elements: at([0, 0, 120, 40]), texts: run(0), grounds: [[0, 0, 120, 40]] }, vp), textImage(90, 150))).not.toBeDefined();
  // A scroll cuts it: most of its line in view is measured (its element is the part in view), a sliver is not.
  expect(ratio(frame({ elements: at([0, 0, 120, 40], true), texts: run(1, [0, 0, 60, 40]), grounds: [[0, 0, 120, 40]] }, vp), textImage(90, 150))).toBeLessThan(3);
  expect(ratio(frame({ elements: at([0, 0, 120, 14], true), texts: run(1, [0, 0, 120, 14]), grounds: [[0, 0, 120, 40]] }, vp), blank)).not.toBeDefined();
});

test("a measure names the element measured, though ids repeat; known ink sets the glyphs' side on a speckled ground", () => {
  type R4 = [number, number, number, number];
  const width = 240, height = 40, rgba = new Uint8Array(width * height * 4);
  // Left: dark strokes on a mid ground speckled with more light pixels than the strokes have (art behind a title).
  // Right: the same strokes on an even ground at 1:1 (no ink).
  for (let y = 0; y < height; y++) for (let x = 0; x < width; x++) {
    const glyph = x < 120 && y >= 12 && y < 28 && x >= 10 && x < 110 && x % 6 < 2;
    const speck = x < 120 && !glyph && (x * 7 + y * 13) % 5 === 0;
    const v = glyph ? 0 : speck ? 255 : 128;
    rgba.set([v, v, v, 255], (y * width + x) * 4);
  }
  const img = { width, height, rgba };
  const vp = { width, height, safeArea: [0, 0, 0, 0] as R4 };
  const run = (x: number) => ({ text: "12", size: 16, bounds: [x + 10, 12, 100, 16] as R4, clip: [0, 0, width, height] as R4, color: [0, 0, 0, 1] });
  const f = frame({
    elements: [el("A", "", "row", "", [0, 0, 120, 40]), el("Text", "A", "label", "text", [0, 0, 120, 40], { text: "12" }), el("B", "", "row", "", [120, 0, 120, 40]), el("Text", "B", "label", "text", [120, 0, 120, 40], { text: "12" })],
    texts: [run(0), run(120)],
    grounds: [[0, 0, width, height]],
  }, vp);
  const got = new Map(measured("contrast", f, cell, img));
  // Each by its own path: the left reads its dark strokes (about 5:1), not the speckles; the right 1:1.
  expect(got.get("A/Text")!).toBeGreaterThan(4);
  expect(got.get("B/Text")).toBe(1);
});

test("text size: each text's size by its element, marked dense under a dense ancestor only", () => {
  const t = (size: number) => [{ text: "Small", size, bounds: [10, 100, 40, 14] as [number, number, number, number], clip: [0, 0, 1920, 1080] as [number, number, number, number], color: [1, 1, 1, 1] }];
  const els = [el("Small", "", "label", "text", [10, 100, 40, 14], { text: "Small" })];
  expect(measured("text-size", frame({ elements: els, texts: t(10) }))).toEqual([["Small", 10]]);
  expect(measured("text-size", frame({ elements: els, texts: t(14.04) }))).toEqual([["Small", 14]]);
  expect(rules(frame({ elements: els, texts: t(10) }))).not.toContain("text-size");
  // ElementProps.dense on an ancestor marks its readouts; a prefix of a path ("Hu") or no dense group does not.
  const desk = { coarsePointer: false, width: 1920, height: 1080, safeArea: [0, 0, 0, 0] as [number, number, number, number] };
  const hud = [el("Hud", "", "panel", "", [0, 0, 400, 200]), el("Small", "Hud", "label", "text", [10, 100, 40, 14], { text: "Small" })];
  const dense = (d: string[]) => measureFrame(frame({ elements: hud, texts: t(12), dense: d }, desk), cell).filter((m) => m.measure === "text-size").map((m) => [m.paths[0], m.detail.includes("(dense)")]);
  expect(dense(["Hud"])).toEqual([["Hud/Small", true]]);
  expect(dense(["Hu"])).toEqual([["Hud/Small", false]]);
  expect(dense([])).toEqual([["Hud/Small", false]]);
});

test("pad reach: a control stacked on another is unreachable from the default focus; a non-focusable button too", () => {
  const node = (path: string, r: [number, number, number, number]) => ({ path, parent: "", role: "button", name: path, states: ["focusable"], rect: r });
  const stacked = frame({ a11y: { focused: "A", focusOrder: ["A", "B", "C"], nodes: [node("A", [40, 60, 40, 40]), node("B", [90, 20, 40, 40]), node("C", [40, 60, 40, 40])] } });
  expect(checkFrame(stacked, cell).filter((x) => x.rule === "focus-reach").map((x) => x.paths[0])).toEqual(["C"]);
  const lined = frame({ a11y: { focused: "A", focusOrder: ["A", "B", "C"], nodes: [node("A", [40, 100, 80, 44]), node("B", [40, 160, 80, 44]), node("C", [40, 220, 80, 44])] } });
  expect(rules(lined)).not.toContain("focus-reach");
  const unfocusable = frame({ a11y: { focused: "", focusOrder: [], nodes: [{ path: "Card", parent: "", role: "button", name: "Card", states: [], rect: [40, 100, 80, 44] }] } });
  expect(rules(unfocusable)).toContain("focus-reach");
  // A screen that took the pad (og.ui.setNavigation(false)) is not walked: neither reach rule; the same frame walked still is.
  const off = (navigation: boolean) => frame({ a11y: { navigation, focused: "A", focusOrder: ["A", "C"], nodes: [node("A", [40, 60, 40, 40]), node("C", [40, 60, 40, 40]), { path: "Card", parent: "", role: "button", name: "Card", states: [], rect: [40, 100, 80, 44] }] } });
  expect(rules(off(false))).not.toContain("focus-reach");
  expect(checkFrame(off(true), cell).filter((x) => x.rule === "focus-reach").map((x) => x.paths[0]).sort()).toEqual(["C", "Card"]);
});

test("pad reach: a control beside the focus in its row beats one diagonally below (edge to edge, beam first)", () => {
  // The kit's deck header on a phone: Gallery and Next in a row, the deck tab under both. Scored centre to centre,
  // right from Gallery took the tab and up from the tab tied and took Gallery, so nothing led to Next.
  const gallery: Rect4 = [186, 63, 80, 44], next: Rect4 = [274, 63, 100, 44], tab: Rect4 = [194, 111, 188, 44];
  const graph = padGraph([gallery, next, tab]);
  expect(neighbour(gallery, [[1, next], [2, tab]], "right")).toBe(1);
  expect(neighbour(tab, [[0, gallery], [1, next]], "up")).toBe(1);
  expect(padFindings(graph, 0)).toEqual([]);
  // A link beside another inside the focusable row around both (a chat line): right goes to the link, not the row.
  const row: Rect4 = [38, 224, 1204, 38], who: Rect4 = [240, 227, 69, 32], ring: Rect4 = [415, 227, 146, 32];
  expect(neighbour(who, [[0, row], [2, ring]], "right")).toBe(2);
  // A shop tile whose whole box runs past its scroll's view (ending at y 576) over Gallery and Next at the screen's
  // foot: scored by the whole box, right from Gallery took the tile and nothing led to Next; by its part in view, the
  // tile stands above them.
  const tile: Rect4 = [24, 356, 482, 627], gal: Rect4 = [16, 608, 216, 96], nxt: Rect4 = [240, 608, 228, 96];
  const shop = { shown: [24, 356, 482, 220] as Rect4, scroller: "Shop/Body/View" };
  const chrome = { shown: null, scroller: "" };
  expect(neighbour(gal, [[1, nxt], [2, tile]], "right")).toBe(2);
  expect(neighbour(gal, [[1, nxt], [2, shop.shown]], "right")).toBe(1);
  expect(padGraph([gal, nxt, tile], [chrome, chrome, shop])[0]).toContain(1);
  expect(padFindings(padGraph([gal, nxt, tile], [chrome, chrome, shop]), 0)).toEqual([]);
});

test("pad reach: options inside a focusable listbox are reached through it; outside one they are not", () => {
  // Wrong: a card hand's options (walked by the listbox's own cursor) reported unreachable, or any option let off.
  const opt = (path: string, parent: string) => ({ path, parent, role: "option", name: path, states: [], rect: [40, 100, 80, 44] as [number, number, number, number] });
  const inside = frame({ a11y: { focused: "Hand", focusOrder: ["Hand"], nodes: [{ path: "Hand", parent: "", role: "listbox", name: "Hand", states: ["focusable"], rect: [0, 80, 400, 120] }, opt("Hand/Strike", "Hand")] } });
  expect(rules(inside)).not.toContain("focus-reach");
  const loose = frame({ a11y: { focused: "Hand", focusOrder: ["Hand"], nodes: [{ path: "Hand", parent: "", role: "group", name: "Hand", states: ["focusable"], rect: [0, 80, 400, 120] }, opt("Hand/Strike", "Hand")] } });
  expect(rules(loose)).toContain("focus-reach");
  // A stepper's − and + step the spinbutton the pad reaches as one stop (Left and Right): reached through it.
  const btn = (path: string) => ({ path, parent: "Amount", role: "button", name: path, states: [], rect: [40, 100, 40, 40] as [number, number, number, number] });
  const spin = frame({ a11y: { focused: "Amount", focusOrder: ["Amount"], nodes: [{ path: "Amount", parent: "", role: "spinbutton", name: "Amount", states: ["focusable"], rect: [0, 100, 200, 40] }, btn("Amount/Dec"), btn("Amount/Inc")] } });
  expect(rules(spin)).not.toContain("focus-reach");
});

test("a pad trap: a control the graph reaches but cannot leave back to the default focus", () => {
  // 0 → 1 → 2, and 2 leads only to itself's neighbour 1, never back to 0.
  expect(padFindings([[1], [2], [1]], 0)).toEqual([{ rule: "focus-trap", index: 1 }, { rule: "focus-trap", index: 2 }]);
  expect(padFindings([[1], [0, 2], [1]], 0)).toEqual([]);
  expect(padFindings([[1], [0], []], 0)).toEqual([{ rule: "focus-reach", index: 2 }]);
});

test("a control with no accessible name is a finding", () => {
  const f = frame({ a11y: { focused: "", focusOrder: [], nodes: [{ path: "Hud/Pause", parent: "", role: "button", name: "", states: [], rect: [0, 0, 44, 44] }] } });
  expect(checkFrame(f, cell).find((x) => x.rule === "a11y-name")?.paths).toEqual(["Hud/Pause"]);
});

test("a name left in the game's language in another locale is a finding", () => {
  const n = (name: string) => ({ path: "Menu/Play", parent: "", role: "button", name, states: [], rect: [0, 0, 44, 44] as [number, number, number, number] });
  const en = frame({ a11y: { focused: "", focusOrder: [], nodes: [n("Play")] } });
  expect(checkLocales(en, frame({ a11y: { focused: "", focusOrder: [], nodes: [n("Play")] } }), { ...cell, locale: "ja" }).map((x) => x.rule)).toEqual(["a11y-locale"]);
  expect(checkLocales(en, frame({ a11y: { focused: "", focusOrder: [], nodes: [n("プレイ")] } }), { ...cell, locale: "ja" })).toEqual([]);
});

test("at 200 % text, a cut is a text-cut finding of that cell, and each text's growth is measured as a percentage", () => {
  const big: UiCell = { ...cell, scale: 200 };
  const els = [el("Panel", "", "panel", "", [0, 100, 100, 40]), el("Name", "Panel", "label", "text", [10, 110, 80, 20], { text: "Strength" }), el("Gold", "Panel", "label", "text", [10, 130, 80, 20], { text: "120" })];
  const line = (text: string, size: number, y: number, w: number) => ({ text, size, bounds: [10, y, w, size * 1.4] as [number, number, number, number], clip: [0, 100, 200, 400] as [number, number, number, number], color: [1, 1, 1, 1] });
  const cut = frame({ elements: els, texts: [{ ...line("Strength", 28, 110, 130), clip: [0, 100, 100, 40] }] });
  expect(rules(cut)).toContain("text-cut");
  expect(checkFrame(cut, big).filter((f) => f.rule === "text-cut").map((f) => f.cells)).toEqual([["s phonePortrait game locale 200 %"]]);
  // "120" stays 14 px where "Strength" doubled: 100 % and 200 % of their sizes.
  const at100 = frame({ elements: els, texts: [line("Strength", 14, 110, 60), line("120", 14, 130, 24)] });
  const at200 = frame({ elements: els, texts: [line("Strength", 28, 110, 120), line("120", 14, 140, 24)] });
  const grow = (a: UiFrameSnapshot, b: UiFrameSnapshot, c = big) => measureScale(a, b, c).map((m) => [m.paths[0], m.value]);
  expect(grow(at100, at200)).toEqual([["Panel/Name", 200], ["Panel/Gold", 100]]);
  expect(measureScale(at100, at200, big)[1]!.detail).toContain('"120" 14 CSS px at 200 % text, 14 at 100 %');
  expect(measureScale(at100, at200, cell)).toEqual([]);
  // The same words on two elements are two texts: a badge's "2" the larger text scrolled out of view is not measured
  // against a row's "2" that doubled. (Paired by words alone in draw order, the row's 30 met the badge's 16.)
  const twos = [el("Badge", "", "label", "text", [10, 110, 20, 30], { text: "2" }), el("Copies", "", "label", "text", [10, 200, 40, 50], { text: "2" })];
  const both = frame({ elements: twos, texts: [line("2", 16, 110, 9), line("2", 15, 200, 9)] });
  expect(grow(both, frame({ elements: twos.slice(1), texts: [line("2", 30, 200, 17)] }))).toEqual([["Copies", 200]]);
  expect(grow(both, frame({ elements: twos.slice(1), texts: [line("2", 15, 200, 9)] }))).toEqual([["Copies", 100]]);
});

test("primary actions: the buttons filled with the primary, counted per layer; a translucent secondary is not one", () => {
  const els = [el("Play", "", "button", "button", [20, 100, 200, 48]), el("Quit", "", "button", "button", [20, 160, 200, 48])];
  const prim = (f: UiFrameSnapshot) => measureFrame(f, cell).filter((m) => m.measure === "primary-actions").map((m) => [m.paths, m.value]);
  expect(prim(frame({ elements: els, fills: [{ rect: [20, 100, 200, 48], color: [0.2, 0.4, 0.9, 1] }, { rect: [20, 160, 200, 48], color: [0.2, 0.4, 0.9, 1] }] }))).toEqual([[["Play", "Quit"], 2]]);
  expect(prim(frame({ elements: els, fills: [{ rect: [20, 100, 200, 48], color: [0.2, 0.4, 0.9, 1] }, { rect: [20, 160, 200, 48], color: [0.25, 0.45, 0.95, 0.06] }] }))).toEqual([[["Play"], 1]]);
  // A dialog is a layer of its own: one primary in it and one behind are two layers of one.
  const modal = [...els, el("Ask", "", "modal", "dialog", [0, 0, 390, 844]), el("Yes", "Ask", "button", "button", [20, 400, 200, 48])];
  const fills = [{ rect: [20, 100, 200, 48] as Rect4, color: [0.2, 0.4, 0.9, 1] }, { rect: [20, 400, 200, 48] as Rect4, color: [0.2, 0.4, 0.9, 1] }];
  expect(prim(frame({ elements: modal, fills }))).toEqual([[["Play"], 1], [["Ask/Yes"], 1]]);
  expect(rules(frame({ elements: els, fills }))).not.toContain("primary-actions");
});

test("feedback: a styled game without press and hover sounds, and a custom interactive container without states", () => {
  expect(rules(frame({ style: "pixel" }))).toContain("feedback");
  expect(rules(frame({ style: "pixel", sounds: { hover: true, press: true, open: true, close: true, success: true, error: true } }))).not.toContain("feedback");
  const card = [el("Card", "", "panel", "button", [20, 100, 200, 80])];
  expect(rules(frame({ elements: card }))).toContain("feedback");
  expect(rules(frame({ elements: card, motion: [{ path: "Card", states: [["press", "rest"]] }] }))).not.toContain("feedback");
});

test("the element count is measured on every frame, with its draws", () => {
  const count = (n: number) => measureFrame(frame({ stats: { elements: n, rebuilt: 0, laidOut: 0, keptCommands: 0 }, draws: 7 }), cell).filter((m) => m.measure === "elements");
  expect(count(301).map((m) => [m.value, m.detail])).toEqual([[301, "7 draw commands"]]);
  expect(count(12).map((m) => m.value)).toEqual([12]);
  expect(rules(frame({ stats: { elements: 301, rebuilt: 0, laidOut: 0, keptCommands: 0 } }))).not.toContain("elements");
});

test("paths come from the accessibility tree, else the parent chain; findings merge across cells", () => {
  const f = frame({
    elements: [el("Hud", "", "anchor", "", [0, 0, 100, 40]), el("Score", "Hud", "label", "text", [0, 0, 60, 20], { text: "1" })],
    a11y: { focused: "", focusOrder: [], nodes: [] },
  });
  expect(elementPaths(f)).toEqual(["Hud", "Hud/Score"]);
  const a = checkFrame(frame({ overlaps: [{ path: "A", other: "B", rect: [0, 0, 5, 5], inFlow: false }] }), cell);
  const b = checkFrame(frame({ overlaps: [{ path: "A", other: "B", rect: [0, 0, 5, 5], inFlow: false }] }), { ...cell, locale: "pseudo" });
  const merged = mergeFindings([...a, ...b]);
  expect(merged.length).toBe(1);
  expect(merged[0]!.cells).toEqual(["s phonePortrait game locale", "s phonePortrait pseudo"]);
  // Another pair is another finding.
  const c = checkFrame(frame({ overlaps: [{ path: "A", other: "C", rect: [0, 0, 5, 5], inFlow: false }] }), cell);
  expect(mergeFindings([...a, ...c]).map((f) => f.paths)).toEqual([["A", "B"], ["A", "C"]]);
});

test("the default screens: the values a show command's usage lists, else each playtest screenshot reached by the steps before it", () => {
  const help = "screenshot[:name.png]\n    Capture a frame.\nshow:<title|play|shop> (registered)\n    Open a screen.\nviewport:<desktop|phone> (registered)\n    Lay out.\n";
  expect(screensFromHelp(help, "show:{screen}")).toEqual([
    { name: "title", commands: "show:title" },
    { name: "play", commands: "show:play" },
    { name: "shop", commands: "show:shop" },
  ]);
  // A usage that names no list (`<screen>`), or no such action: none.
  expect(screensFromHelp("show:<screen> (registered)\n", "show:{screen}")).toEqual([]);
  expect(screensFromHelp(help, "open:{screen}")).toEqual([]);
  expect(screensFromPlaytest("menu", "0:key_down:Enter;0.1:dump_ui:a.json;0.5:screenshot:title.png;0.6:key_up:Enter;1.0:expect:ui:text:Play;1.5:screenshot;2:quit")).toEqual([
    { name: "menu-title", commands: "0:key_down:Enter", at: 0.5 },
    { name: "menu-shot-5", commands: "0:key_down:Enter;0.6:key_up:Enter", at: 1.5 },
  ]);
  // A screenshot while the inspector is open is the inspector's frame: none until inspect:off closes it,
  // and setting its key or gizmo mode neither opens nor closes it.
  expect(screensFromPlaytest("edit", "0:inspect:mode:rotate;0.2:screenshot:a.png;0.5:inspect:crate;1:screenshot:b.png;1.2:inspect:space:local;1.5:screenshot:c.png;2:inspect:off;2.5:screenshot:d.png;3:inspect;3.5:screenshot:e.png")).toEqual([
    { name: "edit-a", commands: "0:inspect:mode:rotate", at: 0.2 },
    { name: "edit-d", commands: "0:inspect:mode:rotate;0.5:inspect:crate;1.2:inspect:space:local;2:inspect:off", at: 2.5 },
  ]);
});

test("a label drawn in two inks with split clips (a meter's fill and track) is one line, neither cut nor mis-scaled", () => {
  // Wrong: each piece judged on its own clip (reported cut), and a frame that draws one piece more than
  // the other matched in drawing order against another control's text of the same string.
  const els = [el("Boost", "", "meter", "progressbar", [312, 1000, 196, 20], { text: "Boost" }), el("boost", "", "action", "button", [720, 1060, 84, 84], { text: "Boost" })];
  const color = [1, 1, 1, 1];
  const base = frame({ elements: els, texts: [{ text: "Boost", size: 13, bounds: [392, 1001, 35, 18], clip: [312, 1000, 196, 20], color }, { text: "Boost", size: 14, bounds: [743, 1092, 38, 20], clip: [720, 1060, 84, 84], color }] });
  const meter = { text: "Boost", size: 26, bounds: [375, 983, 70, 36] as [number, number, number, number], color };
  const scaled = frame({ elements: els, texts: [{ ...meter, clip: [312, 982, 121.5, 38] }, { ...meter, clip: [433.5, 982, 74.5, 38] }, { text: "Boost", size: 28, bounds: [724, 1082, 75, 40], clip: [720, 1060, 84, 84], color }] });
  const c200 = { ...cell, scale: 200 };
  expect(checkFrame(scaled, c200).filter((f) => f.rule === "text-cut")).toEqual([]);
  expect(measureScale(base, scaled, c200).map((m) => [m.paths[0], m.value])).toEqual([["Boost", 200], ["boost", 200]]);
});

test("a measure carries its number, unit, cell and box; the extremes are each path's lowest over the cells", () => {
  // The label sits at 0,0 120×40 in a 120×40 frame; a ground of 90 under ink 150 measures about 2.3:1.
  const vp = { width: 120, height: 40, safeArea: [0, 0, 0, 0] as Rect4 };
  const f = frame({ elements: [el("Hint", "", "label", "text", [0, 0, 120, 40], { text: "Hint text" })] }, vp);
  const c = measureFrame(f, cell, { image: textImage(90, 150) }).find((x) => x.measure === "contrast")!;
  expect([c.unit, c.cell, c.rect]).toEqual([":1", "s phonePortrait game locale", [0, 0, 120, 40]]);
  expect(c.detail).toContain('"Hint text"');
  // A text size's box is the line's, not the element's.
  const small = frame({ elements: [el("Small", "", "label", "text", [10, 100, 80, 20], { text: "Small" })], texts: [{ text: "Small", size: 10, bounds: [12, 103, 40, 14], clip: [0, 0, 390, 844], color: [1, 1, 1, 1] }] });
  const size = measureFrame(small, cell).find((x) => x.measure === "text-size")!;
  expect([size.value, size.unit, size.rect]).toEqual([10, "CSS px", [12, 103, 40, 14]]);
  // The same control in two cells is two measures, each with its own box; the extremes keep the smaller per path.
  const wide = { ...cell, breakpoint: "phoneLandscape" };
  const moved = frame({ elements: [el("Close", "", "button", "button", [700, 60, 30, 30]), el("Open", "", "button", "button", [700, 160, 60, 50])] }, { width: 844, height: 390, safeArea: [47, 0, 47, 21], breakpoint: "phoneLandscape" });
  const all: UiMeasure[] = [...measureFrame(frame({ elements: [el("Close", "", "button", "button", [300, 100, 36, 36]), el("Open", "", "button", "button", [300, 200, 44, 44])] }), cell), ...measureFrame(moved, wide)].filter((x) => x.measure === "touch-target");
  expect(all.map((m) => [m.paths[0], m.value, m.cell, m.rect])).toEqual([
    ["Close", 36, "s phonePortrait game locale", [300, 100, 36, 36]],
    ["Open", 44, "s phonePortrait game locale", [300, 200, 44, 44]],
    ["Close", 30, "s phoneLandscape game locale", [700, 60, 30, 30]],
    ["Open", 50, "s phoneLandscape game locale", [700, 160, 60, 50]],
  ]);
  expect(measureExtremes(all).get("touch-target")!.map((m) => [m.paths[0], m.value])).toEqual([["Close", 30], ["Open", 44]]);
  // The element count reads highest first.
  const counts = [12, 340, 80].map((n, i) => measureFrame(frame({ stats: { elements: n, rebuilt: 0, laidOut: 0, keptCommands: 0 } }), { ...cell, screen: `s${i}` })).flat();
  expect(measureExtremes(counts).get("elements")!.map((m) => m.value)).toEqual([340]);
  // The text report lists them with no bar.
  const text = formatText([], "", all);
  expect(text).toContain("touch-target (lowest first):");
  expect(text).toContain("30 CSS px  Close");
});

test("og ui review's layout measures read overflow, clip, grid offsets and gaps as they are, with no threshold", () => {
  const els = [
    el("Row", "", "row", "", [0, 100, 200, 40]),
    el("A", "Row", "panel", "", [0, 100, 40, 40]),
    el("B", "Row", "panel", "", [48, 100, 40, 40]),
    el("C", "Row", "panel", "", [94, 100, 40, 40]),
    el("Name", "", "label", "", [148, 160, 40, 20], { text: "Name" }),
  ];
  // "Name" draws 50 px wide in its 40 px box (10 past it), and a clip 30 px wide shows 30 of its 50 px.
  const f = frame({ elements: els, texts: [{ text: "Name", size: 14, bounds: [148, 160, 50, 20], clip: [0, 150, 178, 40], color: [1, 1, 1, 1] }] });
  const by = (id: string) => measureLayout(f, cell).filter((m) => m.measure === id).map((m) => [m.paths.join(" × "), m.value, m.detail] as const);
  expect(by("overflow").map(([p, v]) => [p, v])).toEqual([["Name", 10]]);
  expect(by("clip").map(([p, v]) => [p, v])).toEqual([["Name", 400]]);
  // C starts 2 px off the 4 px grid (94), and so does its right edge (134); the rest sit on it.
  expect(by("grid").map(([p, v]) => [p, v])).toEqual([["Row/C", 2]]);
  expect(by("gap")).toEqual([["Row", 2, "row of 3: gaps 8, 6 CSS px"]]);
  // On a 2 px grid nothing is off it.
  expect(measureLayout(f, cell, 2).filter((m) => m.measure === "grid")).toEqual([]);
});
