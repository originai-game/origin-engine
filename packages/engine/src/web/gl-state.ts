// The WebGL2 tier's state filter: a mirror of the context's state in front
// of the WebGL2RenderingContext wgpu's GL backend draws through, which drops
// every call that sets a value the context already holds and defers the
// enables (capabilities, vertex attribute arrays), the active texture unit,
// the program, the framebuffer bindings and the generic buffer bindings to
// the next call that reads them.
//
// Why: wgpu-hal's GLES backend re-states everything it may need on every
// command buffer, pass and draw (seven capabilities reset per command
// buffer, every attachment point detached per pass, every vertex attribute
// re-specified and every bound texture's base and max level set again per
// draw, every attribute array disabled after each pass). On the reference
// scene that is about 3,800 WebGL calls a frame for 41 draws; each one crosses
// into the browser's WebGL implementation and its GPU process, which the
// WebGPU tier never pays. tests/evidence/r4/perf-webgl2/NOTES.md has the
// accounting.
//
// Exact by construction: a call is dropped only when the mirror holds the
// value it would set, and a deferred value is set before any call that could
// read it (a draw, a clear, a blit, an upload, a query), so every such call
// acts on the state it would have acted on unfiltered: frames are identical.
// What the mirror does not know (at the start, after a call it does not
// model, after an extension's call) is unknown, and a call on unknown state
// is forwarded. The hot paths take fixed arguments and allocate nothing (a
// dropped call costs less than the WebGL call it replaces); a call is
// forwarded with as many arguments as it was given, as WebGL picks an
// overload by their count.

/** Calls the filter received, and those it passed to the context. */
export interface GlStateCounts {
  calls: number;
  forwarded: number;
}

type Gl = WebGL2RenderingContext;
type Fn = (...a: unknown[]) => unknown;
type Obj = object | string | null;

const FRAMEBUFFER = 0x8d40;
const READ_FRAMEBUFFER = 0x8ca8;
const DRAW_FRAMEBUFFER = 0x8ca9;
const DEPTH_ATTACHMENT = 0x8d00;
const STENCIL_ATTACHMENT = 0x8d20;
const DEPTH_STENCIL_ATTACHMENT = 0x821a;
const ELEMENT_ARRAY_BUFFER = 0x8893;
const ARRAY_BUFFER = 0x8892;
const TRANSFORM_FEEDBACK_BUFFER = 0x8c8e;
const TEXTURE0 = 0x84c0;
const FRONT_AND_BACK = 0x0408;
const FRONT = 0x0404;
const BACK = 0x0405;
/** SCISSOR_TEST, RASTERIZER_DISCARD, DITHER: the capabilities a clear reads. */
const CLEAR_CAPS = [0x0c11, 0x8c89, 0x0bd0];

/** Unknown: a value no argument can equal. */
const UNKNOWN: unique symbol = Symbol("unknown");
/** Nothing asked: a deferred binding with no call waiting to set it. */
const PENDING: unique symbol = Symbol("nothing asked");
type Known<T> = T | typeof UNKNOWN;

/** Calls that change no state the mirror holds and read none it defers. */
const INDEPENDENT = [
  "uniformBlockBinding", "fenceSync", "deleteSync", "getSyncParameter", "clientWaitSync", "flush", "createBuffer",
  "createFramebuffer", "createRenderbuffer", "createSampler", "createTexture", "createVertexArray", "createQuery",
  "createShader", "createProgram", "shaderSource", "compileShader", "getShaderParameter", "getShaderInfoLog",
  "attachShader", "detachShader", "deleteShader", "linkProgram", "getProgramParameter", "getProgramInfoLog",
  "getUniformLocation", "getUniformBlockIndex", "getAttribLocation", "bindAttribLocation", "samplerParameteri",
  "samplerParameterf", "bindRenderbuffer", "renderbufferStorage", "renderbufferStorageMultisample",
  "vertexAttrib1f", "vertexAttrib2f", "vertexAttrib3f", "vertexAttrib4f", "vertexAttrib1fv", "vertexAttrib2fv",
  "vertexAttrib3fv", "vertexAttrib4fv", "vertexAttribI4i", "vertexAttribI4ui", "vertexAttribI4iv", "vertexAttribI4uiv",
];
/** Clears: they read no deferred state but CLEAR_CAPS and the draw framebuffer. */
const CLEARS = ["clear", "clearBufferfv", "clearBufferiv", "clearBufferuiv", "clearBufferfi"];
/**
 * Calls that change no state the mirror holds but may read any it defers:
 * they run after it is set. A call in none of these lists and not modelled
 * below forgets the whole mirror after it runs.
 */
const OBSERVING = [
  "blitFramebuffer", "readPixels", "getBufferSubData", "texImage2D", "texImage3D", "texSubImage2D", "texSubImage3D",
  "texStorage2D", "texStorage3D", "compressedTexImage2D", "compressedTexImage3D", "compressedTexSubImage2D",
  "compressedTexSubImage3D", "copyTexImage2D", "copyTexSubImage2D", "copyTexSubImage3D", "generateMipmap",
  "getTexParameter", "getParameter", "getIndexedParameter", "getError", "isEnabled", "getVertexAttrib",
  "getVertexAttribOffset", "getFramebufferAttachmentParameter", "getRenderbufferParameter", "getBufferParameter",
  "checkFramebufferStatus", "getSupportedExtensions", "getContextAttributes", "isContextLost", "finish", "beginQuery",
  "endQuery", "getQuery", "getQueryParameter", "getActiveUniform", "getActiveAttrib", "getActiveUniformBlockParameter",
  "getActiveUniformBlockName", "getActiveUniforms", "getUniformIndices", "getUniform", "getShaderSource",
  "getShaderPrecisionFormat", "getAttachedShaders", "getInternalformatParameter", "getSamplerParameter", "isBuffer",
  "isFramebuffer", "isProgram", "isRenderbuffer", "isShader", "isTexture", "isVertexArray", "isSampler", "isSync",
  "isQuery", "validateProgram", "deleteQuery", "hint", "lineWidth", "sampleCoverage", "waitSync", "deleteProgram",
];
/** Draws: they read the capabilities, the attribute arrays, the program and the draw framebuffer. */
const DRAWS = ["drawArrays", "drawElements", "drawArraysInstanced", "drawElementsInstanced", "drawRangeElements"];

/** What one attachment point holds: kind 0 none, 1 a 2D image, 2 a layer, 3 a renderbuffer. */
interface Attachment {
  kind: number;
  obj: Obj;
  a: unknown;
  b: unknown;
}
const NONE: Attachment = { kind: 0, obj: null, a: 0, b: 0 };

/** The state one vertex array object holds, per attribute location. */
class VaoState {
  elements: Known<Obj> = UNKNOWN;
  /** Arrays enabled as the context has them (undefined: unknown), and as asked. */
  readonly enabled: (boolean | undefined)[] = [];
  readonly wanted: (boolean | undefined)[] = [];
  /** Locations with a wanted value. */
  readonly dirty: number[] = [];
  /** Pointers: buffer (UNKNOWN: unknown), integer, size, type, normalised, stride, offset. */
  readonly buffer: Known<Obj>[] = [];
  readonly integer: boolean[] = [];
  readonly size: unknown[] = [];
  readonly type: unknown[] = [];
  readonly norm: unknown[] = [];
  readonly stride: unknown[] = [];
  readonly offset: unknown[] = [];
  readonly divisor: unknown[] = [];
  /** Forgets every pointer that reads `obj` (it was deleted). */
  drop(obj: unknown): void {
    if (this.elements === obj) this.elements = UNKNOWN;
    for (let i = 0; i < this.buffer.length; i++) if (this.buffer[i] === obj) this.buffer[i] = UNKNOWN;
  }
}

/** A fixed-function value: the last arguments set (UNKNOWN: unknown). */
class Value {
  a: unknown = UNKNOWN;
  b: unknown = UNKNOWN;
  c: unknown = UNKNOWN;
  d: unknown = UNKNOWN;
  /** Whether it holds (a, b, c, d); if not, it does from now on. */
  holds(a: unknown, b: unknown, c: unknown, d: unknown): boolean {
    if (this.a === a && this.b === b && this.c === c && this.d === d) return true;
    this.a = a;
    this.b = b;
    this.c = c;
    this.d = d;
    return false;
  }
  forget(): void {
    this.a = UNKNOWN;
  }
}

/**
 * Puts the filter on `gl`'s own methods (shadowing its prototype's) and
 * returns its counts. Install it before anything else draws through `gl`.
 */
export function filterGlState(gl: Gl): GlStateCounts {
  const counts: GlStateCounts = { calls: 0, forwarded: 0 };
  const raw = new Map<string, Fn>();
  for (let p = Object.getPrototypeOf(gl) as object | null; p && p !== Object.prototype; p = Object.getPrototypeOf(p) as object | null) {
    for (const k of Object.getOwnPropertyNames(p)) {
      const d = Object.getOwnPropertyDescriptor(p, k);
      if (k !== "constructor" && !raw.has(k) && d && typeof d.value === "function") raw.set(k, d.value as Fn);
    }
  }
  const noop: Fn = () => undefined;
  const fn = (name: string): Fn => raw.get(name) ?? noop;
  // The filter's methods live on a prototype put between the context and
  // its own: the context keeps no own properties, so its method lookups
  // stay as fast as the unfiltered ones.
  const methods = Object.create(Object.getPrototypeOf(gl) as object) as Record<string, unknown>;
  const define = (name: string, f: Fn) => {
    if (raw.has(name)) Object.defineProperty(methods, name, { value: f, configurable: true, writable: true });
  };

  // ── the mirror ────────────────────────────────────────────────────────
  // Capabilities by a small index (capIndex): as the context has them
  // (undefined: unknown), as asked (undefined: nothing asked), and those asked.
  const capIndex = new Map<number, number>();
  const capOf: number[] = [];
  const caps: (boolean | undefined)[] = [];
  const wantedCaps: (boolean | undefined)[] = [];
  const asked: number[] = [];
  const indexOf = (cap: number): number => {
    let i = capIndex.get(cap);
    if (i === undefined) {
      capIndex.set(cap, (i = capOf.length));
      capOf.push(cap);
    }
    return i;
  };
  const clearCaps = CLEAR_CAPS.map(indexOf);
  /** Active unit index (TEXTURE0 = 0) as the context has it, and as asked. */
  let unit: Known<number> = UNKNOWN;
  let wantedUnit: Known<number> = UNKNOWN;
  /** Texture bound per target, by unit index (undefined: unknown). */
  const textures = new Map<number, (Obj | undefined)[]>();
  const texParams = new Map<Obj, Map<number, unknown>>();
  const samplers: (Obj | undefined)[] = [];
  /** Buffer bound per generic target (not ELEMENT_ARRAY_BUFFER: the VAO's). */
  const buffers = new Map<number, Obj>();
  /** Indexed bindings per target, by index: buffer, offset, size (base: UNKNOWN offset). */
  const ranges = new Map<number, Value[]>();
  // Bindings as the context has them, and as asked (PENDING: nothing asked):
  // the program, the framebuffers and the generic buffer targets are bound
  // when a call reads them.
  let program: Known<Obj> = UNKNOWN;
  let wantedProgram: Obj | typeof PENDING = PENDING;
  let drawFb: Known<Obj> = UNKNOWN;
  let readFb: Known<Obj> = UNKNOWN;
  let wantedDraw: Obj | typeof PENDING = PENDING;
  let wantedRead: Obj | typeof PENDING = PENDING;
  const wantedBuffers = new Map<number, Obj>();
  const attachments = new Map<Obj, Map<number, Attachment>>();
  const drawBuffersOf = new Map<Obj, number[]>();
  const values = new Map<string, Value>();
  const pixelStore = new Map<number, unknown>();
  let vao: Known<Obj> = UNKNOWN;
  const vaos = new Map<Obj, VaoState>();
  /** The bound VAO's state (which one it is may be unknown: `vao`). */
  let current = new VaoState();
  const value = (key: string): Value => {
    let v = values.get(key);
    if (!v) values.set(key, (v = new Value()));
    return v;
  };

  const rEnable = fn("enable");
  const rDisable = fn("disable");
  const rActiveTexture = fn("activeTexture");
  const rEnableAttrib = fn("enableVertexAttribArray");
  const rDisableAttrib = fn("disableVertexAttribArray");
  /** Sets the capability `i` asked for, if one was. */
  const flushCap = (i: number) => {
    const on = wantedCaps[i];
    if (on === undefined) return;
    wantedCaps[i] = undefined;
    if (caps[i] !== on) {
      counts.forwarded++;
      (on ? rEnable : rDisable).call(gl, capOf[i]);
      caps[i] = on;
    }
  };
  /** Sets the capabilities asked for. */
  const flushCaps = () => {
    for (let k = 0; k < asked.length; k++) flushCap(asked[k]!);
    asked.length = 0;
  };
  /** Sets those a clear reads. */
  const flushClearCaps = () => {
    for (let k = 0; k < clearCaps.length; k++) flushCap(clearCaps[k]!);
  };
  const flushUnit = () => {
    if (wantedUnit !== unit && wantedUnit !== UNKNOWN) {
      counts.forwarded++;
      rActiveTexture.call(gl, TEXTURE0 + wantedUnit);
      unit = wantedUnit;
    }
  };
  const flushAttribs = () => {
    const v = current;
    const dirty = v.dirty;
    if (dirty.length === 0) return;
    for (let i = 0; i < dirty.length; i++) {
      const loc = dirty[i]!;
      const on = v.wanted[loc];
      if (on === undefined) continue;
      v.wanted[loc] = undefined;
      if (v.enabled[loc] !== on) {
        counts.forwarded++;
        (on ? rEnableAttrib : rDisableAttrib).call(gl, loc);
        v.enabled[loc] = on;
      }
    }
    dirty.length = 0;
  };
  const rUseProgram = fn("useProgram");
  const flushProgram = () => {
    if (wantedProgram === PENDING) return;
    const p = wantedProgram;
    wantedProgram = PENDING;
    if (program !== p) {
      counts.forwarded++;
      rUseProgram.call(gl, p);
      program = p;
    }
  };
  const rBindFramebuffer = fn("bindFramebuffer");
  /** Binds the framebuffers asked for (`draw`, `read`: which of them a call reads). */
  const flushFramebuffers = (draw: boolean, read: boolean) => {
    const d = draw && wantedDraw !== PENDING && wantedDraw !== drawFb ? wantedDraw : PENDING;
    const r = read && wantedRead !== PENDING && wantedRead !== readFb ? wantedRead : PENDING;
    if (draw) wantedDraw = PENDING;
    if (read) wantedRead = PENDING;
    if (d !== PENDING && r !== PENDING && d === r) {
      counts.forwarded++;
      rBindFramebuffer.call(gl, FRAMEBUFFER, d);
      drawFb = readFb = d;
      return;
    }
    if (d !== PENDING) {
      counts.forwarded++;
      rBindFramebuffer.call(gl, DRAW_FRAMEBUFFER, d);
      drawFb = d;
    }
    if (r !== PENDING) {
      counts.forwarded++;
      rBindFramebuffer.call(gl, READ_FRAMEBUFFER, r);
      readFb = r;
    }
  };
  const rBindBuffer = fn("bindBuffer");
  /** Binds the buffer asked for at `target`. */
  const flushBuffer = (target: number) => {
    if (wantedBuffers.size === 0 || !wantedBuffers.has(target)) return;
    const buf = wantedBuffers.get(target)!;
    wantedBuffers.delete(target);
    if (!buffers.has(target) || buffers.get(target) !== buf) {
      counts.forwarded++;
      rBindBuffer.call(gl, target, buf);
      buffers.set(target, buf);
    }
  };
  const flushBuffers = () => {
    for (const target of wantedBuffers.keys()) flushBuffer(target);
  };
  /** What a draw reads. */
  const flushDraw = () => {
    flushCaps();
    flushAttribs();
    flushProgram();
    flushFramebuffers(true, false);
  };
  const flushAll = () => {
    flushCaps();
    flushUnit();
    flushAttribs();
    flushProgram();
    flushFramebuffers(true, true);
    flushBuffers();
  };
  /** The framebuffer bound to `target` once the calls asked so far are made. */
  const framebufferAt = (target: unknown): Known<Obj> =>
    target === READ_FRAMEBUFFER ? (wantedRead !== PENDING ? wantedRead : readFb) : wantedDraw !== PENDING ? wantedDraw : drawFb;
  /** The buffer bound to `target` once the calls asked so far are made. */
  const bufferAt = (target: number): Known<Obj> => (wantedBuffers.has(target) ? wantedBuffers.get(target)! : buffers.has(target) ? buffers.get(target)! : UNKNOWN);

  /** Forget everything: the context may hold anything. */
  const forget = () => {
    flushAll();
    caps.length = 0;
    unit = UNKNOWN;
    textures.clear();
    texParams.clear();
    samplers.length = 0;
    buffers.clear();
    ranges.clear();
    program = UNKNOWN;
    drawFb = UNKNOWN;
    readFb = UNKNOWN;
    attachments.clear();
    drawBuffersOf.clear();
    for (const v of values.values()) v.forget();
    pixelStore.clear();
    vao = UNKNOWN;
    vaos.clear();
    current = new VaoState();
  };
  // A restored context starts from its defaults with new objects: nothing
  // asked of the lost one is set on it.
  const lost = () => {
    wantedCaps.length = 0;
    asked.length = 0;
    wantedProgram = wantedDraw = wantedRead = PENDING;
    wantedBuffers.clear();
    wantedUnit = UNKNOWN;
    current.dirty.length = 0;
    forget();
  };
  const canvas = gl.canvas as ((OffscreenCanvas | HTMLCanvasElement) & EventTarget) | undefined;
  canvas?.addEventListener?.("webglcontextlost", lost);
  canvas?.addEventListener?.("webglcontextrestored", lost);

  /** Forgets every binding to `obj` the context drops when it is deleted, and every value naming it. */
  const deleted = (obj: unknown) => {
    for (const list of textures.values()) for (let i = 0; i < list.length; i++) if (list[i] === obj) list[i] = null;
    for (let i = 0; i < samplers.length; i++) if (samplers[i] === obj) samplers[i] = null;
    for (const [t, b] of buffers) if (b === obj) buffers.set(t, null);
    for (const list of ranges.values()) for (const r of list) if (r && r.a === obj) r.forget();
    for (const map of attachments.values()) for (const [k, v] of map) if (v.obj === obj) map.delete(k);
    for (const s of vaos.values()) s.drop(obj);
    current.drop(obj);
    texParams.delete(obj as Obj);
    if (drawFb === obj) drawFb = null;
    if (readFb === obj) readFb = null;
    if (obj !== null) {
      attachments.delete(obj as Obj);
      drawBuffersOf.delete(obj as Obj);
    }
  };

  // ── deferred ──────────────────────────────────────────────────────────
  const want = (cap: number, on: boolean) => {
    const i = indexOf(cap);
    if (wantedCaps[i] === undefined) asked.push(i);
    wantedCaps[i] = on;
  };
  define("enable", (cap) => {
    counts.calls++;
    want(cap as number, true);
  });
  define("disable", (cap) => {
    counts.calls++;
    want(cap as number, false);
  });
  define("activeTexture", (u) => {
    counts.calls++;
    wantedUnit = (u as number) - TEXTURE0;
  });
  const wantAttrib = (loc: number, on: boolean) => {
    const v = current;
    if (v.wanted[loc] === undefined) v.dirty.push(loc);
    v.wanted[loc] = on;
  };
  define("enableVertexAttribArray", (loc) => {
    counts.calls++;
    wantAttrib(loc as number, true);
  });
  define("disableVertexAttribArray", (loc) => {
    counts.calls++;
    wantAttrib(loc as number, false);
  });

  // ── textures and samplers ─────────────────────────────────────────────
  const rBindTexture = fn("bindTexture");
  define("bindTexture", (target, tex) => {
    counts.calls++;
    let list = textures.get(target as number);
    if (wantedUnit !== UNKNOWN) {
      if (list && list[wantedUnit] === tex) return;
      if (!list) textures.set(target as number, (list = []));
    }
    flushUnit();
    if (wantedUnit !== UNKNOWN) list![wantedUnit] = tex as Obj;
    counts.forwarded++;
    return rBindTexture.call(gl, target, tex);
  });
  const rTexParameteri = fn("texParameteri");
  define("texParameteri", (target, pname, v) => {
    counts.calls++;
    const tex = wantedUnit === UNKNOWN ? undefined : textures.get(target as number)?.[wantedUnit];
    const params = tex ? texParams.get(tex) : undefined;
    if (params && params.get(pname as number) === v) return;
    flushUnit();
    if (tex) {
      if (params) params.set(pname as number, v);
      else texParams.set(tex, new Map([[pname as number, v]]));
    } else if (tex === undefined) {
      // Bound to an unknown texture: any one's parameters may change.
      texParams.clear();
    }
    counts.forwarded++;
    return rTexParameteri.call(gl, target, pname, v);
  });
  // Not mirrored: it forgets the parameter it sets (every texture's
  // parameters, when which one is bound is unknown).
  const rTexParameterf = fn("texParameterf");
  define("texParameterf", (target, pname, v) => {
    counts.calls++;
    const tex = wantedUnit === UNKNOWN ? undefined : textures.get(target as number)?.[wantedUnit];
    flushUnit();
    if (tex) texParams.get(tex)?.delete(pname as number);
    else if (tex === undefined) texParams.clear();
    counts.forwarded++;
    return rTexParameterf.call(gl, target, pname, v);
  });
  const rBindSampler = fn("bindSampler");
  define("bindSampler", (u, s) => {
    counts.calls++;
    if (samplers[u as number] === s) return;
    samplers[u as number] = s as Obj;
    counts.forwarded++;
    return rBindSampler.call(gl, u, s);
  });

  // ── buffers ───────────────────────────────────────────────────────────
  define("bindBuffer", (target, buf) => {
    counts.calls++;
    if (target !== ELEMENT_ARRAY_BUFFER) {
      // Deferred: bound when a call reads the target.
      wantedBuffers.set(target as number, buf as Obj);
      return;
    }
    if (current.elements === buf) return;
    current.elements = buf as Obj;
    counts.forwarded++;
    return rBindBuffer.call(gl, target, buf);
  });
  // The calls that read a generic target's buffer.
  // (Overloaded: forwarded with the arguments as given.)
  for (const name of ["bufferData", "bufferSubData"]) {
    const r = fn(name);
    define(name, (...a) => {
      counts.calls++;
      flushBuffer(a[0] as number);
      counts.forwarded++;
      return r.apply(gl, a);
    });
  }
  const rCopyBufferSubData = fn("copyBufferSubData");
  define("copyBufferSubData", (read, write, a, b, c) => {
    counts.calls++;
    flushBuffer(read as number);
    flushBuffer(write as number);
    counts.forwarded++;
    return rCopyBufferSubData.call(gl, read, write, a, b, c);
  });
  /** An indexed binding; it also binds the generic target. */
  const range = (target: number, index: number, buf: unknown, offset: unknown, size: unknown): boolean => {
    let list = ranges.get(target);
    if (!list) ranges.set(target, (list = []));
    let r = list[index];
    if (!r) list[index] = r = new Value();
    // A binding asked of the generic target before is overridden.
    wantedBuffers.delete(target);
    const generic = buffers.get(target) === buf && buffers.has(target);
    const held = r.holds(buf, offset, size, 0);
    buffers.set(target, buf as Obj);
    return held && generic;
  };
  const rBindBufferRange = fn("bindBufferRange");
  define("bindBufferRange", (target, index, buf, offset, size) => {
    counts.calls++;
    if (range(target as number, index as number, buf, offset, size)) return;
    counts.forwarded++;
    return rBindBufferRange.call(gl, target, index, buf, offset, size);
  });
  const rBindBufferBase = fn("bindBufferBase");
  define("bindBufferBase", (target, index, buf) => {
    counts.calls++;
    if (range(target as number, index as number, buf, "base", "base")) return;
    counts.forwarded++;
    return rBindBufferBase.call(gl, target, index, buf);
  });
  const rBindTransformFeedback = fn("bindTransformFeedback");
  define("bindTransformFeedback", (target, tf) => {
    counts.calls++;
    flushAll();
    // The indexed transform feedback bindings are the object's.
    ranges.delete(TRANSFORM_FEEDBACK_BUFFER);
    buffers.delete(TRANSFORM_FEEDBACK_BUFFER);
    wantedBuffers.delete(TRANSFORM_FEEDBACK_BUFFER);
    counts.forwarded++;
    return rBindTransformFeedback.call(gl, target, tf);
  });
  for (const name of ["deleteTexture", "deleteBuffer", "deleteFramebuffer", "deleteRenderbuffer", "deleteSampler"]) {
    const r = fn(name);
    define(name, (obj) => {
      counts.calls++;
      flushAll();
      deleted(obj);
      counts.forwarded++;
      return r.call(gl, obj);
    });
  }

  // ── vertex arrays and attributes ──────────────────────────────────────
  const rBindVertexArray = fn("bindVertexArray");
  define("bindVertexArray", (v) => {
    counts.calls++;
    if (vao === v) return;
    flushAttribs();
    vao = v as Obj;
    let s = vaos.get(vao);
    if (!s) vaos.set(vao, (s = new VaoState()));
    current = s;
    counts.forwarded++;
    return rBindVertexArray.call(gl, v);
  });
  const rDeleteVertexArray = fn("deleteVertexArray");
  define("deleteVertexArray", (v) => {
    counts.calls++;
    flushAttribs();
    if (v !== null) vaos.delete(v as Obj);
    // Deleting the bound one binds the default; which one is bound may be unknown.
    if (vao === v) {
      vao = null;
      let s = vaos.get(null);
      if (!s) vaos.set(null, (s = new VaoState()));
      current = s;
    } else if (vao === UNKNOWN) {
      current = new VaoState();
    }
    counts.forwarded++;
    return rDeleteVertexArray.call(gl, v);
  });
  /** Whether location `loc` points as asked (it does from now on); a pointer takes ARRAY_BUFFER's buffer. */
  const points = (loc: number, integer: boolean, size: unknown, type: unknown, norm: unknown, stride: unknown, offset: unknown): boolean => {
    const v = current;
    const buf = bufferAt(ARRAY_BUFFER);
    if (buf !== UNKNOWN && v.buffer[loc] === buf && v.integer[loc] === integer && v.size[loc] === size && v.type[loc] === type && v.norm[loc] === norm && v.stride[loc] === stride && v.offset[loc] === offset) return true;
    v.buffer[loc] = buf;
    v.integer[loc] = integer;
    v.size[loc] = size;
    v.type[loc] = type;
    v.norm[loc] = norm;
    v.stride[loc] = stride;
    v.offset[loc] = offset;
    return false;
  };
  const rPointer = fn("vertexAttribPointer");
  define("vertexAttribPointer", (loc, size, type, norm, stride, offset) => {
    counts.calls++;
    if (points(loc as number, false, size, type, norm, stride, offset)) return;
    flushBuffer(ARRAY_BUFFER);
    counts.forwarded++;
    return rPointer.call(gl, loc, size, type, norm, stride, offset);
  });
  const rIPointer = fn("vertexAttribIPointer");
  define("vertexAttribIPointer", (loc, size, type, stride, offset) => {
    counts.calls++;
    if (points(loc as number, true, size, type, undefined, stride, offset)) return;
    flushBuffer(ARRAY_BUFFER);
    counts.forwarded++;
    return rIPointer.call(gl, loc, size, type, stride, offset);
  });
  const rDivisor = fn("vertexAttribDivisor");
  define("vertexAttribDivisor", (loc, d) => {
    counts.calls++;
    if (current.divisor[loc as number] === d) return;
    current.divisor[loc as number] = d;
    counts.forwarded++;
    return rDivisor.call(gl, loc, d);
  });

  // ── programs and uniforms ─────────────────────────────────────────────
  // Deferred: bound when a draw or a uniform reads it.
  define("useProgram", (p) => {
    counts.calls++;
    wantedProgram = p as Obj;
  });
  // A null location is ignored by WebGL (wgpu sets its first-instance
  // uniform on every draw whether the program has one or not).
  for (const name of raw.keys()) {
    if (!/^uniform(Matrix)?[1-4]/.test(name)) continue;
    const r = fn(name);
    define(name, (...a) => {
      counts.calls++;
      if (a[0] === null || a[0] === undefined) return;
      flushProgram();
      counts.forwarded++;
      return r.apply(gl, a);
    });
  }

  // ── framebuffers ──────────────────────────────────────────────────────
  // Deferred: bound when a call reads the target.
  define("bindFramebuffer", (target, fb) => {
    counts.calls++;
    if (target !== FRAMEBUFFER && target !== DRAW_FRAMEBUFFER && target !== READ_FRAMEBUFFER) {
      flushAll();
      counts.forwarded++;
      return rBindFramebuffer.call(gl, target, fb);
    }
    if (target !== READ_FRAMEBUFFER) wantedDraw = fb as Obj;
    if (target !== DRAW_FRAMEBUFFER) wantedRead = fb as Obj;
  });
  /** Binds the framebuffer a call on `target` reads. */
  const flushTarget = (target: unknown) => flushFramebuffers(target !== READ_FRAMEBUFFER, target === READ_FRAMEBUFFER || target === FRAMEBUFFER);
  /** Whether attachment point `at` of the framebuffer bound to `target` holds what is asked (it does from now on). */
  const attached = (target: unknown, at: number, kind: number, obj: Obj, a: unknown, b: unknown): boolean => {
    const fb = framebufferAt(target);
    // Bound to an unknown framebuffer: any one's attachments may change.
    if (fb === UNKNOWN) {
      attachments.clear();
      return false;
    }
    if (fb === null) return false;
    let map = attachments.get(fb);
    if (!map) attachments.set(fb, (map = new Map()));
    const holds = (p: number) => {
      const v = map!.get(p);
      return v !== undefined && v.kind === kind && v.obj === obj && v.a === a && v.b === b;
    };
    const value = kind === 0 ? NONE : { kind, obj, a, b };
    // DEPTH_STENCIL_ATTACHMENT is the depth and the stencil points at once.
    if (at === DEPTH_STENCIL_ATTACHMENT) {
      if (holds(DEPTH_ATTACHMENT) && holds(STENCIL_ATTACHMENT)) return true;
      map.set(DEPTH_ATTACHMENT, value);
      map.set(STENCIL_ATTACHMENT, value);
      return false;
    }
    if (holds(at)) return true;
    map.set(at, value);
    return false;
  };
  const rTexture2D = fn("framebufferTexture2D");
  define("framebufferTexture2D", (target, at, textarget, tex, level) => {
    counts.calls++;
    if (tex === null ? attached(target, at as number, 0, null, 0, 0) : attached(target, at as number, 1, tex as Obj, textarget, level)) return;
    flushTarget(target);
    counts.forwarded++;
    return rTexture2D.call(gl, target, at, textarget, tex, level);
  });
  const rTextureLayer = fn("framebufferTextureLayer");
  define("framebufferTextureLayer", (target, at, tex, level, layer) => {
    counts.calls++;
    if (tex === null ? attached(target, at as number, 0, null, 0, 0) : attached(target, at as number, 2, tex as Obj, level, layer)) return;
    flushTarget(target);
    counts.forwarded++;
    return rTextureLayer.call(gl, target, at, tex, level, layer);
  });
  const rRenderbuffer = fn("framebufferRenderbuffer");
  define("framebufferRenderbuffer", (target, at, rbTarget, rb) => {
    counts.calls++;
    if (rb === null ? attached(target, at as number, 0, null, 0, 0) : attached(target, at as number, 3, rb as Obj, 0, 0)) return;
    flushTarget(target);
    counts.forwarded++;
    return rRenderbuffer.call(gl, target, at, rbTarget, rb);
  });
  const rDrawBuffers = fn("drawBuffers");
  define("drawBuffers", (list) => {
    counts.calls++;
    const l = list as ArrayLike<number>;
    const fb = framebufferAt(DRAW_FRAMEBUFFER);
    if (fb === UNKNOWN) drawBuffersOf.clear();
    else {
      const held = drawBuffersOf.get(fb);
      let same = held !== undefined && held.length === l.length;
      for (let i = 0; same && i < l.length; i++) same = held![i] === l[i];
      if (same) return;
      drawBuffersOf.set(fb, Array.from(l));
    }
    flushFramebuffers(true, false);
    counts.forwarded++;
    return rDrawBuffers.call(gl, list);
  });

  // ── fixed-function values ─────────────────────────────────────────────
  /**
   * A setter of `arity` (up to four) values, dropped when the context holds
   * them (`held`: whether it does, by default the arguments' own value `key`).
   */
  const setter = (name: string, arity: number, key: string, held?: (a: unknown, b: unknown, c: unknown, d: unknown) => boolean) => {
    const own = value(key);
    const spread = held ?? ((a: unknown, b: unknown, c: unknown, d: unknown) => own.holds(a, b, c, d));
    const r = fn(name);
    define(name, (a, b, c, d) => {
      counts.calls++;
      if (spread(a, b, c, d)) return;
      counts.forwarded++;
      return arity === 1 ? r.call(gl, a) : arity === 2 ? r.call(gl, a, b) : arity === 3 ? r.call(gl, a, b, c) : r.call(gl, a, b, c, d);
    });
  };
  for (const name of ["depthMask", "frontFace", "cullFace", "depthFunc", "clearDepth", "clearStencil"]) setter(name, 1, name);
  for (const name of ["depthRange", "polygonOffset"]) setter(name, 2, name);
  for (const name of ["colorMask", "viewport", "scissor", "blendColor", "clearColor"]) setter(name, 4, name);
  const equation = value("blendEquation");
  setter("blendEquation", 1, "", (a) => equation.holds(a, a, 0, 0));
  setter("blendEquationSeparate", 2, "", (a, b) => equation.holds(a, b, 0, 0));
  const func = value("blendFunc");
  setter("blendFunc", 2, "", (a, b) => func.holds(a, b, a, b));
  setter("blendFuncSeparate", 4, "blendFunc");
  // Stencil state per face: the plain forms and FRONT_AND_BACK set both.
  const faced = (key: string) => {
    const f = value(`${key}:front`);
    const k = value(`${key}:back`);
    return (face: unknown, a: unknown, b: unknown, c: unknown) => {
      const front = face !== BACK;
      const back = face === BACK || face === FRONT_AND_BACK || face === undefined;
      // Both checked before either is set: holds() takes the new value.
      const held = (!front || (f.a === a && f.b === b && f.c === c && f.d === 0)) && (!back || (k.a === a && k.b === b && k.c === c && k.d === 0));
      if (front) f.holds(a, b, c, 0);
      if (back) k.holds(a, b, c, 0);
      return held;
    };
  };
  const isFace = (face: unknown) => face === FRONT || face === BACK || face === FRONT_AND_BACK;
  for (const key of ["stencilMask", "stencilFunc", "stencilOp"]) {
    const of = faced(key);
    if (key === "stencilMask") setter(key, 1, "", (a) => of(undefined, a, 0, 0));
    else setter(key, 3, "", (a, b, c) => of(undefined, a, b, c));
    if (key === "stencilMask") setter(`${key}Separate`, 2, "", (face, a) => isFace(face) && of(face, a, 0, 0));
    else setter(`${key}Separate`, 4, "", (face, a, b, c) => isFace(face) && of(face, a, b, c));
  }
  const rPixelStorei = fn("pixelStorei");
  define("pixelStorei", (pname, v) => {
    counts.calls++;
    if (pixelStore.get(pname as number) === v && pixelStore.has(pname as number)) return;
    pixelStore.set(pname as number, v);
    counts.forwarded++;
    return rPixelStorei.call(gl, pname, v);
  });

  // ── extensions: their calls change state the mirror cannot see ────────
  const wrapped = new WeakSet<object>();
  const rGetExtension = fn("getExtension");
  define("getExtension", (name) => {
    counts.calls++;
    counts.forwarded++;
    const ext = rGetExtension.call(gl, name) as Record<string, unknown> | null;
    if (!ext || wrapped.has(ext)) return ext;
    wrapped.add(ext);
    for (let p = Object.getPrototypeOf(ext) as object | null; p && p !== Object.prototype; p = Object.getPrototypeOf(p) as object | null) {
      for (const k of Object.getOwnPropertyNames(p)) {
        const d = Object.getOwnPropertyDescriptor(p, k);
        if (k === "constructor" || !d || typeof d.value !== "function" || Object.prototype.hasOwnProperty.call(ext, k)) continue;
        const f = d.value as Fn;
        Object.defineProperty(ext, k, {
          value: (...a: unknown[]) => {
            flushAll();
            const r = f.apply(ext, a);
            forget();
            return r;
          },
          configurable: true,
          writable: true,
        });
      }
    }
    return ext;
  });

  // ── everything else ───────────────────────────────────────────────────
  // The calls on a framebuffer target: they read its binding.
  for (const name of ["invalidateFramebuffer", "invalidateSubFramebuffer"]) {
    const r = fn(name);
    define(name, (...a) => {
      counts.calls++;
      flushTarget(a[0]);
      counts.forwarded++;
      return r.apply(gl, a);
    });
  }
  const rReadBuffer = fn("readBuffer");
  define("readBuffer", (src) => {
    counts.calls++;
    flushFramebuffers(false, true);
    counts.forwarded++;
    return rReadBuffer.call(gl, src);
  });
  const independent = new Set(INDEPENDENT);
  const clears = new Set(CLEARS);
  const draws = new Set(DRAWS);
  const observing = new Set(OBSERVING);
  for (const [name, r] of raw) {
    if (Object.prototype.hasOwnProperty.call(methods, name)) continue;
    if (independent.has(name)) {
      define(name, (...a) => {
        counts.calls++;
        counts.forwarded++;
        return r.apply(gl, a);
      });
    } else if (draws.has(name)) {
      define(name, (...a) => {
        counts.calls++;
        flushDraw();
        counts.forwarded++;
        return r.apply(gl, a);
      });
    } else if (clears.has(name)) {
      define(name, (...a) => {
        counts.calls++;
        flushClearCaps();
        flushFramebuffers(true, false);
        counts.forwarded++;
        return r.apply(gl, a);
      });
    } else if (observing.has(name)) {
      define(name, (...a) => {
        counts.calls++;
        flushAll();
        counts.forwarded++;
        return r.apply(gl, a);
      });
    } else {
      // Not modelled: it may change anything.
      define(name, (...a) => {
        counts.calls++;
        flushAll();
        counts.forwarded++;
        const result = r.apply(gl, a);
        forget();
        return result;
      });
    }
  }
  Object.setPrototypeOf(gl, methods);
  return counts;
}

/**
 * Filters the WebGL2 context `canvas` hands out (wgpu's GL backend asks
 * for it while the renderer attaches): installed on the canvas's
 * getContext before then.
 */
export function filterGlStateOf(canvas: OffscreenCanvas): void {
  const get = canvas.getContext.bind(canvas) as (id: string, options?: unknown) => unknown;
  const filtered = new WeakSet<object>();
  Object.defineProperty(canvas, "getContext", {
    value: (id: string, options?: unknown) => {
      const ctx = get(id, options);
      if (id === "webgl2" && ctx && !filtered.has(ctx as object)) {
        filtered.add(ctx as object);
        filterGlState(ctx as Gl);
      }
      return ctx;
    },
    configurable: true,
    writable: true,
  });
}
