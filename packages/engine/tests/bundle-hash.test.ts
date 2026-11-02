// A replay's game identity (bundleSha256, src/node/bundle-hash.ts) does not
// depend on where the game was bundled: templates/game-2d copied into two
// directories at different depths and bundled by src/node/build.ts as og
// does (from the directory og was run in) hashes the same. The bundles
// themselves differ: bun's source markers name each directory, and its
// `//# debugId=` hashes the paths the bundle and its source map carry (the
// engine's absolute path on another machine), so a hash that kept either
// fails here, as a golden recorded on one machine read as a changed game on
// another.
import { expect, test } from "bun:test";
import { cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { bundleHash } from "../src/node/bundle-hash.ts";

declare const Bun: {
  spawnSync(cmd: string[], o: { cwd: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");

test("the same game bundled in two directories has one bundle hash", () => {
  const root = mkdtempSync(join(tmpdir(), "og-bundle-hash-"));
  try {
    const bundles = [join(root, "a", "game"), join(root, "b", "deeper", "game")].map((dir) => {
      mkdirSync(dirname(dir), { recursive: true });
      cpSync(join(repo, "templates", "game-2d", "src"), join(dir, "src"), { recursive: true });
      const out = join(dir, ".og", "game.native.js");
      const r = Bun.spawnSync(["bun", join(pkg, "src/node/build.ts"), "--entry", join(dir, "src/main.ts"), "--out", out, "--target", "native"], { cwd: root });
      if (r.exitCode !== 0) throw new Error(new TextDecoder().decode(r.stderr));
      return out;
    });
    expect(readFileSync(bundles[0]!, "utf8")).not.toBe(readFileSync(bundles[1]!, "utf8"));
    expect(bundleHash(bundles[0]!)).toBe(bundleHash(bundles[1]!));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}, 60_000);
