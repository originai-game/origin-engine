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
        self.run_git(self.root, 'init', '--bare', '-b', 'main', str(self.remote))
        self.run_git(self.root, 'init', '-b', 'main', str(self.a))
        self.run_git(self.a, 'config', 'user.email', 'test@example.invalid')
        self.run_git(self.a, 'config', 'user.name', 'Landing Test')
        # The scratch identity has no key: a host that signs every commit
        # (an Amp orb's system config) refused its commits.
        self.run_git(self.a, 'config', 'commit.gpgsign', 'false')
        (self.a / '.gitignore').write_text('/.og/tmp/\n')
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
# An allowance file admits one gate call per line.
while [ -f "$CONTROL/$name.allow" ] && [ "$(wc -l < "$CONTROL/$name.allow")" -lt "$(wc -l < "$CONTROL/$name.calls")" ]; do sleep 0.05; done
[ ! -f "$CONTROL/$name.fail" ]
[ ! -f breaks-c ] || [ ! -f scripts/c ]
''')
        gate.chmod(0o755)
        (self.a / 'shared').write_text('start\n' + 'context\n' * 20 + 'end\n')
        self.commit(self.a, 'seed')
        self.run_git(self.a, 'remote', 'add', 'origin', str(self.remote))
        self.run_git(self.a, 'push', 'origin', 'main')
        self.run_git(self.a, 'worktree', 'add', '-b', 'second', str(self.b))
        self.env = dict(os.environ, CONTROL=str(self.root), OG_GPU_ROOT=str(self.root / 'gpu'),
                        PYTHONDONTWRITEBYTECODE='1')

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

    def calls(self, name):
        path = self.root / f'{name}.calls'
        return len(path.read_text().splitlines()) if path.exists() else 0

    def test_queued_landers_recheck_at_most_once(self):
        # Two shared-input landers recheck after a third lands. Their rechecks
        # must not race: the loser would recheck again, round after round.
        c = self.root / 'c'
        self.run_git(self.a, 'worktree', 'add', '-b', 'third', str(c))
        (self.a / 'Cargo.toml').write_text('first\n')
        (self.b / 'scripts/second').write_text('second\n')
        (c / 'scripts/third').write_text('third\n')
        for repo, subject in ((self.a, 'first'), (self.b, 'second'), (c, 'third')):
            self.commit(repo, subject)
        for name in ('b', 'c'):
            (self.root / f'{name}.allow').write_text('1\n')
        jobs = {name: self.start(repo) for name, repo in (('a', self.a), ('b', self.b), ('c', c))}
        for name in jobs:
            self.ready(name)
        (self.root / 'a.go').touch()
        self.finish(jobs['a'])
        (self.root / 'b.go').touch()
        deadline = time.monotonic() + 15
        while self.calls('b') < 2:
            self.assertLess(time.monotonic(), deadline, 'b never rechecked')
            time.sleep(0.05)
        (self.root / 'c.go').touch()
        time.sleep(1)  # c's first gate ends while b's recheck is still running
        for name in ('b', 'c'):
            (self.root / f'{name}.allow').write_text('1\n' * 9)
        self.finish(jobs['b'])
        self.assertIn('landed in a batch', self.finish(jobs['c']))
        self.assertEqual((self.calls('b'), self.calls('c')), (3, 1))
        self.assertEqual(sorted(self.run_git(self.remote, 'log', '-3', '--format=%s').splitlines()),
                         ['first', 'second', 'third'])

    def drain(self, breaker=False, outside=False):
        # b rechecks at the head of the queue while c, then d, pass quick and
        # wait. b lands itself, then drains both as one stack. With d breaking
        # c, the stack fails: c (the passing prefix) lands, d goes back.
        repos = {'b': self.b}
        for name in ('c', 'd'):
            repos[name] = self.root / name
            self.run_git(self.a, 'worktree', 'add', '-b', name, str(repos[name]))
        (self.a / 'Cargo.toml').write_text('upstream\n')
        self.commit(self.a, 'upstream')
        for name, repo in repos.items():
            (repo / 'scripts' / name).write_text(name + '\n')
            if breaker and name == 'd':
                (repo / 'breaks-c').write_text('d\n')
            self.commit(repo, name)
        (self.root / 'b.allow').write_text('1\n')
        jobs = {name: self.start(repo) for name, repo in repos.items()}
        for name in jobs:
            self.ready(name)
        self.run_git(self.a, 'push', 'origin', 'main')
        if outside:
            self.outside(None, ('outside', 'o'))
        (self.root / 'b.go').touch()
        deadline = time.monotonic() + 15
        while self.calls('b') < 2:
            self.assertLess(time.monotonic(), deadline, 'b never rechecked')
            time.sleep(0.05)
        offers = self.a / '.git/og-landing/batch'
        for count, name in enumerate(('c', 'd'), 1):
            (self.root / f'{name}.go').touch()
            while len(list(offers.iterdir())) < count:
                self.assertLess(time.monotonic(), deadline, f'{name} never offered')
                time.sleep(0.05)
            time.sleep(1.1)  # offers are named by the second: c before d
        (self.root / 'b.allow').write_text('1\n' * 9)
        self.finish(jobs['b'])
        out = {name: self.finish(jobs[name], ok=not (breaker and name == 'd')) for name in ('c', 'd')}
        self.assertIn('landed in a batch', out['c'])
        self.assertEqual((self.calls('c'), self.calls('d')), (1, 1))
        self.assertEqual(self.run_git(repos['c'], 'rev-parse', 'HEAD'),
                         self.run_git(self.remote, 'rev-parse', 'main' if breaker else 'main~1'))
        self.assertEqual(self.run_git(repos['b'], 'symbolic-ref', '--short', 'HEAD'), 'second')
        return out

    def test_holder_drains_the_queue_as_one_batch(self):
        out = self.drain()
        self.assertIn('landed in a batch', out['d'])
        self.assertEqual(self.calls('b'), 3)
        self.assertEqual(self.run_git(self.remote, 'log', '-4', '--format=%s').splitlines(),
                         ['d', 'c', 'b', 'upstream'])

    def test_failing_head_goes_back_and_the_passing_prefix_lands(self):
        out = self.drain(breaker=True)
        self.assertIn('failed quick', out['d'])
        self.assertEqual(self.calls('b'), 4)  # own recheck, the stack, then c alone
        self.assertEqual(self.run_git(self.remote, 'log', '-3', '--format=%s').splitlines(),
                         ['c', 'b', 'upstream'])
        self.assertFalse(list((self.a / '.git/og-landing/batch').iterdir()))

    def outside(self, *writes):
        # Another orb's lander: a separate clone, so a separate queue. Each
        # push from a's Git directory is first beaten by the next of writes
        # (a path and its text, committed on the remote's main from there),
        # or let through by None.
        o = self.root / 'o'
        self.run_git(self.root, 'clone', '-q', str(self.remote), str(o))
        for key, value in (('user.email', 'o@example.invalid'), ('user.name', 'Outside'), ('commit.gpgsign', 'false')):
            self.run_git(o, 'config', key, value)
        (self.root / 'outside').write_text(''.join('-\n' if w is None else f'{w[0]} {w[1]}\n' for w in writes))
        hooks = self.root / 'hooks'
        hooks.mkdir()
        (hooks / 'pre-push').write_text('''#!/usr/bin/env bash
set -e
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_PREFIX
q="$CONTROL/outside"
[ -s "$q" ] || exit 0
line="$(head -n 1 "$q")"; sed -i 1d "$q"
[ "$line" != - ] || exit 0
n="$(cat "$CONTROL/outside.n" 2>/dev/null || echo 0)"; echo $((n + 1)) > "$CONTROL/outside.n"
cd "$CONTROL/o"
git fetch -q origin && git checkout -q -B w origin/main
mkdir -p "$(dirname "${line%% *}")"; echo "${line#* }" > "${line%% *}"
git add -A && git commit -qm "outside $n" && git push -q origin HEAD:refs/heads/main
''')
        (hooks / 'pre-push').chmod(0o755)
        self.run_git(self.a, 'config', 'core.hooksPath', str(hooks))

    def outside_race(self, writes, change=('first', 'first\n'), ok=True):
        (self.a / change[0]).parent.mkdir(parents=True, exist_ok=True)
        (self.a / change[0]).write_text(change[1])
        self.commit(self.a, 'first')
        self.outside(*writes)
        (self.root / 'a.go').touch()
        output = self.finish(self.start(self.a), ok=ok)
        return output, self.run_git(self.remote, 'log', '--format=%s', 'main').splitlines()

    def test_outside_writer_disjoint_lands_without_a_recheck(self):
        output, log = self.outside_race([('outside', 'o'), ('outside', 'o2')])
        self.assertEqual(output.count('pushing without a recheck'), 2, output)
        self.assertEqual(self.calls('a'), 1)
        self.assertEqual(log[:3], ['first', 'outside 1', 'outside 0'])
        self.assertEqual(self.run_git(self.a, 'rev-parse', 'main'), self.run_git(self.remote, 'rev-parse', 'main'))

    def test_outside_writer_on_shared_inputs_rechecks_then_lands(self):
        output, log = self.outside_race([('Cargo.toml', 'o')])
        self.assertIn('rechecking affected quick steps', output)
        self.assertEqual(self.calls('a'), 2)
        self.assertEqual(log[:2], ['first', 'outside 0'])

    def test_outside_writer_failing_recheck_never_pushes(self):
        output, log = self.outside_race([('scripts/c', 'c')], change=('breaks-c', 'a\n'), ok=False)
        self.assertEqual(self.calls('a'), 2)
        self.assertEqual(log[0], 'outside 0')

    def test_outside_writer_conflict_stops(self):
        output, log = self.outside_race([('shared', 'outside')], change=('shared', 'first\n'), ok=False)
        self.assertIn('conflicts with the new origin/main', output)
        self.assertEqual(log[0], 'outside 0')

    def test_outside_writer_during_a_drain_lands_the_stack(self):
        # b's own push goes through; an outside write beats the stack's
        # push. The stack's scripts/ are a shared input: one more recheck.
        self.drain(outside=True)
        self.assertEqual(self.calls('b'), 4)
        self.assertEqual(self.run_git(self.remote, 'log', '-5', '--format=%s').splitlines(),
                         ['d', 'c', 'outside 0', 'b', 'upstream'])

    # A stand-in for scripts/css-coverage.py: proof calls and MISSING entries,
    # then the checklist written from them, as the engine's does.
    COVERAGE = '''from pathlib import Path
ROWS = []
def proof(features, module, test, scope):
    ROWS.extend(features.split())
proof("seed", "m", "t", "s")
# more proofs
MISSING = {
    "gap": "reason",
}
rows = [f"| {f} | proved |" for f in sorted(ROWS)] + [f"| {k} | missing |" for k in sorted(MISSING)]
'''
    # Today's checklist: a file of its own, written whole.
    WHOLE = '''(Path(__file__).resolve().parent.parent / "docs/css-coverage.md").write_text("\\n".join(rows) + "\\n")
'''
    # Before the move: a marked region of docs/ui.md.
    REGION = '''doc = Path(__file__).resolve().parent.parent / "docs/ui.md"
head, _, rest = doc.read_text().partition("<!-- css-coverage:start -->")
_, _, tail = rest.partition("<!-- css-coverage:end -->")
doc.write_text(head + "<!-- css-coverage:start -->\\n" + "\\n".join(rows) + "\\n<!-- css-coverage:end -->" + tail)
'''

    def add_entries(self, repo, name):
        p = repo / 'scripts/css-coverage.py'
        p.write_text(p.read_text()
                     .replace('# more proofs', f'proof("{name}", "m", "t", "s")\n# more proofs')
                     .replace('    "gap": "reason",', f'    "gap": "reason",\n    "{name}-gap": "reason",'))
        subprocess.check_call(['python3', 'scripts/css-coverage.py'], cwd=repo)

    def seed_checklist(self, layout, doc):
        (self.a / 'scripts/css-coverage.py').write_text(self.COVERAGE + layout)
        (self.a / 'docs').mkdir()
        (self.a / 'docs/ui.md').write_text(doc)
        subprocess.check_call(['python3', 'scripts/css-coverage.py'], cwd=self.a)
        self.commit(self.a, 'checklist')
        self.run_git(self.a, 'push', '-q', 'origin', 'main')
        self.run_git(self.b, 'reset', '-q', '--hard', 'main')

    def checklist_conflict(self, written_conflict=False):
        # docs/css-coverage.md is generated whole from scripts/
        # css-coverage.py's proof lists. a and b each add a proof and a
        # missing row at the same place and regenerate, so both files
        # conflict; a's edit to docs/ui.md's written intro merges cleanly.
        self.seed_checklist(self.WHOLE, '# UI\n\nintro\n\n## 8\n\nsee css-coverage.md\n\noutro\n')
        for repo, name in ((self.a, 'alpha'), (self.b, 'beta')):
            self.add_entries(repo, name)
            doc = repo / 'docs/ui.md'
            if repo == self.a:
                doc.write_text(doc.read_text().replace('intro', 'intro by a'))
            if written_conflict:
                doc.write_text(doc.read_text().replace('outro', f'outro by {name}'))
            self.commit(repo, name)
        job = self.start(self.b)
        self.ready('b')
        self.run_git(self.a, 'push', '-q', 'origin', 'main')
        (self.root / 'b.go').touch()
        return self.finish(job, ok=not written_conflict)

    def test_checklist_conflict_is_regenerated_and_rechecked(self):
        output = self.checklist_conflict()
        self.assertIn('regenerating the CSS checklist', output)
        self.assertEqual(self.calls('b'), 2)
        checklist = self.run_git(self.remote, 'show', 'main:docs/css-coverage.md')
        for row in ('| alpha | proved |', '| beta | proved |', '| alpha-gap | missing |', '| beta-gap | missing |'):
            self.assertIn(row, checklist)
        self.assertNotIn('<<<<<<<', checklist)
        self.assertIn('intro by a', self.run_git(self.remote, 'show', 'main:docs/ui.md'))
        self.assertEqual(self.run_git(self.remote, 'log', '-2', '--format=%s').splitlines(), ['beta', 'alpha'])
        self.assertEqual(self.run_git(self.b, 'status', '--porcelain'), '')

    def test_written_ui_md_conflict_stops(self):
        output = self.checklist_conflict(written_conflict=True)
        self.assertNotIn('regenerating the CSS checklist', output)
        self.assertEqual(self.run_git(self.remote, 'log', '-1', '--format=%s'), 'alpha')
        self.assertIn('docs/ui.md', self.run_git(self.b, 'diff', '--name-only', '--diff-filter=U'))

    def test_a_branch_from_before_the_move_drops_its_old_region(self):
        # b branched while the checklist was a region of docs/ui.md: it adds
        # a proof (regenerating the region), touches only the region in a
        # second commit, and edits the written outro in a third. Main then
        # moves the checklist to docs/css-coverage.md and adds a proof of
        # its own beside b's. b lands with its region edits dropped, the
        # region-only commit gone, its outro kept and the new file
        # regenerated with both proofs.
        old_doc = '# UI\n\nintro\n\n<!-- css-coverage:start -->\n<!-- css-coverage:end -->\n\noutro\n'
        self.seed_checklist(self.REGION, old_doc)
        self.add_entries(self.b, 'beta')
        self.commit(self.b, 'beta')
        doc = self.b / 'docs/ui.md'
        doc.write_text(doc.read_text().replace('| seed | proved |', '| seed | proved (touched) |'))
        self.commit(self.b, 'region only')
        doc.write_text(doc.read_text().replace('outro', 'outro by b'))
        self.commit(self.b, 'written')
        coverage = self.a / 'scripts/css-coverage.py'
        coverage.write_text(coverage.read_text().replace(self.REGION, self.WHOLE))
        (self.a / 'docs/ui.md').write_text('# UI\n\nintro\n\n## 8\n\nsee css-coverage.md\n\noutro\n')
        subprocess.check_call(['python3', 'scripts/css-coverage.py'], cwd=self.a)
        self.commit(self.a, 'move')
        self.add_entries(self.a, 'alpha')
        self.commit(self.a, 'alpha')
        self.run_git(self.a, 'push', '-q', 'origin', 'main')
        (self.root / 'b.go').touch()
        output = self.finish(self.start(self.b))
        self.assertIn('regenerating the CSS checklist', output)
        self.assertEqual(self.run_git(self.remote, 'log', '-4', '--format=%s').splitlines(),
                         ['written', 'beta', 'alpha', 'move'])
        ui = self.run_git(self.remote, 'show', 'main:docs/ui.md')
        self.assertEqual(ui, '# UI\n\nintro\n\n## 8\n\nsee css-coverage.md\n\noutro by b')
        checklist = self.run_git(self.remote, 'show', 'main:docs/css-coverage.md')
        for row in ('| alpha | proved |', '| beta | proved |', '| alpha-gap | missing |', '| beta-gap | missing |'):
            self.assertIn(row, checklist)
        self.assertEqual(self.run_git(self.b, 'status', '--porcelain'), '')

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
