// Writes the launcher's installers, install.sh and install.ps1, each
// carrying packages/launcher/og.ts whole (docs/release.md §3; release.sh
// publish runs it into the version's directory).
//
//   bun packages/launcher/build.ts <out dir>
//
// Refuses a launcher whose pinned key is not the one docs/release.md states,
// or whose text would end an installer's here-document early.

import { readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { ENGINE_PUBKEY } from "./og.ts";

const here = import.meta.dir;

/** The engine's release public key (the key line), as docs/release.md §1 states it: the line after `untrusted comment: minisign public key <id>`. */
export function pinnedKey(): string | undefined {
  const lines = readFileSync(join(here, "../../docs/release.md"), "utf8").split("\n").map((l) => l.trim());
  const at = lines.findIndex((l) => /^untrusted comment: minisign public key [0-9A-F]{16}$/.test(l));
  const key = at >= 0 ? lines[at + 1] : undefined;
  return key && /^RW[A-Za-z0-9+/]{54}$/.test(key) ? key : undefined;
}

export function installers(): { sh: string; ps1: string } {
  const launcher = readFileSync(join(here, "og.ts"), "utf8");
  if (pinnedKey() !== ENGINE_PUBKEY) throw new Error(`og.ts pins ${ENGINE_PUBKEY}; docs/release.md states ${pinnedKey()}`);
  const lines = launcher.split("\n");
  if (lines.some((l) => l === "__OG_LAUNCHER_END__")) throw new Error("og.ts holds install.sh's here-document delimiter");
  if (lines.some((l) => l.startsWith("'@"))) throw new Error("og.ts holds a line starting '@, which ends install.ps1's here-string");
  const body = launcher.endsWith("\n") ? launcher : `${launcher}\n`;
  const fill = (file: string) => readFileSync(join(here, file), "utf8").replace("__OG_LAUNCHER__\n", () => body);
  return { sh: fill("install.sh"), ps1: fill("install.ps1") };
}

if (import.meta.main) {
  const out = resolve(process.argv[2] ?? ".");
  const { sh, ps1 } = installers();
  writeFileSync(join(out, "install.sh"), sh, { mode: 0o755 });
  writeFileSync(join(out, "install.ps1"), ps1);
  process.stdout.write(`launcher: ${join(out, "install.sh")}, ${join(out, "install.ps1")}\n`);
}
