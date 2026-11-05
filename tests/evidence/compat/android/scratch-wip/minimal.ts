// Scratch: minimal shaders on the phone's Dawn/Vulkan/PowerVR stack.
import { Browser } from "../../../packages/engine/tests/web/cdp.ts";
const SHADERS: Record<string, string> = {
  dependent_cross: `
@group(0) @binding(0) var<uniform> screen: vec4<u32>;
@group(0) @binding(1) var<storage, read> attrs: array<vec4<u32>>;
var<private> gdx: vec3<f32>;
var<private> gdy: vec3<f32>;
@vertex fn vs(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4<f32> {
  let p = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u));
  return vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0);
}
@fragment fn fs(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let pixel = u32(pos.y) * screen.x + u32(pos.x);
  let slot = attrs[pixel / 4u][pixel % 4u];
  if (slot == 0u) { discard; }
  let at = screen.z / 4u + (slot - 1u) * screen.w;
  gdx = bitcast<vec3<f32>>(attrs[at + 6u].xyz);
  gdy = bitcast<vec3<f32>>(attrs[at + 7u].xyz);
  return vec4<f32>(normalize(cross(gdx, gdy)), 1.0);
}`,
};
const server = Bun.serve({ port: 0, fetch: () => new Response("<!doctype html><title>m</title>", { headers: { "content-type": "text/html" } }) });
const browser = await Browser.from(["--android"], { reverse: [server.port] });
const page = await browser.open(`http://127.0.0.1:${server.port}/`, []);
await page.until("document.readyState === 'complete'", 15000);
console.log(JSON.stringify(await page.eval(`(async () => {
  const a = await navigator.gpu.requestAdapter();
  const d = await a.requestDevice();
  const out = {};
  for (const [name, code] of Object.entries(${JSON.stringify(SHADERS)})) {
    const m = d.createShaderModule({ code });
    try {
      await d.createRenderPipelineAsync({ layout: "auto", vertex: { module: m, entryPoint: "vs" }, fragment: { module: m, entryPoint: "fs", targets: [{ format: "rgba16float" }] } });
      out[name] = "ok";
    } catch (e) { out[name] = (e.reason || "") + ": " + String(e.message).split("\\n")[0]; }
  }
  return out;
})()`), null, 1));
await browser.close();
server.stop();
