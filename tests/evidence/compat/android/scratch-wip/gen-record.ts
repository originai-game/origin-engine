// Scratch: meshlet_record copies with parts removed, each with its own entry.
import { readFileSync, writeFileSync } from "node:fs";
const rec = readFileSync(process.argv[2]!, "utf8").replace(/\r\n/g, "\n");
const cut = (s: string, re: RegExp) => {
  if (!re.test(s)) throw new Error("no match " + re);
  return s.replace(re, "");
};
const extraBlock = /  if \(meshlet_screen\.w > 8u\) \{[\s\S]*?\n  \}\n/;
const variants: Record<string, (s: string) => string> = {
  full: (s) => s,
  noextra: (s) => cut(s, extraBlock),
  noreturns: (s) => cut(cut(s, /  if \(slot == 0u\) \{ return r; \}\n/), /  if \(c\.z != row\) \{ return r; \}\n/),
  noslotreturn: (s) => cut(s, /  if \(slot == 0u\) \{ return r; \}\n/),
  norowreturn: (s) => cut(s, /  if \(c\.z != row\) \{ return r; \}\n/),
  noanalytic: (s) => cut(cut(cut(s, /  analytic_uv = [^\n]*\n/), /  analytic_dx = [^\n]*\n/), /  analytic_dy = [^\n]*\n/),
  onlyuv: (s) => cut(cut(s, /  analytic_dx = [^\n]*\n/), /  analytic_dy = [^\n]*\n/),
  onlydx: (s) => cut(cut(s, /  analytic_uv = [^\n]*\n/), /  analytic_dy = [^\n]*\n/),
  onlydy: (s) => cut(cut(s, /  analytic_uv = [^\n]*\n/), /  analytic_dx = [^\n]*\n/),
  nouv: (s) => cut(s, /  analytic_uv = [^\n]*\n/),
  notangentif: (s) => cut(s, /  if \(r\.v\.tangent\.w == 0\.0\) [^\n]*\n/),
};
let out = "";
const names: string[] = [];
for (const [k, f] of Object.entries(variants)) {
  const body = f(rec).replace("fn meshlet_record(", `fn rec_${k}(`);
  out += `\n${body}\n@fragment\nfn v_rec_${k}(in: MeshletResolveOut) -> MeshletResolveLit {\n  let r = rec_${k}(in.clip, in.row);\n  if (!r.ok) { discard; }\n  var o: MeshletResolveLit;\n  o.depth = r.depth;\n  hook_front = 1.0;\n  analytic_uv = r.lit_uv;\n  o.color = lit_fragment(r.v);\n  return o;\n}\n`;
  names.push(`v_rec_${k}`);
}
writeFileSync(process.argv[3]!, out);
console.log(names.join(","));

