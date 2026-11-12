<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-MaterialStyle"></a>

### `og.MaterialStyle` (resource)

Stylised defaults for a whole scene: toon shading and its ramp, a coloured shadow side, rim light, outlines, flat facets, made surfaces and wireframe. Each applies to a lit material that leaves that field at its own default; the terrain, grass, scattered copies, impostors and crowds take toon (built-in bands), shadowTint and shadowDarkness. Unlit and shader materials, StandardMaterialDesc.styled false, sprites and the UI are untouched. All off is no change. A look sets it; RenderCapabilities.stylise names what this renderer draws. Read after each tick; it never feeds the simulation.

See `looks §12`.

Token `og.MaterialStyle`, registered under the schema name `og.MaterialStyle`; its value type is `MaterialStyle`.

#### Toon

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `toon` | `boolean` | `false` | — | — | — | Cel shading for every lit material, as StandardMaterialDesc.toon: light quantised by the ramp, hard highlights. A material with a physical lobe (clearcoat, transmission and the rest) keeps its physical shading; set its own toon to make it toon anyway. |
| `gradientMap` | `Texture` | `0` | — | — | — | The toon ramp for materials without their own (StandardMaterialDesc.gradientMap, og.load.texture): its first row maps the light's angle to the light received, read without filtering; 0 is three built-in bands. |

#### Shadow side

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shadowTint` | `Color` | `{ r: 0.18, g: 0.16, b: 0.32 }` | — | — | — | The colour the shadowed side of a toon or lit surface moves towards instead of black (linear): the cool or warm shadow of illustration. |
| `shadowTintAmount` | `number` | `0` | 0 … 1 | `ratio` | tunable | How far shadowed light moves towards shadowTint; 0 is off. |
| `shadowDarkness` | `number` | `0` | 0 … 1 | `ratio` | tunable | How far shadowed surfaces fall towards black: where the sun does not reach, a surface loses this share of the sun's ramp and of the ambient light (1: solid black shadows). Point, spot and rect lights and the emissive still show inside it; shadowTint's light is added after it. 0 is off. See `looks §12`. |

#### Rim light

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `rimColor` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | Rim (fresnel) light for materials without their own (linear). |
| `rimIntensity` | `number` | `0` | ≥ 0 | — | tunable | The rim light's strength; 0 is none. |
| `rimPower` | `number` | `3` | ≥ 0.1 | — | — | How tight the rim is (higher: thinner). |

#### Outlines

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `outlineColor` | `Color` | `{ r: 0.02, g: 0.02, b: 0.03 }` | — | — | — | The outline colour for materials without their own (linear). |
| `outlineWidth` | `number` | `0` | 0 … 31 | — | tunable | The outline width in pixels at outlineDistance for materials without their own; 0 is none. A hull outline (StandardMaterialDesc.outlineWidth): silhouettes and creases against other objects, never the faces of one. A surface seen at a grazing angle is lined only where it ends. See `looks §12`. |
| `outlineDistance` | `number` | `0` | ≥ 0 | `m` | — | The distance at which outlines are outlineWidth wide: nearer they grow, up to four times outlineWidth, farther they thin in proportion (an inked look keeps line weight by depth); 0 keeps every outline outlineWidth wide. |

#### Facets

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `flatShading` | `boolean` | `false` | — | — | — | Shade every lit material with face normals, as StandardMaterialDesc.flatShading: visible facets for low-poly, paper-craft and voxel looks. |

#### Made surface

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `surface` | `MaterialSurface` | `"none"` | — | — | — | A made surface on every lit material (clay, paper, plastic): its normal detail, colour grain and light response, in each mesh's own space; none is no change. Unlit materials, materials with StandardMaterialDesc.styled false and those with a MeshPhysicalMaterial lobe keep their own shading. |
| `surfaceStrength` | `number` | `1` | 0 … 1 | `ratio` | tunable | How strongly the surface shows: its bumps, grain and wrapped light; 0 is none. |
| `surfaceScale` | `number` | `0` | ≥ 0 | `m` | — | The size of the surface's pattern in metres of the mesh's own space (a clay lump, the paper's fibre); 0 takes the surface's own (clay 0.12, paper 0.15, plastic 0.3). |

#### Toon shading

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `toonAmbient` | `number` | `0` | 0 … 1 | `ratio` | tunable | How smooth a toon material's ambient light (the Environment and its hemisphere light) and reflection stay: 0 one flat colour, the light from above and below averaged (MToon's equalised ambient), so the bands sit on flat colour; 1 the smooth PBR ambient and reflections. See `looks §12`. |
| `toonSoftness` | `number` | `1` | 0 … 4 | — | tunable | The toon bands' and highlight's edges, anti-aliased over this many pixels (lilToon's screen-space smoothing); 0 hard steps that only TAA softens. |

#### Outline placement

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `outlinePlace` | `OutlinePlace` | `"inside"` | — | — | — | Where outlines lie against an object's silhouette: inside (the zero value, its own edge pixels), outside (round it, at any width, anti-aliased) or both. Creases between objects stay inside. |

#### Wireframe

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `wireframe` | `boolean` | `false` | — | — | — | Draw every lit material's feature edges over its faces (StandardMaterialDesc.wireframe's; this style's angle and colour where it sets none), in the colour the surface shows along each edge or wireframeColor; the faces hide the edges behind them. A material's own wireframe draws its edges alone. See `looks §12`. |
| `wireframeAngle` | `number` | `30` | 0 … 180 | `deg` | — | The crease angle wireframe draws past (StandardMaterialDesc.wireframeAngle); 0 every triangle edge. |
| `wireframeColor` | `Color` | `{ r: 0, g: 0, b: 0 }` | — | — | — | The look's line colour for wireframe (StandardMaterialDesc.wireframeColor), linear, above 1 blooms; black (the zero value) draws each edge in the colour its surface shows there. |

```ogt
resource MaterialStyle {
  doc "Stylised defaults for a whole scene: toon shading and its ramp, a coloured shadow side, rim light, outlines, flat facets, made surfaces and wireframe. Each applies to a lit material that leaves that field at its own default; the terrain, grass, scattered copies, impostors and crowds take toon (built-in bands), shadowTint and shadowDarkness. Unlit and shader materials, StandardMaterialDesc.styled false, sprites and the UI are untouched. All off is no change. A look sets it; RenderCapabilities.stylise names what this renderer draws. Read after each tick; it never feeds the simulation."
  guide "looks §12"
  group "Toon"
  field toon bool "Cel shading for every lit material, as StandardMaterialDesc.toon: light quantised by the ramp, hard highlights. A material with a physical lobe (clearcoat, transmission and the rest) keeps its physical shading; set its own toon to make it toon anyway."
  field gradientMap Texture "The toon ramp for materials without their own (StandardMaterialDesc.gradientMap, og.load.texture): its first row maps the light's angle to the light received, read without filtering; 0 is three built-in bands."
  group "Shadow side"
  field shadowTint Color "The colour the shadowed side of a toon or lit surface moves towards instead of black (linear): the cool or warm shadow of illustration." {
    default 0.18 0.16 0.32
  }
  field shadowTintAmount f32 "How far shadowed light moves towards shadowTint; 0 is off." {
    min 0
    max 1
    unit ratio
    tunable true
  }
  field shadowDarkness f32 "How far shadowed surfaces fall towards black: where the sun does not reach, a surface loses this share of the sun's ramp and of the ambient light (1: solid black shadows). Point, spot and rect lights and the emissive still show inside it; shadowTint's light is added after it. 0 is off." {
    min 0
    max 1
    unit ratio
    tunable true
    guide "looks §12"
  }
  group "Rim light"
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
  group "Outlines"
  field outlineColor Color "The outline colour for materials without their own (linear)." {
    default 0.02 0.02 0.03
  }
  field outlineWidth f32 "The outline width in pixels at outlineDistance for materials without their own; 0 is none. A hull outline (StandardMaterialDesc.outlineWidth): silhouettes and creases against other objects, never the faces of one. A surface seen at a grazing angle is lined only where it ends." {
    min 0
    max 31
    tunable true
    guide "looks §12"
  }
  field outlineDistance f32 "The distance at which outlines are outlineWidth wide: nearer they grow, up to four times outlineWidth, farther they thin in proportion (an inked look keeps line weight by depth); 0 keeps every outline outlineWidth wide." {
    min 0
    unit m
  }
  group "Facets"
  field flatShading bool "Shade every lit material with face normals, as StandardMaterialDesc.flatShading: visible facets for low-poly, paper-craft and voxel looks."
  group "Made surface"
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
  group "Toon shading"
  field toonAmbient f32 "How smooth a toon material's ambient light (the Environment and its hemisphere light) and reflection stay: 0 one flat colour, the light from above and below averaged (MToon's equalised ambient), so the bands sit on flat colour; 1 the smooth PBR ambient and reflections." {
    guide "looks §12"
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
  group "Outline placement"
  field outlinePlace OutlinePlace "Where outlines lie against an object's silhouette: inside (the zero value, its own edge pixels), outside (round it, at any width, anti-aliased) or both. Creases between objects stay inside."
  group "Wireframe"
  field wireframe bool "Draw every lit material's feature edges over its faces (StandardMaterialDesc.wireframe's; this style's angle and colour where it sets none), in the colour the surface shows along each edge or wireframeColor; the faces hide the edges behind them. A material's own wireframe draws its edges alone." {
    guide "looks §12"
  }
  field wireframeAngle f32 "The crease angle wireframe draws past (StandardMaterialDesc.wireframeAngle); 0 every triangle edge." {
    default 30
    min 0
    max 180
    unit deg
  }
  field wireframeColor Color "The look's line colour for wireframe (StandardMaterialDesc.wireframeColor), linear, above 1 blooms; black (the zero value) draws each edge in the colour its surface shows there."
}
```

<sub>Module `render` line 1903</sub>
