#!/usr/bin/env python3
"""Rebase without text-merging derived API files. Source conflicts stay stopped."""
import ast
import difflib
import os
from pathlib import Path
import subprocess
import sys
import re
import tempfile


# Exactly og_idl::OUTPUT_DIRS, not arbitrary .d.ts or generated-looking paths.
OUTPUTS = (
    'packages/engine/types', 'packages/engine/schema',
    'packages/engine/src/generated', 'crates/og-api/src/generated',
    'crates/og-records/src/generated', 'crates/og-host-web/src/generated',
    'crates/og-host-node/src/generated', 'packages/knowledge/wiki',
)
THEME = 'crates/og-ui/src/theme.rs'
# docs/ui.md's CSS checklist is generated between these markers from the
# proof lists in scripts/css-coverage.py; the rest of the file is written.
CHECKLIST = 'docs/ui.md'
COVERAGE = 'scripts/css-coverage.py'
MARKERS = ('<!-- css-coverage:start -->', '<!-- css-coverage:end -->')
ENTRY_CALLS = {'proof', 'not_applicable', 'part_proof'}
# Conflict blocks as git writes them with merge.conflictStyle=merge.
BLOCK = re.compile(r'^<<<<<<< [^\n]*\n(.*?)^=======\n(.*?)^>>>>>>> [^\n]*\n', re.S | re.M)


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


def checklist_text(text):
    """docs/ui.md with its conflicts taken from one side, or None when a
    conflict reaches outside the generated checklist (the written part)."""
    start, end = (text.find(marker) for marker in MARKERS)
    blocks = list(BLOCK.finditer(text))
    if start < 0 or end < start or not blocks or text.count('<<<<<<< ') != len(blocks):
        return None
    if not all(text.index('\n', start) < b.start() and b.end() <= end for b in blocks):
        return None
    return BLOCK.sub(lambda b: b.group(1), text)


def entries(side):
    """True when a conflict side holds only whole checklist entries: proof
    calls, or MISSING's "feature": "reason" items (comments and blanks too)."""
    try:
        body = ast.parse(side).body
        if all(isinstance(n, ast.Expr) and isinstance(n.value, ast.Call) and
               isinstance(n.value.func, ast.Name) and n.value.func.id in ENTRY_CALLS for n in body):
            return True
    except SyntaxError:
        pass
    try:
        table = ast.parse('{\n' + side + '\n}', mode='eval').body
    except SyntaxError:
        return False
    return isinstance(table, ast.Dict) and all(
        isinstance(n, ast.Constant) and isinstance(n.value, str) for n in table.keys + table.values)


def coverage_text(base, ours, theirs):
    """scripts/css-coverage.py merged line by line from base, keeping both
    sides where both added entries at one place (ours first), or None when
    the sides' edits overlap otherwise, an addition there is not whole
    entries, or the result does not parse or lists a MISSING feature twice.
    The generator itself refuses a feature proved twice or proved and missing."""
    base, ours, theirs = (t.splitlines(keepends=True) for t in (base, ours, theirs))
    edits = []  # (start, end, lines, side) over base's lines
    for side, text in enumerate((ours, theirs)):
        matcher = difflib.SequenceMatcher(None, base, text, autojunk=False)
        edits += [(i1, i2, text[j1:j2], side) for op, i1, i2, j1, j2 in matcher.get_opcodes() if op != 'equal']
    edits.sort(key=lambda e: (e[0], e[1], e[3]))
    merged, at, k = [], 0, 0
    while k < len(edits):
        start, end, lines, _ = edits[k]
        group = [edits[k]]
        while k + 1 < len(edits) and (edits[k + 1][0] < end or edits[k + 1][0] == start == end == edits[k + 1][1]):
            k += 1
            group.append(edits[k])
            end = max(end, edits[k][1])
        k += 1
        if len(group) == 2 and group[0][:3] == group[1][:3]:
            group = group[:1]
        elif len(group) > 1:
            if end != start or not all(entries(''.join(e[2])) for e in group):
                return None
            lines = [line for e in group for line in e[2]]
        merged += base[at:start] + lines
        at = end
    merged = ''.join(merged + base[at:])
    try:
        tree = ast.parse(merged)
    except SyntaxError:
        return None
    for n in tree.body:
        if isinstance(n, ast.Assign) and any(isinstance(t, ast.Name) and t.id == 'MISSING' for t in n.targets):
            keys = [k.value for k in n.value.keys]
            if len(keys) != len(set(keys)):
                return None
    return merged


def checklist():
    if subprocess.run([sys.executable, COVERAGE], stdout=subprocess.DEVNULL).returncode:
        sys.exit('land: scripts/css-coverage.py failed over the merged proofs; inspect the stopped rebase')
    git('add', '--', CHECKLIST, COVERAGE)


def rebase(base):
    old = git('rev-parse', 'HEAD')
    style = ['-c', 'merge.conflictStyle=merge']
    result = subprocess.run(['git', *style, 'rebase', base])
    listed = False
    while result.returncode:
        conflicts = [os.fsdecode(p) for p in
                     git('diff', '--name-only', '--diff-filter=U', '-z').split(b'\0') if p]
        written = {}
        if CHECKLIST in conflicts:
            written[CHECKLIST] = checklist_text(Path(CHECKLIST).read_text())
        if COVERAGE in conflicts:
            stages = [subprocess.run(['git', 'show', f':{n}:{COVERAGE}'], stdout=subprocess.PIPE,
                                     stderr=subprocess.DEVNULL) for n in (1, 2, 3)]
            written[COVERAGE] = None if any(r.returncode for r in stages) else \
                coverage_text(*(r.stdout.decode() for r in stages))
        if not conflicts or None in written.values() or \
                not all(generated(p) or p in written for p in conflicts):
            sys.exit('land: non-generated conflict or rebase failure; inspect the stopped rebase')
        if any(map(generated, conflicts)):
            print('land: regenerating generated-only conflicts', flush=True)
            regenerate()
        if written:
            print('land: regenerating the CSS checklist over its conflicts', flush=True)
            for path, text in written.items():
                Path(path).write_text(text)
            checklist()
            listed = True
        result = subprocess.run(['git', *style, '-c', 'core.editor=true', 'rebase', '--continue'])
    if git('rev-parse', 'HEAD') not in (old, git('rev-parse', base)):
        # Later commits can text-merge cleanly over regenerated output while
        # leaving it stale. Always regenerate the completed source tree too.
        regenerate()
        if listed:
            checklist()
        if git('diff', '--cached', '--name-only'):
            git('-c', 'core.editor=true', 'commit', '--amend', '--no-edit')


if __name__ == '__main__':
    rebase(sys.argv[1])
