<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-StandardMaterialDesc"></a>

### `StandardMaterialDesc` (desc struct)

A physically based metallic-roughness material with three.js MeshStandardMaterial parameter names.

It is og-render's reflected StandardMaterial field for field. Its tangents come from the model (the cook generates MikkTSpace ones where a glTF has none) or are generated when the geometry is made; normals follow a non-uniform scale correctly.

See `render §6`.

A desc: every field may be omitted and takes its default; `{}` is valid.

#### Surface

The metallic-roughness surface and its maps.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The base colour. |
| `map` | `Texture \| null` | `null` | — | — | — | The base colour texture. |
| `metalness` | `number` | `0` | 0 … 1 | — | — | How metallic the surface is. |
| `metalnessMap` | `Texture \| null` | `null` | — | — | — | Metalness from the texture's blue channel (a glTF metallic-roughness texture, the same as roughnessMap's). |
| `roughness` | `number` | `1` | 0 … 1 | — | — | How rough the surface is. |
| `roughnessMap` | `Texture \| null` | `null` | — | — | — | Roughness from the texture's green channel. |
| `normalMap` | `Texture \| null` | `null` | — | — | — | A tangent-space normal map in glTF's tangent frame: red tilts towards +u, green up the image. Cook it with `texture { kind normal }` in its import settings (linear); a colour texture here is read wrong and is a warning in the log. |
| `normalScale` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How strongly the normal map bends the normal, per tangent axis. |
| `aoMap` | `Texture \| null` | `null` | — | — | — | Ambient occlusion from the texture's red channel. |
| `aoMapIntensity` | `number` | `1` | ≥ 0 | — | — | How strongly aoMap darkens the ambient light. |
| `emissive` | `Color` | `{ r: 0, g: 0, b: 0 }` | — | — | — | The emitted colour. |
| `emissiveIntensity` | `number` | `1` | ≥ 0 | — | — | A multiplier on the emitted colour. |
| `emissiveMap` | `Texture \| null` | `null` | — | — | — | The emitted colour texture. |
| `envMapIntensity` | `number` | `1` | ≥ 0 | — | — | A multiplier on image-based lighting. |

#### Alpha and faces

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `opacity` | `number` | `1` | 0 … 1 | — | — | The opacity; below 1 only with transparent. |
| `transparent` | `boolean` | `false` | — | — | — | Blend instead of writing opaque pixels, composited as RenderOptions.transparency says. See `render §2`. |
| `alphaTest` | `number` | `0` | 0 … 1 | — | — | Above 0, discard pixels whose alpha is below this. |
| `alphaToCoverage` | `boolean` | `false` | — | — | — | With MSAA (antialias msaa4), an alpha-tested material turns alpha into sample coverage instead of a hard cut at alphaTest, as three.js alphaToCoverage. Without MSAA it cuts half a pixel's change of alpha below alphaTest (where coverage reaches 0): an edge keeps a few more pixels. See `render §6`. |
| `side` | `Side` | `"front"` | — | — | — | Which faces are drawn. |
| `flatShading` | `boolean` | `false` | — | — | — | Shade with face normals. |

#### Stylised

Unlit, toon, rim, hit flash, dissolve and outline, all off at their defaults; og.MaterialStyle fills the ones a material leaves at their defaults (styled).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `unlit` | `boolean` | `false` | — | — | — | No lighting at all, as three.js MeshBasicMaterial: the colour (times map) plus emissive. |
| `toon` | `boolean` | `false` | — | — | — | Cel shading, as three.js MeshToonMaterial: light is quantized by gradientMap (three built-in bands without one), highlights are hard. |
| `gradientMap` | `Texture \| null` | `null` | — | — | — | The toon ramp: a texture whose first row maps the light's angle (left dark, right lit) to the light received, read without filtering. |
| `rimColor` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | Rim (fresnel) light added at grazing angles, unshadowed (linear). |
| `rimIntensity` | `number` | `0` | ≥ 0 | — | — | The rim light's strength; 0 is none. |
| `rimPower` | `number` | `3` | ≥ 0.1 | — | — | How tight the rim is (higher: thinner). |
| `flashColor` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The colour a hit flash mixes towards (linear, may exceed 1). |
| `flash` | `number` | `0` | 0 … 1 | — | — | How far the final colour is mixed towards flashColor (0 none, 1 flat): animate it per tick for a hit flash. |
| `dissolve` | `number` | `0` | 0 … 1 | — | — | The dissolved fraction: 0 whole, 1 gone. Pixels whose noise is below it are discarded (shadows too); the band just above glows with dissolveEdgeColor. |
| `dissolveMap` | `Texture \| null` | `null` | — | — | — | The dissolve noise from the texture's red channel; without one, value noise over the object-space position. |
| `dissolveEdgeColor` | `Color` | `{ r: 4, g: 1.5, b: 0.375 }` | — | — | — | The glowing edge (linear, usually above 1 so it blooms). |
| `dissolveEdgeWidth` | `number` | `0.0625` | 0 … 1 | — | — | The width of the glowing band in noise units. |
| `dissolveScale` | `number` | `4` | ≥ 0 | — | — | The frequency of the built-in noise (cells per object-space unit). |
| `outlineColor` | `Color` | `{ r: 0, g: 0, b: 0 }` | — | — | — | This material's outline colour (linear). |
| `outlineWidth` | `number` | `0` | 0 … 31 | — | — | This material's hull outline width in pixels; 0 is none (PostEffects.outline outlines everything). It lines the silhouette, where one part passes in front of another and creases against other objects, never edges between its own faces; a face at a grazing angle is lined only where it ends. See `render §6`. |
| `styled` | `boolean` | `true` | — | — | — | Whether og.MaterialStyle (a look's scene-wide stylised fields) reaches this material; false keeps it as declared under any look. A material with a physical lobe (any Physical layers field, or ior not 1.5) keeps that shading under the style's toon and takes the rest; its own toon still applies. See `render §6`. |

#### Physical layers

MeshPhysicalMaterial's fields under three.js's names and glTF's KHR_materials_* meanings, all off at their defaults; drawn on every backend, ignored by unlit and toon materials.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `clearcoat` | `number` | `0` | 0 … 1 | `ratio` | — | A clear lacquer over the surface, as three.js MeshPhysicalMaterial.clearcoat: a second, 4 % reflective specular layer on the surface's normal before normalMap (car paint, varnished wood, wet plastic); 0 is none. |
| `clearcoatRoughness` | `number` | `0` | 0 … 1 | `ratio` | — | How rough the clear coat is, as three.js MeshPhysicalMaterial.clearcoatRoughness. |
| `clearcoatMap` | `Texture \| null` | `null` | — | — | — | The coat's strength from the texture's red channel, times clearcoat (MeshPhysicalMaterial.clearcoatMap). |
| `clearcoatRoughnessMap` | `Texture \| null` | `null` | — | — | — | The coat's roughness from the texture's green channel, times clearcoatRoughness. |
| `clearcoatNormalMap` | `Texture \| null` | `null` | — | — | — | A tangent-space normal map of the coat alone; the base keeps normalMap (MeshPhysicalMaterial.clearcoatNormalMap). |
| `clearcoatNormalScale` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How strongly clearcoatNormalMap bends the coat's normal, per tangent axis. |
| `sheen` | `number` | `0` | 0 … 1 | `ratio` | — | A velvet layer brightening grazing angles, as MeshPhysicalMaterial.sheen (KHR_materials_sheen): Estevez and Kulla's Charlie fibres with shadowing and a soft terminator, lit by the environment through its directional albedo, its light taken from the surface below. Strength times sheenColor; 0 none. See `render §6`. |
| `sheenColor` | `Color` | `{ r: 0, g: 0, b: 0 }` | — | — | — | The sheen's colour (linear), times sheenColorMap; black is none. |
| `sheenRoughness` | `number` | `1` | 0 … 1 | `ratio` | — | How rough the sheen is: low keeps it to a thin rim, high spreads it over the surface. |
| `sheenColorMap` | `Texture \| null` | `null` | — | — | — | The sheen's colour from the texture's RGB (sRGB). |
| `sheenRoughnessMap` | `Texture \| null` | `null` | — | — | — | The sheen's roughness from the texture's alpha channel. |
| `transmission` | `number` | `0` | 0 … 1 | `ratio` | — | How much light passes through, as MeshPhysicalMaterial.transmission (glass, water): drawn after the opaque scene, back to front with the transparent ones, it refracts what was drawn before it (not other transmissive or transparent objects), blurred by roughness; 0 is none. It keeps its reflections. |
| `transmissionMap` | `Texture \| null` | `null` | — | — | — | Transmission from the texture's red channel. |
| `thickness` | `number` | `0` | ≥ 0 | — | — | The volume's thickness under the surface in the mesh's own units, scaled with it (glTF KHR_materials_volume): how far refraction bends the view and how much attenuationColor absorbs; 0 is a thin wall. |
| `thicknessMap` | `Texture \| null` | `null` | — | — | — | Thickness from the texture's green channel. |
| `attenuationDistance` | `number` | `0` | ≥ 0 | — | — | The distance light travels in the volume before it is tinted to attenuationColor (world units); 0 is no absorption, three.js's Infinity. |
| `attenuationColor` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The colour white light becomes after attenuationDistance in the volume (linear). |
| `ior` | `number` | `1.5` | 1 … 2.333 | — | — | The index of refraction (glTF KHR_materials_ior): the reflectance of a dielectric, ((ior - 1) / (ior + 1))^2 (4 % at 1.5), and how far transmission refracts. |
| `iridescence` | `number` | `0` | 0 … 1 | `ratio` | — | A thin-film interference layer, as MeshPhysicalMaterial.iridescence (soap bubbles, oil, beetle shells); 0 is none. |
| `iridescenceIOR` | `number` | `1.3` | 1 … 2.333 | — | — | The film's index of refraction. |
| `iridescenceThicknessRange` | `Vector2` | `{ x: 100, y: 400 }` | — | — | — | The film's thickness in nanometres: x where iridescenceThicknessMap is black, y where it is white and everywhere without the map (three.js's [100, 400]). |
| `iridescenceMap` | `Texture \| null` | `null` | — | — | — | Iridescence from the texture's red channel. |
| `iridescenceThicknessMap` | `Texture \| null` | `null` | — | — | — | The film's thickness between iridescenceThicknessRange's ends from the texture's green channel. |
| `anisotropy` | `number` | `0` | 0 … 1 | `ratio` | — | Stretches the specular highlight along a direction of the surface, as MeshPhysicalMaterial.anisotropy (brushed metal, hair, satin); 0 is none, 1 fully stretched. Needs the geometry's tangents (generated when it has none). |
| `anisotropyRotation` | `number` | `0` | — | `rad` | — | The stretch's direction from the tangent (+u) towards the bitangent. |
| `anisotropyMap` | `Texture \| null` | `null` | — | — | — | The stretch's direction in red and green (-1 to 1 in the tangent frame, turned by anisotropyRotation) and its strength in blue, times anisotropy. |
| `subsurface` | `number` | `0` | 0 … 1 | `ratio` | — | How much of the diffuse light spreads under the surface before it leaves (skin, wax, marble, jade, milk): the shading softens past the terminator where the surface curves, and with a thickness, light from behind comes through. 0 is none. See `render §6`. |
| `subsurfaceRadius` | `Vector3` | `{ x: 0.0037, y: 0.0014, z: 0.0007 }` | ≥ 0 | `m` | — | How far light of the red, green and blue channel travels under the surface: skin's by default, red farthest. See `render §6`. |
| `hair` | `number` | `0` | 0 … 1 | `ratio` | — | How much of the surface is strands of hair, running along v with their roots at the top of the image (turned by anisotropyRotation): a white highlight across them, a second in the hair's color nearer the tips, and with a thickness the light behind them comes through. 0 is none. See `render §6`. |
| `irisRadius` | `number` | `0` | 0 … 0.5 | — | — | An eye: the radius of the iris in uv, round the centre of the uv square. The base colour within it lies under the cornea and is seen refracted by ior, so the iris has depth and stays in view from the side. 0 is no eye. See `render §6`. |
| `irisDepth` | `number` | `0.4` | 0 … 2 | `ratio` | — | How deep the iris lies under the top of the cornea, in iris radii. See `render §6`. |

#### Texture arrays

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `mapArray` | `TextureArray \| null` | `null` | — | — | — | The base colour from layer mapLayer of a texture array (og.load.textureArray) instead of map, each Mesh picking its layer (Mesh.materialLayer), in one draw. With mapArray or normalMapArray, colour and normals come from arrays only (map and normalMap unread). Physical and shader materials take none. See `render §6`. |
| `mapLayer` | `number` | `0` | — | — | — | The layer of mapArray drawn where the Mesh does not pick one (past the array's last layer: the last). |
| `normalMapArray` | `TextureArray \| null` | `null` | — | — | — | A tangent-space normal map from layer normalMapLayer of a texture array (linear: cook it with `kind normal`), the counterpart of mapArray. |
| `normalMapLayer` | `number` | `0` | — | — | — | The layer of normalMapArray read where the Mesh does not pick one. |

#### Texture transforms

Every map samples TEXCOORD_0; a glTF transform's own texCoord (a second uv set) is not read, and the cook notes it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `mapOffset` | `Vector2` | `{ x: 0, y: 0 }` | — | — | — | The uv offset map is sampled at, as three.js Texture.offset and glTF KHR_texture_transform: uv' = offset + rotation(rotation) * (repeat * uv), glTF's order. A loaded glTF's transform is here unless the cook baked it into the model's uvs. See `render §6`. |
| `mapRepeat` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How many times map repeats across the uvs, as three.js Texture.repeat (glTF's scale): tiling and atlas cells. |
| `mapRotation` | `number` | `0` | — | `rad` | — | The uv rotation of map, counter-clockwise in uv space about the uv origin, as glTF KHR_texture_transform's rotation. |
| `metalnessMapOffset` | `Vector2` | `{ x: 0, y: 0 }` | — | — | — | The uv offset metalnessMap and roughnessMap (one glTF texture; a physical material's other maps and aoMap are packed with them and sampled here too) are sampled at, as mapOffset. A loaded glTF's transform is here unless the cook baked it into the model's uvs. |
| `metalnessMapRepeat` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How many times metalnessMap and roughnessMap (and the maps packed with them, as metalnessMapOffset) repeat across the uvs, as three.js Texture.repeat (glTF's scale): tiling and atlas cells. |
| `metalnessMapRotation` | `number` | `0` | — | `rad` | — | The uv rotation of metalnessMap and roughnessMap (and the maps packed with them, as metalnessMapOffset), counter-clockwise in uv space about the uv origin, as glTF KHR_texture_transform's rotation. |
| `normalMapOffset` | `Vector2` | `{ x: 0, y: 0 }` | — | — | — | The uv offset normalMap is sampled at, as three.js Texture.offset and glTF KHR_texture_transform: uv' = offset + rotation(rotation) * (repeat * uv), glTF's order. A loaded glTF's transform is here unless the cook baked it into the model's uvs. |
| `normalMapRepeat` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How many times normalMap repeats across the uvs, as three.js Texture.repeat (glTF's scale): tiling and atlas cells. |
| `normalMapRotation` | `number` | `0` | — | `rad` | — | The uv rotation of normalMap, counter-clockwise in uv space about the uv origin, as glTF KHR_texture_transform's rotation. |
| `emissiveMapOffset` | `Vector2` | `{ x: 0, y: 0 }` | — | — | — | The uv offset emissiveMap is sampled at, as three.js Texture.offset and glTF KHR_texture_transform: uv' = offset + rotation(rotation) * (repeat * uv), glTF's order. A loaded glTF's transform is here unless the cook baked it into the model's uvs. |
| `emissiveMapRepeat` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How many times emissiveMap repeats across the uvs, as three.js Texture.repeat (glTF's scale): tiling and atlas cells. |
| `emissiveMapRotation` | `number` | `0` | — | `rad` | — | The uv rotation of emissiveMap, counter-clockwise in uv space about the uv origin, as glTF KHR_texture_transform's rotation. |
| `aoMapOffset` | `Vector2` | `{ x: 0, y: 0 }` | — | — | — | The uv offset aoMap is sampled at, as three.js Texture.offset and glTF KHR_texture_transform: uv' = offset + rotation(rotation) * (repeat * uv), glTF's order. A loaded glTF's transform is here unless the cook baked it into the model's uvs. |
| `aoMapRepeat` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How many times aoMap repeats across the uvs, as three.js Texture.repeat (glTF's scale): tiling and atlas cells. |
| `aoMapRotation` | `number` | `0` | — | `rad` | — | The uv rotation of aoMap, counter-clockwise in uv space about the uv origin, as glTF KHR_texture_transform's rotation. |

#### Wireframe

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `wireframe` | `boolean` | `false` | — | — | — | Draw the edges of the mesh's triangles instead of its faces, as three.js Material.wireframe: one CSS pixel wide lines, unlit, depth tested, in the surface's colour along each edge (color × map × vertex colour + emissive × emissiveMap) or wireframeColor. Its meshes cast no shadow. See `render §6`. |
| `wireframeAngle` | `number` | `0` | 0 … 180 | `deg` | — | With wireframe: only the mesh's feature edges, its open borders and the edges whose two faces meet at more than this angle (welded by position, so uv seams and split hard edges do not show): a box's 12 edges, nothing across a smooth sphere but where it ends. 0 is every triangle edge. |
| `wireframeColor` | `Color` | `{ r: 0, g: 0, b: 0 }` | — | — | — | With wireframe: one colour for every edge (linear, above 1 blooms) instead of the colour the surface shows along it; black (the zero value) is the surface's. |

#### Leaves, paint and weather

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `foliage` | `FoliageDesc` | `{}` | — | — | — | Leaf shading; omitted parameters keep their values on updateMaterial. The reflected foliage vector stores translucency in x and domeNormals in y. |
| `paintDetail` | `number` | `0` | 0 … 1 | `ratio` | — | How much of its detail the painterly filter (Stylise.paintRadius) keeps on this material, times Stylise.paintDetail: 0 painted at the full radius, 1 kept as drawn (a face, a hand, a sign's lettering). |
| `weathering` | `number` | `1` | 0 … 1 | `ratio` | — | How much og.WeatherState reaches this material: its wetness, rain streaks, puddles, snow and dust (1 all of it, 0 none). 0 for what the weather never touches: an indoor floor seen through a door, a screen, a hero kept clean, a surface a game wets itself; between for a coat that sheds some. |

```ogt
struct StandardMaterialDesc {
  doc "A physically based metallic-roughness material with three.js MeshStandardMaterial parameter names."
  doc "It is og-render's reflected StandardMaterial field for field. Its tangents come from the model (the cook generates MikkTSpace ones where a glTF has none) or are generated when the geometry is made; normals follow a non-uniform scale correctly."
  guide "render §6"
  desc true
  group "Surface" "The metallic-roughness surface and its maps."
  field color Color "The base colour." {
    default 1 1 1
  }
  field map Option.Texture "The base colour texture."
  field metalness f32 "How metallic the surface is." {
    min 0
    max 1
  }
  field metalnessMap Option.Texture "Metalness from the texture's blue channel (a glTF metallic-roughness texture, the same as roughnessMap's)."
  field roughness f32 "How rough the surface is." {
    default 1
    min 0
    max 1
  }
  field roughnessMap Option.Texture "Roughness from the texture's green channel."
  field normalMap Option.Texture "A tangent-space normal map in glTF's tangent frame: red tilts towards +u, green up the image. Cook it with `texture { kind normal }` in its import settings (linear); a colour texture here is read wrong and is a warning in the log."
  field normalScale Vector2 "How strongly the normal map bends the normal, per tangent axis." {
    default 1 1
  }
  field aoMap Option.Texture "Ambient occlusion from the texture's red channel."
  field aoMapIntensity f32 "How strongly aoMap darkens the ambient light." {
    default 1
    min 0
  }
  field emissive Color "The emitted colour."
  field emissiveIntensity f32 "A multiplier on the emitted colour." {
    default 1
    min 0
  }
  field emissiveMap Option.Texture "The emitted colour texture."
  field envMapIntensity f32 "A multiplier on image-based lighting." {
    default 1
    min 0
  }
  group "Alpha and faces"
  field opacity f32 "The opacity; below 1 only with transparent." {
    default 1
    min 0
    max 1
  }
  field transparent bool "Blend instead of writing opaque pixels, composited as RenderOptions.transparency says." {
    guide "render §2"
  }
  field alphaTest f32 "Above 0, discard pixels whose alpha is below this." {
    min 0
    max 1
  }
  field alphaToCoverage bool "With MSAA (antialias msaa4), an alpha-tested material turns alpha into sample coverage instead of a hard cut at alphaTest, as three.js alphaToCoverage. Without MSAA it cuts half a pixel's change of alpha below alphaTest (where coverage reaches 0): an edge keeps a few more pixels." {
    guide "render §6"
  }
  field side Side "Which faces are drawn."
  field flatShading bool "Shade with face normals."
  group "Stylised" "Unlit, toon, rim, hit flash, dissolve and outline, all off at their defaults; og.MaterialStyle fills the ones a material leaves at their defaults (styled)."
  field unlit bool "No lighting at all, as three.js MeshBasicMaterial: the colour (times map) plus emissive."
  field toon bool "Cel shading, as three.js MeshToonMaterial: light is quantized by gradientMap (three built-in bands without one), highlights are hard."
  field gradientMap Option.Texture "The toon ramp: a texture whose first row maps the light's angle (left dark, right lit) to the light received, read without filtering."
  field rimColor Color "Rim (fresnel) light added at grazing angles, unshadowed (linear)." {
    default 1 1 1
  }
  field rimIntensity f32 "The rim light's strength; 0 is none." {
    min 0
  }
  field rimPower f32 "How tight the rim is (higher: thinner)." {
    default 3
    min 0.1
  }
  field flashColor Color "The colour a hit flash mixes towards (linear, may exceed 1)." {
    default 1 1 1
  }
  field flash f32 "How far the final colour is mixed towards flashColor (0 none, 1 flat): animate it per tick for a hit flash." {
    min 0
    max 1
  }
  field dissolve f32 "The dissolved fraction: 0 whole, 1 gone. Pixels whose noise is below it are discarded (shadows too); the band just above glows with dissolveEdgeColor." {
    min 0
    max 1
  }
  field dissolveMap Option.Texture "The dissolve noise from the texture's red channel; without one, value noise over the object-space position."
  field dissolveEdgeColor Color "The glowing edge (linear, usually above 1 so it blooms)." {
    default 4 1.5 0.375
  }
  field dissolveEdgeWidth f32 "The width of the glowing band in noise units." {
    default 0.0625
    min 0
    max 1
  }
  field dissolveScale f32 "The frequency of the built-in noise (cells per object-space unit)." {
    default 4
    min 0
  }
  field outlineColor Color "This material's outline colour (linear)."
  field outlineWidth f32 "This material's hull outline width in pixels; 0 is none (PostEffects.outline outlines everything). It lines the silhouette, where one part passes in front of another and creases against other objects, never edges between its own faces; a face at a grazing angle is lined only where it ends." {
    min 0
    max 31
    guide "render §6"
  }
  field styled bool "Whether og.MaterialStyle (a look's scene-wide stylised fields) reaches this material; false keeps it as declared under any look. A material with a physical lobe (any Physical layers field, or ior not 1.5) keeps that shading under the style's toon and takes the rest; its own toon still applies." {
    default true
    guide "render §6"
  }
  group "Physical layers" "MeshPhysicalMaterial's fields under three.js's names and glTF's KHR_materials_* meanings, all off at their defaults; drawn on every backend, ignored by unlit and toon materials."
  field clearcoat f32 "A clear lacquer over the surface, as three.js MeshPhysicalMaterial.clearcoat: a second, 4 % reflective specular layer on the surface's normal before normalMap (car paint, varnished wood, wet plastic); 0 is none." {
    min 0
    max 1
    unit ratio
  }
  field clearcoatRoughness f32 "How rough the clear coat is, as three.js MeshPhysicalMaterial.clearcoatRoughness." {
    min 0
    max 1
    unit ratio
  }
  field clearcoatMap Option.Texture "The coat's strength from the texture's red channel, times clearcoat (MeshPhysicalMaterial.clearcoatMap)."
  field clearcoatRoughnessMap Option.Texture "The coat's roughness from the texture's green channel, times clearcoatRoughness."
  field clearcoatNormalMap Option.Texture "A tangent-space normal map of the coat alone; the base keeps normalMap (MeshPhysicalMaterial.clearcoatNormalMap)."
  field clearcoatNormalScale Vector2 "How strongly clearcoatNormalMap bends the coat's normal, per tangent axis." {
    default 1 1
  }
  field sheen f32 "A velvet layer brightening grazing angles, as MeshPhysicalMaterial.sheen (KHR_materials_sheen): Estevez and Kulla's Charlie fibres with shadowing and a soft terminator, lit by the environment through its directional albedo, its light taken from the surface below. Strength times sheenColor; 0 none." {
    min 0
    max 1
    unit ratio
    guide "render §6"
  }
  field sheenColor Color "The sheen's colour (linear), times sheenColorMap; black is none."
  field sheenRoughness f32 "How rough the sheen is: low keeps it to a thin rim, high spreads it over the surface." {
    default 1
    min 0
    max 1
    unit ratio
  }
  field sheenColorMap Option.Texture "The sheen's colour from the texture's RGB (sRGB)."
  field sheenRoughnessMap Option.Texture "The sheen's roughness from the texture's alpha channel."
  field transmission f32 "How much light passes through, as MeshPhysicalMaterial.transmission (glass, water): drawn after the opaque scene, back to front with the transparent ones, it refracts what was drawn before it (not other transmissive or transparent objects), blurred by roughness; 0 is none. It keeps its reflections." {
    min 0
    max 1
    unit ratio
  }
  field transmissionMap Option.Texture "Transmission from the texture's red channel."
  field thickness f32 "The volume's thickness under the surface in the mesh's own units, scaled with it (glTF KHR_materials_volume): how far refraction bends the view and how much attenuationColor absorbs; 0 is a thin wall." {
    min 0
  }
  field thicknessMap Option.Texture "Thickness from the texture's green channel."
  field attenuationDistance f32 "The distance light travels in the volume before it is tinted to attenuationColor (world units); 0 is no absorption, three.js's Infinity." {
    min 0
  }
  field attenuationColor Color "The colour white light becomes after attenuationDistance in the volume (linear)." {
    default 1 1 1
  }
  field ior f32 "The index of refraction (glTF KHR_materials_ior): the reflectance of a dielectric, ((ior - 1) / (ior + 1))^2 (4 % at 1.5), and how far transmission refracts." {
    default 1.5
    min 1
    max 2.333
  }
  field iridescence f32 "A thin-film interference layer, as MeshPhysicalMaterial.iridescence (soap bubbles, oil, beetle shells); 0 is none." {
    min 0
    max 1
    unit ratio
  }
  field iridescenceIOR f32 "The film's index of refraction." {
    default 1.3
    min 1
    max 2.333
  }
  field iridescenceThicknessRange Vector2 "The film's thickness in nanometres: x where iridescenceThicknessMap is black, y where it is white and everywhere without the map (three.js's [100, 400])." {
    default 100 400
  }
  field iridescenceMap Option.Texture "Iridescence from the texture's red channel."
  field iridescenceThicknessMap Option.Texture "The film's thickness between iridescenceThicknessRange's ends from the texture's green channel."
  field anisotropy f32 "Stretches the specular highlight along a direction of the surface, as MeshPhysicalMaterial.anisotropy (brushed metal, hair, satin); 0 is none, 1 fully stretched. Needs the geometry's tangents (generated when it has none)." {
    min 0
    max 1
    unit ratio
  }
  field anisotropyRotation f32 "The stretch's direction from the tangent (+u) towards the bitangent." {
    unit rad
  }
  field anisotropyMap Option.Texture "The stretch's direction in red and green (-1 to 1 in the tangent frame, turned by anisotropyRotation) and its strength in blue, times anisotropy."
  field subsurface f32 "How much of the diffuse light spreads under the surface before it leaves (skin, wax, marble, jade, milk): the shading softens past the terminator where the surface curves, and with a thickness, light from behind comes through. 0 is none." {
    min 0
    max 1
    unit ratio
    guide "render §6"
  }
  field subsurfaceRadius Vector3 "How far light of the red, green and blue channel travels under the surface: skin's by default, red farthest." {
    default 0.0037 0.0014 0.0007
    min 0
    unit m
    guide "render §6"
  }
  field hair f32 "How much of the surface is strands of hair, running along v with their roots at the top of the image (turned by anisotropyRotation): a white highlight across them, a second in the hair's color nearer the tips, and with a thickness the light behind them comes through. 0 is none." {
    min 0
    max 1
    unit ratio
    guide "render §6"
  }
  field irisRadius f32 "An eye: the radius of the iris in uv, round the centre of the uv square. The base colour within it lies under the cornea and is seen refracted by ior, so the iris has depth and stays in view from the side. 0 is no eye." {
    min 0
    max 0.5
    guide "render §6"
  }
  field irisDepth f32 "How deep the iris lies under the top of the cornea, in iris radii." {
    default 0.4
    min 0
    max 2
    unit ratio
    guide "render §6"
  }
  group "Texture arrays"
  field mapArray Option.TextureArray "The base colour from layer mapLayer of a texture array (og.load.textureArray) instead of map, each Mesh picking its layer (Mesh.materialLayer), in one draw. With mapArray or normalMapArray, colour and normals come from arrays only (map and normalMap unread). Physical and shader materials take none." {
    guide "render §6"
  }
  field mapLayer u32 "The layer of mapArray drawn where the Mesh does not pick one (past the array's last layer: the last)."
  field normalMapArray Option.TextureArray "A tangent-space normal map from layer normalMapLayer of a texture array (linear: cook it with `kind normal`), the counterpart of mapArray."
  field normalMapLayer u32 "The layer of normalMapArray read where the Mesh does not pick one."
  group "Texture transforms" "Every map samples TEXCOORD_0; a glTF transform's own texCoord (a second uv set) is not read, and the cook notes it."
  field mapOffset Vector2 "The uv offset map is sampled at, as three.js Texture.offset and glTF KHR_texture_transform: uv' = offset + rotation(rotation) * (repeat * uv), glTF's order. A loaded glTF's transform is here unless the cook baked it into the model's uvs." {
    guide "render §6"
  }
  field mapRepeat Vector2 "How many times map repeats across the uvs, as three.js Texture.repeat (glTF's scale): tiling and atlas cells." {
    default 1 1
  }
  field mapRotation f32 "The uv rotation of map, counter-clockwise in uv space about the uv origin, as glTF KHR_texture_transform's rotation." {
    unit rad
  }
  field metalnessMapOffset Vector2 "The uv offset metalnessMap and roughnessMap (one glTF texture; a physical material's other maps and aoMap are packed with them and sampled here too) are sampled at, as mapOffset. A loaded glTF's transform is here unless the cook baked it into the model's uvs."
  field metalnessMapRepeat Vector2 "How many times metalnessMap and roughnessMap (and the maps packed with them, as metalnessMapOffset) repeat across the uvs, as three.js Texture.repeat (glTF's scale): tiling and atlas cells." {
    default 1 1
  }
  field metalnessMapRotation f32 "The uv rotation of metalnessMap and roughnessMap (and the maps packed with them, as metalnessMapOffset), counter-clockwise in uv space about the uv origin, as glTF KHR_texture_transform's rotation." {
    unit rad
  }
  field normalMapOffset Vector2 "The uv offset normalMap is sampled at, as three.js Texture.offset and glTF KHR_texture_transform: uv' = offset + rotation(rotation) * (repeat * uv), glTF's order. A loaded glTF's transform is here unless the cook baked it into the model's uvs."
  field normalMapRepeat Vector2 "How many times normalMap repeats across the uvs, as three.js Texture.repeat (glTF's scale): tiling and atlas cells." {
    default 1 1
  }
  field normalMapRotation f32 "The uv rotation of normalMap, counter-clockwise in uv space about the uv origin, as glTF KHR_texture_transform's rotation." {
    unit rad
  }
  field emissiveMapOffset Vector2 "The uv offset emissiveMap is sampled at, as three.js Texture.offset and glTF KHR_texture_transform: uv' = offset + rotation(rotation) * (repeat * uv), glTF's order. A loaded glTF's transform is here unless the cook baked it into the model's uvs."
  field emissiveMapRepeat Vector2 "How many times emissiveMap repeats across the uvs, as three.js Texture.repeat (glTF's scale): tiling and atlas cells." {
    default 1 1
  }
  field emissiveMapRotation f32 "The uv rotation of emissiveMap, counter-clockwise in uv space about the uv origin, as glTF KHR_texture_transform's rotation." {
    unit rad
  }
  field aoMapOffset Vector2 "The uv offset aoMap is sampled at, as three.js Texture.offset and glTF KHR_texture_transform: uv' = offset + rotation(rotation) * (repeat * uv), glTF's order. A loaded glTF's transform is here unless the cook baked it into the model's uvs."
  field aoMapRepeat Vector2 "How many times aoMap repeats across the uvs, as three.js Texture.repeat (glTF's scale): tiling and atlas cells." {
    default 1 1
  }
  field aoMapRotation f32 "The uv rotation of aoMap, counter-clockwise in uv space about the uv origin, as glTF KHR_texture_transform's rotation." {
    unit rad
  }
  group "Wireframe"
  field wireframe bool "Draw the edges of the mesh's triangles instead of its faces, as three.js Material.wireframe: one CSS pixel wide lines, unlit, depth tested, in the surface's colour along each edge (color × map × vertex colour + emissive × emissiveMap) or wireframeColor. Its meshes cast no shadow." {
    guide "render §6"
  }
  field wireframeAngle f32 "With wireframe: only the mesh's feature edges, its open borders and the edges whose two faces meet at more than this angle (welded by position, so uv seams and split hard edges do not show): a box's 12 edges, nothing across a smooth sphere but where it ends. 0 is every triangle edge." {
    min 0
    max 180
    unit deg
  }
  field wireframeColor Color "With wireframe: one colour for every edge (linear, above 1 blooms) instead of the colour the surface shows along it; black (the zero value) is the surface's."
  group "Leaves, paint and weather"
  field foliage FoliageDesc "Leaf shading; omitted parameters keep their values on updateMaterial. The reflected foliage vector stores translucency in x and domeNormals in y."
  field paintDetail f32 "How much of its detail the painterly filter (Stylise.paintRadius) keeps on this material, times Stylise.paintDetail: 0 painted at the full radius, 1 kept as drawn (a face, a hand, a sign's lettering)." {
    min 0
    max 1
    unit ratio
  }
  field weathering f32 "How much og.WeatherState reaches this material: its wetness, rain streaks, puddles, snow and dust (1 all of it, 0 none). 0 for what the weather never touches: an indoor floor seen through a door, a screen, a hero kept clean, a surface a game wets itself; between for a coat that sheds some." {
    default 1
    min 0
    max 1
    unit ratio
  }
}
```

<sub>Module `render` line 196</sub>
