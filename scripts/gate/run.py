#!/usr/bin/env python3
"""Schedule a dependency DAG; commands and test selection live beside this file."""
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

# Inputs produced in this run, not path-based selection rules.
NEEDS = {
    'fmt': (), 'clippy': ('fmt',), 'rust-build': ('clippy',),
    'rust-cpu': ('rust-build',), 'rust-gpu': ('rust-build',),
    'drift': ('rust-build',), 'bun-quick': (),
    'wasm-build': (), 'wasm-bindgen': ('wasm-build',),
    'bun-test': ('rust-cpu', 'rust-gpu', 'wasm-bindgen', 'bun-quick'),
    'fixtures': ('bun-test',), 'doc-tests': ('rust-cpu',),
    'browser': ('fixtures',),
    'web-reference': ('wasm-bindgen', 'rust-build'),
    'check-conformance': (),
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


QUICK = ('fmt', 'clippy', 'rust-build', 'rust-cpu', 'rust-gpu', 'drift', 'bun-quick')
FULL = QUICK + ('wasm-build', 'wasm-bindgen', 'bun-test', 'fixtures', 'doc-tests', 'browser')


def main():
    args = sys.argv[1:]
    if any(a in args for a in ('--help', '-h', '--list')):
        print('scripts/check.sh [quick|full] [--all]\nscripts/check.sh --only STEP...')
        print('Steps: ' + ' '.join(NEEDS))
        return 0
    force = '--all' in args or bool(os.environ.get('CI'))
    args = [a for a in args if a not in ('--all', '--only')]
    lane = args.pop(0) if args and args[0] in ('quick', 'full') else 'full'
    wanted = set(args or (QUICK if lane == 'quick' else FULL))
    unknown = wanted - NEEDS.keys()
    if unknown:
        raise ValueError('Unknown steps: ' + ', '.join(sorted(unknown)))
    if lane == 'full':
        NEEDS['rust-cpu'] += ('wasm-build',)
        NEEDS['rust-gpu'] += ('wasm-build',)
    while True:
        expanded = wanted | {d for s in wanted for d in NEEDS[s]}
        if expanded == wanted:
            break
        wanted = expanded
    jobs = int(os.environ.get('OG_GATE_JOBS', '2'))
    threads = int(os.environ.get('OG_GATE_THREADS', '2'))
    if jobs < 1 or threads < 1:
        raise ValueError('OG_GATE_JOBS and OG_GATE_THREADS must be positive')
    logs = Path(os.environ['OG_EVIDENCE_DIR']) / 'gate' / 'runs' / f'{time.time_ns()}-{os.getpid()}'
    logs.mkdir(parents=True)
    env = dict(os.environ, GATE_LOGS=str(logs), OG_GATE_LANE=lane,
               FORCE_ALL=str(int(force)), OG_GATE_ALL='1' if lane == 'full' or force else '',
               OG_WASM_PROFILE='wasm-test', PYTHONUNBUFFERED='1')
    # Only the publish panic check needs shipping-shaped modules here, in 3D.
    env['OG_WASM_REFERENCE_ONLY'] = '1' if 'web-reference' in wanted and not wanted.intersection({'bun-test', 'fixtures', 'browser'}) else '0'
    env.pop('OG_WASM_STALE', None)
    # Capture Cargo's authoritative graph once per run, before parallel readers.
    with (logs / 'cargo.json').open('w') as out:
        subprocess.run(['cargo', 'metadata', '--locked', '--format-version=1'], stdout=out, check=True)
    active = {}
    completed = {}
    started = time.monotonic()
    print(f'gate {lane}: {jobs} steps × {threads} jobs; logs {logs}', flush=True)
    (logs / 'run.json').write_text(json.dumps({'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
                                            'tree': env.get('OG_GATE_TREE'), 'steps': sorted(wanted), 'force': force}))

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
                log = (logs / f'{step}.log').open('w')
                step_env = dict(env, OG_GATE_RAN=' ' + ' '.join(s for s in completed if completed[s].startswith('PASS ')) + ' ')
                proc = subprocess.Popen(['bash', 'scripts/gate/step.sh', step], env=step_env,
                                        stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
                active[step] = (proc, log, time.monotonic())
                print(f'--> {step}', flush=True)
            for step, (proc, log, begin) in list(active.items()):
                rc = proc.poll()
                if rc is None:
                    continue
                log.close()
                del active[step]
                duration = time.monotonic() - begin
                print((logs / f'{step}.log').read_text(), end='', flush=True)
                status = 'PASS' if rc == 0 else 'SKIPPED' if rc == 3 else 'FAIL'
                completed[step] = f'{status} {step} ({duration:.1f}s)'
                print(completed[step], flush=True)
                if rc not in (0, 3):
                    return 1
            if not active and len(completed) < len(wanted) and not any(
                s not in completed and all(d in completed for d in NEEDS[s]) for s in wanted
            ):
                raise RuntimeError('No runnable step in gate DAG')
            time.sleep(0.1)
        print(f'gate PASSED ({len(completed)}/{len(wanted)} steps, {time.monotonic() - started:.1f}s)', flush=True)
        return 0
    finally:
        # Each step has its own process group, including its compiler/test children.
        for proc, log, _ in active.values():
            stop_group(proc, signal.SIGTERM)
        for proc, log, _ in active.values():
            try:
                proc.wait(timeout=10)
            except subprocess.TimeoutExpired:
                stop_group(proc, getattr(signal, 'SIGKILL', signal.SIGTERM))
                proc.wait()
            log.close()
        summary = '\n'.join(completed[s] for s in NEEDS if s in completed) + '\n'
        (logs / 'summary.txt').write_text(summary)
        (Path(os.environ['OG_EVIDENCE_DIR']) / 'check-summary.txt').write_text(summary)


if __name__ == '__main__':
    try:
        sys.exit(main())
    except KeyboardInterrupt:
        sys.exit(130)
