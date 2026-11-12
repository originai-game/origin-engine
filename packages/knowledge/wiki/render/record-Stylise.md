<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-Stylise"></a>

### `og.Stylise` (resource)

Stylisation primitives that move the frame away from the photographic: a whole-number pixel grid, fewer tones or a fixed palette through a dither, halftone dots, cross-hatching, paint and watercolour, ink lines on tonal edges, colour bands by depth, and early 3D's vertex snapping, affine textures and coarse depth. Every primitive is off at its zero value (0, false, none) and then costs nothing. RenderCapabilities.stylise names the primitives this renderer draws; a field it does not name is carried and read back but not drawn. Read after each tick; it never feeds back into the simulation.

See `looks §6`.

Token `og.Stylise`, registered under the schema name `og.Stylise`; its value type is `Stylise`.

#### Pixel grid

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `pixelHeight` | `number` | `0` | 0 … 2160 | — | — | The logical resolution: the scene is rendered this many pixels tall and scaled up to the output by a whole number (pixelUpscale) with nearest filtering; 0 renders at the output's resolution. The width follows the output's aspect, at most the output's width over the scale. See `looks §7`. |
| `pixelUpscale` | `number` | `0` | 0 … 16 | — | — | The whole-number scale from the logical to the output resolution, a hard request: honoured whenever pixelHeight times it fits the output's height, else the largest that fits; 0 takes the largest that fits. A frame that does not fill the output is centred with Camera.background round it. See `looks §7`. |
| `pixelSnap` | `boolean` | `true` | — | — | — | Snap the camera and world sprites to the logical pixel grid, so a moving orthographic camera scrolls by whole pixels and sprites in a 3D world keep their edges (pixel sprites in 3D sets). |

#### Tones and palette

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `posterize` | `number` | `0` | 0 … 64 | — | — | Tones per colour channel after tone mapping (2 to 64): smooth gradients become flat steps; 0 is off. |
| `posterizeMode` | `PosterizeMode` | `"channel"` | — | — | — | How posterize steps a colour: each channel apart (channel), or its value and its tint apart so light never changes its hue (tone). |
| `palette` | `Palette` | `0` | — | — | — | Quantise every pixel to the nearest colour of this palette (og.load.palette), measured in a perceptual space after posterize; 0 is none. |
| `dither` | `DitherPattern` | `"none"` | — | — | — | How the steps of posterize and palette are mixed: none rounds every pixel, an ordered or blue-noise pattern alternates the two nearest steps in proportion. |
| `ditherStrength` | `number` | `1` | 0 … 2 | — | tunable | The dither's amplitude in steps: 1 spreads exactly one step, less keeps flatter bands. |
| `ditherScale` | `number` | `1` | 1 … 8 | — | — | The size of one dither cell in logical pixels. |

#### Halftone

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `halftone` | `number` | `0` | 0 … 1 | `ratio` | tunable | Halftone print: tones become dots on a rotated grid, larger where the image is darker; 0 is off, 1 dots only. |
| `halftoneSize` | `number` | `6` | 2 … 64 | — | — | The dot grid's pitch in logical pixels. |
| `halftoneAngle` | `number` | `45` | — | `deg` | — | The dot grid's angle. |
| `halftoneColor` | `Color` | `{ r: 0.02, g: 0.02, b: 0.03 }` | — | — | — | The ink of the dots (linear). |
| `halftoneSpace` | `StyliseSpace` | `"screen"` | — | — | — | Where the dots lie: screen, a grid over the frame; surface, grids on the surfaces that stay on them as the camera moves and wrap round forms (halftoneSize is then about the pitch in pixels where they lie); object, the same grids in each instance's own space, so a moving object carries its dots. |

#### Hatching

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `hatching` | `number` | `0` | 0 … 1 | `ratio` | tunable | Pencil or ink cross-hatching from a tonal art map: darker tones add strokes in more directions over a paper-light ground, each set thinning and fading in from clean paper at its tone; 0 is off, 1 strokes only. |
| `hatchingScale` | `number` | `6` | 2 … 64 | — | — | The spacing of hatch lines in logical pixels. |
| `hatchingLevels` | `number` | `4` | 1 … 6 | — | — | How many tones the hatching distinguishes, each adding one stroke direction or density (1 to 6). |
| `hatchingAngle` | `number` | `45` | — | `deg` | — | The direction of the first stroke set. |
| `hatchingColor` | `Color` | `{ r: 0.04, g: 0.035, b: 0.03 }` | — | — | — | The stroke colour (linear): graphite, sepia or ink. |
| `hatchingPaper` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The paper the strokes lie on (linear): one-ink hatching lifts the ground's light tones towards it, and under hatchingKeepColor the wash lies on the paper's colour at the wash's own lightness (a cream sketchbook, a grey card, a blue print); white keeps the frame's own. |
| `hatchingSpace` | `StyliseSpace` | `"screen"` | — | — | — | Where the strokes lie: screen, stripes over the frame that follow the tone alone; surface, strokes on the surfaces that follow the tone and the form and stay on them as the camera moves (hatchingScale is then about the spacing in pixels where they lie); object, the same in each instance's own space. |
| `hatchingDepth` | `number` | `0` | 0 … 1 | `ratio` | tunable | Hatching from depth as well as luminance: the depth-normal prepass's silhouettes and creases (inkDepthThreshold's and inkNormalThreshold's edges) darken the tone the strokes follow at this strength, so strokes gather along contours and folds. 0 is the tone alone; above 0 the frame runs the prepass. See `looks §9`. |

#### Paint

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `paintRadius` | `number` | `0` | 0 … 8 | — | tunable | Painterly smoothing (an anisotropic Kuwahara filter): flat regions and incoherent texture become strokes of one colour this many logical pixels across; edges and thin lines keep their shape; 0 is off. Under a profile 2d art contract the cook paints each texture and the frame draws neither pass. See `looks §10`. |
| `paintAnisotropy` | `number` | `0` | 0 … 1 | `ratio` | — | How far the strokes follow the image's structure: 0 round brush cells, 0.5 the filter's classic ellipses, 1 thin strokes along every coherent contour, which keep lines a pixel wide. |
| `paintEdgeDarkening` | `number` | `0` | 0 … 1 | `ratio` | tunable | Pigment pooling at the edges of colour regions, as wet watercolour dries darker at its rim; 0 is none. |
| `paintBleed` | `number` | `0` | 0 … 1 | `ratio` | tunable | Colour bleeding across edges into its neighbours, wet in wet; 0 is none. |
| `paintCanvas` | `Texture` | `0` | — | — | — | A tileable paper or canvas texture (og.load.texture) laid over the frame by paintCanvasStrength; 0 is none. |
| `paintCanvasStrength` | `number` | `0` | 0 … 1 | `ratio` | tunable | How much the canvas texture modulates the frame: 0 none, 1 its full grain. |
| `paintCanvasScale` | `number` | `1` | 0.125 … 8 | — | — | Canvas texels per logical pixel, with paintCanvasSpace screen (and on the sky). |
| `paintCanvasSpace` | `StyliseSpace` | `"screen"` | — | — | — | Where the canvas lies: screen, over the frame (the scene slides under the paper as the camera moves); surface, on the surfaces, so the grain sticks to the scene, foreshortened with the floor and walls; object, in each instance's own space, so a moving object carries its grain. |
| `paintCanvasWorldScale` | `number` | `128` | 1 … 4096 | — | — | Canvas texels per metre with paintCanvasSpace surface: 128 repeats a 256-texel canvas every two metres. |

#### Ink

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `ink` | `boolean` | `false` | — | — | — | Ink lines where the tone changes sharply (luminance edges), over every surface, sprite and texture alike: comic and storybook contours without PostEffects.outline's depth prepass. inkDepth adds the prepass's silhouettes and creases. |
| `inkThreshold` | `number` | `0.1` | 0 … 1 | — | tunable | The luminance step that counts as an edge (0 to 1 in display units): lower draws more lines. |
| `inkWidth` | `number` | `1` | 0.5 … 8 | — | tunable | The line width in logical pixels. |
| `inkColor` | `Color` | `{ r: 0.02, g: 0.02, b: 0.03 }` | — | — | — | The line colour (linear). |
| `inkDepth` | `number` | `0` | 0 … 1 | `ratio` | tunable | With ink: lines also where the depth-normal prepass breaks (silhouettes against what is behind, creases between faces), at this strength: comic contours round flat colours that luminance alone misses. 0 is luminance only; above 0 the frame runs the prepass PostEffects.outline uses. |
| `inkDepthThreshold` | `number` | `0.04` | ≥ 0 | — | — | With inkDepth: the relative depth step to a farther neighbour that counts as a silhouette (0.04 is 4 %), as PostEffects.outlineDepthThreshold: lower inks smaller steps. |
| `inkNormalThreshold` | `number` | `0.4` | 0 … 2 | — | — | With inkDepth: the normal change between neighbours of about the same depth that counts as a crease (1 - cos of the angle; 0.4 is about 53 degrees), as PostEffects.outlineNormalThreshold: lower inks gentler folds. |

#### Depth bands

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `depthBands` | `number` | `0` | 0 … 8 | — | — | Colour bands by depth (2 to 8): the scene from depthBandsNear to depthBandsFar is cut into this many equal layers, the first in depthBandsNearColor, the last in depthBandsFarColor, those between stepping evenly from one to the other in lightness, chroma and hue (never through grey); 0 is off. See `looks §11`. |
| `depthBandsNear` | `number` | `5` | ≥ 0 | `m` | — | Where the first band starts. |
| `depthBandsFar` | `number` | `60` | ≥ 0 | `m` | — | Where the last band ends; the last band (the far colour) starts one band's depth before it, and everything beyond it is the far colour too. |
| `depthBandsNearColor` | `Color` | `{ r: 0.02, g: 0.02, b: 0.03 }` | — | — | — | The nearest band's colour (linear, before exposure). |
| `depthBandsFarColor` | `Color` | `{ r: 0.55, g: 0.6, b: 0.7 }` | — | — | — | The farthest band's colour (linear, before exposure). |
| `depthBandsFlatten` | `number` | `1` | 0 … 1 | `ratio` | tunable | How far a band replaces the surfaces' own shading: 1 flat silhouettes in the band's colour; 0 a tint over the lit scene, its colours times the band's at unit luminance (a coloured filter: the shading, the value and the surfaces' own hues show through); between, a mix of the two. |

#### Early 3D

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `vertexSnap` | `number` | `0` | 0 … 16 | — | — | Snap every vertex of the 3D scene to a screen grid this many logical pixels wide, as fixed-point hardware did: geometry jitters by whole steps as it moves; 0 is off. |
| `affineTexturing` | `number` | `0` | 0 … 1 | `ratio` | — | How far texture coordinates are interpolated without perspective correction: 1 fully affine, so textures warp and swim across large polygons as on early consoles; 0 is off. |
| `depthBits` | `number` | `0` | 0 … 24 | — | — | Quantise depth to this many bits, for the coarse depth buffers of early 3D (12 to 16 are period-like: distant surfaces fight); 0 keeps full precision. |

#### Paint detail

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `paintDetail` | `number` | `1` | 0 … 1 | `ratio` | tunable | How far detail keeps its shape under paintRadius: a pixel's radius shrinks by its detail (its material's StandardMaterialDesc.paintDetail, or paintCharacterDetail on a skinned mesh) times this, and no stroke crosses an object's edge. Above 0 the frame runs the prepass; 0 paints every pixel alike. See `looks §10`. |
| `paintCharacterDetail` | `number` | `0.8` | 0 … 1 | `ratio` | tunable | The detail of a skinned mesh's pixels (a character's face and hands) where its material sets less: 0 painted as the rest, 1 kept as drawn. |

#### Watercolour

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `watercolorEdgeDarkening` | `number` | `0` | 0 … 1 | `ratio` | tunable | The wash's dark rim, drawn after paintRadius's strokes: pigment pooled two to three pixels wide at a wash's outer boundary (a colour step wider than a line, or a silhouette against something farther), the colour there raised to one plus the strength. Seams and creases get none; 0 is none. See `looks §10`. |
| `watercolorGranulation` | `number` | `0` | 0 … 1 | `ratio` | tunable | Pigment settling into the paper's grain: a density from the paper's relief (paintCanvas, else a noise) thins each colour towards watercolorPaper on the peaks and thickens it in the pits, weighed by the pigment load so the paper and pale washes stay clean; the wash keeps its mean value. 0 is none. See `looks §10`. |
| `watercolorTurbulence` | `number` | `0` | 0 … 1 | `ratio` | tunable | Pigment drifting as the wash dries: the same density from a slow noise about a hundred canvas texels across, laid where the canvas lies (paintCanvasSpace) and seeded by the object under the pixel so no blotch spans two washes, weighed by the pigment load; 0 is none. See `looks §10`. |
| `watercolorDryBrush` | `number` | `0` | 0 … 1 | `ratio` | tunable | A dry brush skipping over the paper's tooth: its peaks (the relief past half its spread, in the paper's own clusters) left bare as the paper (watercolorPaper) in the rim at a wash's outer boundary (watercolorEdgeDarkening's) and on the light-catching tones, the brightest washes first; 0 is none. |
| `watercolorWobble` | `number` | `0` | 0 … 4 | — | tunable | The painted frame displaced by the paper's relief, up to this many logical pixels, so every edge wavers with the tooth as a wash's does (MNPR's substrate distortion); 0 is none. |
| `watercolorPaper` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The paper's colour (linear): what the wash thins towards and the dry brush leaves bare. |

#### Dither placement

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `ditherSpace` | `StyliseSpace` | `"screen"` | — | — | — | Where the dither's cells lie: screen, a grid over the frame (it crawls as the camera moves); surface, cells on the surfaces at least ditherScale pixels on the screen each way, the sky's on a cube round the camera; object, in each mesh instance's own space. Off screen the frame runs the prepass. See `looks §8`. |

#### Halftone print

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `halftoneMode` | `HalftoneMode` | `"ink"` | — | — | — | How the halftone inks: one ink (halftoneColor) or four-colour process print (cmyk). |
| `halftoneMisregistration` | `number` | `0` | 0 … 8 | — | tunable | With halftoneMode cmyk: how far the cyan, magenta and yellow plates sit off the black one, in logical pixels, as a cheap print's colour slips past its line art; 0 is in register. |
| `halftoneRegion` | `HalftoneRegion` | `"all"` | — | — | — | Which tones the halftone prints on: all, the shade only or the highlights only (the darkest or lightest third of the tones). Under a region the dots are Ben-Day dots, a tint over the region's own colour at most 0.4 of a cell wide. They fade to flat ink on surfaces turned away from the camera. See `looks §9`. |

#### Colour hatching

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `hatchingKeepColor` | `boolean` | `false` | — | — | — | Colour hatching: the ground is a light wash of its own colour and each stroke is that colour darkened towards hatchingColor, so hue and chroma stay under the strokes; false lays strokes of hatchingColor over a paper-light ground. See `looks §9`. |

```ogt
resource Stylise {
  doc "Stylisation primitives that move the frame away from the photographic: a whole-number pixel grid, fewer tones or a fixed palette through a dither, halftone dots, cross-hatching, paint and watercolour, ink lines on tonal edges, colour bands by depth, and early 3D's vertex snapping, affine textures and coarse depth. Every primitive is off at its zero value (0, false, none) and then costs nothing. RenderCapabilities.stylise names the primitives this renderer draws; a field it does not name is carried and read back but not drawn. Read after each tick; it never feeds back into the simulation."
  guide "looks §6"
  group "Pixel grid"
  field pixelHeight u32 "The logical resolution: the scene is rendered this many pixels tall and scaled up to the output by a whole number (pixelUpscale) with nearest filtering; 0 renders at the output's resolution. The width follows the output's aspect, at most the output's width over the scale." {
    min 0
    max 2160
    guide "looks §7"
  }
  field pixelUpscale u32 "The whole-number scale from the logical to the output resolution, a hard request: honoured whenever pixelHeight times it fits the output's height, else the largest that fits; 0 takes the largest that fits. A frame that does not fill the output is centred with Camera.background round it." {
    min 0
    max 16
    guide "looks §7"
  }
  field pixelSnap bool "Snap the camera and world sprites to the logical pixel grid, so a moving orthographic camera scrolls by whole pixels and sprites in a 3D world keep their edges (pixel sprites in 3D sets)." {
    default true
  }
  group "Tones and palette"
  field posterize u32 "Tones per colour channel after tone mapping (2 to 64): smooth gradients become flat steps; 0 is off." {
    min 0
    max 64
  }
  field posterizeMode PosterizeMode "How posterize steps a colour: each channel apart (channel), or its value and its tint apart so light never changes its hue (tone)."
  field palette Palette "Quantise every pixel to the nearest colour of this palette (og.load.palette), measured in a perceptual space after posterize; 0 is none."
  field dither DitherPattern "How the steps of posterize and palette are mixed: none rounds every pixel, an ordered or blue-noise pattern alternates the two nearest steps in proportion."
  field ditherStrength f32 "The dither's amplitude in steps: 1 spreads exactly one step, less keeps flatter bands." {
    default 1
    min 0
    max 2
    tunable true
  }
  field ditherScale u32 "The size of one dither cell in logical pixels." {
    default 1
    min 1
    max 8
  }
  group "Halftone"
  field halftone f32 "Halftone print: tones become dots on a rotated grid, larger where the image is darker; 0 is off, 1 dots only." {
    min 0
    max 1
    unit ratio
    tunable true
  }
  field halftoneSize f32 "The dot grid's pitch in logical pixels." {
    default 6
    min 2
    max 64
  }
  field halftoneAngle f32 "The dot grid's angle." {
    default 45
    unit deg
  }
  field halftoneColor Color "The ink of the dots (linear)." {
    default 0.02 0.02 0.03
  }
  field halftoneSpace StyliseSpace "Where the dots lie: screen, a grid over the frame; surface, grids on the surfaces that stay on them as the camera moves and wrap round forms (halftoneSize is then about the pitch in pixels where they lie); object, the same grids in each instance's own space, so a moving object carries its dots."
  group "Hatching"
  field hatching f32 "Pencil or ink cross-hatching from a tonal art map: darker tones add strokes in more directions over a paper-light ground, each set thinning and fading in from clean paper at its tone; 0 is off, 1 strokes only." {
    min 0
    max 1
    unit ratio
    tunable true
  }
  field hatchingScale f32 "The spacing of hatch lines in logical pixels." {
    default 6
    min 2
    max 64
  }
  field hatchingLevels u32 "How many tones the hatching distinguishes, each adding one stroke direction or density (1 to 6)." {
    default 4
    min 1
    max 6
  }
  field hatchingAngle f32 "The direction of the first stroke set." {
    default 45
    unit deg
  }
  field hatchingColor Color "The stroke colour (linear): graphite, sepia or ink." {
    default 0.04 0.035 0.03
  }
  field hatchingPaper Color "The paper the strokes lie on (linear): one-ink hatching lifts the ground's light tones towards it, and under hatchingKeepColor the wash lies on the paper's colour at the wash's own lightness (a cream sketchbook, a grey card, a blue print); white keeps the frame's own." {
    default 1 1 1
  }
  field hatchingSpace StyliseSpace "Where the strokes lie: screen, stripes over the frame that follow the tone alone; surface, strokes on the surfaces that follow the tone and the form and stay on them as the camera moves (hatchingScale is then about the spacing in pixels where they lie); object, the same in each instance's own space."
  field hatchingDepth f32 "Hatching from depth as well as luminance: the depth-normal prepass's silhouettes and creases (inkDepthThreshold's and inkNormalThreshold's edges) darken the tone the strokes follow at this strength, so strokes gather along contours and folds. 0 is the tone alone; above 0 the frame runs the prepass." {
    min 0
    max 1
    unit ratio
    tunable true
    guide "looks §9"
  }
  group "Paint"
  field paintRadius f32 "Painterly smoothing (an anisotropic Kuwahara filter): flat regions and incoherent texture become strokes of one colour this many logical pixels across; edges and thin lines keep their shape; 0 is off. Under a profile 2d art contract the cook paints each texture and the frame draws neither pass." {
    min 0
    max 8
    tunable true
    guide "looks §10"
  }
  field paintAnisotropy f32 "How far the strokes follow the image's structure: 0 round brush cells, 0.5 the filter's classic ellipses, 1 thin strokes along every coherent contour, which keep lines a pixel wide." {
    min 0
    max 1
    unit ratio
  }
  field paintEdgeDarkening f32 "Pigment pooling at the edges of colour regions, as wet watercolour dries darker at its rim; 0 is none." {
    min 0
    max 1
    unit ratio
    tunable true
  }
  field paintBleed f32 "Colour bleeding across edges into its neighbours, wet in wet; 0 is none." {
    min 0
    max 1
    unit ratio
    tunable true
  }
  field paintCanvas Texture "A tileable paper or canvas texture (og.load.texture) laid over the frame by paintCanvasStrength; 0 is none."
  field paintCanvasStrength f32 "How much the canvas texture modulates the frame: 0 none, 1 its full grain." {
    min 0
    max 1
    unit ratio
    tunable true
  }
  field paintCanvasScale f32 "Canvas texels per logical pixel, with paintCanvasSpace screen (and on the sky)." {
    default 1
    min 0.125
    max 8
  }
  field paintCanvasSpace StyliseSpace "Where the canvas lies: screen, over the frame (the scene slides under the paper as the camera moves); surface, on the surfaces, so the grain sticks to the scene, foreshortened with the floor and walls; object, in each instance's own space, so a moving object carries its grain."
  field paintCanvasWorldScale f32 "Canvas texels per metre with paintCanvasSpace surface: 128 repeats a 256-texel canvas every two metres." {
    default 128
    min 1
    max 4096
  }
  group "Ink"
  field ink bool "Ink lines where the tone changes sharply (luminance edges), over every surface, sprite and texture alike: comic and storybook contours without PostEffects.outline's depth prepass. inkDepth adds the prepass's silhouettes and creases."
  field inkThreshold f32 "The luminance step that counts as an edge (0 to 1 in display units): lower draws more lines." {
    default 0.1
    min 0
    max 1
    tunable true
  }
  field inkWidth f32 "The line width in logical pixels." {
    default 1
    min 0.5
    max 8
    tunable true
  }
  field inkColor Color "The line colour (linear)." {
    default 0.02 0.02 0.03
  }
  field inkDepth f32 "With ink: lines also where the depth-normal prepass breaks (silhouettes against what is behind, creases between faces), at this strength: comic contours round flat colours that luminance alone misses. 0 is luminance only; above 0 the frame runs the prepass PostEffects.outline uses." {
    min 0
    max 1
    unit ratio
    tunable true
  }
  field inkDepthThreshold f32 "With inkDepth: the relative depth step to a farther neighbour that counts as a silhouette (0.04 is 4 %), as PostEffects.outlineDepthThreshold: lower inks smaller steps." {
    default 0.04
    min 0
  }
  field inkNormalThreshold f32 "With inkDepth: the normal change between neighbours of about the same depth that counts as a crease (1 - cos of the angle; 0.4 is about 53 degrees), as PostEffects.outlineNormalThreshold: lower inks gentler folds." {
    default 0.4
    min 0
    max 2
  }
  group "Depth bands"
  field depthBands u32 "Colour bands by depth (2 to 8): the scene from depthBandsNear to depthBandsFar is cut into this many equal layers, the first in depthBandsNearColor, the last in depthBandsFarColor, those between stepping evenly from one to the other in lightness, chroma and hue (never through grey); 0 is off." {
    min 0
    max 8
    guide "looks §11"
  }
  field depthBandsNear f32 "Where the first band starts." {
    default 5
    min 0
    unit m
  }
  field depthBandsFar f32 "Where the last band ends; the last band (the far colour) starts one band's depth before it, and everything beyond it is the far colour too." {
    default 60
    min 0
    unit m
  }
  field depthBandsNearColor Color "The nearest band's colour (linear, before exposure)." {
    default 0.02 0.02 0.03
  }
  field depthBandsFarColor Color "The farthest band's colour (linear, before exposure)." {
    default 0.55 0.6 0.7
  }
  field depthBandsFlatten f32 "How far a band replaces the surfaces' own shading: 1 flat silhouettes in the band's colour; 0 a tint over the lit scene, its colours times the band's at unit luminance (a coloured filter: the shading, the value and the surfaces' own hues show through); between, a mix of the two." {
    default 1
    min 0
    max 1
    unit ratio
    tunable true
  }
  group "Early 3D"
  field vertexSnap f32 "Snap every vertex of the 3D scene to a screen grid this many logical pixels wide, as fixed-point hardware did: geometry jitters by whole steps as it moves; 0 is off." {
    min 0
    max 16
  }
  field affineTexturing f32 "How far texture coordinates are interpolated without perspective correction: 1 fully affine, so textures warp and swim across large polygons as on early consoles; 0 is off." {
    min 0
    max 1
    unit ratio
  }
  field depthBits u32 "Quantise depth to this many bits, for the coarse depth buffers of early 3D (12 to 16 are period-like: distant surfaces fight); 0 keeps full precision." {
    min 0
    max 24
  }
  group "Paint detail"
  field paintDetail f32 "How far detail keeps its shape under paintRadius: a pixel's radius shrinks by its detail (its material's StandardMaterialDesc.paintDetail, or paintCharacterDetail on a skinned mesh) times this, and no stroke crosses an object's edge. Above 0 the frame runs the prepass; 0 paints every pixel alike." {
    guide "looks §10"
    default 1
    min 0
    max 1
    unit ratio
    tunable true
  }
  field paintCharacterDetail f32 "The detail of a skinned mesh's pixels (a character's face and hands) where its material sets less: 0 painted as the rest, 1 kept as drawn." {
    default 0.8
    min 0
    max 1
    unit ratio
    tunable true
  }
  group "Watercolour"
  field watercolorEdgeDarkening f32 "The wash's dark rim, drawn after paintRadius's strokes: pigment pooled two to three pixels wide at a wash's outer boundary (a colour step wider than a line, or a silhouette against something farther), the colour there raised to one plus the strength. Seams and creases get none; 0 is none." {
    min 0
    max 1
    unit ratio
    tunable true
    guide "looks §10"
  }
  field watercolorGranulation f32 "Pigment settling into the paper's grain: a density from the paper's relief (paintCanvas, else a noise) thins each colour towards watercolorPaper on the peaks and thickens it in the pits, weighed by the pigment load so the paper and pale washes stay clean; the wash keeps its mean value. 0 is none." {
    min 0
    max 1
    unit ratio
    tunable true
    guide "looks §10"
  }
  field watercolorTurbulence f32 "Pigment drifting as the wash dries: the same density from a slow noise about a hundred canvas texels across, laid where the canvas lies (paintCanvasSpace) and seeded by the object under the pixel so no blotch spans two washes, weighed by the pigment load; 0 is none." {
    guide "looks §10"
    min 0
    max 1
    unit ratio
    tunable true
  }
  field watercolorDryBrush f32 "A dry brush skipping over the paper's tooth: its peaks (the relief past half its spread, in the paper's own clusters) left bare as the paper (watercolorPaper) in the rim at a wash's outer boundary (watercolorEdgeDarkening's) and on the light-catching tones, the brightest washes first; 0 is none." {
    min 0
    max 1
    unit ratio
    tunable true
  }
  field watercolorWobble f32 "The painted frame displaced by the paper's relief, up to this many logical pixels, so every edge wavers with the tooth as a wash's does (MNPR's substrate distortion); 0 is none." {
    min 0
    max 4
    tunable true
  }
  field watercolorPaper Color "The paper's colour (linear): what the wash thins towards and the dry brush leaves bare." {
    default 1 1 1
  }
  group "Dither placement"
  field ditherSpace StyliseSpace "Where the dither's cells lie: screen, a grid over the frame (it crawls as the camera moves); surface, cells on the surfaces at least ditherScale pixels on the screen each way, the sky's on a cube round the camera; object, in each mesh instance's own space. Off screen the frame runs the prepass." {
    guide "looks §8"
  }
  group "Halftone print"
  field halftoneMode HalftoneMode "How the halftone inks: one ink (halftoneColor) or four-colour process print (cmyk)."
  field halftoneMisregistration f32 "With halftoneMode cmyk: how far the cyan, magenta and yellow plates sit off the black one, in logical pixels, as a cheap print's colour slips past its line art; 0 is in register." {
    min 0
    max 8
    tunable true
  }
  field halftoneRegion HalftoneRegion "Which tones the halftone prints on: all, the shade only or the highlights only (the darkest or lightest third of the tones). Under a region the dots are Ben-Day dots, a tint over the region's own colour at most 0.4 of a cell wide. They fade to flat ink on surfaces turned away from the camera." {
    guide "looks §9"
  }
  group "Colour hatching"
  field hatchingKeepColor bool "Colour hatching: the ground is a light wash of its own colour and each stroke is that colour darkened towards hatchingColor, so hue and chroma stay under the strokes; false lays strokes of hatchingColor over a paper-light ground." {
    guide "looks §9"
  }
}
```

<sub>Module `render` line 1615</sub>
