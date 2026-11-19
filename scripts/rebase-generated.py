#!/usr/bin/env python3
"""Rebase without text-merging derived API files. Source conflicts stay stopped."""
import ast
import difflib
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
    'crates/og-host-node/src/generated', 'crates/og-host-c/src/generated',
    'packages/knowledge/wiki',
)
THEME = 'crates/og-ui/src/theme.rs'
# docs/css-coverage.md is wholly generated from scripts/css-coverage.py's
# proof lists. It was once a region of docs/ui.md between MARKERS; a branch
# from before the move still edits that region (transition_text).
CHECKLIST = 'docs/css-coverage.md'
COVERAGE = 'scripts/css-coverage.py'
DESIGN = 'docs/ui.md'
MARKERS = ('<!-- css-coverage:start -->', '<!-- css-coverage:end -->')
ENTRY_CALLS = {'proof', 'not_applicable', 'part_proof'}


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


def region(text):
    """text split around its old checklist region, markers included, or None."""
    start, end = text.find(MARKERS[0]), text.find(MARKERS[1])
    if start < 0 or end < start:
        return None
    end += len(MARKERS[1])
    return text[:start], text[start:end], text[end:]


def transition_text(base, ours, theirs):
    """docs/ui.md for a commit from before the checklist left it: ours has
    no region, base and theirs do. Theirs' region edits are dropped (the
    regenerated docs/css-coverage.md carries them); everything else merges
    as git would. None when the region is not there to drop or the written
    text conflicts."""
    if region(ours) is not None or region(base) is None or region(theirs) is None:
        return None
    before, _, after = region(theirs)
    with tempfile.TemporaryDirectory() as scratch:
        paths = []
        for name, text in (('ours', ours), ('base', base), ('theirs', before + region(base)[1] + after)):
            paths.append(Path(scratch) / name)
            paths[-1].write_text(text)
        merged = subprocess.run(['git', 'merge-file', '-p', *map(str, paths)], stdout=subprocess.PIPE)
    return merged.stdout.decode() if merged.returncode == 0 else None


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


def stages(path):
    """The merge base's, ours and theirs texts of a conflicted path, or None."""
    shown = [subprocess.run(['git', 'show', f':{n}:{path}'], stdout=subprocess.PIPE,
                            stderr=subprocess.DEVNULL) for n in (1, 2, 3)]
    return None if any(r.returncode for r in shown) else [r.stdout.decode() for r in shown]


def checklist():
    if subprocess.run([sys.executable, COVERAGE], stdout=subprocess.DEVNULL).returncode:
        sys.exit('land: scripts/css-coverage.py failed over the merged proofs; inspect the stopped rebase')
    git('add', '--', *(p for p in (CHECKLIST, COVERAGE, DESIGN) if Path(p).exists()))


def rebase(base):
    old = git('rev-parse', 'HEAD')
    result = subprocess.run(['git', 'rebase', base])
    listed = False
    while result.returncode:
        conflicts = [os.fsdecode(p) for p in
                     git('diff', '--name-only', '--diff-filter=U', '-z').split(b'\0') if p]
        if not conflicts and listed and not git('diff', '--cached', '--name-only') and \
                Path(os.fsdecode(git('rev-parse', '--git-path', 'rebase-merge').strip())).is_dir():
            # A commit that only edited the old region is empty once that is dropped.
            print('land: dropping a commit the regenerated CSS checklist left empty', flush=True)
            result = subprocess.run(['git', 'rebase', '--skip'])
            continue
        written = {}
        for path, resolve in ((DESIGN, transition_text), (COVERAGE, coverage_text)):
            if path in conflicts:
                texts = stages(path)
                written[path] = texts and resolve(*texts)
        if CHECKLIST in conflicts:
            written[CHECKLIST] = ''  # regenerated whole below
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
        result = subprocess.run(['git', '-c', 'core.editor=true', 'rebase', '--continue'])
    if git('rev-parse', 'HEAD') not in (old, git('rev-parse', base)):
        # Later commits can text-merge cleanly over regenerated output while
        # leaving it stale. Always regenerate the completed source tree too.
        regenerate()
        if listed or git('diff', '--name-only', base, 'HEAD', '--', COVERAGE).strip():
            checklist()
        if git('diff', '--cached', '--name-only'):
            git('-c', 'core.editor=true', 'commit', '--amend', '--no-edit')


if __name__ == '__main__':
    rebase(sys.argv[1])
