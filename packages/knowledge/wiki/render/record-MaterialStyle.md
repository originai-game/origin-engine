<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-MaterialStyle"></a>

### `og.MaterialStyle` (resource)

Stylised defaults for every lit material, so a whole scene takes one treatment without editing each material: toon shading and its ramp, a coloured shadow side, rim light, outlines whose width falls off with distance, and flat facets. Each applies to a material (og.render.standardMaterial or a loaded glTF's) that leaves that field at its own default; a terrain (og.Terrain) takes toon (the built-in three bands, matte) and shadowTint; unlit and shader materials, materials with StandardMaterialDesc.styled false, sprites and the UI are untouched, and toon passes by a material with a MeshPhysicalMaterial lobe (clearcoat, sheen, transmission, iridescence, anisotropy, ior), which keeps its physical shading. All off (false, 0) is no change. A look sets it; RenderCapabilities.stylise names what this renderer draws. Read after each tick; it never feeds back into the simulation.

Token `og.MaterialStyle`, registered under the schema name `og.MaterialStyle`; its value type is `MaterialStyle`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `toon` | `boolean` | `false` | — | — | — | Cel shading for every lit material, as StandardMaterialDesc.toon: light quantised by the ramp, hard highlights. A material with a physical lobe (clearcoat, transmission and the rest) keeps its physical shading; set its own toon to make it toon anyway. |
| `gradientMap` | `Texture` | `0` | — | — | — | The toon ramp for materials without their own (StandardMaterialDesc.gradientMap, og.load.texture): its first row maps the light's angle to the light received, read without filtering; 0 is three built-in bands. |
| `shadowTint` | `Color` | `{ r: 0.18, g: 0.16, b: 0.32 }` | — | — | — | The colour the shadowed side of a toon or lit surface moves towards instead of black (linear): the cool or warm shadow of illustration. |
| `shadowTintAmount` | `number` | `0` | 0 … 1 | `ratio` | tunable | How far shadowed light moves towards shadowTint; 0 is off. |
| `shadowDarkness` | `number` | `0` | 0 … 1 | `ratio` | tunable | How far shadowed surfaces fall towards black: where the sun does not reach (the toon ramp's dark band, the side turned away, a cast shadow) a surface loses this share of the sun's ramp and of the ambient light (the Environment's map and its hemisphere light) as well, so 1 draws every shadow as a solid black mass (the ink shadows of noir comics) and the toon ramp's middle band keeps half its light; point, spot and rect lights and the emissive still show inside it, and shadowTint's light is added after it. 0 is off. |
| `rimColor` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | Rim (fresnel) light for materials without their own (linear). |
| `rimIntensity` | `number` | `0` | ≥ 0 | — | tunable | The rim light's strength; 0 is none. |
| `rimPower` | `number` | `3` | ≥ 0.1 | — | — | How tight the rim is (higher: thinner). |
| `outlineColor` | `Color` | `{ r: 0.02, g: 0.02, b: 0.03 }` | — | — | — | The outline colour for materials without their own (linear). |
| `outlineWidth` | `number` | `0` | 0 … 31 | — | tunable | The outline width in pixels at outlineDistance for materials without their own; 0 is none. A hull outline (StandardMaterialDesc.outlineWidth): silhouettes and creases against other objects, never the faces of one, so voxel and flat-shaded models stay clean. A surface seen at a grazing angle is lined only where it ends: a 1,600 m ground under a chase camera keeps its far edge and nothing else (no band along the horizon). |
| `outlineDistance` | `number` | `0` | ≥ 0 | `m` | — | The distance at which outlines are outlineWidth wide: nearer they grow, up to four times outlineWidth, farther they thin in proportion (an inked look keeps line weight by depth); 0 keeps every outline outlineWidth wide. |
| `flatShading` | `boolean` | `false` | — | — | — | Shade every lit material with face normals, as StandardMaterialDesc.flatShading: visible facets for low-poly, paper-craft and voxel looks. |
| `surface` | `MaterialSurface` | `"none"` | — | — | — | A made surface on every lit material (clay, paper, plastic): its normal detail, colour grain and light response, in each mesh's own space; none is no change. Unlit materials, materials with StandardMaterialDesc.styled false and those with a MeshPhysicalMaterial lobe keep their own shading. |
| `surfaceStrength` | `number` | `1` | 0 … 1 | `ratio` | tunable | How strongly the surface shows: its bumps, grain and wrapped light; 0 is none. |
| `surfaceScale` | `number` | `0` | ≥ 0 | `m` | — | The size of the surface's pattern in metres of the mesh's own space (a clay lump, the paper's fibre); 0 takes the surface's own (clay 0.12, paper 0.15, plastic 0.3). |
| `toonAmbient` | `number` | `0` | 0 … 1 | `ratio` | tunable | How smooth a toon material's ambient light (the Environment and its hemisphere light) stays: 0 one flat colour, the light from above and below averaged (MToon's equalised ambient), with the environment's reflection as one flat colour, so the bands sit on flat colour; 1 the smooth PBR ambient and reflections, which wash the bands out. |
| `toonSoftness` | `number` | `1` | 0 … 4 | — | tunable | The toon bands' and highlight's edges, anti-aliased over this many pixels (lilToon's screen-space smoothing); 0 hard steps that only TAA softens. |
| `outlinePlace` | `OutlinePlace` | `"inside"` | — | — | — | Where outlines lie against an object's silhouette: inside (the zero value, its own edge pixels), outside (round it, at any width, anti-aliased) or both. Creases between objects stay inside. |
| `wireframe` | `boolean` | `false` | — | — | — | Draw every lit material's feature edges over its faces (StandardMaterialDesc.wireframe's edges, with wireframeAngle and wireframeColor from this style where the material sets none), in the colour the surface shows along each edge (its colour times its maps, plus its emissive light, so emissive colours glow through bloom) or wireframeColor; the faces stay drawn as usual and hide the edges behind them, as vector displays drew with hidden lines removed: vector and neon looks, whose dark materials make the fill. A material's own wireframe draws its edges alone. |
| `wireframeAngle` | `number` | `30` | 0 … 180 | `deg` | — | The crease angle wireframe draws past (StandardMaterialDesc.wireframeAngle); 0 every triangle edge. |
| `wireframeColor` | `Color` | `{ r: 0, g: 0, b: 0 }` | — | — | — | The look's line colour for wireframe (StandardMaterialDesc.wireframeColor), linear, above 1 blooms; black (the zero value) draws each edge in the colour its surface shows there. |

```ogt
resource MaterialStyle {
  doc "Stylised defaults for every lit material, so a whole scene takes one treatment without editing each material: toon shading and its ramp, a coloured shadow side, rim light, outlines whose width falls off with distance, and flat facets. Each applies to a material (og.render.standardMaterial or a loaded glTF's) that leaves that field at its own default; a terrain (og.Terrain) takes toon (the built-in three bands, matte) and shadowTint; unlit and shader materials, materials with StandardMaterialDesc.styled false, sprites and the UI are untouched, and toon passes by a material with a MeshPhysicalMaterial lobe (clearcoat, sheen, transmission, iridescence, anisotropy, ior), which keeps its physical shading. All off (false, 0) is no change. A look sets it; RenderCapabilities.stylise names what this renderer draws. Read after each tick; it never feeds back into the simulation."
  field toon bool "Cel shading for every lit material, as StandardMaterialDesc.toon: light quantised by the ramp, hard highlights. A material with a physical lobe (clearcoat, transmission and the rest) keeps its physical shading; set its own toon to make it toon anyway."
  field gradientMap Texture "The toon ramp for materials without their own (StandardMaterialDesc.gradientMap, og.load.texture): its first row maps the light's angle to the light received, read without filtering; 0 is three built-in bands."
  field shadowTint Color "The colour the shadowed side of a toon or lit surface moves towards instead of black (linear): the cool or warm shadow of illustration." {
    default 0.18 0.16 0.32
  }
  field shadowTintAmount f32 "How far shadowed light moves towards shadowTint; 0 is off." {
    min 0
    max 1
    unit ratio
    tunable true
  }
  field shadowDarkness f32 "How far shadowed surfaces fall towards black: where the sun does not reach (the toon ramp's dark band, the side turned away, a cast shadow) a surface loses this share of the sun's ramp and of the ambient light (the Environment's map and its hemisphere light) as well, so 1 draws every shadow as a solid black mass (the ink shadows of noir comics) and the toon ramp's middle band keeps half its light; point, spot and rect lights and the emissive still show inside it, and shadowTint's light is added after it. 0 is off." {
    min 0
    max 1
    unit ratio
    tunable true
  }
  field rimColor Color "Rim (fresnel) light for materials without their own (linear)." {
    default 1 1 1
  }
  field rimIntensity f32 "The rim light's strength; 0 is none." {
    min 0
    tunable true
  }
  field rimPower f32 "How tight the rim is (higher: thinner)." {
    default 3
    min 0.1
  }
  field outlineColor Color "The outline colour for materials without their own (linear)." {
    default 0.02 0.02 0.03
  }
  field outlineWidth f32 "The outline width in pixels at outlineDistance for materials without their own; 0 is none. A hull outline (StandardMaterialDesc.outlineWidth): silhouettes and creases against other objects, never the faces of one, so voxel and flat-shaded models stay clean. A surface seen at a grazing angle is lined only where it ends: a 1,600 m ground under a chase camera keeps its far edge and nothing else (no band along the horizon)." {
    min 0
    max 31
    tunable true
  }
  field outlineDistance f32 "The distance at which outlines are outlineWidth wide: nearer they grow, up to four times outlineWidth, farther they thin in proportion (an inked look keeps line weight by depth); 0 keeps every outline outlineWidth wide." {
    min 0
    unit m
  }
  field flatShading bool "Shade every lit material with face normals, as StandardMaterialDesc.flatShading: visible facets for low-poly, paper-craft and voxel looks."
  field surface MaterialSurface "A made surface on every lit material (clay, paper, plastic): its normal detail, colour grain and light response, in each mesh's own space; none is no change. Unlit materials, materials with StandardMaterialDesc.styled false and those with a MeshPhysicalMaterial lobe keep their own shading."
  field surfaceStrength f32 "How strongly the surface shows: its bumps, grain and wrapped light; 0 is none." {
    default 1
    min 0
    max 1
    unit ratio
    tunable true
  }
  field surfaceScale f32 "The size of the surface's pattern in metres of the mesh's own space (a clay lump, the paper's fibre); 0 takes the surface's own (clay 0.12, paper 0.15, plastic 0.3)." {
    min 0
    unit m
  }
  field toonAmbient f32 "How smooth a toon material's ambient light (the Environment and its hemisphere light) stays: 0 one flat colour, the light from above and below averaged (MToon's equalised ambient), with the environment's reflection as one flat colour, so the bands sit on flat colour; 1 the smooth PBR ambient and reflections, which wash the bands out." {
    min 0
    max 1
    unit ratio
    tunable true
  }
  field toonSoftness f32 "The toon bands' and highlight's edges, anti-aliased over this many pixels (lilToon's screen-space smoothing); 0 hard steps that only TAA softens." {
    default 1
    min 0
    max 4
    tunable true
  }
  field outlinePlace OutlinePlace "Where outlines lie against an object's silhouette: inside (the zero value, its own edge pixels), outside (round it, at any width, anti-aliased) or both. Creases between objects stay inside."
  field wireframe bool "Draw every lit material's feature edges over its faces (StandardMaterialDesc.wireframe's edges, with wireframeAngle and wireframeColor from this style where the material sets none), in the colour the surface shows along each edge (its colour times its maps, plus its emissive light, so emissive colours glow through bloom) or wireframeColor; the faces stay drawn as usual and hide the edges behind them, as vector displays drew with hidden lines removed: vector and neon looks, whose dark materials make the fill. A material's own wireframe draws its edges alone."
  field wireframeAngle f32 "The crease angle wireframe draws past (StandardMaterialDesc.wireframeAngle); 0 every triangle edge." {
    default 30
    min 0
    max 180
    unit deg
  }
  field wireframeColor Color "The look's line colour for wireframe (StandardMaterialDesc.wireframeColor), linear, above 1 blooms; black (the zero value) draws each edge in the colour its surface shows there."
}
```

<sub>`api.ogt` line 5508</sub>
