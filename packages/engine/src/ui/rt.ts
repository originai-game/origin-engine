// Ported from NSH (longbridge/nsh 6fa6368f, Apache-2.0, third_party/nsh-jsx/LICENSE-APACHE):
// nsh-platform-web/web/browser/dom.js's renderer calls.
// The renderer compiled TSX calls (crates/og-ui-next/CONVENTIONS.md §JSX
// compile, §Ops): nsh-jsx turns a host element into `element` + `prop` +
// `insert` calls on this module, the way NSH's renderer (nsh:rt) takes them.
// Each call writes ops into one reused buffer per world; `flush` sends the
// batch through og_engine_ui_patch and dispatches the events of its reply to
// the handlers by node id. Nodes are plain handles: their state lives in Rust.

import type { World } from "@origingame/engine";
import {
  type Accessor,
  createMemo,
  createRenderEffect,
  createRoot,
  createContext,
  createSignal,
  getOwner,
  onCleanup,
  runWithOwner,
  untrack,
  useContext,
  batch,
} from "./signals.ts";

const TAGS: Record<string, number> = { view: 0, text: 1, image: 2, scroll: 3, input: 4, icon: 5, line: 6 };
/** Event codes (§Ops) and the handler props they call. */
const EVENTS = ["onPress", "onTap", "onPointerDown", "onPointerMove", "onPointerUp", "onWheel", "onFocus", "onBlur", "onKey", "onChange", "onAction", "onLongPress", "onHold"] as const;
const EVENT_CODE = new Map<string, number>(EVENTS.map((name, code) => [name, code]));
const APPEND = 0xffff_ffff;

/** A node the renderer made: a handle on the Rust tree's node `id`. */
export class UiNode {
  /** The text of a run `insert` made for a string, which it keeps for the next one. */
  text?: string;
  readonly id: number;
  readonly ui: Renderer;
  constructor(id: number, ui: Renderer) { this.id = id; this.ui = ui; }
}

/** What an event handler is given. */
export interface UiEvent {
  /** The world the UI belongs to, for writing intents and commands. */
  readonly world: World;
  readonly type: string;
  /** The pointer on the UI's surface. */
  readonly x: number;
  readonly y: number;
  /** A key's name (`onKey`) or the input action (`onAction`). */
  readonly key: string;
  /** A control's new value (`onChange`). */
  readonly value: string;
}

type Transport = (bytes: Uint8Array) => Uint8Array;

const encoder = new TextEncoder();
const decoder = new TextDecoder();

/** One world's UI: its op buffer, its ids and its handlers. */
export class Renderer {
  private buf = new ArrayBuffer(4096);
  private view = new DataView(this.buf);
  private bytes = new Uint8Array(this.buf);
  private at = 8;
  private count = 0;
  private nextId = 1;
  private readonly free: number[] = [];
  private readonly handlers = new Map<number, Map<string, (e: UiEvent) => void>>();
  private readonly owners = new Map<number, unknown>();
  /** Each node's children as the renderer put them, to free their ids with it. */
  private readonly kids = new Map<number, Set<number>>();
  private readonly parents = new Map<number, number>();
  /** Nodes taken out of their parent this batch: released at the flush unless put back. */
  private readonly orphans = new Set<number>();
  private flushing = false;
  errors: string[] = [];

  private readonly transport: Transport;
  readonly world: World;
  constructor(transport: Transport, world: World) { this.transport = transport; this.world = world; }

  private room(n: number): void {
    if (this.at + n <= this.buf.byteLength) return;
    let size = this.buf.byteLength * 2;
    while (size < this.at + n) size *= 2;
    const next = new ArrayBuffer(size);
    new Uint8Array(next).set(this.bytes.subarray(0, this.at));
    this.buf = next;
    this.view = new DataView(next);
    this.bytes = new Uint8Array(next);
  }
  private u8(v: number): void { this.room(1); this.view.setUint8(this.at, v); this.at += 1; }
  private u32(v: number): void { this.room(4); this.view.setUint32(this.at, v, true); this.at += 4; }
  private str(s: string): void {
    // UTF-8 is at most three bytes per UTF-16 unit.
    this.room(4 + s.length * 3);
    const { written } = encoder.encodeInto(s, this.bytes.subarray(this.at + 4));
    this.view.setUint32(this.at, written, true);
    this.at += 4 + written;
  }
  private op(code: number): void { this.count++; this.u8(code); }

  private alloc(): UiNode { return new UiNode(this.free.pop() ?? this.nextId++, this); }

  element(tag: string): UiNode {
    const code = TAGS[tag];
    if (code === undefined) throw new Error(`og.ui: no host element <${tag}>`);
    const node = this.alloc();
    this.op(1); this.u32(node.id); this.u8(code);
    return node;
  }
  text(value: string): UiNode {
    const node = this.alloc();
    this.op(2); this.u32(node.id); this.str(value);
    return node;
  }
  placeholder(): UiNode {
    const node = this.alloc();
    this.op(11); this.u32(node.id);
    return node;
  }
  setText(node: UiNode, value: string): void { this.op(3); this.u32(node.id); this.str(value); }
  setClass(node: UiNode, value: string): void { this.op(4); this.u32(node.id); this.str(value); }
  setStyle(node: UiNode, value: string): void { this.op(5); this.u32(node.id); this.str(value); }
  setProp(node: UiNode, name: string, value: string): void { this.op(6); this.u32(node.id); this.str(name); this.str(value); }
  listen(node: UiNode, name: string, handler: ((e: UiEvent) => void) | undefined): void {
    const code = EVENT_CODE.get(name)!;
    let map = this.handlers.get(node.id);
    if (!map) this.handlers.set(node.id, (map = new Map()));
    const had = map.has(name);
    if (handler) { map.set(name, handler); this.owners.set(node.id, getOwner()); } else map.delete(name);
    if (had !== Boolean(handler)) { this.op(7); this.u32(node.id); this.u8(code); this.u8(handler ? 1 : 0); }
  }
  insert(parent: number, child: UiNode, before: UiNode | null): void {
    this.op(8); this.u32(parent); this.u32(child.id); this.u32(before ? before.id : APPEND);
    const old = this.parents.get(child.id);
    if (old !== undefined) this.kids.get(old)?.delete(child.id);
    this.parents.set(child.id, parent);
    let set = this.kids.get(parent);
    if (!set) this.kids.set(parent, (set = new Set()));
    set.add(child.id);
    this.orphans.delete(child.id);
  }
  remove(parent: number, child: UiNode): void {
    this.op(9); this.u32(parent); this.u32(child.id);
    this.kids.get(parent)?.delete(child.id);
    this.parents.delete(child.id);
    this.orphans.add(child.id);
  }
  private forget(id: number): void {
    for (const kid of this.kids.get(id) ?? []) this.forget(kid);
    this.kids.delete(id);
    this.parents.delete(id);
    this.handlers.delete(id);
    this.owners.delete(id);
    this.free.push(id);
  }

  /**
   * Sends the ops written since the last flush (an empty batch asks only for
   * the events) and runs the handlers of the events in the reply. Their writes
   * are sent before this returns.
   */
  flush(): void {
    if (this.flushing) return;
    this.flushing = true;
    try {
      for (let round = 0; round < 16; round++) {
        for (const id of this.orphans) { this.op(10); this.u32(id); this.forget(id); }
        this.orphans.clear();
        this.bytes.set([0x4f, 0x47, 0x4f, 0x50]); // OGOP
        this.view.setUint32(4, this.count, true);
        const reply = this.transport(this.bytes.subarray(0, this.at));
        this.at = 8;
        this.count = 0;
        const dispatched = this.events(reply);
        if (!dispatched || this.count === 0) return;
      }
    } finally {
      this.flushing = false;
    }
  }

  /** Runs the reply's handlers; whether there were any. */
  private events(reply: Uint8Array): boolean {
    const v = new DataView(reply.buffer, reply.byteOffset, reply.byteLength);
    const magic = decoder.decode(reply.subarray(0, 4));
    if (magic === "OGER") {
      this.errors.push(decoder.decode(reply.subarray(4)));
      return false;
    }
    if (magic !== "OGEV") {
      this.errors.push("og.ui: the host's reply to an op batch is not OGEV or OGER");
      return false;
    }
    let at = 4;
    const n = v.getUint32(at, true); at += 4;
    const str = () => { const len = v.getUint32(at, true); at += 4; const s = decoder.decode(reply.subarray(at, at + len)); at += len; return s; };
    const due: [(e: UiEvent) => void, unknown, UiEvent][] = [];
    for (let i = 0; i < n; i++) {
      const id = v.getUint32(at, true); at += 4;
      const code = v.getUint8(at); at += 1;
      const x = v.getFloat32(at, true), y = v.getFloat32(at + 4, true); at += 8;
      const key = str(), value = str();
      const name = EVENTS[code];
      const handler = name && this.handlers.get(id)?.get(name);
      if (handler) due.push([handler, this.owners.get(id), { world: this.world, type: name.slice(2).toLowerCase(), x, y, key, value }]);
    }
    for (const [handler, owner, event] of due) {
      try { batch(() => runWithOwner(owner, () => untrack(() => handler(event)))); }
      catch (error) { this.errors.push(String(error instanceof Error ? (error.stack ?? error.message) : error)); }
    }
    return due.length > 0;
  }
}

// ── the calls compiled code makes ──────────────────────────────────────────

/** The mount's renderer, for everything made under it (rows made later too). */
export const RendererContext = createContext<Renderer | null>(null);

function renderer(): Renderer {
  const ui = useContext(RendererContext);
  if (!ui) throw new Error("og.ui: UI elements are made inside a mount (mount from @origingame/engine/ui)");
  return ui;
}

export function element(tag: string): UiNode { return renderer().element(tag); }
export function text(value: unknown): UiNode { return renderer().text(String(value)); }

function styleText(value: unknown): string {
  if (value == null || value === false) return "";
  if (typeof value === "string") return value;
  return Object.entries(value as Record<string, unknown>)
    .filter(([, v]) => v != null && v !== false)
    .map(([k, v]) => `${k.replace(/[A-Z]/g, (c) => `-${c.toLowerCase()}`)}: ${String(v)}`)
    .join("; ");
}

function scalar(value: unknown): string {
  if (value == null || value === false) return value === false ? "false" : "";
  return String(value);
}

/** Sets prop `name` of `node` once (a static value, or a handler). */
export function prop(node: UiNode, name: string, value: unknown): void {
  const ui = node.ui;
  if (name === "children") return;
  if (name === "ref") { ref(value, node); return; }
  if (name === "class") { ui.setClass(node, typeof value === "string" ? value : ""); return; }
  if (name === "classList") {
    const list = Object.entries((value ?? {}) as Record<string, unknown>).filter(([, on]) => on).map(([c]) => c);
    ui.setClass(node, list.join(" "));
    return;
  }
  if (name === "style") { ui.setStyle(node, styleText(value)); return; }
  if (name === "onAction") {
    // Input actions where focus is: their names as one prop, one listener.
    const actions = (value ?? {}) as Record<string, ((e: UiEvent) => void) | undefined>;
    const names = Object.keys(actions).filter((k) => typeof actions[k] === "function");
    ui.setProp(node, "actions", names.join(" "));
    ui.listen(node, "onAction", names.length ? (e) => actions[e.key]?.(e) : undefined);
    return;
  }
  if (EVENT_CODE.has(name)) { ui.listen(node, name, typeof value === "function" ? (value as (e: UiEvent) => void) : undefined); return; }
  ui.setProp(node, name, scalar(value));
}

/** Follows prop `name`: set again whenever `read`'s value changes. */
export function bindProp(node: UiNode, name: string, read: () => unknown): void {
  createRenderEffect(() => prop(node, name, read()));
}

/** Sets every prop of `props`, followed; a prop that goes is unset. */
export function spread(node: UiNode, props: Record<string, unknown>): void {
  let keys: string[] = [];
  createRenderEffect(() => {
    const next = Object.keys(props).filter((k) => k !== "children");
    for (const key of keys) if (!next.includes(key)) prop(node, key, undefined);
    for (const key of next) prop(node, key, props[key]);
    keys = next;
  });
}

/** What a child expression may give: nodes, text, nothing, lists and functions of them. */
export type UiChild = UiNode | string | number | boolean | null | undefined | UiChild[] | (() => UiChild);

/** The nodes `value` is now, text nodes reused from `previous` where they were text. */
function nodes(ui: Renderer, value: unknown, previous: UiNode[], out: UiNode[], texts: { i: number }): void {
  if (value == null || typeof value === "boolean") return;
  if (value instanceof UiNode) { out.push(value); return; }
  if (Array.isArray(value)) { for (const item of value) nodes(ui, item, previous, out, texts); return; }
  if (typeof value === "function") { nodes(ui, (value as () => unknown)(), previous, out, texts); return; }
  const s = String(value);
  // A text run that was text before keeps its node.
  while (texts.i < previous.length && previous[texts.i]!.text === undefined) texts.i++;
  const reuse = previous[texts.i];
  if (reuse) {
    texts.i++;
    if (reuse.text !== s) { ui.setText(reuse, s); reuse.text = s; }
    out.push(reuse);
    return;
  }
  const made = ui.text(s);
  made.text = s;
  out.push(made);
}

function dynamic(value: unknown, depth = 0): boolean {
  return typeof value === "function" || (depth < 64 && Array.isArray(value) && value.some((v) => dynamic(v, depth + 1)));
}

/** Puts `value`'s nodes in `parent` before `before` (last without), followed. */
export function insert(parent: UiNode, value: unknown, before: UiNode | null = null): void {
  const ui = parent.ui;
  let previous: UiNode[] = [];
  const show = (v: unknown) => {
    const next: UiNode[] = [];
    nodes(ui, v, previous, next, { i: 0 });
    const kept = new Set(next);
    for (const old of previous) if (!kept.has(old)) ui.remove(parent.id, old);
    const same = next.length === previous.length && next.every((n, i) => n === previous[i]);
    if (!same) for (const n of next) ui.insert(parent.id, n, before);
    previous = next;
  };
  // A list with functions in it (consecutive dynamic children, as the
  // compile groups them) is followed like a function.
  if (!dynamic(value)) { show(value); return; }
  createRenderEffect(() => {
    const v = typeof value === "function" ? (value as () => unknown)() : value;
    // A function returned is followed by an effect of its own, so what it
    // reads does not make the outer components again.
    if (typeof v === "function") createRenderEffect(() => show((v as () => unknown)()));
    else show(v);
  });
}

/** A child node or text, set once (`append`) or followed (`insert`). */
export function append(parent: UiNode, child: unknown): void {
  if (child instanceof UiNode) parent.ui.insert(parent.id, child, null);
  else insert(parent, child);
}

/** Calls a component once, untracked: its body is not followed. */
export function component<P>(Component: (props: P) => unknown, props: P): unknown {
  return untrack(() => Component(props));
}

/** `<Show when fallback keyed>`: the branch for `when`, made again when its truthiness (keyed: its value) changes. */
export function cond(when: () => unknown, then: () => unknown, otherwise?: () => unknown, keyed?: boolean): Accessor<unknown> {
  const condition = createMemo(() => (keyed ? when() : Boolean(when())));
  return createMemo(() => {
    const render = condition() ? then : otherwise;
    return typeof render === "function" ? untrack(render) : undefined;
  });
}

interface Row { item: (() => unknown) & { set(v: unknown): void }; index: (() => number) & { set(v: number): void }; output: unknown; dispose: () => void }

/** `<For each key>`: one row per item, kept while its key is (by default the item itself). */
export function each<T>(source: () => readonly T[] | null | undefined, render: (item: () => T, index: () => number) => unknown, key?: (item: T) => unknown): Accessor<unknown[]> {
  let rows = new Map<unknown, Row[]>();
  onCleanup(() => { for (const list of rows.values()) for (const row of list) row.dispose(); });
  const owner = getOwner();
  return createMemo(() => {
    const items = source() ?? [];
    return untrack(() => {
      const next = new Map<unknown, Row[]>();
      const out: unknown[] = [];
      items.forEach((item, i) => {
        const k = key ? key(item) : item;
        let row = rows.get(k)?.shift();
        if (row) {
          row.item.set(item);
          row.index.set(i);
        } else {
          row = runWithOwner(owner, () => createRoot((dispose) => {
            const it = createSignal<unknown>(item), index = createSignal(i);
            return { item: it, index, dispose, output: render(it as () => T, index) } as Row;
          }));
        }
        let list = next.get(k);
        if (!list) next.set(k, (list = []));
        list.push(row);
        out.push(row.output);
      });
      for (const list of rows.values()) for (const row of list) row.dispose();
      rows = next;
      return out;
    });
  });
}

/** Props from several sources, later ones winning, read through (getters kept). */
export function merge(...sources: unknown[]): Record<string, unknown> {
  const read = (s: unknown) => (typeof s === "function" ? (s as () => unknown)() : s) as Record<string, unknown> | null | undefined;
  return new Proxy(Object.create(null), {
    get(_, name) {
      for (let i = sources.length - 1; i >= 0; i--) {
        const s = read(sources[i]);
        if (s && name in s) return s[name as string];
      }
      return undefined;
    },
    has: (_, name) => sources.some((s) => { const v = read(s); return v != null && name in v; }),
    ownKeys: () => [...new Set(sources.flatMap((s) => Reflect.ownKeys(read(s) ?? {})))],
    getOwnPropertyDescriptor: () => ({ enumerable: true, configurable: true }),
  });
}

/** `ref={x}`: a signal is set, a function called with the node; false when neither (the compile assigns). */
export function ref(target: unknown, node: UiNode, name?: string): boolean {
  if (typeof target === "function") {
    if ("set" in target && typeof (target as { set?: unknown }).set === "function") (target as { set(n: UiNode): void }).set(node);
    else untrack(() => (target as (n: UiNode) => void)(node));
    return true;
  }
  if (typeof name === "string") throw new TypeError(`ref={${name}}: binding cannot be assigned and is neither a signal nor a function`);
  return false;
}

export function metaUrl(path: string): string { return path; }
export function setup(): void {}
export function list(): never { throw new Error("og.ui: <List> is not an engine primitive; use <For>"); }
