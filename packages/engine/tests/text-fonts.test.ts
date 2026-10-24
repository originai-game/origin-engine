// The browser harness's cook of og-ui's text fonts (tests/web/text-fonts.ts),
// which reference-gfx.ts runs before og-ui's cooked-font text frames: every
// font the scenes take (crates/og-ui/tests/support/text_scenes.rs FONTS) has
// its settings in crates/og-ui/tests/text-fonts, is cooked, and its blob is
// among the files handed to og-tests-web, under the path the manifest gives.
// Runs `og cook` (tests/support/og-bin.ts), no browser.

import { expect, test } from "bun:test";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { cookTextFonts } from "./web/text-fonts.ts";

const repo = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");

test("the text frames' fonts cook for the browser harness as the scenes name them", () => {
  const rust = readFileSync(join(repo, "crates", "og-ui", "tests", "support", "text_scenes.rs"), "utf8");
  const list = /pub const FONTS: \[&str; \d+\] = \[([^\]]*)\]/.exec(rust)?.[1];
  expect(list).toBeDefined();
  const named = [...list!.matchAll(/"([^"]+)"/g)].map((m) => m[1]!).sort();
  expect(named.length).toBe(6);
  const t = cookTextFonts(repo, join(repo, "target", "evidence", "text-fonts-test"));
  expect(t.fonts).toEqual(named);
  for (const f of named) {
    const file = `fonts/${f}.ogb`;
    expect(t.manifest).toContain(`output "fonts/${f}" {`);
    expect(t.manifest).toContain(`file "${file}"`);
    expect(t.files).toContain(file);
    expect(existsSync(join(t.cooked, file))).toBe(true);
  }
}, 120_000);
