// og-render's glass pyramid in raw WebGL2, built the way wgpu's GL
// backend builds it (one RGBA16F texture, 6 levels; each down pass samples
// level k-1 through TEXTURE_BASE_LEVEL = TEXTURE_MAX_LEVEL = k-1 and renders
// to level k), against the same filter on one texture per level (no base
// level). Then the panel's read (textureLod at lod 3) of each, compared.
// bun tests/evidence/release/windows-x64-de04f53d/angle-glass-pyramid.ts <browser exe> [chrome flags] [--full]
import { Browser } from "../../../../packages/engine/tests/web/cdp.ts";

const page = `<!doctype html><script>
window.run = () => {
  const W = 320, H = 200, LEVELS = 6;
  const c = document.createElement('canvas'); c.width = 4; c.height = 4;
  const gl = c.getContext('webgl2', { antialias: false });
  gl.getExtension('EXT_color_buffer_float');
  const d = gl.getExtension('WEBGL_debug_renderer_info');
  const vs = '#version 300 es\\nout vec2 uv; void main(){ float x = float((gl_VertexID & 1) * 4 - 1); float y = float((gl_VertexID & 2) * 2 - 1); gl_Position = vec4(x, y, 0.0, 1.0); uv = vec2(x * 0.5 + 0.5, y * 0.5 + 0.5); }';
  const prog = (fs) => { const p = gl.createProgram(); for (const [t, s] of [[gl.VERTEX_SHADER, vs], [gl.FRAGMENT_SHADER, fs]]) { const sh = gl.createShader(t); gl.shaderSource(sh, s); gl.compileShader(sh); if (!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) throw gl.getShaderInfoLog(sh); gl.attachShader(p, sh); } gl.linkProgram(p); if (!gl.getProgramParameter(p, gl.LINK_STATUS)) throw gl.getProgramInfoLog(p); return p; };
  // The source: stripes (4 of 8 px white) and an orange bar at y 92..108.
  const src = prog('#version 300 es\\nprecision highp float; out vec4 o; void main(){ vec2 p = gl_FragCoord.xy; float s = mod(floor(p.x), 8.0) < 4.0 ? 1.0 : 0.0; vec3 c = vec3(s); if (p.y > 92.0 && p.y < 108.0 && p.x > 40.0 && p.x < 280.0) c = vec3(1.0, 0.45, 0.0); o = vec4(c, 1.0); }');
  const down = prog('#version 300 es\\nprecision highp float; uniform sampler2D s; in vec2 uv; out vec4 o;\\nvec3 tap(vec2 u){ return textureLod(s, u, 0.0).rgb; }\\nvoid main(){ vec2 h = vec2(1.0) / vec2(uvec2(textureSize(s, 0).xy)); vec3 c = tap(uv) * 4.0; c += tap(uv + vec2(-h.x, -h.y)); c += tap(uv + vec2(h.x, -h.y)); c += tap(uv + vec2(-h.x, h.y)); c += tap(uv + vec2(h.x, h.y)); vec2 f = 2.0 * h; c += tap(uv + vec2(-f.x, 0.0)); c += tap(uv + vec2(f.x, 0.0)); c += tap(uv + vec2(0.0, -f.y)); c += tap(uv + vec2(0.0, f.y)); o = vec4(c / 12.0, 1.0); }');
  const read = prog('#version 300 es\\nprecision highp float; uniform sampler2D s; uniform float lod; in vec2 uv; out vec4 o; void main(){ o = textureLod(s, gl_FragCoord.xy / vec2(' + W + '.0, ' + H + '.0), lod); }');
  const fb = gl.createFramebuffer();
  const target = (tex, level, w, h) => { gl.bindFramebuffer(gl.FRAMEBUFFER, fb); gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex, level); gl.viewport(0, 0, w, h); gl.clearColor(0, 0, 0, 1); gl.clear(gl.COLOR_BUFFER_BIT); };
  const linear = (mip) => { gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, mip ? gl.LINEAR_MIPMAP_LINEAR : gl.LINEAR); gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR); gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE); gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE); };
  const source = gl.createTexture(); gl.bindTexture(gl.TEXTURE_2D, source); gl.texStorage2D(gl.TEXTURE_2D, 1, gl.RGBA8, W, H); linear(false);
  target(source, 0, W, H); gl.useProgram(src); gl.drawArrays(gl.TRIANGLES, 0, 3);
  const pw = W / 2, ph = H / 2;
  const sizes = []; for (let k = 0; k < LEVELS; k++) sizes.push([Math.max(pw >> k, 1), Math.max(ph >> k, 1)]);
  const drawDown = (from, level, to, w, h) => { target(to, level, w, h); gl.useProgram(down); gl.activeTexture(gl.TEXTURE0); gl.bindTexture(gl.TEXTURE_2D, from.tex); gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_BASE_LEVEL, from.base); gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAX_LEVEL, from.max); gl.uniform1i(gl.getUniformLocation(down, 's'), 0); gl.drawArrays(gl.TRIANGLES, 0, 3); };
  // (a) wgpu's way: one texture, base/max per pass.
  const pyr = gl.createTexture(); gl.bindTexture(gl.TEXTURE_2D, pyr); gl.texStorage2D(gl.TEXTURE_2D, LEVELS, gl.RGBA16F, pw, ph); linear(true);
  drawDown({ tex: source, base: 0, max: 0 }, 0, pyr, ...sizes[0]);
  for (let k = 1; k < LEVELS; k++) drawDown({ tex: pyr, base: k - 1, max: k - 1 }, k, pyr, ...sizes[k]);
  // (c) as (a), but the down passes sample without a mip filter (MIN_FILTER LINEAR).
  const pyc = gl.createTexture(); gl.bindTexture(gl.TEXTURE_2D, pyc); gl.texStorage2D(gl.TEXTURE_2D, LEVELS, gl.RGBA16F, pw, ph); linear(false);
  drawDown({ tex: source, base: 0, max: 0 }, 0, pyc, ...sizes[0]);
  for (let k = 1; k < LEVELS; k++) drawDown({ tex: pyc, base: k - 1, max: k - 1 }, k, pyc, ...sizes[k]);
  // (b) one texture per level.
  const sep = sizes.map(([w, h]) => { const t = gl.createTexture(); gl.bindTexture(gl.TEXTURE_2D, t); gl.texStorage2D(gl.TEXTURE_2D, 1, gl.RGBA16F, w, h); linear(false); return t; });
  drawDown({ tex: source, base: 0, max: 0 }, 0, sep[0], ...sizes[0]);
  for (let k = 1; k < LEVELS; k++) drawDown({ tex: sep[k - 1], base: 0, max: 0 }, k === 0 ? 0 : 0, sep[k], ...sizes[k]);
  // Read: the panel samples lod 3 over the full frame; read column x = 120.
  const outTex = gl.createTexture(); gl.bindTexture(gl.TEXTURE_2D, outTex); gl.texStorage2D(gl.TEXTURE_2D, 1, gl.RGBA8, W, H);
  const column = (tex, base, max, lod) => {
    target(outTex, 0, W, H); gl.useProgram(read); gl.activeTexture(gl.TEXTURE0); gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_BASE_LEVEL, base); gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAX_LEVEL, max);
    gl.uniform1i(gl.getUniformLocation(read, 's'), 0); gl.uniform1f(gl.getUniformLocation(read, 'lod'), lod);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
    const px = new Uint8Array(W * H * 4); gl.readPixels(0, 0, W, H, gl.RGBA, gl.UNSIGNED_BYTE, px);
    const col = []; for (const y of [40, 60, 70, 80, 86, 90, 95, 100, 105, 110, 114, 120, 130, 140, 160]) { const i = (y * W + 120) * 4; col.push(y + ':' + px[i] + ',' + px[i + 1] + ',' + px[i + 2]); }
    return { col: col.join(' '), px };
  };
  const out = { renderer: gl.getParameter(d.UNMASKED_RENDERER_WEBGL) };
  const a = column(pyr, 0, LEVELS - 1, 3);
  const b = column(sep[3], 0, 0, 0);
  out.a_lod3 = a.col; out.b_level3 = b.col;
  let maxd = 0; for (let i = 0; i < a.px.length; i++) maxd = Math.max(maxd, Math.abs(a.px[i] - b.px[i]));
  out.max_diff_a_b = maxd;
  // Each level of (a) on its own (base = max = k), against (b).
  out.per_level = [];
  for (let k = 0; k < LEVELS; k++) { const x = column(pyr, k, k, 0), y = column(sep[k], 0, 0, 0); let m = 0; for (let i = 0; i < x.px.length; i++) m = Math.max(m, Math.abs(x.px[i] - y.px[i])); out.per_level.push(m); }
  out.per_level_c = [];
  for (let k = 0; k < LEVELS; k++) { const x = column(pyc, k, k, 0), y = column(sep[k], 0, 0, 0); let m = 0; for (let i = 0; i < x.px.length; i++) m = Math.max(m, Math.abs(x.px[i] - y.px[i])); out.per_level_c.push(m); }
  out.err = gl.getError();
  return out;
};
</script>`;

const server = Bun.serve({ port: 0, fetch: () => new Response(page, { headers: { "content-type": "text/html" } }) });
const browser = await Browser.launch({ chrome: process.argv[2], flags: process.argv.slice(3).filter((f) => f !== "--full") });
try {
  const pg = await browser.open(`http://127.0.0.1:${server.port}/`);
  await new Promise((r) => setTimeout(r, 500));
  const r = (await pg.eval("window.run()")) as Record<string, unknown>; console.log(process.argv.includes("--full") ? JSON.stringify(r, null, 1) : `${r.renderer}\n  a vs b max ${r.max_diff_a_b}; per level (a) ${JSON.stringify(r.per_level)} (c) ${JSON.stringify(r.per_level_c)} err ${r.err}`);
} finally {
  await browser.close();
  server.stop(true);
}
