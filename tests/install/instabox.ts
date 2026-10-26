// The install test inside an InstaBox machine (docs/release.md §3 and §4
// stage D): the development sandbox the platform's threads run on
// (dashboard.instabox.sh), checked the way an author's machine and the
// linux-x64 container are, from the registry or from the release tarballs.
//
//   bun tests/install/instabox.ts --version <v> (--registry | --from <dir>) [--manifest <manifest.json>]
//                                 [--template agent-browser] [--machine <id>] [--keep]
//
// It creates a machine from the template (agent-browser, the one the free
// plan creates from; 4 vCPU, 8 GiB, a 20 GiB /data: the account's one
// spec), installs bun 1.4.4 beside the machine's node and
// git as tests/install/Dockerfile does and nothing of the engine, uploads
// run.ts (and, from the tarballs, the four packages in 8 MiB parts: a file
// write is 10 MiB at most), runs `run.ts --host-only` there in the
// background (a foreground command is capped at ten minutes and its output
// is not streamed), follows its log, and deletes the machine unless --keep
// (a failed run keeps it to look at; --machine reuses one, a kept one for
// instance, instead of creating it). With
// --manifest the pass is recorded as verified.instabox. The key is
// INSTABOX_API_KEY in the environment, read by the SDK; it never reaches
// the machine (commands travel through the API), and the machine takes no
// Vault secrets (vault: false), so nothing of it is worth keeping.

import { createHash } from "node:crypto";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";
import { CommandExitError, Sandbox } from "@insforge/instabox-sdk";

const args = process.argv.slice(2);
const opt = (name: string): string | undefined => {
  const i = args.indexOf(name);
  return i >= 0 ? args[i + 1] : undefined;
};
const flag = (name: string) => args.includes(name);
const VERSION = opt("--version");
const FROM = opt("--from") ? resolve(opt("--from")!) : undefined;
const REGISTRY = flag("--registry");
const MANIFEST = opt("--manifest") ? resolve(opt("--manifest")!) : undefined;
const TEMPLATE = opt("--template") ?? "agent-browser"; // the one template the free plan creates from; a paid plan may name universal
const MACHINE = opt("--machine");
const KEEP = flag("--keep");
const BUN_VERSION = "1.4.4"; // tests/install/Dockerfile's, docs/ci.md §Toolchain
const HERE = dirname(new URL(import.meta.url).pathname);
const PART = 8 << 20;
const RUN_TIMEOUT_MS = 30 * 60_000;

if (!VERSION || (!FROM && !REGISTRY)) {
  process.stderr.write("usage: bun tests/install/instabox.ts --version <v> (--registry | --from <dir>) [--manifest <file>] [--template <id>] [--machine <id>] [--keep]\n");
  process.exit(2);
}
if (!process.env.INSTABOX_API_KEY) {
  process.stderr.write("instabox: no INSTABOX_API_KEY in the environment (an API key from dashboard.instabox.sh, scopes machines:write and machines:delete)\n");
  process.exit(2);
}

const t0 = Date.now();
const log = (s: string) => process.stdout.write(`[instabox ${((Date.now() - t0) / 1000).toFixed(1)}s] ${s}\n`);
const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
const sha256 = (data: Uint8Array) => createHash("sha256").update(data).digest("hex");

/** A command in the machine with bun on the PATH; throws with its output on a non-zero exit. */
async function sh(sbx: Sandbox, cmd: string, opts: { timeoutMs?: number; cwd?: string } = {}): Promise<string> {
  try {
    const r = await sbx.commands.run(`export PATH="$HOME/.bun/bin:$PATH"; ${cmd}`, { timeoutMs: opts.timeoutMs ?? 300_000, cwd: opts.cwd });
    return r.stdout;
  } catch (e) {
    if (e instanceof CommandExitError) throw new Error(`\`${cmd}\` exited ${e.exitCode}${e.timedOut ? " (timed out)" : ""}\n--- stdout\n${e.stdout.slice(-3000)}\n--- stderr\n${e.stderr.slice(-3000)}`);
    throw e;
  }
}

/** Writes a local file into the machine in parts no larger than the API's 10 MiB, and checks the bytes. */
async function upload(sbx: Sandbox, local: string, remote: string): Promise<void> {
  const data = readFileSync(local);
  const parts = Math.max(1, Math.ceil(data.length / PART));
  if (parts === 1) {
    await sbx.files.write(remote, data);
  } else {
    const names: string[] = [];
    for (let i = 0; i < parts; i++) {
      const name = `${remote}.part${i}`;
      await sbx.files.write(name, data.subarray(i * PART, (i + 1) * PART));
      names.push(name);
    }
    await sh(sbx, `cat ${names.join(" ")} > ${remote} && rm ${names.join(" ")}`);
  }
  const got = (await sh(sbx, `sha256sum ${remote}`)).split(/\s+/)[0];
  const want = sha256(data);
  if (got !== want) throw new Error(`${remote}: sha256 ${got} after upload, ${want} locally`);
  log(`   ${basename(local)} (${data.length} bytes, ${parts} part${parts === 1 ? "" : "s"})`);
}

async function main(): Promise<void> {
  const source = REGISTRY ? "registry" : "tarballs";
  let sbx: Sandbox;
  if (MACHINE) {
    log(`connecting to ${MACHINE}`);
    sbx = await Sandbox.connect(MACHINE);
  } else {
    log(`creating a machine from ${TEMPLATE}`);
    sbx = await Sandbox.create(TEMPLATE, {
      name: `og-install-${VERSION!.replace(/[^a-z0-9]/gi, "-")}-${Math.random().toString(36).slice(2, 8)}`,
      metadata: { purpose: "og-install-test", version: VERSION!, source },
      vault: false,
    });
  }
  const info = await sbx.getInfo();
  log(`machine ${sbx.sandboxId} (${info.name}, ${info.templateId}, ${info.state})`);
  let passed = false;
  try {
    // The machine as the Dockerfile has it: node, git, bun 1.4.4, nothing of the engine.
    const facts = await sh(sbx, "uname -sm; nproc; free -m | awk '/Mem:/ {print $2 \" MiB\"}'; node --version; git --version; (bun --version 2>/dev/null || echo 'bun: missing'); (ffmpeg -version 2>/dev/null | head -1 || echo 'ffmpeg: missing'); (vulkaninfo --summary 2>/dev/null | grep -m1 deviceName || echo 'renderer: none')");
    log(`machine: ${facts.trim().split("\n").join(" | ")}`);
    if (!facts.includes(`${BUN_VERSION}`)) {
      log(`installing bun ${BUN_VERSION}`);
      await sh(sbx, `curl -fsSL https://bun.sh/install | bash -s "bun-v${BUN_VERSION}" >/dev/null && ln -sf "$HOME/.bun/bin/bun" "$HOME/.bun/bin/bunx" && bun --version`);
    }
    for (const env of ["OG_ENGINE_PACKAGE", "OG_NATIVE_ADDON", "ORIGIN_OG_BIN"]) {
      const v = (await sh(sbx, `printenv ${env} || true`)).trim();
      if (v) throw new Error(`the machine has ${env}=${v}; the install test wants none`);
    }

    // run.ts and, from the tarballs, the packages, under HOME (/data/home on
    // the persistent disk; /data itself is not the guest user's).
    log("uploading");
    const REMOTE = `${(await sh(sbx, "echo $HOME")).trim()}/og-install`;
    await sh(sbx, `rm -rf ${REMOTE} && mkdir -p ${REMOTE}/tgz`);
    await upload(sbx, join(HERE, "run.ts"), `${REMOTE}/run.ts`);
    if (FROM) {
      for (const name of ["engine", "engine-linux-x64", "create-game"]) {
        const f = join(FROM, `origingame-${name}-${VERSION}.tgz`);
        if (!existsSync(f)) throw new Error(`no ${f}`);
        await upload(sbx, f, `${REMOTE}/tgz/${basename(f)}`);
      }
    }

    // The test, in the background, its log followed from here.
    const inner = ["bun", `${REMOTE}/run.ts`, "--version", VERSION!, "--host-only", ...(REGISTRY ? ["--registry"] : ["--from", `${REMOTE}/tgz`])].join(" ");
    log(`$ ${inner}`);
    await sbx.commands.run(`export PATH="$HOME/.bun/bin:$PATH"; cd /tmp && (${inner} > ${REMOTE}/log 2>&1; echo $? > ${REMOTE}/exit)`, { background: true });
    let offset = 0;
    let exit: string | undefined;
    const deadline = Date.now() + RUN_TIMEOUT_MS;
    while (exit === undefined) {
      if (Date.now() > deadline) throw new Error(`the run did not end in ${RUN_TIMEOUT_MS / 60_000} minutes; the machine is kept: ${sbx.sandboxId}`);
      await sleep(5_000);
      const r = await sbx.commands.run(`tail -c +${offset + 1} ${REMOTE}/log 2>/dev/null; echo; echo "--exit:$(cat ${REMOTE}/exit 2>/dev/null)"`);
      const m = /\n--exit:(\d*)\n?$/.exec(r.stdout);
      const chunk = m ? r.stdout.slice(0, m.index) : r.stdout;
      if (chunk.length) {
        offset += Buffer.byteLength(chunk);
        process.stdout.write(chunk.split("\n").filter((l) => l.length).map((l) => `   ${l}\n`).join(""));
      }
      if (m?.[1]) exit = m[1];
    }
    if (exit !== "0") throw new Error(`run.ts exited ${exit} in the machine`);
    passed = true;
    log(`PASS in ${sbx.sandboxId} (${TEMPLATE}, ${source})`);
    if (MANIFEST) {
      const m = JSON.parse(readFileSync(MANIFEST, "utf8")) as { verified?: Record<string, unknown> };
      m.verified = { ...m.verified, instabox: { at: new Date().toISOString(), platform: `linux-x64 (instabox ${TEMPLATE})`, machine: sbx.sandboxId, source } };
      writeFileSync(MANIFEST, `${JSON.stringify(m, null, 2)}\n`);
    }
  } finally {
    if (KEEP || !passed) {
      log(`the machine is kept: ${sbx.sandboxId}${passed ? "" : " (the run failed; look, then Sandbox.kill it or delete it in the dashboard)"}`);
    } else {
      await sbx.kill();
      log(`machine ${sbx.sandboxId} deleted`);
    }
  }
}

try {
  await main();
  log("install test PASSED on InstaBox");
} catch (e) {
  process.stderr.write(`install test FAILED on InstaBox: ${e instanceof Error ? e.message : String(e)}\n`);
  process.exit(1);
}
