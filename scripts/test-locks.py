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
        self.script = self.root / 'build.sh'
        self.script.write_text('''#!/bin/sh
set -eu
. "$LOCK"
echo $$ > "$ROOT/$1.pid"
case "$2" in
  gate) og_lock_acquire "$ROOT/gate" "$1" ;;
  one) og_gpu_acquire "$1" ;;
  all) og_gpu_acquire_all "$1" ;;
  shared) og_admission_acquire shared "$1" ;;
  exclusive) og_admission_acquire exclusive "$1" ;;
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

    def start(self, name, kind, slots=1):
        with (self.root / f'{name}.log').open('w') as log:
            job = subprocess.Popen(['sh', str(self.script), name, kind], stdout=log, stderr=log,
                                   env=dict(os.environ, LOCK=str(LOCK), ROOT=str(self.root),
                                            OG_GPU_ROOT=str(self.root / 'gpu'), OG_GPU_SLOTS=str(slots),
                                            OG_LOCK_OWNER=''))
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
        holder = self.start('holder', 'one', slots=2)
        self.acquired('holder')
        exclusive = self.start('all', 'all', slots=2)
        self.queued(exclusive)
        later = self.start('later', 'one', slots=2)
        self.queued(later)
        time.sleep(2.2)
        self.assertFalse((self.root / 'later.acquired').exists(), 'later single bypassed an all-slots waiter')
        self.release('holder', holder)
        self.acquired('all')
        self.assertFalse((self.root / 'later.acquired').exists())
        self.release('all', exclusive)
        self.acquired('later')
        self.release('later', later)

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


if __name__ == '__main__':
    unittest.main()
