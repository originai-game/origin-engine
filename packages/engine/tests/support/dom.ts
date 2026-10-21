// A stand-in document for the shell's DOM work in bun (no browser): the
// elements, attributes, tree and focus that the text entry
// (src/web/text-input.ts) and the accessibility mirror (src/web/a11y.ts)
// touch, and what a screen reader would read of them. Not a DOM: enough of
// one that those two run as written. `install()` puts its Node and
// HTMLInputElement on globalThis (the mirror's instanceof checks) and gives
// them back.

export class StandInNode extends EventTarget {
  nodeType = 1;
  parentElement: StandInElement | null = null;
  readonly ownerDocument: StandInDocument;
  #text = "";
  constructor(doc: StandInDocument) {
    super();
    this.ownerDocument = doc;
  }
  get parentNode(): StandInElement | null {
    return this.parentElement;
  }
  get textContent(): string {
    return this.#text;
  }
  set textContent(v: string) {
    this.#text = v;
  }
  remove(): void {
    const p = this.parentElement;
    if (!p) return;
    p.childNodes.splice(p.childNodes.indexOf(this), 1);
    this.parentElement = null;
  }
}

export class StandInText extends StandInNode {
  override nodeType = 3;
}

export class StandInElement extends StandInNode {
  readonly tagName: string;
  readonly childNodes: StandInNode[] = [];
  readonly style: Record<string, string> = {};
  readonly dataset: Record<string, string> = {};
  readonly #attributes = new Map<string, string>();
  tabIndex = -1;
  id = "";
  constructor(doc: StandInDocument, tag: string) {
    super(doc);
    this.tagName = tag.toUpperCase();
  }
  setAttribute(name: string, value: string): void {
    this.#attributes.set(name, String(value));
  }
  getAttribute(name: string): string | null {
    if (name === "id") return this.id || null;
    return this.#attributes.get(name) ?? null;
  }
  hasAttribute(name: string): boolean {
    return this.getAttribute(name) !== null;
  }
  removeAttribute(name: string): void {
    this.#attributes.delete(name);
  }
  get children(): StandInElement[] {
    return this.childNodes.filter((c): c is StandInElement => c instanceof StandInElement);
  }
  get firstChild(): StandInNode | null {
    return this.childNodes[0] ?? null;
  }
  get firstElementChild(): StandInElement | null {
    return this.children[0] ?? null;
  }
  get lastElementChild(): StandInElement | null {
    return this.children.at(-1) ?? null;
  }
  get childElementCount(): number {
    return this.children.length;
  }
  override get textContent(): string {
    return this.childNodes.map((c) => c.textContent).join("");
  }
  override set textContent(v: string) {
    for (const c of [...this.childNodes]) c.remove();
    if (v) this.appendChild(this.ownerDocument.createTextNode(v));
  }
  appendChild<T extends StandInNode>(child: T): T {
    child.remove();
    this.childNodes.push(child);
    child.parentElement = this;
    return child;
  }
  append(...nodes: StandInNode[]): void {
    for (const n of nodes) this.appendChild(n);
  }
  insertBefore<T extends StandInNode>(child: T, before: StandInNode | null): T {
    child.remove();
    const at = before ? this.childNodes.indexOf(before) : -1;
    if (at < 0) this.childNodes.push(child);
    else this.childNodes.splice(at, 0, child);
    child.parentElement = this;
    return child;
  }
  contains(node: unknown): boolean {
    for (let n = node as StandInNode | null; n; n = n.parentElement) if (n === this) return true;
    return false;
  }
  focus(): void {
    const doc = this.ownerDocument;
    if (doc.activeElement === this) return;
    // No focusin: the mirror's own moves are not a reader's (a11y.ts), and nothing here is a reader's.
    doc.activeElement = this;
  }
  blur(): void {
    const doc = this.ownerDocument;
    if (doc.activeElement === this) doc.activeElement = doc.body;
  }
  getBoundingClientRect(): { left: number; top: number; width: number; height: number } {
    return { left: 0, top: 0, width: Number.parseFloat(this.style["width"] ?? "0") || 0, height: Number.parseFloat(this.style["height"] ?? "0") || 0 };
  }
}

export class StandInInput extends StandInElement {
  type = "text";
  value = "";
  min = "";
  max = "";
  step = "";
}

export class StandInDocument {
  readonly body: StandInElement;
  activeElement: StandInElement | null;
  constructor() {
    this.body = new StandInElement(this, "body");
    this.activeElement = this.body;
  }
  createElement(tag: string): StandInElement {
    return tag === "input" ? new StandInInput(this, tag) : new StandInElement(this, tag);
  }
  createTextNode(text: string): StandInText {
    const t = new StandInText(this);
    t.textContent = text;
    return t;
  }
  /** Every element under the body, depth first. */
  all(): StandInElement[] {
    const out: StandInElement[] = [];
    const walk = (e: StandInElement) => {
      for (const c of e.children) {
        out.push(c);
        walk(c);
      }
    };
    walk(this.body);
    return out;
  }
  byId(id: string): StandInElement | undefined {
    return this.all().find((e) => e.id === id);
  }
}

/** A canvas of `width`×`height` CSS pixels in `doc`, with an OffscreenCanvas to hand over. */
export function standInCanvas(doc: StandInDocument, width = 1280, height = 720): HTMLCanvasElement {
  const canvas = doc.createElement("canvas");
  Object.assign(canvas, {
    clientWidth: width,
    clientHeight: height,
    width: 0,
    height: 0,
    transferControlToOffscreen: () => ({ offscreen: true }),
    getBoundingClientRect: () => ({ left: 0, top: 0, width, height }),
  });
  doc.body.appendChild(canvas);
  return canvas as unknown as HTMLCanvasElement;
}

/** The globals the mirror's instanceof checks read; returns what puts them back. */
export function install(): () => void {
  const g = globalThis as Record<string, unknown>;
  const saved = { Node: g["Node"], HTMLInputElement: g["HTMLInputElement"] };
  g["Node"] = StandInNode;
  g["HTMLInputElement"] = StandInInput;
  return () => {
    g["Node"] = saved.Node;
    g["HTMLInputElement"] = saved.HTMLInputElement;
  };
}

/** Whether a reader can reach `el`: neither it nor an ancestor is aria-hidden. */
export function exposed(el: StandInElement): boolean {
  for (let e: StandInElement | null = el; e; e = e.parentElement) if (e.getAttribute("aria-hidden") === "true") return false;
  return true;
}

/** The role a reader is told: the explicit one, else a textarea's or an input's own. */
export function roleOf(el: StandInElement): string | null {
  return el.getAttribute("role") ?? (el.tagName === "TEXTAREA" ? "textbox" : el.tagName === "INPUT" ? "textbox" : null);
}
