// Every `.ogt` example a guide writes holds to og-core's line shape, so an
// author who copies one cooks: these tests prove the committed guides pass
// and that each kind of broken example is caught.

import { expect, test } from "bun:test";
import { defaultRoots } from "../scripts/lib.ts";
import { guideTexts, ogtShape } from "../scripts/validate.ts";

const guides = () => guideTexts(defaultRoots());

/** The guides with `name`'s text changed by `f`. */
function withGuide(name: string, f: (s: string) => string): Map<string, string> {
  const all = guides();
  all.set(name, f(all.get(name)!));
  return all;
}

test("every .ogt example in the guides has the parser's line shape; a one-line block is refused", () => {
  expect(guides().size).toBeGreaterThan(0);
  expect(ogtShape(guides())).toEqual([]);
  const one = ogtShape(withGuide("looks", (s) => s.replace("PostEffects {\n  vignetteIntensity 0.3\n}", "PostEffects { vignetteIntensity 0.3 }")));
  expect(one.length).toBe(2);
  expect(one[0]).toMatch(/^wiki\/guide-looks\.md:\d+: `PostEffects \{ vignetteIntensity 0\.3 \}` opens a block mid-line/);
  expect(one[1]).toContain("'}' must be alone on its line");
  // A file without its header, and a block left open.
  const head = ogtShape(new Map([["og-x", "```text\ntype terrain\n---\nlayer {\n  tile 6\n```\n"]]));
  expect(head).toEqual([
    "wiki/guide-og-x.md:2: an .ogt example opens with `type` but not `version <n>` and `---` after it; the parser refuses a file without its header",
    "wiki/guide-og-x.md:6: an .ogt example leaves 1 block(s) open",
  ]);
  // TypeScript and shell blocks are not .ogt.
  expect(ogtShape(new Map([["og-x", "```ts\nconst a = { b: 1 };\n```\n"]]))).toEqual([]);
});
