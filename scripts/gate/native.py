"""Discover source files and GPU requirements from the actual libtest binaries."""
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
from xml.etree import ElementTree

from inputs import LOGS, GREEN, ROOT, crate_inputs, mark, metadata, unaffected


def excluded():
    """Crates and binary ids the gate skips (scripts/gate-exclude.txt)."""
    crates, bins = [], []
    list_file = ROOT / 'scripts' / 'gate-exclude.txt'
    if list_file.is_file():
        for line in list_file.read_text().splitlines():
            line = line.split('#', 1)[0].strip()
            if not line:
                continue
            if line.startswith('bin '):
                bins.append(line.split()[1])
            else:
                crates.append(line)
    return crates, bins


def build_key(binary_id):
    return 'rust.build.' + hashlib.sha256(binary_id.encode()).hexdigest()[:20]


def package_name(entry):
    return entry['package-id'].rsplit('/', 1)[-1].split('#')[0]


def test_packages():
    """Workspace packages that own a test target, from cargo metadata."""
    names = set()
    for package in metadata()['packages']:
        if package['id'] not in metadata()['workspace_members']:
            continue
        if any('test' in t['kind'] for t in package['targets']):
            names.add(package['name'])
    return names


def build():
    lane = os.environ['OG_GATE_LANE']
    ex_crates, ex_bins = excluded()
    if lane == 'quick':
        ex_crates = ex_crates + ['og-tests']
    allowed_packages = test_packages() - set(ex_crates)

    # Per-binary input records (scripts/gate/inputs.py): a binary whose
    # inputs — its package's Cargo closure — are unchanged since its last
    # build reuses its stored listing, and cargo never relinks it. Only the
    # changed binaries' packages go to cargo, so a warm run relinks just
    # their packages' tests. A new or removed workspace package forces a
    # full list, as does any missing record, tests file or binary.
    stored = {}
    try:
        stored = json.loads((GREEN / 'rust.build.list.binaries.json').read_text())['rust-binaries']
    except (OSError, ValueError, KeyError):
        stored = {}
    listed = set(package_name(e) for e in stored.values())
    reuse, affected = {}, set()
    for binary_id, entry in stored.items():
        package = package_name(entry)
        if package not in allowed_packages or entry['binary-id'] in ex_bins:
            continue
        tests_file = GREEN / f'{build_key(binary_id)}.tests.json'
        try:
            reusable = (tests_file.is_file() and Path(entry['binary-path']).is_file()
                        and unaffected(build_key(binary_id), crate_inputs([package])))
        except (OSError, ValueError):
            reusable = False
        if reusable:
            reuse[binary_id] = entry
        else:
            affected.add(package)
    affected |= allowed_packages - listed

    if not affected and reuse:
        listing = json.loads((GREEN / 'rust.build.list.binaries.json').read_text())
    else:
        command = ['cargo', 'nextest', 'list', '--locked', '--list-type', 'binaries-only', '--message-format', 'json']
        # A scoped list builds only what it lists, so name the changed
        # packages; with none changed, --workspace covers new and removed
        # packages and the exclusions keep their targets unbuilt.
        if affected:
            command += [x for p in sorted(affected) for x in ('-p', p)]
        else:
            command += ['--workspace'] + [f'--exclude={c}' for c in sorted(ex_crates)]
        with (LOGS / 'binaries.json').open('w') as out:
            subprocess.run(command, stdout=out, check=True)
        listing = json.loads((LOGS / 'binaries.json').read_text())
        for binary_id, entry in reuse.items():
            listing['rust-binaries'].setdefault(binary_id, entry)
    (LOGS / 'binaries.json').write_text(json.dumps(listing))
    binaries = {b: e for b, e in listing['rust-binaries'].items()
                if package_name(e) in allowed_packages and e['binary-id'] not in ex_bins}
    listing['rust-binaries'] = binaries
    meta = listing['rust-build-meta']
    env = dict(os.environ, OG_RENDERER='none')
    env.pop('OG_GPU_REPORT_DIR', None)
    env.pop('OG_REFERENCE_REPORT_DIR', None)
    # Windows finds DLLs on PATH only, and keeps the toolchain's std DLL (proc-macro
    # test binaries link it) in the sysroot's bin/, not the reported libdir.
    loader = {'darwin': 'DYLD_FALLBACK_LIBRARY_PATH', 'win32': 'PATH'}.get(sys.platform, 'LD_LIBRARY_PATH')
    libraries = [str(Path(meta['target-directory']) / p) for p in meta['linked-paths']]
    libraries.extend(str(Path(meta['target-directory']) / p / 'deps') for p in meta['base-output-directories'])
    libraries.append(meta['platforms']['host']['libdir']['path'])
    if sys.platform == 'win32':
        libraries.append(str(Path(meta['platforms']['host']['libdir']['path']).parents[3] / 'bin'))
    env[loader] = os.pathsep.join(libraries + [env.get(loader, '')])
    # The dev addon and og every later step uses (packages/engine/scripts/
    # build.sh copies them, drift and fixtures run target/debug/og), built
    # here once: features match the test build (docs/ci.md §Tests), so only
    # the addon compiles. Each cargo build re-links target/debug/og, so no
    # step after this one runs cargo for them.
    subprocess.run(['cargo', 'build', '--locked', '-q', '-p', 'og-host-node', '-p', 'og-cli'], check=True)
    discovered = {}
    for binary_id in [b for b in binaries if b not in reuse]:
        binary = binaries[binary_id]
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
            tests.append({'name': record['name'], 'source': record.get('source_path'), 'gpu': gpu,
                          'ignored': bool(record.get('ignore')) and not gpu})
        discovered[binary_id] = {'binary': binary, 'tests': tests}
        GREEN.mkdir(parents=True, exist_ok=True)
        (GREEN / f'{build_key(binary_id)}.tests.json').write_text(json.dumps(tests))
        mark(build_key(binary_id))
    for binary_id, entry in reuse.items():
        tests = json.loads((GREEN / f'{build_key(binary_id)}.tests.json').read_text())
        discovered[binary_id] = {'binary': binaries[binary_id], 'tests': tests}
    (LOGS / 'discovered.json').write_text(json.dumps(discovered, indent=2))
    # The next run reuses these entries and listings for the binaries whose
    # inputs have not moved since.
    (GREEN / 'rust.build.list.binaries.json').write_text(json.dumps(listing))


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


def passed(junit, expected):
    """The binaries of a failed run that ran every one of their tests and
    failed none, from that run's JUnit report: one failing binary does not
    send the others back through the next run."""
    try:
        suites = ElementTree.parse(junit).getroot().iter('testsuite')
        return {s.get('name') for s in suites if s.get('failures') == '0' and s.get('errors') == '0'
                and len(s.findall('testcase')) == expected.get(s.get('name'))}
    except (OSError, ElementTree.ParseError):
        return set()


def run(kind):
    discovered = json.loads((LOGS / 'discovered.json').read_text())
    packages = {p['id']: p['name'] for p in json.loads((LOGS / 'cargo.json').read_text())['packages']}
    groups, keys, expected = [], {}, {}
    for binary_id, entry in discovered.items():
        tests = [t for t in entry['tests'] if t['gpu'] == (kind == 'gpu')]
        if not tests:
            continue
        key = 'rust.' + kind + '.' + hashlib.sha256(binary_id.encode()).hexdigest()[:20]
        inputs = crate_inputs([packages[entry['binary']['package-id']]])
        if unaffected(key, inputs):
            print(f'SKIPPED {binary_id} {kind}: dependency inputs unchanged')
            continue
        (GREEN / key).unlink(missing_ok=True)
        keys[binary_id] = key
        expected[binary_id] = sum(not t.get('ignored') for t in tests)
        groups.append(selection(binary_id, entry['tests'], kind == 'gpu'))
    if not groups:
        return 3
    reports = LOGS / f'{kind}-gpu-reports'
    reports.mkdir()
    # Reference frames that moved: reported by the harnesses, never failed
    # (docs/ci.md §Reference frames).
    moved = LOGS / f'{kind}-reference-mismatches'
    moved.mkdir()
    env = dict(os.environ, OG_GPU_REPORT_DIR=str(reports), OG_REFERENCE_REPORT_DIR=str(moved))
    # CPU discovery and execution must not open an adapter in an unrelated binary.
    if kind == 'cpu':
        env['OG_RENDERER'] = 'none'
    else:
        # A software rasterizer is no GPU (docs/ci.md §Lanes): its tests are
        # reported unchecked, never drawn on the CPU. OG_RENDERER=software
        # asks for it explicitly.
        env.setdefault('OG_RENDERER', 'hardware')
    threads = os.environ['OG_GATE_THREADS']
    gpu_config = None
    if kind == 'gpu':
        # step.sh reserves the adapter. One nextest thread is one GiB;
        # undeclared tests take the whole pool. Earlier explicit per-test
        # budgets in the checked-in config win over this fallback.
        threads = os.environ['OG_GPU_CAPACITY']
        gpu_config = LOGS / 'gpu-nextest.toml'
        gpu_config.write_text((ROOT / '.config/nextest.toml').read_text() + f'''
[[profile.gpu.overrides]]
filter = 'all()'
threads-required = {threads}
''')
        print(f'GPU memory pool: {threads} GiB credits; undeclared tests reserve all', flush=True)
    command = ['cargo', 'nextest', 'run', '--cargo-metadata', str(LOGS / 'cargo.json'),
               '--binaries-metadata', str(LOGS / 'binaries.json'), '--profile', kind,
               '--test-threads', threads, '-E', ' | '.join(groups)]
    if gpu_config:
        command.extend(['--config-file', str(gpu_config)])
    if kind == 'gpu':
        # Without a hardware adapter every GPU test is skipped: nothing to run
        # is the unchecked report below, not nextest's failure.
        command.append('--no-tests=warn')
    junit = Path(os.environ['OG_TARGET_DIR']) / 'nextest' / kind / 'junit.xml'
    junit.unlink(missing_ok=True)
    rc = subprocess.call(command, env=env)
    if junit.exists():
        shutil.copyfile(junit, LOGS / f'rust-{kind}.junit.xml')
    count = 0
    for report in sorted(moved.glob('*.txt')):
        text = report.read_text()
        count += text.count('REFERENCE MISMATCH ')
        print(text, end='')
    if count:
        print(f'{count} reference frames moved: reported, not failed; review each actual and diff PNG named above')
    if kind == 'gpu' and list(reports.glob('*.txt')):
        for report in reports.glob('*.txt'):
            print(report.read_text(), end='')
        print('GPU checks are unchecked; no passing snapshot recorded')
        return rc or 3
    green = passed(junit, expected) if rc else set(keys)
    for binary_id, key in keys.items():
        if binary_id in green:
            mark(key)
    return rc


if __name__ == '__main__':
    if sys.argv[1] == 'build':
        build()
    else:
        sys.exit(run(sys.argv[1]))
