// Text entry on the page (og-input's typed-text stream, crates/og-ui/README.md
// §Input): the canvas cannot take text, so while the engine's UI has a text
// control focused the shell keeps a hidden <textarea> over it, focused. What
// the page inserts there becomes `{ kind: "text" }` input: `beforeinput`'s
// typed characters and pastes are committed text, an input method's
// `compositionupdate` is its composition so far (the field shows it
// underlined) and `compositionend` commits it. The textarea itself stays
// empty. Its keys still reach the engine as key events (the shell's keydown
// listener lets this element through), except while an input method is
// composing, when they are the method's. Copy and cut put the engine's
// selection on the clipboard (the Worker reports it every frame the target
// changes); the engine deletes the cut text from the Ctrl/Cmd+X it sees.
//
// The engine side: the frame answer's `text` (`{ path, rect, multiline,
// selection, composing }` or null) and `copy` (text a copy took), which the
// Worker forwards as `{ type: "text", target }` and `{ type: "copy", text }`.
//
// For assistive technology the textarea is the field while it has the
// focus: it is named after the field's node in the accessibility tree
// (a11y.ts: its name, its placeholder as aria-placeholder, its text as the
// description), and the mirror's own element for the field steps aside
// (aria-hidden) while its parent owns the textarea (aria-owns), so a screen
// reader meets one text box, inside the dialog it belongs to. While no field
// is focused the textarea is aria-hidden; it never holds the focus hidden
// (Chrome blocks that and a reader lands on an unnamed box:
// examples/WEB-FINDINGS.md #7).

import type { InjectedInput } from "./input.ts";

/** What the textarea is named from: the accessibility mirror (a11y.ts A11yMirror). */
export interface TextLabels {
  /** The mirrored node at `path`: its name, placeholder (description) and text (value). */
  node(path: string): { readonly name: string; readonly description: string; readonly value: string } | undefined;
  /** The textarea stands in for the field at `path` (null: for none). */
  standIn(path: string | null, element: HTMLElement | null): void;
}

let entries = 0;

/** The engine's focused text control, as the Worker reports it. */
export interface TextTarget {
  /** The element's path in the UI. */
  path: string;
  /** Its rectangle in CSS pixels of the play surface: x, y, width, height. */
  rect: [number, number, number, number];
  multiline: boolean;
  /** The selected text (empty: none). */
  selection: string;
  /** An input method is composing in it. */
  composing: boolean;
}

/** The hidden textarea that takes the page's text for the engine's focused field. */
export class TextEntry {
  readonly element: HTMLTextAreaElement;
  #canvas: HTMLCanvasElement;
  #send: (event: InjectedInput) => void;
  #target: TextTarget | null = null;
  #labels: TextLabels | undefined;
  #composing = false;
  /** The editing key pressed in this task, so a virtual keyboard's delete without one still deletes. */
  #keyed = "";
  /** When the page last copied through the copy event (the engine's own copy is then already on the clipboard). */
  #copiedAt = 0;
  #off: Array<() => void> = [];

  constructor(canvas: HTMLCanvasElement, send: (event: InjectedInput) => void, labels?: TextLabels) {
    this.#canvas = canvas;
    this.#send = send;
    this.#labels = labels;
    const area = canvas.ownerDocument.createElement("textarea");
    area.id = `og-text-entry${entries++ ? `-${entries}` : ""}`;
    area.setAttribute("aria-hidden", "true");
    area.setAttribute("autocomplete", "off");
    area.setAttribute("autocapitalize", "off");
    area.setAttribute("spellcheck", "false");
    area.tabIndex = -1;
    // Invisible but laid out where the field is, so an input method's
    // candidate window opens beside it; 16px keeps mobile browsers from zooming.
    Object.assign(area.style, {
      position: "fixed",
      opacity: "0",
      pointerEvents: "none",
      resize: "none",
      border: "0",
      padding: "0",
      margin: "0",
      overflow: "hidden",
      whiteSpace: "pre",
      fontSize: "16px",
      caretColor: "transparent",
      background: "transparent",
      color: "transparent",
      left: "0px",
      top: "0px",
      width: "1px",
      height: "1px",
    } satisfies Partial<CSSStyleDeclaration>);
    canvas.ownerDocument.body.appendChild(area);
    this.element = area;
    const on = <K extends keyof HTMLElementEventMap>(type: K, fn: (e: HTMLElementEventMap[K]) => void) => {
      area.addEventListener(type, fn as EventListener);
      this.#off.push(() => area.removeEventListener(type, fn as EventListener));
    };
    on("keydown", (e) => {
      this.#keyed = e.code;
      queueMicrotask(() => (this.#keyed = ""));
    });
    on("beforeinput", (e) => this.#beforeInput(e));
    on("compositionstart", () => {
      this.#composing = true;
    });
    on("compositionupdate", (e) => {
      this.#send({ kind: "text", text: e.data ?? "", composing: true });
    });
    on("compositionend", (e) => {
      this.#composing = false;
      if (e.data) this.#send({ kind: "text", text: e.data });
      else this.#send({ kind: "text", text: "", composing: true });
      area.value = "";
    });
    on("paste", (e) => {
      e.preventDefault();
      const text = e.clipboardData?.getData("text/plain") ?? "";
      if (text) this.#send({ kind: "text", text: this.#target?.multiline ? text : text.replace(/\r?\n/g, " ") });
    });
    const copy = (e: ClipboardEvent) => {
      const text = this.#target?.selection ?? "";
      if (!text) return;
      e.preventDefault();
      e.clipboardData?.setData("text/plain", text);
      this.#copiedAt = performance.now();
    };
    on("copy", copy);
    on("cut", copy);
  }

  /** Whether a key event's target is this textarea (the shell forwards its keys). */
  owns(target: EventTarget | null): boolean {
    return target === this.element;
  }

  /** An input method is composing: its keys are not the game's. */
  get composing(): boolean {
    return this.#composing;
  }

  /** The engine's focused text control changed: follow it, focus or let go. */
  update(target: TextTarget | null): void {
    const had = this.#target !== null;
    this.#target = target;
    const area = this.element;
    if (!target) {
      if (had && area.ownerDocument.activeElement === area) area.blur();
      this.relabel();
      return;
    }
    // Named and visible to assistive technology before it takes the focus.
    this.relabel();
    const box = this.#canvas.getBoundingClientRect();
    const sx = box.width / (this.#canvas.clientWidth || box.width || 1);
    const sy = box.height / (this.#canvas.clientHeight || box.height || 1);
    const [x, y, w, h] = target.rect;
    Object.assign(area.style, {
      left: `${box.left + x * sx}px`,
      top: `${box.top + y * sy}px`,
      width: `${Math.max(1, w * sx)}px`,
      height: `${Math.max(1, h * sy)}px`,
    });
    if (!had || area.ownerDocument.activeElement !== area) {
      area.focus({ preventScroll: true });
      // An empty commit tells the engine the page has a text stream now, so
      // a key's character and its typed text never both type.
      this.#send({ kind: "text", text: "" });
    }
  }

  /**
   * Name the textarea after the focused field's node (call when the
   * mirrored tree changed): or hide it from assistive technology while no
   * field is focused.
   */
  relabel(): void {
    const area = this.element;
    const target = this.#target;
    const set = (name: string, value: string | null) => {
      if (value === null) area.removeAttribute(name);
      else if (area.getAttribute(name) !== value) area.setAttribute(name, value);
    };
    if (!target) {
      for (const a of ["aria-label", "aria-placeholder", "aria-description", "aria-multiline"]) set(a, null);
      set("aria-hidden", "true");
      this.#labels?.standIn(null, null);
      return;
    }
    const node = this.#labels?.node(target.path);
    set("aria-hidden", null);
    // Unmirrored (no tree yet): the field's own id stands for its name.
    set("aria-label", node?.name || target.path.split("/").pop() || null);
    set("aria-placeholder", node?.description || null);
    set("aria-description", node?.value || null);
    set("aria-multiline", target.multiline ? "true" : "false");
    this.#labels?.standIn(target.path, area);
  }

  /** The engine copied without a copy event (a game's own copy): best effort. */
  copied(text: string): void {
    if (performance.now() - this.#copiedAt < 500) return;
    void globalThis.navigator?.clipboard?.writeText(text).catch(() => {});
  }

  #beforeInput(e: InputEvent): void {
    switch (e.inputType) {
      // The composition: compositionupdate and compositionend carry it.
      case "insertCompositionText":
      case "insertFromComposition":
      case "deleteByComposition":
      case "deleteCompositionText":
        return;
      case "insertText":
      case "insertReplacementText":
      case "insertFromDrop":
      case "insertFromPaste": {
        e.preventDefault();
        const text = e.data ?? e.dataTransfer?.getData("text/plain") ?? "";
        if (text) this.#send({ kind: "text", text });
        return;
      }
      case "deleteContentBackward":
      case "deleteContentForward": {
        e.preventDefault();
        // A virtual keyboard deletes without a key event the engine saw.
        const key = e.inputType === "deleteContentBackward" ? "Backspace" : "Delete";
        if (this.#keyed !== key) this.#send({ kind: "key", key, action: "press" });
        return;
      }
      default:
        // Line breaks arrive as Enter, everything else is the engine's.
        e.preventDefault();
    }
  }

  dispose(): void {
    for (const off of this.#off.splice(0)) off();
    this.#labels?.standIn(null, null);
    this.element.remove();
  }
}
