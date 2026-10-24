# three.js to Origin Engine

For an author who knows three.js, or who has a three.js file to bring
over. Origin Engine keeps three.js's vocabulary wherever it does not
contradict the engine: `Vector3`, position, quaternion and scale, the
material parameters, the geometry classes' parameters and defaults, glTF
loading, layers. What it does not keep is the object model. A three.js
program is a tree of mutable objects that a loop you write renders; an
Origin Engine game is a world of entities with plain components, changed
by systems on a fixed tick, and the engine draws every frame itself. Read
og-game §3 (components, systems, the game) and og-core first: this page
only maps names.

`og migrate three <file.ts>` (§11) rewrites the mechanical part of a
file with these mappings and marks every line it cannot, citing the
section here that says what to do.

## 1. The model, side by side

| three.js | Origin Engine |
|---|---|
| `new THREE.Scene()` | the world: every entity spawned is in it, nothing is added |
| an `Object3D`, `Group` or `Mesh` | an entity (`world.spawn`), its data as components |
| `object.position`, `quaternion`, `scale` | the og.Transform component, read with `world.get` and written with `world.set` |
| `parent.add(child)` | og.Parent on the child (`SpawnDesc.parent` at spawn) |
| a `BufferGeometry`, a `Material`, a `Texture` | a handle the world owns (Geometry, Material, Texture), made by `og.render` or `og.load` |
| `new THREE.Mesh(geometry, material)` | og.Mesh `{ geometry, material }` on an entity |
| `renderer.render(scene, camera)` in `requestAnimationFrame` | nothing: the engine draws every frame; game logic runs in systems on the fixed tick (§8) |
| `camera` | the og.Camera resource (one per world), og.CameraView for more |
| `Raycaster` | `og.render.raycast` (§9) |
| `GLTFLoader`, `TextureLoader` | `og.load.gltf`, `og.load.texture` of cooked assets (§7) |
| a thrown error | a Result: `{ ok: true, value }` or `{ ok: false, reason }` |
| `Math.random()`, `clock.getDelta()` | og.Rng and og.Time: deterministic, so replays and tests repeat |

Three rules follow from the model and catch every port:

- **Values are plain.** og.Transform's position is `{ x, y, z }`, not a
  Vector3 object with methods. `vec3`, `quat`, `mat4` and `color` are
  three.js's methods as pure functions (`vec3.add(a, b)` returns a new
  value; `a.add(b)` does not exist), so a result can go straight into a
  component (`reference/math.md`).
- **Every call that can fail answers a Result.** The examples here use
  a `must` helper, as the templates do:
  `function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T { if (!r.ok) throw new Error(r.reason); return r.value; }`.
- **Calls take the world.** Setup goes in `GameDesc.start(world)` (or a
  command), per-tick logic in a system; neither keeps state in module
  variables (og-core).

## 2. The scene graph: entities and og.Parent

```ts
// three.js
const ship = new THREE.Group();
const hull = new THREE.Mesh(new THREE.BoxGeometry(2, 0.5, 4), steel);
ship.add(hull);
scene.add(ship);

// Origin Engine, in start(world)
const ship = must(world.spawn({ name: "ship" }));
const hull = must(world.spawn({ name: "hull", parent: ship }));
world.insert(hull, og.Mesh, { geometry: must(og.render.boxGeometry(world, { width: 2, height: 0.5, depth: 4 })), material: steel });
```

- `parent.add(child)` after the spawn is `world.insert(child, og.Parent,
  { entity: parent })`; `parent.remove(child)` is `world.remove(child,
  og.Parent)`; `scene.remove(object)` is `world.despawn(object)`.
- A child's og.Transform is relative to its parent, as three.js's
  `position` is; the engine composes og.MatrixWorld down the chain after
  the systems run, so there is no `updateMatrixWorld` to call.
- `getObjectByName("hull")` is `world.find("hull")`. Names are unique and
  the probe, the command channel and the tests address entities by them:
  name every entity anyone will ask about.
- `scene.traverse(fn)` over everything of a kind is a system's query over
  the components that kind has (og-core): `reads: [og.Mesh]` visits every
  drawn mesh, archetype by archetype.
- A scene written by hand in three.js code is often better as a scene
  file: `.ogt` text with nested `entity` blocks, cooked and diffable
  (og-game §4).

## 3. Object3D: og.Transform and the components

| three.js `Object3D` | Origin Engine |
|---|---|
| `position.set(x, y, z)` | `world.set(e, og.Transform, { position: { x, y, z } })` |
| `quaternion.set(x, y, z, w)` | `world.set(e, og.Transform, { quaternion: { x, y, z, w } })` |
| `rotation.set(x, y, z)` (an Euler) | `{ quaternion: quat.fromEuler({ x, y, z }) }`: radians, order `"XYZ"` as three.js's default |
| `scale.set(x, y, z)`, `scale.setScalar(s)` | `{ scale: { x, y, z } }` |
| `position.x += v` | read, change, write: `const t = must(world.get(e, og.Transform)); world.set(e, og.Transform, { position: { ...t.position, x: t.position.x + v } })`, or a system over the column (og-core) |
| `lookAt(target)` (a camera or a light) | `{ quaternion: og.transform.lookAt(eye, target) }`: it points -Z at the target, as three.js turns cameras and lights |
| `lookAt(target)` (any other object) | three.js points such an object's +Z at the target: `og.transform.lookAt(target, eye)`, the reverse, does the same |
| `matrixWorld` | og.MatrixWorld (read only; the engine writes it) |
| `visible`, `castShadow`, `receiveShadow`, `layers` | og.Mesh's (and og.Model's) `visible`, `castShadow`, `receiveShadow`, `layers` |
| `name` | `SpawnDesc.name` |
| `userData` | your own component (`defineComponent`): reflected, saved, inspectable |

Two defaults differ from three.js: `Mesh.receiveShadow` is on (three.js's
is off) and `DirectionalLight.castShadow` is on. Set them where the port
depends on the old value.

## 4. Geometry: og.render and og-geom

The geometry classes keep their parameters, names and defaults; the
positional arguments become a desc object of the same names, and the
result is a Geometry handle (a Result) that draws through og.Mesh, carries
a BVH for `og.render.raycast` and is a `Collider.mesh` at once
(og-game §18 has the details).

| three.js | Origin Engine | Not carried over |
|---|---|---|
| `BoxGeometry(width, height, depth)` | `og.render.boxGeometry(world, { width, height, depth })` | the segment counts |
| `PlaneGeometry(width, height)` | `og.render.planeGeometry` | the segment counts |
| `SphereGeometry(radius, widthSegments, heightSegments)` | `og.render.sphereGeometry` | `phiStart`, `phiLength`, `thetaStart`, `thetaLength` |
| `CylinderGeometry(radiusTop, radiusBottom, height, radialSegments, heightSegments, openEnded, thetaStart, thetaLength)` | `og.render.cylinderGeometry` | — |
| `ConeGeometry(radius, height, radialSegments, heightSegments, openEnded)` | `og.render.coneGeometry` | `thetaStart`, `thetaLength` |
| `CapsuleGeometry(radius, height, capSegments, radialSegments, heightSegments)` | `og.render.capsuleGeometry` (`height` the straight middle, as r174's) | — |
| `TorusGeometry(radius, tube, radialSegments, tubularSegments, arc)` | `og.render.torusGeometry` | — |
| `RingGeometry(innerRadius, outerRadius, thetaSegments, phiSegments, thetaStart, thetaLength)` | `og.render.ringGeometry` | — |
| `CircleGeometry(radius, segments, thetaStart, thetaLength)` | `og.render.circleGeometry` | — |
| `TubeGeometry(new CatmullRomCurve3(points), tubularSegments, radius, radialSegments, closed)` | `og.render.tubeGeometry(world, { points, ... })`, or `path` an og.path | other Curve classes: write the curve as an og.path |
| `LatheGeometry(points, segments, phiStart, phiLength)` | `og.render.latheGeometry` (points `{ x, y }`) | — |
| `Shape`, `Path` | `og.Shape`, `og.Path2` (the root `shapes` module, `reference/shapes.md`): the same calls | — |
| `ShapeGeometry(shapes, curveSegments)` | `og.render.shapeGeometry(world, { shapes, curveSegments })` | — |
| `ExtrudeGeometry(shapes, { depth, bevelEnabled, ... })` | `og.render.extrudeGeometry(world, { shapes, depth, bevelEnabled, ... })`; `extrudePath` an og.path | `UVGenerator` |
| `PolyhedronGeometry(vertices, indices, radius, detail)` | `og.render.polyhedronGeometry` | — |
| `IcosahedronGeometry`, `DodecahedronGeometry`, `OctahedronGeometry`, `TetrahedronGeometry` `(radius, detail)` | `og.render.icosahedronGeometry` and the other three | — |
| `BufferGeometry` with `setAttribute` and `setIndex` | `og.render.bufferGeometry(world, { positions, normals, uvs, colors, indices })` (typed arrays, normals computed when left out) | groups (one material per geometry: split it) |
| `attribute.needsUpdate = true` | `og.render.updateGeometry` of a geometry made with `dynamic: true` | — |
| `BufferGeometryUtils.mergeGeometries([...])` | `og.render.mergeGeometries(world, { parts: [{ geometry, position, quaternion, scale, mirror }] })` | — |
| three-bvh-csg's `Evaluator` | `og.render.csgGeometry(world, { op, a, b })` with exact predicates | — |
| `TorusKnotGeometry`, `TextGeometry`, `EdgesGeometry`, `WireframeGeometry` | none: a knot as `tubeGeometry` through its points, text as `og.render.text` or og.ui, edges as `wireframe: true` on the material or `og.render.lines` | — |

Two more go beyond three.js: `og.render.sweepGeometry` sweeps a section
along an og.path with a scale and twist over its length (horns, tusks,
handles), and a prop built of these parts belongs in an asset rather
than in code: an `.ogt` of `type model` names each part by its
`og.render` call with the desc's fields, places it as `mergeGeometries`
does and gives it a material and a pivot; the cook builds it into the
model blob a glTF cooks to, with levels of detail and the BVH, and
`og.load.gltf` loads it (og-game §18 has the grammar). The og-model
skill builds such props from a reference image.

`og.render.geometryInfo` answers what `geometry.attributes.position.count`
and `computeBoundingBox` did. Geometries are made where the world is
(start, a command, a system), never at module load.

## 5. Materials and textures

Every lit three.js mesh material becomes `og.render.standardMaterial(world,
desc)`, with the StandardMaterialDesc fields under three.js's names:
`color`, `metalness`, `roughness`, `emissive`, `emissiveIntensity`, `map`,
`normalMap`, `normalScale`, `roughnessMap`, `metalnessMap`, `aoMap`,
`emissiveMap`, `opacity`, `transparent`, `alphaTest`, `side`,
`flatShading`, `wireframe`, `envMapIntensity`, and MeshPhysicalMaterial's
`clearcoat`, `sheen`, `transmission`, `thickness`, `ior`, `iridescence`
and `anisotropy`.

| three.js | Origin Engine |
|---|---|
| `MeshStandardMaterial(params)`, `MeshPhysicalMaterial(params)` | `standardMaterial(world, params)` |
| `MeshBasicMaterial(params)` | `standardMaterial(world, { ...params, unlit: true })` |
| `MeshToonMaterial(params)` | `standardMaterial(world, { ...params, toon: true })` (`gradientMap` quantizes the light) |
| `MeshLambertMaterial`, `MeshPhongMaterial` | `standardMaterial`: roughness near 1 for Lambert; Phong's `shininess` and `specular` have no field (lower roughness for a tighter highlight) |
| `color: 0xff8800`, `new THREE.Color("#ff8800")` | `color: color.fromHex(0xff8800)`: sRGB hex to linear, as three.js's colour management does |
| `new THREE.Color(r, g, b)` | `{ r, g, b }`, linear, as three.js's components are |
| `side: THREE.DoubleSide` (`FrontSide`, `BackSide`) | `side: "double"` (`"front"`, `"back"`) |
| `material.roughness = 0.3` | `og.render.updateMaterial(world, material, { roughness: 0.3 })` |
| `vertexColors` on | the geometry's `colors` multiply the colour whenever it has them |
| `texture.repeat.set(4, 4)` with `RepeatWrapping` | the geometry's `uvRepeat: { x: 4, y: 4 }`; textures always wrap |
| `texture.colorSpace = THREE.SRGBColorSpace` | the cook's import settings (og-game §5): colour textures are sRGB, a `kind normal` texture is linear |
| `scene.environment = pmrem.fromEquirectangular(hdr)` | og.Environment's `map` from `og.load.environment` of a `type environment` asset (§7) |
| `LineBasicMaterial`, `PointsMaterial` | `og.render.lines` and `og.render.points` each tick, or the og.Lines and og.Points components over a geometry |
| `SpriteMaterial` with `THREE.Sprite` | og.Sprite, or `og.render.sprite` each tick |
| `ShaderMaterial`, `RawShaderMaterial`, `onBeforeCompile` | §10 |

Toon, rim, outline, dissolve and hit flash are fields of the same desc
(og-game §10). One geometry has one material: three.js's material arrays
and geometry groups become one entity per group.

## 6. Lights, the camera and the frame

| three.js | Origin Engine |
|---|---|
| `DirectionalLight(color, intensity)`, `position` and `target` | og.DirectionalLight `{ color, intensity }` on an entity: it shines along the entity's -Z, so aim it with `og.transform.lookAt(position, target)` (three.js's default target is the origin) |
| `PointLight(color, intensity, distance)` | og.PointLight `{ color, intensity, distance }`: `distance` is never infinite (10 by default); `decay` is not carried over |
| `SpotLight(color, intensity, distance, angle, penumbra)` | og.SpotLight, along the entity's -Z like the directional light |
| `RectAreaLight(color, intensity, width, height)` | og.RectLight |
| `AmbientLight`, `HemisphereLight` | no light entity: the sky lights every surface (RenderSettings.environmentIntensity), or an environment map does (og.Environment); og.Sky colours the sky itself |
| `light.castShadow = true`, `light.shadow.mapSize` | the light's `castShadow`; `RenderOptions.shadowMapSize` and RenderSettings.shadowMapScale |
| `renderer.shadowMap.enabled = true` | nothing: shadows are on per light |
| `PerspectiveCamera(fov, aspect, near, far)` | the og.Camera resource: `const camera = must(world.resource(og.Camera)); camera.fov = 50;` (degrees, vertical, as three.js); the aspect is the play surface's |
| `camera.aspect = w / h; camera.updateProjectionMatrix()` on resize | nothing: the engine sizes the canvas and the projection |
| `OrthographicCamera(left, right, top, bottom, near, far)` | og.Camera with `orthographic: true` and `height` the visible world height (`top - bottom`); the width follows the aspect |
| `camera.position.set(x, y, z)`, `camera.lookAt(x, y, z)` | `camera.position = { x, y, z }`, `camera.lookAt({ x, y, z })` on the resource handle |
| `scene.background = new THREE.Color(0x87ceeb)` | `camera.background = color.fromHex(0x87ceeb)` (og.Camera's `background`) |
| `scene.fog = new THREE.Fog(color, near, far)` | og.Fog's `color`, `start`, `end` (`density` for FogExp2), with `enabled` true (og-game §11) |
| `new THREE.WebGLRenderer({ antialias: true })` | `GameDesc.render` (`render: { antialias: "msaa4" }`); the engine owns the canvas, on an OffscreenCanvas in a Worker on the web |
| `renderer.toneMapping`, `toneMappingExposure` | RenderSettings.toneMapping (`"aces"` by default, `"agx"`, `"none"`) and `exposure` (og-game §16) |
| a second camera into a `WebGLRenderTarget` | og.CameraView with `target` from `og.render.renderTarget` (og-game §23) |
| `EffectComposer` and its passes | og.PostEffects (bloom, AO, depth of field, outline, LUTs and more, og-game §10) and custom post passes (§10 here) |

Light intensities are three.js's physical units in spirit (a point
light's `intensity` is its radiance at one metre) but not calibrated
against them: judge a ported scene's light in a frame, not by its
numbers.

## 7. Loaders: og.load and the cook

There is no loader object and nothing is fetched by URL. Sources go in
`assets/`, `og cook` turns them into `cooked/` (meshopt meshes with LODs
and BVHs, KTX2 textures, QOA audio, MSDF fonts), and every `og.load`
call names an asset id, the path under `assets/` (og-game §5, §6).

| three.js | Origin Engine |
|---|---|
| `new GLTFLoader().load(url, (gltf) => scene.add(gltf.scene))` | `const ship = must(og.load.gltf(world, "models/ship.glb"))`, then `world.instantiate(ship, { name: "ship", position })` once `og.load.status(world, ship)` is `"ready"`, or the `AssetReady` event; a scene's `model` line instantiates it for you |
| `gltf.animations` with an `AnimationMixer` | og.AnimationPlayer or og.AnimationGraph on the instance (og-game §9) |
| `DRACOLoader`, `KTX2Loader`, `MeshoptDecoder` | the cook compresses; give it the uncompressed glTF (it reads no Draco) |
| `new THREE.TextureLoader().load("textures/crate.png")` | `og.load.texture(world, "textures/crate.png")` |
| `RGBELoader`, `EXRLoader`, `CubeTextureLoader` for the environment | an `.ogt` of `type environment` naming the `.hdr`, `.exr` or cube, loaded with `og.load.environment` (prefiltered at cook time) |
| `AudioLoader` with `THREE.Audio` | `og.load.sound` and og.AudioSource (og-audio) |
| `FontLoader` | `og.load.font` of a TTF or OTF, for `og.render.text` and og.ui |
| `loader.loadAsync(url)` and `await` | no promises: a load completes at a tick boundary, and under `og run` before the next tick, so it lands on the same tick every run |

A MagicaVoxel `.vox` loads with `og.load.gltf` too, and so does a
`type model` asset (§4).

## 8. The render loop: the tick

```ts
// three.js
const clock = new THREE.Clock();
renderer.setAnimationLoop(() => {
  const dt = clock.getDelta();
  cube.rotation.y += dt;
  renderer.render(scene, camera);
});

// Origin Engine: a component marks what turns, a system turns it
const Spin = defineComponent("Spin", { speed: f32(1) });
const Turn = defineSystem("Turn", { reads: [Spin], writes: [og.Transform] }, (q, time) => {
  for (const a of q.archetypes()) {
    const rot = a.column(og.Transform).quaternion;
    const speed = a.column(Spin).speed;
    for (let i = 0; i < a.count; i++) {
      const next = quat.multiply(
        { x: rot[i * 4], y: rot[i * 4 + 1], z: rot[i * 4 + 2], w: rot[i * 4 + 3] },
        og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, speed[i] * time.dt),
      );
      rot.set([next.x, next.y, next.z, next.w], i * 4);
    }
  }
});
export default og.game({ systems: [Turn], start(world) { /* spawn the cube with Spin */ } });
```

- The loop body becomes one or more systems, each one job, listed in
  `GameDesc.systems`. A system runs once per tick (60 Hz by default,
  fixed) over the archetypes that hold its components; the engine
  renders between ticks and interpolates.
- `clock.getDelta()` is `time.dt` (the og.Time resource, fixed per
  tick); `clock.getElapsedTime()` is `time.elapsed`. Nothing in a tick
  reads the wall clock.
- `Math.random()` is og.Rng (`world.resource(og.Rng)`, `range`, `chance`,
  `integer`): seeded from the project, so a replay repeats the run.
- `renderer.render(scene, camera)`, `requestAnimationFrame` and the
  resize handler are deleted: the engine draws every frame.
- Input listeners (`addEventListener("keydown")`) become reads of
  og.Input in a system (`input.pressed("Space")`, og-game §7): the main
  thread owns the events and the tick sees them at its start.

## 9. Picking: og.render.raycast

```ts
// three.js
raycaster.setFromCamera(pointer, camera);
const hits = raycaster.intersectObjects(scene.children, true);

// Origin Engine, in a system that reads og.Input and og.Camera
const camera = must(world.resource(og.Camera));
const input = must(world.resource(og.Input));
const ray = camera.screenToRay(input.pointer.x, input.pointer.y);   // CSS pixels, not -1..1
const hits = must(og.render.raycast(world, ray, { first: true }));
if (hits.length > 0) og.debug.sphere(world, hits[0].point, 0.05);
```

- It tests what is drawn (every og.Mesh and og.Model at its world
  transform, through each geometry's BVH, both faces), nearest first,
  always over the whole world: there is no list of objects to pass. Each
  hit has `entity`, `geometry`, `distance`, `point`, `normal`, `uv` and
  `faceIndex`, as three.js's intersection.
- `near` and `far` are `maxDistance`; three.js's `layers` are the desc's
  `layers`, tested as three.js tests them; `first: true` stops at the
  nearest hit.
- For gameplay against colliders (ground checks, bullets) use
  `og.physics.raycast`, which reads the physics world instead
  (og-game §13).

## 10. The gaps, named

- **ShaderMaterial and RawShaderMaterial** have no one-line counterpart.
  A shader is an asset of `type shader` written in WGSL, compiled by the
  cook for every back end: four hooks into the lit shader (vertex offset,
  surface colour, normal, emissive; og-game §12) for what most
  `onBeforeCompile` and ShaderMaterial uses do, or `mode full lit` /
  `mode full unlit` for the whole vertex and fragment stages (og-game §23),
  made with `og.render.shaderMaterial` and fed with
  `og.render.setShaderParam`. A screen-space effect (an EffectComposer
  ShaderPass) is a `type post` asset in PostEffects.custom. GLSL is
  rewritten by hand into WGSL; uniforms become `param` lines.
- **InstancedMesh is not needed.** Draws are batched per geometry and
  material: spawn one entity per instance with the same Geometry and
  Material handles and the renderer draws them instanced, each culled and
  shadowed on its own. `setMatrixAt(i, m)` is that entity's og.Transform
  (`og.transform.decompose(m)`), `setColorAt` a per-entity colour through
  `Mesh.flash` or a texture-array layer (`Mesh.materialLayer`), or one
  material per colour. A part repeated inside one model is instanced by
  the cook (`type model`'s `at { }` blocks).
- **No OrbitControls object.** Nothing takes a DOM element or listens to
  events. The og.OrbitCamera component is the same controller as data:
  put it on the entity to orbit, list `...og.systems.cameras` first in
  `GameDesc.systems`, and its fields keep OrbitControls' names
  (`target`, `minDistance`, `maxDistance`, `minPolarAngle`,
  `maxPolarAngle`, `rotateSpeed`, `zoomSpeed`, `autoRotate`,
  `autoRotateSpeed`, `enabled`); `controls.update()` is gone. There is no
  damping and no panning. og.FollowCamera follows a target instead
  (og-feel §8 covers both).
- **No Euler objects.** A rotation is a quaternion; Euler angles go
  through `quat.fromEuler` once, where they are written.
- **No mutation helpers on values.** `vector.add(v)`, `clone()`,
  `multiplyScalar(s)` and the other in-place methods are the pure `vec3`
  and `quat` functions.
- **No dispose.** Geometries, materials and textures are handles the
  world owns; a streaming group frees its assets when it unloads
  (og-game §6). Drop the `dispose()` calls.
- **Not in the engine at all**: CSS2DRenderer and CSS3DRenderer (use
  og.ui, or `camera.worldToScreen` for a label's position), the
  `TransformControls` gizmos, `Line2` fat lines (`og.render.lines` has
  `width` already), and any direct WebGL call.

## 11. `og migrate three`

```sh
og migrate three src/scene.ts --out src/scene.og.ts
og check .
```

It reads one TypeScript file that imports three.js (`import * as THREE
from "three"`, named imports, and the `three/addons` and
`three/examples/jsm` paths) and writes it back with every mechanical
mapping above applied:

- the three.js imports replaced by one import of `og` (and `color` or
  `quat` when the rewrite uses them) from `@origingame/engine`, and a
  `must` helper added when the file has none;
- the geometry constructors of §4 to their `og.render` calls with the
  positional arguments named, `new THREE.Shape` and `Path` to `og.Shape`
  and `og.Path2`;
- the material constructors of §5 to `og.render.standardMaterial`, hex
  and `#rrggbb` colours through `color.fromHex`, the side constants to
  their strings;
- `Vector2`, `Vector3`, `Quaternion` and `Color` constructions to plain
  values; `MathUtils.degToRad` and `radToDeg` to arithmetic;
- `const x = new THREE.Mesh(g, m)`, `Group` and `Object3D` to a spawn
  (named after the variable) with og.Mesh; the directional, point, spot
  and rect area lights to a spawn with their component, a directional or spot light's `position.set`
  also aiming it at the origin as three.js's default target does;
- `add` to og.Parent, the scene's `add` deleted, `remove` to
  `world.remove` or `world.despawn`; `position.set`, `quaternion.set`,
  `rotation.set`, `scale.set` and `setScalar` to `world.set` of
  og.Transform; `castShadow`, `receiveShadow`, `visible`, `geometry` and
  `material` assignments to `world.set` of the component;
- the camera to the og.Camera resource handle, its aspect and
  `updateProjectionMatrix` deleted; `scene.background` to the camera's
  background;
- the scene, the renderer, its size, pixel ratio, shadow map and colour
  space deleted, `document.body.appendChild(renderer.domElement)` too
  (its animation loop, tone mapping and clear colour are listed);
- `TextureLoader` loads of a string to `og.load.texture` of the asset
  id (a leading `./`, `/`, `public/` or `assets/` removed).

What is not mechanical stays as it was, under a line
`// og migrate: <what to do> (three-to-og §<n>)` naming the section here,
and is listed with its line: the render loop and the Clock (§8), the
glTF loader (§7), the Raycaster (§9), OrbitControls, InstancedMesh and
ShaderMaterial (§10), ambient and hemisphere lights (§6), Euler and
in-place vector arithmetic, a material or geometry parameter og does not
have, and a `new THREE.Mesh` that is not a declaration of its own. The
output is not a game yet: the rewritten calls take `world`, so move the
setup into `GameDesc.start(world)` and the loop into systems, then
`og check` until it is clean. `--write` rewrites the file in place,
without `--out` or `--write` the result goes to standard output, and
`--json` prints the list as one `migrate` event (each item's `line`,
`kind` of `rewritten`, `removed` or `manual`, the three.js text, what it
became or what to do, and the section).
