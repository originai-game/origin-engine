# Run-time vector paths

`crates/og-render/src/ui/path.rs` and `sd_path` in
`crates/og-render/shaders/ui_common.wgsl` draw a path's signed distance per
pixel from its quadratic curves. Three permissive sources are ported:

| What | Source | Licence | Where |
|---|---|---|---|
| Curves sorted into horizontal bands; a fragment reads its band only. Root eligibility of a ray to +x against a quadratic (the `0x2E74` lookup on the signs of the three control points' y), which counts a crossing at a shared endpoint once and a tangent never. | Eric Lengyel, *GPU-Centered Font Rendering Directly from Glyph Outlines*, JCGT 6(2), 2017; reference shaders https://github.com/EricLengyel/Slug (`SlugPixelShader.hlsl`, revision of 2026-03-13). The patent was dedicated to the public domain in March 2026. | MIT or Apache-2.0 (`LICENSE-SLUG-MIT` below) | `write` (bands), `quad_crossings` |
| The exact distance to a quadratic Bézier: the real roots of its cubic, by Cardano. | Inigo Quilez, *Quadratic Bezier - exact* (Shadertoy MsyBzc, 2018), https://iquilezles.org/articles/distfunctions2d/ | MIT (`LICENSE-IQ` below) | `dist_quad` |
| A cubic as quadratics by the midpoint approximation, n pieces from the error bound √3/36 · \|p3 − 3p2 + 3p1 − p0\| / n³. | kurbo `CubicBez::to_quads`, https://github.com/linebender/kurbo | MIT or Apache-2.0 (`LICENSE-KURBO-MIT` below) | `cubic_to_quads` |

Changes from the sources: Slug counts coverage along two rays and blends
them; here the winding number of one ray signs the distance instead, so a
path takes every mode a shape has (strokes as a band around the outline,
shadows and glows through the same erf). Bands are cut over the curves' box
grown by what the instance draws past the outline, so every curve within
reach of a pixel is in its band. Lines are stored as quadratics with a flag
and measured as segments.

## LICENSE-SLUG-MIT

```
MIT License

Copyright (c) 2017 Eric Lengyel

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## LICENSE-IQ

```
The MIT License
Copyright © 2018 Inigo Quilez
Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions: The above copyright
notice and this permission notice shall be included in all copies or
substantial portions of the Software. THE SOFTWARE IS PROVIDED "AS IS",
WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO
THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE
FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR
THE USE OR OTHER DEALINGS IN THE SOFTWARE.
```

## LICENSE-KURBO-MIT

```
Copyright (c) 2018 Raph Levien

Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.
```
