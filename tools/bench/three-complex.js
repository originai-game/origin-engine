// tools/bench/complex (the complex scene, the judge of docs/design.md §7's
// 1.5x budget) in three.js 0.180, the same content with three's stock parts.
// Everything is placed from the project's own src/layout.js and loaded from
// the same assets/ (tools/bench/complex/tools/make-assets.ts):
//
// - the ground (ground.glb, castShadow off and receiveShadow on, as the
//   engine loads it) and the lake's blended water plane;
// - 5,261 props from 19 glTF models: every mesh of every model is an
//   InstancedMesh, one per (mesh, material) and 40 m tile of the ground by
//   default, so three's frustum culling drops whole tiles (the engine culls
//   per instance; one InstancedMesh per part over the whole level with
//   `?tiles=0`); every prop casts and receives shadows; the 24 ghost carts'
//   matrices are rewritten every frame;
// - 48 SkinnedMeshes (SkeletonUtils.clone of walker.glb and villager.glb),
//   an AnimationMixer each on its clip, speed and phase;
// - the sun through CSM (three cascades of 2048 over 120 m, as the
//   engine's), two shadowed SpotLights and two shadowed PointLights (512
//   maps, cube shadows, as the engine's local lights);
// - the 20 emitters of fx/fire.ogt and fx/smoke.ogt simulated on the CPU
//   from the same table (src/layout.js FX) and drawn as one instanced
//   billboard draw per blend mode;
// - textures: the engine's own cooked KTX2 files (cooked/*.ogb), through
//   KTX2Loader, so both sides sample the same block-compressed texels
//   (`?ktx2=0`: GLTFLoader's PNGs);
// - the sky (Sky, prefiltered by PMREM) as background and IBL;
// - post: GTAO, UnrealBloom, ACES (OutputPass) and SMAA on WebGL2; on
//   `?webgpu=1` (WebGPURenderer and its node post chain) GTAO, bloom, ACES
//   and TRAA, three's own temporal antialiasing, as the engine's full tier
//   has TAA.
import * as THREE from "three";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";
import * as SkeletonUtils from "three/addons/utils/SkeletonUtils.js";
import { Sky } from "three/addons/objects/Sky.js";
import * as L from "./complex/src/layout.js";
import { bench, fail } from "./three-bench.js";

const W = 1280, H = 720, DT = 1 / 60;
const ASSETS = "/three/complex/assets/";
const COOKED = "/three/complex/cooked/";
const TILE = 40;

try {
  const params = new URLSearchParams(location.search);
  const webgpu = params.get("webgpu") === "1";
  const tiles = params.get("tiles") !== "0";
  const ktx2 = params.get("ktx2") !== "0";
  const canvas = document.getElementById("c");

  let renderer, csm = null, WG = null;
  if (webgpu) {
    WG = await import("three/webgpu");
    renderer = new WG.WebGPURenderer({ canvas, antialias: false });
    await renderer.init();
  } else {
    renderer = new THREE.WebGLRenderer({ canvas, antialias: false, powerPreference: "high-performance" });
    renderer.shadowMap.type = THREE.PCFSoftShadowMap;
  }
  renderer.setPixelRatio(1);
  renderer.setSize(W, H, false);
  renderer.shadowMap.enabled = true;
  renderer.toneMapping = THREE.ACESFilmicToneMapping;
  const scene = new THREE.Scene();
  const view = L.cameraAt(0);
  const camera = new THREE.PerspectiveCamera(view.fov, W / H, view.near, view.far);
  const aim = (t) => {
    const v = L.cameraAt(t);
    camera.position.set(v.eye.x, v.eye.y, v.eye.z);
    camera.lookAt(v.at.x, v.at.y, v.at.z);
  };
  aim(0);

  // Sky and IBL: Sky rendered once into a PMREM cube, as three-reference.js.
  const sunDir = new THREE.Vector3(L.SUN.x, L.SUN.y, L.SUN.z).normalize();
  {
    const sky = new Sky();
    sky.scale.setScalar(1000);
    const u = sky.material.uniforms;
    if (u) {
      u.turbidity.value = 4;
      u.rayleigh.value = 1.5;
      u.sunPosition.value.copy(sunDir).negate();
    }
    const skyScene = new THREE.Scene();
    skyScene.add(sky);
    if (webgpu) {
      // WebGPURenderer's PMREM: Sky's ShaderMaterial does not run there, so the sky is the
      // node version three ships for it (SkyMesh), prefiltered the same way.
      const { SkyMesh } = await import("three/addons/objects/SkyMesh.js");
      const node = new SkyMesh();
      node.scale.setScalar(1000);
      node.turbidity.value = 4;
      node.rayleigh.value = 1.5;
      node.sunPosition.value.copy(sunDir).negate();
      skyScene.clear();
      skyScene.add(node);
      const pmrem = new WG.PMREMGenerator(renderer);
      const env = pmrem.fromScene(skyScene, 0, 0.1, 2000).texture;
      scene.environment = env;
      scene.background = env;
    } else {
      const pmrem = new THREE.PMREMGenerator(renderer);
      const env = pmrem.fromScene(skyScene, 0, 0.1, 2000).texture;
      scene.environment = env;
      scene.background = env;
    }
  }

  // The sun: three cascades of 2048 over the first 120 m.
  const materials = new Set();
  if (webgpu) {
    const { CSMShadowNode } = await import("three/addons/csm/CSMShadowNode.js");
    const sun = new THREE.DirectionalLight(0xffffff, 2.6);
    sun.position.copy(sunDir).multiplyScalar(-100);
    sun.castShadow = true;
    sun.shadow.mapSize.set(2048, 2048);
    sun.shadow.camera.near = 1;
    sun.shadow.camera.far = 400;
    sun.shadow.bias = -0.0003;
    sun.shadow.shadowNode = new CSMShadowNode(sun, { cascades: 3, maxFar: L.SHADOW_DISTANCE, mode: "practical", lightMargin: 60 });
    scene.add(sun, sun.target);
  } else {
    const { CSM } = await import("three/addons/csm/CSM.js");
    csm = new CSM({ maxFar: L.SHADOW_DISTANCE, cascades: 3, mode: "practical", parent: scene, shadowMapSize: 2048, lightDirection: sunDir, camera, lightIntensity: 2.6, lightMargin: 60, shadowBias: -0.0003 });
  }
  const shade = (m) => {
    if (materials.has(m)) return;
    materials.add(m);
    if (csm) csm.setupMaterial(m);
  };

  // Textures: the engine's own cooked KTX2 (ETC1S colour, UASTC normals and linear data, from
  // og cook's cooked/*.ogb), transcoded by three's KTX2Loader to the formats the device samples, as
  // the engine transcodes them: both sides sample the same block-compressed texels. `?ktx2=0` keeps
  // GLTFLoader's PNGs (RGBA8).
  let ktx2Loader = null;
  if (ktx2) {
    const { KTX2Loader } = await import("three/addons/loaders/KTX2Loader.js");
    ktx2Loader = new KTX2Loader().setTranscoderPath("/three/node_modules/three/examples/jsm/libs/basis/").detectSupport(renderer);
  }
  /** The KTX2 file inside a cooked texture blob (crates/og-blob: a 64-byte header, 48-byte section entries, names). */
  const cooked = async (id) => {
    const buf = await (await fetch(COOKED + id + ".ogb")).arrayBuffer();
    const dv = new DataView(buf);
    const count = dv.getUint32(12, true), names = 64 + count * 48;
    for (let i = 0; i < count; i++) {
      const e = 64 + i * 48;
      const name = new TextDecoder().decode(new Uint8Array(buf, names + dv.getUint32(e, true), dv.getUint32(e + 4, true)));
      if (name !== "ktx2") continue;
      const url = URL.createObjectURL(new Blob([new Uint8Array(buf, Number(dv.getBigUint64(e + 8, true)), Number(dv.getBigUint64(e + 16, true)))]));
      try {
        return await ktx2Loader.loadAsync(url);
      } finally {
        URL.revokeObjectURL(url);
      }
    }
    throw new Error(`${id}: no ktx2 section`);
  };
  /**
   * Swaps a loaded glTF's textures for the cooked ones: og cook numbers a model's textures
   * (`<model>.tex<i>`) in first use over its materials, each material's slots in the order
   * base colour, metallic-roughness, normal, emissive, occlusion, one per (image, kind).
   */
  const useCooked = async (file, gltf) => {
    const json = gltf.parser.json;
    const slots = (m) => [[m.pbrMetallicRoughness?.baseColorTexture, "color"], [m.pbrMetallicRoughness?.metallicRoughnessTexture, "linear"], [m.normalTexture, "normal"], [m.emissiveTexture, "color"], [m.occlusionTexture, "linear"]];
    const uses = [];
    for (const m of json.materials ?? [])
      for (const [slot, kind] of slots(m)) {
        if (!slot) continue;
        const image = json.textures[slot.index].source;
        if (!uses.some((u) => u.image === image && u.kind === kind)) uses.push({ image, kind });
      }
    if (!uses.length) return;
    const tex = await Promise.all(uses.map((_, i) => cooked(`models/${file}.tex${i}`)));
    const of = (slot, kind, old) => {
      const t = tex[uses.findIndex((u) => u.image === json.textures[slot.index].source && u.kind === kind)];
      t.wrapS = old.wrapS;
      t.wrapT = old.wrapT;
      t.colorSpace = kind === "color" ? THREE.SRGBColorSpace : THREE.NoColorSpace;
      old.dispose();
      return t;
    };
    const done = new Set();
    gltf.scene.traverse((o) => {
      if (!o.isMesh || done.has(o.material)) return;
      done.add(o.material);
      const m = o.material, j = json.materials[gltf.parser.associations.get(m)?.materials];
      if (!j) return;
      const pbr = j.pbrMetallicRoughness ?? {};
      if (m.map && pbr.baseColorTexture) m.map = of(pbr.baseColorTexture, "color", m.map);
      if (m.metalnessMap && pbr.metallicRoughnessTexture) m.metalnessMap = m.roughnessMap = of(pbr.metallicRoughnessTexture, "linear", m.metalnessMap);
      if (m.normalMap && j.normalTexture) m.normalMap = of(j.normalTexture, "normal", m.normalMap);
      if (m.emissiveMap && j.emissiveTexture) m.emissiveMap = of(j.emissiveTexture, "color", m.emissiveMap);
      if (m.aoMap && j.occlusionTexture) m.aoMap = of(j.occlusionTexture, "linear", m.aoMap);
      m.needsUpdate = true;
    });
  };

  const loader = new GLTFLoader();
  const load = async (file) => {
    const gltf = await loader.loadAsync(ASSETS + "models/" + file);
    if (ktx2) await useCooked(file, gltf);
    return gltf;
  };

  // The ground and the water.
  {
    const g = await load("ground.glb");
    g.scene.traverse((o) => {
      if (o.isMesh) {
        o.castShadow = false;
        o.receiveShadow = true;
        shade(o.material);
      }
    });
    scene.add(g.scene);
    const water = new THREE.Mesh(
      new THREE.PlaneGeometry(L.LAKE.r * 2.4, L.LAKE.r * 2.4),
      new THREE.MeshStandardMaterial({ color: new THREE.Color(0.12, 0.3, 0.38), roughness: 0.08, metalness: 0, opacity: 0.6, transparent: true }),
    );
    water.rotation.x = -Math.PI / 2;
    water.position.set(L.LAKE.x, 0, L.LAKE.z);
    water.receiveShadow = true;
    shade(water.material);
    scene.add(water);
  }

  // The props: each model's meshes as InstancedMeshes, per 40 m tile unless ?tiles=0.
  const where = L.placements();
  const half = L.SIZE / 2;
  const tileOf = (p) => (tiles ? Math.floor((p.x + half) / TILE) * 64 + Math.floor((p.z + half) / TILE) : 0);
  const m4 = new THREE.Matrix4(), q = new THREE.Quaternion(), up = new THREE.Vector3(0, 1, 0), pv = new THREE.Vector3(), sv = new THREE.Vector3();
  const poseMatrix = (p, out) => out.compose(pv.set(p.x, p.y, p.z), q.setFromAxisAngle(up, p.yaw), sv.setScalar(p.scale));
  let drawsBuilt = 0;
  const ghosts = [];
  for (const m of L.MODELS) {
    const g = await load(m.file);
    g.scene.updateMatrixWorld(true);
    const parts = [];
    g.scene.traverse((o) => {
      if (o.isMesh) {
        parts.push({ geometry: o.geometry, material: o.material, local: o.matrixWorld.clone() });
        shade(o.material);
      }
    });
    const copies = where[m.file];
    if (m.moving) {
      for (const part of parts) {
        const inst = new THREE.InstancedMesh(part.geometry, part.material, copies.length);
        inst.castShadow = inst.receiveShadow = true;
        inst.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
        scene.add(inst);
        ghosts.push({ inst, local: part.local });
        drawsBuilt++;
      }
      continue;
    }
    const byTile = new Map();
    for (const p of copies) {
      const k = tileOf(p);
      if (!byTile.has(k)) byTile.set(k, []);
      byTile.get(k).push(p);
    }
    for (const list of byTile.values()) {
      for (const part of parts) {
        const inst = new THREE.InstancedMesh(part.geometry, part.material, list.length);
        list.forEach((p, i) => inst.setMatrixAt(i, poseMatrix(p, m4).multiply(part.local)));
        inst.castShadow = inst.receiveShadow = true;
        inst.computeBoundingSphere();
        inst.matrixAutoUpdate = false;
        scene.add(inst);
        drawsBuilt++;
      }
    }
  }
  const moveGhosts = (t) => {
    for (const { inst, local } of ghosts) {
      for (let i = 0; i < inst.count; i++) inst.setMatrixAt(i, poseMatrix(L.ghostPose(i, t), m4).multiply(local));
      inst.instanceMatrix.needsUpdate = true;
      inst.computeBoundingSphere();
    }
  };
  moveGhosts(0);

  // The characters.
  const mixers = [];
  const rigs = {};
  for (const c of L.characters()) {
    rigs[c.model] ??= await load(c.model);
    const gltf = rigs[c.model];
    const w = SkeletonUtils.clone(gltf.scene);
    w.position.set(c.x, c.y, c.z);
    w.rotation.y = c.yaw;
    w.traverse((o) => {
      if (o.isMesh) {
        o.castShadow = o.receiveShadow = true;
        shade(o.material);
      }
      // WebGPURenderer's SkinningNode is shared by the clones (they share the material) and writes every
      // clone's previous bone matrices once one of them has them (the velocity TRAA reads): each needs its own.
      if (webgpu && o.isSkinnedMesh) o.skeleton.previousBoneMatrices = new Float32Array(o.skeleton.boneMatrices);
    });
    scene.add(w);
    const mixer = new THREE.AnimationMixer(w);
    const clip = THREE.AnimationClip.findByName(gltf.animations, c.clip);
    if (!clip) throw new Error(`${c.model} has no clip ${c.clip}`);
    const action = mixer.clipAction(clip);
    action.timeScale = c.speed;
    action.time = c.time;
    action.play();
    mixers.push(mixer);
  }

  // The four shadowed local lights.
  for (const l of L.lights()) {
    const color = new THREE.Color(l.color[0], l.color[1], l.color[2]);
    const light = l.kind === "spot" ? new THREE.SpotLight(color, l.intensity, l.distance, l.angle, l.penumbra, 2) : new THREE.PointLight(color, l.intensity, l.distance, 2);
    light.position.set(l.x, l.y, l.z);
    light.castShadow = true;
    light.shadow.mapSize.set(512, 512);
    scene.add(light);
    if (l.kind === "spot") {
      light.target.position.set(l.tx, l.ty, l.tz);
      scene.add(light.target);
    }
  }

  // The particles: fx/fire.ogt and fx/smoke.ogt, simulated per emitter on the CPU, one instanced draw per blend mode.
  const texLoader = new THREE.TextureLoader();
  const particles = [];
  for (const [name, f] of Object.entries(L.FX)) {
    const ems = L.emitters().filter((e) => e.fx === name);
    const cap = f.max * ems.length;
    const geo = new THREE.InstancedBufferGeometry();
    geo.setAttribute("position", new THREE.Float32BufferAttribute([-0.5, -0.5, 0, 0.5, -0.5, 0, 0.5, 0.5, 0, -0.5, 0.5, 0], 3));
    geo.setAttribute("uv", new THREE.Float32BufferAttribute([0, 0, 1, 0, 1, 1, 0, 1], 2));
    geo.setAttribute("normal", new THREE.Float32BufferAttribute([0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1], 3));
    geo.setIndex([0, 1, 2, 0, 2, 3]);
    const offset = new THREE.InstancedBufferAttribute(new Float32Array(cap * 3), 3).setUsage(THREE.DynamicDrawUsage);
    const size = new THREE.InstancedBufferAttribute(new Float32Array(cap), 1).setUsage(THREE.DynamicDrawUsage);
    const tint = new THREE.InstancedBufferAttribute(new Float32Array(cap * 4), 4).setUsage(THREE.DynamicDrawUsage);
    geo.setAttribute("offset", offset);
    geo.setAttribute("size", size);
    geo.setAttribute("tint", tint);
    geo.instanceCount = 0;
    const map = ktx2 ? await cooked(f.texture) : await texLoader.loadAsync(ASSETS + f.texture);
    map.colorSpace = THREE.SRGBColorSpace;
    const blending = f.blend === "additive" ? THREE.AdditiveBlending : THREE.NormalBlending;
    let material;
    if (webgpu) {
      const T = await import("three/tsl");
      material = new WG.MeshBasicNodeMaterial({ transparent: true, depthWrite: false, blending });
      const o = T.attribute("offset", "vec3"), s = T.attribute("size", "float"), c = T.attribute("tint", "vec4");
      material.vertexNode = T.cameraProjectionMatrix.mul(T.cameraViewMatrix.mul(T.vec4(o, 1)).add(T.vec4(T.positionGeometry.xy.mul(s), 0, 0)));
      const tex = T.texture(map, T.uv());
      material.colorNode = T.vec4(tex.rgb.mul(c.rgb), tex.a.mul(c.a));
    } else {
      material = new THREE.ShaderMaterial({
        uniforms: { map: { value: map } },
        vertexShader: `attribute vec3 offset; attribute float size; attribute vec4 tint; varying vec2 vUv; varying vec4 vTint;
          void main() { vec4 mv = viewMatrix * vec4(offset, 1.0); mv.xy += position.xy * size; gl_Position = projectionMatrix * mv; vUv = uv; vTint = tint; }`,
        fragmentShader: `uniform sampler2D map; varying vec2 vUv; varying vec4 vTint;
          void main() { vec4 t = texture2D(map, vUv); gl_FragColor = vec4(t.rgb * vTint.rgb, t.a * vTint.a); }`,
        transparent: true,
        depthWrite: false,
        blending,
      });
    }
    const mesh = new THREE.Mesh(geo, material);
    mesh.frustumCulled = false;
    scene.add(mesh);
    const state = ems.map((e) => ({ e, r: L.rng(e.seed), acc: 0, live: [] }));
    particles.push({ f, geo, offset, size, tint, state });
  }
  const colourAt = (keys, t, out) => {
    let k = 0;
    while (k < keys.length - 2 && t > keys[k + 1][0]) k++;
    const a = keys[k], b = keys[k + 1], u = Math.min(1, Math.max(0, (t - a[0]) / (b[0] - a[0])));
    for (let i = 0; i < 4; i++) out[i] = a[i + 1] + (b[i + 1] - a[i + 1]) * u;
    return out;
  };
  const rgba = [0, 0, 0, 0];
  const stepParticles = (dt) => {
    for (const P of particles) {
      const f = P.f;
      let n = 0;
      for (const s of P.state) {
        s.acc += f.rate * dt;
        while (s.acc >= 1) {
          s.acc -= 1;
          if (s.live.length >= f.max) continue;
          const r = s.r;
          const th = r() * Math.PI * 2, rad = Math.sqrt(r()) * f.radius;
          const cone = (f.cone * Math.PI) / 180, polar = Math.acos(1 - r() * (1 - Math.cos(cone))), az = r() * Math.PI * 2;
          const sp = f.speed[0] + r() * (f.speed[1] - f.speed[0]);
          s.live.push({
            x: s.e.x + Math.cos(th) * rad, y: s.e.y, z: s.e.z + Math.sin(th) * rad,
            vx: Math.sin(polar) * Math.cos(az) * sp, vy: Math.cos(polar) * sp, vz: Math.sin(polar) * Math.sin(az) * sp,
            age: 0, life: f.life[0] + r() * (f.life[1] - f.life[0]), size: f.size[0] + r() * (f.size[1] - f.size[0]),
          });
        }
        let w = 0;
        for (const p of s.live) {
          p.age += dt;
          if (p.age >= p.life) continue;
          p.vy += f.gravity * dt;
          p.x += p.vx * dt;
          p.y += p.vy * dt;
          p.z += p.vz * dt;
          s.live[w++] = p;
          const t = p.age / p.life;
          P.offset.array.set([p.x, p.y, p.z], n * 3);
          P.size.array[n] = p.size * (f.sizeOverLife[0] + (f.sizeOverLife[1] - f.sizeOverLife[0]) * t);
          P.tint.array.set(colourAt(f.color, t, rgba), n * 4);
          n++;
        }
        s.live.length = w;
      }
      P.geo.instanceCount = n;
      P.offset.needsUpdate = P.size.needsUpdate = P.tint.needsUpdate = true;
    }
  };

  // Post.
  let render;
  if (webgpu) {
    const T = await import("three/tsl");
    const { ao } = await import("three/addons/tsl/display/GTAONode.js");
    const { bloom } = await import("three/addons/tsl/display/BloomNode.js");
    const { traa } = await import("three/addons/tsl/display/TRAANode.js");
    const post = new WG.PostProcessing(renderer);
    const scenePass = T.pass(scene, camera);
    scenePass.setMRT(T.mrt({ output: T.output, normal: T.directionToColor(T.normalView), velocity: T.velocity }));
    const color = scenePass.getTextureNode("output");
    const depth = scenePass.getTextureNode("depth");
    const normalTex = scenePass.getTextureNode("normal");
    const normal = T.sample((uv) => T.colorToDirection(normalTex.sample(uv)));
    const velocity = scenePass.getTextureNode("velocity");
    const aoPass = ao(depth, normal, camera);
    const lit = T.vec4(color.rgb.mul(aoPass.getTextureNode().r), color.a);
    const withBloom = lit.add(bloom(lit, 0.25, 0.4, 0.9));
    post.outputNode = traa(withBloom, depth, velocity, camera);
    render = () => post.render();
  } else {
    const { EffectComposer } = await import("three/addons/postprocessing/EffectComposer.js");
    const { RenderPass } = await import("three/addons/postprocessing/RenderPass.js");
    const { GTAOPass } = await import("three/addons/postprocessing/GTAOPass.js");
    const { UnrealBloomPass } = await import("three/addons/postprocessing/UnrealBloomPass.js");
    const { SMAAPass } = await import("three/addons/postprocessing/SMAAPass.js");
    const { OutputPass } = await import("three/addons/postprocessing/OutputPass.js");
    const composer = new EffectComposer(renderer, new THREE.WebGLRenderTarget(W, H, { type: THREE.HalfFloatType }));
    composer.setPixelRatio(1);
    composer.setSize(W, H);
    composer.addPass(new RenderPass(scene, camera));
    composer.addPass(new GTAOPass(scene, camera, W, H));
    composer.addPass(new UnrealBloomPass(new THREE.Vector2(W, H), 0.25, 0.4, 0.9));
    composer.addPass(new SMAAPass());
    composer.addPass(new OutputPass());
    render = () => composer.render(DT);
  }

  let t = 0;
  const step = () => {
    t += DT;
    aim(t);
    moveGhosts(t);
    for (const m of mixers) m.update(DT);
    stepParticles(DT);
    if (csm) csm.update();
    render();
  };
  step();
  const live = () => particles.reduce((a, p) => a + p.geo.instanceCount, 0);
  globalThis.__bench_info__ = {
    renderer: webgpu ? "three WebGPURenderer + PostProcessing (GTAO, bloom, TRAA)" : "three WebGLRenderer + EffectComposer (GTAO, UnrealBloom, SMAA)",
    draws: `${drawsBuilt} InstancedMesh (${tiles ? `${TILE} m tiles` : "one per part"}), 48 SkinnedMesh, 2 particle draws`,
    textures: ktx2 ? "the engine's cooked KTX2 (KTX2Loader)" : "glTF PNGs (RGBA8)",
    props: L.MODELS.reduce((a, m) => a + m.count, 0),
    materials: materials.size,
    get particles() {
      return live();
    },
    version: THREE.REVISION,
  };
  bench(step);
} catch (e) {
  fail(e);
}
