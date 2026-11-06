import { expect, test } from "bun:test";

import { og } from "@origingame/engine";

import { photo, type PhotoModeResult } from "@origingame/engine/extensions/photo/index.ts";

import { run, byPath, tapOn, key } from "../../../tests/support/patterns.ts";

import "../../../tests/support/patterns.ts";

// ── photo mode ──────────────────────────────────────────────────────────

test("photoMode: a tab, a filter tile and the shutter answer; Back while hidden only brings the controls back, Back while shown exits; the flash follows a capture", () => {
  // Wrong: Back while hidden leaving photo mode (the player loses the shot), the chosen tile answered again, or no flash after Capture.
  const results: PhotoModeResult[] = [];
  const st = { tab: "camera", filter: "none", hidden: false };
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 1232 }, height: { kind: "fixed", px: 672 } }, () => {
      const r = photo.photoMode("Photo", {
        tabs: [{ key: "camera", label: "Camera" }, { key: "filters", label: "Filters" }],
        tab: st.tab,
        controls: [{ key: "fov", tab: "camera", label: "Field of view", value: 60, min: 20, max: 110, step: 1, unit: "°" }],
        filters: [{ key: "none", name: "None" }, { key: "noir", name: "Noir", tint: "#00000080" }],
        filter: st.filter,
        strength: 0.5,
        hidden: st.hidden,
      });
      results.push(r);
      if (r.tab) st.tab = r.tab;
      if (r.filter) st.filter = r.filter;
      if (r.hide) st.hidden = !st.hidden;
    });
  });
  engine.advance(10);
  expect(byPath(engine, "Rows/fov/Head/Value")?.text).toBe("60°");
  tapOn(engine, byPath(engine, "Tabs/filters")!);
  expect(st.tab).toBe("filters");
  tapOn(engine, byPath(engine, "Rows/Line#0/noir")!);
  tapOn(engine, byPath(engine, "Rows/Line#0/noir")!);
  expect(results.filter((r) => r.filter).map((r) => r.filter)).toEqual(["noir"]);
  tapOn(engine, byPath(engine, "Bar/Shutter")!);
  expect(results.filter((r) => r.capture).length).toBe(1);
  expect(byPath(engine, "Photo/Flash")).toBeDefined();
  engine.advance(60);
  expect(byPath(engine, "Photo/Flash") ?? null).toBeNull();
  key(engine, "KeyX");
  expect(st.hidden).toBeTrue();
  expect(byPath(engine, "Photo/Panel") ?? null).toBeNull();
  expect(byPath(engine, "Photo/Hint")).toBeDefined();
  key(engine, "Escape");
  expect(st.hidden).toBeFalse();
  expect(results.some((r) => r.exit)).toBeFalse();
  key(engine, "Escape");
  expect(results.filter((r) => r.exit).length).toBe(1);
});
