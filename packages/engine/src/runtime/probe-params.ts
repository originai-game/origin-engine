import bindings from "../../types/bindings.json" with { type: "json" };

/** A probe member's parameter as the binding table declares it. */
interface ProbeParam {
  readonly name: string;
  readonly type: string;
  readonly optional: boolean;
}

/**
 * The probe members' string and number parameters, with their positions. A
 * probe is driven by untyped callers (the engine Worker's `call("probe", …)`,
 * a console): on the web host a non-string where the ABI takes a string traps
 * the wasm inside wasm-bindgen's string passing (out-of-bounds memory), and
 * the native host throws a conversion error, so both hosts refuse it here,
 * before the host sees it, with the probe's own words ("script must be a
 * string").
 */
const PROBE_PARAMS = new Map(
  (bindings.calls as Array<{ module: string; kind: string; name: string; params?: ProbeParam[] }>)
    .filter((c) => c.module === "probe" && c.kind === "method")
    .map((c) => [
      c.name,
      (c.params ?? [])
        .map((p, at) => ({ ...p, at, want: p.type === "String" ? "string" : ["u32", "i32", "f32", "f64"].includes(p.type) ? "number" : "" }))
        .filter((p) => p.want !== ""),
    ]),
);

function checkedProbe<T extends Record<string, unknown>>(members: T): T {
  const out: Record<string, unknown> = {};
  for (const [name, member] of Object.entries(Object.getOwnPropertyDescriptors(members))) {
    const params = PROBE_PARAMS.get(name) ?? [];
    const f = member.value as ((...args: unknown[]) => unknown) | undefined;
    if (params.length === 0 || typeof f !== "function") {
      Object.defineProperty(out, name, member);
      continue;
    }
    out[name] = (...args: unknown[]) => {
      for (const p of params) {
        const v = args[p.at];
        if (v === undefined && p.optional) continue;
        if (typeof v !== p.want) throw new TypeError(`probe.${name}: ${p.name} must be a ${p.want}, not ${v === null ? "null" : typeof v}`);
      }
      return f(...args);
    };
  }
  return Object.freeze(out) as T;
}

export { checkedProbe };
