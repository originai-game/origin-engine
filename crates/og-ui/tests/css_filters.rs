//! SVG `<filter>` in documents (Filter Effects 1): CSS `filter: url(#id)`
//! mixed with filter functions in order, an inline SVG element's `filter`
//! attribute, and a tile for each filter primitive. The frames match `references/metal/css-filters-{1280,390}.png`
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

use og_ui::*;
use support::*;

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

fn render(width: u32, height: u32) -> (Vec<u8>, og_render::RenderInfo) {
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
    scene.overlay = Some(overlay);
    let bg = ui.theme().palette.backdrop.linear();
    scene.environment.background = Background::Color(og_core::glam::Vec3::new(bg[0], bg[1], bg[2]));
    scene.post.tonemap = Tonemap::None;
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width,
        height,
        ..Default::default()
    })
    .expect("a headless adapter");
    renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
    let pixels = renderer.read_offscreen().unwrap().pixels;
    (pixels, renderer.info().clone())
}

#[test]
#[ignore = "requires GPU"]
fn css_filters_match_their_reference_frames() {
    for (width, height) in [(1280u32, 720u32), (390, 844)] {
        let (pixels, info) = render(width, height);
        let name = format!("css-filters-{width}.png");
        image::save_buffer(
            out_dir().join(&name),
            &pixels,
            width,
            height,
            image::ColorType::Rgba8,
        )
        .unwrap();
        let own = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("references/metal")
            .join(&name);
        if std::env::var_os("OG_UI_CAPTURE").is_some() {
            record_reference(&own, width, height, &pixels);
        }
        let expected = image::open(&own)
            .unwrap_or_else(|e| panic!("{}: {e} (OG_UI_CAPTURE=1 records it)", own.display()))
            .to_rgba8();
        let (rmse, _) = compare(&pixels, expected.as_raw(), &info, &own)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        println!("{name}: RMSE {rmse:.6}");
    }
}
