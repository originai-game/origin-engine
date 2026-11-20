// The page half of `og dev --web` (dev-web.ts serves it as _dev/client.js beside the
// development build's own play.js): the server's socket, each change applied in the running
// engine (cooked files in place, the game's code by snapshot, swap and restore), and the Stage's
// dev channel. Every URL is relative to the page, so it works under a proxy's prefix.
//
// window.__og_dev_reloads__ lists each change applied with its steps, save to frame:
//   files: saveToCookedMs (the server: the save seen, the cook) + fetchMs + applyMs (the
//          Worker's devFiles; workerMs and its steps inside) + frameMs (the next drawn frame)
//   code:  bundleMs, writtenToFrameMs = fetch/import + reload (Worker) + frame
const base = document.baseURI;
const reloads = (window.__og_dev_reloads__ = []);
/** The build this page booted (dev-web.ts writes it into the page): a reconnect to another reloads. */
const build = document.querySelector('meta[name="og-dev-build"]')?.content ?? "";
let seq = 0;

const probe = () => window.__runtime_probe__;
const until = async (ok, ms = 60000) => {
  const t0 = performance.now();
  while (!ok()) {
    if (performance.now() - t0 > ms) throw new Error("og dev: the engine did not start");
    await new Promise((r) => setTimeout(r, 50));
  }
};
const frame = () => new Promise((done) => requestAnimationFrame(() => requestAnimationFrame(done)));
const round = (ms) => Math.round(ms * 10) / 10;
const say = (text, warn) => (warn ? console.warn : console.log)(`og dev: ${text}`);

// ── applying, one change at a time; a newer one supersedes what waits ───────
let applying = false;
let waiting = null;
function queue(m) {
  if (!waiting) waiting = { files: new Map(), code: null, seq: 0, edits: new Set(), steps: {} };
  if (m.type === "files") {
    for (const f of m.files) waiting.files.set(f.path, f);
    waiting.steps.files = m.steps;
  } else {
    waiting.code = m;
  }
  for (const e of m.edits ?? []) waiting.edits.add(e);
  waiting.seq = Math.max(waiting.seq, m.seq);
  if (!applying) void drain();
}
async function drain() {
  applying = true;
  try {
    while (waiting) {
      const w = waiting;
      waiting = null;
      await until(() => probe()?.devFiles);
      if (w.files.size) await applyFiles(w);
      if (w.code) await applyCode(w);
      seq = Math.max(seq, w.seq);
    }
  } catch (e) {
    say(String(e), true);
  } finally {
    applying = false;
  }
}
async function applyFiles(w) {
  const t0 = performance.now();
  const files = await Promise.all([...w.files.values()].map(async (f) => {
    const url = new URL(f.path, base).href;
    const r = await fetch(url, { cache: "no-store" });
    return [url, new Uint8Array(await r.arrayBuffer())];
  }));
  const fetched = performance.now();
  const r = await probe().devFiles(files);
  const applied = performance.now();
  await frame();
  const drawn = performance.now();
  const s = w.steps.files ?? {};
  // Save to frame: from the edited file's save (the server's clock read as this page's: the same machine, or near it) to the frame.
  const saveToFrameMs = s.savedAt ? Math.max(0, Date.now() - s.savedAt) : null;
  const steps = { saveToCookedMs: s.saveToCookedMs, cookMs: s.cookMs, detectMs: s.detectMs, fetchMs: round(fetched - t0), applyMs: round(applied - fetched), workerMs: r?.ms ?? null, worker: r?.steps, frameMs: round(drawn - applied) };
  const ok = !!r && !r.error;
  report("files", w, ok, saveToFrameMs ?? round(drawn - t0), steps, r?.error ?? (r?.warnings?.length ? r.warnings.join("\n") : undefined));
}
async function applyCode(w) {
  const t0 = performance.now();
  let r = null;
  let error;
  try {
    r = await probe().devCode(new URL(w.code.url, base).href);
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
  }
  const swapped = performance.now();
  await frame();
  const drawn = performance.now();
  const s = w.code.steps ?? {};
  const writtenToFrameMs = s.writtenAt ? Math.max(0, Date.now() - s.writtenAt) : round(drawn - t0);
  const steps = { saveToBundledMs: s.saveToBundledMs, bundleMs: s.bundleMs, writtenToFrameMs, callMs: round(swapped - t0), importMs: r?.importMs, reloadMs: r?.reloadMs, frameMs: round(drawn - swapped) };
  const ok = !!r && r.report !== null && !error;
  report("code", w, ok, (s.saveToBundledMs ?? 0) + writtenToFrameMs, steps, error ?? (r && r.report === null ? "the probe's reload refused" : undefined));
}
function report(kind, w, ok, ms, steps, error) {
  const entry = { kind, seq: w.seq, edits: [...w.edits], ok, ms: round(ms), steps, ...(error ? { error } : {}) };
  reloads.push(entry);
  window.__og_dev_last__ = entry;
  say(`${entry.edits.join(", ") || "resync"}: ${kind} ${ok ? "applied" : `refused: ${error}`}, save to frame ${entry.ms} ms ${JSON.stringify(steps)}`, !ok);
  socket?.readyState === 1 && socket.send(JSON.stringify({ type: "applied", seq: w.seq, kind, ok, ms: entry.ms, steps, ...(error ? { error } : {}) }));
}

// ── the socket, reconnecting with backoff ───────────────────────────────────
let socket = null;
let backoff = 250;
function connect() {
  const url = new URL("_dev/socket", base);
  url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
  socket = new WebSocket(url);
  socket.onopen = () => {
    backoff = 250;
    socket.send(JSON.stringify({ type: "hello", build, seq }));
  };
  socket.onmessage = (ev) => {
    const m = JSON.parse(ev.data);
    if (m.type === "hello" && m.error) say(m.error, true);
    // A cold restart (or a reconnect to a server that made one since): the new build.
    if (m.type === "reload" && m.build && m.build !== build) location.reload();
    if (m.type === "files" || m.type === "code") queue(m);
    if (m.type === "error") (m.error ? say(`${(m.edits ?? []).join(", ")}: ${m.error}`, true) : say("the build is clean again"));
  };
  socket.onclose = () => {
    socket = null;
    setTimeout(connect, backoff);
    backoff = Math.min(backoff * 2, 5000);
  };
}
connect();

// ── the Stage's dev channel (same document, window.postMessage) ─────────────
// {og:"dev", v:1, from:"page"|"host", type, id?, …}: hello, inspector {open}. v1 has no Keep
// (`keep: false`: the overlay's own Keep stays) and sends no `edits`. The inspector's Save and
// read_source go over the og-probe wire (shell/wire.ts, docs/web-export.md §og dev --web).
const post = (type, body = {}) => window.postMessage({ og: "dev", v: 1, from: "page", type, ...body }, location.origin);
let hostSeen = false;
// The page tracks the inspector's state: the host's `inspector` sets it, and its toggle key (F1
// unless the game binds F1, then Ctrl+F1) flips it.
let inspectorOpen = false;
const setInspector = (open, tellHost) => {
  if (open === inspectorOpen) return;
  inspectorOpen = open;
  if (tellHost) post("inspector", { open });
};
window.addEventListener("message", (ev) => {
  if (ev.source !== window || ev.origin !== location.origin) return;
  const m = ev.data;
  if (!m || m.og !== "dev" || m.v !== 1 || m.from !== "host") return;
  if (m.type === "hello") hostSeen = true;
  if (m.type === "inspector") {
    const open = !!m.open;
    void probe()?.devInspector?.(open).then((r) => {
      if (r?.ok !== false) setInspector(open, true);
    });
  }
});
window.addEventListener("keydown", (ev) => {
  if (ev.code === "F1" && !ev.repeat) setInspector(!inspectorOpen, true);
}, true);
const engine = JSON.parse(document.getElementById("og-config")?.textContent ?? "{}").engine ?? "";
void (async () => {
  for (let i = 0; i < 20 && !hostSeen; i++) {
    post("hello", { inspector: true, keep: false, engine });
    await new Promise((r) => setTimeout(r, 250));
  }
})();
