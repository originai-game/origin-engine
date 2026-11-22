#!/usr/bin/env python3
"""Rebase without text-merging derived API files. Source conflicts stay stopped."""
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile


def output_dirs():
    """og_idl::OUTPUT_DIRS, read from crates/og-idl/src/lib.rs: the one list
    of generated directories, which a rebase cannot ask og-idl for (a
    conflicted host crate cannot build og-cli, so nothing here runs cargo)."""
    source = Path('crates/og-idl/src/lib.rs').read_text()
    block = re.search(r'pub const OUTPUT_DIRS: &\[&str\] = &\[(.*?)\];', source, re.S).group(1)
    return tuple(re.findall(r'"([^"]+)"', block))


OUTPUTS = output_dirs()


def git(*args, check=True):
    return subprocess.run(['git', *args], check=check, stdout=subprocess.PIPE).stdout


def generated(path):
    return any(path.startswith(root + '/') for root in OUTPUTS)


def regenerate():
    # Build only the generator: conflicted/stale host Rust cannot compile og-cli.
    # A temporary example works even at commits predating this script, using
    # that revision's library and this checkout's normal Cargo target directory.
    examples = Path('crates/og-idl/examples')
    examples.mkdir(exist_ok=True)
    with tempfile.NamedTemporaryFile(mode='w', prefix='land_', suffix='.rs', dir=examples) as source:
        source.write('''
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
''')
        source.flush()
        subprocess.run(['cargo', 'run', '-q', '-p', 'og-idl', '--example', Path(source.name).stem], check=True)
    git('add', '-A', '--', *OUTPUTS)


def rebase(base):
    old = git('rev-parse', 'HEAD')
    result = subprocess.run(['git', 'rebase', base])
    while result.returncode:
        conflicts = [os.fsdecode(p) for p in
                     git('diff', '--name-only', '--diff-filter=U', '-z').split(b'\0') if p]
        if not conflicts or not all(map(generated, conflicts)):
            sys.exit('land: non-generated conflict or rebase failure; inspect the stopped rebase')
        print('land: regenerating generated-only conflicts', flush=True)
        regenerate()
        result = subprocess.run(['git', '-c', 'core.editor=true', 'rebase', '--continue'])
    if git('rev-parse', 'HEAD') not in (old, git('rev-parse', base)):
        # Later commits can text-merge cleanly over regenerated output while
        # leaving it stale. Always regenerate the completed source tree too.
        regenerate()
        if git('diff', '--cached', '--name-only'):
            git('-c', 'core.editor=true', 'commit', '--amend', '--no-edit')

if __name__ == '__main__':
    rebase(sys.argv[1])
