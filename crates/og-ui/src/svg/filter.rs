//! SVG `<filter>` (Filter Effects 1 §9-15) as og-render's filter graph
//! ([`UiFilter`]): a definition read once ([`parse`]) into primitives in
//! its primitive units, then resolved for an element's bounding box
//! ([`Filter::resolve`]) into user space and mapped to CSS px ([`map`]).
//! An element's filter draws its subtree into a layer over the filter
//! region, which composites the graph's last result.
//!
//! Each primitive's `in`/`in2` names `SourceGraphic`, `SourceAlpha` or an
//! earlier `result`; `BackgroundImage`, `BackgroundAlpha`, `FillPaint` and
//! `StrokePaint` read as transparent black. `feMerge` is a chain of
//! over steps and `feDropShadow` its blur, offset, flood, composite and
//! merge, as the specification defines it.

use crate::element::Affine;
use crate::layout::Rect;
use og_core::glam::Vec3;
use og_render::{
    TextureHandle, UiFilter, UiFilterComposite, UiFilterInput, UiFilterOp, UiFilterPrimitive,
    UiLight, UiTransfer,
};

/// What finds an `feImage`'s `href`: its texture and uv rect.
pub type FindImage<'a> = &'a dyn Fn(&str) -> Result<(TextureHandle, [f32; 4]), String>;

/// A parsed `<filter>`: its region (`[x, y, width, height]`, fractions of
/// the bounding box when `region_bbox`, else user units) and its
/// primitives, whose lengths and subregions are in primitive units
/// (fractions of the box when `primitive_bbox`); a subregion's NaN is
/// left to its default.
#[derive(Clone, Debug, PartialEq)]
pub struct Filter {
    pub region: [f32; 4],
    pub region_bbox: bool,
    pub primitive_bbox: bool,
    pub primitives: Vec<UiFilterPrimitive>,
}

/// An element of a filter as the document holds it: its tag, attributes
/// and children.
pub struct Node<'a> {
    pub tag: &'a str,
    pub attrs: Vec<(String, String)>,
    pub children: Vec<Node<'a>>,
}

impl Node<'_> {
    fn get(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.trim())
    }

    fn num(&self, name: &str, default: f32) -> Result<f32, String> {
        match self.get(name) {
            None => Ok(default),
            Some(v) => number(v).map_err(|e| format!("<{}> {name}: {e}", self.tag)),
        }
    }

    fn nums(&self, name: &str) -> Result<Vec<f32>, String> {
        self.get(name).map_or(Ok(Vec::new()), |v| {
            numbers(v).map_err(|e| format!("<{}> {name}: {e}", self.tag))
        })
    }

    /// One or two numbers (`stdDeviation`, `radius`, `baseFrequency`):
    /// the second is the first when left out.
    fn pair(&self, name: &str, default: f32) -> Result<[f32; 2], String> {
        match self.nums(name)?.as_slice() {
            [] => Ok([default; 2]),
            [a] => Ok([*a; 2]),
            [a, b] => Ok([*a, *b]),
            _ => Err(format!("<{}> {name}: one or two numbers", self.tag)),
        }
    }
}

fn number(v: &str) -> Result<f32, String> {
    let v = v.trim();
    let (n, k) = match v.strip_suffix('%') {
        Some(p) => (p, 0.01),
        None => (v, 1.0),
    };
    n.trim()
        .parse::<f32>()
        .map(|x| x * k)
        .map_err(|_| format!("'{v}' is not a number"))
}

fn numbers(v: &str) -> Result<Vec<f32>, String> {
    v.split(|c: char| c.is_whitespace() || c == ',')
        .filter(|s| !s.is_empty())
        .map(number)
        .collect()
}

/// sRGB channel 0–1 to linear light.
fn linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// A `<filter>` and its primitives. `viewport` is what a user-unit
/// percentage is of (the `<svg>`'s view box size); `image` finds an
/// `feImage`'s `href`.
pub fn parse(filter: &Node, viewport: [f32; 2], image: FindImage) -> Result<Filter, String> {
    let region_bbox = filter.get("filterUnits") != Some("userSpaceOnUse");
    let primitive_bbox = filter.get("primitiveUnits") == Some("objectBoundingBox");
    let srgb = |n: &Node, parent: bool| match n.get("color-interpolation-filters") {
        Some("sRGB") => true,
        Some("linearRGB") => false,
        _ => parent,
    };
    let filter_srgb = srgb(filter, false);
    // A box's x, y, width or height: a fraction of the box, or user units
    // (a percentage of the viewport).
    let edge = |n: &Node, name: &str, k: usize, bbox: bool| -> Result<Option<f32>, String> {
        let Some(v) = n.get(name) else {
            return Ok(None);
        };
        let x = number(v).map_err(|e| format!("<{}> {name}: {e}", n.tag))?;
        Ok(Some(if bbox || !v.ends_with('%') {
            x
        } else {
            x * viewport[k % 2]
        }))
    };
    let mut region = [-0.1, -0.1, 1.2, 1.2];
    for (k, name) in ["x", "y", "width", "height"].into_iter().enumerate() {
        let default = if region_bbox {
            region[k]
        } else {
            region[k] * viewport[k % 2]
        };
        region[k] = edge(filter, name, k, region_bbox)?.unwrap_or(default);
    }
    let mut out = Graph {
        primitives: Vec::new(),
        names: Vec::new(),
        last: None,
        clear: None,
    };
    for n in &filter.children {
        let mut sub = [f32::NAN; 4];
        for (k, name) in ["x", "y", "width", "height"].into_iter().enumerate() {
            if let Some(v) = edge(n, name, k, primitive_bbox)? {
                sub[k] = v;
            }
        }
        let srgb = srgb(n, filter_srgb);
        let in1 = out.input(n.get("in"), sub, srgb);
        let in2 = out.input(n.get("in2"), sub, srgb);
        let at = Prim { sub, srgb };
        let result = primitive(n, &mut out, at, in1, in2, image)?;
        out.last = Some(result);
        if let Some(name) = n.get("result").filter(|r| !r.is_empty()) {
            out.names.push((name.to_string(), result));
        }
    }
    Ok(Filter {
        region,
        region_bbox,
        primitive_bbox,
        primitives: out.primitives,
    })
}

/// A primitive's subregion (primitive units, NaN: default) and space.
#[derive(Clone, Copy)]
struct Prim {
    sub: [f32; 4],
    srgb: bool,
}

/// The graph as it is read: its primitives, the results named so far, the
/// last result and the transparent one.
struct Graph {
    primitives: Vec<UiFilterPrimitive>,
    names: Vec<(String, UiFilterInput)>,
    last: Option<UiFilterInput>,
    clear: Option<UiFilterInput>,
}

impl Graph {
    fn push(&mut self, op: UiFilterOp, inputs: [UiFilterInput; 2], at: Prim) -> UiFilterInput {
        self.primitives.push(UiFilterPrimitive {
            op,
            inputs,
            subregion: at.sub,
            srgb: at.srgb,
        });
        UiFilterInput::Result(self.primitives.len() as u32 - 1)
    }

    /// What `in` names; left out (or naming no result), the last result,
    /// else the source.
    fn input(&mut self, name: Option<&str>, sub: [f32; 4], srgb: bool) -> UiFilterInput {
        match name {
            Some("SourceGraphic") => UiFilterInput::Source,
            Some("SourceAlpha") => UiFilterInput::SourceAlpha,
            Some("BackgroundImage" | "BackgroundAlpha" | "FillPaint" | "StrokePaint") => {
                self.clear(sub, srgb)
            }
            Some(n) => self
                .names
                .iter()
                .rev()
                .find(|(k, _)| k == n)
                .map(|(_, r)| *r)
                .or(self.last)
                .unwrap_or_default(),
            None => self.last.unwrap_or_default(),
        }
    }

    fn clear(&mut self, sub: [f32; 4], srgb: bool) -> UiFilterInput {
        if let Some(c) = self.clear {
            return c;
        }
        let c = self.push(
            UiFilterOp::Flood { color: [0.0; 4] },
            [UiFilterInput::Source; 2],
            Prim { sub, srgb },
        );
        self.clear = Some(c);
        c
    }
}

/// A colour attribute (`flood-color`, `lighting-color`) with its opacity
/// attribute, straight sRGB.
fn color(
    n: &Node,
    name: &str,
    opacity: Option<&str>,
    default: [f32; 4],
) -> Result<[f32; 4], String> {
    let mut c = match n.get(name) {
        Some(v) => super::color(v).map_err(|e| format!("<{}> {name}: {e}", n.tag))?,
        None => default,
    };
    if let Some(o) = opacity {
        c[3] *= n.num(o, 1.0)?.clamp(0.0, 1.0);
    }
    Ok(c)
}

fn flood(c: [f32; 4]) -> [f32; 4] {
    [linear(c[0]), linear(c[1]), linear(c[2]), c[3]]
}

/// One primitive into the graph; its result.
fn primitive(
    n: &Node,
    g: &mut Graph,
    at: Prim,
    in1: UiFilterInput,
    in2: UiFilterInput,
    image: FindImage,
) -> Result<UiFilterInput, String> {
    let tag = n.tag;
    let one = [in1, UiFilterInput::Source];
    let op = match tag {
        "feGaussianBlur" => UiFilterOp::Blur {
            std_dev: n.pair("stdDeviation", 0.0)?.map(|v| v.max(0.0)),
        },
        "feOffset" => UiFilterOp::Offset {
            d: [n.num("dx", 0.0)?, n.num("dy", 0.0)?],
        },
        "feFlood" => UiFilterOp::Flood {
            color: flood(color(
                n,
                "flood-color",
                Some("flood-opacity"),
                [0.0, 0.0, 0.0, 1.0],
            )?),
        },
        "feComposite" => {
            let op = match n.get("operator").unwrap_or("over") {
                "over" => UiFilterComposite::Over,
                "in" => UiFilterComposite::In,
                "out" => UiFilterComposite::Out,
                "atop" => UiFilterComposite::Atop,
                "xor" => UiFilterComposite::Xor,
                "lighter" => UiFilterComposite::Lighter,
                "arithmetic" => UiFilterComposite::Arithmetic,
                o => {
                    return Err(format!(
                        "<feComposite> operator \"{o}\": over, in, out, atop, xor, lighter or arithmetic"
                    ));
                }
            };
            let k = [
                n.num("k1", 0.0)?,
                n.num("k2", 0.0)?,
                n.num("k3", 0.0)?,
                n.num("k4", 0.0)?,
            ];
            return Ok(g.push(UiFilterOp::Composite { op, k }, [in1, in2], at));
        }
        "feBlend" => {
            let mode = n.get("mode").unwrap_or("normal");
            let mode =
                crate::doc::paint_css::blend(mode).map_err(|e| format!("<feBlend> mode: {e}"))?;
            return Ok(g.push(UiFilterOp::Blend(mode), [in1, in2], at));
        }
        "feColorMatrix" => UiFilterOp::ColorMatrix(color_matrix(n)?),
        "feComponentTransfer" => {
            let mut funcs: [UiTransfer; 4] = Default::default();
            for f in &n.children {
                let ch = match f.tag {
                    "feFuncR" => 0,
                    "feFuncG" => 1,
                    "feFuncB" => 2,
                    "feFuncA" => 3,
                    _ => continue,
                };
                funcs[ch] = transfer(f)?;
            }
            UiFilterOp::ComponentTransfer(Box::new(funcs))
        }
        "feMorphology" => UiFilterOp::Morphology {
            radius: n.pair("radius", 0.0)?.map(|v| v.max(0.0)),
            dilate: match n.get("operator").unwrap_or("erode") {
                "erode" => false,
                "dilate" => true,
                o => return Err(format!("<feMorphology> operator \"{o}\": erode or dilate")),
            },
        },
        "feDisplacementMap" => {
            let channel = |name: &str| -> Result<u8, String> {
                Ok(match n.get(name).unwrap_or("A") {
                    "R" => 0,
                    "G" => 1,
                    "B" => 2,
                    "A" => 3,
                    c => return Err(format!("<feDisplacementMap> {name} \"{c}\": R, G, B or A")),
                })
            };
            let op = UiFilterOp::Displacement {
                scale: n.num("scale", 0.0)?,
                channels: [channel("xChannelSelector")?, channel("yChannelSelector")?],
            };
            return Ok(g.push(op, [in1, in2], at));
        }
        "feTurbulence" => {
            let f = n.pair("baseFrequency", 0.0)?;
            if f[0] < 0.0 || f[1] < 0.0 {
                return Err("<feTurbulence> baseFrequency is not negative".into());
            }
            UiFilterOp::Turbulence {
                base_frequency: f,
                octaves: n.num("numOctaves", 1.0)?.clamp(0.0, 24.0) as u32,
                seed: n.num("seed", 0.0)?,
                stitch: n.get("stitchTiles") == Some("stitch"),
                fractal: match n.get("type").unwrap_or("turbulence") {
                    "turbulence" => false,
                    "fractalNoise" => true,
                    t => {
                        return Err(format!(
                            "<feTurbulence> type \"{t}\": turbulence or fractalNoise"
                        ));
                    }
                },
                user: [0.0, 0.0, 1.0],
                tile: [0.0; 4],
            }
        }
        "feImage" => {
            let href = n.get("href").unwrap_or_default();
            if href.starts_with('#') {
                return Err("<feImage> href names an image (an atlas name or a texture handle), not an element".into());
            }
            let (texture, uv) = image(href).map_err(|e| format!("<feImage> {e}"))?;
            UiFilterOp::Image {
                texture,
                uv,
                rect: [f32::NAN; 4],
            }
        }
        "feTile" => UiFilterOp::Tile { tile: [0.0; 4] },
        "feConvolveMatrix" => {
            let order = match n.nums("order")?.as_slice() {
                [] => [3, 3],
                [a] => [*a as u32; 2],
                [a, b] => [*a as u32, *b as u32],
                _ => return Err("<feConvolveMatrix> order: one or two numbers".into()),
            };
            let kernel = n.nums("kernelMatrix")?;
            let cells = (order[0] * order[1]) as usize;
            if order[0] == 0 || order[1] == 0 || kernel.len() != cells || cells > 64 {
                return Err(format!(
                    "<feConvolveMatrix> kernelMatrix: order's {} × {} values (at most 64)",
                    order[0], order[1]
                ));
            }
            let sum: f32 = kernel.iter().sum();
            let target = |name: &str, k: usize| -> Result<i32, String> {
                let t = n.num(name, (order[k] / 2) as f32)? as i32;
                if t < 0 || t >= order[k] as i32 {
                    return Err(format!("<feConvolveMatrix> {name} is within the order"));
                }
                Ok(t)
            };
            UiFilterOp::Convolve {
                order,
                divisor: n.num("divisor", if sum == 0.0 { 1.0 } else { sum })?,
                bias: n.num("bias", 0.0)?,
                target: [target("targetX", 0)?, target("targetY", 1)?],
                edge: match n.get("edgeMode").unwrap_or("duplicate") {
                    "duplicate" => 0,
                    "wrap" => 1,
                    "none" => 2,
                    e => {
                        return Err(format!(
                            "<feConvolveMatrix> edgeMode \"{e}\": duplicate, wrap or none"
                        ));
                    }
                },
                preserve_alpha: n.get("preserveAlpha") == Some("true"),
                kernel,
            }
        }
        "feDiffuseLighting" | "feSpecularLighting" => {
            let specular = tag == "feSpecularLighting";
            let c = color(n, "lighting-color", None, [1.0; 4])?;
            let light = n
                .children
                .iter()
                .find_map(|l| light(l).transpose())
                .transpose()?
                .ok_or_else(|| {
                    format!("<{tag}> holds a feDistantLight, fePointLight or feSpotLight")
                })?;
            UiFilterOp::Lighting {
                specular,
                surface_scale: n.num("surfaceScale", 1.0)?,
                constant: if specular {
                    n.num("specularConstant", 1.0)?
                } else {
                    n.num("diffuseConstant", 1.0)?
                },
                exponent: n.num("specularExponent", 1.0)?.clamp(1.0, 128.0),
                // In the primitive's space.
                color: if at.srgb {
                    [c[0], c[1], c[2]]
                } else {
                    [linear(c[0]), linear(c[1]), linear(c[2])]
                },
                light,
            }
        }
        "feMerge" => {
            let mut nodes = Vec::new();
            for m in n.children.iter().filter(|m| m.tag == "feMergeNode") {
                let i = g.input(m.get("in"), at.sub, at.srgb);
                nodes.push(i);
            }
            return Ok(match nodes.as_slice() {
                [] => g.clear(at.sub, at.srgb),
                [only] => g.push(
                    UiFilterOp::Offset { d: [0.0; 2] },
                    [*only, UiFilterInput::Source],
                    at,
                ),
                [first, rest @ ..] => {
                    let mut under = *first;
                    for &over in rest {
                        under = g.push(UiFilterOp::Merge, [over, under], at);
                    }
                    under
                }
            });
        }
        "feDropShadow" => {
            let std_dev = n.pair("stdDeviation", 2.0)?.map(|v| v.max(0.0));
            let d = [n.num("dx", 2.0)?, n.num("dy", 2.0)?];
            let c = flood(color(
                n,
                "flood-color",
                Some("flood-opacity"),
                [0.0, 0.0, 0.0, 1.0],
            )?);
            let blur = g.push(UiFilterOp::Blur { std_dev }, one, at);
            let moved = g.push(UiFilterOp::Offset { d }, [blur, UiFilterInput::Source], at);
            let fill = g.push(UiFilterOp::Flood { color: c }, one, at);
            let shadow = g.push(
                UiFilterOp::Composite {
                    op: UiFilterComposite::In,
                    k: [0.0; 4],
                },
                [fill, moved],
                at,
            );
            return Ok(g.push(UiFilterOp::Merge, [in1, shadow], at));
        }
        t => {
            return Err(format!(
                "<{t}> is not a filter primitive: feBlend, feColorMatrix, feComponentTransfer, feComposite, feConvolveMatrix, feDiffuseLighting, feDisplacementMap, feDropShadow, feFlood, feGaussianBlur, feImage, feMerge, feMorphology, feOffset, feSpecularLighting, feTile or feTurbulence"
            ));
        }
    };
    Ok(g.push(op, one, at))
}

/// `feColorMatrix`'s matrix (§15.10), rows R, G, B, A of five.
fn color_matrix(n: &Node) -> Result<[f32; 20], String> {
    let values = n.nums("values")?;
    let mut m = [0.0; 20];
    let one = |k: f32| -> Result<f32, String> {
        match values.as_slice() {
            [] => Ok(k),
            [v] => Ok(*v),
            _ => Err("<feColorMatrix> values: one number".into()),
        }
    };
    let m3 = |r: [[f32; 3]; 3]| {
        let mut m = [0.0; 20];
        for (i, row) in r.iter().enumerate() {
            m[i * 5..i * 5 + 3].copy_from_slice(row);
        }
        m[18] = 1.0;
        m
    };
    match n.get("type").unwrap_or("matrix") {
        "matrix" => match values.len() {
            0 => {
                for i in 0..4 {
                    m[i * 6] = 1.0;
                }
            }
            20 => m.copy_from_slice(&values),
            k => return Err(format!("<feColorMatrix> values: 20 numbers, not {k}")),
        },
        "saturate" => {
            let s = one(1.0)?;
            m = m3([
                [0.213 + 0.787 * s, 0.715 - 0.715 * s, 0.072 - 0.072 * s],
                [0.213 - 0.213 * s, 0.715 + 0.285 * s, 0.072 - 0.072 * s],
                [0.213 - 0.213 * s, 0.715 - 0.715 * s, 0.072 + 0.928 * s],
            ]);
        }
        "hueRotate" => {
            let (sn, c) = og_core::math::sin_cos(one(0.0)?.to_radians());
            m = m3([
                [
                    0.213 + c * 0.787 - sn * 0.213,
                    0.715 - c * 0.715 - sn * 0.715,
                    0.072 - c * 0.072 + sn * 0.928,
                ],
                [
                    0.213 - c * 0.213 + sn * 0.143,
                    0.715 + c * 0.285 + sn * 0.140,
                    0.072 - c * 0.072 - sn * 0.283,
                ],
                [
                    0.213 - c * 0.213 - sn * 0.787,
                    0.715 - c * 0.715 + sn * 0.715,
                    0.072 + c * 0.928 + sn * 0.072,
                ],
            ]);
        }
        "luminanceToAlpha" => {
            m[15] = 0.2125;
            m[16] = 0.7154;
            m[17] = 0.0721;
        }
        t => {
            return Err(format!(
                "<feColorMatrix> type \"{t}\": matrix, saturate, hueRotate or luminanceToAlpha"
            ));
        }
    }
    Ok(m)
}

/// A `feFunc*` (§15.11). Tables keep their first 16 values.
fn transfer(f: &Node) -> Result<UiTransfer, String> {
    let values = || -> Result<Vec<f32>, String> {
        let mut v = f.nums("tableValues")?;
        v.truncate(16);
        Ok(v)
    };
    Ok(match f.get("type").unwrap_or("identity") {
        "identity" => UiTransfer::Identity,
        "table" => match values()? {
            v if v.is_empty() => UiTransfer::Identity,
            v => UiTransfer::Table(v),
        },
        "discrete" => match values()? {
            v if v.is_empty() => UiTransfer::Identity,
            v => UiTransfer::Discrete(v),
        },
        "linear" => UiTransfer::Linear {
            slope: f.num("slope", 1.0)?,
            intercept: f.num("intercept", 0.0)?,
        },
        "gamma" => UiTransfer::Gamma {
            amplitude: f.num("amplitude", 1.0)?,
            exponent: f.num("exponent", 1.0)?,
            offset: f.num("offset", 0.0)?,
        },
        t => {
            return Err(format!(
                "<{}> type \"{t}\": identity, table, discrete, linear or gamma",
                f.tag
            ));
        }
    })
}

/// A light source element, in primitive units; `None` for another tag.
fn light(l: &Node) -> Result<Option<UiLight>, String> {
    let at = |l: &Node| -> Result<Vec3, String> {
        Ok(Vec3::new(
            l.num("x", 0.0)?,
            l.num("y", 0.0)?,
            l.num("z", 0.0)?,
        ))
    };
    Ok(Some(match l.tag {
        "feDistantLight" => UiLight::Distant {
            azimuth: l.num("azimuth", 0.0)?.to_radians(),
            elevation: l.num("elevation", 0.0)?.to_radians(),
        },
        "fePointLight" => UiLight::Point { at: at(l)? },
        "feSpotLight" => UiLight::Spot {
            at: at(l)?,
            points_at: Vec3::new(
                l.num("pointsAtX", 0.0)?,
                l.num("pointsAtY", 0.0)?,
                l.num("pointsAtZ", 0.0)?,
            ),
            exponent: l.num("specularExponent", 1.0)?,
            cone: match l.get("limitingConeAngle") {
                Some(_) => Some(l.num("limitingConeAngle", 90.0)?.abs().to_radians()),
                None => None,
            },
        },
        _ => return Ok(None),
    }))
}

impl Filter {
    /// The filter for an element whose bounding box is `bbox` (`[x, y,
    /// width, height]`, user units): its region and graph in user units.
    pub fn resolve(&self, bbox: [f32; 4]) -> (Rect, UiFilter) {
        let [bx, by, bw, bh] = bbox;
        let to_user = |r: [f32; 4], on: bool| {
            if on {
                [bx + r[0] * bw, by + r[1] * bh, r[2] * bw, r[3] * bh]
            } else {
                r
            }
        };
        let region = to_user(self.region, self.region_bbox);
        let units = if self.primitive_bbox {
            Affine {
                a: bw,
                d: bh,
                e: bx,
                f: by,
                ..Affine::IDENTITY
            }
        } else {
            Affine::IDENTITY
        };
        let mut subs: Vec<[f32; 4]> = Vec::with_capacity(self.primitives.len());
        let mut primitives = Vec::with_capacity(self.primitives.len());
        for p in &self.primitives {
            let mut p = p.clone();
            let given = to_user(
                p.subregion.map(|v| if v.is_nan() { 0.0 } else { v }),
                self.primitive_bbox,
            );
            // Left out: the union of its inputs' subregions, or the region.
            let of = |i: UiFilterInput| match i {
                UiFilterInput::Result(k) => subs.get(k as usize).copied().unwrap_or(region),
                _ => region,
            };
            let reads = match &p.op {
                // A tile fills the region (as browsers do), not its cell.
                UiFilterOp::Flood { .. }
                | UiFilterOp::Turbulence { .. }
                | UiFilterOp::Image { .. }
                | UiFilterOp::Tile { .. } => 0,
                UiFilterOp::Composite { .. }
                | UiFilterOp::Blend(_)
                | UiFilterOp::Displacement { .. }
                | UiFilterOp::Merge => 2,
                _ => 1,
            };
            let default = p.inputs[..reads]
                .iter()
                .map(|i| of(*i))
                .reduce(|a, b| {
                    let x = a[0].min(b[0]);
                    let y = a[1].min(b[1]);
                    [
                        x,
                        y,
                        (a[0] + a[2]).max(b[0] + b[2]) - x,
                        (a[1] + a[3]).max(b[1] + b[3]) - y,
                    ]
                })
                .unwrap_or(region);
            let mut sub = given;
            for k in 0..4 {
                if p.subregion[k].is_nan() {
                    sub[k] = default[k];
                }
            }
            map_op(&mut p.op, &units);
            match &mut p.op {
                UiFilterOp::Tile { tile } => *tile = of(p.inputs[0]),
                UiFilterOp::Image { rect, .. } => *rect = sub,
                UiFilterOp::Turbulence { user, tile, .. } => {
                    // The stitch tile is its subregion, in primitive units.
                    let k = user[2];
                    *tile = [
                        (sub[0] - user[0]) * k,
                        (sub[1] - user[1]) * k,
                        sub[2] * k,
                        sub[3] * k,
                    ];
                }
                _ => {}
            }
            p.subregion = sub;
            subs.push(sub);
            primitives.push(p);
        }
        (
            Rect::new(region[0], region[1], region[2], region[3]),
            UiFilter { primitives },
        )
    }
}

/// `f`, in some space, mapped by `m` into another: its subregions,
/// positions and lengths (an axis's lengths by that axis's scale, others
/// by the mean scale). A turn or skew turns offsets but not the axes
/// blurs and morphologies run along.
pub fn map(f: &mut UiFilter, m: &Affine) {
    if m.is_identity() {
        return;
    }
    for p in &mut f.primitives {
        let [x, y, w, h] = p.subregion;
        let r = m.map_rect(&Rect::new(x, y, w, h));
        p.subregion = [r.x, r.y, r.width, r.height];
        map_op(&mut p.op, m);
    }
}

fn map_op(op: &mut UiFilterOp, m: &Affine) {
    let s = m.determinant().abs().sqrt();
    let sx = m.a.hypot(m.b);
    let sy = m.c.hypot(m.d);
    let point = |v: &mut Vec3| {
        let (x, y) = m.apply(v.x, v.y);
        *v = Vec3::new(x, y, v.z * s);
    };
    let rect = |r: &mut [f32; 4]| {
        let o = m.map_rect(&Rect::new(r[0], r[1], r[2], r[3]));
        *r = [o.x, o.y, o.width, o.height];
    };
    match op {
        UiFilterOp::Blur { std_dev } => *std_dev = [std_dev[0] * sx, std_dev[1] * sy],
        UiFilterOp::Morphology { radius, .. } => *radius = [radius[0] * sx, radius[1] * sy],
        UiFilterOp::Offset { d } => *d = [m.a * d[0] + m.c * d[1], m.b * d[0] + m.d * d[1]],
        UiFilterOp::Displacement { scale, .. } => *scale *= s,
        UiFilterOp::Turbulence { user, .. } => {
            let (x, y) = m.apply(user[0], user[1]);
            *user = [x, y, user[2] / s.max(1e-6)];
        }
        UiFilterOp::Image { rect: r, .. } | UiFilterOp::Tile { tile: r } => {
            if !r[0].is_nan() {
                rect(r);
            }
        }
        UiFilterOp::Lighting {
            surface_scale,
            light,
            ..
        } => {
            *surface_scale *= s;
            match light {
                UiLight::Distant { .. } => {}
                UiLight::Point { at } => point(at),
                UiLight::Spot { at, points_at, .. } => {
                    point(at);
                    point(points_at);
                }
            }
        }
        UiFilterOp::Flood { .. }
        | UiFilterOp::Composite { .. }
        | UiFilterOp::Blend(_)
        | UiFilterOp::ColorMatrix(_)
        | UiFilterOp::ComponentTransfer(_)
        | UiFilterOp::Merge
        | UiFilterOp::Convolve { .. } => {}
    }
}
