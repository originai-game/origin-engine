// The wasm-bindgen glue's place for tests/web-shell-crash.test.ts: the Worker
// imports it as `glue`, and loadWebHost keeps the host already bound (the
// native addon), so the real Worker (src/web/worker.ts) runs a game without
// a browser or a wasm build. Only what the start path reads is here, and
// the trace's two members (tests/web-trace.test.ts).
import { native } from "../../../src/node/index.ts";

export default async function init(): Promise<never> {
  throw new Error("the native host is bound before the Worker starts; this glue is never initialised");
}
export function og_web_init(): void {}
export function og_web_set_viewport(_engine: number, _viewport: string): string {
  return "";
}
export async function og_web_attach_canvas(_engine: number, _canvas: unknown, _webgl: boolean): Promise<string> {
  return "none";
}
// The trace (src/web/worker.ts `trace`): the wasm's og_web_trace and
// og_web_trace_take are og-runtime's trace, as the native addon's are.
export function og_web_trace(engine: number, on: boolean): void {
  native().og_node_trace(engine, on);
}
export function og_web_trace_take(engine: number, last: boolean): string {
  return native().og_node_trace_take(engine, last);
}
