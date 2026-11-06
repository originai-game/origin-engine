// The install test (docs/release.md §3): what an author does on a machine
// that has never seen this repository, from the release archives (the
// download channel's files, scripts/release.sh pack), on this machine and in
// the linux-x64 container. No npm, no bun install: the og launcher
// (packages/launcher, through its install.sh or install.ps1) downloads the
// version from a channel laid out like /downloads/engine (the parent of
// --from) and runs it.
//
//   bun tests/install/run.ts --from dist/release/<v> --version <v> [--record <release.json>] [--pubkey <key>]
//                            [--installer <install.sh|install.ps1>] [--core] [--host-only | --container-only]
//                            [--keep] [--image <name>]
//
// --core runs steps 1 to 5 and 7's validate only: what every version from
// 0.5.0 has (a backfilled archive of an older version, docs/release.md §3);
// 4b, 6, 7's mount, 8 and 9 probe features of later versions.
//
// Steps, in an empty directory with no OG_ENGINE_PACKAGE, OG_NATIVE_ADDON or
// ORIGIN_OG_BIN in the environment:
//   1. the launcher's installer (built from packages/launcher, or --installer)
//      into an empty ORIGIN_HOME; `og engine install <v>` verifies
//      SHA256SUMS.minisig (with --pubkey, else the engine's pinned key) and the
//      archive's sha256 and unpacks it into an empty cache
//      (engines/<v>/<target>/); its layout and origin-engine.json's build record
//   2. `og new demo --engine <v>` scaffolds from the dist's create/: the pin,
//      no node_modules, `.og/engine` pointing at the dist's engine/
//   3. `og --version` prints the version; `og doctor` passes (the renderer
//      line may be `none`: then 4b's og style check must refuse, naming it)
//   4. `og check`, `og test` (frames unchecked without a renderer), `og build
//      --publish`: og-build.json names the version, the shipped publish wasm is
//      the dist's byte for byte
//   5. the runner archetype through steps 2 to 4's test
//   6. the 2D starter (`--profile 2d`) through the same test
//   7. the knowledge inside the dist's engine/ (knowledge/): its validate.ts,
//      mount.ts --package into a stand-in origin-agent knowledge/ tree, then --check
//   8. a project with an MP3 sound (fixtures/tone.mp3): og run cooks it, plays
//      it (og.AudioPlayback's playhead at 0.5 s) and the mix is not silent
//   9. a look naming the built-in fantasy style: og cook reads it from the
//      engine's styles/ and the bundle adds that pack only to the demo's own
//
// --container builds tests/install/Dockerfile (node 22, bun) and runs this
// script inside it with --host-only and OG_RENDERER=none (it has no GPU), the
// archives mounted read-only. With
// --record the pass is recorded in release.json (verified.host,
// verified.container) for scripts/release.sh publish to read.

import { spawnSync, type SpawnSyncReturns } from "node:child_process";
import { createHash } from "node:crypto";
import { cpSync, existsSync, lstatSync, mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
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
const RECORD = opt("--record") ? resolve(opt("--record")!) : undefined;
const PUBKEY = opt("--pubkey");
const INSTALLER = opt("--installer") ? resolve(opt("--installer")!) : undefined;
const CORE = flag("--core");
const KEEP = flag("--keep");
const IMAGE = opt("--image") ?? "og-install:node22-bun";
const HOST_ONLY = flag("--host-only");
const CONTAINER_ONLY = flag("--container-only");
const PLATFORM = `${process.platform}-${process.arch}`;
const TRIPLE = ({ "linux-x64": "x86_64-unknown-linux-gnu", "darwin-arm64": "aarch64-apple-darwin", "win32-x64": "x86_64-pc-windows-msvc" } as Record<string, string>)[PLATFORM];
const HERE = dirname(fileURLToPath(import.meta.url));

if (!VERSION || !FROM) {
  process.stderr.write("usage: bun tests/install/run.ts --from <dir> --version <v> [--record <release.json>] [--host-only|--container-only] [--keep] [--image <name>]\n");
  process.exit(2);
}
if (!TRIPLE) {
  process.stderr.write(`the engine ships no archive for ${PLATFORM}\n`);
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

/** The environment a fresh author has: no engine checkout, no overrides; the launcher's home, cache and channel. */
const launcherEnv: Record<string, string> = {};
function env(): NodeJS.ProcessEnv {
  const e = { ...process.env };
  for (const k of ["OG_ENGINE_PACKAGE", "OG_NATIVE_ADDON", "ORIGIN_OG_BIN", "OG_TSRS", "CARGO_TARGET_DIR", "OG_ENGINE_VERSION", "ORIGIN_ENGINE_PUBKEY"]) delete e[k];
  Object.assign(e, launcherEnv);
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

/** A 16-bit PCM WAV's RMS (0 to 1) over its data chunk. */
function wavRms(path: string): number {
  const b = readFileSync(path);
  let at = 12;
  while (at + 8 <= b.length && b.toString("ascii", at, at + 4) !== "data") at += 8 + b.readUInt32LE(at + 4);
  const end = Math.min(b.length, at + 8 + b.readUInt32LE(at + 4));
  let sum = 0;
  let n = 0;
  for (let i = at + 8; i + 1 < end; i += 2, n++) sum += (b.readInt16LE(i) / 32767) ** 2;
  return Math.sqrt(sum / Math.max(1, n));
}

/** Installs the launcher with its installer, then this version through it; returns the dist in the cache. */
function install(step: Step, work: string): string {
  const home = join(work, "home");
  Object.assign(launcherEnv, { ORIGIN_HOME: home, ORIGIN_ENGINE_CACHE: join(work, "cache"), ORIGIN_ENGINE_DOWNLOADS: dirname(FROM!) }, PUBKEY ? { ORIGIN_ENGINE_PUBKEY: PUBKEY } : {});
  const windows = process.platform === "win32";
  let installer = INSTALLER;
  if (!installer) {
    // In a checkout: the installers as publish builds them (the container gets them through --installer).
    const build = join(HERE, "..", "..", "packages", "launcher", "build.ts");
    run(step, ["bun", build, work], work);
    installer = join(work, windows ? "install.ps1" : "install.sh");
  }
  run(step, windows ? ["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", installer] : ["sh", installer], work);
  ogCmd = windows ? ["cmd", "/c", join(home, "bin", "og.cmd")] : [join(home, "bin", "og")];
  if (!existsSync(ogCmd.at(-1)!)) step.fail(`the installer wrote no ${ogCmd.at(-1)}`);
  const installed = run(step, [...ogCmd, "engine", "install", VERSION!], work);
  if (!installed.stderr.includes(`downloading engine ${VERSION}`)) step.fail(`og engine install downloaded nothing:\n${installed.stderr}`);
  const dist = join(work, "cache", "engines", VERSION!, TRIPLE!);
  const build = (JSON.parse(readFileSync(join(dist, "origin-engine.json"), "utf8")) as { build: { version: string; target: string; triple: string; og: string } }).build;
  if (build.version !== VERSION || build.target !== PLATFORM || build.triple !== TRIPLE) step.fail(`origin-engine.json's build record: ${JSON.stringify(build)}`);
  if (sha256(join(dist, "bin", windows ? "og.exe" : "og")) !== build.og) step.fail("bin/og is not the build record's og");
  for (const f of ["overlay/project.ogt", "archetypes/runner/project.ogt", "create/bin/create.ts", "engine/dist/web-publish/origin-engine_bg.wasm", "engine/dist/web-publish-2d/origin-engine_bg.wasm", "engine/dist/web-publish/origin-engine-lanes_bg.wasm", "engine/shell", "engine/types/index.d.ts", "engine/schema/api.schema.json", `engine/native/origin-engine.${PLATFORM}.node`]) {
    if (!existsSync(join(dist, f))) step.fail(`the dist has no ${f}`);
  }
  const list = run(step, [...ogCmd, "engine", "list"], work);
  if (!list.stdout.includes(VERSION!)) step.fail(`og engine list:\n${list.stdout}`);
  return dist;
}

/** `og new <name> --engine <v>` in the work directory; returns the game. */
function scaffold(step: Step, work: string, name: string, ...args: string[]): string {
  run(step, [...ogCmd, "new", name, "--engine", VERSION!, ...args, "--no-prompt"], work);
  const game = join(work, name);
  const pinned = /^engine\s+"([^"]+)"/m.exec(readFileSync(join(game, "project.ogt"), "utf8"))?.[1];
  if (pinned !== VERSION) step.fail(`${name}/project.ogt pins engine "${pinned}", expected "${VERSION}"`);
  if (existsSync(join(game, "node_modules"))) step.fail(`${name} has a node_modules/: nothing is installed into a game`);
  if (!lstatSync(join(game, ".og", "engine")).isSymbolicLink() && process.platform !== "win32") step.fail(`${name}/.og/engine is not a link`);
  if (realpathSync(join(game, ".og", "engine")) !== realpathSync(enginePkg)) step.fail(`${name}/.og/engine points at ${realpathSync(join(game, ".og", "engine"))}`);
  return game;
}

function writeJson(path: string, value: unknown): void {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`);
}

let ogCmd: string[] = [];
let enginePkg = "";
function og(step: Step, project: string, ...rest: string[]): SpawnSyncReturns<string> {
  return run(step, [...ogCmd, ...rest], project);
}

function host(): void {
  const work = mkdtempSync(join(tmpdir(), "og-install-"));
  log(`work directory ${work} (${PLATFORM}, ${TRIPLE}, from ${FROM})`);
  try {
    // 1. The launcher, and the version through it.
    let step = new Step("1 the launcher, og engine install");
    const dist = install(step, work);
    enginePkg = join(dist, "engine");

    // 2. The scaffold: og new, from the dist's create/.
    step = new Step("2 og new");
    const demo = scaffold(step, work, "demo", "--name", "Demo");
    for (const f of ["project.ogt", "src/main.ts", "tests", "assets/looks/demo.ogt", ".gitignore", ".gitattributes"]) {
      if (!existsSync(join(demo, f))) step.fail(`the scaffold wrote no ${f}`);
    }
    if (!CORE && readFileSync(join(demo, "package.json"), "utf8").includes("@origingame/engine")) step.fail("the scaffold's package.json depends on @origingame/engine");

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
    if (sha256(shipped) !== sha256(packaged)) step.fail("the shipped publish wasm differs from the dist's engine/dist/web-publish/origin-engine_bg.wasm");

    if (!CORE) {
      // 4b. og ui check from the package (src/node/ui-run.ts): a reviewer's contrast numbers come from it, so the
      // package must carry it (0.7.8-dev.17 did not). It exits 0 whatever it found; its JSON names each
      // finding's box, the count of its measures, and every cell's frame.
      step = new Step("4b og ui check, og style check --sizes, og frame measure");
      const ui = run(step, [...ogCmd, "ui", "check", ".", "--breakpoints", "desktop,phoneLandscape", "--locales", "game", "--scales", "100", "--json"], demo, { allowFail: true });
      if (ui.status !== 0) step.fail(`og ui check exited ${ui.status}\n${ui.stderr.slice(-2000)}`);
      const checked = JSON.parse(ui.stdout.trim().split("\n").at(-1)!) as { cells: number; findings: Array<{ at?: unknown[] }>; measures: number; frames?: Array<{ viewport: number[] }> };
      if (!(checked.cells > 0) || checked.frames?.length !== checked.cells || !(checked.measures > 0)) step.fail(`og ui check --json: ${ui.stdout.slice(0, 400)}`);
      if (checked.findings.some((f) => !Array.isArray(f.at))) step.fail("og ui check --json: a finding without its places (at)");
      log(`   og ui check: ${checked.cells} cell(s), ${checked.findings.length} finding(s)`);
      // The reviewer's other two: og style check at desktop and phone size (labelled frames with their tier), and a region
      // of one of them measured by og frame measure.
      // Exit 3 is the template's dropship off faceted-low-poly's face budget (project.ogt says so): still a run.
      const style = run(step, [...ogCmd, "style", "check", ".", "--title", "0.5:screenshot", "--sizes", "desktop,phone", "--json"], demo, { allowFail: true });
      const styled = JSON.parse(style.stdout.trim().split("\n").at(-1)!) as { frames: Array<{ label: string; path: string; size: number[] }>; frameErrors?: string[] };
      if (renderer?.[1] === "none") {
        // No GPU (the container, OG_RENDERER=none): og style check has no frame to judge, and must say so
        // rather than pass on nothing; og frame measure has no frame to read.
        const said = (styled.frameErrors ?? []).filter((e) => e.includes("no renderer"));
        if (style.status !== 1 || styled.frames.length !== 0 || said.length !== 2) step.fail(`og style check without a renderer exited ${style.status}, framed ${styled.frames.length}, frameErrors ${JSON.stringify(styled.frameErrors)}`);
        log(`   og style check without a renderer: refused, ${said.length} frame errors naming it`);
      } else {
        if (style.status !== 0 && style.status !== 3) step.fail(`og style check exited ${style.status}\n${style.stderr.slice(-2000)}`);
        const labels = styled.frames.map((f) => f.label).sort();
        if (labels.join(",") !== "title@desktop,title@phoneLandscape") step.fail(`og style check --sizes framed ${labels.join(", ")}`);
        const measured = og(step, demo, "frame", "measure", styled.frames[0]!.path, "--units", "fraction", "--region", "0,0,0.5,0.5", "--json");
        const ratio = (JSON.parse(measured.stdout) as { metrics: { contrast: { ratio: number } } }).metrics.contrast.ratio;
        if (!(ratio >= 1)) step.fail(`og frame measure printed ${measured.stdout.slice(0, 200)}`);
      }

    
    }

    // 5. An archetype.
    step = new Step("5 the runner archetype");
    const runner = scaffold(step, work, "demo-runner", "--name", "Demo Runner", "--archetype", "runner");
    for (const f of [".gitignore", ".gitattributes"]) if (!existsSync(join(runner, f))) step.fail(`the archetype scaffold wrote no ${f}`);
    og(step, runner, "test", ".");

    if (!CORE) {
      // 6. The 2D starter: create/'s template-2d/, written for --profile 2d.
      step = new Step("6 the 2D starter (--profile 2d)");
      const flat = scaffold(step, work, "demo-2d", "--name", "Demo 2D", "--profile", "2d");
      for (const f of ["project.ogt", "src/main.ts", "tests", "assets/looks/demo-2d.ogt", ".gitignore", ".gitattributes"]) {
        if (!existsSync(join(flat, f))) step.fail(`the 2D scaffold wrote no ${f}`);
      }
      const manifest2d = readFileSync(join(flat, "project.ogt"), "utf8");
      if (!/^profile\s+"2d"/m.test(manifest2d)) step.fail("the 2D scaffold's project.ogt has no `profile \"2d\"` line");
      const pinned2d = /^engine\s+"([^"]+)"/m.exec(manifest2d)?.[1];
      if (pinned2d !== VERSION) step.fail(`the 2D scaffold's project.ogt pins engine "${pinned2d}", expected "${VERSION}"`);
      og(step, flat, "check", ".");
      const test2d = og(step, flat, "test", ".");
      log(`   og test: ${test2d.stdout.trim().split("\n").slice(-2).join(" | ")}`);

    
    }

    // 7. The knowledge, inside the installed engine package (knowledge/).
    step = new Step("7 knowledge: validate, mount --package, --check");
    const knPkg = join(enginePkg, "knowledge");
    if (!existsSync(join(knPkg, "knowledge.json"))) step.fail(`${enginePkg} carries no knowledge/`);
    run(step, ["bun", join(knPkg, "scripts", "validate.ts")], demo);
    // The mount layout follows origin-agent (the agent-repo split, 0.6.5): not a step for older versions.
    if (!CORE) {
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
    }

    if (!CORE) {
      // 8. A compressed sound: an MP3 (the platform's generators return MP3) cooks with the installed og and plays.
      step = new Step("8 an MP3 sound cooks and plays");
      const sound = scaffold(step, work, "demo-sound", "--name", "Demo Sound");
      rmSync(join(sound, "tests"), { recursive: true, force: true });
      mkdirSync(join(sound, "assets", "audio"), { recursive: true });
      // tone.mp3: 1 s, 44.1 kHz stereo, a 0.5-amplitude sine (og-cook's tests/fixtures/audio/tone.mp3).
      cpSync(join(HERE, "fixtures", "tone.mp3"), join(sound, "assets", "audio", "tone.mp3"));
      writeFileSync(
        join(sound, "src", "main.ts"),
        `import { og } from "@origingame/engine";

  export default og.game({
    tick: 60,
    seed: 1,
    start(world) {
      const sound = og.load.sound(world, "audio/tone.mp3");
      if (!sound.ok) throw new Error(sound.reason);
      const voice = world.spawn({ name: "tone" });
      if (!voice.ok) throw new Error(voice.reason);
      const put = world.insert(voice.value, og.AudioSource, { sound: sound.value, playing: true });
      if (!put.ok) throw new Error(put.reason);
    },
  });
  `,
      );
      const heard = join(sound, "og-out", "sound");
      og(step, sound, "run", ".", "--headless", "--commands", "0.5:get:tone.og.AudioPlayback;0.5:listen:tone.wav:0.5;0.6:quit", "--out", heard);
      const got = join(heard, "get-000030.json");
      if (!existsSync(got)) step.fail(`og run wrote no ${got}`);
      const playback = (JSON.parse(readFileSync(got, "utf8")) as { value?: { cursor: number; loopEnd: number } }).value;
      // Half a second into a 1 s sound at 44.1 kHz: the playhead near 22,050 frames, the sound 44,100 long.
      if (!playback || Math.abs(playback.cursor - 22_050) > 1_000 || Math.abs(playback.loopEnd - 44_100) > 1_300) step.fail(`tone's og.AudioPlayback at 0.5 s: ${JSON.stringify(playback)}`);
      const rms = wavRms(join(heard, "tone.wav"));
      if (rms < 0.2) step.fail(`the mixer's last 0.5 s is near silent (RMS ${rms.toFixed(3)})`);
      log(`   tone.mp3: cursor ${playback.cursor} of ${playback.loopEnd} frames at 0.5 s, mix RMS ${rms.toFixed(3)}`);

      // 9. A built-in style, read from the dist's engine/styles/ (og
      // compiles none in): a look naming fantasy cooks that pack, its three
      // densities, and no other pack. The demo already cooks what its own
      // look names (the template's look extends a preset with a pack, and
      // packs read templates/styles/shared/'s fonts and icons), so "no other"
      // is against the demo's cook without the fantasy look.
      step = new Step("9 a built-in UI style from the engine's styles/");
      if (!existsSync(join(enginePkg, "styles", "fantasy", "atlas@3x.png"))) step.fail(`${enginePkg} carries no styles/fantasy/atlas@3x.png`);
      const packs = (): Set<string> => {
        og(step, demo, "cook", ".");
        const cooked = readFileSync(join(demo, "cooked", ".cook.ogt"), "utf8");
        return new Set([...cooked.matchAll(/@styles\/([a-z0-9-]+)\//g)].map(m => m[1]!));
      };
      const before = packs();
      if (before.has("fantasy")) step.fail(`the demo cooks fantasy before a look names it: ${[...before].join(", ")}`);
      writeFileSync(join(demo, "assets", "looks", "fantasy-ui.ogt"), "type look\nversion 1\n---\nui \"fantasy\"\n");
      const after = packs();
      const cooked = readFileSync(join(demo, "cooked", ".cook.ogt"), "utf8");
      for (const f of ["style.ogt", "atlas.png", "atlas@2x.png", "atlas@3x.png"]) {
        if (!cooked.includes(`@styles/fantasy/${f}`)) step.fail(`the cook has no @styles/fantasy/${f}`);
      }
      const others = [...after].filter(p => p !== "fantasy" && !before.has(p));
      if (others.length) step.fail(`a look naming fantasy cooked other packs too: ${others.join(", ")} (the demo alone cooks ${[...before].join(", ") || "none"})`);
      log(`   fantasy cooked beside the demo's own ${[...before].join(", ") || "none"}`);
      rmSync(join(demo, "assets", "looks", "fantasy-ui.ogt"));

    
    }

    log(`PASS on ${PLATFORM}${CORE ? " (core steps)" : ""}`);
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
  // The installers as publish builds them; the channel is --from's parent, as /downloads/engine/ holds <version>/.
  const launcher = mkdtempSync(join(tmpdir(), "og-install-launcher-"));
  run(step, ["bun", join(repo, "packages", "launcher", "build.ts"), launcher], repo);
  const mounts = ["-v", `${HERE}:/install:ro`, "-v", `${launcher}:/launcher:ro`, "-v", `${FROM}:/channel/${VERSION}:ro`];
  const inner = ["bun", "/install/run.ts", "--version", VERSION!, "--host-only", "--from", `/channel/${VERSION}`, "--installer", "/launcher/install.sh", ...(CORE ? ["--core"] : []), ...(PUBKEY ? ["--pubkey", PUBKEY] : [])];
  // The container has no GPU: a sandbox without one runs og with OG_RENDERER=none (docs/platform-asks.md
  // #20), so og test runs every suite's checks and leaves the frames unchecked, rather than failing at the adapter.
  const r = spawnSync("docker", ["run", "--rm", "--platform", "linux/amd64", "-e", "OG_RENDERER=none", ...mounts, "-w", "/tmp", IMAGE, ...inner], { encoding: "utf8", stdio: ["ignore", "inherit", "inherit"], timeout: 60 * 60_000 });
  if (r.status !== 0) step.fail(`the container run exited ${r.status}`);
}

function record(where: "host" | "container"): void {
  if (!RECORD) return;
  const m = JSON.parse(readFileSync(RECORD, "utf8")) as { verified?: Record<string, unknown> };
  m.verified = { ...m.verified, [where]: { at: new Date().toISOString(), platform: where === "host" ? PLATFORM : "linux-x64 (container)", source: "archives" } };
  writeFileSync(RECORD, `${JSON.stringify(m, null, 2)}\n`);
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
