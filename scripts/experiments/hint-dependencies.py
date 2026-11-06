"""Run from the repository root; measure dependency compiles, not consumer speed."""

import json
import os
from pathlib import Path
import statistics
import subprocess
import time

out = Path('target/evidence/test-restructure/build-experiments/hint')
out.mkdir(parents=True, exist_ok=True)
for package in ['wgpu', 'naga', 'icu_segmenter_data']:
    samples = {'no': [], 'yes': []}
    for run in range(-1, 5):
        for enabled in (['no', 'yes'] if run % 2 == 0 else ['yes', 'no']):
            command = ['cargo', 'rustc', '-p', package, '--lib', '--',
                       '-Zhint-mostly-unused=' + enabled,
                       f'-Cmetadata=hint-{package}-{run}-{enabled}']
            load = os.getloadavg()
            start = time.monotonic()
            result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
            elapsed = time.monotonic() - start
            log = out / f'{package}-{run}-{enabled}.log'
            log.write_bytes(result.stdout)
            built = f'Compiling {package} ' in result.stdout.decode()
            row = dict(package=package, run=run, hint=enabled, seconds=elapsed,
                       exit=result.returncode, compiled=built, load_before=load,
                       load_after=os.getloadavg(), command=command)
            print(json.dumps(row), flush=True)
            if result.returncode or not built:
                raise SystemExit(f'Invalid sample: {log}')
            if run >= 0:
                samples[enabled].append(elapsed)
    for enabled, values in samples.items():
        print(json.dumps(dict(package=package, hint=enabled,
                              median=statistics.median(values),
                              minimum=min(values), maximum=max(values))), flush=True)
