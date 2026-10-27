// ANGLE on D3D11: textureSize(s, 0) with TEXTURE_BASE_LEVEL = b returns the
// size of level 2b, not b (the glass pyramid's sequence under wgpu's GL
// backend: each level drawn from the one above it, sampled through its base
// level). Prints what textureSize returned against the level's real size.
// bun tests/evidence/release/windows-x64-de04f53d/angle-texture-size.ts <browser exe> [chrome flags]
import { Browser } from "../../../../packages/engine/tests/web/cdp.ts";

const page = `<!doctype html><script>
window.run = () => {
  const c = document.createElement('canvas'); c.width = 4; c.height = 4;
  const gl = c.getContext('webgl2', { antialias: false });
  const d = gl.getExtension('WEBGL_debug_renderer_info');
  const N = 5, S = 64;
  const tex = gl.createTexture();
  gl.bindTexture(gl.TEXTURE_2D, tex);
  gl.texStorage2D(gl.TEXTURE_2D, N, gl.RGBA8, S, S);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST_MIPMAP_NEAREST);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
  const vs = '#version 300 es\\nvoid main(){ float x = float((gl_VertexID & 1) * 4 - 1); float y = float((gl_VertexID & 2) * 2 - 1); gl_Position = vec4(x, y, 0.0, 1.0); }';
  const prog = (fs) => { const p = gl.createProgram(); for (const [t, s] of [[gl.VERTEX_SHADER, vs], [gl.FRAGMENT_SHADER, fs]]) { const sh = gl.createShader(t); gl.shaderSource(sh, s); gl.compileShader(sh); if (!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) throw gl.getShaderInfoLog(sh); gl.attachShader(p, sh); } gl.linkProgram(p); return p; };
  // Level k = colour (k*40) + what it read from level k-1 via textureLod(.., 0) and its textureSize.
  const down = prog('#version 300 es\\nprecision highp float; uniform sampler2D s; uniform float k; out vec4 o; void main(){ vec4 a = textureLod(s, vec2(0.5), 0.0); float w = float(textureSize(s, 0).x); o = vec4(k * 40.0 / 255.0, a.r, w / 255.0, 1.0); }');
  const read = prog('#version 300 es\\nprecision highp float; uniform sampler2D s; uniform float lod; out vec4 o; void main(){ o = textureLod(s, vec2(0.5), lod); }');
  const fb = gl.createFramebuffer();
  const out = {};
  // level 0 by clear
  gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
  gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex, 0);
  gl.clearColor(0, 1, 0, 1); gl.clear(gl.COLOR_BUFFER_BIT);
  for (let k = 1; k < N; k++) {
    gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex, k);
    gl.viewport(0, 0, S >> k, S >> k);
    gl.useProgram(down); gl.uniform1f(gl.getUniformLocation(down, 'k'), k);
    gl.activeTexture(gl.TEXTURE0); gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_BASE_LEVEL, k - 1);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAX_LEVEL, k - 1);
    gl.uniform1i(gl.getUniformLocation(down, 's'), 0);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
  }
  // now the full view: base 0, max N-1; read each level by lod into a 1x1 target
  const rt = gl.createTexture(); gl.bindTexture(gl.TEXTURE_2D, rt); gl.texStorage2D(gl.TEXTURE_2D, 1, gl.RGBA8, 1, 1);
  const fb2 = gl.createFramebuffer(); gl.bindFramebuffer(gl.FRAMEBUFFER, fb2);
  gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, rt, 0);
  gl.viewport(0, 0, 1, 1);
  gl.useProgram(read);
  gl.activeTexture(gl.TEXTURE0); gl.bindTexture(gl.TEXTURE_2D, tex);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_BASE_LEVEL, 0);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAX_LEVEL, N - 1);
  gl.uniform1i(gl.getUniformLocation(read, 's'), 0);
  const px = new Uint8Array(4);
  out.levels = [];
  for (let lod = 0; lod < N; lod++) {
    gl.uniform1f(gl.getUniformLocation(read, 'lod'), lod);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
    gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, px);
    out.levels.push({ lod, k: px[0] / 40, readFromPrev_g: px[1], prevSize: px[2] });
  }
  { const dbg = gl.getExtension('WEBGL_debug_shaders'); const sh = gl.getAttachedShaders(down)[1]; out.hlsl = dbg.getTranslatedShaderSource(sh); }
  out.err = gl.getError();
  out.renderer = gl.getParameter(d.UNMASKED_RENDERER_WEBGL);
  return out;
};
</script>`;

const server = Bun.serve({ port: 0, fetch: () => new Response(page, { headers: { "content-type": "text/html" } }) });
const browser = await Browser.launch({ chrome: process.argv[2], flags: process.argv.slice(3) });
try {
  const pg = await browser.open(`http://127.0.0.1:${server.port}/`);
  await new Promise((r) => setTimeout(r, 500));
  const r = (await pg.eval("window.run()")) as { levels: { k: number; prevSize: number }[]; renderer: string }; console.log(r.renderer + "\n  textureSize(s, 0).x with BASE_LEVEL = k-1 (level k-1 is " + "64>>(k-1)" + "): " + r.levels.slice(1).map((l) => "base " + (l.k - 1) + ": " + l.prevSize + " (want " + (64 >> (l.k - 1)) + ")").join(", "));
} finally {
  await browser.close();
  server.stop(true);
}
