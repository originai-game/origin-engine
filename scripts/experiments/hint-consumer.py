"""Run from the repository root; cleans only og-cook in the existing target directory."""

import json
import os
from pathlib import Path
import statistics
import subprocess
import time

out = Path('target/evidence/test-restructure/build-experiments/hint-consumer')
out.mkdir(parents=True, exist_ok=True)
env = dict(os.environ, RUSTC_WRAPPER='')
samples = {'no': [], 'yes': []}
for run in range(5):
    for enabled in (['no', 'yes'] if run % 2 == 0 else ['yes', 'no']):
        command = ['cargo', '-Zprofile-hint-mostly-unused', '--config',
                   'profile.dev.package.wgpu.hint-mostly-unused=' + str(enabled == 'yes').lower(),
                   '--config', 'profile.dev.package.naga.hint-mostly-unused=' + str(enabled == 'yes').lower(),
                   'build', '-p', 'og-cli', '-p', 'og-host-node']
        with (out / f'{run}-{enabled}-warm.log').open('wb') as log:
            subprocess.run(command, stdout=log, stderr=subprocess.STDOUT, env=env, check=True)
        subprocess.run(['cargo', 'clean', '-p', 'og-cook'], check=True, stdout=subprocess.DEVNULL)
        load = os.getloadavg()
        start = time.monotonic()
        result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, env=env)
        elapsed = time.monotonic() - start
        log = out / f'{run}-{enabled}.log'
        log.write_bytes(result.stdout)
        compiled = 'Compiling og-cook ' in result.stdout.decode()
        print(json.dumps(dict(run=run, hint=enabled, seconds=elapsed, exit=result.returncode,
                              compiled=compiled, load_before=load, load_after=os.getloadavg(),
                              command=command)), flush=True)
        if result.returncode or not compiled:
            raise SystemExit(f'Invalid sample: {log}')
        samples[enabled].append(elapsed)
for enabled, values in samples.items():
    print(json.dumps(dict(hint=enabled, median=statistics.median(values),
                          minimum=min(values), maximum=max(values))), flush=True)
