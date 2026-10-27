// The install test (docs/release.md §3): what an author does on a machine
// that has never seen this repository, from the release tarballs or from the
// registry, on this machine and in the linux-x64 container.
//
//   bun tests/install/run.ts --from dist/release/<v> --version <v> [--manifest <manifest.json>]
//                            [--registry] [--host-only | --container-only] [--keep] [--image <name>]
//
// Steps, in an empty directory with no OG_ENGINE_PACKAGE, OG_NATIVE_ADDON or
// ORIGIN_OG_BIN in the environment:
//   1. a consumer that installs @origingame/create-game; `create-game demo`
//   2. the demo's `bun install` (from the tarballs: the engine and this
//      platform's package as file: specs; tsrs from the registry)
//   3. `og --version` prints the version; `og doctor` passes (the renderer
//      line may be `none`)
//   4. `og check`, `og test` (frames unchecked without a renderer), `og build
//      --publish`: og-build.json names the version, the shipped publish wasm is
//      the package's byte for byte
//   5. the runner archetype through steps 1 to 4's test
//   6. the 2D starter (`create-game --profile 2d`) through the same test
//   7. the knowledge inside the engine package (knowledge/): its validate.ts,
//      mount.ts --package into a stand-in origin-agent knowledge/ tree, then --check
//
// --container builds tests/install/Dockerfile (node 22, bun) and runs this
// script inside it with --host-only, the tarballs mounted read-only. With
// --manifest the pass is recorded in manifest.json (verified.host,
// verified.container) for scripts/release.sh publish to read.

import { spawnSync, type SpawnSyncReturns } from "node:child_process";
import { createHash } from "node:crypto";
import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const args = process.argv.slice(2);
const opt = (name: string): string | undefined => {
  const i = args.indexOf(name);
  return i >= 0 ? args[i + 1] : undefined;
};
const flag = (name: string) => args.includes(name);
const FROM = opt("--from") ? resolve(opt("--from")!) : undefined;
const VERSION = opt("--version");
const MANIFEST = opt("--manifest") ? resolve(opt("--manifest")!) : undefined;
const REGISTRY = flag("--registry");
const KEEP = flag("--keep");
const IMAGE = opt("--image") ?? "og-install:node22-bun";
const HOST_ONLY = flag("--host-only");
const CONTAINER_ONLY = flag("--container-only");
const PLATFORM = `${process.platform}-${process.arch}`;
const SCOPE = "@origingame";
const HERE = dirname(fileURLToPath(import.meta.url));

if (!VERSION || (!FROM && !REGISTRY)) {
  process.stderr.write("usage: bun tests/install/run.ts --from <dir> --version <v> [--registry] [--manifest <file>] [--host-only|--container-only] [--keep] [--image <name>]\n");
  process.exit(2);
}

const t0 = Date.now();
const log = (s: string) => process.stdout.write(`[install ${((Date.now() - t0) / 1000).toFixed(1)}s] ${s}\n`);

class Step {
  constructor(readonly name: string) {
    log(`-- ${name}`);
  }
  fail(why: string): never {
    throw new Error(`${this.name}: ${why}`);
  }
}

/** The environment a fresh author has: no engine checkout, no overrides. */
function env(): NodeJS.ProcessEnv {
  const e = { ...process.env };
  for (const k of ["OG_ENGINE_PACKAGE", "OG_NATIVE_ADDON", "ORIGIN_OG_BIN", "OG_TSRS", "CARGO_TARGET_DIR"]) delete e[k];
  e.CI = "1";
  return e;
}

function run(step: Step, cmd: string[], cwd: string, opts: { allowFail?: boolean; timeoutMs?: number } = {}): SpawnSyncReturns<string> {
  log(`   $ ${cmd.join(" ")}`);
  const r = spawnSync(cmd[0]!, cmd.slice(1), { cwd, env: env(), encoding: "utf8", stdio: ["ignore", "pipe", "pipe"], timeout: opts.timeoutMs ?? 20 * 60_000, maxBuffer: 64 << 20 });
  if (r.error) step.fail(`${cmd[0]} did not start: ${r.error.message}`);
  if (r.status !== 0 && !opts.allowFail) {
    step.fail(`\`${cmd.join(" ")}\` exited ${r.status}\n--- stdout\n${r.stdout.slice(-4000)}\n--- stderr\n${r.stderr.slice(-4000)}`);
  }
  return r;
}

function sha256(path: string): string {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

function tarball(name: string): string {
  const f = join(FROM!, `origingame-${name}-${VERSION}.tgz`);
  if (!existsSync(f)) throw new Error(`no ${f}`);
  return f;
}

/** The dependency spec of a package: the registry's version, or the tarball. */
function spec(name: string): string {
  return REGISTRY ? VERSION! : `file:${tarball(name)}`;
}

/** package.json `overrides` so the engine's optionalDependency on this platform's package resolves to the tarball. */
function overrides(): Record<string, string> {
  return REGISTRY ? {} : { [`${SCOPE}/engine-${PLATFORM}`]: `file:${tarball(`engine-${PLATFORM}`)}` };
}

function writeJson(path: string, value: unknown): void {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`);
}

/** Points a scaffolded project's engine dependency at what this run installs. */
function pinProject(dir: string): void {
  const p = join(dir, "package.json");
  const pkg = JSON.parse(readFileSync(p, "utf8")) as { dependencies: Record<string, string>; overrides?: Record<string, string> };
  if (pkg.dependencies[`${SCOPE}/engine`] !== VERSION) throw new Error(`${p}: depends on ${SCOPE}/engine ${pkg.dependencies[`${SCOPE}/engine`]}, the scaffold should write ${VERSION}`);
  if (!REGISTRY) {
    pkg.dependencies[`${SCOPE}/engine`] = spec("engine");
    pkg.overrides = { ...pkg.overrides, ...overrides() };
    writeJson(p, pkg);
  }
}

function og(step: Step, project: string, ...rest: string[]): SpawnSyncReturns<string> {
  return run(step, ["bun", "x", "og", ...rest], project);
}

function host(): void {
  const work = mkdtempSync(join(tmpdir(), "og-install-"));
  log(`work directory ${work} (${PLATFORM}, ${REGISTRY ? "from the registry" : `from ${FROM}`})`);
  try {
    // 1. The consumer and the scaffold.
    let step = new Step("1 create-game");
    const consumer = join(work, "consumer");
    writeJson(join(consumer, "package.json"), { name: "consumer", private: true, dependencies: { [`${SCOPE}/create-game`]: spec("create-game") } });
    run(step, ["bun", "install"], consumer);
    const create = join(consumer, "node_modules", SCOPE, "create-game", "bin", "create.ts");
    if (!existsSync(create)) step.fail(`${create} is not installed`);
    run(step, ["bun", create, join(work, "demo"), "--name", "Demo", "--no-prompt"], consumer);
    const demo = join(work, "demo");
    for (const f of ["project.ogt", "src/main.ts", "package.json", "tests", "assets/looks/demo.ogt", ".gitignore", ".gitattributes"]) {
      if (!existsSync(join(demo, f))) step.fail(`the scaffold wrote no ${f}`);
    }
    const pinned = /^engine\s+"([^"]+)"/m.exec(readFileSync(join(demo, "project.ogt"), "utf8"))?.[1];
    if (pinned !== VERSION) step.fail(`project.ogt pins engine "${pinned}", expected "${VERSION}"`);

    // 2. The project's install.
    step = new Step("2 bun install");
    pinProject(demo);
    run(step, ["bun", "install"], demo);
    const enginePkg = join(demo, "node_modules", SCOPE, "engine");
    const installed = (JSON.parse(readFileSync(join(enginePkg, "package.json"), "utf8")) as { version: string }).version;
    if (installed !== VERSION) step.fail(`installed ${SCOPE}/engine is ${installed}`);
    const ogBin = join(demo, "node_modules", SCOPE, `engine-${PLATFORM}`, process.platform === "win32" ? "og.exe" : "og");
    if (!existsSync(ogBin)) step.fail(`no ${ogBin}: the platform package did not install`);
    for (const f of ["dist/web-publish/origin-engine_bg.wasm", "dist/web-publish-2d/origin-engine_bg.wasm", "dist/web-publish/origin-engine-lanes_bg.wasm", "shell", "types/index.d.ts", "schema/api.schema.json"]) {
      if (!existsSync(join(enginePkg, f))) step.fail(`the engine package has no ${f}`);
    }

    // 3. og --version, og doctor.
    step = new Step("3 og --version, og doctor");
    const v = og(step, demo, "--version");
    if (!v.stdout.includes(`og ${VERSION}`)) step.fail(`og --version printed ${JSON.stringify(v.stdout.trim())}`);
    const doctor = og(step, demo, "doctor");
    for (const want of ["ok       engine", "ok       addon", "ok       wasm-publish", "ok       pin"]) {
      if (!doctor.stdout.includes(want)) step.fail(`og doctor has no "${want}" line:\n${doctor.stdout}`);
    }
    const renderer = /^(ok|none)\s+renderer\s+(.*)$/m.exec(doctor.stdout);
    log(`   renderer: ${renderer ? `${renderer[1]} ${renderer[2]}` : "(no line)"}`);
    if (!doctor.stdout.includes("tsrs") || /missing\s+tsrs/.test(doctor.stdout)) step.fail(`og doctor: tsrs is not beside the package:\n${doctor.stdout}`);
    if (process.platform === "win32" && !doctor.stdout.includes("og check uses tsgo 7.0.2 through bun x")) step.fail(`og doctor: Windows must report its tsgo fallback:\n${doctor.stdout}`);
    log(`   checker: ${doctor.stdout.split("\n").find(line => /^(ok|optional)\s+tsrs\s/.test(line))?.trim()}`);

    // 4. check, test, build --publish.
    step = new Step("4 og check, og test, og build --publish");
    og(step, demo, "check", ".");
    const test = og(step, demo, "test", ".");
    log(`   og test: ${test.stdout.trim().split("\n").slice(-2).join(" | ")}`);
    og(step, demo, "build", ".", "--publish");
    const buildJson = join(demo, "dist", "og-build.json");
    if (!existsSync(buildJson)) step.fail("og build --publish wrote no dist/og-build.json");
    const build = JSON.parse(readFileSync(buildJson, "utf8")) as { engine: { version: string; pinned: string } };
    if (build.engine.version !== VERSION || build.engine.pinned !== VERSION) step.fail(`og-build.json engine ${JSON.stringify(build.engine)}`);
    const shipped = join(demo, "dist", "origin-engine_bg.wasm");
    if (!existsSync(shipped)) step.fail("the build has no origin-engine_bg.wasm");
    const packaged = join(enginePkg, "dist", "web-publish", "origin-engine_bg.wasm");
    if (sha256(shipped) !== sha256(packaged)) step.fail("the shipped publish wasm differs from the package's dist/web-publish/origin-engine_bg.wasm");

    // 5. An archetype.
    step = new Step("5 the runner archetype");
    const runner = join(work, "demo-runner");
    run(step, ["bun", create, runner, "--name", "Demo Runner", "--archetype", "runner", "--no-prompt"], consumer);
    for (const f of [".gitignore", ".gitattributes"]) if (!existsSync(join(runner, f))) step.fail(`the archetype scaffold wrote no ${f}`);
    pinProject(runner);
    run(step, ["bun", "install"], runner);
    og(step, runner, "test", ".");

    // 6. The 2D starter: the package's template-2d/, written for --profile 2d.
    step = new Step("6 the 2D starter (--profile 2d)");
    const flat = join(work, "demo-2d");
    run(step, ["bun", create, flat, "--name", "Demo 2D", "--profile", "2d", "--no-prompt"], consumer);
    for (const f of ["project.ogt", "src/main.ts", "tests", "assets/looks/demo-2d.ogt", ".gitignore", ".gitattributes"]) {
      if (!existsSync(join(flat, f))) step.fail(`the 2D scaffold wrote no ${f}`);
    }
    const manifest2d = readFileSync(join(flat, "project.ogt"), "utf8");
    if (!/^profile\s+"2d"/m.test(manifest2d)) step.fail("the 2D scaffold's project.ogt has no `profile \"2d\"` line");
    const pinned2d = /^engine\s+"([^"]+)"/m.exec(manifest2d)?.[1];
    if (pinned2d !== VERSION) step.fail(`the 2D scaffold's project.ogt pins engine "${pinned2d}", expected "${VERSION}"`);
    pinProject(flat);
    run(step, ["bun", "install"], flat);
    og(step, flat, "check", ".");
    const test2d = og(step, flat, "test", ".");
    log(`   og test: ${test2d.stdout.trim().split("\n").slice(-2).join(" | ")}`);

    // 7. The knowledge, inside the installed engine package (knowledge/).
    step = new Step("7 knowledge: validate, mount --package, --check");
    const knPkg = join(enginePkg, "knowledge");
    if (!existsSync(join(knPkg, "knowledge.json"))) step.fail(`${enginePkg} carries no knowledge/`);
    run(step, ["bun", join(knPkg, "scripts", "validate.ts")], demo);
    const platform = join(work, "platform");
    mkdirSync(join(platform, "knowledge/game/skills/someone-else"), { recursive: true });
    writeFileSync(join(platform, "knowledge/game/skills/someone-else/SKILL.md"), "---\nname: someone-else\ndescription: not ours\n---\n");
    mkdirSync(join(platform, "knowledge/game-scaffold/three"), { recursive: true });
    writeFileSync(join(platform, "knowledge/game-scaffold/three/package.json"), "{}\n");
    const mounted = run(step, ["bun", join(knPkg, "scripts", "mount.ts"), platform, "--package", enginePkg], demo);
    if (!/text files: \d+/.test(mounted.stdout)) step.fail(`mount printed no summary:\n${mounted.stdout}`);
    run(step, ["bun", join(knPkg, "scripts", "mount.ts"), platform, "--package", enginePkg, "--check"], demo);
    if (!existsSync(join(platform, "knowledge/game/skills/og-game/SKILL.md"))) step.fail("the mount wrote no og-game skill");
    if (!existsSync(join(platform, "knowledge/game-scaffold/origin-engine-ts/project.ogt"))) step.fail("the mount wrote no scaffold overlay");
    if (!existsSync(join(platform, "knowledge/game/skills/someone-else/SKILL.md"))) step.fail("the mount removed someone else's skill");

    log(`PASS on ${PLATFORM}`);
    if (!KEEP) rmSync(work, { recursive: true, force: true });
  } catch (e) {
    log(`FAIL on ${PLATFORM}: ${e instanceof Error ? e.message : String(e)}`);
    log(`the work directory is kept: ${work}`);
    throw e;
  }
}

function container(): void {
  const step = new Step("container (linux-x64)");
  const repo = resolve(HERE, "..", "..");
  run(step, ["docker", "build", "-q", "-t", IMAGE, "--platform", "linux/amd64", HERE], repo, { timeoutMs: 15 * 60_000 });
  const mounts = ["-v", `${HERE}:/install:ro`];
  if (FROM) mounts.push("-v", `${FROM}:/tgz:ro`);
  const inner = ["bun", "/install/run.ts", "--version", VERSION!, "--host-only", ...(REGISTRY ? ["--registry"] : ["--from", "/tgz"])];
  const r = spawnSync("docker", ["run", "--rm", "--platform", "linux/amd64", ...mounts, "-w", "/tmp", IMAGE, ...inner], { encoding: "utf8", stdio: ["ignore", "inherit", "inherit"], timeout: 60 * 60_000 });
  if (r.status !== 0) step.fail(`the container run exited ${r.status}`);
}

function record(where: "host" | "container"): void {
  if (!MANIFEST) return;
  const m = JSON.parse(readFileSync(MANIFEST, "utf8")) as { verified?: Record<string, unknown> };
  m.verified = { ...m.verified, [where]: { at: new Date().toISOString(), platform: where === "host" ? PLATFORM : "linux-x64 (container)", source: REGISTRY ? "registry" : "tarballs" } };
  writeFileSync(MANIFEST, `${JSON.stringify(m, null, 2)}\n`);
}

try {
  if (!CONTAINER_ONLY) {
    host();
    record("host");
  }
  if (!HOST_ONLY) {
    container();
    record("container");
  }
  log("install test PASSED");
} catch (e) {
  process.stderr.write(`install test FAILED: ${e instanceof Error ? e.message : String(e)}\n`);
  process.exit(1);
}
