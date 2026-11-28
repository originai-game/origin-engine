// One transport policy for the web host, including synchronous split-wasm
// placeholders. Plain JS so the unbundled rayon pool imports this same code.
// Four asynchronous bodies plus one reserved synchronous body stay below
// Stage's six per-thread streams, leaving one for browser-owned scripts or
// serial offline precaching.
// The reservation matters: a blocking placeholder must never wait for an
// asynchronous body whose reader runs on its own blocked Worker.
export const FETCH_CONCURRENCY = 4;
export const FETCH_DEADLINE_MS = 300_000;
const ATTEMPTS = 6;
let shared;
let active = 0;

export function fetchState(state) {
  if (state) shared = new Int32Array(state);
  else if (!shared && typeof SharedArrayBuffer === "function") shared = new Int32Array(new SharedArrayBuffer(8));
  return shared?.buffer;
}

function enter(sync) {
  fetchState();
  if (!shared) {
    if (!sync && active >= FETCH_CONCURRENCY) return false;
    if (!sync) active++;
    return true;
  }
  const index = sync ? 1 : 0;
  const limit = sync ? 1 : FETCH_CONCURRENCY;
  const n = Atomics.load(shared, index);
  return n < limit && Atomics.compareExchange(shared, index, n, n + 1) === n;
}

function leave(sync) {
  if (shared) {
    Atomics.sub(shared, sync ? 1 : 0, 1);
    Atomics.notify(shared, sync ? 1 : 0);
  } else if (!sync) active--;
}

function delay(attempt, retryAfter) {
  const seconds = Number(retryAfter);
  const date = Date.parse(retryAfter);
  const asked = retryAfter === null ? 0 : Number.isFinite(seconds) ? seconds * 1000 : Number.isFinite(date) ? date - Date.now() : 0;
  return Math.max(Math.min(8000, 500 * 2 ** attempt), asked);
}

function retry(status) {
  return status === 0 || status === 429 || status === 503 || status >= 500;
}

function failure(url, status, attempt, cause) {
  return Object.assign(new Error(`cannot fetch ${url}: ${status ? `HTTP ${status}` : "network error"} after ${attempt + 1} attempt(s)${cause ? ` (${String(cause)})` : ""}`), { status });
}

function pause(ms, signal) {
  return new Promise((resolve, reject) => {
    signal.throwIfAborted();
    const abort = () => { clearTimeout(timer); reject(signal.reason); };
    const timer = setTimeout(() => { signal.removeEventListener("abort", abort); resolve(); }, ms);
    signal.addEventListener("abort", abort, { once: true });
  });
}

/** A complete body: the slot is not returned at headers, nor before a failed body's cancellation. */
export async function fetchResource(url, init = {}, progress) {
  const source = url instanceof Request ? url : String(url).replaceAll("#", "%23");
  url = source instanceof Request ? source.url : source;
  // Browser SRI can hide a 429's status and Retry-After behind TypeError.
  // Verify successful bytes here, before any consumer can see them.
  const { integrity = source instanceof Request ? source.integrity : undefined, ...options } = init;
  const deadline = AbortSignal.timeout(FETCH_DEADLINE_MS);
  const aborted = init.signal ?? (source instanceof Request ? source.signal : undefined);
  const signal = aborted ? AbortSignal.any([aborted, deadline]) : deadline;
  let status = 0;
  let cause;
  let attempt = 0;
  try {
    for (; attempt < ATTEMPTS; attempt++) {
      while (!enter(false)) await pause(10, signal);
      let response;
      let retryAfter = null;
      let invalidIntegrity = false;
      status = 0;
      try {
        signal.throwIfAborted();
        response = await fetch(source, { ...options, integrity: "", signal });
        status = response.status;
        if (response.ok) {
          const total = response.headers.has("content-encoding") ? 0 : Number(response.headers.get("content-length"));
          const chunks = [];
          let length = 0;
          if (response.body) {
            const reader = response.body.getReader();
            try {
              for (;;) {
                const { done, value } = await reader.read();
                if (done) break;
                chunks.push(value);
                length += value.length;
                progress?.(length, total);
              }
            } finally {
              await reader.cancel().catch(() => {});
              reader.releaseLock();
            }
          }
          const bytes = new Uint8Array(length);
          let offset = 0;
          for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.length; }
          if (integrity) {
            invalidIntegrity = true;
            const [algorithm] = integrity.split("-");
            const digest = new Uint8Array(await crypto.subtle.digest(algorithm.toUpperCase().replace("SHA", "SHA-"), bytes));
            let bin = "";
            for (const b of digest) bin += String.fromCharCode(b);
            if (`${algorithm}-${btoa(bin)}` !== integrity) throw new Error(`integrity: bytes do not match ${integrity}`);
            invalidIntegrity = false;
          }
          return new Response(status === 204 || status === 205 ? null : bytes, { status, statusText: response.statusText, headers: response.headers });
        }
        // Cross-origin asset bases must expose Retry-After through CORS.
        retryAfter = response.headers.get("retry-after");
        await response.body?.cancel();
        cause = undefined;
      } catch (e) {
        if (!invalidIntegrity) status = 0;
        cause = e;
        // A read can fail after headers; abandoning it must also release the
        // upstream stream before another attempt acquires a slot.
        if (response?.body && !response.body.locked) await response.body.cancel().catch(() => {});
      } finally {
        leave(false);
      }
      progress?.(0, 0);
      if (!retry(status) || attempt === ATTEMPTS - 1) break;
      await pause(delay(attempt, retryAfter), signal);
    }
  } catch (e) { cause = e; }
  throw failure(url, status, Math.min(attempt, ATTEMPTS - 1), cause);
}

// A synchronous placeholder already blocks its Worker. Atomics.wait sleeps
// without burning CPU on isolated pages (all lanes pages); without SAB the
// host still supports synchronous lazy modules, with the same bounded delay.
function sleep(ms) {
  if (shared) Atomics.wait(shared, 1, Atomics.load(shared, 1), ms);
  else { const until = Date.now() + ms; while (Date.now() < until) { /* blocking host */ } }
}

export function fetchResourceSync(url) {
  url = String(url).replaceAll("#", "%23");
  const until = Date.now() + FETCH_DEADLINE_MS;
  let status = 0;
  let cause;
  let attempt = 0;
  for (; attempt < ATTEMPTS; attempt++) {
    while (!enter(true)) {
      if (Date.now() >= until) throw failure(url, status, attempt, "deadline exceeded");
      sleep(10);
    }
    let retryAfter = null;
    try {
      const x = new XMLHttpRequest();
      x.open("GET", url, false);
      x.responseType = "arraybuffer";
      x.timeout = Math.max(1, until - Date.now());
      x.send();
      status = x.status;
      if (status >= 200 && status < 300) return x.response;
      retryAfter = x.getResponseHeader("retry-after");
      cause = undefined;
    } catch (e) { status = 0; cause = e; }
    finally { leave(true); }
    if (!retry(status) || attempt === ATTEMPTS - 1) break;
    const wait = delay(attempt, retryAfter);
    if (Date.now() + wait >= until) { cause = "retry exceeds deadline"; break; }
    // Notifications from other streams cannot shorten Retry-After.
    const wake = Date.now() + wait;
    while (Date.now() < wake) sleep(wake - Date.now());
  }
  throw failure(url, status, attempt, cause);
}
