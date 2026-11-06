#!/usr/bin/env python3
"""Exercise the real landing command and locks; replace only the costly gate."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import time
import unittest

ROOT = Path(__file__).resolve().parent


class Landing(unittest.TestCase):
    def run_git(self, cwd, *args):
        return subprocess.check_output(['git', '-C', str(cwd), *args], stderr=subprocess.DEVNULL).decode().strip()

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.remote = self.root / 'remote'
        self.a = self.root / 'a'
        self.b = self.root / 'b'
        self.run_git(self.root, 'init', '--bare', str(self.remote))
        self.run_git(self.root, 'init', '-b', 'main', str(self.a))
        self.run_git(self.a, 'config', 'user.email', 'test@example.invalid')
        self.run_git(self.a, 'config', 'user.name', 'Landing Test')
        (self.a / 'scripts').mkdir()
        for name in ('land.sh', 'lock.sh'):
            shutil.copyfile(ROOT / name, self.a / 'scripts' / name)
        gate = self.a / 'scripts/check.sh'
        gate.write_text('''#!/usr/bin/env bash
set -eu
name="$(basename "$PWD")"
echo gate >> "$CONTROL/$name.calls"
touch "$CONTROL/$name.ready"
while [ ! -f "$CONTROL/$name.go" ]; do sleep 0.05; done
[ ! -f "$CONTROL/$name.fail" ]
''')
        gate.chmod(0o755)
        (self.a / 'shared').write_text('start\n' + 'context\n' * 20 + 'end\n')
        self.commit(self.a, 'seed')
        self.run_git(self.a, 'remote', 'add', 'origin', str(self.remote))
        self.run_git(self.a, 'push', 'origin', 'main')
        self.run_git(self.a, 'worktree', 'add', '-b', 'second', str(self.b))
        self.env = dict(os.environ, CONTROL=str(self.root), OG_GPU_ROOT=str(self.root / 'gpu'))

    def commit(self, cwd, subject):
        self.run_git(cwd, 'add', '.')
        self.run_git(cwd, 'commit', '-m', subject)

    def start(self, cwd):
        log = open(self.root / (cwd.name + '.log'), 'w+')
        self.addCleanup(log.close)
        proc = subprocess.Popen(['bash', 'scripts/land.sh'], cwd=cwd, env=self.env, stdout=log, stderr=log)
        self.addCleanup(lambda: proc.poll() is None and proc.kill())
        return proc, log

    def ready(self, name):
        deadline = time.monotonic() + 15
        while not (self.root / (name + '.ready')).exists():
            if time.monotonic() > deadline:
                self.fail(f'{name} never reached gate')
            time.sleep(0.05)

    def finish(self, job, ok=True):
        proc, log = job
        rc = proc.wait(timeout=30)
        log.seek(0)
        output = log.read()
        self.assertEqual(rc == 0, ok, output)
        return output

    def race(self, common=False, overlap=False):
        if overlap:
            p = self.a / 'shared'
            p.write_text(p.read_text().replace('start', 'first'))
            p = self.b / 'shared'
            p.write_text(p.read_text().replace('end', 'second'))
        else:
            (self.a / ('Cargo.toml' if common else 'first')).write_text('first\n')
            (self.b / 'second').write_text('second\n')
        self.commit(self.a, 'first')
        self.commit(self.b, 'second')
        a, b = self.start(self.a), self.start(self.b)
        self.ready('a')
        self.ready('b')
        (self.root / 'a.go').touch()
        self.finish(a)
        (self.root / 'b.go').touch()
        output = self.finish(b)
        calls = (self.root / 'b.calls').read_text().splitlines()
        self.assertEqual(len(calls), 2 if common or overlap else 1, output)
        self.assertEqual(self.run_git(self.remote, 'log', '-2', '--format=%s').splitlines(), ['second', 'first'])
        self.finish(self.start(self.b))
        self.assertEqual((self.root / 'b.calls').read_text().splitlines(), calls)
        self.assertFalse((self.a / '.git/og-landing/og-gate.lock').exists())

    def test_disjoint_race(self):
        self.race()

    def test_shared_input_race(self):
        self.race(common=True)

    def test_overlapping_file_race(self):
        self.race(overlap=True)

    def test_failed_gate_never_pushes(self):
        (self.a / 'first').write_text('first')
        self.commit(self.a, 'first')
        (self.root / 'a.go').touch()
        (self.root / 'a.fail').touch()
        self.finish(self.start(self.a), ok=False)
        self.assertEqual(self.run_git(self.remote, 'log', '-1', '--format=%s'), 'seed')


if __name__ == '__main__':
    unittest.main()
