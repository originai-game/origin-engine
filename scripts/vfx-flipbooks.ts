// Offline stable-fluids flipbooks (Stam 1999), buoyancy and vorticity
// confinement (Fedkiw, Stam & Jensen 2001), shadowed single scattering and
// blackbody emission. bun scripts/vfx-flipbooks.ts <out-dir> [name ...]
// 64^3 f64 cells, 64 frames, 8x8 straight-alpha sheets of 128px cells.
// Noise uses seeded MT19937 + Box-Muller instead of NumPy's PCG64/Ziggurat;
// the solve, dimensions, seed selection and rendering remain the same.
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { encodePngBest } from "../packages/engine/src/node/png.ts";
import { Random } from "./random.ts";
const N = 64,
  NN = N * N,
  SIZE = N ** 3,
  FRAMES = 64,
  CELL = 128,
  GRID = 8;
type Field = Float64Array;
const field = () => new Float64Array(SIZE),
  clamp = (v: number) => Math.max(0, Math.min(1, v));
function gaussian(input: Field, sigma: number): Field {
  const radius = Math.floor(4 * sigma + 0.5),
    weights = Array.from({ length: radius * 2 + 1 }, (_, i) => Math.exp(-0.5 * ((i - radius) / sigma) ** 2)),
    total = weights.reduce((a, b) => a + b, 0);
  for (let i = 0; i < weights.length; i++) weights[i]! /= total;
  let src = input;
  for (const stride of [NN, N, 1]) {
    const out = field();
    for (let i = 0; i < SIZE; i++) {
      const c = Math.floor(i / stride) % N;
      let sum = 0;
      for (let k = -radius; k <= radius; k++) {
        let p = c + k;
        if (p < 0) p = -p - 1;
        else if (p >= N) p = 2 * N - p - 1;
        sum += src[i + (p - c) * stride]! * weights[k + radius]!;
      }
      out[i] = sum;
    }
    src = out;
  }
  return src;
}
function advect(src: Field, u: Field, v: Field, w: Field, dt: number): Field {
  const out = field();
  for (let z = 0, i = 0; z < N; z++)
    for (let y = 0; y < N; y++)
      for (let x = 0; x < N; x++, i++) {
        const xx = x - dt * u[i]!,
          yy = y - dt * v[i]!,
          zz = z - dt * w[i]!;
        if (xx < 0 || xx > N - 1 || yy < 0 || yy > N - 1 || zz < 0 || zz > N - 1) continue;
        const x0 = Math.floor(xx),
          y0 = Math.floor(yy),
          z0 = Math.floor(zz),
          fx = xx - x0,
          fy = yy - y0,
          fz = zz - z0;
        let val = 0;
        for (let k = 0; k < 2; k++)
          for (let j = 0; j < 2; j++)
            for (let h = 0; h < 2; h++) {
              const index = Math.min(N - 1, z0 + k) * NN + Math.min(N - 1, y0 + j) * N + Math.min(N - 1, x0 + h);
              val += src[index]! * (k ? fz : 1 - fz) * (j ? fy : 1 - fy) * (h ? fx : 1 - fx);
            }
        out[i] = val;
      }
  return out;
}
function project(u: Field, v: Field, w: Field): void {
  const div = field();
  let p = field(),
    next = field();
  for (let z = 1; z < N - 1; z++)
    for (let y = 1; y < N - 1; y++)
      for (let x = 1; x < N - 1; x++) {
        const i = z * NN + y * N + x;
        div[i] = -0.5 * (u[i + 1]! - u[i - 1]! + v[i + N]! - v[i - N]! + w[i + NN]! - w[i - NN]!);
      }
  for (let iteration = 0; iteration < 30; iteration++) {
    for (let z = 1; z < N - 1; z++)
      for (let y = 1; y < N - 1; y++)
        for (let x = 1; x < N - 1; x++) {
          const i = z * NN + y * N + x;
          next[i] = (div[i]! + p[i + 1]! + p[i - 1]! + p[i + N]! + p[i - N]! + p[i + NN]! + p[i - NN]!) / 6;
        }
    [p, next] = [next, p];
  }
  for (let z = 1; z < N - 1; z++)
    for (let y = 1; y < N - 1; y++)
      for (let x = 1; x < N - 1; x++) {
        const i = z * NN + y * N + x;
        u[i]! -= 0.5 * (p[i + 1]! - p[i - 1]!);
        v[i]! -= 0.5 * (p[i + N]! - p[i - N]!);
        w[i]! -= 0.5 * (p[i + NN]! - p[i - NN]!);
      }
}
function gradient(f: Field, stride: number): Field {
  return Float64Array.from(f, (_, i) => {
    const c = Math.floor(i / stride) % N;
    return c === 0 ? f[i + stride]! - f[i]! : c === N - 1 ? f[i]! - f[i - stride]! : (f[i + stride]! - f[i - stride]!) / 2;
  });
}
function vorticity(u: Field, v: Field, w: Field, eps: number, dt: number): void {
  const diff = (a: Field, b: Field) => Float64Array.from(a, (x, i) => x - b[i]!);
  const wx = diff(gradient(w, N), gradient(v, NN)),
    wy = diff(gradient(u, NN), gradient(w, 1)),
    wz = diff(gradient(v, 1), gradient(u, N));
  const mag = Float64Array.from(wx, (x, i) => Math.sqrt(x * x + wy[i]! ** 2 + wz[i]! ** 2) + 1e-6),
    nx = gradient(mag, 1),
    ny = gradient(mag, N),
    nz = gradient(mag, NN);
  for (let i = 0; i < SIZE; i++) {
    const nl = Math.sqrt(nx[i]! ** 2 + ny[i]! ** 2 + nz[i]! ** 2) + 1e-6,
      x = nx[i]! / nl,
      y = ny[i]! / nl,
      z = nz[i]! / nl;
    u[i]! += dt * eps * (y * wz[i]! - z * wy[i]!);
    v[i]! += dt * eps * (z * wx[i]! - x * wz[i]!);
    w[i]! += dt * eps * (x * wy[i]! - y * wx[i]!);
  }
}
const stops = [
  [0, 0, 0],
  [0.55, 0.05, 0.01],
  [1.6, 0.42, 0.06],
  [3.2, 1.6, 0.35],
  [5, 4.2, 2.6],
];
type Frame = { rgb: Float64Array; alpha: Float64Array; glow: Float64Array };
function render(d: Field, temp: Field, smoke: number[], absorb: number, emit: number): Frame {
  const sigma = Float64Array.from(d, (x) => ((absorb * x) / N) * 8),
    shadow = field();
  for (const [stride, weight] of [
    [N, 1],
    [1, 0.8],
    [NN, 0.05],
  ]) {
    const cumsum = field();
    for (let i = SIZE - 1; i >= 0; i--) {
      const c = Math.floor(i / stride!) % N;
      cumsum[i] = sigma[i]! + (c < N - 1 ? cumsum[i + stride!]! : 0);
      shadow[i]! += cumsum[i]! * weight!;
    }
  }
  const blurred = gaussian(shadow, 1.2),
    rgb = new Float64Array(NN * 3),
    alpha = new Float64Array(NN),
    glow = new Float64Array(NN * 3);
  for (let y = 0; y < N; y++)
    for (let x = 0; x < N; x++) {
      let trans = 1;
      const col = [0, 0, 0];
      for (let z = 0; z < N; z++) {
        const i = z * NN + y * N + x,
          stepT = Math.exp(-sigma[i]!),
          a = 1 - stepT,
          lit = Math.exp(-blurred[i]!) + 0.16 + (0.12 * y) / N,
          t = clamp(temp[i]!) * 4,
          k = Math.min(3, Math.floor(t)),
          f = t - k;
        for (let c = 0; c < 3; c++) {
          const e = (stops[k]![c]! * (1 - f) + stops[k + 1]![c]! * f) * emit;
          col[c]! += trans * (a * lit * smoke[c]! + e * (1 / N) * 8);
        }
        trans *= stepT;
      }
      const at = (N - 1 - y) * N + x,
        a = 1 - trans;
      alpha[at] = a;
      for (let c = 0; c < 3; c++) {
        const color = a > 1e-4 ? col[c]! / Math.max(a, 1e-4) : 0;
        rgb[at * 3 + c] = color;
        glow[at * 3 + c] = Math.max(0, col[c]! - color * a);
      }
    }
  return { rgb, alpha, glow };
}
const PARAMS = {
  fire: { buoy: 0.55, cool: 0.035, fade: 0.985, eps: 1.6, smoke: [0.25, 0.22, 0.2], absorb: 2, emit: 1.4 },
  smoke: { buoy: 0.55, cool: 0.012, fade: 0.996, eps: 1.4, smoke: [0.78, 0.78, 0.8], absorb: 7, emit: 0 },
  explosion: { buoy: 0.9, cool: 0.011, fade: 0.996, eps: 1.6, smoke: [0.32, 0.29, 0.27], absorb: 6, emit: 1.6 },
  dust: { buoy: 0.15, cool: 0.02, fade: 0.994, eps: 1, smoke: [0.62, 0.52, 0.4], absorb: 5, emit: 0 },
};
function simulate(kind: keyof typeof PARAMS, seed: number): Frame[] {
  const rng = new Random(seed);
  let spare: number | undefined;
  const normal = () => {
    if (spare !== undefined) {
      const n = spare;
      spare = undefined;
      return n;
    }
    const r = Math.sqrt(-2 * Math.log(1 - rng.random())),
      a = 2 * Math.PI * rng.random();
    spare = r * Math.sin(a);
    return r * Math.cos(a);
  };
  const noiseField = (sigma: number) => {
    const f = gaussian(Float64Array.from({ length: SIZE }, normal), sigma);
    let max = 0;
    for (const v of f) max = Math.max(max, Math.abs(v));
    return { f, max };
  };
  let u = field(),
    v = field(),
    w = field(),
    d = field(),
    t = field();
  const { f: noise, max } = noiseField(2);
  for (let i = 0; i < SIZE; i++) noise[i]! /= max;
  const p = PARAMS[kind];
  if (!p) throw new Error(`unknown effect ${kind}`);
  const sponge = Float64Array.from({ length: SIZE }, (_, i) => {
    const x = i % N,
      y = Math.floor(i / N) % N,
      z = Math.floor(i / NN);
    return clamp(Math.min(x, N - 1 - x, z, N - 1 - z, N - 1 - y) / 8) ** 0.25;
  });
  const frames: Frame[] = [];
  let turb: Field[] = [];
  for (let s = 0; s < FRAMES * 2; s++) {
    const dt = 0.9;
    for (let z = 0, i = 0; z < N; z++)
      for (let y = 0; y < N; y++)
        for (let x = 0; x < N; x++, i++) {
          const r = Math.hypot(x - N / 2, z - N / 2),
            n = noise[i]!;
          if (kind === "fire") {
            const jitter = 1 + 0.4 * Math.sin(s * 0.37 + noise[z * NN + 6 * N + x]! * 6);
            if (r < 10 * jitter + 2.5 * n && y > 3 && y < 8) {
              t[i] = Math.max(t[i]!, 0.95);
              d[i] = Math.max(d[i]!, 0.25);
              v[i] = Math.max(v[i]!, 0.5);
            }
          } else if (kind === "smoke" && s < FRAMES) {
            if (Math.sqrt((x - N / 2) ** 2 + (y - 8) ** 2 + (z - N / 2) ** 2) < 5.5 + 2.5 * n) {
              d[i] = Math.max(d[i]!, 0.9 + 0.1 * n);
              t[i] = Math.max(t[i]!, 0.5);
              v[i] = Math.max(v[i]!, 0.45);
            }
          } else if (kind === "explosion" && s === 0) {
            const radius = Math.sqrt((x - N / 2) ** 2 + (y - 24) ** 2 + (z - N / 2) ** 2);
            if (radius < 7 + 2.5 * n) {
              d[i] = 1;
              t[i] = 1;
              const k = 0.9 / (radius + 1);
              u[i]! += (x - N / 2) * k;
              v[i]! += (y - 24) * k;
              w[i]! += (z - N / 2) * k;
            }
          } else if (kind === "dust" && s === 0 && r < 9 && y < 8) {
            d[i] = 0.9;
            const k = 2.2 / (r + 1);
            u[i]! += (x - N / 2) * k;
            w[i]! += (z - N / 2) * k;
            v[i]! += 0.4;
          }
          v[i]! += dt * p.buoy * (t[i]! - 0.05 * d[i]!) * 0.45;
        }
    if (s % 4 === 0)
      turb = Array.from({ length: 3 }, () => {
        const { f, max } = noiseField(3);
        for (let i = 0; i < SIZE; i++) f[i]! /= max + 1e-6;
        return f;
      });
    for (let i = 0; i < SIZE; i++) {
      const active = d[i]! > 0.02 ? 1 : 0;
      u[i] = (u[i]! + dt * 0.35 * turb[0]![i]! * active) * 0.995;
      v[i] = (v[i]! + dt * 0.2 * turb[1]![i]! * active) * 0.995;
      w[i] = (w[i]! + dt * 0.35 * turb[2]![i]! * active) * 0.995;
    }
    vorticity(u, v, w, p.eps, dt);
    project(u, v, w);
    [u, v, w] = [advect(u, u, v, w, dt), advect(v, u, v, w, dt), advect(w, u, v, w, dt)];
    project(u, v, w);
    d = advect(d, u, v, w, dt);
    for (let i = 0; i < SIZE; i++) {
      d[i]! *= p.fade * sponge[i]!;
      t[i]! *= sponge[i]!;
      v[i]! *= sponge[i]!;
    }
    const advected = advect(t, u, v, w, dt);
    for (let i = 0; i < SIZE; i++) advected[i] = Math.max(advected[i]! - p.cool * dt * (0.3 + t[i]!), 0);
    t = advected;
    if ((s + 1) % 2 === 0) frames.push(render(d, t, p.smoke, p.absorb, p.emit));
  }
  return frames;
}
function resize(src: Float64Array, channels: number): Float32Array {
  // Pillow's float bicubic resize, normalized over the in-bounds support.
  const cubic = (x: number) => {
    x = Math.abs(x);
    return x < 1 ? (1.5 * x - 2.5) * x * x + 1 : x < 2 ? ((-0.5 * x + 2.5) * x - 4) * x + 2 : 0;
  };
  const pass = (input: Float32Array, width: number, height: number, horizontal: boolean): Float32Array => {
    const outW = horizontal ? CELL : width,
      outH = horizontal ? height : CELL,
      out = new Float32Array(outW * outH * channels);
    for (let y = 0; y < outH; y++)
      for (let x = 0; x < outW; x++) {
        const center = (((horizontal ? x : y) + 0.5) * N) / CELL,
          first = Math.max(0, Math.floor(center - 2 + 0.5)),
          last = Math.min(N, Math.floor(center + 2 + 0.5));
        let sum = 0;
        const values = Array(channels).fill(0);
        for (let k = first; k < last; k++) {
          const weight = cubic(k - center + 0.5);
          sum += weight;
          const index = horizontal ? y * width + k : k * width + x;
          for (let c = 0; c < channels; c++) values[c] += input[index * channels + c]! * weight;
        }
        for (let c = 0; c < channels; c++) out[(y * outW + x) * channels + c] = values[c] / sum;
      }
    return out;
  };
  return pass(pass(Float32Array.from(src), N, N, true), CELL, N, false);
}
const tone = (x: number) => clamp((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14));
function sheet(frames: Frame[], stylised: boolean): Uint8Array {
  const rgba = new Uint8Array((GRID * CELL) ** 2 * 4);
  frames.forEach((f, i) => {
    const col = resize(f.rgb, 3),
      gl = resize(f.glow, 3),
      al = resize(f.alpha, 1);
    for (let y = 0; y < CELL; y++)
      for (let x = 0; x < CELL; x++) {
        const at = y * CELL + x,
          a = clamp(al[at]!),
          lum = Math.max(gl[at * 3]!, gl[at * 3 + 1]!, gl[at * 3 + 2]!);
        let alpha = clamp(Math.max(a, clamp(lum)));
        let color = [0, 1, 2].map((c) => tone((col[at * 3 + c]! * a + gl[at * 3 + c]!) / Math.max(alpha, 1e-4)));
        if (stylised) {
          const l = color.reduce((a, b) => a + b, 0) / 3,
            band = l > 0.62 ? 1 : l > 0.3 ? 0.62 : 0.32;
          color = color.map((v) => clamp((v / Math.max(l, 0.001)) * band));
          alpha = (alpha > 0.35 ? 1 : 0) * clamp((alpha - 0.25) * 6);
        }
        alpha *= clamp(Math.min(x / (CELL - 1), 1 - x / (CELL - 1), y / (CELL - 1), 1 - y / (CELL - 1)) * 14);
        const dest = ((Math.floor(i / GRID) * CELL + y) * GRID * CELL + (i % GRID) * CELL + x) * 4;
        rgba.set(
          [...color, alpha].map((v) => Math.trunc(clamp(v) * 255 + 0.5)),
          dest,
        );
      }
  });
  return rgba;
}
const out = process.argv[2];
if (!out) throw new Error("expected <out-dir> [name ...]");
const names = process.argv.length > 3 ? process.argv.slice(3) : Object.keys(PARAMS);
mkdirSync(out, { recursive: true });
for (const [i, name] of names.entries()) {
  const frames = simulate(name as keyof typeof PARAMS, 1000 + i);
  for (const stylised of [false, true]) {
    const path = join(out, `${name}${stylised ? "_toon" : ""}_8x8.png`);
    writeFileSync(path, encodePngBest({ width: GRID * CELL, height: GRID * CELL, rgba: sheet(frames, stylised) }));
    console.log(path);
  }
}
