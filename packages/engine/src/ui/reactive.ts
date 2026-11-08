import type { UiCollection, UiCollectionDesc, UiReadable, UiScope, UiSource, UiStart, UiStores, UiStoreValues, UiWritable } from "@origingame/engine";
import type { Result } from "../runtime.ts";
import { decodeUiPatch, encodeUiPatch } from "./reactive-wire.ts";

type Transport = (bytes: Uint8Array) => Uint8Array;
type Kind = "writable" | "readable" | "derived" | "collection";
type Callback = { source: UiSource; run: () => unknown };
type Registration = Map<Scope, { stores: Set<string>; mounts: Set<string> }>;

function isSource(value: unknown): value is UiSource {
  return typeof value === "object" && value !== null && "scope" in value && typeof value.scope === "string"
    && "id" in value && typeof value.id === "string";
}

function reply(value: unknown): Result<unknown> {
  if (typeof value === "object" && value !== null && "ok" in value) {
    if (value.ok === true && "value" in value) return { ok: true, value: value.value };
    if (value.ok === false && "reason" in value && typeof value.reason === "string") return { ok: false, reason: value.reason };
  }
  return { ok: false, reason: "invalid UI patch reply" };
}

function freezeValue<T>(value: T): T {
  if (typeof value === "object" && value !== null) {
    for (const item of Object.values(value)) freezeValue(item);
    Object.freeze(value);
  }
  return value;
}

/** Rust owns values and graph edges; this registry owns executable callbacks. */
export class UiReactor {
  readonly scopes = new Map<string, Scope>();
  readonly handles = new WeakMap<object, Store>();
  private readonly callbacks = new Map<string, Map<string, Callback>>();
  private readonly transport: Transport;
  private draining = false;
  private evaluating = 0;
  private subscription = 0;
  private registration: Registration | undefined;
  private managed: Registration = new Map();
  private registrationErrors: string[] | undefined;
  private readonly subscriptions = new Map<Store, Set<() => void>>();

  constructor(transport: Transport) { this.transport = transport; }

  restoreMounts(): Result<void> {
    const source = { scope: "restoration", id: "mounts" };
    const result = this.operation(source, { op: "mountInputs" });
    if (!result.ok) return result;
    if (!Array.isArray(result.value)) return this.error(source, "invalid mounted inputs reply");
    const restored = new Map<Scope, Map<string, Set<Store>>>();
    const entries: unknown[] = result.value;
    for (const entry of entries) {
      if (!entry || typeof entry !== "object" || !("mount" in entry) || !isSource(entry.mount)
        || !("inputs" in entry) || !Array.isArray(entry.inputs) || !entry.inputs.every(isSource)) {
        return this.error(source, "invalid mounted inputs reply");
      }
      const scope = this.scopes.get(entry.mount.scope);
      if (!scope) continue;
      const inputs = new Set<Store>();
      for (const source of entry.inputs) {
        const store = this.scopes.get(source.scope)?.stores.get(source.id);
        if (store) inputs.add(store);
      }
      let mounts = restored.get(scope);
      if (!mounts) restored.set(scope, mounts = new Map());
      mounts.set(entry.mount.id, inputs);
    }
    // Acquire before releasing so a source transferred between mounts stays live.
    for (const [scope, mounts] of restored) {
      for (const [id, inputs] of mounts) {
        for (const store of inputs) if (!scope.mounts.get(id)?.has(store)) store.retain();
      }
    }
    for (const scope of this.scopes.values()) {
      const mounts = restored.get(scope);
      for (const [id, inputs] of scope.mounts) {
        for (const store of inputs) if (!mounts?.get(id)?.has(store)) store.release();
      }
      scope.mounts.clear();
      for (const [id, inputs] of mounts ?? []) scope.mounts.set(id, inputs);
    }
    this.flush();
    return { ok: true, value: undefined };
  }

  registerPresentation(run: () => void): Result<void> {
    if (this.registration) return { ok: false, reason: "UI registration is already running" };
    const current: Registration = new Map();
    this.registration = current;
    this.registrationErrors = [];
    try { run(); }
    catch (error) { this.registrationErrors.push(String(error)); }
    this.registration = undefined;
    if (this.registrationErrors.length === 0) {
      for (const [scope, previous] of this.managed) {
        const next = current.get(scope);
        for (const id of previous.mounts) {
          if (!next?.mounts.has(id)) scope.unmount(id);
        }
        for (const name of previous.stores) {
          const store = scope.stores.get(name);
          if (store && !next?.stores.has(name)) {
            for (const stop of [...this.subscriptions.get(store) ?? []]) stop();
            store.stop();
            this.operation(store.source, { op: "removeSource", source: store.source });
            this.callbacks.get(scope.id)?.delete(name);
            scope.stores.delete(name);
          }
        }
      }
      this.managed = current;
      this.operation({ scope: "registration", id: "complete" }, {
        op: "registered",
        sources: [...current].flatMap(([scope, ids]) => [...ids.stores].map(id => ({ scope: scope.id, id }))),
        mounts: [...current].flatMap(([scope, ids]) => [...ids.mounts].map(id => ({ scope: scope.id, id }))),
      });
    }
    const errors = this.registrationErrors;
    this.registrationErrors = undefined;
    this.flush();
    return errors.length ? { ok: false, reason: errors.join("\n") } : { ok: true, value: undefined };
  }

  note(scope: Scope, id: string, kind: "stores" | "mounts" = "stores"): boolean {
    if (!this.registration) return true;
    let seen = this.registration.get(scope);
    if (!seen) this.registration.set(scope, seen = { stores: new Set(), mounts: new Set() });
    if (seen[kind].has(id)) {
      this.error({ scope: scope.id, id }, "duplicate ID within GameDesc.ui registration");
      return false;
    }
    seen[kind].add(id);
    return true;
  }

  scope(id: string): UiScope {
    let scope = this.scopes.get(id);
    if (!scope) this.scopes.set(id, scope = new Scope(this, id));
    return scope;
  }

  send(command: object): Result<unknown> {
    const encoded = encodeUiPatch([command]);
    if (!encoded.ok) {
      this.send({ op: "fail", reason: encoded.reason });
      return encoded;
    }
    try {
      const decoded = decodeUiPatch(this.transport(encoded.value));
      if (!decoded.ok) return decoded;
      return reply(Array.isArray(decoded.value) ? decoded.value[0] : decoded.value);
    } catch (error) { return { ok: false, reason: `UI host operation failed: ${String(error)}` }; }
  }

  error<T>(source: UiSource, reason: string): Result<T> {
    const message = `og.ui scope '${source.scope}' store '${source.id}': ${reason}`;
    this.scopes.get(source.scope)?.errors.push(message);
    this.registrationErrors?.push(message);
    this.send({ op: "fail", reason: message });
    return { ok: false, reason: message };
  }

  operation(source: UiSource, command: object): Result<unknown> {
    const result = this.send(command);
    return result.ok ? result : this.error(source, result.reason);
  }

  write(source: UiSource, command: object): Result<void> {
    const result = this.operation(source, command);
    if (!result.ok) return result;
    this.flush();
    return { ok: true, value: undefined };
  }

  register(source: UiSource, run: () => unknown): void {
    let scope = this.callbacks.get(source.scope);
    if (!scope) this.callbacks.set(source.scope, scope = new Map());
    scope.set(source.id, { source, run });
  }

  evaluate(source: UiSource): void {
    const begin = this.operation(source, { op: "begin", source });
    if (!begin.ok) return;
    const callback = this.callbacks.get(source.scope)?.get(source.id);
    let value: unknown = null;
    let error: string | null = null;
    this.evaluating++;
    try {
      if (!callback) error = "store callback has not re-registered";
      else {
        value = callback.run();
        const encoded = encodeUiPatch(value);
        if (!encoded.ok) { error = encoded.reason; value = null; }
      }
    } catch (caught) { error = String(caught); value = null; }
    finally { this.evaluating--; }
    this.operation(source, { op: "finish", source, value, error });
  }

  flush(): void {
    if (this.draining) return;
    this.draining = true;
    try {
      for (;;) {
        const next = this.send({ op: "next" });
        if (!next.ok || next.value === null || !isSource(next.value)) break;
        this.evaluate(next.value);
      }
    } finally { this.draining = false; }
  }

  read(source: UiSource, fallback: unknown): unknown {
    for (;;) {
      const result = this.operation(source, { op: "read", source });
      if (!result.ok) return fallback;
      const answer = result.value;
      if (typeof answer !== "object" || answer === null) {
        this.error(source, "invalid store reply");
        return fallback;
      }
      if ("evaluate" in answer && isSource(answer.evaluate)) {
        this.evaluate(answer.evaluate);
        continue;
      }
      if ("value" in answer) return freezeValue(answer.value);
      this.error(source, "store reply has neither value nor computation");
      return fallback;
    }
  }

  subscribe(store: Store, run: (value: unknown) => void, invalidate: () => void): () => void {
    store.retain();
    const source = { scope: store.source.scope, id: `$subscription:${++this.subscription}` };
    let initial = true;
    const registered = this.operation(source, { op: "effect", source });
    if (!registered.ok) { store.release(); return () => {}; }
    this.register(source, () => {
      const value = store.read();
      if (!initial) invalidate();
      initial = false;
      run(value);
      return null;
    });
    this.evaluate(source);
    this.flush();
    let subscribed = true;
    const unsubscribe = () => {
      if (!subscribed) return;
      const result = this.operation(source, { op: "forget", source });
      if (!result.ok) return;
      subscribed = false;
      this.subscriptions.get(store)?.delete(unsubscribe);
      this.callbacks.get(source.scope)?.delete(source.id);
      store.release();
    };
    let subscriptions = this.subscriptions.get(store);
    if (!subscriptions) this.subscriptions.set(store, subscriptions = new Set());
    subscriptions.add(unsubscribe);
    return unsubscribe;
  }

  readable<T>(store: Store): UiReadable<T> {
    const handle: UiReadable<T> = {
      source: store.source,
      // T is the declaration's type. The core stores only binary-validated values
      // written through that declaration; returned objects are immutable copies.
      subscribe: (run, invalidate = () => {}) => this.subscribe(store, value => run(value as T), invalidate),
    };
    this.handles.set(handle, store);
    return handle;
  }

  get<T>(store: UiReadable<T>): T {
    const own = this.handles.get(store);
    if (own && this.evaluating) return own.read() as T;
    // A svelte/store subscription synchronously supplies its initial value.
    let value!: T;
    const unsubscribe = store.subscribe(next => { value = next; });
    unsubscribe();
    return value;
  }

  dispose(scope: Scope): Result<void> {
    const source = { scope: scope.id, id: "dispose" };
    const idle = this.operation(source, { op: "checkIdle", source });
    if (!idle.ok) return idle;
    for (const id of [...scope.mounts.keys()]) {
      const result = scope.unmount(id);
      if (!result.ok) return result;
    }
    for (const store of scope.stores.values()) {
      for (const stop of [...this.subscriptions.get(store) ?? []]) stop();
      this.subscriptions.delete(store);
      store.stop();
    }
    const result = this.operation(source, { op: "dispose", scope: scope.id });
    if (!result.ok) return result;
    this.callbacks.delete(scope.id);
    this.scopes.delete(scope.id);
    this.managed.delete(scope);
    this.flush();
    return { ok: true, value: undefined };
  }
}

class Store {
  readonly source: UiSource;
  readonly kind: Kind;
  readonly reactor: UiReactor;
  readonly initial: unknown;
  start: () => void | (() => void) = () => {};
  private stopCallback: (() => void) | undefined;
  private references = 0;

  constructor(reactor: UiReactor, source: UiSource, kind: Kind, initial: unknown) {
    this.reactor = reactor; this.source = source; this.kind = kind; this.initial = initial;
  }
  read(): unknown { return this.reactor.read(this.source, this.initial); }
  set(value: unknown): Result<void> {
    return this.reactor.write(this.source, { op: "set", source: this.source, value, force: typeof value === "object" && value !== null });
  }
  retain(): void {
    if (this.references++ === 0) {
      try { this.stopCallback = this.start() ?? undefined; }
      catch (error) { this.reactor.error(this.source, String(error)); }
    }
  }
  release(): void { if (--this.references === 0) this.stop(); }
  stop(): void {
    const stop = this.stopCallback;
    this.stopCallback = undefined;
    try { stop?.(); } catch (error) { this.reactor.error(this.source, String(error)); }
  }
  restart(): void {
    if (this.references > 0) {
      this.stop();
      try { this.stopCallback = this.start() ?? undefined; }
      catch (error) { this.reactor.error(this.source, String(error)); }
    }
  }
}

class Scope implements UiScope {
  readonly errors: string[] = [];
  readonly stores = new Map<string, Store>();
  readonly mounts = new Map<string, Set<Store>>();
  private readonly reactor: UiReactor;
  readonly id: string;
  constructor(reactor: UiReactor, id: string) { this.reactor = reactor; this.id = id; }
  private source(id: string): UiSource { return Object.freeze({ scope: this.id, id }); }

  private declare(id: string, kind: Kind, initial: unknown, command: object): Store {
    const source = this.source(id);
    const existing = this.stores.get(id);
    if (!this.reactor.note(this, id)) return existing ?? new Store(this.reactor, source, kind, initial);
    if (existing && existing.kind !== kind) {
      this.reactor.error(source, "store was already declared with a different kind");
      return existing;
    }
    if (id.startsWith("$")) this.reactor.error(source, "store IDs beginning with $ are reserved");
    else this.reactor.operation(source, command);
    const store = existing ?? new Store(this.reactor, source, kind, initial);
    this.stores.set(id, store);
    return store;
  }

  writable<T>(id: string, initial: T): UiWritable<T> {
    const source = this.source(id);
    const store = this.declare(id, "writable", initial, { op: "signal", source, initial });
    const handle: UiWritable<T> = {
      ...this.reactor.readable<T>(store),
      set: value => store.set(value),
      update: updater => {
        try { return store.set(updater(store.read() as T)); }
        catch (error) { return this.reactor.error(source, String(error)); }
      },
    };
    this.reactor.handles.set(handle, store);
    return handle;
  }

  readable<T>(id: string, initial: T, start: UiStart<T> = () => {}): UiReadable<T> {
    const source = this.source(id);
    const store = this.declare(id, "readable", initial, { op: "signal", source, initial, readable: true });
    store.start = () => start(value => { store.set(value); }, update => { store.set(update(store.read() as T)); });
    store.restart();
    return this.reactor.readable<T>(store);
  }

  derived<S extends UiStores, T>(id: string, stores: S, compute: (values: UiStoreValues<S>) => T, initial?: T): UiReadable<T> {
    const source = this.source(id);
    const inputs: readonly UiReadable<unknown>[] = "subscribe" in stores ? [stores] : stores;
    const owned = inputs.map(input => this.reactor.handles.get(input));
    const declaration = { op: "memo", source, inputs: inputs.map(input => input.source), ...(initial === undefined ? {} : { initial }) };
    const store = this.declare(id, "derived", initial, { ...declaration, inactive: true });
    this.reactor.register(source, () => {
      const values = owned.map(input => input?.read());
      return compute(("subscribe" in stores ? values[0] : values) as UiStoreValues<S>);
    });
    store.start = () => {
      if (owned.some(input => !input)) {
        this.reactor.error(source, "derived inputs must be named stores from this world");
        return;
      }
      for (const input of owned) input?.retain();
      this.reactor.operation(source, declaration);
      this.reactor.flush();
      return () => {
        this.reactor.operation(source, { op: "suspend", source });
        for (const input of owned) input?.release();
      };
    };
    store.restart();
    return this.reactor.readable<T>(store);
  }

  get<T>(store: UiReadable<T>): T { return this.reactor.get(store); }

  collection<T>(id: string, items: readonly T[], desc?: UiCollectionDesc): UiCollection<T> {
    const source = this.source(id);
    const store = this.declare(id, "collection", items, { op: "collection", source, items, key: desc?.key ?? "id" });
    const reactor = this.reactor;
    const handle: UiCollection<T> = {
      ...reactor.readable<readonly T[]>(store),
      keys: () => {
        const result = reactor.operation(source, { op: "keys", source });
        return result.ok && Array.isArray(result.value) && result.value.every(k => typeof k === "string") ? Object.freeze(result.value) : [];
      },
      row: key => {
        const result = reactor.operation(source, { op: "row", source, key });
        return result.ok ? { ok: true, value: freezeValue(result.value as T) } : result;
      },
      insert: (row, before) => reactor.write(source, { op: "insert", source, row, before: before ?? null }),
      remove: key => reactor.write(source, { op: "remove", source, key }),
      move: (key, before) => reactor.write(source, { op: "move", source, key, before: before ?? null }),
      patch: (key, fields) => reactor.write(source, { op: "patch", source, key, fields }),
    };
    reactor.handles.set(handle, store);
    return handle;
  }

  mount(id: string, document: string, inputs: Readonly<Record<string, UiReadable<unknown>>>): Result<void> {
    const mount = this.source(id);
    if (!this.reactor.note(this, id, "mounts")) return { ok: false, reason: "duplicate mount ID" };
    const idle = this.reactor.operation(mount, { op: "checkIdle", source: mount });
    if (!idle.ok) return idle;
    const sources: Record<string, UiSource> = {};
    const next = new Set<Store>();
    for (const [name, handle] of Object.entries(inputs)) {
      const store = this.reactor.handles.get(handle);
      if (!store || this.reactor.scopes.get(store.source.scope)?.stores.get(store.source.id) !== store) {
        return this.reactor.error(mount, "mount inputs must be live named stores from this world");
      }
      Object.defineProperty(sources, name, { value: store.source, enumerable: true });
      next.add(store);
    }
    const previous = this.mounts.get(id) ?? new Set<Store>();
    for (const store of next) if (!previous.has(store)) store.retain();
    this.reactor.flush();
    const result = this.reactor.operation(mount, { op: "mount", mount, document, inputs: sources });
    if (!result.ok) {
      for (const store of next) if (!previous.has(store)) store.release();
      return result;
    }
    this.mounts.set(id, next);
    for (const store of previous) if (!next.has(store)) store.release();
    return { ok: true, value: undefined };
  }

  unmount(id: string): Result<void> {
    const mount = this.source(id);
    const result = this.reactor.operation(mount, { op: "unmount", mount });
    if (!result.ok) return result;
    const previous = this.mounts.get(id);
    this.mounts.delete(id);
    for (const store of previous ?? []) store.release();
    this.reactor.flush();
    return { ok: true, value: undefined };
  }

  takeErrors(): readonly string[] { return this.errors.splice(0); }
  dispose(): Result<void> { return this.reactor.dispose(this); }
}
