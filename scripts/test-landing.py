#!/usr/bin/env python3
"""Exercise real Git rebases, landing and locks with cheap gate/generator fixtures."""
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
        shutil.copyfile(ROOT / 'rebase-generated.py', self.a / 'scripts/landing_impl.py')
        (self.a / 'scripts/rebase-generated.py').write_text('''
import landing_impl as impl
from pathlib import Path
import sys
def regenerate():
    if Path("fail-generation").exists():
        raise RuntimeError("generation failed")
    if Path("inputs").exists():
        value = "".join(p.read_text() for p in sorted(Path("inputs").iterdir()))
        for name in ("packages/engine/types/index.d.ts", "packages/knowledge/wiki/search.tsv"):
            p = Path(name)
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text(value)
        impl.git("add", "-A", "--", "packages")
impl.regenerate = regenerate
impl.rebase(sys.argv[1])
''')
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

    def generated_conflict(self, race=False, source_conflict=False, fail=False, later_generated=True):
        path = 'packages/engine/types/index.d.ts'
        for repo, value in ((self.a, 'upstream'), (self.b, 'branch')):
            (repo / 'inputs').mkdir()
            (repo / 'inputs' / value).write_text(value + '\n')
            for name in (path, 'packages/knowledge/wiki/search.tsv'):
                p = repo / name
                p.parent.mkdir(parents=True)
                p.write_text(value)
            if source_conflict:
                (repo / 'shared').write_text(value)
            self.commit(repo, value)
        # A later commit rewrites the generated text again. Final regeneration
        # must include both upstream and every replayed source change.
        (self.b / 'inputs/later').write_text('later\n')
        if later_generated:
            (self.b / path).write_text('later')
        if fail:
            (self.b / 'fail-generation').touch()
        self.commit(self.b, 'later')
        if race:
            job = self.start(self.b)
            self.ready('b')
        self.run_git(self.a, 'push', 'origin', 'main')
        (self.root / 'b.go').touch()
        output = self.finish(job if race else self.start(self.b), ok=not (source_conflict or fail))
        if source_conflict or fail:
            self.assertEqual(self.run_git(self.remote, 'log', '-1', '--format=%s'), 'upstream')
            self.assertTrue(self.run_git(self.b, 'diff', '--name-only', '--diff-filter=U'))
        else:
            self.assertIn('regenerating generated-only conflicts', output)
            for name in (path, 'packages/knowledge/wiki/search.tsv'):
                self.assertEqual(self.run_git(self.remote, 'show', 'main:' + name), 'branch\nlater\nupstream')
            self.assertEqual(self.run_git(self.b, 'status', '--porcelain'), '')

    def test_generated_conflict(self):
        self.generated_conflict()

    def test_generated_conflict_after_gate(self):
        self.generated_conflict(race=True)

    def test_clean_later_commit_is_regenerated(self):
        self.generated_conflict(later_generated=False)

    def test_mixed_conflict_stops(self):
        self.generated_conflict(source_conflict=True)

    def test_generation_failure_stops(self):
        self.generated_conflict(fail=True)

    def test_handwritten_declarations_stop(self):
        for repo, value in ((self.a, 'upstream'), (self.b, 'branch')):
            (repo / 'manual.d.ts').write_text(value)
            self.commit(repo, value)
        self.run_git(self.a, 'push', 'origin', 'main')
        output = self.finish(self.start(self.b), ok=False)
        self.assertNotIn('regenerating generated-only conflicts', output)
        self.assertEqual(self.run_git(self.remote, 'log', '-1', '--format=%s'), 'upstream')
        self.assertEqual(self.run_git(self.b, 'diff', '--name-only', '--diff-filter=U'), 'manual.d.ts')


if __name__ == '__main__':
    unittest.main()
