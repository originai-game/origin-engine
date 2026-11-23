// Every built-in fixture laid out in a game's 390x844 column on a touch
// phone (support/fixture-frames.ts) and held to the motion facts of
// support/fixture-motion.ts: each control a finger presses squashes under
// the press, and each screen and modal enters and leaves with motion.
//
// Ways it could regress that this catches: a control whose press handler
// moved to a wrapper the tree does not see as pressable, a pack whose
// Press.feedback lost its scale, a screen's root that lost the pack's
// Screen.frame, a modal without Modal.frame.

import { expect, test } from "bun:test";
import { fixtureFrames, HOSTS } from "./support/fixture-frames.ts";
import { presenceless, pressless } from "./support/fixture-motion.ts";

test("every built-in control squashes under a press, and every screen and modal enters and leaves with motion", async () => {
  const { list, frame, dispose } = await fixtureFrames();
  const host = HOSTS.find((h) => h.column)!;
  const found: string[] = [];
  try {
    for (const page of list) {
      const els = frame(page, host);
      for (const f of [...pressless(els, host), ...presenceless(els, host, page.place === "screen")]) found.push(`${page.name} · ${page.fixture} | ${f.key}: ${f.detail}`);
    }
  } finally {
    dispose();
  }
  expect(found).toEqual([]);
}, 900_000);
