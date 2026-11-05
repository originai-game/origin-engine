// The launcher's contract (packages/launcher/og.ts): what it accepts as
// signed, what an install leaves in the cache, which version a command runs.
import { afterAll, describe, expect, test } from "bun:test";
import { spawnSync } from "node:child_process";
import { createHash, generateKeyPairSync, randomBytes, sign } from "node:crypto";
import { chmodSync, existsSync, lstatSync, mkdirSync, mkdtempSync, readFileSync, readlinkSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { compareVersions, config, ENGINE_PUBKEY, install, installed, pinOf, projectOf, verifyMinisign } from "../og.ts";
import { installers, pinnedKey } from "../build.ts";

const fixtures = join(import.meta.dir, "fixtures");
const made: string[] = [];
const scratch = (prefix: string) => {
  const d = mkdtempSync(join(tmpdir(), prefix));
  made.push(d);
  return d;
};
afterAll(() => {
  for (const d of made) rmSync(d, { recursive: true, force: true });
});
const read = (f: string) => readFileSync(join(fixtures, f));
const fixtureKey = read("test.pub").toString().split("\n")[1]!;

describe("minisign", () => {
  test("accepts minisign's prehashed and legacy signatures and returns the trusted comment", () => {
    expect(verifyMinisign(read("SHA256SUMS"), read("SHA256SUMS.minisig").toString(), fixtureKey)).toBe("origin-engine 1.0.0 test");
    expect(verifyMinisign(read("SHA256SUMS"), read("SHA256SUMS.legacy.minisig").toString(), fixtureKey)).toBe("origin-engine 1.0.0 legacy");
  });

  test("refuses a changed file, a changed trusted comment and another key", () => {
    const sums = Buffer.from(read("SHA256SUMS"));
    sums[0] = sums[0]! ^ 1;
    expect(() => verifyMinisign(sums, read("SHA256SUMS.minisig").toString(), fixtureKey)).toThrow(/does not verify/);
    const forged = read("SHA256SUMS.minisig").toString().replace("origin-engine 1.0.0 test", "origin-engine 9.9.9 test");
    expect(() => verifyMinisign(read("SHA256SUMS"), forged, fixtureKey)).toThrow(/trusted comment does not verify/);
    expect(() => verifyMinisign(read("SHA256SUMS"), read("SHA256SUMS.minisig").toString(), ENGINE_PUBKEY)).toThrow(/signed by key 0F1E686A6F406931, not the engine's F6CC11AC12997D89/);
  });

  test("pins the key docs/release.md states, and the installers carry the launcher whole", () => {
    expect(ENGINE_PUBKEY).toBe(pinnedKey()!);
    const launcher = readFileSync(join(import.meta.dir, "../og.ts"), "utf8");
    const { sh, ps1 } = installers();
    expect(sh).toContain(launcher);
    expect(ps1).toContain(launcher);
    // install.sh, run with sh, writes og.ts byte for byte and a shim that runs it.
    const home = scratch("og-launcher-home-");
    const r = spawnSync("sh", ["-s"], { input: sh, env: { ...process.env, ORIGIN_HOME: home }, encoding: "utf8" });
    expect(r.status).toBe(0);
    expect(readFileSync(join(home, "lib/og.ts"), "utf8")).toBe(launcher);
    const v = spawnSync(join(home, "bin/og"), ["--launcher-version"], { encoding: "utf8" });
    expect(v.stdout.trim()).toBe("og launcher 1");
    rmSync(home, { recursive: true, force: true });
  });
});

/** A channel in a directory, signed with a fresh key, holding fake engines whose og prints its arguments and engine. */
function channel() {
  const dir = scratch("og-channel-");
  const { publicKey, privateKey } = generateKeyPairSync("ed25519");
  const id = randomBytes(8);
  const pubkey = Buffer.concat([Buffer.from("Ed"), id, publicKey.export({ format: "der", type: "spki" }).subarray(12)]).toString("base64");
  const target = config().target;
  const signFile = (file: string, comment: string) => {
    const sig = sign(null, createHash("blake2b512").update(readFileSync(file)).digest(), privateKey);
    const global = sign(null, Buffer.concat([sig, Buffer.from(comment)]), privateKey);
    writeFileSync(`${file}.minisig`, `untrusted comment: test\n${Buffer.concat([Buffer.from("ED"), id, sig]).toString("base64")}\ntrusted comment: ${comment}\n${global.toString("base64")}\n`);
  };
  const publish = (v: string, opts: { holds?: string; corrupt?: boolean } = {}) => {
    const name = `origin-engine-${v}-${target}`;
    const stage = scratch("og-stage-");
    mkdirSync(join(stage, name, "bin"), { recursive: true });
    mkdirSync(join(stage, name, "engine"), { recursive: true });
    writeFileSync(join(stage, name, "origin-engine.json"), JSON.stringify({ build: { version: opts.holds ?? v } }));
    writeFileSync(join(stage, name, "bin/og"), `#!/bin/sh\necho "engine ${v} $OG_ENGINE_PACKAGE $*"\n`);
    chmodSync(join(stage, name, "bin/og"), 0o755);
    mkdirSync(join(dir, v), { recursive: true });
    const archive = join(dir, v, `${name}.tar.gz`);
    spawnSync("tar", ["-czf", archive, "-C", stage, name]);
    rmSync(stage, { recursive: true, force: true });
    const sum = createHash("sha256").update(readFileSync(archive)).digest("hex");
    writeFileSync(join(dir, v, "SHA256SUMS"), `${opts.corrupt ? "0".repeat(64) : sum}  ${name}.tar.gz\n`);
    signFile(join(dir, v, "SHA256SUMS"), `origin-engine ${v}`);
  };
  const latest = (v: string) => {
    mkdirSync(join(dir, "latest"), { recursive: true });
    writeFileSync(join(dir, "latest/manifest.json"), JSON.stringify({ version: v }));
  };
  const cache = scratch("og-cache-");
  return { dir, pubkey, publish, latest, cache, c: { cache, downloads: dir, pubkey, target } };
}

const unix = process.platform !== "win32";

describe("install", () => {
  test("verifies, unpacks into engines/<v>/<target>/, and never downloads an installed version again", async () => {
    const ch = channel();
    ch.publish("1.2.3");
    const dist = await install(ch.c, "1.2.3", true);
    expect(dist).toBe(join(ch.cache, "engines/1.2.3", ch.c.target));
    expect(JSON.parse(readFileSync(join(dist, "origin-engine.json"), "utf8")).build.version).toBe("1.2.3");
    // The archive's modes: og stays executable.
    if (unix) expect(statSync(join(dist, "bin/og")).mode & 0o111).not.toBe(0);
    rmSync(join(ch.dir, "1.2.3"), { recursive: true });
    expect(await install(ch.c, "1.2.3", true)).toBe(dist);
    expect(installed(ch.c)).toEqual(["1.2.3"]);
  });

  test("refuses an archive that does not match the signed SHA256SUMS, a signature by another key, and an archive of another version, leaving nothing", async () => {
    const ch = channel();
    ch.publish("1.0.0", { corrupt: true });
    await expect(install(ch.c, "1.0.0", true)).rejects.toThrow(/the signed SHA256SUMS says 0000/);
    ch.publish("1.0.1");
    await expect(install({ ...ch.c, pubkey: fixtureKey }, "1.0.1", true)).rejects.toThrow(/signed by key/);
    ch.publish("1.0.2", { holds: "1.0.1" });
    await expect(install(ch.c, "1.0.2", true)).rejects.toThrow(/holds engine 1.0.1, not 1.0.2/);
    expect(installed(ch.c)).toEqual([]);
    expect(existsSync(join(ch.cache, "engines/1.0.0", ch.c.target))).toBe(false);
  });

  test("two installs at once leave one complete copy", async () => {
    const ch = channel();
    ch.publish("2.0.0");
    const [a, b] = await Promise.all([install(ch.c, "2.0.0", true), install(ch.c, "2.0.0", true)]);
    expect(a).toBe(b);
    expect(existsSync(join(a, "bin/og"))).toBe(true);
  });
});

describe("versions and projects", () => {
  test("orders a dev build before its release and after the release before it", () => {
    expect(["0.7.8", "0.7.8-dev.17", "0.7.7", "0.7.8-dev.2", "0.10.0"].sort(compareVersions)).toEqual(["0.7.7", "0.7.8-dev.2", "0.7.8-dev.17", "0.7.8", "0.10.0"]);
  });

  test("finds the project from an argument inside it, else from the working directory", () => {
    const root = scratch("og-project-");
    mkdirSync(join(root, "game/src"), { recursive: true });
    writeFileSync(join(root, "game/project.ogt"), 'type project\nversion 1\n---\nengine "0.7.8-dev.17"\n');
    expect(projectOf(["run", "game/src", "--headless"], root)).toBe(resolve(root, "game"));
    expect(projectOf(["--version"], join(root, "game/src"))).toBe(resolve(root, "game"));
    expect(projectOf(["--version"], root)).toBeUndefined();
    expect(pinOf(join(root, "game"))).toBe("0.7.8-dev.17");
  });
});

describe.if(unix)("og", () => {
  const og = join(import.meta.dir, "../og.ts");
  const run = (ch: ReturnType<typeof channel>, args: string[], cwd: string) =>
    spawnSync("bun", [og, ...args], { cwd, encoding: "utf8", env: { ...process.env, ORIGIN_ENGINE_CACHE: ch.cache, ORIGIN_ENGINE_DOWNLOADS: ch.dir, ORIGIN_ENGINE_PUBKEY: ch.pubkey, OG_ENGINE_VERSION: "" } });

  test("runs the pinned version's og with OG_ENGINE_PACKAGE and keeps .og/engine on it; outside a project the latest release", () => {
    const ch = channel();
    ch.publish("0.7.8-dev.17");
    ch.publish("0.7.8");
    ch.latest("0.7.8");
    const game = scratch("og-game-");
    writeFileSync(join(game, "project.ogt"), 'type project\nversion 1\n---\nengine "0.7.8-dev.17"\n');
    const dist = join(ch.cache, "engines/0.7.8-dev.17", ch.c.target);
    const r = run(ch, ["test", "."], game);
    expect(r.stderr).toContain("downloading engine 0.7.8-dev.17");
    expect(r.stdout.trim()).toBe(`engine 0.7.8-dev.17 ${join(dist, "engine")} test .`);
    expect(lstatSync(join(game, ".og/engine")).isSymbolicLink()).toBe(true);
    expect(readlinkSync(join(game, ".og/engine"))).toBe(join(dist, "engine"));
    expect(run(ch, ["--version"], tmpdir()).stdout).toStartWith("engine 0.7.8 ");
  });

  test("engine list, path and prune keep the newest release and the project's pin", () => {
    const ch = channel();
    for (const v of ["0.6.0", "0.7.0", "0.7.8-dev.17", "0.7.8"]) {
      ch.publish(v);
      expect(run(ch, ["engine", "install", v], tmpdir()).status).toBe(0);
    }
    const game = scratch("og-game-");
    writeFileSync(join(game, "project.ogt"), 'engine "0.7.0"\n');
    expect(run(ch, ["engine", "list"], game).stdout).toContain("0.7.0  (this project's pin)");
    expect(run(ch, ["engine", "path"], game).stdout.trim()).toBe(join(ch.cache, "engines/0.7.0", ch.c.target));
    const pruned = run(ch, ["engine", "prune"], game);
    expect(pruned.stdout).toContain("removing engine 0.6.0");
    expect(pruned.stdout).toContain("removing engine 0.7.8-dev.17");
    expect(installed(ch.c)).toEqual(["0.7.8", "0.7.0"]);
  });
});
