//! The frame's post stack, 2D lighting and stylise settings, read from the world's resources.

use super::*;

/// `PostEffects` and `Lighting2D` into the scene's post stack, shake and 2D lighting.
pub(crate) fn settings(world: &World, rs: &mut RenderState) {
    let RenderState {
        fx,
        scene,
        draw,
        settings: render_settings,
        textures,
        art,
        ..
    } = rs;
    let layouts = &draw.fx;
    // RenderSettings.decalAtlas: the texture Decal components sample.
    scene.effects.decal_atlas = resource(world, render_settings.as_ref())
        .and_then(|s| record::RenderSettings::from_json(&s).ok())
        .and_then(|s| crate::assets::index_of(s.decal_atlas as f64))
        .and_then(|i| textures.get(i).copied());
    let post = &mut scene.look.post;
    post.ao = None;
    post.outline = None;
    post.depth_of_field = None;
    post.motion_blur = None;
    post.god_rays = None;
    post.lens_flare = None;
    post.ssr = None;
    post.gi = None;
    post.gi_probes = None;
    post.glow = None;
    post.color_grading = None;
    post.color_adjust = None;
    post.vignette = None;
    post.chromatic_aberration = None;
    post.lens_distortion = None;
    post.screen_blur = None;
    post.film_grain = None;
    post.sharpen = None;
    post.flash = None;
    scene.frame.shake = CameraShake::default();
    scene.sprite.lighting_2d = None;
    if let Some(p) = resource(world, layouts.post.as_ref())
        && let Ok(p) = record::PostEffects::from_json(&p)
    {
        let rgb = |c: &wire::Color| Vec3::new(c.r, c.g, c.b);
        let resolution = |r: wire::AoResolution| match r {
            wire::AoResolution::Full => AoResolution::Full,
            wire::AoResolution::Half => AoResolution::Half,
        };
        if p.ao {
            post.ao = Some(AmbientOcclusion {
                radius: p.ao_radius,
                intensity: p.ao_intensity,
                method: match p.ao_method {
                    wire::AoMethod::Auto => AoMethod::Auto,
                    wire::AoMethod::Gtao => AoMethod::Gtao,
                    wire::AoMethod::Ssao => AoMethod::Ssao,
                    wire::AoMethod::Screen => AoMethod::Screen,
                },
                resolution: resolution(p.ao_resolution),
                direct: p.ao_direct.clamp(0.0, 1.0),
                radius_scale: p.ao_radius_scale.max(0.0),
            });
        }
        if p.outline {
            let c = rgb(&p.outline_color);
            post.outline = Some(Outline {
                color: [c.x, c.y, c.z, 1.0],
                width: p.outline_width,
                depth_threshold: p.outline_depth_threshold,
                normal_threshold: p.outline_normal_threshold,
            });
        }
        if p.depth_of_field {
            post.depth_of_field = Some(DepthOfField {
                focus_distance: p.dof_focus_distance,
                focus_range: p.dof_focus_range,
                max_blur: p.dof_max_blur,
                falloff: p.dof_falloff,
                quality: match p.dof_quality {
                    wire::DofQuality::High => DofQuality::High,
                    wire::DofQuality::Low => DofQuality::Low,
                },
            });
        }
        if p.motion_blur {
            post.motion_blur = Some(MotionBlur {
                shutter: p.motion_blur_shutter,
                samples: p.motion_blur_samples,
                max_pixels: p.motion_blur_max_pixels,
            });
        }
        let blur_kind = match p.screen_blur {
            wire::ScreenBlur::None => None,
            wire::ScreenBlur::Zoom => Some(ScreenBlurKind::Zoom),
            wire::ScreenBlur::Spin => Some(ScreenBlurKind::Spin),
            wire::ScreenBlur::Directional => Some(ScreenBlurKind::Directional),
        };
        let blur_strength = p.screen_blur_strength;
        if let Some(kind) = blur_kind
            && blur_strength != 0.0
        {
            post.screen_blur = Some(ScreenBlur {
                kind,
                strength: blur_strength,
                center: p.screen_blur_center,
                angle: p.screen_blur_angle.to_radians(),
                samples: p.screen_blur_samples,
            });
        }
        if p.god_rays {
            post.god_rays = Some(GodRays {
                intensity: p.god_rays_intensity,
                decay: p.god_rays_decay,
                density: p.god_rays_density,
                tint: rgb(&p.god_rays_tint),
                samples: p.god_rays_samples,
                threshold: p.god_rays_threshold,
            });
        }
        if p.ssr {
            post.ssr = Some(og_render::ScreenSpaceReflections {
                max_distance: p.ssr_max_distance,
                thickness: p.ssr_thickness,
                steps: p.ssr_steps,
                fade: p.ssr_fade,
                intensity: p.ssr_intensity,
                max_roughness: p.ssr_max_roughness,
                resolution: resolution(p.ssr_resolution),
            });
        }
        if p.glow {
            post.glow = Some(Glow {
                intensity: p.glow_intensity,
            });
        }
        if let Some(lut) = crate::assets::index_of(p.lut as f64)
            .and_then(|i| fx.luts.get(i))
            .and_then(|l| l.handle)
        {
            post.color_grading = Some(ColorGrading {
                lut,
                intensity: p.lut_intensity,
            });
        }
        let adjust = ColorAdjust {
            temperature: p.temperature,
            tint: p.tint,
            brightness: p.brightness,
            contrast: p.contrast,
            shadows: p.balance_shadows,
            midtones: p.balance_midtones,
            highlights: p.balance_highlights,
            preserve_luminosity: p.preserve_luminosity,
            hue: p.hue,
            saturation: p.saturation,
            vibrance: p.vibrance,
            grayscale: p.grayscale,
            invert: p.invert,
            threshold: p.threshold,
        };
        post.color_adjust = (!adjust.is_identity()).then_some(adjust);
        if p.vignette {
            post.vignette = Some(Vignette {
                intensity: p.vignette_intensity,
                radius: p.vignette_radius,
                smoothness: p.vignette_smoothness,
                color: rgb(&p.vignette_color),
            });
        }
        let aberration = p.chromatic_aberration;
        if aberration > 0.0 {
            post.chromatic_aberration = Some(ChromaticAberration {
                intensity: aberration,
            });
        }
        let lens = p.lens_distortion;
        if lens != 0.0 {
            post.lens_distortion = Some(LensDistortion {
                amount: lens,
                center: p.lens_distortion_center,
            });
        }
        let grain = p.film_grain;
        if grain > 0.0 {
            post.film_grain = Some(FilmGrain {
                intensity: grain,
                size: p.film_grain_size,
            });
        }
        let sharpen = p.sharpen;
        if sharpen >= 0.0 {
            post.sharpen = Some(sharpen);
        }
        let flash = p.flash;
        if flash > 0.0 {
            post.flash = Some(ScreenFlash {
                color: rgb(&p.flash_color),
                amount: flash,
            });
        }
        let trauma = p.shake_trauma;
        if trauma > 0.0 {
            scene.frame.shake = CameraShake::trauma(
                trauma,
                scene.frame.tick,
                scene.frame.tick_rate,
                u64::from(p.shake_seed),
                p.shake_max_offset,
                p.shake_max_roll,
                p.shake_frequency,
            );
        }
    }
    scene.look.stylise = stylise(
        resource(world, layouts.stylise.as_ref())
            .and_then(|s| record::Stylise::from_json(&s).ok())
            .as_ref(),
        resource(world, layouts.material_style.as_ref())
            .and_then(|m| record::MaterialStyle::from_json(&m).ok())
            .as_ref(),
        &fx.look.palettes,
        textures,
    );
    // Painted 2D (crate::art::texel_paint): the cook painted and washed
    // each texture in its own texels, and the frame draws neither pass.
    if art.texel_paint {
        scene.look.stylise.paint = None;
    }
    if let Some(p) = resource(world, layouts.lens_flare.as_ref())
        && let Ok(p) = record::LensFlare::from_json(&p)
        && p.enabled
    {
        post.lens_flare = Some(LensFlare {
            intensity: p.intensity,
            ghosts: p.ghosts,
            ghost_spacing: p.ghost_spacing,
            ghost_threshold: p.ghost_threshold,
            halo_radius: p.halo_radius,
            halo_thickness: p.halo_thickness,
            halo_threshold: p.halo_threshold,
            chromatic_aberration: p.chromatic_aberration,
            starburst: p.starburst,
            sun: p.sun,
            lights: p.lights,
        });
    }
    // The art contract's line round meshes (ArtLine, art.rs).
    crate::art::mesh_line(&art.frame, &mut scene.look.stylise.material);
    // Under the art contract a frame that quantises without a palette of
    // its own takes the contract's (ArtPalette.colors): the colours the
    // cook put the art on are the ones the frame keeps.
    if let Some(q) = scene.look.stylise.quantize.as_mut()
        && q.palette.is_none()
    {
        q.palette = art.frame.palette;
    }
    if let Some(v) = resource(world, layouts.global_illumination.as_ref())
        && let Ok(g) = record::GlobalIllumination::from_json(&v)
        && g.enabled
        && gi_allowed(Some(&v), fx.gi_preset)
    {
        if g.method != wire::GiMethod::Screen {
            post.gi_probes = Some(og_render::ProbeGlobalIllumination {
                spacing: g.probe_spacing.max(0.05),
                cascades: g.probe_cascades.clamp(1, 4),
                rays: g.probe_rays.clamp(32, 256),
                intensity: g.intensity.max(0.0),
            });
        }
        if g.method != wire::GiMethod::Probes {
            post.gi = Some(og_render::ScreenSpaceGlobalIllumination {
                intensity: g.intensity,
                max_distance: g.max_distance,
                thickness: g.thickness,
                steps: g.steps,
                rays: g.rays,
                occlusion: g.occlusion.clamp(0.0, 1.0),
                resolution: match g.resolution {
                    wire::AoResolution::Full => AoResolution::Full,
                    wire::AoResolution::Half => AoResolution::Half,
                },
            });
        }
    }
    if let Some(l) = resource(world, layouts.lighting_2d.as_ref())
        && let Ok(l) = record::Lighting2D::from_json(&l)
        && l.enabled
    {
        scene.sprite.lighting_2d = Some(Lighting2d {
            ambient: Vec3::new(l.ambient.r, l.ambient.g, l.ambient.b),
            min_layer: l.min_layer,
            max_layer: l.max_layer,
            resolution: l.resolution,
            shadow_angles: l.shadow_angles,
        });
    }
}

/// Pixel snap (`Stylise.pixelSnap`): on a logical pixel grid with the 2D
/// camera (orthographic, looking down -z), the camera and every world
/// sprite move by whole logical pixels, so a scrolling scene never swims
/// between pixel rows and sprites keep their edges. Under a project's art
/// contract, `ArtCamera.pixelPerfect` does the same on the contract's grid.
pub(crate) fn snap_to_grid(rs: &mut RenderState) {
    let art = &rs.art.frame;
    let scene = &mut rs.scene;
    // The frame's grid (Stylise.pixelHeight with pixelSnap), else the art
    // contract's (ArtCamera.pixelPerfect: ArtPixels.logicalHeight rows of
    // the view, or 1 / ArtPixels.unit world units).
    let rows = match scene.look.stylise.pixel.filter(|p| p.snap && p.height > 0) {
        Some(p) => Some(p.height),
        None if art.pixel_perfect && art.logical_height > 0 => Some(art.logical_height),
        None => None,
    };
    let og_render::Projection::Orthographic { height } = scene.frame.camera.projection else {
        return;
    };
    let unit = match rows {
        Some(rows) => height / rows as f32,
        None if art.pixel_perfect && art.unit > 0.0 => 1.0 / art.unit,
        None => return,
    };
    let forward = (scene.frame.camera.target - scene.frame.camera.position).normalize_or_zero();
    if forward.z > -0.999 {
        return;
    }
    if unit <= 0.0 {
        return;
    }
    let snap = |v: f32| (v / unit).round() * unit;
    let c = &mut scene.frame.camera;
    let d = c.target - c.position;
    c.position.x = snap(c.position.x);
    c.position.y = snap(c.position.y);
    c.target = c.position + d;
    for s in &mut scene.sprite.sprites {
        s.position.x = snap(s.position.x);
        s.position.y = snap(s.position.y);
    }
}

/// `Stylise` and `MaterialStyle` as og-render's description (every
/// primitive `None` or zero unless its field turns it on).
pub(super) fn stylise(
    s: Option<&record::Stylise>,
    m: Option<&record::MaterialStyle>,
    palettes: &[(String, Option<og_render::PaletteHandle>)],
    textures: &[og_render::TextureHandle],
) -> og_render::Stylise {
    let rgb = |c: &wire::Color| [c.r, c.g, c.b];
    let texture = |h: u64| crate::assets::index_of(h as f64).and_then(|i| textures.get(i).copied());
    let mut out = og_render::Stylise::default();
    if let Some(s) = s {
        let height = s.pixel_height;
        if height > 0 {
            out.pixel = Some(og_render::PixelScale {
                height,
                upscale: s.pixel_upscale,
                snap: s.pixel_snap,
            });
        }
        let posterize = s.posterize;
        // A palette whose streaming group is not resident quantises to none.
        let palette = crate::assets::index_of(s.palette as f64).and_then(|i| palettes.get(i)?.1);
        let dither = match s.dither {
            wire::DitherPattern::Bayer2 => og_render::DitherPattern::Bayer2,
            wire::DitherPattern::Bayer4 => og_render::DitherPattern::Bayer4,
            wire::DitherPattern::Bayer8 => og_render::DitherPattern::Bayer8,
            wire::DitherPattern::BlueNoise => og_render::DitherPattern::BlueNoise,
            wire::DitherPattern::InterleavedGradient => {
                og_render::DitherPattern::InterleavedGradient
            }
            wire::DitherPattern::None => og_render::DitherPattern::None,
        };
        let space = |v: wire::StyliseSpace| match v {
            wire::StyliseSpace::Surface => og_render::StyliseSpace::Surface,
            wire::StyliseSpace::Object => og_render::StyliseSpace::Object,
            wire::StyliseSpace::Screen => og_render::StyliseSpace::Screen,
        };
        if posterize > 0 || palette.is_some() || dither != og_render::DitherPattern::None {
            out.quantize = Some(og_render::Quantize {
                posterize,
                mode: match s.posterize_mode {
                    wire::PosterizeMode::Tone => og_render::PosterizeMode::Tone,
                    wire::PosterizeMode::Channel => og_render::PosterizeMode::Channel,
                },
                palette,
                dither,
                dither_strength: s.dither_strength,
                dither_scale: s.dither_scale.max(1),
                space: space(s.dither_space),
            });
        }
        let halftone = s.halftone;
        if halftone > 0.0 {
            out.halftone = Some(og_render::Halftone {
                strength: halftone,
                size: s.halftone_size,
                angle_degrees: s.halftone_angle,
                color: rgb(&s.halftone_color),
                space: space(s.halftone_space),
                mode: match s.halftone_mode {
                    wire::HalftoneMode::Cmyk => og_render::HalftoneMode::Cmyk,
                    wire::HalftoneMode::Ink => og_render::HalftoneMode::Ink,
                },
                misregistration: s.halftone_misregistration,
                region: match s.halftone_region {
                    wire::HalftoneRegion::Shade => og_render::HalftoneRegion::Shade,
                    wire::HalftoneRegion::Highlight => og_render::HalftoneRegion::Highlight,
                    wire::HalftoneRegion::All => og_render::HalftoneRegion::All,
                },
            });
        }
        let hatching = s.hatching;
        if hatching > 0.0 {
            out.hatching = Some(og_render::Hatching {
                strength: hatching,
                scale: s.hatching_scale,
                levels: s.hatching_levels,
                angle_degrees: s.hatching_angle,
                color: rgb(&s.hatching_color),
                paper: rgb(&s.hatching_paper),
                space: space(s.hatching_space),
                depth: s.hatching_depth,
                keep_color: s.hatching_keep_color,
            });
        }
        let paint = og_render::Paint {
            radius: s.paint_radius,
            anisotropy: s.paint_anisotropy,
            edge_darkening: s.paint_edge_darkening,
            bleed: s.paint_bleed,
            canvas: texture(s.paint_canvas),
            canvas_strength: s.paint_canvas_strength,
            canvas_scale: s.paint_canvas_scale,
            canvas_space: space(s.paint_canvas_space),
            canvas_world_scale: s.paint_canvas_world_scale,
            detail: s.paint_detail,
            character_detail: s.paint_character_detail,
            watercolor: og_render::Watercolor {
                edge_darkening: s.watercolor_edge_darkening,
                granulation: s.watercolor_granulation,
                turbulence: s.watercolor_turbulence,
                dry_brush: s.watercolor_dry_brush,
                wobble: s.watercolor_wobble,
                dilution: s.watercolor_dilution,
                paper: rgb(&s.watercolor_paper),
            },
        };
        if paint.paints() || paint.watercolor.active() {
            out.paint = Some(paint);
        }
        if s.ink {
            out.ink = Some(og_render::Ink {
                threshold: s.ink_threshold,
                width: s.ink_width,
                color: rgb(&s.ink_color),
                depth: s.ink_depth,
                depth_threshold: s.ink_depth_threshold,
                normal_threshold: s.ink_normal_threshold,
            });
        }
        let bands = s.depth_bands;
        if bands > 0 {
            out.depth_bands = Some(og_render::DepthBands {
                count: bands,
                near: s.depth_bands_near,
                far: s.depth_bands_far,
                near_color: rgb(&s.depth_bands_near_color),
                far_color: rgb(&s.depth_bands_far_color),
                flatten: s.depth_bands_flatten,
            });
        }
        let retro = og_render::Retro3d {
            vertex_snap: s.vertex_snap,
            affine: s.affine_texturing,
            depth_bits: s.depth_bits,
        };
        if retro.vertex_snap > 0.0 || retro.affine > 0.0 || retro.depth_bits > 0 {
            out.retro = Some(retro);
        }
    }
    if let Some(m) = m {
        out.material = og_render::MaterialStyle {
            toon: m.toon,
            gradient_map: texture(m.gradient_map),
            shadow_tint: rgb(&m.shadow_tint),
            shadow_tint_amount: m.shadow_tint_amount,
            shadow_darkness: m.shadow_darkness,
            rim_color: rgb(&m.rim_color),
            rim_intensity: m.rim_intensity,
            rim_power: m.rim_power,
            outline_color: rgb(&m.outline_color),
            outline_width: m.outline_width,
            outline_distance: m.outline_distance,
            flat_shading: m.flat_shading,
            surface: og_render::MaterialSurface::from_index(m.surface as usize),
            surface_strength: m.surface_strength,
            surface_scale: m.surface_scale,
            toon_ambient: m.toon_ambient,
            toon_softness: m.toon_softness,
            outline_place: match m.outline_place {
                wire::OutlinePlace::Outside => og_render::OutlinePlace::Outside,
                wire::OutlinePlace::Both => og_render::OutlinePlace::Both,
                wire::OutlinePlace::Inside => og_render::OutlinePlace::Inside,
            },
            wireframe: m.wireframe,
            wireframe_angle: m.wireframe_angle,
            wireframe_color: rgb(&m.wireframe_color),
        };
    }
    out
}
