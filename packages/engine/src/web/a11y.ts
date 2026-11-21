// The accessibility mirror: the UI is drawn into a canvas, which a screen
// reader cannot read, so the shell keeps a DOM copy of the new core's
// accessibility tree (crates/og-ui-next/src/a11y.rs) over it. The Worker
// sends what changed with a frame's answer (`A11yUpdate`: the nodes whose
// fingerprint moved, og-ui-next `a11y::web`), and nothing when nothing did;
// the mirror keeps every node it was sent, and a node no node's `children`
// reach any more is gone. Each node becomes an invisible element at its
// bounds, nested as the tree nests it, with its ARIA role, name, value and
// states.
//
// - Focus follows the UI's: the element of the focused node takes DOM focus
//   (unless the page's focus is in a field of its own, or the UI's text
//   control has the hidden textarea, text-input.ts). While that textarea
//   has the focus it is the field for assistive technology (named after
//   the field's node): the field's own element is aria-hidden and its
//   parent owns the textarea (`standIn`).
// - Live regions are the page's: a node with a live setting (a status, an
//   alert, `live`) carries aria-live and aria-atomic, and its text is its
//   content, so the browser announces what changes in it.
// - The DOM reaches the UI as input: a screen reader's click on an element
//   is `{ kind: "a11y", action: "activate", path }` (the path is the node's
//   id) and its moving focus onto one is `focus`; `adjust` sends
//   `increment`/`decrement` for an adjustable one. A slider and a
//   spinbutton are mirrored as an `<input type="range">` (with its role)
//   held at the UI's value, one step from either end: a screen reader's own
//   adjust gesture moves the native value and fires `input`, which becomes
//   `increment` or `decrement` and puts the value back (the UI owns the
//   value and its step; `aria-valuenow`, `aria-valuemin`, `aria-valuemax`
//   and `aria-valuetext` say it). A range's children follow it as siblings
//   (an input holds no children). Keys pressed while a mirrored element has
//   the DOM focus still go to the engine as keys.
//
// The elements draw nothing: they are transparent through their background
// and colour (not opacity, so they count as shown), and a screen reader's
// focus ring lands where the control is drawn. Only the controls a player
// operates (`CONTROLS`) take pointer events, so a page's own scan of its
// controls (an autoplay's: role, label, box, an `elementFromPoint` hit at
// the centre) finds them over the canvas; everything else lets the pointer
// through to the canvas (behind an open modal the tree holds only it).
// A pointer or wheel event on a control is dispatched again on the canvas,
// so the engine's hit test, hover, press and drag see it exactly as a
// press on the canvas (the canvas captures the pointer at the press); the
// click that follows a pointer's press is not an activation, only a
// screen reader's click (no pointer, `detail` 0) is.

import type { InjectedInput } from "./input.ts";

/** One node as the Worker sends it (og-ui-next `a11y::web`). */
export interface A11yWireNode extends Omit<A11yNode, "parent"> {
  /** Its children's paths, in order. */
  readonly children: readonly string[];
}

/** What changed in the tree since the last update (a frame answer's `a11y`). */
export interface A11yUpdate {
  /** It holds the whole tree: what the mirror had is dropped. */
  readonly full: boolean;
  /** The focused node's path; empty for none. */
  readonly focus: string;
  /** The nodes that changed, the root (path "0", the canvas) among them when its children did. */
  readonly nodes: readonly A11yWireNode[];
}

/** The tree's root: the canvas itself, not mirrored. */
const ROOT = "0";

/** One node of the mirrored tree, with its parent's path ("" under the root). */
export interface A11yNode {
  /** Its id in the tree, as text. */
  readonly path: string;
  readonly parent: string;
  readonly role: string;
  readonly name: string;
  /** Read after the name: a text field's placeholder; empty for none. */
  readonly description: string;
  readonly value: string;
  readonly states: readonly string[];
  /** A range's smallest value (aria-valuemin); null (or absent) for none. */
  readonly valueMin?: number | null;
  /** A range's largest value (aria-valuemax); null (or absent) for none. */
  readonly valueMax?: number | null;
  /** A range's value as a number (aria-valuenow); null (or absent) for none. */
  readonly valueNow?: number | null;
  /** A live region's setting, polite or assertive; empty (or absent) for none. */
  readonly live?: string;
  /** A live region that announces its whole text. */
  readonly atomic?: boolean;
  /** A heading's level, 1 to 6; null (or absent) for none. */
  readonly level?: number | null;
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

/** The mirrored tree: its nodes in tree order, the focusable ones, the focused one. */
export interface A11yTree {
  readonly nodes: readonly A11yNode[];
  readonly focusOrder: readonly string[];
  readonly focused: string;
}

/** Roles whose `checked` state is reported either way. */
const CHECKABLE = new Set(["checkbox", "radio", "switch"]);
/** Roles whose `selected` state is reported either way. */
const SELECTABLE = new Set(["tab", "option"]);
/** Roles whose `expanded` state is reported either way. */
const EXPANDABLE = new Set(["combobox"]);
/** Roles with a numeric value (aria-valuenow). */
const RANGES = new Set(["slider", "spinbutton", "progressbar", "meter"]);
/** Roles mirrored as a native range: a screen reader adjusts those itself (their children follow the range as siblings). */
const ADJUSTABLE = new Set(["slider", "spinbutton"]);
/** Roles a player operates: mirrored over the canvas, taking the pointer. */
const CONTROLS = new Set(["button", "link", "menuitem", "menuitemcheckbox", "menuitemradio", "tab", "option", "checkbox", "radio", "switch", "slider", "spinbutton", "textbox", "combobox"]);
/** Pointer events a control passes to the canvas, and the wheel. */
const FORWARDED = ["pointerdown", "pointermove", "pointerup", "pointercancel", "wheel"] as const;
/** Roles read as their text content rather than a label. */
const TEXTS = new Set(["text", "heading", "status", "alert", "tooltip"]);

/** The DOM mirror of the UI's accessibility tree, over the canvas. */
export class A11yMirror {
  readonly root: HTMLDivElement;
  readonly #canvas: HTMLCanvasElement;
  readonly #send: (event: InjectedInput) => void;
  readonly #elements = new Map<string, HTMLElement>();
  /** Every node the Worker sent that is still in the tree, by path. */
  readonly #sent = new Map<string, A11yWireNode>();
  #tree: A11yTree = { nodes: [], focusOrder: [], focused: "" };
  /** The path this mirror last moved DOM focus to (its own focus events are not the reader's). */
  #focusing = "";
  /** The text field the text entry's textarea stands in for (text-input.ts), and the textarea. */
  #standIn: { path: string; element: HTMLElement } | null = null;
  /** What standing in changed: the field's element hidden, its parent owning the textarea. */
  #stood: { hidden: HTMLElement; owner: HTMLElement } | null = null;
  #off: Array<() => void> = [];
  /** A pointer pressed on a control is held: a range's own input under it is not a reader's adjust. */
  #held = 0;

  constructor(canvas: HTMLCanvasElement, send: (event: InjectedInput) => void) {
    this.#canvas = canvas;
    this.#send = send;
    const doc = canvas.ownerDocument;
    const root = doc.createElement("div");
    root.dataset.ogA11y = "";
    Object.assign(root.style, { position: "fixed", left: "0px", top: "0px", width: "0px", height: "0px", pointerEvents: "none", overflow: "visible", zIndex: "0" } satisfies Partial<CSSStyleDeclaration>);
    doc.body.appendChild(root);
    this.root = root;
    // A native range draws its thumb whatever its own style says.
    const style = doc.createElement("style");
    style.textContent = THUMB;
    root.appendChild(style);
    const on = <K extends keyof HTMLElementEventMap>(type: K, fn: (e: HTMLElementEventMap[K]) => void) => {
      root.addEventListener(type, fn as EventListener);
      this.#off.push(() => root.removeEventListener(type, fn as EventListener));
    };
    // A screen reader activates an element with a click (its default
    // action, with no pointer: detail 0). A pointer's click on a control
    // follows a press the canvas already had.
    on("click", (e) => {
      const path = pathOf(e.target);
      if (path === null) return;
      e.preventDefault();
      if (e.detail > 0) return;
      this.#send({ kind: "a11y", action: "activate", path });
    });
    // The pointer on a control is the canvas's: the same event, dispatched
    // on it (its press captures the pointer, so the rest of a drag goes
    // there directly).
    for (const type of FORWARDED) {
      on(type, (e: PointerEvent | WheelEvent) => {
        if (pathOf(e.target) === null) return;
        e.preventDefault();
        e.stopPropagation();
        if (e.type === "pointerdown") this.#held++;
        this.#canvas.dispatchEvent(e instanceof WheelEvent ? new WheelEvent(e.type, e) : new PointerEvent(e.type, e));
      });
    }
    // A press on a control does not move the page's focus (the UI's moves it).
    on("mousedown", (e) => e.preventDefault());
    on("contextmenu", (e) => e.preventDefault());
    // The canvas's box moves without the tree changing.
    const view = doc.defaultView;
    if (view && "ResizeObserver" in view) {
      const watch = new view.ResizeObserver(() => this.#place());
      watch.observe(canvas);
      this.#off.push(() => watch.disconnect());
    }
    if (view) {
      // The canvas has the pointer after the press: its release is the page's.
      const released = () => (this.#held = Math.max(0, this.#held - 1));
      for (const type of ["pointerup", "pointercancel"]) {
        view.addEventListener(type, released, true);
        this.#off.push(() => view.removeEventListener(type, released, true));
      }
      const place = () => this.#place();
      view.addEventListener("resize", place);
      view.addEventListener("scroll", place, { passive: true });
      this.#off.push(() => {
        view.removeEventListener("resize", place);
        view.removeEventListener("scroll", place);
      });
    }
    // A screen reader adjusted a mirrored range: one step, and back to the UI's value.
    const adjusted = (e: Event) => {
      const el = e.target as HTMLInputElement | null;
      if (!el || el.tagName !== "INPUT" || el.type !== "range") return;
      const path = pathOf(el);
      const v = Number(el.value);
      const at = Number(el.dataset.ogValue ?? "1");
      el.value = String(at);
      if (this.#held > 0 || path === null || !Number.isFinite(v) || v === at) return;
      this.#send({ kind: "a11y", action: v > at ? "increment" : "decrement", path });
    };
    on("input", adjusted);
    on("change", adjusted);
    // A screen reader moved focus onto an element: the UI's focus follows it.
    on("focusin", (e) => {
      const path = pathOf(e.target);
      if (path === null || path === this.#focusing || path === this.#tree.focused) return;
      this.#send({ kind: "a11y", action: "focus", path });
    });
  }

  /** The tree as last mirrored. */
  get tree(): A11yTree {
    return this.#tree;
  }

  /** Whether an event target is one of the mirror's elements (its keys are the game's, not a field's). */
  owns(target: EventTarget | null): boolean {
    return target instanceof Node && this.root.contains(target);
  }

  /** The element mirroring the node at `path`, if it is in the tree. */
  element(path: string): HTMLElement | undefined {
    return this.#elements.get(path);
  }

  /** The node at `path` in the tree as last mirrored. */
  node(path: string): A11yNode | undefined {
    return this.#tree.nodes.find((n) => n.path === path);
  }

  /**
   * The text entry's textarea (text-input.ts) takes the page's text for the
   * field at `path` and has the focus: the field's element steps aside
   * (aria-hidden) and its parent owns the textarea (aria-owns), so a reader
   * meets one text box where the field is. null: none.
   */
  standIn(path: string | null, element: HTMLElement | null): void {
    this.#standIn = path !== null && element ? { path, element } : null;
    this.#stand();
  }

  #stand(): void {
    const field = this.#standIn ? this.#elements.get(this.#standIn.path) : undefined;
    const owner = field ? ((field.parentElement as HTMLElement | null) ?? this.root) : undefined;
    if (this.#stood && (this.#stood.hidden !== field || this.#stood.owner !== owner)) {
      this.#stood.hidden.removeAttribute("aria-hidden");
      this.#stood.owner.removeAttribute("aria-owns");
      this.#stood = null;
    }
    if (!field || !owner || !this.#standIn) return;
    field.setAttribute("aria-hidden", "true");
    owner.setAttribute("aria-owns", this.#standIn.element.id);
    this.#stood = { hidden: field, owner };
  }

  /** Increment (or decrement) the adjustable control at `path`, as a screen reader's range action does. */
  adjust(path: string, up: boolean): void {
    this.#send({ kind: "a11y", action: up ? "increment" : "decrement", path });
  }

  /** What changed in the UI's tree: merged into what was sent, then mirrored. */
  update(update: A11yUpdate): void {
    if (update.full) this.#sent.clear();
    for (const n of update.nodes) this.#sent.set(n.path, n);
    // Tree order from the root; what no node reaches any more is gone.
    const nodes: A11yNode[] = [];
    const reached = new Set<string>([ROOT]);
    const walk = (path: string, parent: string) => {
      for (const c of this.#sent.get(path)?.children ?? []) {
        const n = this.#sent.get(c);
        if (!n || reached.has(c)) continue;
        reached.add(c);
        const { children: _, ...rest } = n;
        nodes.push({ ...rest, parent });
        walk(c, c);
      }
    };
    walk(ROOT, "");
    for (const path of this.#sent.keys()) if (!reached.has(path)) this.#sent.delete(path);
    const focusOrder = nodes.filter((n) => n.states.includes("focusable")).map((n) => n.path);
    this.#mirror({ nodes, focusOrder, focused: update.focus });
  }

  /** Brings the DOM in line with the whole tree. */
  #mirror(tree: A11yTree): void {
    const doc = this.root.ownerDocument;
    this.#place();
    const byPath = new Map(tree.nodes.map((n) => [n.path, n]));
    // A native range holds no children: a node under one is placed beside it.
    const ranged = (path: string) => ADJUSTABLE.has(byPath.get(path)?.role ?? "");
    const host = (path: string): string => {
      let p = path;
      while (p && ranged(p)) p = byPath.get(p)?.parent ?? "";
      return p;
    };
    // Remove what left the tree.
    for (const [path, el] of this.#elements) {
      if (!byPath.has(path)) {
        el.remove();
        this.#elements.delete(path);
      }
    }
    for (const node of tree.nodes) {
      let el = this.#elements.get(node.path);
      // A slider or a spinbutton is a native range a screen reader adjusts; anything else a div.
      const range = ADJUSTABLE.has(node.role);
      if (el && (el.tagName === "INPUT") !== range) {
        el.remove();
        el = undefined;
      }
      if (!el) {
        el = doc.createElement(range ? "input" : "div");
        if (el instanceof HTMLInputElement) {
          Object.assign(el, { type: "range", min: "0", max: "2", step: "1", value: "1" });
          el.dataset.ogValue = "1";
        }
        el.dataset.ogPath = node.path;
        Object.assign(el.style, NODE);
        this.#elements.set(node.path, el);
      }
      const above = host(node.parent);
      const parent = (above && this.#elements.get(above)) || this.root;
      const px = above ? (byPath.get(above)?.x ?? 0) : 0;
      const py = above ? (byPath.get(above)?.y ?? 0) : 0;
      // Tree order is DOM order: append moves an existing element to the end of its parent.
      if (el.parentElement !== parent || parent.lastElementChild !== el) parent.appendChild(el);
      this.#describe(el, node, tree);
      const control = CONTROLS.has(node.role);
      Object.assign(el.style, {
        pointerEvents: control ? "auto" : "none",
        touchAction: control ? "none" : "",
        left: `${node.x - px}px`,
        top: `${node.y - py}px`,
        width: `${Math.max(1, node.width)}px`,
        height: `${Math.max(1, node.height)}px`,
      });
    }
    this.#tree = tree;
    this.#stand();
    this.#follow(tree);
  }

  #describe(el: HTMLElement, node: A11yNode, tree: A11yTree): void {
    const states = new Set(node.states);
    const set = (name: string, value: string | null) => {
      if (value === null) el.removeAttribute(name);
      else if (el.getAttribute(name) !== value) el.setAttribute(name, value);
    };
    set("role", node.role);
    const text = TEXTS.has(node.role);
    // Text reads as its content; controls are labelled.
    if (text) {
      if (el.firstChild?.nodeType !== 3 || el.textContent !== node.name) {
        for (const c of [...el.childNodes]) if (c.nodeType === 3) c.remove();
        el.insertBefore(el.ownerDocument.createTextNode(node.name), el.firstChild);
      }
      set("aria-label", null);
    } else {
      for (const c of [...el.childNodes]) if (c.nodeType === 3) c.remove();
      set("aria-label", node.name || null);
    }
    // A text box's description slot carries its text (below); its
    // placeholder, which the tree gives as the description, is aria-placeholder.
    const box = node.role === "textbox" || node.role === "combobox";
    set("aria-description", box ? null : node.description || null);
    set("aria-placeholder", node.role === "textbox" ? node.description || null : null);
    set("aria-level", node.role === "heading" ? String(node.level ?? 2) : null);
    // The page announces a live node's changes itself.
    set("aria-live", node.live || null);
    set("aria-atomic", node.live ? String(node.atomic ?? false) : null);
    if (node.role === "dialog") set("aria-modal", "true");
    const focusable = states.has("focusable") || tree.focusOrder.includes(node.path);
    // A native range is in the page's Tab order unless told otherwise; the UI owns focus.
    set("tabindex", focusable || el instanceof HTMLInputElement ? "-1" : null);
    set("aria-disabled", states.has("disabled") ? "true" : null);
    set("aria-checked", CHECKABLE.has(node.role) ? String(states.has("checked")) : null);
    set("aria-selected", SELECTABLE.has(node.role) ? String(states.has("selected")) : null);
    set("aria-expanded", EXPANDABLE.has(node.role) || states.has("collapsed") || states.has("expanded") ? String(states.has("expanded")) : null);
    set("aria-pressed", node.role === "button" && states.has("pressed") ? "true" : null);
    set("aria-busy", states.has("busy") ? "true" : null);
    set("aria-current", states.has("focused") ? "true" : null);
    if (RANGES.has(node.role)) {
      // The tree's numbers (valueNow, valueMin, valueMax); a tree without
      // them reads the value text, a percent taking 0 to 100.
      const n = node.valueNow ?? Number.parseFloat(node.value);
      const percent = node.value.endsWith("%");
      const min = node.valueMin ?? (percent ? 0 : null);
      const max = node.valueMax ?? (percent ? 100 : null);
      set("aria-valuenow", Number.isFinite(n) ? String(n) : null);
      set("aria-valuemin", min !== null && Number.isFinite(min) ? String(min) : null);
      set("aria-valuemax", max !== null && Number.isFinite(max) ? String(max) : null);
      // A native range sits at the UI's value, a step from either end, so
      // a reader's step (from the native value or from aria-valuenow) lands
      // on one side of it.
      if (el instanceof HTMLInputElement) {
        const at = Number.isFinite(n) ? n : 1;
        if (el.dataset.ogValue !== String(at)) {
          Object.assign(el, { min: String(at - 1), max: String(at + 1), step: "1" });
          el.value = String(at);
          el.dataset.ogValue = String(at);
        }
      }
      set("aria-valuetext", node.value || null);
    } else {
      for (const a of ["aria-valuenow", "aria-valuemin", "aria-valuemax"]) set(a, null);
      if (node.role === "textbox" || node.role === "combobox") {
        set("aria-valuetext", null);
        set("aria-description", node.value || null);
      } else set("aria-valuetext", node.value || null);
    }
  }

  /** The DOM focus follows the UI's focus. */
  #follow(tree: A11yTree): void {
    const doc = this.root.ownerDocument;
    const active = doc.activeElement as HTMLElement | null;
    // The page's own fields and the text control's textarea keep their focus.
    const ours = !active || active === doc.body || active === this.#canvas || this.root.contains(active);
    if (!ours) return;
    const el = tree.focused ? this.#elements.get(tree.focused) : undefined;
    if (!el || el === active || el.getAttribute("role") === "textbox") return;
    this.#focusing = tree.focused;
    try {
      el.focus({ preventScroll: true });
    } finally {
      this.#focusing = "";
    }
  }

  /** Over the canvas, in its CSS pixels (the tree's unit). */
  #place(): void {
    const box = this.#canvas.getBoundingClientRect();
    const sx = box.width / (this.#canvas.clientWidth || box.width || 1);
    const sy = box.height / (this.#canvas.clientHeight || box.height || 1);
    Object.assign(this.root.style, {
      left: `${box.left}px`,
      top: `${box.top}px`,
      width: `${box.width}px`,
      height: `${box.height}px`,
      transform: sx === 1 && sy === 1 ? "" : `scale(${sx}, ${sy})`,
      transformOrigin: "0 0",
    });
    // Over the canvas, whatever its stacking.
    const z = Number.parseInt(this.#canvas.ownerDocument.defaultView?.getComputedStyle(this.#canvas).zIndex ?? "", 10);
    this.root.style.zIndex = String(Number.isFinite(z) ? z + 1 : 1);
  }

  dispose(): void {
    for (const off of this.#off.splice(0)) off();
    this.#standIn = null;
    this.#stand();
    this.root.remove();
    this.#elements.clear();
  }
}

/** The path of the mirrored element an event landed on (or inside). */
function pathOf(target: EventTarget | null): string | null {
  let el = target as HTMLElement | null;
  while (el && el.dataset) {
    if (el.dataset.ogPath !== undefined) return el.dataset.ogPath;
    el = el.parentElement;
  }
  return null;
}

/** A mirrored node: where the control is drawn, drawing nothing. */
const NODE: Partial<CSSStyleDeclaration> = {
  position: "absolute",
  margin: "0",
  padding: "0",
  border: "0",
  overflow: "visible",
  pointerEvents: "none",
  background: "transparent",
  color: "transparent",
  caretColor: "transparent",
  textShadow: "none",
  boxShadow: "none",
  userSelect: "none",
  outline: "none",
  fontSize: "1px",
  whiteSpace: "nowrap",
  appearance: "none",
  boxSizing: "border-box",
};

/** A mirrored range's thumb and track, drawing nothing. */
const THUMB = `[data-og-a11y] input[type=range]::-webkit-slider-thumb{-webkit-appearance:none;appearance:none;background:transparent;border:0;box-shadow:none}
[data-og-a11y] input[type=range]::-webkit-slider-runnable-track{background:transparent;border:0}
[data-og-a11y] input[type=range]::-moz-range-thumb{appearance:none;background:transparent;border:0;box-shadow:none}
[data-og-a11y] input[type=range]::-moz-range-track{background:transparent;border:0}
[data-og-a11y] *::selection{background:transparent}`;
