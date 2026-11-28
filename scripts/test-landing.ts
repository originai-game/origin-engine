/** Exercise real Git rebases, landing and locks with cheap gate/generator fixtures. */
import { afterEach, beforeEach, test } from "bun:test";
import assert from "node:assert/strict";
import { chmodSync, closeSync, copyFileSync, existsSync, mkdirSync, mkdtempSync, openSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, join } from "node:path";
import { execFileSync } from "node:child_process";

const ROOT = import.meta.dir;
const read = (path: string) => readFileSync(path, "utf8");
const write = (path: string, text = "") => { mkdirSync(dirname(path), { recursive: true }); writeFileSync(path, text); };
type Job = { proc: Bun.Subprocess; path: string };
class Landing {
  root = mkdtempSync(join(tmpdir(), "og-landing-"));
  remote = join(this.root, "remote"); a = join(this.root, "a"); b = join(this.root, "b");
  jobs: Job[] = [];
  env = { ...process.env, CONTROL: this.root, OG_GPU_ROOT: join(this.root, "gpu") };
  git(cwd: string, ...args: string[]): string { return execFileSync("git", ["-C", cwd, ...args], { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] }).trim(); }
  setup(): void {
    this.git(this.root, "init", "--bare", "-b", "main", this.remote);
    this.git(this.root, "init", "-b", "main", this.a);
    for (const [key, value] of [["user.email", "test@example.invalid"], ["user.name", "Landing Test"], ["commit.gpgsign", "false"]]) this.git(this.a, "config", key, value);
    write(join(this.a, ".gitignore"), "/.og/tmp/\n");
    mkdirSync(join(this.a, "scripts"));
    for (const name of ["land.sh", "lock.sh", "assets.ts"]) copyFileSync(join(ROOT, name), join(this.a, "scripts", name));
    copyFileSync(join(ROOT, "rebase-generated.ts"), join(this.a, "scripts/landing-impl.ts"));
    mkdirSync(join(this.a, "crates/og-idl/src"), { recursive: true });
    copyFileSync(join(ROOT, "../crates/og-idl/src/lib.rs"), join(this.a, "crates/og-idl/src/lib.rs"));
    write(join(this.a, "scripts/rebase-generated.ts"), `
import { rebase, git } from "./landing-impl.ts";
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
function regenerate() {
  if (existsSync("fail-generation")) throw new Error("generation failed");
  if (existsSync("inputs")) {
    const value = readdirSync("inputs").sort().map(p => readFileSync(join("inputs", p), "utf8")).join("");
    for (const name of ["packages/engine/types/index.d.ts", "packages/knowledge/wiki/search.tsv"]) {
      mkdirSync(dirname(name), {recursive:true}); writeFileSync(name, value);
    }
    git("add", "-A", "--", "packages");
  }
}
rebase(process.argv[2], regenerate);
`);
    write(join(this.a, "scripts/check.sh"), `#!/usr/bin/env bash
set -eu
name="$(basename "$PWD")"
echo gate >> "$CONTROL/$name.calls"
touch "$CONTROL/$name.ready"
while [ ! -f "$CONTROL/$name.go" ]; do sleep 0.05; done
# An allowance file admits one gate call per line.
while [ -f "$CONTROL/$name.allow" ] && [ "$(wc -l < "$CONTROL/$name.allow")" -lt "$(wc -l < "$CONTROL/$name.calls")" ]; do sleep 0.05; done
[ ! -f "$CONTROL/$name.fail" ]
[ ! -f breaks-c ] || [ ! -f scripts/c ]
`);
    chmodSync(join(this.a, "scripts/check.sh"), 0o755);
    write(join(this.a, "shared"), "start\n" + "context\n".repeat(20) + "end\n");
    this.commit(this.a, "seed");
    this.git(this.a, "remote", "add", "origin", this.remote);
    this.git(this.a, "push", "origin", "main");
    this.git(this.a, "worktree", "add", "-b", "second", this.b);
  }
  async cleanup(): Promise<void> {
    for (const { proc } of this.jobs) if (proc.exitCode === null) {
      if (process.platform === "win32") { try { execFileSync("taskkill", ["/T", "/F", "/PID", String(proc.pid)], { stdio: "ignore" }); } catch {} }
      else { try { process.kill(-proc.pid, "SIGKILL"); } catch {} }
      await proc.exited;
    }
    rmSync(this.root, { recursive: true, force: true });
  }
  commit(cwd: string, subject: string): void { this.git(cwd, "add", "."); this.git(cwd, "commit", "-m", subject); }
  start(cwd: string): Job {
    const path = join(this.root, basename(cwd) + ".log"), fd = openSync(path, "w");
    try {
      const proc = Bun.spawn(["bash", "scripts/land.sh"], { cwd, env: this.env, stdout: fd, stderr: fd, detached: true });
      const job = { proc, path }; this.jobs.push(job); return job;
    } finally { closeSync(fd); }
  }
  async wait(predicate: () => boolean, why: string): Promise<void> {
    const deadline = performance.now() + 15000;
    while (!predicate()) { assert(performance.now() < deadline, why); await Bun.sleep(50); }
  }
  ready(name: string): Promise<void> { return this.wait(() => existsSync(join(this.root, name + ".ready")), `${name} never reached gate`); }
  async finish({ proc, path }: Job, ok = true): Promise<string> {
    let timer: ReturnType<typeof setTimeout> | undefined;
    try {
      const rc = await Promise.race([proc.exited, new Promise<never>((_, reject) => { timer = setTimeout(() => reject(new Error("landing timed out: " + read(path))), 30000); })]);
      const output = read(path); assert.equal(rc === 0, ok, output); return output;
    } finally { clearTimeout(timer); }
  }
  calls(name: string): number { const path = join(this.root, name + ".calls"); return existsSync(path) ? read(path).trimEnd().split("\n").length : 0; }
  async race(common = false, overlap = false): Promise<void> {
    if (overlap) {
      write(join(this.a, "shared"), read(join(this.a, "shared")).replace("start", "first"));
      write(join(this.b, "shared"), read(join(this.b, "shared")).replace("end", "second"));
    } else {
      write(join(this.a, common ? "Cargo.toml" : "docs/first.md"), "first\n");
      write(join(this.b, "docs/second.md"), "second\n");
    }
    this.commit(this.a, "first"); this.commit(this.b, "second");
    const a = this.start(this.a), b = this.start(this.b);
    await this.ready("a"); await this.ready("b");
    write(join(this.root, "a.go")); await this.finish(a);
    write(join(this.root, "b.go")); const output = await this.finish(b);
    const calls = this.calls("b");
    assert.equal(calls, common || overlap ? 2 : 1, output);
    assert.deepEqual(this.git(this.remote, "log", "-2", "--format=%s").split("\n"), ["second", "first"]);
    await this.finish(this.start(this.b)); assert.equal(this.calls("b"), calls);
    assert(!existsSync(join(this.a, ".git/og-landing/og-gate.lock")));
  }
  test_disjoint_race() { return this.race(); }
  test_shared_input_race() { return this.race(true); }
  test_overlapping_file_race() { return this.race(false, true); }
  async test_queued_landers_recheck_at_most_once(): Promise<void> {
    const c = join(this.root, "c"); this.git(this.a, "worktree", "add", "-b", "third", c);
    write(join(this.a, "Cargo.toml"), "first\n"); write(join(this.b, "scripts/second"), "second\n"); write(join(c, "scripts/third"), "third\n");
    for (const [repo, subject] of [[this.a, "first"], [this.b, "second"], [c, "third"]]) this.commit(repo, subject);
    for (const name of ["b", "c"]) write(join(this.root, name + ".allow"), "1\n");
    const jobs = { a: this.start(this.a), b: this.start(this.b), c: this.start(c) };
    for (const name of Object.keys(jobs)) await this.ready(name);
    write(join(this.root, "a.go")); await this.finish(jobs.a);
    write(join(this.root, "b.go")); await this.wait(() => this.calls("b") >= 2, "b never rechecked");
    write(join(this.root, "c.go")); await Bun.sleep(1000);
    for (const name of ["b", "c"]) write(join(this.root, name + ".allow"), "1\n".repeat(9));
    await this.finish(jobs.b); assert((await this.finish(jobs.c)).includes("landed in a batch"));
    assert.deepEqual([this.calls("b"), this.calls("c")], [3, 1]);
    assert.deepEqual(this.git(this.remote, "log", "-3", "--format=%s").split("\n").sort(), ["first", "second", "third"]);
  }
  async drain(breaker = false, outside = false): Promise<Record<string, string>> {
    const repos: Record<string, string> = { b: this.b };
    for (const name of ["c", "d"]) { repos[name] = join(this.root, name); this.git(this.a, "worktree", "add", "-b", name, repos[name]); }
    write(join(this.a, "Cargo.toml"), "upstream\n"); this.commit(this.a, "upstream");
    for (const [name, repo] of Object.entries(repos)) {
      write(join(repo, "scripts", name), name + "\n");
      if (breaker && name === "d") write(join(repo, "breaks-c"), "d\n");
      this.commit(repo, name);
    }
    write(join(this.root, "b.allow"), "1\n");
    const jobs = Object.fromEntries(Object.entries(repos).map(([name, repo]) => [name, this.start(repo)]));
    for (const name of Object.keys(jobs)) await this.ready(name);
    this.git(this.a, "push", "origin", "main");
    if (outside) this.outside(null, ["outside", "o"]);
    write(join(this.root, "b.go")); await this.wait(() => this.calls("b") >= 2, "b never rechecked");
    const offers = join(this.a, ".git/og-landing/batch");
    for (const [index, name] of ["c", "d"].entries()) {
      write(join(this.root, name + ".go")); await this.wait(() => readdirSync(offers).length >= index + 1, `${name} never offered`);
      await Bun.sleep(1100);
    }
    write(join(this.root, "b.allow"), "1\n".repeat(9)); await this.finish(jobs.b);
    const out: Record<string, string> = {};
    for (const name of ["c", "d"]) out[name] = await this.finish(jobs[name], !(breaker && name === "d"));
    assert(out.c.includes("landed in a batch")); assert.deepEqual([this.calls("c"), this.calls("d")], [1, 1]);
    assert.equal(this.git(repos.c, "rev-parse", "HEAD"), this.git(this.remote, "rev-parse", breaker ? "main" : "main~1"));
    assert.equal(this.git(repos.b, "symbolic-ref", "--short", "HEAD"), "second"); return out;
  }
  async test_holder_drains_the_queue_as_one_batch() {
    assert((await this.drain()).d.includes("landed in a batch")); assert.equal(this.calls("b"), 3);
    assert.deepEqual(this.git(this.remote, "log", "-4", "--format=%s").split("\n"), ["d", "c", "b", "upstream"]);
  }
  async test_failing_head_goes_back_and_the_passing_prefix_lands() {
    assert((await this.drain(true)).d.includes("failed quick")); assert.equal(this.calls("b"), 4);
    assert.deepEqual(this.git(this.remote, "log", "-3", "--format=%s").split("\n"), ["c", "b", "upstream"]);
    assert.deepEqual(readdirSync(join(this.a, ".git/og-landing/batch")), []);
  }
  outside(...writes: ([string, string] | null)[]): void {
    const o = join(this.root, "o"); this.git(this.root, "clone", "-q", this.remote, o);
    for (const [key, value] of [["user.email", "o@example.invalid"], ["user.name", "Outside"], ["commit.gpgsign", "false"]]) this.git(o, "config", key, value);
    write(join(this.root, "outside"), writes.map(w => w === null ? "-\n" : `${w[0]} ${w[1]}\n`).join(""));
    const hooks = join(this.root, "hooks"); mkdirSync(hooks);
    write(join(hooks, "pre-push"), `#!/usr/bin/env bash
set -e
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_PREFIX
q="$CONTROL/outside"
[ -s "$q" ] || exit 0
line="$(head -n 1 "$q")"
tail -n +2 "$q" > "$q.next"; mv "$q.next" "$q"
[ "$line" != - ] || exit 0
n="$(cat "$CONTROL/outside.n" 2>/dev/null || echo 0)"; echo $((n + 1)) > "$CONTROL/outside.n"
cd "$CONTROL/o"
git fetch -q origin && git checkout -q -B w origin/main
mkdir -p "$(dirname "\${line%% *}")"; echo "\${line#* }" > "\${line%% *}"
git add -A && git commit -qm "outside $n" && git push -q origin HEAD:refs/heads/main
`);
    chmodSync(join(hooks, "pre-push"), 0o755); this.git(this.a, "config", "core.hooksPath", hooks);
  }
  async outsideRace(writes: [string, string][], change = ["first", "first\n"], ok = true): Promise<[string, string[]]> {
    write(join(this.a, change[0]), change[1]); this.commit(this.a, "first"); this.outside(...writes);
    write(join(this.root, "a.go")); const output = await this.finish(this.start(this.a), ok);
    return [output, this.git(this.remote, "log", "--format=%s", "main").split("\n")];
  }
  async test_outside_writer_disjoint_lands_without_a_recheck() {
    const [output, log] = await this.outsideRace([["docs/outside.md", "o"], ["docs/outside2.md", "o2"]], ["docs/first.md", "first\n"]);
    assert.equal(output.split("pushing without a recheck").length - 1, 2, output); assert.equal(this.calls("a"), 1);
    assert.deepEqual(log.slice(0, 3), ["first", "outside 1", "outside 0"]);
    assert.equal(this.git(this.a, "rev-parse", "main"), this.git(this.remote, "rev-parse", "main"));
  }
  async test_outside_writer_on_shared_inputs_rechecks_then_lands() {
    const [output, log] = await this.outsideRace([["Cargo.toml", "o"]]);
    assert(output.includes("rechecking affected quick steps")); assert.equal(this.calls("a"), 2); assert.deepEqual(log.slice(0, 2), ["first", "outside 0"]);
  }
  async test_outside_writer_failing_recheck_never_pushes() {
    const [, log] = await this.outsideRace([["scripts/c", "c"]], ["breaks-c", "a\n"], false);
    assert.equal(this.calls("a"), 2); assert.equal(log[0], "outside 0");
  }
  async test_outside_writer_conflict_stops() {
    const [output, log] = await this.outsideRace([["shared", "outside"]], ["shared", "first\n"], false);
    assert(output.includes("conflicts with the new origin/main")); assert.equal(log[0], "outside 0");
  }
  async test_rejected_push_without_a_remote_move_stops() {
    write(join(this.a, "first"), "first\n"); this.commit(this.a, "first");
    const hooks = join(this.root, "hooks"); mkdirSync(hooks);
    write(join(hooks, "pre-push"), '#!/usr/bin/env bash\necho "push rejected by fixture" >&2\nexit 1\n'); chmodSync(join(hooks, "pre-push"), 0o755);
    this.git(this.a, "config", "core.hooksPath", hooks); write(join(this.root, "a.go"));
    const output = await this.finish(this.start(this.a), false);
    assert(output.includes("origin/main did not move")); assert.equal(output.split("push rejected by fixture").length - 1, 1, output);
    assert.equal(this.calls("a"), 1); assert.equal(this.git(this.remote, "log", "-1", "--format=%s"), "seed");
  }
  async test_outside_writer_during_a_drain_lands_the_stack() {
    await this.drain(false, true); assert.equal(this.calls("b"), 4);
    assert.deepEqual(this.git(this.remote, "log", "-5", "--format=%s").split("\n"), ["d", "c", "outside 0", "b", "upstream"]);
  }
  async test_failed_gate_never_pushes() {
    write(join(this.a, "first"), "first"); this.commit(this.a, "first"); write(join(this.root, "a.go")); write(join(this.root, "a.fail"));
    await this.finish(this.start(this.a), false); assert.equal(this.git(this.remote, "log", "-1", "--format=%s"), "seed");
  }
  async generatedConflict({ race = false, sourceConflict = false, fail = false, laterGenerated = true } = {}) {
    const paths = ["packages/engine/types/index.d.ts", "packages/knowledge/wiki/search.tsv"];
    for (const [repo, value] of [[this.a, "upstream"], [this.b, "branch"]]) {
      write(join(repo, "inputs", value), value + "\n");
      for (const path of paths) write(join(repo, path), value);
      if (sourceConflict) write(join(repo, "shared"), value);
      this.commit(repo, value);
    }
    write(join(this.b, "inputs/later"), "later\n");
    if (laterGenerated) write(join(this.b, paths[0]), "later");
    if (fail) write(join(this.b, "fail-generation"));
    this.commit(this.b, "later");
    let job: Job | undefined;
    if (race) { job = this.start(this.b); await this.ready("b"); }
    this.git(this.a, "push", "origin", "main"); write(join(this.root, "b.go"));
    const output = await this.finish(job ?? this.start(this.b), !(sourceConflict || fail));
    if (sourceConflict || fail) {
      assert.equal(this.git(this.remote, "log", "-1", "--format=%s"), "upstream"); assert(this.git(this.b, "diff", "--name-only", "--diff-filter=U"));
    } else {
      assert(output.includes("regenerating generated-only conflicts"));
      for (const path of paths) assert.equal(this.git(this.remote, "show", "main:" + path), "branch\nlater\nupstream");
      assert.equal(this.git(this.b, "status", "--porcelain"), "");
    }
  }
  test_generated_conflict() { return this.generatedConflict(); }
  test_generated_conflict_after_gate() { return this.generatedConflict({ race: true }); }
  test_clean_later_commit_is_regenerated() { return this.generatedConflict({ laterGenerated: false }); }
  test_mixed_conflict_stops() { return this.generatedConflict({ sourceConflict: true }); }
  test_generation_failure_stops() { return this.generatedConflict({ fail: true }); }
  async test_handwritten_declarations_stop() {
    for (const [repo, value] of [[this.a, "upstream"], [this.b, "branch"]]) { write(join(repo, "manual.d.ts"), value); this.commit(repo, value); }
    this.git(this.a, "push", "origin", "main"); const output = await this.finish(this.start(this.b), false);
    assert(!output.includes("regenerating generated-only conflicts")); assert.equal(this.git(this.remote, "log", "-1", "--format=%s"), "upstream");
    assert.equal(this.git(this.b, "diff", "--name-only", "--diff-filter=U"), "manual.d.ts");
  }
}
let fixture: Landing;
beforeEach(() => { fixture = new Landing(); fixture.setup(); });
afterEach(async () => { await fixture.cleanup(); });
for (const name of Object.getOwnPropertyNames(Landing.prototype).filter(n => n.startsWith("test_"))) {
  test(name.slice(5).replaceAll("_", " "), () => (fixture[name as keyof Landing] as () => Promise<void>).call(fixture), 60000);
}
