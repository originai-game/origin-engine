// A game as a desktop store package (Steam: macOS, Windows, Linux): its
// TypeScript as Hermes bytecode, the desktop shell (crates/og-host-c/hermes)
// and the engine's C ABI host beside it, and the cooked project.
//
//   bun src/hermes/package.ts <project> <out-dir> --hermes <dir> [--hermes-build <dir>] [--host <lib>] [--name <name>] [--size WxH]
//     [--provider <module.ts> [--provider-lib <lib>]]
//
// --hermes: a Hermes checkout; --hermes-build its build (hermesc, the
// hermesvm library, jsi; default <checkout>/build). --host: og-host-c's library (default: the checkout's
// target/release). --provider: a store's provider module, bundled with the
// game; --provider-lib its native library (platform.ts). Without them the
// game's og.platform is the local document. The project must be cooked. Writes, by platform:
//
//   linux    <out>/<name>/<name>, its libraries, game.hbc, project.ogt, cooked/, shell.env
//   windows  <out>/<name>/<name>.exe, its DLLs, and the same files
//   macos    <out>/<name>.app: Contents/MacOS (the shell, its libraries),
//            Contents/Resources (the game), ad-hoc signed
//
// The shell finds game.hbc beside it (or in ../Resources) and runs it.

import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { engineFile } from "../node/build.ts";
import { readProject } from "../node/project.ts";

declare const Bun: {
  build(o: {
    entrypoints: string[];
    target: string;
    format: string;
    external?: string[];
    plugins?: unknown[];
  }): Promise<{ success: boolean; logs: unknown[]; outputs: Array<{ text(): Promise<string> }> }>;
  spawnSync(cmd: string[], o: { stdout: "inherit"; stderr: "inherit" }): { exitCode: number };
};

const here = dirname(fileURLToPath(import.meta.url));

/** Bundles `game` (a project's src/main.ts or an entry) with `driver`, whose `main(game)` runs it: an ES module for node, one classic script for Hermes. */
export async function bundle(game: string, driver: string, target: "node" | "hermes", out: string, provider?: string): Promise<void> {
  const dir = mkdtempSync(join(tmpdir(), "og-bundle-"));
  const entry = join(dir, "entry.ts");
  const imports = [`import game from ${JSON.stringify(resolve(game))};`, `import { main } from ${JSON.stringify(resolve(driver))};`];
  if (provider) imports.push(`import provider from ${JSON.stringify(resolve(provider))};`);
  writeFileSync(entry, `${imports.join("\n")}\nmain(game${provider ? ", provider" : ""});\n`);
  const r = await Bun.build({
    entrypoints: [entry],
    target: target === "node" ? "node" : "browser",
    format: target === "node" ? "esm" : "iife",
    // The addon's loader is node's; Hermes has none.
    external: target === "node" ? ["*.node"] : [],
    plugins: [{
      name: "origin-engine",
      setup(build: { onResolve(o: { filter: RegExp }, f: (a: { path: string }) => { path: string }): void }) {
        build.onResolve({ filter: /^@origingame\/engine(\/.*)?$/ }, (a) => {
          const file = engineFile(a.path);
          if (!file) throw new Error(`${a.path} is not an export of @origingame/engine`);
          return { path: file };
        });
      },
    }],
  });
  rmSync(dir, { recursive: true, force: true });
  if (!r.success) throw new Error(r.logs.map(String).join("\n"));
  writeFileSync(out, await r.outputs[0]!.text());
}

/** The first file under `dir` (depth first) whose name is one of `names`. */
function find(dir: string, names: readonly string[]): string | undefined {
  for (const e of readdirSync(dir, { withFileTypes: true })) {
    const p = join(dir, e.name);
    if (e.isFile() && names.includes(e.name)) return p;
    if (e.isDirectory() && !e.isSymbolicLink()) {
      const found = find(p, names);
      if (found) return found;
    }
  }
  return undefined;
}

function run(cmd: string[]): void {
  const r = Bun.spawnSync(cmd, { stdout: "inherit", stderr: "inherit" });
  if (r.exitCode !== 0) throw new Error(`${cmd[0]} failed (${r.exitCode})`);
}

export interface PackageOptions {
  readonly project: string;
  readonly out: string;
  readonly hermes: string;
  readonly hermesBuild?: string;
  readonly host?: string;
  readonly name?: string;
  readonly size?: string;
  /** A store's provider module (default export: hermes/platform.ts's ShellProviderFactory), bundled with the game. */
  readonly provider?: string;
  /** Its native library (exports `og_provider_call`), placed beside the shell. */
  readonly providerLib?: string;
}

/** Builds the package; answers its path. */
export async function packageGame(o: PackageOptions): Promise<string> {
  const os = process.platform;
  const exe = os === "win32" ? ".exe" : "";
  const project = resolve(o.project);
  if (!existsSync(join(project, "cooked"))) throw new Error(`${project} has no prepared assets: run og build to prepare them automatically`);
  const name = o.name ?? basename(project);
  const repo = resolve(here, "../../../..");
  const hermes = resolve(o.hermes);
  const build = resolve(o.hermesBuild ?? join(hermes, "build"));
  const lib = { linux: "libog_host_c.so", darwin: "libog_host_c.dylib", win32: "og_host_c.dll" }[os as "linux"] ?? "libog_host_c.so";
  const host = resolve(o.host ?? join(repo, "target/release", lib));
  const vm = find(build, ["libhermesvm.so", "libhermesvm.dylib", "hermesvm.dll"]);
  const hermesc = find(build, [`hermesc${exe}`]);
  const jsi = find(build, ["libjsi.a", "jsi.lib"]);
  for (const [what, p] of [["og-host-c", host], ["hermesvm", vm], ["hermesc", hermesc], ["jsi", jsi]] as const)
    if (!p || !existsSync(p)) throw new Error(`no ${what} (${p ?? `under ${build}`})`);
  // The shell detaches column buffers through an interface the checkout gains from our patch.
  if (!readFileSync(join(hermes, "API/hermes/hermes.h"), "utf8").includes("IHermesArrayBufferDetach"))
    throw new Error(`${hermes} lacks IHermesArrayBufferDetach: apply third_party/hermes/detach-array-buffer.patch and rebuild Hermes`);

  // The layout: where the shell and its libraries go, and where the game.
  const out = resolve(o.out);
  const root = os === "darwin" ? join(out, `${name}.app`) : join(out, name);
  rmSync(root, { recursive: true, force: true });
  const bin = os === "darwin" ? join(root, "Contents/MacOS") : root;
  const data = os === "darwin" ? join(root, "Contents/Resources") : root;
  mkdirSync(bin, { recursive: true });
  mkdirSync(data, { recursive: true });

  // The game: bytecode with block scoping (without it Hermes gives a loop's
  // let and const one binding for the whole function).
  const work = mkdtempSync(join(tmpdir(), "og-package-"));
  const js = join(work, "game.js");
  await bundle(join(project, "src/main.ts"), join(here, "shell.ts"), "hermes", js, o.provider);
  run([hermesc!, "-O", "-Xes6-block-scoping", "-emit-binary", "-out", join(data, "game.hbc"), js]);
  rmSync(work, { recursive: true, force: true });
  const checked = readProject(project);
  if (!checked) throw new Error(`${project}: no og-project.json project`);
  writeFileSync(join(data, "project-startup.json"), JSON.stringify(checked.startup));
  cpSync(join(project, "cooked"), join(data, "cooked"), { recursive: true });
  const providerLib = o.providerLib ? resolve(o.providerLib) : undefined;
  if (providerLib && !o.provider) throw new Error("--provider-lib needs --provider, the module that calls it");
  if (providerLib) cpSync(providerLib, join(bin, basename(providerLib)));
  writeFileSync(join(data, "shell.env"), `OG_TITLE=${name}\nOG_SIZE=${o.size ?? "1280x720"}\n${providerLib ? `OG_PROVIDER_LIB=${basename(providerLib)}\n` : ""}`);

  // The shell, linked to the libraries beside it.
  cpSync(vm!, join(bin, basename(vm!)));
  cpSync(host, join(bin, basename(host)));
  // Cargo names a dylib by its build path; the package loads it beside the shell.
  if (os === "darwin") run(["install_name_tool", "-id", `@rpath/${basename(host)}`, join(bin, basename(host))]);
  const shell = join(repo, "crates/og-host-c");
  const includes = [join(hermes, "API"), join(hermes, "API/jsi"), join(hermes, "public"), join(build, "include"), join(shell, "include")].map((d) => `-I${d}`);
  const target = join(bin, `${name}${exe}`);
  if (os === "win32") {
    // MSVC's ABI: link the import libraries; the DLLs sit beside the .exe.
    // clang++ defaults to the static CRT (/MT); jsi.lib, hermesvm and Rust use the DLL one (/MD).
    const vmLib = find(build, ["hermesvm.lib"]);
    run(["clang++", "-std=c++17", "-O2", "-fms-runtime-lib=dll", "-D_CRT_SECURE_NO_WARNINGS", ...includes, join(shell, "hermes/shell.cpp"), "-o", target, vmLib!, jsi!, `${host}.lib`]);
  } else {
    const rpath = os === "darwin" ? "@executable_path" : "$ORIGIN";
    run(["clang++", "-std=c++17", "-O2", ...includes, join(shell, "hermes/shell.cpp"), "-o", target, `-L${bin}`, "-lhermesvm", jsi!, "-log_host_c", `-Wl,-rpath,${rpath}`]);
  }
  if (os === "darwin") {
    writeFileSync(
      join(root, "Contents/Info.plist"),
      `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>${name}</string>
<key>CFBundleIdentifier</key><string>ai.origingame.${name.replace(/[^A-Za-z0-9.-]/g, "-")}</string>
<key>CFBundleName</key><string>${name}</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>1.0</string>
<key>NSHighResolutionCapable</key><true/>
<key>LSMinimumSystemVersion</key><string>13.0</string>
</dict></plist>
`,
    );
    run(["codesign", "--force", "--deep", "--sign", "-", root]);
  }
  return root;
}

if (import.meta.main) {
  const args = process.argv.slice(2);
  const flag = (f: string) => {
    const i = args.indexOf(f);
    return i < 0 ? undefined : args.splice(i, 2)[1];
  };
  const hermes = flag("--hermes");
  const hermesBuild = flag("--hermes-build");
  const host = flag("--host");
  const name = flag("--name");
  const size = flag("--size");
  const provider = flag("--provider");
  const providerLib = flag("--provider-lib");
  const [project, out] = args;
  if (!project || !out || !hermes || !existsSync(project) || !statSync(project).isDirectory()) {
    process.stderr.write("usage: bun src/hermes/package.ts <project> <out-dir> --hermes <dir> [--hermes-build <dir>] [--host <lib>] [--name <name>] [--size WxH] [--provider <module.ts> [--provider-lib <lib>]]\n");
    process.exit(2);
  }
  const made = await packageGame({ project, out, hermes, ...(hermesBuild ? { hermesBuild } : {}), ...(host ? { host } : {}), ...(name ? { name } : {}), ...(size ? { size } : {}), ...(provider ? { provider } : {}), ...(providerLib ? { providerLib } : {}) });
  process.stdout.write(`${made}\n`);
}
