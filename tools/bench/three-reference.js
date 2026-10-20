// crates/og-tests/reference-scene (docs/design.md §7's reference scene) in
// three.js 0.180, as close as three's stock parts allow:
// - 500 objects on a 25 x 20 grid 1.6 m apart, boxes (0.8 m) and spheres
//   (r 0.45, 32 x 16) alternating, in four MeshStandardMaterials, every one
//   casting and receiving shadows and turning about +Y at its own rate;
//   `?naive=1` is one Mesh each (500 draws), otherwise four InstancedMeshes
//   (one per geometry and material, every matrix rewritten per frame);
// - 20 skinned walkers (walker.glb, SkeletonUtils.clone, an AnimationMixer
//   each on walk, run, idle or wave at its own speed and phase);
// - the sun (shadowed, 2048² PCF soft, a 60 m box where the engine has
//   cascades) and a shadowed spot light;
// - a procedural sky (Sky) prefiltered by PMREM as background and IBL;
// - post: GTAO, UnrealBloom, ACES (OutputPass); SMAA where the engine has
//   TAA (three's TAARenderPass re-renders the scene per sample on a moving
//   scene, which would not be the same work), so three's antialiasing is
//   the cheaper of the two.
import * as THREE from "three";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";
import * as SkeletonUtils from "three/addons/utils/SkeletonUtils.js";
import { Sky } from "three/addons/objects/Sky.js";
import { EffectComposer } from "three/addons/postprocessing/EffectComposer.js";
import { RenderPass } from "three/addons/postprocessing/RenderPass.js";
import { GTAOPass } from "three/addons/postprocessing/GTAOPass.js";
import { UnrealBloomPass } from "three/addons/postprocessing/UnrealBloomPass.js";
import { SMAAPass } from "three/addons/postprocessing/SMAAPass.js";
import { OutputPass } from "three/addons/postprocessing/OutputPass.js";
import { bench, fail } from "./three-bench.js";

try {
  const params = new URLSearchParams(location.search);
  const naive = params.get("naive") === "1";
  const W = 1280, H = 720;
  const canvas = document.getElementById("c");
  const renderer = new THREE.WebGLRenderer({ canvas, antialias: false, powerPreference: "high-performance" });
  renderer.setPixelRatio(1);
  renderer.setSize(W, H, false);
  renderer.shadowMap.enabled = true;
  renderer.shadowMap.type = THREE.PCFSoftShadowMap;
  renderer.toneMapping = THREE.ACESFilmicToneMapping;
  const scene = new THREE.Scene();

  const camera = new THREE.PerspectiveCamera(50, W / H, 0.1, 200);
  camera.position.set(0, 15, 27);
  camera.lookAt(0, 0, -1);

  // Sky and IBL: Sky rendered once into a PMREM cube.
  {
    const sky = new Sky();
    sky.scale.setScalar(1000);
    const u = sky.material.uniforms;
    u.turbidity.value = 4;
    u.rayleigh.value = 1.5;
    u.sunPosition.value.set(0.45, 0.75, 0.5);
    const skyScene = new THREE.Scene();
    skyScene.add(sky);
    const pmrem = new THREE.PMREMGenerator(renderer);
    const env = pmrem.fromScene(skyScene, 0, 0.1, 2000).texture;
    scene.environment = env;
    scene.background = env;
  }

  const floor = new THREE.Mesh(new THREE.PlaneGeometry(80, 80), new THREE.MeshStandardMaterial({ color: new THREE.Color(0.32, 0.33, 0.3), roughness: 0.92 }));
  floor.rotation.x = -Math.PI / 2;
  floor.receiveShadow = true;
  scene.add(floor);

  const box = new THREE.BoxGeometry(0.8, 0.8, 0.8);
  const sphere = new THREE.SphereGeometry(0.45, 32, 16);
  const materials = [
    new THREE.MeshStandardMaterial({ color: new THREE.Color(1, 0.77, 0.34), metalness: 1, roughness: 0.3 }),
    new THREE.MeshStandardMaterial({ color: new THREE.Color(0.8, 0.12, 0.1), metalness: 0, roughness: 0.45 }),
    new THREE.MeshStandardMaterial({ color: new THREE.Color(0.55, 0.56, 0.58), metalness: 0, roughness: 0.85 }),
    new THREE.MeshStandardMaterial({ color: new THREE.Color(0.95, 0.95, 0.97), metalness: 1, roughness: 0.08 }),
  ];
  const COLUMNS = 25, ROWS = 20, SPACING = 1.6, N = COLUMNS * ROWS;
  const pos = [], speed = [], angle = new Float32Array(N);
  for (let r = 0; r < ROWS; r++) {
    for (let c = 0; c < COLUMNS; c++) {
      const i = r * COLUMNS + c;
      pos.push(new THREE.Vector3((c - (COLUMNS - 1) / 2) * SPACING, 0.45, -(r * SPACING) + 6));
      speed.push(0.4 + (i % 7) * 0.15);
    }
  }
  let turn;
  if (naive) {
    const meshes = [];
    for (let i = 0; i < N; i++) {
      const m = new THREE.Mesh(i % 2 === 0 ? box : sphere, materials[i % 4]);
      m.position.copy(pos[i]);
      m.castShadow = m.receiveShadow = true;
      scene.add(m);
      meshes.push(m);
    }
    turn = (dt) => {
      for (let i = 0; i < N; i++) meshes[i].rotation.y += speed[i] * dt;
    };
  } else {
    // One InstancedMesh per (geometry, material): i % 4 picks both.
    const groups = [0, 1, 2, 3].map((k) => {
      const ids = [];
      for (let i = k; i < N; i += 4) ids.push(i);
      const inst = new THREE.InstancedMesh(k % 2 === 0 ? box : sphere, materials[k], ids.length);
      inst.castShadow = inst.receiveShadow = true;
      inst.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
      scene.add(inst);
      return { inst, ids };
    });
    const m = new THREE.Matrix4(), q = new THREE.Quaternion(), one = new THREE.Vector3(1, 1, 1), up = new THREE.Vector3(0, 1, 0);
    turn = (dt) => {
      for (const { inst, ids } of groups) {
        for (let j = 0; j < ids.length; j++) {
          const i = ids[j];
          angle[i] += speed[i] * dt;
          inst.setMatrixAt(j, m.compose(pos[i], q.setFromAxisAngle(up, angle[i]), one));
        }
        inst.instanceMatrix.needsUpdate = true;
        inst.computeBoundingSphere();
      }
    };
    turn(0);
  }

  // The walkers.
  const gltf = await new GLTFLoader().loadAsync("/scene/walker.glb");
  const clips = ["walk", "run", "idle", "wave"].map((n) => THREE.AnimationClip.findByName(gltf.animations, n));
  const mixers = [];
  for (let i = 0; i < 20; i++) {
    const a = (i / 20) * Math.PI * 2;
    const w = SkeletonUtils.clone(gltf.scene);
    w.position.set(Math.cos(a) * 9, 0, Math.sin(a) * 4 + 10);
    w.rotation.y = Math.atan2(-Math.sin(a), Math.cos(a));
    w.traverse((o) => {
      if (o.isMesh) o.castShadow = o.receiveShadow = true;
    });
    scene.add(w);
    const mixer = new THREE.AnimationMixer(w);
    const action = mixer.clipAction(clips[i % 4]);
    action.timeScale = 0.8 + (i % 5) * 0.1;
    action.time = (i * 0.137) % 1;
    action.play();
    mixers.push(mixer);
  }

  // Two shadowed lights.
  const sun = new THREE.DirectionalLight(0xffffff, 2.6);
  sun.position.set(0.45, 0.75, 0.5).multiplyScalar(40);
  sun.castShadow = true;
  sun.shadow.mapSize.set(2048, 2048);
  Object.assign(sun.shadow.camera, { left: -30, right: 30, top: 30, bottom: -30, near: 1, far: 100 });
  scene.add(sun, sun.target);
  const spot = new THREE.SpotLight(new THREE.Color(1, 0.85, 0.6), 120, 30, 0.6, 0.3, 2);
  spot.position.set(2, 9, -2);
  spot.target.position.set(0, 0, -6);
  spot.castShadow = true;
  spot.shadow.mapSize.set(1024, 1024);
  scene.add(spot, spot.target);

  const composer = new EffectComposer(renderer, new THREE.WebGLRenderTarget(W, H, { type: THREE.HalfFloatType }));
  composer.setPixelRatio(1);
  composer.setSize(W, H);
  composer.addPass(new RenderPass(scene, camera));
  composer.addPass(new GTAOPass(scene, camera, W, H));
  composer.addPass(new UnrealBloomPass(new THREE.Vector2(W, H), 0.25, 0.4, 0.9));
  composer.addPass(new SMAAPass());
  composer.addPass(new OutputPass());

  const DT = 1 / 60;
  const step = () => {
    turn(DT);
    for (const m of mixers) m.update(DT);
    composer.render(DT);
  };
  step();
  globalThis.__bench_info__ = { renderer: "three WebGLRenderer + EffectComposer", draws: naive ? "500 meshes" : "4 InstancedMesh", version: THREE.REVISION };
  bench(step);
} catch (e) {
  fail(e);
}
