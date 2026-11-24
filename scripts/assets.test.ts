import { expect, test } from "bun:test";
import { join } from "node:path";

// Binaries live in the asset store (docs/ci.md §The asset store). Git keeps only what a bare cargo build
// compiles in, the knowledge package's previews (mounted by `git archive`), the install test's sample,
// which runs in a sandbox without the store, and vendored third-party data.
const IN_GIT = new Set([
  "crates/og-api/search/words.bin",
  "crates/og-render/data/blue-noise-64.r8",
  "crates/og-render/data/grass-tile-1024.f32",
  "templates/looks/assets/canvas/linen.png",
  "templates/looks/assets/canvas/paper.png",
  "templates/looks/assets/canvas/watercolor.png",
  "tests/fixtures/ui-text/assets/fonts/Inter-Display.ttf",
  "tests/fixtures/ui-text/assets/fonts/NotoSansArabic-subset.ttf",
  "tests/fixtures/ui-text/assets/fonts/NotoSansDevanagari-subset.ttf",
  "tests/install/fixtures/tone.mp3",
]);
test("Git holds no binary outside its list: every other one is pinned in the asset store", () => {
  const git = (...args: string[]) => Bun.spawnSync(["git", ...args], { cwd: join(import.meta.dir, "..") }).stdout.toString().split("\0").filter(Boolean);
  // Git's own reading of the committed bytes as binary, whatever the file's
  // name: every non-empty file in the index that `git grep -I` (threaded,
  // unlike `ls-files --eol`, which reads each blob in turn) does not list as text.
  const EMPTY = "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391";
  const text = new Set(git("grep", "--cached", "-I", "-l", "-z", "-e", ""));
  // A symlink's blob is its target's path (the checkout's node_modules/@origingame/engine).
  const binary = git("ls-files", "-s", "-z")
    .filter((line) => line.split(" ")[1] !== EMPTY && !line.startsWith("120000 "))
    .map((line) => line.slice(line.indexOf("\t") + 1))
    .filter((path) => !text.has(path));
  const stray = binary.filter((path) => !IN_GIT.has(path) && !/^packages\/knowledge\/previews\/[^/]+\.webp$/.test(path) && !path.startsWith("third_party/"));
  expect(stray).toEqual([]);
});
