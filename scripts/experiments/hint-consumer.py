"""Run from the root; rebuild dependencies and consumers, then only og-cook.

Each sample takes the machine gate lock and all GPU slots. CPU activity
outside the gate is recorded, not excluded.
No sccache hits are timed. Run -1 is an unmeasured warm-up on each side.
"""

import json
import os
from pathlib import Path
import re
import statistics
import subprocess

out = Path('target/evidence/test-restructure/build-experiments/hint-consumer')
out.mkdir(parents=True, exist_ok=True)
env = dict(os.environ, RUSTC_WRAPPER='')
samples = {phase: {'no': [], 'yes': []} for phase in ('end-to-end', 'consumer')}
for run in range(-1, 5):
    for enabled in ('no', 'yes'):
        command = ['cargo', '-Zprofile-hint-mostly-unused', '--config',
                   'profile.dev.package.wgpu.hint-mostly-unused=' + str(enabled == 'yes').lower(),
                   '--config', 'profile.dev.package.naga.hint-mostly-unused=' + str(enabled == 'yes').lower(),
                   'build', '-vv', '-p', 'og-cli', '-p', 'og-host-node']
        for phase in samples:
            packages = ['wgpu', 'naga', 'og-cook'] if phase == 'end-to-end' else ['og-cook']
            clean = ['cargo', 'clean'] + [arg for p in packages for arg in ('-p', p)]
            subprocess.run(clean, check=True, stdout=subprocess.DEVNULL, env=env)
            # Start timing only after acquiring locks, not while queued.
            # Older macOS Python uses a process-relative monotonic clock: both
            # timestamps must be taken in the same process.
            timer = ('import os,time,json,subprocess,sys; load=os.getloadavg(); '
                     'start=time.monotonic(); result=subprocess.run(sys.argv[1:]); '
                     'print(json.dumps({"seconds":time.monotonic()-start,"load":load}),flush=True); '
                     'sys.exit(result.returncode)')
            result = subprocess.run(['sh', 'scripts/experiments/build.sh', 'python3', '-c', timer, *command],
                                    stdout=subprocess.PIPE, stderr=subprocess.STDOUT, env=env)
            log = out / f'{run}-{enabled}-{phase}.log'
            log.write_bytes(result.stdout)
            text = result.stdout.decode()
            timing = next(json.loads(line) for line in text.splitlines() if line.startswith('{"seconds":'))
            elapsed = timing['seconds']
            compiled = re.findall(r'^\s*Compiling (\S+)', text, re.M)
            if phase == 'end-to-end':
                for package in ('wgpu', 'naga'):
                    invocation = next(line for line in text.splitlines()
                                      if 'Running `' in line and f'--crate-name {package} ' in line)
                    assert ('-Zhint-mostly-unused' in invocation) == (enabled == 'yes'), invocation
                    assert '-Zthreads=4' in invocation and '-C opt-level=3' in invocation
            print(json.dumps(dict(run=run, phase=phase, hint=enabled, seconds=elapsed,
                                  exit=result.returncode, compiled=compiled, load_before=timing['load'],
                                  load_after=os.getloadavg(), command=command)), flush=True)
            if result.returncode or 'og-cook' not in compiled:
                raise SystemExit(f'Invalid sample: {log}')
            if run >= 0:
                samples[phase][enabled].append(elapsed)
for phase, sides in samples.items():
    for enabled, values in sides.items():
        print(json.dumps(dict(phase=phase, hint=enabled, median=statistics.median(values),
                              minimum=min(values), maximum=max(values))), flush=True)
