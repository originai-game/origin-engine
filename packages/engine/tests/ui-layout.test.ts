// Every built-in fixture laid out the way games compose it (docs/ui.md
// §6.2): mounted on the surface and as the child of a game's 390x844 flex
// column on a touch phone, and on a 1280x720 desktop. Three facts are held
// for each (support/fixture-layout.ts): a control a finger cannot take (one
// drawn under 24 px, or under 44 px with its grown target on another
// control), a control drawn off the surface with nothing to scroll it in,
// and a line of text cut by its box.
//
// The findings the built-ins still have are listed in
// fixtures/ui-layout-known.json, each to be fixed in its component: a new
// finding fails, and so does a listed one that is gone (take it off the
// list), so the list only shrinks. OG_UI_LAYOUT_WRITE=1 rewrites it.
//
// Ways the layout could regress that this catches: the mount's grid cell
// sized to its content (a long row carried a screen off a phone), a
// component's control shrunk under a finger, a nowrap label put back
// without truncate or a scroller.

import { expect, test } from "bun:test";
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { fixtureFrames, HOSTS } from "./support/fixture-frames.ts";
import { cutText, offSurface, touchTargets } from "./support/fixture-layout.ts";

const KNOWN = fileURLToPath(new URL("./fixtures/ui-layout-known.json", import.meta.url));

test("every built-in fixture keeps its controls in reach of a finger and on the surface, and its text uncut", async () => {
  const { list, frame, dispose } = await fixtureFrames();
  const found = new Map<string, string>();
  try {
    for (const page of list) {
      for (const host of HOSTS) {
        const els = frame(page, host);
        for (const f of [...touchTargets(els, host), ...offSurface(els, host), ...cutText(els, host)]) {
          found.set(`${host.name} | ${page.name} · ${page.fixture} | ${f.key}`, f.detail);
        }
      }
    }
  } finally {
    dispose();
  }
  if (process.env.OG_UI_LAYOUT_WRITE === "1") writeFileSync(KNOWN, JSON.stringify([...found.keys()].sort(), null, 2) + "\n");
  const known = new Set(JSON.parse(readFileSync(KNOWN, "utf8")) as string[]);
  const added = [...found].filter(([k]) => !known.has(k)).map(([k, d]) => `${k}: ${d}`);
  const gone = [...known].filter((k) => !found.has(k));
  expect({ added, gone }).toEqual({ added: [], gone: [] });
}, 900_000);

// HudBar fills to value over max: the port drew value x 100% (2184 of 3420
// filled the whole well). Its fill is the meter's child at its left with
// the fill's share of its width; its ghost, at the ghost's share, is wider,
// and with no ghost it rests at the fill.
test("a HudBar fills to its value over its max", async () => {
  const { list, frame, dispose } = await fixtureFrames((p) => p.name === "hud/bar" && (p.fixture === "default" || p.fixture === "mana" || p.fixture === "empty"));
  const host = HOSTS.find((h) => h.column)!;
  try {
    const widths = Object.fromEntries(list.map((page) => {
      const els = frame(page, host);
      // The well: the thin box that holds the ghost and the fill.
      const meter = els.find((e) => e.height > 0 && e.height < 20 && els.some((k) => k.parent === e.id))!;
      const kids = els.filter((e) => e.parent === meter.id).map((e) => Math.round((e.width / meter.width) * 100) / 100);
      return [page.fixture, kids.slice(0, 2)];
    }));
    expect(widths).toEqual({ default: [0.78, 0.64], mana: [0.72, 0.72], empty: [0, 0] });
  } finally {
    dispose();
  }
}, 900_000);
