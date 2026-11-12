//! SVG `<filter>` in documents (Filter Effects 1): CSS `filter: url(#id)`
//! mixed with filter functions in order, an inline SVG element's `filter`
//! attribute, and a tile for each filter primitive. The frames match `references/vulkan/css-filters-{1280,390}.png`
//! (`OG_UI_CAPTURE=1` records them).
//!
//! Ways it could fail: a primitive's `in`/`in2`/`result` graph wired to
//! the wrong result, the filter region cut to what the element draws (a
//! drop shadow or a dilation cut at the box), objectBoundingBox units read
//! as user units, an SVG element's filter in the `<svg>`'s space rather
//! than the element's, a filter function after `url()` applied before it,
//! a primitive run in the wrong colour space, or an idle filtered element
//! that rebuilds every frame.

use crate::support;
use og_ui::prelude::*;

use og_ui::*;
use support::*;

/// A tinted image (`ImageDesc::tint`) and the same image through a
/// linearRGB diagonal `feColorMatrix` draw the same picture, within one
/// channel value, on both document paths.
///
/// Both draw into a linear Rgba16Float output, read back through the
/// capture's exact sRGB encode, so the comparison checks the matrix maths
/// rather than the GPU's blender. Drawn into the default 8-bit sRGB output,
/// the two paths' final blend inputs differ by the filter layer's
/// Rgba16Float round trip (about 0.06 of a step), and NVIDIA's blend into
/// an 8-bit sRGB target is accurate to only ±1.2 steps and is stepped, so
/// that difference came out as 2 at translucent texels, even with the
/// tinted image drawn through the same isolation layer. Measured on an
/// NVIDIA L4 (Vulkan, driver 595.99.02) with a raw wgpu probe blending a
/// 4096-value colour sweep at alpha 0.8614 over a fixed background into
/// Rgba8UnormSrgb: largest error 1.21 steps against the exact blend, 698 of
/// 4096 values off by more than 0.6, six jumps of 2 between neighbouring
/// inputs; at alpha 0.5, 1.19 and 505; unblended (alpha 1), at most 0.598,
/// rounded correctly. lavapipe stays within 0.74 with no jump.
/// docs/roadmap.md §Open has blending the UI in a linear target instead.
#[test]
#[ignore = "requires GPU"]
fn color_matrix_preserves_texture_tint_multiplication() {
    use og_render::*;
    struct Env(TextureHandle);
    impl bind::Env for Env {
        fn texture(&self, _: f64) -> Option<TextureHandle> {
            Some(self.0)
        }
    }
    let mut scene = RenderScene::new();
    scene.lighting.environment.background =
        Background::Color(og_core::glam::Vec3::new(0.08, 0.2, 0.4));
    scene.look.post.tonemap = Tonemap::None;
    let texture = scene.add_texture(TextureData {
        width: 2,
        height: 2,
        rgba: vec![
            211, 67, 129, 255, 43, 193, 87, 177, 19, 101, 233, 83, 239, 159, 31, 0,
        ],
        color_space: ColorSpace::Srgb,
        mips: Mips::None,
    });
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width: 64,
        height: 32,
        output_format: Some(wgpu::TextureFormat::Rgba16Float),
        ..Default::default()
    })
    .unwrap();
    let mut maximum = 0;
    for tint in ["#80c040", "#d060a080"] {
        let linear = style_file::parse_color(tint).unwrap().linear();
        let source = |css: &str| {
            format!(
                r##"<meta name="component" content="t"><div><img id="Pic" src={{1}} alt=""/><svg width="0" height="0" viewBox="0 0 1 1"><filter id="Tint" x="0" y="0" width="1" height="1" color-interpolation-filters="linearRGB"><feColorMatrix type="matrix" values="{r} 0 0 0 0 0 {g} 0 0 0 0 0 {b} 0 0 0 0 0 {a} 0"/></filter></svg></div><style>#Pic {{ width: 64px; height: 32px; {css} }}</style>"##,
                r = linear[0],
                g = linear[1],
                b = linear[2],
                a = linear[3]
            )
        };
        for mounted in [false, true] {
            let mut render = |css: &str| {
                let mut ui = ui_with_font(test_font_metrics());
                ui.define_document(UiDocument::parse("tint.svelte", &source(css)).unwrap());
                if mounted && !css.is_empty() {
                    ui.mount_document(
                        reactive::SourceId::new("test", "tint"),
                        "t",
                        Default::default(),
                        Default::default(),
                        &Env(texture),
                    )
                    .unwrap();
                }
                for _ in 0..2 {
                    ui.begin(InputFrame::default(), Viewport::new(64.0, 32.0));
                    if css.is_empty() {
                        ui.image(
                            "Pic",
                            ImageDesc {
                                texture: Some(texture),
                                tint: Some(style_file::parse_color(tint).unwrap()),
                                width: Size::Fixed(64.0),
                                height: Size::Fixed(32.0),
                                ..Default::default()
                            },
                        );
                    } else if mounted {
                        ui.build_retained_documents(&Env(texture)).unwrap();
                    } else {
                        ui.component("T", "t", &mut serde_json::json!({}), &Env(texture));
                    }
                    ui.end();
                }
                assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
                let mut overlay = Overlay::new(64.0, 32.0, 1.0);
                ui.frame().emit(&EmitDesc::default(), &mut overlay);
                scene.ui.overlay = Some(overlay);
                renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
                renderer.read_offscreen().unwrap().pixels
            };
            let before = render("");
            let after = render("filter: url(#Tint);");
            assert!(
                before
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .any(|pixel| pixel != &before[..4]),
                "the texture must draw, not an empty background"
            );
            let diff = before
                .iter()
                .zip(&after)
                .map(|(a, b)| a.abs_diff(*b))
                .max()
                .unwrap();
            maximum = maximum.max(diff);
            assert!(
                diff <= 1,
                "{tint}, mounted={mounted}: maximum channel difference {diff}"
            );
        }
    }
    println!("maximum tint/matrix channel difference: {maximum}");
    assert!(
        UiDocument::parse(
            "removed.svelte",
            "<meta name=\"component\" content=\"t\"><img style=\"-og-tint: #80c040\"/>"
        )
        .is_err()
    );
}

const CARD: &str = r##"<meta name="component" content="filters">
<column id="Center">
<div id="Card">
  <h2 id="Title">SVG filters</h2>
  <div id="Row">
    <div id="Glow" class="tile">Glow</div>
    <div id="Mixed" class="tile">Mixed</div>
    <div id="Frost" class="tile"><div class="stripes"></div><div id="Pane">Pane</div></div>
    <svg id="Art" viewBox="0 0 120 80" width="120" height="80">
      <g filter="url(#Emboss)">
        <circle cx="34" cy="40" r="24" fill="#58a6ff"/>
        <rect x="64" y="18" width="40" height="44" rx="6" fill="#f0b429"/>
      </g>
      <path d="M8 74 L112 74" stroke="#7ee787" stroke-width="4" filter="url(#Wave)"/>
    </svg>
  </div>
  <div id="Prims">
    <div class="p" id="PBlur"></div>
    <div class="p" id="PMatrix"></div>
    <div class="p" id="PTransfer"></div>
    <div class="p" id="PMorph"></div>
    <div class="p" id="PComposite"></div>
    <div class="p" id="PBlend"></div>
    <div class="p" id="PTurb"></div>
    <div class="p" id="PDisplace"></div>
    <div class="p" id="PConvolve"></div>
    <div class="p" id="PSpot"></div>
    <div class="p" id="PTile"></div>
    <div class="p" id="PFlood"></div>
  </div>
  <svg id="Defs" width="0" height="0">
    <filter id="GlowF" x="-30%" y="-30%" width="160%" height="160%">
      <feMorphology in="SourceAlpha" operator="dilate" radius="2" result="fat"/>
      <feGaussianBlur in="fat" stdDeviation="4" result="soft"/>
      <feFlood flood-color="#ffd166" result="gold"/>
      <feComposite in="gold" in2="soft" operator="in" result="halo"/>
      <feMerge><feMergeNode in="halo"/><feMergeNode in="SourceGraphic"/></feMerge>
    </filter>
    <filter id="Shadow"><feDropShadow dx="4" dy="4" stdDeviation="3" flood-color="#000" flood-opacity="0.8"/></filter>
    <filter id="Tint" color-interpolation-filters="sRGB">
      <feColorMatrix type="hueRotate" values="140"/>
    </filter>
    <filter id="Emboss" filterUnits="userSpaceOnUse" x="0" y="0" width="120" height="70">
      <feGaussianBlur in="SourceAlpha" stdDeviation="2" result="bump"/>
      <feSpecularLighting in="bump" surfaceScale="4" specularConstant="1" specularExponent="18" lighting-color="#fff" result="spec">
        <fePointLight x="20" y="0" z="60"/>
      </feSpecularLighting>
      <feComposite in="spec" in2="SourceAlpha" operator="in" result="shine"/>
      <feComposite in="SourceGraphic" in2="shine" operator="arithmetic" k2="1" k3="1"/>
    </filter>
    <filter id="Wave" filterUnits="userSpaceOnUse" x="0" y="64" width="120" height="16">
      <feTurbulence type="turbulence" baseFrequency="0.08" numOctaves="2" seed="3" result="noise"/>
      <feDisplacementMap in="SourceGraphic" in2="noise" scale="6" xChannelSelector="R" yChannelSelector="G"/>
    </filter>
    <filter id="Blur" primitiveUnits="objectBoundingBox"><feGaussianBlur stdDeviation="0.05 0.02"/></filter>
    <filter id="Matrix"><feColorMatrix type="matrix" values="0 0 1 0 0  1 0 0 0 0  0 1 0 0 0  0 0 0 1 0"/></filter>
    <filter id="Transfer">
      <feComponentTransfer>
        <feFuncR type="discrete" tableValues="0 0.5 1"/>
        <feFuncG type="gamma" amplitude="1" exponent="3" offset="0"/>
        <feFuncB type="table" tableValues="1 0"/>
      </feComponentTransfer>
    </filter>
    <filter id="Morph"><feMorphology operator="erode" radius="6 2"/></filter>
    <filter id="Composite">
      <feFlood flood-color="#ff7b72" x="0" y="0" width="30" height="48" result="left"/>
      <feComposite in="left" in2="SourceGraphic" operator="xor"/>
    </filter>
    <filter id="Blend">
      <feFlood flood-color="#7ee787" result="green"/>
      <feBlend in="green" in2="SourceGraphic" mode="difference"/>
    </filter>
    <filter id="Turb"><feTurbulence type="fractalNoise" baseFrequency="0.06" numOctaves="3" seed="7" stitchTiles="stitch"/></filter>
    <filter id="Displace">
      <feTurbulence baseFrequency="0.1" numOctaves="1" seed="1" result="t"/>
      <feDisplacementMap in="SourceGraphic" in2="t" scale="12" xChannelSelector="R" yChannelSelector="B"/>
    </filter>
    <filter id="Convolve"><feConvolveMatrix order="3" kernelMatrix="-2 -1 0 -1 1 1 0 1 2" edgeMode="none"/></filter>
    <filter id="Spot">
      <feDiffuseLighting in="SourceAlpha" surfaceScale="2" diffuseConstant="1.2" lighting-color="#ffd166">
        <feSpotLight x="0" y="0" z="40" pointsAtX="30" pointsAtY="30" pointsAtZ="0" specularExponent="4" limitingConeAngle="35"/>
      </feDiffuseLighting>
    </filter>
    <filter id="Tile">
      <feOffset dx="0" dy="0" x="0" y="0" width="16" height="16" result="cell"/>
      <feTile in="cell"/>
    </filter>
    <filter id="Flood" x="0" y="0" width="1" height="1">
      <feFlood flood-color="#1f6feb" flood-opacity="0.6"/>
      <feComposite in2="SourceGraphic" operator="atop"/>
    </filter>
  </svg>
</div>
</column>
<style>
#Center { width: 100%; height: 100%; align-items: center; justify-content: center; }
#Card { padding: 24px; display: flex; flex-direction: column; gap: 16px;
  background: linear-gradient(160deg, #1f2a44, #0d1117 70%); border-radius: 16px; }
#Title { margin: 0; }
#Row { display: flex; flex-direction: row; gap: 20px; align-items: center; }
.tile { width: 96px; height: 72px; border-radius: 12px; display: flex; align-items: center;
  justify-content: center; background: linear-gradient(135deg, #d2a8ff, #1f6feb); }
#Glow { filter: url(#GlowF); }
#Mixed { filter: url(#Shadow) grayscale(1) url("#Tint"); background: linear-gradient(135deg, #ff7b72, #f0b429); }
#Frost { position: relative; overflow: hidden; background: #0d1117; }
.stripes { position: absolute; inset: 0;
  background: repeating-linear-gradient(45deg, #f0b429 0 8px, #58a6ff 8px 16px); }
#Pane { position: absolute; inset: 14px; border-radius: 10px; background: #ffffff55; display: flex; align-items: center;
  justify-content: center; filter: blur(1px) url(#Matrix) opacity(0.8); }
#Prims { display: grid; grid-template-columns: repeat(6, 64px); gap: 12px; }
.p { width: 64px; height: 48px; border-radius: 8px;
  background: linear-gradient(90deg, #ff7b72, #f0b429 50%, #58a6ff); }
#PBlur { filter: url(#Blur); }
#PMatrix { filter: url(#Matrix); }
#PTransfer { filter: url(#Transfer); }
#PMorph { filter: url(#Morph); }
#PComposite { filter: url(#Composite); }
#PBlend { filter: url(#Blend); }
#PTurb { filter: url(#Turb); }
#PDisplace { filter: url(#Displace); }
#PConvolve { filter: url(#Convolve); }
#PSpot { filter: url(#Spot); }
#PTile { filter: url(#Tile); }
#PFlood { border-radius: 24px; filter: url(#Flood); }
@media (max-width: 500px) {
  #Row { flex-wrap: wrap; gap: 12px; width: 300px; }
  #Prims { grid-template-columns: repeat(4, 64px); }
}
</style>"##;

fn render(width: u32, height: u32) -> Rendered {
    use og_render::*;
    let mut scene = RenderScene::new();
    let font = add_test_font(&mut scene);
    let mut ui = ui_with_font(font);
    ui.define_document(UiDocument::parse("ui/filters.svelte", CARD).unwrap());
    ui.mount_document(
        reactive::SourceId::new("test", "filters"),
        "filters",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    let vp = Viewport::new(width as f32, height as f32);
    let mut stats = None;
    for _ in 0..4 {
        ui.begin(InputFrame::default(), vp);
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        stats = Some(ui.end().stats);
    }
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let stats = stats.unwrap();
    assert_eq!(
        (stats.bindings, stats.reconciled),
        (0, 0),
        "idle: {stats:?}"
    );
    let mut overlay = Overlay::new(width as f32, height as f32, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    let filtered = overlay
        .ui
        .layers
        .iter()
        .filter(|l| l.filter.is_some())
        .count();
    assert_eq!(
        filtered, 18,
        "a layer per url(): 12 tiles, 4 more in CSS, 2 in SVG"
    );
    scene.ui.overlay = Some(overlay);
    on_backdrop(&ui, &mut scene);
    render_frame(&scene, &sized(width, height))
}

#[test]
#[ignore = "requires GPU"]
fn css_filters_match_their_reference_frames() {
    for (width, height) in [(1280u32, 720u32), (390, 844)] {
        let frame = render(width, height);
        let name = format!("css-filters-{width}");
        matches_reference(&name, &frame.image, &frame.info, Against::Recorded);
    }
}

const SPACE: &str = r##"<meta name="component" content="filter-space">
<column id="Center">
<div id="Card">
  <h2 id="Title">Backdrops and filter space</h2>
  <div id="Row">
    <div class="ground"><div class="pane" id="Frost">blur</div></div>
    <div class="ground"><div class="pane" id="Warp">url()</div></div>
    <div class="ground" id="Group"><div class="pane" id="Inner">group</div></div>
    <div class="ground"><div class="pane" id="Shift">mixed</div></div>
    <div class="ground" id="Tilt"><div class="pane" id="Tilted">tilted</div></div>
  </div>
  <div id="Row2">
    <div class="t" id="TurnBlur"></div>
    <div class="t" id="TurnMorph"></div>
    <div class="t" id="TurnLight"></div>
    <div class="t" id="Unit"></div>
    <div class="t" id="UnitLight"></div>
    <div class="t" id="Element"></div>
    <svg id="Pic" width="64" height="48" viewBox="0 0 64 48">
      <rect x="6" y="6" width="52" height="36" rx="6" fill="#30363d" transform="rotate(-12 32 24)" filter="url(#Stamp2)"/>
    </svg>
  </div>
  <div id="Row3"><div id="Wide"></div></div>
  <svg id="Defs" width="0" height="0">
    <defs>
      <linearGradient id="Gold" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" stop-color="#ffd166"/><stop offset="1" stop-color="#f77f00"/>
      </linearGradient>
      <pattern id="Dots" width="4" height="4" patternUnits="userSpaceOnUse">
        <rect width="2" height="2" fill="#0d1117"/>
      </pattern>
      <circle id="Dot" cx="32" cy="16" r="3" fill="#58a6ff"/>
    </defs>
    <g id="Star" transform="translate(0 2)">
      <path d="M32 4 L40 24 L60 24 L44 36 L50 56 L32 44 L14 56 L20 36 L4 24 L24 24 Z" fill="url(#Gold)" stroke="#ff7b72" stroke-width="2"/>
      <circle cx="32" cy="40" r="6" fill="url(#Dots)"/>
      <use href="#Dot" x="-18" y="18"/>
      <g transform="rotate(-10 32 30)"><text x="32" y="30" font-size="9" text-anchor="middle" fill="#ffffff">OG</text></g>
    </g>
    <filter id="Ripple" x="0" y="0" width="1" height="1">
      <feTurbulence type="turbulence" baseFrequency="0.05" numOctaves="2" seed="4" result="t"/>
      <feDisplacementMap in="SourceGraphic" in2="t" scale="18" xChannelSelector="R" yChannelSelector="G"/>
    </filter>
    <filter id="Streak"><feGaussianBlur stdDeviation="8 0"/></filter>
    <filter id="Smear" filterUnits="userSpaceOnUse" x="-130" y="0" width="284" height="12"><feGaussianBlur stdDeviation="40 0"/></filter>
    <filter id="Spread" x="-20%" y="-20%" width="140%" height="140%"><feMorphology operator="dilate" radius="8 1"/></filter>
    <filter id="Raised">
      <feGaussianBlur in="SourceAlpha" stdDeviation="3" result="bump"/>
      <feDiffuseLighting in="bump" surfaceScale="6" diffuseConstant="1" lighting-color="#ffffff" result="lit">
        <feDistantLight azimuth="0" elevation="35"/>
      </feDiffuseLighting>
      <feComposite in="lit" in2="SourceGraphic" operator="arithmetic" k1="1"/>
    </filter>
    <filter id="Edges"><feConvolveMatrix order="3" kernelUnitLength="4" kernelMatrix="-1 -1 -1 -1 8 -1 -1 -1 -1" preserveAlpha="true"/></filter>
    <filter id="Coarse">
      <feSpecularLighting in="SourceAlpha" surfaceScale="3" specularConstant="1.2" specularExponent="12" kernelUnitLength="3" lighting-color="#58a6ff" result="s">
        <fePointLight x="10" y="6" z="40"/>
      </feSpecularLighting>
      <feComposite in="s" in2="SourceAlpha" operator="in" result="lit"/>
      <feComposite in="lit" in2="SourceGraphic" operator="arithmetic" k2="1" k3="1"/>
    </filter>
    <filter id="Stamp2" primitiveUnits="objectBoundingBox" x="0" y="0" width="1" height="1">
      <feImage href="#Star" x="0.1" y="-0.2" result="star"/>
      <feComposite in="star" in2="SourceGraphic" operator="over"/>
    </filter>
    <filter id="Stamp" x="0" y="0" width="1" height="1">
      <feImage href="#Star" x="0" y="-6" result="star"/>
      <feComposite in="star" in2="SourceGraphic" operator="over"/>
    </filter>
  </svg>
</div>
</column>
<style>
#Center { width: 100%; height: 100%; align-items: center; justify-content: center; }
#Card { padding: 24px; display: flex; flex-direction: column; gap: 18px;
  background: linear-gradient(160deg, #1f2a44, #0d1117 70%); border-radius: 16px; }
#Title { margin: 0; }
#Row, #Row2 { display: flex; flex-direction: row; gap: 20px; align-items: center; }
.ground { position: relative; width: 120px; height: 84px; border-radius: 12px;
  background: repeating-linear-gradient(45deg, #f0b429 0 8px, #58a6ff 8px 16px); }
.pane { position: absolute; inset: 14px; border-radius: 10px; display: flex; align-items: center;
  justify-content: center; background: #ffffff22; }
#Frost { backdrop-filter: blur(6px) saturate(1.8); }
#Warp { backdrop-filter: url(#Ripple); }
#Group { opacity: 0.9; }
#Inner { backdrop-filter: blur(3px) invert(1); }
#Tilt { transform: perspective(260px) rotateY(32deg); }
#Tilted { backdrop-filter: blur(2px) invert(1); }
#Shift { backdrop-filter: url(#Ripple) grayscale(0.7) drop-shadow(0 0 4px #000); }
.t { width: 64px; height: 48px; border-radius: 8px;
  background: linear-gradient(90deg, #ff7b72, #f0b429 50%, #58a6ff); }
#TurnBlur { transform: rotate(35deg); filter: url(#Streak); }
#TurnMorph { width: 40px; height: 20px; transform: rotate(-30deg); filter: url(#Spread); }
#TurnLight { border-radius: 24px; transform: rotate(90deg); filter: url(#Raised); }
#Unit { filter: url(#Edges); }
#UnitLight { border-radius: 24px; filter: url(#Coarse); }
#Element { transform: rotate(15deg); filter: url(#Stamp); }
#Row3 { display: flex; justify-content: center; align-self: stretch; padding: 0 0 4px; }
#Wide { width: 24px; height: 12px; filter: url(#Smear); background: #f0b429; }
@media (max-width: 500px) {
  #Row, #Row2 { flex-wrap: wrap; gap: 12px; width: 300px; }
}
</style>"##;

fn render_space(width: u32, height: u32) -> Rendered {
    use og_render::*;
    let mut scene = RenderScene::new();
    let font = add_test_font(&mut scene);
    let mut ui = ui_with_font(font);
    ui.define_document(UiDocument::parse("ui/filter_space.svelte", SPACE).unwrap());
    ui.mount_document(
        reactive::SourceId::new("test", "filter-space"),
        "filter-space",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    let vp = Viewport::new(width as f32, height as f32);
    let mut stats = None;
    for _ in 0..4 {
        ui.begin(InputFrame::default(), vp);
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        stats = Some(ui.end().stats);
    }
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let stats = stats.unwrap();
    assert_eq!(
        (stats.bindings, stats.reconciled),
        (0, 0),
        "idle: {stats:?}"
    );
    let mut overlay = Overlay::new(width as f32, height as f32, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    let layers = &overlay.ui.layers;
    assert_eq!(
        layers.iter().filter(|l| l.backdrop).count(),
        5,
        "a backdrop layer per backdrop-filter"
    );
    // Each stamp's element image is a layer its filter reads.
    let stamps: Vec<u32> = layers
        .iter()
        .filter_map(|l| l.filter.as_ref())
        .flat_map(|f| &f.primitives)
        .filter_map(|p| match p.op {
            UiFilterOp::Layer { layer } => Some(layer),
            _ => None,
        })
        .collect();
    assert_eq!(stamps.len(), 2, "feImage of #Star on a box and on a shape");
    assert!(
        stamps.iter().all(|&k| (k as usize) < layers.len()),
        "placed: {stamps:?}"
    );
    scene.ui.overlay = Some(overlay);
    on_backdrop(&ui, &mut scene);
    render_frame(&scene, &sized(width, height))
}

/// Backdrops read what is really under them (the screen, or the group
/// they are drawn in) through functions and `url()` alike; a turned
/// element's blur, dilation and lighting turn with it; `kernelUnitLength`
/// sizes a convolution's and a light's cells; `feImage` draws an element
/// as it draws: a group's gradient, pattern, `<use>` and turned text, on a
/// CSS box and on a turned SVG shape (moved by a bounding-box `x` and `y`).
///
/// Ways it could fail: a backdrop reading the layer drawn so far instead
/// of the screen (a pane over nothing), a backdrop in an opacity group
/// reading the screen instead of the group, a turned streak still
/// horizontal, a kernel unit read as a device px, the star drawn where it
/// is defined instead of in the filtered element's space, or only its
/// literal solid shapes drawn (no gradient, dots, blue dot or text).
#[test]
#[ignore = "requires GPU"]
fn css_filter_space_matches_its_reference_frames() {
    for (width, height) in [(1280u32, 720u32), (390, 844)] {
        let frame = render_space(width, height);
        let name = format!("css-filter-space-{width}");
        matches_reference(&name, &frame.image, &frame.info, Against::Recorded);
    }
}
