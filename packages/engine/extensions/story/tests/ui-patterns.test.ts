import { expect, test } from "bun:test";

import { og } from "@origingame/engine";

import { story, type CgGalleryResult, type RouteChartResult } from "@origingame/engine/extensions/story/index.ts";

import { storyRun, elements, key, run, tapOn, byPath } from "../../../tests/support/patterns.ts";

import "../../../tests/support/patterns.ts";

test("backlog: each line under its speaker with the reply chosen on it; back closes it", () => {
  // Wrong: the reply lost (the player's choices missing from the log), or back ignored.
  const closed: boolean[] = [];
  const engine = storyRun((w) => {
    const r = story.backlog("Log", {
      lines: [
        { seq: 1, node: "greet", speaker: { id: "smith", name: "Brannoc", entity: 0 as never, portrait: "", expression: "", side: "right" }, key: "smith.hello", args: {}, text: "Steel's not cheap.", voice: "vo/a.wav", seen: false, chosen: { index: 1, text: "Haggle" } },
        { seq: 2, node: "angry", speaker: { id: "narrator", name: "", entity: 0 as never, portrait: "", expression: "", side: "center" }, key: "n", args: {}, text: "He slams the anvil.", voice: "", seen: false, chosen: null },
      ],
    });
    closed.push(r.closed);
    void w;
  });
  engine.advance(4);
  const items = elements(engine).filter((e) => e.role === "listitem");
  expect(items.map((e) => e.name)).toEqual(["Brannoc: Steel's not cheap. → Haggle", "He slams the anvil."]);
  expect(elements(engine).some((e) => e.role === "button" && e.name === "Play voice: Steel's not cheap.")).toBeTrue();
  key(engine, "Escape");
  expect(closed.some((c) => c)).toBeTrue();
});

// ── VN gallery and route chart ──────────────────────────────────────────

test("cgGallery: a locked picture is refused, an open one opens; the viewer's next skips the locked and wraps; Back closes the viewer", () => {
  // Wrong: a locked picture opened (a spoiler), next stopping on a locked one, or Back leaving the gallery instead of the viewer.
  const results: CgGalleryResult[] = [];
  const st = { viewing: "" };
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 1232 }, height: { kind: "fixed", px: 672 } }, () => {
      const r = story.cgGallery("Gallery", {
        entries: [
          { key: "a", title: "Riverboat" },
          { key: "b", title: "Storm", locked: true, hint: "Clear Mira's route" },
          { key: "c", title: "Dawn gate" },
        ],
        viewing: st.viewing,
      });
      results.push(r);
      if (r.open) st.viewing = r.open;
      if (r.step) st.viewing = r.step;
      if (r.close) st.viewing = "";
    });
  });
  engine.advance(10);
  tapOn(engine, byPath(engine, "Grid/Line#0/b")!);
  expect(st.viewing).toBe("");
  expect(results.filter((r) => r.refused).map((r) => r.refused)).toEqual(["b"]);
  tapOn(engine, byPath(engine, "Grid/Line#0/c")!);
  expect(st.viewing).toBe("c");
  tapOn(engine, byPath(engine, "Viewer/Bar/Next")!);
  expect(st.viewing).toBe("a");
  tapOn(engine, byPath(engine, "Viewer/Bar/Next")!);
  expect(st.viewing).toBe("c");
  key(engine, "Escape");
  expect(st.viewing).toBe("");
  expect(byPath(engine, "Gallery/Viewer") ?? null).toBeNull();
});

test("routeChart: a locked scene is refused; Jump stands only on a seen scene; a branch runs from one card's end edge to the next card's start edge", () => {
  // Wrong: a locked scene selected, Jump offered on a scene never played, or a branch drawn through the cards.
  const results: RouteChartResult[] = [];
  const st = { selected: "s1" };
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 1232 }, height: { kind: "fixed", px: 672 } }, () => {
      const r = story.routeChart("Routes", {
        scenes: [
          { key: "s1", title: "Arrival", column: 0, lane: 0, seen: true },
          { key: "s2", title: "The ferry", column: 1, lane: 0, seen: true, current: true },
          { key: "s3", title: "The storm", column: 1, lane: 1 },
          { key: "s4", title: "Dawn", column: 2, lane: 0, locked: true, ending: "Good end", good: true },
        ],
        links: [{ from: "s1", to: "s2", label: "Board", taken: true }, { from: "s1", to: "s3", label: "Stay" }, { from: "s2", to: "s4" }],
        selected: st.selected,
      });
      results.push(r);
      if (r.selected) st.selected = r.selected;
    });
  });
  engine.advance(10);
  tapOn(engine, byPath(engine, "Map/s4")!);
  expect(st.selected).toBe("s1");
  expect(results.filter((r) => r.refused).map((r) => r.refused)).toEqual(["s4"]);
  tapOn(engine, byPath(engine, "Detail/Jump")!);
  expect(results.filter((r) => r.jump).map((r) => r.jump)).toEqual(["s1"]);
  tapOn(engine, byPath(engine, "Map/s3")!);
  expect(st.selected).toBe("s3");
  expect(byPath(engine, "Detail/Jump") ?? null).toBeNull();
  const a = byPath(engine, "Map/s1")!;
  const b = byPath(engine, "Map/s2")!;
  const link = byPath(engine, "Map/Link#0")!;
  expect(Math.abs(link.x - (a.x + a.width))).toBeLessThan(2);
  expect(Math.abs(link.x + link.width - b.x)).toBeLessThan(2);
});
