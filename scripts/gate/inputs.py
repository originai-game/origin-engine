"""Cargo dependency closures and conservative content-snapshot selection."""
from functools import lru_cache
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path.cwd()
LOGS = Path(os.environ['GATE_LOGS'])
GREEN = Path(os.environ['OG_TARGET_DIR']) / 'og-gate' / 'green'
TREE = os.environ.get('OG_GATE_TREE', '')


@lru_cache(maxsize=1)
def metadata():
    return json.loads((LOGS / 'cargo.json').read_text())


def crate_inputs(names):
    meta = metadata()
    packages = {p['id']: p for p in meta['packages']}
    nodes = {n['id']: n['dependencies'] for n in meta['resolve']['nodes']}
    pending = [p['id'] for p in packages.values() if p['name'] in names]
    if len(pending) < len(set(names)):
        return {'.'}
    seen, paths = set(), set()
    while pending:
        package = pending.pop()
        if package in seen:
            continue
        seen.add(package)
        pending.extend(nodes[package])
        directory = Path(packages[package]['manifest_path']).parent
        if directory.is_relative_to(ROOT):
            paths.add(str(directory.relative_to(ROOT)))
    return paths


def changed(key):
    try:
        if os.environ.get('FORCE_ALL') == '1' or not TREE:
            return None
        base = (GREEN / key).read_text().strip()
        data = subprocess.check_output(['git', 'diff', '--no-renames', '--name-only', '-z', base, TREE])
        return set(data.decode().rstrip('\0').split('\0')) - {''}
    except (OSError, subprocess.CalledProcessError):
        return None


def within(file, paths):
    return any(p == '.' or file == p or file.startswith(p + '/') for p in paths)


def unaffected(key, paths):
    delta = changed(key)
    if delta is None:
        return False
    # Only Rust source paths belong to the Cargo graph. Runtime resources,
    # generated files, manifests, deleted paths, tools and configurations
    # have unproven reachability and force a run rather than a false skip.
    known = crate_inputs([p['name'] for p in metadata()['packages'] if p['id'] in metadata()['workspace_members']])
    return all(f.endswith('.rs') and Path(f).is_file() and within(f, known)
               and not within(f, paths) for f in delta)


def mark(key):
    if TREE:
        GREEN.mkdir(parents=True, exist_ok=True)
        (GREEN / key).write_text(TREE + '\n')


if __name__ == '__main__':
    action, *args = sys.argv[1:]
    if action == 'crates':
        print(' '.join(sorted(crate_inputs(args))))
    elif action == 'fixture':
        fixture = Path(args[0])
        paths = crate_inputs(['og-cli', 'og-host-node']) | {str(fixture), 'packages/engine', 'packages/create'}
        inputs = fixture / 'inputs'
        if inputs.exists():
            paths.update(line.split('#', 1)[0].strip() for line in inputs.read_text().splitlines() if line.split('#', 1)[0].strip())
        print(' '.join(sorted(paths)))
    elif action == 'unchanged':
        sys.exit(0 if unaffected(args[0], set(' '.join(args[1:]).split())) else 1)
    else:
        raise ValueError(action)
