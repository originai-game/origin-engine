#!/usr/bin/env python3
"""Rebase without text-merging derived API files. Source conflicts stay stopped."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile


# Exactly og_idl::OUTPUT_DIRS, not arbitrary .d.ts or generated-looking paths.
OUTPUTS = (
    'packages/engine/types', 'packages/engine/schema',
    'packages/engine/src/generated', 'crates/og-api/src/generated',
    'crates/og-records/src/generated', 'crates/og-host-web/src/generated',
    'crates/og-host-node/src/generated', 'packages/knowledge/wiki',
)
THEME = 'crates/og-ui/src/theme.rs'


def git(*args, check=True):
    return subprocess.run(['git', *args], check=check, stdout=subprocess.PIPE).stdout


def generated(path):
    return path == THEME or any(path.startswith(root + '/') for root in OUTPUTS)


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
    let files = og_idl::generate(&api);
    og_idl::write(root, &files)?;
    for drift in og_idl::check(root)? {
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
    if THEME.encode() in git('diff', '--name-only', '--diff-filter=U', '-z').split(b'\0'):
        # Bootstrap the theme generator's own crate without conflict markers.
        # This is not staged as resolution; generation must succeed first.
        git('checkout', '--ours', '--', THEME)
    subprocess.run(['bash', 'crates/og-ui/scripts/gen-theme.sh'], check=True)
    git('add', '-A', '--', *OUTPUTS, THEME)


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
