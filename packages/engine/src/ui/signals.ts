// Solid 1.9's scheduling as NSH's reactive graph ports it (third_party/nsh-reactive,
// Apache-2.0), in TypeScript for the game's own JS engine.
// The UI's reactive graph, Solid 1.9's model (crates/og-ui-next/CONVENTIONS.md
// §Components): signals, memos, render effects and effects, owners that dispose
// what was made under them, batches, untracked reads. A component's body runs
// once; what changes is a function the renderer follows. Nothing here reads the
// simulation: a store reaches it through `useStore`.

/** Reads a value and, inside a computation, subscribes it. */
export type Accessor<T> = () => T;
/** A written value: read it by calling it, write it with `set`. */
export interface Signal<T> extends Accessor<T> {
  set(value: T): void;
  update(fn: (value: T) => T): void;
}

interface Owner {
  owner: Owner | null;
  owned: Computation[] | null;
  cleanups: (() => void)[] | null;
  context: Map<symbol, unknown> | null;
}

interface Source {
  observers: Set<Computation>;
}

interface Computation extends Owner, Source {
  fn: (prev: unknown) => unknown;
  value: unknown;
  sources: Set<Source>;
  state: number;
  /** A memo has observers to tell; an effect runs after the render effects. */
  kind: "memo" | "render" | "effect";
  equals: ((a: unknown, b: unknown) => boolean) | false;
}

const CLEAN: number = 0, STALE: number = 1, PENDING: number = 2, DISPOSED: number = 3;

let owner: Owner | null = null;
let listener: Computation | null = null;
let batching = 0;
const queue = new Set<Computation>();
const effects: Computation[] = [];
let runs = 0;

function track(source: Source): void {
  if (!listener) return;
  source.observers.add(listener);
  listener.sources.add(source);
}

// A changed source makes its readers stale; a memo among them makes its own
// readers pending: they run only if the memo's value turns out changed.
function mark(source: Source): void {
  for (const c of source.observers) {
    if (c.state !== CLEAN && c.state !== PENDING) continue;
    c.state = STALE;
    queue.add(c);
    if (c.kind === "memo") pend(c);
  }
}

function pend(memo: Computation): void {
  for (const c of memo.observers) {
    if (c.state !== CLEAN) continue;
    c.state = PENDING;
    queue.add(c);
    if (c.kind === "memo") pend(c);
  }
}

function clean(c: Owner): void {
  if (c.owned) {
    for (const child of c.owned) dispose(child);
    c.owned = null;
  }
  if (c.cleanups) {
    const cleanups = c.cleanups;
    c.cleanups = null;
    for (const fn of cleanups) fn();
  }
  if ("sources" in c) {
    const comp = c as Computation;
    for (const s of comp.sources) s.observers.delete(comp);
    comp.sources.clear();
  }
}

function dispose(c: Computation): void {
  clean(c);
  c.state = DISPOSED;
  queue.delete(c);
}

function run(c: Computation): void {
  if (c.state === DISPOSED || c.state === CLEAN) return;
  if (c.state === PENDING) {
    // Only memos it reads may have changed: bring them up to date, and run
    // only if one did (it then made this one stale).
    for (const s of [...c.sources]) {
      if ("fn" in s) run(s as Computation);
      if ((c.state as number) === STALE) break;
    }
    if ((c.state as number) === PENDING) { c.state = CLEAN; queue.delete(c); return; }
  }
  queue.delete(c);
  clean(c);
  const [prevOwner, prevListener] = [owner, listener];
  owner = c;
  listener = c;
  let value: unknown;
  try {
    value = c.fn(c.value);
  } finally {
    owner = prevOwner;
    listener = prevListener;
  }
  if (c.state === DISPOSED) return;
  c.state = CLEAN;
  const same = c.kind === "memo" && c.equals !== false && c.equals(c.value, value);
  c.value = value;
  if (c.kind === "memo" && !same) {
    for (const o of c.observers) {
      if (o.state === CLEAN || o.state === PENDING) { o.state = STALE; queue.add(o); }
    }
  }
}

function flush(): void {
  if (batching > 0) return;
  batching++;
  try {
    while (queue.size > 0 || effects.length > 0) {
      while (queue.size > 0) {
        if (++runs > 1_000_000) throw new Error("og.ui: a computation keeps writing what it reads");
        const [c] = queue;
        queue.delete(c!);
        if (c!.kind === "effect") effects.push(c!);
        else run(c!);
      }
      const due = effects.splice(0);
      for (const c of due) run(c);
    }
  } finally {
    batching--;
    runs = 0;
  }
}

function computation(fn: (prev: unknown) => unknown, value: unknown, kind: Computation["kind"], equals: Computation["equals"] = Object.is): Computation {
  const c: Computation = { fn, value, kind, equals, sources: new Set(), observers: new Set(), state: STALE, owner, owned: null, cleanups: null, context: null };
  if (owner) (owner.owned ??= []).push(c);
  return c;
}

/** A value that is written: `count()` reads it, `count.set(n)` writes it. */
export function createSignal<T>(value: T, options?: { equals?: false | ((a: T, b: T) => boolean) }): Signal<T> {
  const source: Source = { observers: new Set() };
  const equals = options?.equals === undefined ? Object.is : options.equals;
  const read = (() => { track(source); return value; }) as Signal<T>;
  read.set = (next: T) => {
    if (equals !== false && equals(value, next)) return;
    value = next;
    mark(source);
    flush();
  };
  read.update = (fn) => read.set(fn(untrack(() => value)));
  return read;
}

/** A value computed from others, cached until one of them changes. */
export function createMemo<T>(fn: (prev: T | undefined) => T, initial?: T, options?: { equals?: false | ((a: T, b: T) => boolean) }): Accessor<T> {
  const c = computation(fn as (p: unknown) => unknown, initial, "memo", (options?.equals ?? Object.is) as Computation["equals"]);
  run(c);
  return () => {
    if (c.state === STALE || c.state === PENDING) run(c);
    track(c);
    return c.value as T;
  };
}

/** Runs `fn` now and again whenever what it read changes, before effects. */
export function createRenderEffect<T>(fn: (prev: T | undefined) => T, initial?: T): void {
  run(computation(fn as (p: unknown) => unknown, initial, "render"));
}

/** Runs `fn` after the render effects, and again whenever what it read changes. */
export function createEffect<T>(fn: (prev: T | undefined) => T, initial?: T): void {
  const c = computation(fn as (p: unknown) => unknown, initial, "effect");
  if (batching > 0) effects.push(c);
  else run(c);
}

/** Reads without subscribing. */
export function untrack<T>(fn: () => T): T {
  const prev = listener;
  listener = null;
  try { return fn(); } finally { listener = prev; }
}

/** Writes inside `fn` notify once, when it returns. */
export function batch<T>(fn: () => T): T {
  batching++;
  try { return fn(); } finally { batching--; flush(); }
}

/** An owner nothing disposes but the `dispose` it is handed. */
export function createRoot<T>(fn: (dispose: () => void) => T): T {
  const root: Owner = { owner, owned: null, cleanups: null, context: null };
  const [prevOwner, prevListener] = [owner, listener];
  owner = root;
  listener = null;
  try { return fn(() => clean(root)); } finally { owner = prevOwner; listener = prevListener; }
}

/** Runs `fn` when the current owner is disposed or runs again. */
export function onCleanup(fn: () => void): void {
  if (owner) (owner.cleanups ??= []).push(fn);
}

export function getOwner(): unknown { return owner; }

/** Runs `fn` under `o` (from `getOwner`), so what it makes belongs there. */
export function runWithOwner<T>(o: unknown, fn: () => T): T {
  const [prevOwner, prevListener] = [owner, listener];
  owner = o as Owner | null;
  listener = null;
  try { return fn(); } finally { owner = prevOwner; listener = prevListener; }
}

/** A value components below a provider read with `useContext`. */
export interface Context<T> { readonly id: symbol; readonly defaultValue: T }

export function createContext<T>(defaultValue: T): Context<T> {
  return { id: Symbol("context"), defaultValue };
}

export function useContext<T>(context: Context<T>): T {
  for (let o = owner; o; o = o.owner) {
    if (o.context?.has(context.id)) return o.context.get(context.id) as T;
  }
  return context.defaultValue;
}

/** Runs `fn` with `context` set to `value` for everything made under it. */
export function provide<T, R>(context: Context<T>, value: T, fn: () => R): R {
  const o: Owner = { owner, owned: null, cleanups: null, context: new Map([[context.id, value]]) };
  if (owner) (owner.cleanups ??= []).push(() => clean(o));
  const prev = owner;
  owner = o;
  try { return fn(); } finally { owner = prev; }
}
