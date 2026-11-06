import { expect, test } from "bun:test";

import { og } from "@origingame/engine";

import { rhythm, type SongSelectResult } from "@origingame/engine/extensions/rhythm/index.ts";

import { run, tapOn, byPath, key } from "../../../tests/support/patterns.ts";

import "../../../tests/support/patterns.ts";

// ── song select ─────────────────────────────────────────────────────────

test("songSelect: a press on another row selects it, on the selected one plays; the next-tab action steps over a locked chart; a locked song is refused", () => {
  // Wrong: a first press playing a song never selected, the step landing on a locked chart, or a locked song played.
  const results: SongSelectResult[] = [];
  const st = { selected: "a", difficulty: "easy" };
  const charts = [{ key: "easy", name: "Easy", level: 3 }, { key: "hard", name: "Hard", level: 9, locked: true }, { key: "expert", name: "Expert", level: 12 }];
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 1232 }, height: { kind: "fixed", px: 672 } }, () => {
      const r = rhythm.songSelect("Songs", {
        songs: [{ key: "a", title: "Alpha", difficulties: charts }, { key: "b", title: "Beta", difficulties: charts }, { key: "c", title: "Gamma", difficulties: charts, locked: true }],
        selected: st.selected,
        difficulty: st.difficulty,
      });
      results.push(r);
      if (r.selected) st.selected = r.selected;
      if (r.difficulty) st.difficulty = r.difficulty;
    });
  });
  engine.advance(10);
  tapOn(engine, byPath(engine, "Wheel/b")!);
  expect(st.selected).toBe("b");
  expect(results.some((r) => r.play)).toBeFalse();
  engine.advance(10);
  tapOn(engine, byPath(engine, "Wheel/b")!);
  expect(results.filter((r) => r.play).length).toBe(1);
  key(engine, "KeyE");
  expect(st.difficulty).toBe("expert");
  tapOn(engine, byPath(engine, "Wheel/c")!);
  engine.advance(10);
  tapOn(engine, byPath(engine, "Wheel/c")!);
  expect(results.filter((r) => r.refused).map((r) => r.refused)).toEqual(["c"]);
  expect(results.filter((r) => r.play).length).toBe(1);
});
