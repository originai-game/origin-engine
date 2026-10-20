// tools/bench/spheres/src/main.ts in three.js 0.180: 20,000 spheres (16 x
// 12 segments, one geometry, one MeshStandardMaterial), each moved every
// frame on its own phase and speed, one directional light without shadows,
// no post, MSAA 4x, ACES. `?naive=1` is one Mesh per sphere (20,001 draws:
// how three.js games are usually written); otherwise one InstancedMesh with
// every matrix rewritten per frame (three's batching path, by hand).
// `?webgpu=1` uses WebGPURenderer instead of WebGLRenderer.
import * as THREE from "three";
import { bench, fail } from "./three-bench.js";

try {
  const params = new URLSearchParams(location.search);
  const naive = params.get("naive") === "1";
  const COUNT = 20000, COLUMNS = 200, SPACING = 1.2;
  const canvas = document.getElementById("c");
  let renderer;
  if (params.get("webgpu") === "1") {
    const { WebGPURenderer } = await import("three/webgpu");
    renderer = new WebGPURenderer({ canvas, antialias: true });
    await renderer.init();
  } else renderer = new THREE.WebGLRenderer({ canvas, antialias: true, powerPreference: "high-performance" });
  renderer.setPixelRatio(1);
  renderer.setSize(1280, 720, false);
  renderer.toneMapping = THREE.ACESFilmicToneMapping;
  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0.08, 0.09, 0.11);
  scene.add(new THREE.HemisphereLight(0xbfd4ff, 0x3a3020, 0.6));
  const sun = new THREE.DirectionalLight(0xffffff, 3);
  sun.position.set(0.45, 0.8, 0.35);
  scene.add(sun);
  const camera = new THREE.PerspectiveCamera(50, 1280 / 720, 0.1, 400);
  camera.position.set(0, 70, 95);
  camera.lookAt(0, 0, 0);
  const geometry = new THREE.SphereGeometry(0.5, 16, 12);
  const material = new THREE.MeshStandardMaterial({ color: new THREE.Color(0.85, 0.35, 0.2), roughness: 0.45, metalness: 0.1 });
  const rows = COUNT / COLUMNS;
  const xs = new Float32Array(COUNT), zs = new Float32Array(COUNT), phase = new Float32Array(COUNT), speed = new Float32Array(COUNT);
  for (let i = 0; i < COUNT; i++) {
    xs[i] = ((i % COLUMNS) - (COLUMNS - 1) / 2) * SPACING;
    zs[i] = (Math.floor(i / COLUMNS) - (rows - 1) / 2) * SPACING;
    phase[i] = (i * 0.37) % (Math.PI * 2);
    speed[i] = 1 + (i % 7) * 0.25;
  }
  let step;
  if (naive) {
    const meshes = [];
    for (let i = 0; i < COUNT; i++) {
      const m = new THREE.Mesh(geometry, material);
      m.position.set(xs[i], 0, zs[i]);
      scene.add(m);
      meshes.push(m);
    }
    step = (t) => {
      for (let i = 0; i < COUNT; i++) meshes[i].position.y = Math.sin(t * speed[i] + phase[i]) * 0.5;
      renderer.render(scene, camera);
    };
  } else {
    const inst = new THREE.InstancedMesh(geometry, material, COUNT);
    inst.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    inst.frustumCulled = false;
    scene.add(inst);
    const e = inst.instanceMatrix.array;
    for (let i = 0; i < COUNT; i++) {
      e[i * 16] = 1; e[i * 16 + 5] = 1; e[i * 16 + 10] = 1; e[i * 16 + 15] = 1;
      e[i * 16 + 12] = xs[i]; e[i * 16 + 14] = zs[i];
    }
    step = (t) => {
      for (let i = 0; i < COUNT; i++) e[i * 16 + 13] = Math.sin(t * speed[i] + phase[i]) * 0.5;
      inst.instanceMatrix.needsUpdate = true;
      renderer.render(scene, camera);
    };
  }
  step(0);
  globalThis.__bench_info__ = { renderer: renderer.isWebGPURenderer ? "three WebGPURenderer" : "three WebGLRenderer", draws: naive ? COUNT : 1, version: THREE.REVISION };
  bench(step);
} catch (e) {
  fail(e);
}
