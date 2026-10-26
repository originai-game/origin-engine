// A stand-in for WebGL2's EXT_disjoint_timer_query_webgl2 where the
// browser has none (Chrome on SwiftShader: a machine without a GPU), for
// the GPU pass timing of the WebGL2 tier (tests/web/gpu-timing.ts,
// tools/bench/gpu-passes.ts --timer-shim). Evaluated in the engine Worker
// before its first line (cdp.ts prepareWorkers), it gives that Worker's
// WebGL2 contexts the extension as ANGLE's Metal backend has it (Chrome on
// macOS): TIME_ELAPSED_EXT queries, a TIMESTAMP_EXT counter of 0 bits and
// no queryCounterEXT (QueryMtl::queryCounter is UNIMPLEMENTED), each
// result available only in a task after the one that ended its query, as
// WebGL requires. Its clock is the Worker's: a query measures the time from
// its beginQuery call to its endQuery call, the CPU issuing the commands
// between, so its numbers say nothing of a GPU. A context whose browser has
// the extension keeps its own. `globalThis.__og_timer_shim__` counts the
// queries begun, ended and read (and those read before available).
export const TIMER_SHIM = `(() => {
  const P = globalThis.WebGL2RenderingContext && WebGL2RenderingContext.prototype;
  if (!P) return;
  const NAME = "EXT_disjoint_timer_query_webgl2";
  const TIME_ELAPSED = 0x88bf, TIMESTAMP = 0x8e28, COUNTER_BITS = 0x8864, DISJOINT = 0x8fbb, RESULT = 0x8866, AVAILABLE = 0x8867;
  const stats = { begun: 0, ended: 0, read: 0, readUnavailable: 0, queryCounter: 0, nested: 0 };
  const ext = {
    QUERY_COUNTER_BITS_EXT: COUNTER_BITS, CURRENT_QUERY_EXT: 0x8865, QUERY_RESULT_EXT: RESULT, QUERY_RESULT_AVAILABLE_EXT: AVAILABLE,
    TIME_ELAPSED_EXT: TIME_ELAPSED, TIMESTAMP_EXT: TIMESTAMP, GPU_DISJOINT_EXT: DISJOINT,
    // ANGLE's Metal backend: UNIMPLEMENTED (QueryMtl::queryCounter).
    queryCounterEXT() { stats.queryCounter++; },
  };
  const timed = new WeakMap();
  let active = null;
  const o = { gse: P.getSupportedExtensions, ge: P.getExtension, bq: P.beginQuery, eq: P.endQuery, gqp: P.getQueryParameter, gp: P.getParameter, gq: P.getQuery };
  // A context whose browser has the extension keeps it: every call passes through.
  const own = new WeakSet();
  P.getSupportedExtensions = function () { const l = o.gse.call(this) || []; return l.includes(NAME) ? l : [...l, NAME]; };
  P.getExtension = function (n) {
    if (n !== NAME) return o.ge.call(this, n);
    const real = o.ge.call(this, n);
    if (real) own.add(this);
    return real || ext;
  };
  P.beginQuery = function (t, q) {
    if (t !== TIME_ELAPSED || own.has(this)) return o.bq.call(this, t, q);
    if (active) stats.nested++;
    stats.begun++;
    active = { start: performance.now(), ns: 0, available: false };
    timed.set(q, active);
  };
  P.endQuery = function (t) {
    if (t !== TIME_ELAPSED || own.has(this)) return o.eq.call(this, t);
    const r = active;
    active = null;
    if (!r) return;
    stats.ended++;
    r.ns = Math.max(1, Math.round((performance.now() - r.start) * 1e6));
    setTimeout(() => { r.available = true; }, 0);
  };
  P.getQueryParameter = function (q, p) {
    const r = timed.get(q);
    if (!r) return o.gqp.call(this, q, p);
    if (p === AVAILABLE) return r.available;
    if (p === RESULT) { stats.read++; if (!r.available) stats.readUnavailable++; return r.available ? r.ns : 0; }
    return null;
  };
  P.getParameter = function (p) { return p === DISJOINT && !own.has(this) ? false : o.gp.call(this, p); };
  P.getQuery = function (t, p) {
    if (p === COUNTER_BITS && !own.has(this)) return t === TIMESTAMP ? 0 : 64;
    return o.gq.call(this, t, p);
  };
  globalThis.__og_timer_shim__ = stats;
})()`;
