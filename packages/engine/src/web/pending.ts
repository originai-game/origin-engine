// The requests one side has sent and not yet heard an answer to, by id: the
// shell's calls and its probe requests (shell.ts) each keep one.

/** Requests waiting for their answer. */
export class Pending {
  #next = 1;
  readonly #waiting = new Map<number, { resolve(value: unknown): void; reject(error: Error): void }>();

  /** How many wait. */
  get size(): number {
    return this.#waiting.size;
  }

  /** A new request: `send` is handed its id, and the promise settles with its answer. */
  ask(send: (id: number) => void): Promise<unknown> {
    const id = this.#next++;
    return new Promise((resolve, reject) => {
      this.#waiting.set(id, { resolve, reject });
      send(id);
    });
  }

  /** Request `id`'s answer: `error` rejects it, else `value` resolves it. An id nothing waits for is ignored. */
  settle(id: unknown, error: Error | undefined, value: unknown): void {
    const p = typeof id === "number" ? this.#waiting.get(id) : undefined;
    if (!p) return;
    this.#waiting.delete(id as number);
    if (error) p.reject(error);
    else p.resolve(value);
  }

  /** No answer will come: every waiting request fails with `error`. */
  fail(error: Error): void {
    for (const p of this.#waiting.values()) p.reject(error);
    this.#waiting.clear();
  }
}
