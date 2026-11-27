#!/usr/bin/env python3
"""Schedule a dependency DAG; commands and test selection live beside this file."""
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import time

# Inputs produced in this run, not path-based selection rules.
NEEDS = {
    'preflight': (), 'fmt': (), 'clippy': ('fmt',), 'rust-build': ('clippy', 'preflight'),
    'native-ready': ('rust-build',), 'drift': ('rust-build',), 'api-surface': ('native-ready',),
    # og-cli's style check bundles a game with the package's own addon.
    'rust-cpu': ('rust-build', 'native-ready'), 'rust-gpu': ('rust-build',),
    'bun-quick': ('preflight',), 'engine-typecheck': ('rust-build',),
    'wasm-check': ('preflight',),
    'wasm-build': ('preflight',), 'wasm-bindgen': ('wasm-build',),
    'web-ready': ('native-ready', 'wasm-bindgen'),
    'bun-test': ('web-ready', 'api-surface', 'drift', 'bun-quick', 'engine-typecheck'),
    # Fixtures run the native og alone: a fixtures-only run builds no wasm.
    'fixtures': ('native-ready',), 'doc-tests': ('rust-build',),
    'showcase': ('native-ready',),
    # The release's preview slots, cooked and run by the native og.
    'looks': ('native-ready',),
    'browser': ('web-ready', 'api-surface', 'drift', 'engine-typecheck'),
    'web-reference': ('web-ready',),
    # Safari (macOS): the browser tests' lanes and publish builds over WebDriver.
    'safari': ('wasm-bindgen', 'rust-build'),
    'check-conformance': (),
    'shape-wasm32': ('preflight',),
}


def stop_group(proc, sig):
    """End a step and its compiler/test children."""
    if os.name == 'nt':
        # Git Bash on studio-win: Windows has no process groups; taskkill ends the tree.
        subprocess.run(['taskkill', '/T', '/F', '/PID', str(proc.pid)], capture_output=True)
        return
    try:
        os.killpg(proc.pid, sig)
    except ProcessLookupError:
        pass


QUICK = ('fmt', 'clippy', 'rust-build', 'rust-cpu', 'rust-gpu', 'drift', 'api-surface', 'bun-quick', 'engine-typecheck', 'wasm-check')
FULL = QUICK + ('wasm-build', 'wasm-bindgen', 'bun-test', 'fixtures', 'showcase', 'looks', 'doc-tests', 'browser', 'shape-wasm32')
# These consumers can cook the same projects. Serialize them when selected
# together without adding tests to a scoped run merely to enforce ordering.
PROJECT_STEPS = {'bun-test', 'fixtures', 'showcase', 'looks', 'browser', 'web-reference'}


def output_bytes(step, env):
    """Logical size of published products; don't walk Cargo caches or scratch."""
    wasm = Path(env.get('OG_WASM_OUT', 'target/og-wasm'))
    paths = []
    if step == 'wasm-build':
        paths = list((wasm / 'raw').glob('*.wasm'))
    elif step == 'wasm-bindgen':
        paths = [p for d in ('bindgen', 'dev', 'split') for p in (wasm / d).rglob('*')]
    elif step in ('native-ready', 'web-ready'):
        roots = [Path('packages/engine/native')] if step == 'native-ready' else list(Path('packages/engine/dist').glob('web*'))
        paths = [p for root in roots for p in root.rglob('*')]
    elif step == 'rust-build':
        try:
            data = json.loads((Path(env['GATE_LOGS']) / 'binaries.json').read_text())
            paths = [Path(b['binary-path']) for b in data['rust-binaries'].values()]
        except (OSError, ValueError, KeyError):
            return None
    else:
        return None
    return sum(p.stat().st_size for p in set(paths) if p.is_file())


def record_timing(row, logs, step, duration, rc, env, cancelled=False):
    def marker(name):
        try:
            return float((logs / f'{step}.{name}-start').read_text())
        except (OSError, ValueError):
            return None

    start, command = marker('resource'), marker('command')
    wait = max(0, command - start) if start is not None and command is not None else None
    try:
        products = output_bytes(step, env) if not cancelled and rc == 0 else None
    except OSError:
        products = None
    row.update(wall_s=duration, resource_wait_s=wait,
               command_s=max(0, row['launch_monotonic'] + duration - command) if command is not None else None,
               log_bytes=(logs / f'{step}.log').stat().st_size,
               product_bytes=products, exit_code=rc, cancelled=cancelled)
    del row['launch_monotonic']


def main():
    args = sys.argv[1:]
    if any(a in args for a in ('--help', '-h', '--list')):
        print('scripts/check.sh [quick|full] [--all | --resume RUN_ID]\nscripts/check.sh --only STEP...')
        print('Steps: ' + ' '.join(NEEDS))
        return 0
    resume = None
    if '--resume' in args:
        i = args.index('--resume')
        if i + 1 >= len(args) or not re.fullmatch(r'[0-9]+-[0-9]+', args[i + 1]):
            raise ValueError('--resume requires a gate run directory name (timestamp-pid)')
        if '--all' in args or os.environ.get('CI'):
            raise ValueError('--resume is incompatible with a fresh forced run (--all or CI)')
        resume = args[i + 1]
        del args[i:i + 2]
    force = bool(resume) or '--all' in args or bool(os.environ.get('CI'))
    only = '--only' in args
    args = [a for a in args if a not in ('--all', '--only')]
    lane = args.pop(0) if args and args[0] in ('quick', 'full') else 'full'
    wanted = set(args or (QUICK if lane == 'quick' else FULL))
    unknown = wanted - NEEDS.keys()
    if unknown:
        raise ValueError('Unknown steps: ' + ', '.join(sorted(unknown)))
    # Landing batches that request Bun still cover both native suites. This
    # is selection policy, not a build/runtime prerequisite for Bun.
    if 'bun-test' in wanted:
        wanted.update(('rust-cpu', 'rust-gpu'))
    while True:
        expanded = wanted | {d for s in wanted for d in NEEDS[s]}
        if expanded == wanted:
            break
        wanted = expanded
    # Order early checks before execution, but don't add them to aside scopes.
    for step in ('rust-cpu', 'rust-gpu', 'fixtures', 'showcase', 'looks'):
        NEEDS[step] += tuple(s for s in ('drift', 'api-surface', 'engine-typecheck') if s in wanted)
    # OG_GATE_SKIP: steps the coordinator named for an exception (docs/ci.md
    # §Lanes). A dependent still runs; the skip is printed, never a pass.
    # A landing's quick refuses every skip but rust-gpu: the standing
    # exception (docs/ci.md §Lanes); a skipped fmt, drift, wasm or test step
    # is how a tree a cold run fails lands on main.
    skip = [s.strip() for s in os.environ.get('OG_GATE_SKIP', '').split(',') if s.strip()]
    unknown = set(skip) - NEEDS.keys()
    if unknown:
        raise ValueError('OG_GATE_SKIP names unknown steps: ' + ', '.join(sorted(unknown)))
    unlandable = sorted(s for s in skip if s != 'rust-gpu')
    if os.environ.get('OG_GATE_LANDING') and unlandable:
        raise ValueError('landing: OG_GATE_SKIP dropped steps a landing runs ('
                         + ', '.join(unlandable)
                         + '); only rust-gpu may be skipped for a landing')
    skipped = [s for s in NEEDS if s in wanted and s in skip]
    # In a browser run the all-variant wasm build type-checks the lanes
    # variants first, and step_wasm_check's lanes half then reuses it.
    if {'browser', 'wasm-build', 'wasm-check'} <= wanted:
        NEEDS['wasm-check'] += ('wasm-build',)
    if resume and skip:
        raise ValueError('--resume is incompatible with OG_GATE_SKIP')
    jobs = int(os.environ.get('OG_GATE_JOBS', '2'))
    threads = int(os.environ.get('OG_GATE_THREADS', '2'))
    if jobs < 1 or threads < 1:
        raise ValueError('OG_GATE_JOBS and OG_GATE_THREADS must be positive')
    runs = Path(os.environ['OG_EVIDENCE_DIR']) / 'gate' / 'runs'
    if resume and not (runs / resume / 'run.json').is_file():
        raise ValueError(f'--resume: no run {resume} in {runs}')
    logs = runs / f'{time.time_ns()}-{os.getpid()}'
    logs.mkdir(parents=True)
    env = dict(os.environ, GATE_LOGS=str(logs), OG_GATE_LANE=lane,
               FORCE_ALL=str(int(force)), OG_GATE_ALL='1' if lane == 'full' or force else '',
               OG_WASM_PROFILE='wasm-test', PYTHONUNBUFFERED='1',
               OG_BIN=str(Path(os.environ['OG_TARGET_DIR']) / 'debug' / 'og'))
    # The wasm beside the development module (commands.sh step_wasm_build):
    # the browser tests (Chrome's or Safari's) load every publish and lanes variant, web-reference's
    # panic check the 3D publish pair, and bun-test (a landing) none of them.
    env['OG_WASM_VARIANTS'] = 'all' if 'browser' in wanted or 'safari' in wanted else 'reference' if 'web-reference' in wanted else 'dev'
    env['OG_GATE_STEPS'] = ' '.join(sorted(wanted - set(skipped)))
    env.pop('OG_WASM_STALE', None)
    env['OG_GATE_RESUME'] = resume or ''
    # Capture Cargo's authoritative graph once per run, before parallel readers.
    with (logs / 'cargo.json').open('w') as out:
        subprocess.run(['cargo', 'metadata', '--locked', '--format-version=1'], stdout=out, check=True)
    lane_record = Path(os.environ.get('OG_TARGET_DIR', 'target')) / 'og-gate' / 'green' / 'lane.full'
    whole = lane == 'full' and force and not only and not skipped
    if whole:
        lane_record.unlink(missing_ok=True)
    active = {}
    timings = {}
    completed = {s: f'SKIPPED {s} (OG_GATE_SKIP)' for s in skipped}
    started = time.monotonic()
    print(f'gate {lane}: {jobs} steps × {threads} jobs; logs {logs}', flush=True)
    for s in skipped:
        print(completed[s], flush=True)
    (logs / 'run.json').write_text(json.dumps({'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
                                            'tree': env.get('OG_GATE_TREE'), 'steps': sorted(wanted), 'skipped': skipped, 'force': force}))

    def interrupt(signum, frame):
        raise KeyboardInterrupt

    signal.signal(signal.SIGTERM, interrupt)
    try:
        while len(completed) < len(wanted):
            for step in NEEDS:
                if step not in wanted or step in completed or step in active or len(active) >= jobs:
                    continue
                if not all(d in completed for d in NEEDS[step]):
                    continue
                if step in PROJECT_STEPS and PROJECT_STEPS.intersection(active):
                    continue
                log = (logs / f'{step}.log').open('w')
                step_env = dict(env, OG_GATE_RAN=' ' + ' '.join(s for s in completed if completed[s].startswith('PASS ')) + ' ')
                begin = time.monotonic()
                proc = subprocess.Popen(['bash', 'scripts/gate/step.sh', step], env=step_env,
                                        stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
                timings[step] = {'started_unix_ns': time.time_ns(), 'launch_monotonic': begin}
                active[step] = (proc, log, begin)
                print(f'--> {step}', flush=True)
            for step, (proc, log, begin) in list(active.items()):
                rc = proc.poll()
                if rc is None:
                    continue
                log.close()
                del active[step]
                duration = time.monotonic() - begin
                record_timing(timings[step], logs, step, duration, rc, env)
                print((logs / f'{step}.log').read_text(), end='', flush=True)
                reused = resume and rc == 4 and step == 'bun-test' and (logs / 'bun-test-suite.receipt').is_file()
                status = 'REUSED' if reused else {0: 'PASS', 3: 'SKIPPED'}.get(rc, 'FAIL')
                completed[step] = f'{status} {step} ({duration:.1f}s)'
                print(completed[step], flush=True)
                if status == 'FAIL':
                    return 1
            if not active and len(completed) < len(wanted) and not any(
                s not in completed and all(d in completed for d in NEEDS[s]) for s in wanted
            ):
                raise RuntimeError('No runnable step in gate DAG')
            time.sleep(0.1)
        ran = len(completed) - len(skipped)
        # The whole full lane forced through (--all), every step run and
        # passed, none unchecked: scripts/land.sh lands this tree without its
        # opening quick (docs/ci.md §Lanes).
        if whole and not resume and env.get('OG_GATE_TREE') and all(completed[s].startswith('PASS ') for s in wanted):
            lane_record.parent.mkdir(parents=True, exist_ok=True)
            lane_record.write_text(env['OG_GATE_TREE'] + '\n')
        note = f'; NOT RUN (OG_GATE_SKIP): {", ".join(skipped)}' if skipped else ''
        if resume:
            note += f'; continuation of {resume} (see per-suite reuse records)'
        print(f'gate PASSED ({ran}/{len(wanted)} steps, {time.monotonic() - started:.1f}s{note})', flush=True)
        return 0
    finally:
        # Each step has its own process group, including its compiler/test children.
        for proc, log, _ in active.values():
            stop_group(proc, signal.SIGTERM)
        for step, (proc, log, begin) in active.items():
            try:
                proc.wait(timeout=10)
            except subprocess.TimeoutExpired:
                stop_group(proc, getattr(signal, 'SIGKILL', signal.SIGTERM))
                proc.wait()
            log.close()
            record_timing(timings[step], logs, step, time.monotonic() - begin, proc.returncode, env, cancelled=True)
        summary = '\n'.join(completed[s] for s in NEEDS if s in completed) + '\n'
        (logs / 'timings.json').write_text(json.dumps(timings, indent=2) + '\n')
        (logs / 'summary.txt').write_text(summary)
        (Path(os.environ['OG_EVIDENCE_DIR']) / 'check-summary.txt').write_text(summary)


if __name__ == '__main__':
    try:
        sys.exit(main())
    except KeyboardInterrupt:
        sys.exit(130)
