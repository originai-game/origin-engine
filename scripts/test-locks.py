#!/usr/bin/env python3
"""Exercise FIFO admission and dead-owner recovery without real gate/GPU work."""
import os
from pathlib import Path
import subprocess
import tempfile
import time
import unittest

LOCK = Path(__file__).resolve().with_name('lock.sh')


class Locks(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.jobs = []
        self.named = {}
        self.addCleanup(self.stop_jobs)
        self.bin = self.root / 'bin'
        self.bin.mkdir()
        query = self.bin / 'nvidia-smi'
        query.write_text('#!/bin/sh\necho 23034\n')
        query.chmod(0o755)
        self.script = self.root / 'build.sh'
        self.script.write_text('''#!/bin/sh
set -eu
. "$LOCK"
echo $$ > "$ROOT/$1.pid"
case "$2" in
  gate) og_lock_acquire "$ROOT/gate" "$1" ;;
  one) og_gpu_acquire "$1" "$BUDGET" ;;
  all) og_gpu_acquire_all "$1" ;;
  shared) og_admission_acquire shared "$1" ;;
  exclusive) og_admission_acquire exclusive "$1" ;;
  recheck) og_admission_acquire shared "landing recheck: $1" ;;
esac
echo "$1" >> "$ROOT/order"
touch "$ROOT/$1.acquired"
while [ ! -f "$ROOT/$1.release" ]; do sleep 0.05; done
''')

    @staticmethod
    def kill(job):
        # Windows: Popen.kill ends sh alone, and its sleep keeps the log open.
        if os.name == 'nt':
            subprocess.run(['taskkill', '/T', '/F', '/PID', str(job.pid)], capture_output=True)
        job.kill()

    def stop_jobs(self):
        for job in self.jobs:
            if job.poll() is None:
                self.kill(job)
            job.wait()

    def wait(self, predicate):
        deadline = time.monotonic() + 20
        while not predicate():
            if time.monotonic() > deadline:
                self.fail('timed out; logs: ' + '\n'.join(p.read_text() for p in self.root.glob('*.log')))
            time.sleep(0.02)

    def start(self, name, kind, slots=1, budget=1024):
        with (self.root / f'{name}.log').open('w') as log:
            job = subprocess.Popen(['sh', str(self.script), name, kind], stdout=log, stderr=log,
                                   env=dict(os.environ, LOCK=str(LOCK), ROOT=str(self.root),
                                            PATH=str(self.bin) + os.pathsep + os.environ['PATH'],
                                            BUDGET='' if budget is None else str(budget),
                                            OG_GPU_ROOT=str(self.root / 'gpu'),
                                            OG_GPU_SLOTS='' if slots is None else str(slots),
                                            OG_LOCK_OWNER='', OG_MEASURE_BACKOFF='0'))
        self.jobs.append(job)
        self.named[name] = job
        return job

    def pid(self, job):
        # The shell's own $$: under Git Bash it is not the Windows PID Popen knows.
        name = next(n for n, j in self.named.items() if j is job)
        self.wait(lambda: (self.root / f'{name}.pid').exists() and (self.root / f'{name}.pid').read_text().strip())
        return (self.root / f'{name}.pid').read_text().strip()

    def queued(self, job):
        pid = self.pid(job)

        def present():
            for p in self.root.rglob('*.ticket'):
                try:
                    if p.read_text().startswith(f'{pid} '):
                        return True
                except FileNotFoundError:
                    pass  # An earlier waiter acquired its lock during the scan.
            return False
        self.wait(present)

    def acquired(self, name):
        self.wait(lambda: (self.root / f'{name}.acquired').exists())

    def release(self, name, job):
        (self.root / f'{name}.release').touch()
        self.assertEqual(job.wait(timeout=10), 0)

    def test_three_waiters_arrive_fifo(self):
        for kind in ('gate', 'one'):
            with self.subTest(kind=kind):
                holder = self.start('holder', kind)
                self.acquired('holder')
                waiters = []
                for name in ('first', 'second', 'third'):
                    job = self.start(name, kind)
                    self.queued(job)
                    waiters.append((name, job))
                self.wait(lambda: 'position 3' in (self.root / 'third.log').read_text())
                self.release('holder', holder)
                for name, job in waiters:
                    self.acquired(name)
                    self.release(name, job)
                self.assertEqual((self.root / 'order').read_text().splitlines(), ['holder', 'first', 'second', 'third'])
                self.assertIn('position 3', (self.root / 'third.log').read_text())
                for p in self.root.glob('*'):
                    if p.suffix in ('.acquired', '.release', '.pid') or p.name == 'order':
                        p.unlink()

    def test_all_slots_blocks_later_single_waiters(self):
        holders = []
        for i in range(6):
            name = f'holder{i}'
            holder = self.start(name, 'one', slots=6)
            self.acquired(name)
            holders.append((name, holder))
        seventh = self.start('seventh', 'one', slots=6)
        self.queued(seventh)
        self.wait(lambda: 'waiting for 1 of 6 GiB slots' in (self.root / 'seventh.log').read_text())
        self.assertFalse((self.root / 'seventh.acquired').exists())
        self.release(*holders.pop(0))
        self.acquired('seventh')
        holders.append(('seventh', seventh))
        # Keep slot 1 occupied: later singles must wait despite five free slots.
        for name, holder in holders:
            if name != 'seventh':
                self.release(name, holder)
        exclusive = self.start('all', 'all', slots=6)
        self.queued(exclusive)
        barrier = self.root / 'gpu/gpu-all.lock/owner'
        self.wait(lambda: barrier.exists() and barrier.read_text().startswith(f'{self.pid(exclusive)} '))
        self.assertFalse((self.root / 'all.acquired').exists())
        later = self.start('later', 'one', slots=6)
        self.queued(later)
        time.sleep(2.2)
        self.assertFalse((self.root / 'later.acquired').exists(), 'later single bypassed an all-slots waiter')
        self.release('seventh', seventh)
        self.acquired('all')
        self.assertEqual(len(list((self.root / 'gpu').glob('gpu-slot-*.lock'))), 6)
        for i in range(1, 7):
            self.assertTrue((self.root / f'gpu/gpu-slot-{i}.lock/owner').read_text().startswith(f'{self.pid(exclusive)} '))
        self.assertFalse((self.root / 'later.acquired').exists())
        self.release('all', exclusive)
        self.acquired('later')
        self.release('later', later)

    def test_l4_serializes_undeclared_work_and_packs_only_declared_budgets(self):
        heavy = self.start('heavy', 'one', slots=None, budget=None)
        self.acquired('heavy')
        self.assertEqual(len(list((self.root / 'gpu').glob('gpu-slot-*.lock'))), 22)
        second = self.start('second', 'one', slots=None, budget=None)
        self.queued(second)
        self.wait(lambda: 'waiting for 22 of 22 GiB slots' in (self.root / 'second.log').read_text())
        self.assertFalse((self.root / 'second.acquired').exists())
        self.release('heavy', heavy)
        self.acquired('second')
        self.release('second', second)

        # Round UP: two 10 GiB + 1 MiB reservations exhaust 22 credits.
        first = self.start('small1', 'one', slots=None, budget=10241)
        self.acquired('small1')
        second = self.start('small2', 'one', slots=None, budget=10241)
        self.acquired('small2')
        third = self.start('small3', 'one', slots=None, budget=1025)
        self.queued(third)
        self.wait(lambda: 'waiting for 2 of 22 GiB slots' in (self.root / 'small3.log').read_text())
        self.assertFalse((self.root / 'small3.acquired').exists())
        self.release('small1', first)
        self.acquired('small3')
        self.release('small2', second)
        self.release('small3', third)

    def test_ownerless_gpu_locks_are_recovered(self):
        for name in ('gpu-all.lock', 'gpu-slot-1.lock'):
            path = self.root / 'gpu' / name
            path.mkdir(parents=True, exist_ok=True)
            os.utime(path, (1, 1))
        job = self.start('live', 'all')
        self.acquired('live')
        self.release('live', job)

    def test_measurement_drains_readers_and_blocks_new_admission(self):
        first = self.start('first', 'shared')
        self.acquired('first')
        second = self.start('second', 'shared')
        self.acquired('second')
        exclusive = self.start('measurement', 'exclusive')
        barrier = self.root / 'gpu' / 'admission' / 'og-gate.lock' / 'owner'
        self.wait(lambda: barrier.exists() and barrier.read_text().startswith(f'{self.pid(exclusive)} '))
        later = self.start('later', 'shared')
        self.queued(later)
        self.assertFalse((self.root / 'measurement.acquired').exists())
        self.release('first', first)
        self.assertFalse((self.root / 'measurement.acquired').exists())
        self.release('second', second)
        self.acquired('measurement')
        self.assertFalse((self.root / 'later.acquired').exists())
        self.release('measurement', exclusive)
        self.acquired('later')
        self.release('later', later)

    def test_measurement_yields_to_landing_recheck(self):
        # A draining measurement must not hold a landing's recheck behind
        # busy lanes; it lets the recheck in and measures after it.
        first = self.start('first', 'shared')
        self.acquired('first')
        exclusive = self.start('measurement', 'exclusive')
        barrier = self.root / 'gpu' / 'admission' / 'og-gate.lock' / 'owner'
        self.wait(lambda: barrier.exists() and barrier.read_text().startswith(f'{self.pid(exclusive)} '))
        recheck = self.start('recheck', 'recheck')
        self.acquired('recheck')
        self.release('first', first)
        self.assertFalse((self.root / 'measurement.acquired').exists())
        self.release('recheck', recheck)
        self.acquired('measurement')
        self.release('measurement', exclusive)

    def test_dead_reader_and_measurement_do_not_block_admission(self):
        reader = self.start('reader', 'shared')
        self.acquired('reader')
        self.kill(reader)
        reader.wait()
        exclusive = self.start('measurement', 'exclusive')
        self.acquired('measurement')
        self.kill(exclusive)
        exclusive.wait()
        later = self.start('later', 'shared')
        self.acquired('later')
        self.release('later', later)

    def test_dead_holder_and_waiter_release_their_turns(self):
        for kind in ('gate', 'all'):
            with self.subTest(kind=kind):
                holder = self.start('holder', kind)
                self.acquired('holder')
                dead = self.start('dead', kind)
                self.queued(dead)
                live = self.start('live', kind)
                self.queued(live)
                self.kill(dead)
                dead.wait()
                self.kill(holder)
                holder.wait()
                self.acquired('live')
                self.release('live', live)
                for p in self.root.glob('*'):
                    if p.suffix in ('.acquired', '.release', '.pid'):
                        p.unlink()


class GateTemp(unittest.TestCase):
    def test_term_removes_only_owned_scratch(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            sentinel = root / '.og/tmp/other-run'
            sentinel.mkdir(parents=True)
            job = subprocess.Popen(['sh', '-eu', '-c', '''
. "$LOCK"
og_temp_prepare "$ROOT"
echo "$TMPDIR"
while :; do sleep 0.1; done
'''], env=dict(os.environ, LOCK=str(LOCK), ROOT=str(root),
              OG_GPU_ROOT=str(root / 'gpu')), stdout=subprocess.PIPE, text=True)
            try:
                path = Path(job.stdout.readline().strip())
                self.assertTrue(path.is_dir())
                job.terminate()
                self.assertEqual(job.wait(timeout=5), 143)
                self.assertFalse(path.exists())
                self.assertTrue(sentinel.is_dir())
            finally:
                if job.poll() is None:
                    job.kill()
                    job.wait()
                job.stdout.close()

    def test_private_checkout_temp_and_cleanup(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / 'checkout with spaces'
            root.mkdir()
            env = dict(os.environ, LOCK=str(LOCK), ROOT=str(root),
                       GATE=str(Path(directory) / 'gate'),
                       OG_GPU_ROOT=str(root / 'gpu'), TMPDIR='/nonexistent-system-temp')
            result = subprocess.run(['sh', '-eu', '-c', '''
. "$LOCK"
og_temp_prepare "$ROOT"
echo "$TMPDIR"
test "$TMP" = "$TMPDIR" && test "$TEMP" = "$TMPDIR"
touch "$TMPDIR/parent"
sh -eu -c '. "$LOCK"; og_temp_prepare "$ROOT"; echo "$TMPDIR"; touch "$TMPDIR/child"'
test -f "$TMPDIR/parent"
og_lock_acquire "$GATE" test
exit 17
'''], env=env, capture_output=True, text=True)
            self.assertEqual(result.returncode, 17, result.stderr)
            parent, child = map(Path, result.stdout.splitlines())
            self.assertEqual(parent.parent, root / '.og/tmp')
            self.assertEqual(child.parent, parent.parent)
            self.assertNotEqual(parent, child)
            self.assertFalse(parent.exists())
            self.assertFalse(child.exists())
            self.assertFalse((Path(directory) / 'gate/og-gate.lock').exists())

    def test_temp_space_boundary(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for available, expected in ((2097151, 1), (2097152, 0)):
                result = subprocess.run(['sh', '-eu', '-c', '''
. "$LOCK"
df() { printf 'Filesystem blocks used available capacity mount\nvolume 9999999 0 %s 0%% /\n' "$AVAILABLE"; }
og_temp_prepare "$ROOT"
'''], env=dict(os.environ, LOCK=str(LOCK), ROOT=str(root),
              OG_GPU_ROOT=str(root / 'gpu'), AVAILABLE=str(available)),
                    capture_output=True, text=True)
                self.assertEqual(result.returncode, expected, result.stderr)
                if expected:
                    self.assertIn('at least 2 GiB required', result.stderr)
                    self.assertIn(str(root / '.og/tmp'), result.stderr)
                self.assertEqual(list((root / '.og/tmp').iterdir()), [])


if __name__ == '__main__':
    unittest.main()
