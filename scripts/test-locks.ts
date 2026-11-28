/** Exercise FIFO admission and dead-owner recovery without real gate/GPU work. */
import { afterEach, beforeEach, describe, test } from "bun:test";
import assert from "node:assert/strict";
import { chmodSync, closeSync, existsSync, mkdirSync, mkdtempSync, openSync, readFileSync, readdirSync, rmSync, utimesSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { delimiter, join } from "node:path";
import { spawnSync } from "node:child_process";

const LOCK = join(import.meta.dir, "lock.sh");
const read = (p: string) => readFileSync(p, "utf8");
const touch = (p: string) => writeFileSync(p, "");
const lines = (p: string) => read(p).trimEnd().split("\n");
function kill(job: Bun.Subprocess): void {
  if (process.platform === "win32") spawnSync("taskkill", ["/T", "/F", "/PID", String(job.pid)], { stdio: "ignore" });
  else job.kill("SIGKILL");
}
class Locks {
  root = mkdtempSync(join(tmpdir(), "og-locks-"));
  bin = join(this.root, "bin"); script = join(this.root, "build.sh");
  jobs: Bun.Subprocess[] = []; named = new Map<string, Bun.Subprocess>();
  setup(): void {
    mkdirSync(this.bin);
    writeFileSync(join(this.bin, "nvidia-smi"), "#!/bin/sh\necho 23034\n"); chmodSync(join(this.bin, "nvidia-smi"), 0o755);
    writeFileSync(this.script, `#!/bin/sh
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
`);
  }
  async cleanup(): Promise<void> {
    for (const job of this.jobs) { if (job.exitCode === null) kill(job); await job.exited; }
    rmSync(this.root, { recursive: true, force: true });
  }
  async wait(predicate: () => unknown): Promise<void> {
    const deadline = performance.now() + 20000;
    while (!predicate()) {
      assert(performance.now() < deadline, "timed out; logs: " + readdirSync(this.root).filter(p => p.endsWith(".log")).map(p => read(join(this.root, p))).join("\n"));
      await Bun.sleep(20);
    }
  }
  start(name: string, kind: string, slots: number | null = 1, budget: number | null = 1024): Bun.Subprocess {
    const fd = openSync(join(this.root, `${name}.log`), "w");
    try {
      const job = Bun.spawn(["sh", this.script, name, kind], { stdout: fd, stderr: fd, env: { ...process.env,
        LOCK, ROOT: this.root, PATH: this.bin + delimiter + process.env.PATH, BUDGET: budget === null ? "" : String(budget),
        OG_GPU_ROOT: join(this.root, "gpu"), OG_GPU_SLOTS: slots === null ? "" : String(slots), OG_LOCK_OWNER: "", OG_MEASURE_BACKOFF: "0" } });
      this.jobs.push(job); this.named.set(name, job); return job;
    } finally { closeSync(fd); }
  }
  async pid(job: Bun.Subprocess): Promise<string> {
    const name = [...this.named].find(([, j]) => j === job)![0], path = join(this.root, name + ".pid");
    await this.wait(() => existsSync(path) && read(path).trim()); return read(path).trim();
  }
  async queued(job: Bun.Subprocess): Promise<void> {
    const pid = await this.pid(job);
    await this.wait(() => readdirSync(this.root, { recursive: true, withFileTypes: true }).some(e => {
      try { return e.isFile() && e.name.endsWith(".ticket") && read(join(e.parentPath, e.name)).startsWith(pid + " "); }
      catch (e) { if ((e as NodeJS.ErrnoException).code === "ENOENT") return false; throw e; }
    }));
  }
  acquired(name: string) { return this.wait(() => existsSync(join(this.root, name + ".acquired"))); }
  async release(name: string, job: Bun.Subprocess): Promise<void> {
    touch(join(this.root, name + ".release"));
    let timer: ReturnType<typeof setTimeout> | undefined;
    try { assert.equal(await Promise.race([job.exited, new Promise<never>((_, reject) => { timer = setTimeout(() => reject(new Error(`${name} did not exit`)), 10000); })]), 0); }
    finally { clearTimeout(timer); }
  }
  async test_three_waiters_arrive_fifo() {
    for (const kind of ["gate", "one"]) {
      const holder = this.start("holder", kind); await this.acquired("holder");
      const waiters: [string, Bun.Subprocess][] = [];
      for (const name of ["first", "second", "third"]) { const job = this.start(name, kind); await this.queued(job); waiters.push([name, job]); }
      await this.wait(() => read(join(this.root, "third.log")).includes("position 3"));
      await this.release("holder", holder);
      for (const [name, job] of waiters) { await this.acquired(name); await this.release(name, job); }
      assert.deepEqual(lines(join(this.root, "order")), ["holder", "first", "second", "third"]);
      assert(read(join(this.root, "third.log")).includes("position 3"));
      for (const p of readdirSync(this.root)) if (/\.(acquired|release|pid)$/.test(p) || p === "order") rmSync(join(this.root, p));
    }
  }
  async test_all_slots_blocks_later_single_waiters() {
    const holders: [string, Bun.Subprocess][] = [];
    for (let i = 0; i < 6; i++) { const name = `holder${i}`, job = this.start(name, "one", 6); await this.acquired(name); holders.push([name, job]); }
    const seventh = this.start("seventh", "one", 6); await this.queued(seventh);
    await this.wait(() => read(join(this.root, "seventh.log")).includes("waiting for 1 of 6 GiB slots"));
    assert(!existsSync(join(this.root, "seventh.acquired")));
    await this.release(...holders.shift()!); await this.acquired("seventh"); holders.push(["seventh", seventh]);
    for (const [name, job] of holders) if (name !== "seventh") await this.release(name, job);
    const exclusive = this.start("all", "all", 6); await this.queued(exclusive);
    const barrier = join(this.root, "gpu/gpu-all.lock/owner"), pid = await this.pid(exclusive);
    await this.wait(() => existsSync(barrier) && read(barrier).startsWith(pid + " "));
    assert(!existsSync(join(this.root, "all.acquired")));
    const later = this.start("later", "one", 6); await this.queued(later); await Bun.sleep(2200);
    assert(!existsSync(join(this.root, "later.acquired")), "later single bypassed an all-slots waiter");
    await this.release("seventh", seventh); await this.acquired("all");
    assert.equal(readdirSync(join(this.root, "gpu")).filter(n => /^gpu-slot-.*\.lock$/.test(n)).length, 6);
    for (let i = 1; i <= 6; i++) assert(read(join(this.root, `gpu/gpu-slot-${i}.lock/owner`)).startsWith(pid + " "));
    assert(!existsSync(join(this.root, "later.acquired")));
    await this.release("all", exclusive); await this.acquired("later"); await this.release("later", later);
  }
  async test_l4_serializes_undeclared_work_and_packs_only_declared_budgets() {
    const heavy = this.start("heavy", "one", null, null); await this.acquired("heavy");
    assert.equal(readdirSync(join(this.root, "gpu")).filter(n => /^gpu-slot-.*\.lock$/.test(n)).length, 22);
    const second = this.start("second", "one", null, null); await this.queued(second);
    await this.wait(() => read(join(this.root, "second.log")).includes("waiting for 22 of 22 GiB slots"));
    assert(!existsSync(join(this.root, "second.acquired")));
    await this.release("heavy", heavy); await this.acquired("second"); await this.release("second", second);
    // Round UP: two 10 GiB + 1 MiB reservations exhaust 22 credits.
    const a = this.start("small1", "one", null, 10241); await this.acquired("small1");
    const b = this.start("small2", "one", null, 10241); await this.acquired("small2");
    const c = this.start("small3", "one", null, 1025); await this.queued(c);
    await this.wait(() => read(join(this.root, "small3.log")).includes("waiting for 2 of 22 GiB slots"));
    assert(!existsSync(join(this.root, "small3.acquired")));
    await this.release("small1", a); await this.acquired("small3"); await this.release("small2", b); await this.release("small3", c);
  }
  async test_ownerless_gpu_locks_are_recovered() {
    for (const name of ["gpu-all.lock", "gpu-slot-1.lock"]) { const path = join(this.root, "gpu", name); mkdirSync(path, { recursive: true }); utimesSync(path, 1, 1); }
    const job = this.start("live", "all"); await this.acquired("live"); await this.release("live", job);
  }
  async test_measurement_drains_readers_and_blocks_new_admission() {
    const first = this.start("first", "shared"); await this.acquired("first");
    const second = this.start("second", "shared"); await this.acquired("second");
    const exclusive = this.start("measurement", "exclusive"), pid = await this.pid(exclusive);
    const barrier = join(this.root, "gpu/admission/og-gate.lock/owner");
    await this.wait(() => existsSync(barrier) && read(barrier).startsWith(pid + " "));
    const later = this.start("later", "shared"); await this.queued(later);
    assert(!existsSync(join(this.root, "measurement.acquired")));
    await this.release("first", first); assert(!existsSync(join(this.root, "measurement.acquired")));
    await this.release("second", second); await this.acquired("measurement"); assert(!existsSync(join(this.root, "later.acquired")));
    await this.release("measurement", exclusive); await this.acquired("later"); await this.release("later", later);
  }
  async test_measurement_yields_to_landing_recheck() {
    const first = this.start("first", "shared"); await this.acquired("first");
    const exclusive = this.start("measurement", "exclusive"), pid = await this.pid(exclusive);
    const barrier = join(this.root, "gpu/admission/og-gate.lock/owner");
    await this.wait(() => existsSync(barrier) && read(barrier).startsWith(pid + " "));
    const recheck = this.start("recheck", "recheck"); await this.acquired("recheck");
    await this.release("first", first); assert(!existsSync(join(this.root, "measurement.acquired")));
    await this.release("recheck", recheck); await this.acquired("measurement"); await this.release("measurement", exclusive);
  }
  async test_dead_reader_and_measurement_do_not_block_admission() {
    const reader = this.start("reader", "shared"); await this.acquired("reader"); kill(reader); await reader.exited;
    const exclusive = this.start("measurement", "exclusive"); await this.acquired("measurement"); kill(exclusive); await exclusive.exited;
    const later = this.start("later", "shared"); await this.acquired("later"); await this.release("later", later);
  }
  async test_dead_holder_and_waiter_release_their_turns() {
    for (const kind of ["gate", "all"]) {
      const holder = this.start("holder", kind); await this.acquired("holder");
      const dead = this.start("dead", kind); await this.queued(dead);
      const live = this.start("live", kind); await this.queued(live);
      kill(dead); await dead.exited; kill(holder); await holder.exited;
      await this.acquired("live"); await this.release("live", live);
      for (const p of readdirSync(this.root)) if (/\.(acquired|release|pid)$/.test(p)) rmSync(join(this.root, p));
    }
  }
}
describe("locks", () => {
  let fixture: Locks;
  beforeEach(() => { fixture = new Locks(); fixture.setup(); });
  afterEach(async () => { await fixture.cleanup(); });
  for (const name of Object.getOwnPropertyNames(Locks.prototype).filter(n => n.startsWith("test_"))) {
    test(name.slice(5).replaceAll("_", " "), () => (fixture[name as keyof Locks] as () => Promise<void>).call(fixture), 60000);
  }
});
describe("gate scratch", () => {
  let root: string;
  beforeEach(() => { root = mkdtempSync(join(tmpdir(), "og-gate-temp-")); });
  afterEach(() => { rmSync(root, { recursive: true, force: true }); });
  test("TERM removes only owned scratch", async () => {
    const sentinel = join(root, ".og/tmp/other-run"); mkdirSync(sentinel, { recursive: true });
    const job = Bun.spawn(["sh", "-eu", "-c", `
. "$LOCK"
og_temp_prepare "$ROOT"
echo "$TMPDIR"
while :; do sleep 0.1; done
`], { env: { ...process.env, LOCK, ROOT: root, OG_GPU_ROOT: join(root, "gpu") }, stdout: "pipe" });
    try {
      const reader = job.stdout.getReader();
      const { value } = await reader.read(); const path = new TextDecoder().decode(value).trim(); reader.releaseLock();
      assert(existsSync(path)); job.kill("SIGTERM");
      assert.equal(await job.exited, 143); assert(!existsSync(path)); assert(existsSync(sentinel));
    } finally { if (job.exitCode === null) { kill(job); await job.exited; } }
  });
  test("private checkout temp and cleanup", () => {
    const checkout = join(root, "checkout with spaces"); mkdirSync(checkout);
    const result = spawnSync("sh", ["-eu", "-c", `
. "$LOCK"
og_temp_prepare "$ROOT"
echo "$TMPDIR"
test "$TMP" = "$TMPDIR" && test "$TEMP" = "$TMPDIR"
touch "$TMPDIR/parent"
sh -eu -c '. "$LOCK"; og_temp_prepare "$ROOT"; echo "$TMPDIR"; touch "$TMPDIR/child"'
test -f "$TMPDIR/parent"
og_lock_acquire "$GATE" test
exit 17
`], { encoding: "utf8", env: { ...process.env, LOCK, ROOT: checkout, GATE: join(root, "gate"), OG_GPU_ROOT: join(checkout, "gpu"), TMPDIR: "/nonexistent-system-temp" } });
    assert.equal(result.status, 17, result.stderr);
    const [parent, child] = result.stdout.trim().split("\n");
    for (const path of [parent, child]) { assert.equal(join(path, ".."), join(checkout, ".og/tmp")); assert(!existsSync(path)); }
    assert.notEqual(parent, child); assert(!existsSync(join(root, "gate/og-gate.lock")));
  });
  test("temp space boundary", () => {
    for (const [available, expected] of [[2097151, 1], [2097152, 0]]) {
      const result = spawnSync("sh", ["-eu", "-c", `
. "$LOCK"
df() { printf 'Filesystem blocks used available capacity mount\\nvolume 9999999 0 %s 0%% /\\n' "$AVAILABLE"; }
og_temp_prepare "$ROOT"
`], { encoding: "utf8", env: { ...process.env, LOCK, ROOT: root, OG_GPU_ROOT: join(root, "gpu"), AVAILABLE: String(available) } });
      assert.equal(result.status, expected, result.stderr);
      if (expected) { assert(result.stderr.includes("at least 2 GiB required")); assert(result.stderr.includes(join(root, ".og/tmp"))); }
      assert.deepEqual(readdirSync(join(root, ".og/tmp")), []);
    }
  });
});
