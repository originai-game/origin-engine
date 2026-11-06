#!/usr/bin/env python3
"""Count the test surface and verify retained reference bytes against a revision."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess


def git(*args):
    return subprocess.check_output(['git', *args])


def inventory(revision=None):
    if revision:
        paths = git('ls-tree', '-r', '--name-only', revision).decode().splitlines()
        def read(path):
            return git('show', f'{revision}:{path}')
    else:
        paths = git('ls-files', '--cached', '--others', '--exclude-standard').decode().splitlines()
        paths = sorted(set(p for p in paths if Path(p).is_file()))
        def read(path):
            return Path(path).read_bytes()
    result = dict(rust_tests=0, integration_source_files=0, integration_lines=0,
                  playtests=0, engine_reference_pngs=0, engine_reference_bytes=0,
                  package_reference_pngs=0, package_reference_bytes=0)
    frames = {}
    for path in paths:
        if path.startswith('crates/') and path.endswith('.rs'):
            data = read(path)
            result['rust_tests'] += len(re.findall(rb'#\[test\]', data))
            if '/tests/' in path:
                result['integration_source_files'] += 1
                result['integration_lines'] += len(data.splitlines())
        if path.endswith('.og.ts'):
            result['playtests'] += 1
        if '/references/' in path and path.endswith('.png'):
            data = read(path)
            prefix = 'engine' if path.startswith('crates/') else 'package'
            result[f'{prefix}_reference_pngs'] += 1
            result[f'{prefix}_reference_bytes'] += len(data)
            frames[path] = hashlib.sha256(data).hexdigest()
    return result, frames


parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--base', help='Compare with this Git revision; fail if a retained PNG changed')
args = parser.parse_args()
current, frames = inventory()
report = {'current': current}
if args.base:
    before, old_frames = inventory(args.base)
    changed = [p for p in frames.keys() & old_frames.keys() if frames[p] != old_frames[p]]
    report.update(base=args.base, before=before, changed_retained_frames=sorted(changed))
print(json.dumps(report, indent=2))
if args.base and changed:
    raise SystemExit('Retained reference PNGs changed')
