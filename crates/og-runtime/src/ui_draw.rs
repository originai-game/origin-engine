//! `og.ui.setDraw` and the assets an element draws with, `og.load.vector`
//! and `og.load.uiMaterial` (`docs/ui.md` §5): the IDL's DrawDesc read
//! into og-ui's [`ElementDraw`]. Colours given as theme tokens stay tokens,
//! resolved through the style when the frame is drawn.

use super::assets::{handle, index_of};
use super::engine::engine;
use super::modules::{texture_of, ui_engine, with_ui};
use super::{ok, parse, refuse};
use og_ui::Srgba;
use og_ui::paint::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// What a world's renderer holds of this module.
#[derive(Default)]
pub(crate) struct UiDrawState {
    /// Vectors by VectorArt handle index, and the asset each came from.
    vectors: Vec<UiVectorHandle>,
    vector_paths: BTreeMap<String, usize>,
    /// UI materials by UiMaterial handle index, their parameters' defaults,
    /// and the asset each came from.
    materials: Vec<(UiMaterialHandle, [[f32; 4]; og_render::UI_MATERIAL_PARAMS])>,
    material_paths: BTreeMap<String, usize>,
}

/// `og.load.vector`: a cooked vector, added to the scene once per path.
pub fn og_load_vector(world: u32, path: String) -> String {
    let r = (|| -> Result<Value, String> {
        let eng = engine(world)?;
        Ok(json!(handle(load_vector(&eng, &path)?.0)))
    })();
    match r {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

/// A cooked vector added to the scene once per path: its VectorArt index
/// and its handle (a style's vector fills load through here too).
pub(crate) fn load_vector(
    eng: &super::engine::Engine,
    path: &str,
) -> Result<(usize, UiVectorHandle), String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    if let Some(&i) = eng.render.borrow().ui_draw.vector_paths.get(&id) {
        return Ok((i, eng.render.borrow().ui_draw.vectors[i]));
    }
    let vector = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        bundle
            .vector(&id, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    let mut rs = eng.render.borrow_mut();
    let h = rs.scene.add_ui_vector(vector);
    let st = &mut rs.ui_draw;
    st.vectors.push(h);
    let i = st.vectors.len() - 1;
    st.vector_paths.insert(id, i);
    Ok((i, h))
}

/// `og.load.uiMaterial`: a cooked UI material, added to the scene once per path.
pub fn og_load_ui_material(world: u32, path: String) -> String {
    let r = (|| -> Result<Value, String> {
        let eng = engine(world)?;
        let id = og_bundle::runtime::asset_id(&path).to_string();
        if let Some(&i) = eng.render.borrow().ui_draw.material_paths.get(&id) {
            return Ok(json!(handle(i)));
        }
        let material = {
            let assets = eng.assets.borrow();
            let (bytes, bundle) = assets.blob(&id)?;
            bundle
                .ui_material(&id, bytes.bytes())
                .map_err(|e| e.to_string())?
        };
        let defaults = material.defaults();
        let mut rs = eng.render.borrow_mut();
        let h = rs.scene.add_ui_material(material);
        let st = &mut rs.ui_draw;
        st.materials.push((h, defaults));
        let i = st.materials.len() - 1;
        st.material_paths.insert(id, i);
        Ok(json!(handle(i)))
    })();
    match r {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

/// `og.ui.setDraw`: what the element added last draws with.
pub fn og_ui_set_draw(desc: Option<String>) {
    let d = parse(desc.as_deref()).unwrap_or(Value::Null);
    // The handles it names, read before the UI is locked.
    let assets = ui_engine().and_then(|id| engine(id).ok()).map(|eng| {
        let rs = eng.render.borrow();
        (rs.ui_draw.vectors.clone(), rs.ui_draw.materials.clone())
    });
    let (vectors, materials) = assets.unwrap_or_default();
    let read = Reader {
        vectors: &vectors,
        materials: &materials,
    };
    let draw = read.draw(&d);
    if d["dressable"].as_bool().unwrap_or(false) {
        with_ui("og.ui.setDraw", |ui| ui.set_draw_dressable(draw));
    } else {
        with_ui("og.ui.setDraw", |ui| ui.set_draw(draw));
    }
}

/// A LayerDesc's material and parameters as og-render takes them (a meta
/// layer's material, `docs/ui.md` §14); `None` for a value that is not one.
pub(crate) fn layer_material(
    v: &Value,
) -> Option<(
    og_render::UiMaterial,
    [[f32; 4]; og_render::UI_MATERIAL_PARAMS],
)> {
    let assets = ui_engine().and_then(|id| engine(id).ok()).map(|eng| {
        let rs = eng.render.borrow();
        (rs.ui_draw.vectors.clone(), rs.ui_draw.materials.clone())
    });
    let (vectors, materials) = assets.unwrap_or_default();
    let read = Reader {
        vectors: &vectors,
        materials: &materials,
    };
    let l = read.layer(v)?;
    let material = match l.material {
        Material::Plain => og_render::UiMaterial::Plain,
        Material::Builtin(b) => og_render::UiMaterial::Builtin(b),
        Material::Custom(h) => og_render::UiMaterial::Custom(h),
    };
    Some((material, l.params))
}

fn num(v: &Value, k: &str, d: f32) -> f32 {
    v[k].as_f64().map_or(d, |x| x as f32)
}

fn vec2(v: &Value, d: [f32; 2]) -> [f32; 2] {
    if !v.is_object() {
        return d;
    }
    [num(v, "x", d[0]), num(v, "y", d[1])]
}

fn array4(v: &Value) -> Option<[f32; 4]> {
    let a = v.as_array().filter(|a| a.len() == 4)?;
    let n = |i: usize| a[i].as_f64().unwrap_or(0.0) as f32;
    Some([n(0), n(1), n(2), n(3)])
}

/// A Paint as a colour reference: its token, else its sRGB colour, else
/// `default` (a token), with its alpha.
fn color(v: &Value, default: &str) -> ColorRef {
    let alpha = num(v, "alpha", 1.0).clamp(0.0, 1.0);
    if let Some(t) = v["token"].as_str() {
        return ColorRef::Token {
            name: t.to_string(),
            alpha,
        };
    }
    let c = &v["color"];
    if c.is_object() {
        let ch = |k: &str| (num(c, k, 1.0).clamp(0.0, 1.0) * 255.0).round() as u8;
        return ColorRef::Literal(Srgba::rgb(ch("r"), ch("g"), ch("b")).fade(alpha));
    }
    match default {
        "" => ColorRef::Literal(Srgba::WHITE.fade(alpha)),
        t => ColorRef::Token {
            name: t.to_string(),
            alpha,
        },
    }
}

/// A colour that is white when the Paint is absent.
fn tint(v: &Value) -> ColorRef {
    color(v, "")
}

fn cap(v: &Value) -> Cap {
    match v.as_str() {
        Some("round") => Cap::Round,
        Some("square") => Cap::Square,
        _ => Cap::Butt,
    }
}

fn align(v: &Value) -> StrokeAlign {
    match v.as_str() {
        Some("inside") => StrokeAlign::Inside,
        Some("outside") => StrokeAlign::Outside,
        _ => StrokeAlign::Center,
    }
}

fn gradient(v: &Value) -> Gradient {
    let stops = v["stops"]
        .as_array()
        .map(|a| {
            a.iter()
                .take(og_render::UI_GRADIENT_STOPS)
                .map(|s| Stop {
                    offset: num(s, "offset", 0.0).clamp(0.0, 1.0),
                    color: color(&s["color"], ""),
                })
                .collect()
        })
        .unwrap_or_default();
    Gradient {
        kind: match v["type"].as_str() {
            Some("radial") => GradientKind::Radial,
            Some("conic") => GradientKind::Conic,
            Some("diamond") => GradientKind::Diamond,
            _ => GradientKind::Linear,
        },
        from: vec2(&v["from"], [0.0, 0.0]),
        to: vec2(&v["to"], [0.0, 1.0]),
        stops,
        spread: match v["spread"].as_str() {
            Some("repeat") => Spread::Repeat,
            Some("reflect") => Spread::Reflect,
            _ => Spread::Pad,
        },
        dither: !v["noDither"].as_bool().unwrap_or(false),
    }
}

/// The handles of the world's loaded vectors and materials.
struct Reader<'a> {
    vectors: &'a [UiVectorHandle],
    materials: &'a [(UiMaterialHandle, [[f32; 4]; og_render::UI_MATERIAL_PARAMS])],
}

impl Reader<'_> {
    fn fill(&self, v: &Value) -> Option<Paint> {
        Some(match v["kind"].as_str()? {
            "solid" => Paint::Solid(color(&v["color"], "")),
            "gradient" => Paint::Gradient(gradient(v)),
            "image" => {
                let texture = texture_of(&v["texture"])?;
                let size = vec2(&v["size"], [0.0, 0.0]);
                let size = if size[0] > 0.0 && size[1] > 0.0 {
                    size
                } else {
                    texture_size(texture).unwrap_or([1.0, 1.0])
                };
                let slice = array4(&v["slice"]).unwrap_or([0.0; 4]);
                let tile = vec2(&v["tile"], [0.0, 0.0]);
                Paint::Image(ImagePaint {
                    texture,
                    uv: array4(&v["uv"]).unwrap_or([0.0, 0.0, 1.0, 1.0]),
                    size,
                    tint: tint(&v["tint"]),
                    fit: match v["fit"].as_str() {
                        Some("cover") => Fit::Cover,
                        Some("contain") => Fit::Contain,
                        Some("none") => Fit::None,
                        Some("nineSlice") => Fit::NineSlice {
                            source: slice,
                            dest: array4(&v["border"]).unwrap_or(slice),
                            edges: match v["edges"].as_str() {
                                Some("repeat") => Edge::Repeat,
                                Some("round") => Edge::Round,
                                _ => Edge::Stretch,
                            },
                            center: !v["hollow"].as_bool().unwrap_or(false),
                        },
                        Some("pattern") => Fit::Pattern {
                            tile: if tile[0] > 0.0 && tile[1] > 0.0 {
                                tile.into()
                            } else {
                                size.into()
                            },
                            rotation: num(v, "patternRotation", 0.0),
                            offset: vec2(&v["offset"], [0.0, 0.0]).into(),
                            scroll: vec2(&v["scroll"], [0.0, 0.0]).into(),
                        },
                        _ => Fit::Fill,
                    },
                    focal: vec2(&v["focal"], [0.5, 0.5]),
                })
            }
            "vector" => Paint::Vector {
                vector: *index_of(v["vector"].as_f64()?).and_then(|i| self.vectors.get(i))?,
                tint: tint(&v["tint"]),
            },
            _ => return None,
        })
    }

    fn stroke(&self, v: &Value) -> Stroke {
        let dash = num(v, "dash", 0.0).max(0.0);
        let gap = num(v, "gap", 0.0).max(0.0);
        Stroke {
            width: num(v, "width", 1.0).max(0.0),
            align: align(&v["align"]),
            dash: [dash, if gap > 0.0 { gap } else { dash }],
            dash_offset: num(v, "dashOffset", 0.0),
            dash_cap: cap(&v["dashCap"]),
            paint: self
                .fill(&v["paint"])
                .unwrap_or(Paint::Solid(ColorRef::token("line"))),
        }
    }

    fn effect(v: &Value) -> Option<Effect> {
        let shadow = |c| {
            (
                vec2(&v["offset"], [0.0, 4.0]),
                num(v, "blur", 12.0).max(0.0),
                num(v, "spread", 0.0),
                color(&v["color"], c),
            )
        };
        let glow = |c| {
            (
                num(v, "blur", 12.0).max(0.0),
                num(v, "spread", 0.0),
                color(&v["color"], c),
            )
        };
        Some(match v["kind"].as_str()? {
            "dropShadow" => {
                let (offset, blur, spread, color) = shadow("shadow");
                Effect::DropShadow {
                    offset,
                    blur,
                    spread,
                    color,
                }
            }
            "innerShadow" => {
                let (offset, blur, spread, color) = shadow("shadow");
                Effect::InnerShadow {
                    offset,
                    blur,
                    spread,
                    color,
                }
            }
            "outerGlow" => {
                let (blur, spread, color) = glow("primary");
                Effect::OuterGlow {
                    blur,
                    spread,
                    color,
                }
            }
            "innerGlow" => {
                let (blur, spread, color) = glow("primary");
                Effect::InnerGlow {
                    blur,
                    spread,
                    color,
                }
            }
            "layerBlur" => Effect::LayerBlur {
                radius: num(v, "radius", 4.0).max(0.0),
            },
            "backdrop" => Effect::Backdrop {
                blur: num(v, "blur", 16.0).max(0.0),
                saturation: num(v, "saturation", 1.0).max(0.0),
                brightness: num(v, "brightness", 1.0).max(0.0),
                tint: if v["tint"].is_object() {
                    color(&v["tint"], "panel")
                } else {
                    ColorRef::Token {
                        name: "panel".into(),
                        alpha: 0.6,
                    }
                },
                glass: og_ui::paint::GlassParts {
                    weight: match v["glass"].as_str() {
                        Some("thin") => Some(og_render::UiGlassWeight::Thin),
                        Some("regular") => Some(og_render::UiGlassWeight::Regular),
                        Some("thick") => Some(og_render::UiGlassWeight::Thick),
                        _ => None,
                    },
                    elevation: match v["elevation"].as_str() {
                        Some("raised") => Some(og_render::UiElevation::Raised),
                        Some("float") => Some(og_render::UiElevation::Float),
                        Some("lift") => Some(og_render::UiElevation::Lift),
                        _ => None,
                    },
                    edge: num(v, "edge", 0.0).max(0.0),
                    edge_color: v["edgeColor"]
                        .is_object()
                        .then(|| color(&v["edgeColor"], "line")),
                    solid: v["solid"].as_bool().unwrap_or(false),
                    css_mix: v["cssMix"].as_bool().unwrap_or(false),
                    own_tint: v["tint"].is_object(),
                },
            },
            _ => return None,
        })
    }

    fn shape(v: &Value) -> ShapeKind {
        match v["kind"].as_str() {
            Some("ellipse") => ShapeKind::Ellipse,
            Some("polygon") => ShapeKind::Polygon {
                sides: v["sides"].as_u64().map_or(6, |s| s.clamp(3, 64) as u32),
                inner: num(v, "inner", 1.0).max(0.0),
                rounding: num(v, "rounding", 0.0).max(0.0),
                rotation: num(v, "rotation", 0.0),
            },
            Some("arc") => ShapeKind::Arc {
                start: num(v, "start", 0.0),
                sweep: num(v, "sweep", std::f32::consts::TAU),
                thickness: num(v, "thickness", 4.0).max(0.0),
                cap: cap(&v["cap"]),
            },
            Some("path") => ShapeKind::Path {
                points: v["points"]
                    .as_array()
                    .map(|a| a.iter().map(|p| vec2(p, [0.0, 0.0])).collect())
                    .unwrap_or_default(),
                closed: v["closed"].as_bool().unwrap_or(false),
                filled: v["filled"].as_bool().unwrap_or(false),
                curves: v["curves"].as_bool().unwrap_or(false),
                width: num(v, "width", 2.0).max(0.0),
                cap: cap(&v["cap"]),
                join: match v["join"].as_str() {
                    Some("round") => Join::Round,
                    Some("bevel") => Join::Bevel,
                    _ => Join::Miter(num(v, "miterLimit", 4.0).max(1.0)),
                },
            },
            _ => ShapeKind::Rect {
                radii: array4(&v["radii"]),
                corner: match v["corner"].as_str() {
                    Some("squircle") => Corner::Squircle(num(v, "squircle", 4.0).clamp(2.0, 12.0)),
                    Some("bevel") => Corner::Bevel,
                    Some("notch") => Corner::Notch,
                    _ => Corner::Round,
                },
            },
        }
    }

    fn layer(&self, v: &Value) -> Option<Layer> {
        if !v.is_object() {
            return None;
        }
        let (material, mut params) = match v["material"].as_str() {
            Some("custom") => {
                let (h, defaults) =
                    *index_of(v["custom"].as_f64()?).and_then(|i| self.materials.get(i))?;
                (Material::Custom(h), defaults)
            }
            Some(name) => (
                match name {
                    "dissolve" => Material::Builtin(Builtin::Dissolve),
                    "shine" => Material::Builtin(Builtin::Shine),
                    "hologram" => Material::Builtin(Builtin::Hologram),
                    "glitch" => Material::Builtin(Builtin::Glitch),
                    "ripple" => Material::Builtin(Builtin::Ripple),
                    "pixelate" => Material::Builtin(Builtin::Pixelate),
                    "colorAdjust" => Material::Builtin(Builtin::ColorAdjust),
                    "outline" => Material::Builtin(Builtin::Outline),
                    "blur" => Material::Builtin(Builtin::Blur),
                    _ => Material::Plain,
                },
                [[0.0; 4]; og_render::UI_MATERIAL_PARAMS],
            ),
            None => (Material::Plain, [[0.0; 4]; og_render::UI_MATERIAL_PARAMS]),
        };
        if let Some(a) = v["params"].as_array() {
            for (k, x) in a.iter().take(4 * og_render::UI_MATERIAL_PARAMS).enumerate() {
                if let Some(x) = x.as_f64() {
                    params[k / 4][k % 4] = x as f32;
                }
            }
        }
        Some(Layer { material, params })
    }

    fn text(v: &Value) -> Option<TextEffects> {
        if !v.is_object() {
            return None;
        }
        Some(TextEffects {
            shadows: v["shadows"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .map(|s| TextShadow {
                            offset: vec2(&s["offset"], [0.0, 2.0]),
                            blur: num(s, "blur", 2.0).max(0.0),
                            color: color(&s["color"], "shadow"),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            outline: v["outline"].is_object().then(|| {
                let o = &v["outline"];
                TextOutline {
                    width: num(o, "width", 2.0).max(0.0),
                    color: color(&o["color"], "shadow"),
                    align: align(&o["align"]),
                }
            }),
            fill: v["fill"].is_object().then(|| gradient(&v["fill"])),
        })
    }

    fn draw(&self, d: &Value) -> ElementDraw {
        let list = |k: &str| d[k].as_array().map(Vec::as_slice).unwrap_or(&[]);
        let clip = match d["clip"].as_str() {
            Some("shape") => Clip::Shape,
            Some("mask") => {
                let m = &d["mask"];
                if m["gradient"].is_object() {
                    Clip::Mask(Mask::Gradient(gradient(&m["gradient"])))
                } else if let Some(texture) = texture_of(&m["texture"]) {
                    Clip::Mask(Mask::Image {
                        texture,
                        uv: array4(&m["uv"]).unwrap_or([0.0, 0.0, 1.0, 1.0]),
                    })
                } else {
                    Clip::None
                }
            }
            _ => Clip::None,
        };
        ElementDraw {
            shape: if d["shape"].is_object() {
                Reader::shape(&d["shape"])
            } else {
                ShapeKind::default()
            },
            fills: list("fills").iter().filter_map(|f| self.fill(f)).collect(),
            strokes: list("strokes").iter().map(|s| self.stroke(s)).collect(),
            effects: list("effects").iter().filter_map(Reader::effect).collect(),
            opacity: num(d, "opacity", 1.0).clamp(0.0, 1.0),
            blend: match d["blend"].as_str() {
                Some("multiply") => Blend::Multiply,
                Some("screen") => Blend::Screen,
                Some("add") => Blend::Add,
                Some("overlay") => Blend::Overlay,
                Some("softLight") => Blend::SoftLight,
                Some("colorDodge") => Blend::ColorDodge,
                _ => Blend::Normal,
            },
            clip,
            layer: self.layer(&d["layer"]),
            text: Reader::text(&d["text"]),
        }
    }
}

/// A loaded texture's size in pixels.
fn texture_size(t: og_render::TextureHandle) -> Option<[f32; 2]> {
    let eng = engine(ui_engine()?).ok()?;
    let rs = eng.render.borrow();
    let (w, h) = rs.scene.texture_size(t)?;
    Some([w as f32, h as f32])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(d: Value) -> ElementDraw {
        Reader {
            vectors: &[UiVectorHandle(0)],
            materials: &[(
                UiMaterialHandle(0),
                [[0.5; 4]; og_render::UI_MATERIAL_PARAMS],
            )],
        }
        .draw(&d)
    }

    /// The IDL's DrawDesc reads into the element's draw: tokens stay
    /// tokens, defaults are the IDL's, unions by their kind, and a custom
    /// material keeps its defaults past the params given.
    #[test]
    fn a_draw_desc_reads_with_its_tokens_and_defaults() {
        let d = read(json!({
            "shape": { "kind": "rect", "corner": "squircle", "radii": [4, 8, 12, 16] },
            "fills": [
                { "kind": "solid", "color": { "token": "panel", "alpha": 0.5 } },
                { "kind": "gradient", "type": "radial", "from": { "x": 0.5, "y": 0.5 },
                  "stops": [{ "offset": 0, "color": { "color": { "r": 1, "g": 0, "b": 0 } } }] },
                { "kind": "vector", "vector": handle(0) as f64 },
                { "kind": "image" }
            ],
            "strokes": [{ "width": 2, "dash": 6 }],
            "effects": [{ "kind": "dropShadow" }, { "kind": "backdrop", "blur": 20 }],
            "clip": "shape",
            "layer": { "material": "custom", "custom": handle(0) as f64, "params": [1, 2] },
            "text": { "outline": {} }
        }));
        assert_eq!(
            d.shape,
            ShapeKind::Rect {
                radii: Some([4.0, 8.0, 12.0, 16.0]),
                corner: Corner::Squircle(4.0)
            }
        );
        // An image fill without a loaded texture is left out.
        assert_eq!(d.fills.len(), 3);
        assert_eq!(
            d.fills[0],
            Paint::Solid(ColorRef::Token {
                name: "panel".into(),
                alpha: 0.5
            })
        );
        let Paint::Gradient(g) = &d.fills[1] else {
            panic!("{:?}", d.fills[1])
        };
        assert_eq!(
            (g.kind, g.from, g.to),
            (GradientKind::Radial, [0.5, 0.5], [0.0, 1.0])
        );
        assert!(g.dither);
        assert_eq!(d.strokes[0].dash, [6.0, 6.0]);
        assert_eq!(d.strokes[0].paint, Paint::Solid(ColorRef::token("line")));
        assert_eq!(
            d.effects[0],
            Effect::DropShadow {
                offset: [0.0, 4.0],
                blur: 12.0,
                spread: 0.0,
                color: ColorRef::token("shadow")
            }
        );
        let Effect::Backdrop { blur, tint, .. } = &d.effects[1] else {
            panic!()
        };
        assert_eq!(*blur, 20.0);
        assert_eq!(
            *tint,
            ColorRef::Token {
                name: "panel".into(),
                alpha: 0.6
            }
        );
        assert_eq!(d.clip, Clip::Shape);
        let layer = d.layer.expect("a layer");
        assert_eq!(layer.material, Material::Custom(UiMaterialHandle(0)));
        assert_eq!(layer.params[0], [1.0, 2.0, 0.5, 0.5]);
        assert_eq!(layer.params[1], [0.5; 4]);
        assert_eq!(d.text.expect("text").outline.expect("outline").width, 2.0);
        assert_eq!(d.opacity, 1.0);
        // Nothing given: the element's rectangle, drawing nothing of its own.
        assert_eq!(read(Value::Null), ElementDraw::default());
    }
}
