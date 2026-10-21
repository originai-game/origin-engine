// The accessibility mirror (crates/og-ui/README.md §Accessibility tree): the
// UI is drawn into a canvas, which a screen reader cannot read, so the shell
// keeps a DOM copy of `og.ui.a11y()` over it. Each node with a role becomes
// an invisible element at the node's bounds, nested as the tree nests it,
// with its ARIA role, name, value and states; the Worker sends the tree
// with a frame's answer only when its digest changed (og-runtime's
// `a11y_changed`), so an unchanged screen costs the page nothing.
//
// - Focus follows og-ui's: the element of the focused path takes DOM focus
//   (unless the page's focus is in a field of its own, or the UI's text
//   control has the hidden textarea, text-input.ts). While that textarea
//   has the focus it is the field for assistive technology (named after
//   the field's node): the field's own element is aria-hidden and its
//   parent owns the textarea (`standIn`).
// - Toasts are announced: a status node that appears is written into a
//   polite live region, an alert into an assertive one.
// - The DOM reaches the UI as input: a screen reader's click on an element
//   is `{ kind: "a11y", action: "activate", path }` and its moving focus
//   onto one is `focus`; `adjust` sends `increment`/`decrement` for an
//   adjustable one. A slider and a stepper's spinbutton are mirrored as an
//   `<input type="range">` (with its role: `slider`, `spinbutton`) held at
//   the UI's value, one step from either end: a screen reader's own adjust
//   gesture (VoiceOver's swipe up or down, TalkBack's volume keys, an arrow
//   while interacting) moves the native value, or sets it from
//   `aria-valuenow` and the step, and fires `input`, which becomes
//   `increment` or `decrement` and puts the value back (the UI owns the
//   value and its step; `aria-valuenow` and `aria-valuetext` say it, and
//   `aria-valuemin` and `aria-valuemax` its bounds from the tree's
//   `valueMin` and `valueMax`, over the native range's own). A
//   stepper's − and + follow its range as siblings (an input holds no
//   children). Keys pressed while a
//   mirrored element has the DOM focus still go to the engine as keys (the
//   shell listens on the page and keeps the arrows from the native range),
//   where the focused control takes them: arrows move a focused slider.
//   og-ui acts on the requests at the next frame against the controls it
//   drew (nav.rs `apply_a11y`).
//
// The elements take no pointer events (the canvas does) and draw nothing:
// they exist for assistive technology, and a screen reader's focus ring
// lands where the control is drawn.

import type { InjectedInput } from "./input.ts";

/** One node of the UI's accessibility tree (IDL ui.A11yNode). */
export interface A11yNode {
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
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

/** The UI's accessibility tree (IDL ui.A11yTree). */
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
/** Roles read as their text content rather than a label. */
const TEXTS = new Set(["text", "heading", "status", "alert", "tooltip"]);

/** The DOM mirror of the UI's accessibility tree, over the canvas. */
export class A11yMirror {
  readonly root: HTMLDivElement;
  readonly #canvas: HTMLCanvasElement;
  readonly #send: (event: InjectedInput) => void;
  readonly #polite: HTMLDivElement;
  readonly #assertive: HTMLDivElement;
  readonly #elements = new Map<string, HTMLElement>();
  #tree: A11yTree = { nodes: [], focusOrder: [], focused: "" };
  /** Status and alert nodes already announced, by path and text. */
  #announced = new Set<string>();
  /** The path this mirror last moved DOM focus to (its own focus events are not the reader's). */
  #focusing = "";
  /** The text field the text entry's textarea stands in for (text-input.ts), and the textarea. */
  #standIn: { path: string; element: HTMLElement } | null = null;
  /** What standing in changed: the field's element hidden, its parent owning the textarea. */
  #stood: { hidden: HTMLElement; owner: HTMLElement } | null = null;
  #off: Array<() => void> = [];

  constructor(canvas: HTMLCanvasElement, send: (event: InjectedInput) => void) {
    this.#canvas = canvas;
    this.#send = send;
    const doc = canvas.ownerDocument;
    const root = doc.createElement("div");
    root.dataset.ogA11y = "";
    Object.assign(root.style, { position: "fixed", left: "0px", top: "0px", width: "0px", height: "0px", pointerEvents: "none", overflow: "visible", zIndex: "0" } satisfies Partial<CSSStyleDeclaration>);
    const region = (live: "polite" | "assertive") => {
      const r = doc.createElement("div");
      r.setAttribute("aria-live", live);
      r.setAttribute("aria-atomic", "false");
      r.setAttribute("role", live === "polite" ? "status" : "alert");
      r.dataset.ogA11yLive = live;
      Object.assign(r.style, HIDDEN);
      root.appendChild(r);
      return r;
    };
    this.#polite = region("polite");
    this.#assertive = region("assertive");
    doc.body.appendChild(root);
    this.root = root;
    const on = <K extends keyof HTMLElementEventMap>(type: K, fn: (e: HTMLElementEventMap[K]) => void) => {
      root.addEventListener(type, fn as EventListener);
      this.#off.push(() => root.removeEventListener(type, fn as EventListener));
    };
    // A screen reader activates an element with a click (its default action).
    on("click", (e) => {
      const path = pathOf(e.target);
      if (path === null) return;
      e.preventDefault();
      this.#send({ kind: "a11y", action: "activate", path });
    });
    // A screen reader adjusted a mirrored range: one step, and back to the UI's value.
    const adjusted = (e: Event) => {
      const el = e.target as HTMLInputElement | null;
      if (!el || el.tagName !== "INPUT" || el.type !== "range") return;
      const path = pathOf(el);
      const v = Number(el.value);
      const at = Number(el.dataset.ogValue ?? "1");
      el.value = String(at);
      if (path === null || !Number.isFinite(v) || v === at) return;
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

  /** The UI's tree changed: bring the DOM in line with it. */
  update(tree: A11yTree): void {
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
      Object.assign(el.style, {
        left: `${node.x - px}px`,
        top: `${node.y - py}px`,
        width: `${Math.max(1, node.width)}px`,
        height: `${Math.max(1, node.height)}px`,
      });
    }
    // The live regions stay last, after every layer.
    this.root.append(this.#polite, this.#assertive);
    this.#announce(tree);
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
    if (node.role === "heading") set("aria-level", "2");
    if (node.role === "dialog") set("aria-modal", "true");
    const focusable = states.has("focusable") || tree.focusOrder.includes(node.path);
    // A native range is in the page's Tab order unless told otherwise; the UI owns focus.
    set("tabindex", focusable || el instanceof HTMLInputElement ? "-1" : null);
    set("aria-disabled", states.has("disabled") ? "true" : null);
    set("aria-checked", CHECKABLE.has(node.role) ? String(states.has("checked")) : null);
    set("aria-selected", SELECTABLE.has(node.role) ? String(states.has("selected")) : null);
    set("aria-expanded", EXPANDABLE.has(node.role) ? String(states.has("expanded")) : null);
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

  /** Status and alert nodes that appeared are said once through the live regions. */
  #announce(tree: A11yTree): void {
    const now = new Set<string>();
    for (const n of tree.nodes) {
      if ((n.role !== "status" && n.role !== "alert") || !n.name) continue;
      const key = `${n.path}\u0000${n.name}`;
      now.add(key);
      if (this.#announced.has(key)) continue;
      const region = n.role === "alert" ? this.#assertive : this.#polite;
      const line = region.ownerDocument.createElement("div");
      line.textContent = n.name;
      region.appendChild(line);
      // Keep the region short: a reader has said what was added.
      while (region.childElementCount > 3) region.firstElementChild?.remove();
    }
    this.#announced = now;
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
  opacity: "0",
  overflow: "visible",
  pointerEvents: "none",
  background: "transparent",
  color: "transparent",
  outline: "none",
  fontSize: "1px",
  whiteSpace: "nowrap",
  appearance: "none",
  boxSizing: "border-box",
};

/** A live region: read, never seen. */
const HIDDEN: Partial<CSSStyleDeclaration> = {
  position: "absolute",
  width: "1px",
  height: "1px",
  overflow: "hidden",
  clipPath: "inset(50%)",
  whiteSpace: "nowrap",
};
