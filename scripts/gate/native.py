"""Discover source files and GPU requirements from the actual libtest binaries."""
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

from inputs import LOGS, GREEN, crate_inputs, mark, unaffected


def build():
    command = ['cargo', 'nextest', 'list', '--locked', '--workspace', '--list-type', 'binaries-only', '--message-format', 'json']
    if os.environ['OG_GATE_LANE'] == 'quick':
        command += ['--exclude', 'og-tests']
    with (LOGS / 'binaries.json').open('w') as out:
        subprocess.run(command, stdout=out, check=True)
    # The dev addon and og every later step uses (packages/engine/scripts/
    # build.sh copies them, drift and fixtures run target/debug/og), built
    # here once: features match the test build (docs/ci.md §Tests), so only
    # the addon compiles. Each cargo build re-links target/debug/og, so no
    # step after this one runs cargo for them.
    subprocess.run(['cargo', 'build', '--locked', '-q', '-p', 'og-host-node', '-p', 'og-cli'], check=True)
    listing = json.loads((LOGS / 'binaries.json').read_text())
    binaries = listing['rust-binaries']
    meta = listing['rust-build-meta']
    discovered = {}
    env = dict(os.environ, OG_RENDERER='none')
    env.pop('OG_GPU_REPORT_DIR', None)
    # Windows finds DLLs on PATH only, and keeps the toolchain's std DLL (proc-macro
    # test binaries link it) in the sysroot's bin/, not the reported libdir.
    loader = {'darwin': 'DYLD_FALLBACK_LIBRARY_PATH', 'win32': 'PATH'}.get(sys.platform, 'LD_LIBRARY_PATH')
    libraries = [str(Path(meta['target-directory']) / p) for p in meta['linked-paths']]
    libraries.extend(str(Path(meta['target-directory']) / p / 'deps') for p in meta['base-output-directories'])
    libraries.append(meta['platforms']['host']['libdir']['path'])
    if sys.platform == 'win32':
        libraries.append(str(Path(meta['platforms']['host']['libdir']['path']).parents[3] / 'bin'))
    env[loader] = os.pathsep.join(libraries + [env.get(loader, '')])
    for binary_id, binary in binaries.items():
        result = subprocess.run([binary['binary-path'], '--list', '--format=json', '-Zunstable-options'],
                                env=env, capture_output=True, text=True, check=True)
        tests = []
        for line in result.stdout.splitlines():
            record = json.loads(line)
            if record.get('type') != 'test' or record.get('event') != 'discovered':
                continue
            # The adapter-aware harness reports a GPU requirement without
            # creating an adapter. Nextest later preserves genuine ignored tests.
            gpu = 'OG_RENDERER=none' in (record.get('ignore_message') or '')
            tests.append({'name': record['name'], 'source': record.get('source_path'), 'gpu': gpu})
        discovered[binary_id] = {'binary': binary, 'tests': tests}
    (LOGS / 'discovered.json').write_text(json.dumps(discovered, indent=2))


def selection(binary_id, tests, gpu):
    """The binary's tests of one kind, as the shortest exact filterset.

    Discovery lists every test of the binary, so the whole binary, or all but
    the other kind, selects the same tests as naming them. Naming every CPU test
    exceeded Windows' 32 KiB command line (WinError 206 on studio-win).
    """
    def names(kind):
        return '|'.join(re.escape(t['name']).replace('/', r'\/') for t in tests if t['gpu'] == kind)
    mine, other = names(gpu), names(not gpu)
    if not other:
        return f'binary_id(={binary_id})'
    if len(other) < len(mine):
        return f'(binary_id(={binary_id}) & not test(/^(?:{other})$/))'
    return f'(binary_id(={binary_id}) & test(/^(?:{mine})$/))'


def run(kind):
    discovered = json.loads((LOGS / 'discovered.json').read_text())
    packages = {p['id']: p['name'] for p in json.loads((LOGS / 'cargo.json').read_text())['packages']}
    groups, keys = [], []
    for binary_id, entry in discovered.items():
        tests = [t for t in entry['tests'] if t['gpu'] == (kind == 'gpu')]
        if not tests:
            continue
        key = 'rust.' + kind + '.' + hashlib.sha256(binary_id.encode()).hexdigest()[:20]
        # Tests may invoke the CLI or addon without a Cargo dependency edge.
        inputs = crate_inputs([packages[entry['binary']['package-id']], 'og-cli', 'og-host-node'])
        if unaffected(key, inputs):
            print(f'SKIPPED {binary_id} {kind}: dependency inputs unchanged')
            continue
        (GREEN / key).unlink(missing_ok=True)
        keys.append(key)
        groups.append(selection(binary_id, entry['tests'], kind == 'gpu'))
    if not groups:
        return 3
    reports = LOGS / f'{kind}-gpu-reports'
    reports.mkdir()
    env = dict(os.environ, OG_GPU_REPORT_DIR=str(reports))
    # CPU discovery and execution must not open an adapter in an unrelated binary.
    if kind == 'cpu':
        env['OG_RENDERER'] = 'none'
    command = ['cargo', 'nextest', 'run', '--cargo-metadata', str(LOGS / 'cargo.json'),
               '--binaries-metadata', str(LOGS / 'binaries.json'), '--profile', kind,
               '--test-threads', os.environ['OG_GATE_THREADS'], '-E', ' | '.join(groups)]
    rc = subprocess.call(command, env=env)
    junit = Path(os.environ['OG_TARGET_DIR']) / 'nextest' / kind / 'junit.xml'
    if junit.exists():
        shutil.copyfile(junit, LOGS / f'rust-{kind}.junit.xml')
    if rc:
        return rc
    if kind == 'gpu' and list(reports.glob('*.txt')):
        for report in reports.glob('*.txt'):
            print(report.read_text(), end='')
        print('GPU checks are unchecked; no passing snapshot recorded')
        return 3
    for key in keys:
        mark(key)
    return 0


if __name__ == '__main__':
    if sys.argv[1] == 'build':
        build()
    else:
        sys.exit(run(sys.argv[1]))
