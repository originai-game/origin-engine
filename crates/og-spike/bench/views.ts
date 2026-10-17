// Spike §10.3: a TypeScript system over typed views. The engine would hand a system
// Float32Array views over SoA component columns in wasm memory; the per-tick cost that
// matters is the JS loop over those views. Here the columns live in one ArrayBuffer
// (a WebAssembly.Memory when available, so the backing store is the same kind).
// Measures: (a) an integrate system over 10k entities (position += velocity * dt),
// (b) the same through a per-entity accessor API (what we must NOT do), (c) 100k.
const N = Number(process.argv[2] ?? 10000);
const TICKS = 2000;
const mem = typeof WebAssembly !== "undefined" ? new WebAssembly.Memory({ initial: 256 }) : null;
const buf = mem ? mem.buffer : new ArrayBuffer(16 * 1024 * 1024);
const alloc = (() => { let off = 0; return (bytes: number) => { const o = off; off += (bytes + 15) & ~15; return o; }; })();
const posOff = alloc(N * 3 * 4), velOff = alloc(N * 3 * 4), hpOff = alloc(N * 4);
const pos = new Float32Array(buf, posOff, N * 3), vel = new Float32Array(buf, velOff, N * 3), hp = new Float32Array(buf, hpOff, N);
for (let i = 0; i < N; i++) { vel[i * 3] = (i % 7) * 0.1; vel[i * 3 + 1] = 0.01; vel[i * 3 + 2] = -(i % 5) * 0.1; hp[i] = 100; }

function integrate(p: Float32Array, v: Float32Array, n: number, dt: number) {
  for (let i = 0; i < n * 3; i += 3) { p[i] += v[i] * dt; p[i + 1] += v[i + 1] * dt; p[i + 2] += v[i + 2] * dt; }
}
// A second system with a branch and two components, closer to gameplay code.
function damageBelowGround(p: Float32Array, h: Float32Array, n: number) {
  let dead = 0;
  for (let i = 0; i < n; i++) { if (p[i * 3 + 1] < -1) { h[i] -= 1; if (h[i] <= 0) dead++; } }
  return dead;
}
// The anti-pattern: a per-entity object accessor (allocation + boundary per entity).
const entity = (i: number) => ({ get x() { return pos[i * 3]; }, set x(v: number) { pos[i * 3] = v; }, get vx() { return vel[i * 3]; } });
function integrateViaAccessors(n: number, dt: number) { for (let i = 0; i < n; i++) { const e = entity(i); e.x = e.x + e.vx * dt; } }

function bench(label: string, fn: () => void) {
  for (let i = 0; i < 100; i++) fn();
  const t0 = performance.now();
  for (let i = 0; i < TICKS; i++) fn();
  const ms = (performance.now() - t0) / TICKS;
  console.log(`${label.padEnd(44)} ${ms.toFixed(4)} ms/tick`);
  return ms;
}
console.log(`entities=${N} ticks=${TICKS} runtime=${typeof Bun !== "undefined" ? "bun " + Bun.version : "node " + process.version} memory=${mem ? "WebAssembly.Memory" : "ArrayBuffer"}`);
const a = bench("integrate (typed views, batch)", () => integrate(pos, vel, N, 1 / 60));
const b = bench("integrate + damage (two systems)", () => { integrate(pos, vel, N, 1 / 60); damageBelowGround(pos, hp, N); });
const c = bench("integrate via per-entity accessors", () => integrateViaAccessors(N, 1 / 60));
console.log(JSON.stringify({ entities: N, batchMs: +a.toFixed(4), twoSystemsMs: +b.toFixed(4), accessorMs: +c.toFixed(4) }));
