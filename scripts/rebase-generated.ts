#!/usr/bin/env bun
/** Rebase without text-merging derived API files. Source conflicts stay stopped. */
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { execFileSync, spawnSync } from "node:child_process";
import { randomUUID } from "node:crypto";

// Read og_idl::OUTPUT_DIRS: a conflicted host crate cannot build og-cli.
const source = readFileSync("crates/og-idl/src/lib.rs", "utf8");
const block = source.match(/pub const OUTPUT_DIRS: &\[&str\] = &\[([\s\S]*?)\];/)![1];
const OUTPUTS = [...block.matchAll(/"([^"]+)"/g)].map(m => m[1]);
export const git = (...args: string[]) => execFileSync("git", args, { encoding: "utf8" });
export function regenerate(): void {
  // Build the generator alone, using this checkout's normal target directory.
  const examples = "crates/og-idl/examples";
  mkdirSync(examples, { recursive: true });
  const name = "land_" + randomUUID().replaceAll("-", ""), path = join(examples, name + ".rs");
  writeFileSync(path, `
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(".");
    let api = og_idl::read(&root.join(og_idl::IDL_PATH))?;
    let mut files = og_idl::generate(&api);
    // @origingame/engine/ui's declarations, with the tsrs og check finds first.
    let tsrs = std::env::var_os("OG_TSRS")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::split_paths(&std::env::var_os("PATH")?).map(|d| d.join("tsrs")).find(|p| p.is_file()))
        .ok_or("no tsrs: run scripts/install-tools.sh or set OG_TSRS")?;
    let ui = og_idl::ui_declarations(root, &tsrs)?;
    files.extend(ui.iter().cloned());
    og_idl::write(root, &files)?;
    for drift in og_idl::check(root, &ui)? {
        if let og_idl::Drift::Unexpected(path) = drift {
            std::fs::remove_file(root.join(path))?;
        } else {
            return Err(format!("{drift}").into());
        }
    }
    Ok(())
}
`);
  try { execFileSync("cargo", ["run", "-q", "-p", "og-idl", "--example", name], { stdio: "inherit" }); }
  finally { rmSync(path); }
  git("add", "-A", "--", ...OUTPUTS);
}
export function rebase(base: string, generate: () => void = regenerate): void {
  const old = git("rev-parse", "HEAD");
  let result = spawnSync("git", ["rebase", base], { stdio: "inherit" });
  if (result.error) throw result.error;
  while (result.status !== 0) {
    const conflicts = git("diff", "--name-only", "--diff-filter=U", "-z").split("\0").filter(Boolean);
    if (!conflicts.length || !conflicts.every(path => OUTPUTS.some(root => path.startsWith(root + "/")))) throw new Error("land: non-generated conflict or rebase failure; inspect the stopped rebase");
    console.log("land: regenerating generated-only conflicts");
    generate();
    result = spawnSync("git", ["-c", "core.editor=true", "rebase", "--continue"], { stdio: "inherit" });
    if (result.error) throw result.error;
  }
  const head = git("rev-parse", "HEAD");
  if (head !== old && head !== git("rev-parse", base)) {
    // A clean text merge later in the stack may still leave stale output.
    generate();
    if (git("diff", "--cached", "--name-only")) git("-c", "core.editor=true", "commit", "--amend", "--no-edit");
  }
}
if (import.meta.main) rebase(process.argv[2]);
