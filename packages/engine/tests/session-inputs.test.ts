import { expect, test } from "bun:test";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { createServer } from "node:net";
import { freezeInputs } from "../src/node/session-inputs.ts";

test.skipIf(process.platform === "win32")("admitted inputs survive edits and keep source locations while output and stores stay live", () => {
  const dir = mkdtempSync(join(tmpdir(), "og-session-inputs-"));
  try {
    const root = join(dir, "project");
    const generations = join(dir, "generations");
    mkdirSync(join(root, "cooked"), { recursive: true });
    mkdirSync(join(root, ".og"));
    const entry = join(root, ".og", "game.native.js");
    const asset = join(root, "cooked", "model.ogb");
    const fake = join(dir, "fake.json");
    writeFileSync(entry, "export default 7;");
    writeFileSync(`${entry}.map`, JSON.stringify({ version: 3, sources: ["../src/main.ts", "og:worker"], sourceRoot: "", mappings: "" }));
    writeFileSync(asset, "old mesh");
    writeFileSync(join(root, "project.ogt"), "old project");
    writeFileSync(fake, "old responses");
    symlinkSync(asset, join(root, "cooked", "alias.ogb"));
    const argv = ["--entry", entry, "--root", root, "--platform-fake", fake, "--platform-store", "store.json", "--out", "output"];
    const frozen = freezeInputs(argv, dir, generations);
    try {
      writeFileSync(entry, "export default 9;");
      writeFileSync(asset, "new mesh");
      writeFileSync(fake, "new responses");
      expect(readFileSync(frozen.argv[1]!, "utf8")).toBe("export default 7;");
      expect(readFileSync(join(frozen.inputRoot!, "cooked", "alias.ogb"), "utf8")).toBe("old mesh");
      expect(readFileSync(join(frozen.inputRoot!, "project.ogt"), "utf8")).toBe("old project");
      expect(readFileSync(frozen.argv[5]!, "utf8")).toBe("old responses");
      expect(frozen.argv.slice(6)).toEqual(argv.slice(6));
      expect(frozen.argv[3]).toBe(root);
      expect(existsSync(join(frozen.inputRoot!, ".og"))).toBeFalse();
      expect(JSON.parse(readFileSync(`${frozen.argv[1]}.map`, "utf8")).sources).toEqual([join(root, "src/main.ts"), "og:worker"]);
      const next = freezeInputs(argv, dir, generations);
      try {
        expect(next.argv[1]).not.toBe(frozen.argv[1]);
        expect(readFileSync(next.argv[1]!, "utf8")).toBe("export default 9;");
      } finally {
        next.dispose();
      }
    } finally {
      frozen.dispose();
    }
    expect(readdirSync(generations)).toEqual([]);
    expect(existsSync(dirname(entry))).toBeTrue();
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test.skipIf(process.platform === "win32")("cyclic input links fail admission and remove the partial generation", () => {
  const dir = mkdtempSync(join(tmpdir(), "og-session-cycle-"));
  try {
    const root = join(dir, "project");
    const generations = join(dir, "generations");
    mkdirSync(root);
    symlinkSync(root, join(root, "cycle"));
    expect(() => freezeInputs(["--root", root], dir, generations)).toThrow();
    expect(readdirSync(generations)).toEqual([]);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test.skipIf(process.platform === "win32")("socket inputs fail admission without leaving a generation", async () => {
  const dir = mkdtempSync(join(tmpdir(), "og-session-special-"));
  const root = join(dir, "project");
  const generations = join(dir, "generations");
  mkdirSync(root);
  const server = createServer((socket) => socket.destroy());
  await new Promise<void>((resolve) => server.listen(join(root, "service.sock"), resolve));
  try {
    expect(() => freezeInputs(["--root", root], dir, generations)).toThrow("not a regular file");
    expect(readdirSync(generations)).toEqual([]);
  } finally {
    server.close();
    rmSync(dir, { recursive: true, force: true });
  }
});
