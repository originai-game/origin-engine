// `og ui check`'s rules (src/node/ui-check.ts, docs/ui.md §16.5), each on a
// frame that violates it and on the same frame put right: every finding
// names its rule, paths, measured value and the prop to change. The rules
// that wait for later phases are listed as such, never dropped.
import { expect, test } from "bun:test";
import { checkFrame, checkLocales, checkScale, elementPaths, formatText, mergeFindings, padFindings, pseudo, RULES, screensFromHelp, screensFromPlaytest, type UiCell, type UiElementDump, type UiFrameSnapshot } from "../src/node/ui-check.ts";

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

test("every §16.5 rule is listed, and those waiting for a later phase say so with the reason", () => {
  const ids = RULES.map((r) => r.id);
  for (const id of ["overlap", "text-cut", "text-truncated", "pseudo-overflow", "text-scale", "safe-area", "covered", "touch-target", "contrast", "text-size", "focus-reach", "focus-trap", "layer-back", "a11y-name", "a11y-locale", "primary-actions", "feedback", "motion", "art-slot", "budget"]) expect(ids).toContain(id);
  for (const r of RULES.filter((r) => r.status !== "checked")) expect(r.note ?? "").not.toBe("");
  const text = formatText([]);
  for (const id of ["layer-back", "motion", "art-slot"]) expect(text).toContain(`${id} (not checkable yet)`);
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
  expect(checkFrame(across, cell).find((x) => x.rule === "text-cut")?.severity).toBe("warning");
});

test("text drawn wider than its own box (sized to English) is a finding", () => {
  const f = frame({ elements: [el("Go", "", "label", "text", [10, 100, 40, 20], { text: "[Gó fórwárd ñów]" })], texts: [{ text: "[Gó fórwárd ñów]", size: 14, bounds: [10, 100, 110, 20], clip: [0, 0, 390, 844], color: [1, 1, 1, 1] }] });
  expect(rules(f, { ...cell, locale: "pseudo" })).toContain("text-cut");
});

test("text truncated with an ellipsis asks for a tooltip", () => {
  const f = frame({ elements: [el("Name", "", "label", "text", [10, 100, 60, 20], { text: "Excalibur of the North" })], texts: [{ text: "Excalib…", size: 14, bounds: [10, 100, 60, 20], clip: [0, 0, 390, 844], color: [1, 1, 1, 1] }] });
  const got = checkFrame(f, cell).find((x) => x.rule === "text-truncated");
  expect(got?.severity).toBe("warning");
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

test("a 30 px button on a coarse pointer is under 44 CSS px; on a mouse it is not checked", () => {
  const small = [el("Close", "", "button", "button", [300, 100, 30, 30])];
  expect(rules(frame({ elements: small }))).toContain("touch-target");
  expect(rules(frame({ elements: small }, { coarsePointer: false, width: 1280, height: 720, safeArea: [0, 0, 0, 0] }))).not.toContain("touch-target");
  expect(rules(frame({ elements: [el("Close", "", "button", "button", [300, 100, 44, 44])] }))).not.toContain("touch-target");
});

test("a link inside a sentence is exempt from the target size (WCAG 2.5.8); a standalone small link is not", () => {
  const para = el("P", "", "richtext", "text", [20, 100, 300, 20], { text: "Read the pact first." });
  expect(rules(frame({ elements: [para, el("pact", "P", "link", "link", [90, 100, 30, 20], { text: "pact" })] }))).not.toContain("touch-target");
  expect(rules(frame({ elements: [el("More", "", "link", "link", [90, 300, 30, 20], { text: "More" })] }))).toContain("touch-target");
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

test("contrast is measured on pixels: grey on grey fails 4.5:1, white on black passes; high contrast holds 7:1", () => {
  const f = frame({ elements: [el("Hint", "", "label", "text", [0, 0, 120, 40], { text: "Hint text" })] }, { width: 120, height: 40, safeArea: [0, 0, 0, 0] });
  expect(checkFrame(f, cell, { image: textImage(90, 150) }).map((x) => x.rule)).toContain("contrast");
  expect(checkFrame(f, cell, { image: textImage(0, 255) }).map((x) => x.rule)).not.toContain("contrast");
  // About 5.7:1: enough for 4.5, not for the high-contrast bar of 7.
  const mid = textImage(20, 150);
  expect(checkFrame(f, cell, { image: mid }).map((x) => x.rule)).not.toContain("contrast");
  expect(checkFrame(f, { ...cell, highContrast: true }, { image: mid }).find((x) => x.rule === "contrast")?.limit).toContain("7:1");
});

test("text under the minimum size: 10 px on a phone, 14 px at 1080p desktop", () => {
  const t = (size: number) => [{ text: "Small", size, bounds: [10, 100, 40, 14] as [number, number, number, number], clip: [0, 0, 1920, 1080] as [number, number, number, number], color: [1, 1, 1, 1] }];
  const els = [el("Small", "", "label", "text", [10, 100, 40, 14], { text: "Small" })];
  expect(rules(frame({ elements: els, texts: t(10) }))).toContain("text-size");
  expect(rules(frame({ elements: els, texts: t(14) }))).not.toContain("text-size");
  const desk = { coarsePointer: false, width: 1920, height: 1080, safeArea: [0, 0, 0, 0] as [number, number, number, number] };
  expect(rules(frame({ elements: els, texts: t(14) }, desk))).toContain("text-size");
  expect(rules(frame({ elements: els, texts: t(18) }, desk))).not.toContain("text-size");
  // A dense HUD's readouts (ElementProps.dense, on an ancestor) go to 12 px
  // at 1080p, never below, and the floor grows with the text scale; a
  // sibling outside the dense group keeps 18.
  const hud = [el("Hud", "", "panel", "", [0, 0, 400, 200]), el("Small", "Hud", "label", "text", [10, 100, 40, 14], { text: "Small" })];
  const dense = (size: number, d = ["Hud"]) => frame({ elements: hud, texts: t(size), dense: d }, desk);
  expect(rules(dense(12))).not.toContain("text-size");
  expect(rules(dense(11))).toContain("text-size");
  expect(rules(dense(12, ["Hu"]))).toContain("text-size");
  expect(rules(dense(12, []))).toContain("text-size");
  expect(rules(dense(18), { ...cell, scale: 200 })).toContain("text-size");
  expect(rules(dense(24), { ...cell, scale: 200 })).not.toContain("text-size");
});

test("pad reach: a control stacked on another is unreachable from the default focus; a non-focusable button too", () => {
  const node = (path: string, r: [number, number, number, number]) => ({ path, parent: "", role: "button", name: path, states: ["focusable"], rect: r });
  const stacked = frame({ a11y: { focused: "A", focusOrder: ["A", "B", "C"], nodes: [node("A", [40, 60, 40, 40]), node("B", [90, 20, 40, 40]), node("C", [40, 60, 40, 40])] } });
  expect(checkFrame(stacked, cell).filter((x) => x.rule === "focus-reach").map((x) => x.paths[0])).toEqual(["C"]);
  const lined = frame({ a11y: { focused: "A", focusOrder: ["A", "B", "C"], nodes: [node("A", [40, 100, 80, 44]), node("B", [40, 160, 80, 44]), node("C", [40, 220, 80, 44])] } });
  expect(rules(lined)).not.toContain("focus-reach");
  const unfocusable = frame({ a11y: { focused: "", focusOrder: [], nodes: [{ path: "Card", parent: "", role: "button", name: "Card", states: [], rect: [40, 100, 80, 44] }] } });
  expect(rules(unfocusable)).toContain("focus-reach");
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

test("at 200 % text, a cut is the text-scale rule's and a text that did not grow is a finding", () => {
  const big: UiCell = { ...cell, scale: 200 };
  const els = [el("Panel", "", "panel", "", [0, 100, 100, 40]), el("Name", "Panel", "label", "text", [10, 110, 80, 20], { text: "Strength" }), el("Gold", "Panel", "label", "text", [10, 130, 80, 20], { text: "120" })];
  const line = (text: string, size: number, y: number, w: number) => ({ text, size, bounds: [10, y, w, size * 1.4] as [number, number, number, number], clip: [0, 100, 200, 400] as [number, number, number, number], color: [1, 1, 1, 1] });
  // The same line cut in a 100 % cell is text-cut, in a 200 % cell text-scale.
  const cut = frame({ elements: els, texts: [{ ...line("Strength", 28, 110, 130), clip: [0, 100, 100, 40] }] });
  expect(rules(cut)).toContain("text-cut");
  expect(rules(cut, big)).toContain("text-scale");
  expect(rules(cut, big)).not.toContain("text-cut");
  // "120" stays 14 px where "Strength" doubled: a size the scale does not reach.
  const at100 = frame({ elements: els, texts: [line("Strength", 14, 110, 60), line("120", 14, 130, 24)] });
  const at200 = frame({ elements: els, texts: [line("Strength", 28, 110, 120), line("120", 14, 140, 24)] });
  const got = checkScale(at100, at200, big);
  expect(got.map((f) => f.rule)).toEqual(["text-scale"]);
  expect(got[0]!.measured).toContain('"120" 14 CSS px at 200 %');
  expect(checkScale(at100, at200, cell)).toEqual([]);
  expect(checkScale(at100, frame({ elements: els, texts: [line("Strength", 28, 110, 120), line("120", 28, 140, 48)] }), big)).toEqual([]);
});

test("two primary buttons in one layer are a finding; a primary and a translucent secondary are not", () => {
  const els = [el("Play", "", "button", "button", [20, 100, 200, 48]), el("Quit", "", "button", "button", [20, 160, 200, 48])];
  const two = frame({ elements: els, fills: [{ rect: [20, 100, 200, 48], color: [0.2, 0.4, 0.9, 1] }, { rect: [20, 160, 200, 48], color: [0.2, 0.4, 0.9, 1] }] });
  expect(checkFrame(two, cell).find((x) => x.rule === "primary-actions")?.paths).toEqual(["Play", "Quit"]);
  const one = frame({ elements: els, fills: [{ rect: [20, 100, 200, 48], color: [0.2, 0.4, 0.9, 1] }, { rect: [20, 160, 200, 48], color: [0.25, 0.45, 0.95, 0.06] }] });
  expect(rules(one)).not.toContain("primary-actions");
});

test("feedback: a styled game without press and hover sounds, and a custom interactive container without states", () => {
  expect(rules(frame({ style: "pixel" }))).toContain("feedback");
  expect(rules(frame({ style: "pixel", sounds: { hover: true, press: true, open: true, close: true, success: true, error: true } }))).not.toContain("feedback");
  const card = [el("Card", "", "panel", "button", [20, 100, 200, 80])];
  expect(rules(frame({ elements: card }))).toContain("feedback");
  expect(rules(frame({ elements: card, motion: [{ path: "Card", states: [["press", "rest"]] }] }))).not.toContain("feedback");
});

test("over 300 elements is over the dense-menu budget", () => {
  expect(rules(frame({ stats: { elements: 301, rebuilt: 0, laidOut: 0, keptCommands: 0 } }))).toContain("budget");
  expect(rules(frame({ stats: { elements: 300, rebuilt: 0, laidOut: 0, keptCommands: 0 } }))).not.toContain("budget");
});

test("the pseudo-locale is 40 % longer, accented and bracketed; numbers stay as they are", () => {
  const p = pseudo("Settings");
  expect(p.startsWith("[") && p.endsWith("]")).toBeTrue();
  expect(p).toContain("Séttíñgs");
  // Eight letters, three more (40 %), a space and the brackets.
  expect([...p].length).toBe(8 + 3 + 1 + 2);
  expect(pseudo("4,250")).toBe("4,250");
  expect(pseudo(p)).toBe(p);
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
});
