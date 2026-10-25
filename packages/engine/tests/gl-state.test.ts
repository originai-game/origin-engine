// The WebGL2 tier's state filter (src/web/gl-state.ts) against a literal
// model of the GL state: the calls that act (draws, clears, uploads, reads,
// syncs) reach the context in the same order with the same arguments, and
// each acts on the same state, filtered or not; the reference scene's
// recorded frame reaches the context in under a quarter of its calls.
//
// fixtures/webgl2-frame/reference-scene.jsonl is one steady frame of
// crates/og-tests/reference-scene on the WebGL2 tier as wgpu's GL backend
// issues it, recorded with `bun tools/bench/webgl2-trace.ts --sequence`
// (objects by the order they first appear: "#12").
import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { filterGlState } from "../src/web/gl-state.ts";

type Call = [string, ...unknown[]];

// ── the model: every call applied as GL applies it ──────────────────────
const FRAMEBUFFER = 0x8d40, DRAW_FRAMEBUFFER = 0x8ca9, READ_FRAMEBUFFER = 0x8ca8;
const DEPTH_ATTACHMENT = 0x8d00, STENCIL_ATTACHMENT = 0x8d20, DEPTH_STENCIL_ATTACHMENT = 0x821a;
const ELEMENT_ARRAY_BUFFER = 0x8893, ARRAY_BUFFER = 0x8892;
const FRONT = 0x0404, BACK = 0x0405, FRONT_AND_BACK = 0x0408;
const SCISSOR_TEST = 0x0c11, RASTERIZER_DISCARD = 0x8c89, DITHER = 0x0bd0;

/** Calls that only set state (a filter may drop or defer them). */
const SETTERS = new Set([
  "enable", "disable", "activeTexture", "bindTexture", "texParameteri", "bindSampler", "bindBuffer", "bindBufferRange",
  "bindBufferBase", "useProgram", "bindFramebuffer", "framebufferTexture2D", "framebufferTextureLayer",
  "framebufferRenderbuffer", "drawBuffers", "colorMask", "depthMask", "stencilMask", "stencilMaskSeparate", "stencilFunc",
  "stencilFuncSeparate", "stencilOp", "stencilOpSeparate", "frontFace", "cullFace", "depthFunc", "depthRange", "viewport",
  "scissor", "blendColor", "blendEquation", "blendEquationSeparate", "blendFunc", "blendFuncSeparate", "clearColor",
  "clearDepth", "clearStencil", "polygonOffset", "pixelStorei", "enableVertexAttribArray", "disableVertexAttribArray",
  "vertexAttribPointer", "vertexAttribIPointer", "vertexAttribDivisor", "bindVertexArray", "uniform1ui", "uniform1i",
  "uniform4f", "uniformMatrix4fv",
]);
const DRAWS = new Set(["drawArrays", "drawElements", "drawArraysInstanced", "drawElementsInstanced"]);
const CLEARS = new Set(["clear", "clearBufferfv", "clearBufferiv", "clearBufferuiv", "clearBufferfi"]);

class GlModel {
  /** Context state by key; per-VAO state under `vao:<vao>:...`. */
  readonly s = new Map<string, unknown>();
  vao: unknown = null;
  unit: unknown = 0x84c0;
  apply(c: Call): void {
    const [n, ...a] = c;
    const s = this.s;
    const set = (k: string, v: unknown) => s.set(k, JSON.stringify(v));
    const fbs = (t: unknown) => (t === FRAMEBUFFER ? ["draw", "read"] : t === READ_FRAMEBUFFER ? ["read"] : ["draw"]);
    const fbOf = (t: unknown) => s.get(`fb:${t === READ_FRAMEBUFFER ? "read" : "draw"}`) ?? JSON.stringify(null);
    const deleted = (obj: unknown) => {
      // Bindings in the current context revert to none.
      for (const [k, v] of s) if (v === JSON.stringify(obj) && (k.startsWith("tex:") || k.startsWith("sampler:") || k.startsWith("buf:") || k.startsWith("fb:") || k === `vao:${this.vao}:elements`)) s.set(k, JSON.stringify(null));
      for (const [k, v] of s) if (typeof v === "string" && v.includes(JSON.stringify(obj)) && (k.startsWith("range:") || k.startsWith(`vao:${this.vao}:ptr:`))) s.set(k, JSON.stringify(["gone"]));
      for (const [k, v] of s) if (typeof v === "string" && v.includes(JSON.stringify(obj)) && k.startsWith("att:")) {
        const fb = k.split(":")[1];
        if (fb === s.get("fb:draw") || fb === s.get("fb:read")) s.set(k, JSON.stringify([]));
      }
      for (const k of [...s.keys()]) if (k.startsWith(`param:${JSON.stringify(obj)}:`)) s.delete(k);
    };
    switch (n) {
      case "enable": case "disable": set(`cap:${a[0]}`, n === "enable"); break;
      case "activeTexture": this.unit = a[0]; break;
      case "bindTexture": set(`tex:${this.unit}:${a[0]}`, a[1]); break;
      case "texParameteri": case "texParameterf": set(`param:${s.get(`tex:${this.unit}:${a[0]}`)}:${a[1]}`, a[2]); break;
      case "bindSampler": set(`sampler:${a[0]}`, a[1]); break;
      case "bindBuffer": set(a[0] === ELEMENT_ARRAY_BUFFER ? `vao:${this.vao}:elements` : `buf:${a[0]}`, a[1]); break;
      case "bindBufferRange": set(`range:${a[0]}:${a[1]}`, a.slice(2)); set(`buf:${a[0]}`, a[2]); break;
      case "bindBufferBase": set(`range:${a[0]}:${a[1]}`, [a[2]]); set(`buf:${a[0]}`, a[2]); break;
      case "useProgram": set("program", a[0]); break;
      case "bindFramebuffer": for (const f of fbs(a[0])) set(`fb:${f}`, a[1]); break;
      case "framebufferTexture2D": case "framebufferTextureLayer": case "framebufferRenderbuffer": {
        const obj = n === "framebufferTexture2D" ? a[3] : n === "framebufferTextureLayer" ? a[2] : a[3];
        const v = obj === null ? [] : n === "framebufferTexture2D" ? ["2d", a[3], a[2], a[4]] : n === "framebufferTextureLayer" ? ["layer", a[2], a[3], a[4]] : ["rb", a[3]];
        const points = a[1] === DEPTH_STENCIL_ATTACHMENT ? [DEPTH_ATTACHMENT, STENCIL_ATTACHMENT] : [a[1]];
        for (const p of points) set(`att:${fbOf(a[0])}:${p}`, v);
        break;
      }
      case "drawBuffers": set(`drawBuffers:${s.get("fb:draw")}`, [...(a[0] as number[])]); break;
      case "stencilMask": case "stencilFunc": case "stencilOp":
        for (const f of [FRONT, BACK]) set(`${n}:${f}`, a);
        break;
      case "stencilMaskSeparate": case "stencilFuncSeparate": case "stencilOpSeparate":
        for (const f of a[0] === FRONT_AND_BACK ? [FRONT, BACK] : [a[0]]) set(`${n.replace("Separate", "")}:${f}`, a.slice(1));
        break;
      case "blendEquation": set("blendEquation", [a[0], a[0]]); break;
      case "blendEquationSeparate": set("blendEquation", a); break;
      case "blendFunc": set("blendFunc", [a[0], a[1], a[0], a[1]]); break;
      case "blendFuncSeparate": set("blendFunc", a); break;
      case "pixelStorei": set(`pixelStore:${a[0]}`, a[1]); break;
      case "enableVertexAttribArray": case "disableVertexAttribArray": set(`vao:${this.vao}:on:${a[0]}`, n.startsWith("enable")); break;
      case "vertexAttribPointer": case "vertexAttribIPointer":
        set(`vao:${this.vao}:ptr:${a[0]}`, [s.get(`buf:${ARRAY_BUFFER}`) ?? null, n, ...a.slice(1)]);
        break;
      case "vertexAttribDivisor": set(`vao:${this.vao}:div:${a[0]}`, a[1]); break;
      case "bindVertexArray": this.vao = a[0]; break;
      // Draw buffer 0's mask: the context's colorMask no longer holds for it.
      case "colorMaskiOES": set("colorMask", ["indexed", ...a]); break;
      case "deleteVertexArray": if (this.vao === a[0]) this.vao = null; break;
      case "uniform1ui": case "uniform1i": case "uniform4f": case "uniformMatrix4fv":
        if (a[0] !== null && a[0] !== undefined) set(`uniform:${s.get("program")}:${a[0]}`, a.slice(1));
        break;
      case "deleteTexture": case "deleteBuffer": case "deleteFramebuffer": case "deleteSampler": case "deleteRenderbuffer":
        deleted(a[0]);
        break;
      default:
        if (SETTERS.has(n)) set(n, a);
    }
  }
  /**
   * The state `call` reads. The bindings a filter may defer (capabilities,
   * attribute arrays, the active unit, the program, the framebuffers, the
   * generic buffer targets) only where the call reads them: a draw reads all
   * but the active unit, the read framebuffer and the generic buffers; a
   * clear the draw framebuffer and SCISSOR_TEST, RASTERIZER_DISCARD, DITHER;
   * a buffer upload its target; any call this test does not know, all.
   */
  view(call: Call): string {
    const [n, ...a] = call;
    const deferred = (k: string) => k.startsWith("cap:") || k.includes(":on:") || k === "program" || k.startsWith("fb:") || k.startsWith("buf:");
    const known = DRAWS.has(n) || CLEARS.has(n) || n.startsWith("delete") || ["bufferSubData", "bufferData", "fenceSync", "getSyncParameter", "flush", "invalidateFramebuffer"].includes(n);
    const reads = (k: string): boolean => {
      if (!known || !deferred(k)) return true;
      if (DRAWS.has(n)) return !k.startsWith("buf:") && k !== "fb:read";
      if (CLEARS.has(n)) return k === "fb:draw" || [SCISSOR_TEST, RASTERIZER_DISCARD, DITHER].some((c) => k === `cap:${c}`);
      if (n === "bufferSubData" || n === "bufferData") return k === `buf:${a[0]}`;
      if (n === "invalidateFramebuffer") return k === (a[0] === READ_FRAMEBUFFER ? "fb:read" : "fb:draw");
      return false;
    };
    const entries = [...this.s].filter(([k]) => reads(k)).sort(([x], [y]) => (x < y ? -1 : x > y ? 1 : 0));
    return JSON.stringify([known ? null : this.unit, this.vao, entries]);
  }
}

/** A context that records what reaches it; objects are names. */
function fakeGl(names: Iterable<string>) {
  const seen: Call[] = [];
  let next = 0;
  const proto: Record<string, unknown> = {};
  for (const n of new Set([...names, "getExtension"])) {
    proto[n] = function (this: unknown, ...a: unknown[]) {
      // As given: WebGL picks an overload by the count of arguments.
      seen.push([n, ...a]);
      if (n === "getExtension") return extension;
      return n.startsWith("create") ? `@${next++}` : undefined;
    };
  }
  // An extension with state of its own (OES_draw_buffers_indexed's colorMaskiOES).
  const extProto = { colorMaskiOES(...a: unknown[]) { seen.push(["colorMaskiOES", ...a]); } };
  const extension = Object.create(extProto) as Record<string, (...a: unknown[]) => void>;
  const gl = Object.create(proto) as Record<string, (...a: unknown[]) => unknown>;
  return { gl, seen };
}

/**
 * Runs `calls` through the filter and checks the actions against the model
 * of the unfiltered stream; returns the calls that reached the context.
 */
function check(calls: Call[], extra: string[] = []): { forwarded: Call[]; received: number } {
  const names = new Set([...calls.map((c) => c[0]), ...SETTERS, ...extra]);
  const { gl, seen } = fakeGl(names);
  const counts = filterGlState(gl as unknown as WebGL2RenderingContext);
  for (const [n, ...a] of calls) {
    if (n === "colorMaskiOES") (gl.getExtension!("OES_draw_buffers_indexed") as Record<string, (...a: unknown[]) => void>).colorMaskiOES!(...a);
    else gl[n]!(...a);
  }
  // Each colorMaskiOES asks for its extension first.
  expect(counts.calls).toBe(calls.length);
  const actions = (stream: Call[]) => {
    const m = new GlModel();
    const out: string[] = [];
    for (const c of stream) {
      if (c[0] === "getExtension") continue;
      if (!SETTERS.has(c[0])) out.push(`${JSON.stringify(c)} on ${m.view(c)}`);
      m.apply(c);
    }
    return out;
  };
  const want = actions(calls);
  const got = actions(seen);
  expect(got.length).toBe(want.length);
  for (let i = 0; i < want.length; i++) if (got[i] !== want[i]) expect(got[i]).toBe(want[i]!);
  return { forwarded: seen.filter((c) => c[0] !== "getExtension"), received: counts.calls };
}

const fixture = (): Call[] =>
  readFileSync(join(dirname(fileURLToPath(import.meta.url)), "fixtures", "webgl2-frame", "reference-scene.jsonl"), "utf8")
    .trim()
    .split("\n")
    .map((l) => {
      const [n, ...a] = JSON.parse(l) as [string, ...unknown[]];
      // The recording names a byte array by its length; drawBuffers takes a list.
      return [n.replace(/^gl\./, ""), ...a.map((v, i) => (n === "gl.drawBuffers" && i === 0 ? [0x8ce0] : v))] as Call;
    });

test("the reference scene's WebGL2 frame reaches the context in under a quarter of its calls, acting on the same state", () => {
  const calls = fixture();
  expect(calls.length).toBe(3797);
  const draws = (cs: Call[]) => cs.filter((c) => DRAWS.has(c[0])).length;
  expect(draws(calls)).toBe(41);
  // The first frame starts from a context the filter knows nothing of;
  // the second is every later one.
  const first = check(calls).forwarded;
  const both = check([...calls, ...calls]).forwarded;
  const second = both.slice(first.length);
  expect(first.length).toBeLessThan(1100);
  expect(second.length).toBeLessThan(950);
  expect(draws(second)).toBe(41);
  // The per-draw texture levels, the pass-start capability resets and the
  // first-instance uniform without a location are gone.
  const count = (cs: Call[], n: string) => cs.filter((c) => c[0] === n).length;
  expect(count(second, "texParameteri")).toBe(0);
  expect(count(second, "uniform1ui")).toBe(0);
  expect(count(second, "disable")).toBeLessThan(20);
  expect(count(second, "framebufferTexture2D")).toBeLessThan(60);
});

/** A seeded generator (mulberry32). */
function rng(seed: number) {
  return () => {
    seed = (seed + 0x6d2b79f5) | 0;
    let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

test("random call streams act on the same state filtered and unfiltered", () => {
  for (let seed = 1; seed <= 300; seed++) {
    const r = rng(seed);
    const pick = <T>(xs: readonly T[]): T => xs[Math.floor(r() * xs.length)]!;
    // Few objects and values, so calls repeat and objects die while bound.
    let made = 0;
    const live = { tex: ["t0", "t1"], buf: ["b0", "b1"], fb: ["f0", "f1"], rb: ["r0"], vao: ["v0"], smp: ["s0"] } as Record<string, string[]>;
    const obj = (kind: string) => (r() < 0.15 ? null : live[kind]!.length ? pick(live[kind]!) : null);
    const kill = (kind: string): Call => {
      const name = { tex: "deleteTexture", buf: "deleteBuffer", fb: "deleteFramebuffer", rb: "deleteRenderbuffer", vao: "deleteVertexArray", smp: "deleteSampler" }[kind]!;
      const list = live[kind]!;
      const o = list.length ? list.splice(Math.floor(r() * list.length), 1)[0]! : null;
      list.push(`${kind}${100 + made++}`);
      return [name, o];
    };
    const small = [0, 1, 2];
    const gens: Array<() => Call> = [
      () => [pick(["enable", "disable"]), pick([0x0b71, 0x0b90, SCISSOR_TEST, 0x0be2, RASTERIZER_DISCARD])],
      () => ["activeTexture", 0x84c0 + pick(small)],
      () => ["bindTexture", pick([0x0de1, 0x8c1a]), obj("tex")],
      () => ["texParameteri", pick([0x0de1, 0x8c1a]), pick([0x813c, 0x813d]), pick(small)],
      () => ["bindSampler", pick(small), obj("smp")],
      () => ["bindBuffer", pick([ARRAY_BUFFER, ELEMENT_ARRAY_BUFFER, 0x8f37, 0x8a11]), obj("buf")],
      () => ["bindBufferRange", 0x8a11, pick(small), obj("buf"), pick([0, 256]), 256],
      () => ["bindBufferBase", 0x8a11, pick(small), obj("buf")],
      () => ["useProgram", pick(["p0", "p1", null])],
      () => ["bindFramebuffer", pick([FRAMEBUFFER, DRAW_FRAMEBUFFER, READ_FRAMEBUFFER]), obj("fb")],
      () => ["framebufferTexture2D", pick([FRAMEBUFFER, DRAW_FRAMEBUFFER, READ_FRAMEBUFFER]), pick([0x8ce0, DEPTH_ATTACHMENT, STENCIL_ATTACHMENT, DEPTH_STENCIL_ATTACHMENT]), 0x0de1, obj("tex"), pick([0, 1])],
      () => ["framebufferTextureLayer", DRAW_FRAMEBUFFER, pick([0x8ce0, DEPTH_ATTACHMENT]), obj("tex"), 0, pick(small)],
      () => ["framebufferRenderbuffer", DRAW_FRAMEBUFFER, pick([DEPTH_ATTACHMENT, STENCIL_ATTACHMENT, DEPTH_STENCIL_ATTACHMENT]), 0x8d41, obj("rb")],
      () => ["drawBuffers", pick([[0x8ce0], [0], [0x8ce0, 0x8ce1]])],
      () => ["colorMask", r() < 0.5, true, true, r() < 0.5],
      () => ["depthMask", r() < 0.5],
      () => ["stencilMask", pick([0, 255])],
      () => ["stencilMaskSeparate", pick([FRONT, BACK, FRONT_AND_BACK]), pick([0, 255])],
      () => ["stencilFuncSeparate", pick([FRONT, BACK, FRONT_AND_BACK]), 519, 0, pick([0, 255])],
      () => ["stencilFunc", 519, 0, pick([0, 255])],
      () => ["blendFunc", 1, pick([0, 771])],
      () => ["blendFuncSeparate", 1, pick([0, 771]), 1, pick([0, 771])],
      () => ["blendEquation", pick([0x8006, 0x800a])],
      () => ["viewport", 0, 0, pick([64, 128]), 64],
      () => ["scissor", 0, 0, pick([64, 128]), 64],
      () => ["depthRange", 0, pick([0.5, 1])],
      () => ["pixelStorei", pick([0x0cf5, 0x0d05]), pick([1, 4])],
      () => [pick(["enableVertexAttribArray", "disableVertexAttribArray"]), pick(small)],
      () => ["vertexAttribPointer", pick(small), 3, 0x1406, false, 12, pick([0, 12])],
      () => ["vertexAttribIPointer", pick(small), 4, 0x1403, 8, 0],
      () => ["vertexAttribDivisor", pick(small), pick([0, 1])],
      () => ["bindVertexArray", obj("vao")],
      () => ["uniform1ui", pick([null, undefined, "u0"]), pick(small)],
      () => kill(pick(["tex", "buf", "fb", "rb", "vao", "smp"])),
      () => ["drawElementsInstanced", 4, 36, 0x1405, 0, pick([1, 20])],
      () => ["drawArrays", 4, 0, 3],
      () => ["clear", 0x4100],
      () => ["clearBufferfv", 0x1800, 0, "rgba"],
      () => ["texSubImage2D", 0x0de1, 0, 0, 0, 1, 1, 0x1908, 0x1401, "pixels"],
      () => ["bufferSubData", 0x8f37, 0, "bytes"],
      () => ["blitFramebuffer", 0, 0, 1, 1, 0, 0, 1, 1, 0x4000, 0x2600],
      () => ["frobnicate", pick(small)],
      () => ["colorMaskiOES", 0, true, false, true, false],
    ];
    const calls: Call[] = [];
    for (let i = 0; i < 400; i++) calls.push(pick(gens)());
    try {
      check(calls, ["frobnicate"]);
    } catch (e) {
      throw new Error(`seed ${seed}: ${e instanceof Error ? e.message : String(e)}`);
    }
  }
});

test("a call the filter does not model, or an extension's, forgets what it knew", () => {
  const calls: Call[] = [
    ["useProgram", "p0"],
    ["frobnicate"],
    ["useProgram", "p0"],
    ["colorMask", true, true, true, true],
    ["colorMaskiOES", 0, true, false, true, false],
    ["colorMask", true, true, true, true],
  ];
  const { forwarded } = check(calls, ["frobnicate"]);
  // The program is bound when a call reads it: the unknown call, then the extension's.
  expect(forwarded.map((c) => c[0])).toEqual(["useProgram", "frobnicate", "colorMask", "useProgram", "colorMaskiOES", "colorMask"]);
});
