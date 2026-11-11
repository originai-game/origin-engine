//! How the frame is presented (`og.render.setDisplay`, `og.render.display`):
//! the player's graphics settings (the frame cap, battery saver, render
//! scale, automatic quality (og-render's `AutoQuality`), the graphics
//! preset, fullscreen and the GPU's power preference), and what a
//! presenting host reports back (the display's rate, device losses, GPU
//! memory, whether the page is fullscreen, the adapter's power preference);
//! and the host's mouse (`og.input.setMouse`: raw input, what the browser
//! granted).
//!
//! Presentation only. The settings live in the engine's render state, not
//! in the world, so the world's hash, snapshots, saves and replays never
//! see them; and only a host that presents to a display
//! (`RenderBackend::presents`: the web canvas) applies the render scale and
//! automatic quality, so a native run (`og run --commands`, `og test`, the
//! reference captures) draws every frame at its fixed size and scale 1.
//! The frame cap is the host's to keep (the web Worker skips display
//! frames, `packages/engine/src/web/worker.ts`); the engine answers it with
//! every frame (`maxFps`), and with it what the page does for the player:
//! `fullscreen` (the shell enters or leaves it), `lowPower` (the shell
//! keeps it for the next load: the adapter is chosen once, at the boot)
//! and `rawMouse` (pointer lock's unadjustedMovement). The graphics preset
//! (`quality`) is drawn on every host, as RenderSettings' quality is.

use super::engine::engine;
use super::extract::RenderState;
use super::wire::ToJson;
use super::{ok_void, refuse, wire};
use og_render::{AutoQuality, AutoQualityBounds, FrameSample, QualityPreset};
use serde_json::{Value, json};

/// The IDL `FrameCap`'s variants, in order.
const FRAME_CAPS: &[&str] = &["auto", "display", "fps60", "fps30", "fps120", "fps144"];

/// The IDL `GraphicsQuality`'s variants, in order, with og-render's preset.
const PRESETS: [(&str, QualityPreset); 3] = [
    ("medium", QualityPreset::Medium),
    ("low", QualityPreset::Low),
    ("high", QualityPreset::High),
];

/// The IDL `PowerPreference` of a low-power choice.
fn power(low_power: bool) -> wire::PowerPreference {
    if low_power {
        wire::PowerPreference::LowPower
    } else {
        wire::PowerPreference::HighPerformance
    }
}

/// The battery saver's frame rate and largest render scale.
const SAVER_FPS: u32 = 30;
const SAVER_SCALE: f32 = 0.75;

/// The display's interval taken where the host measured none (60 Hz).
const UNKNOWN_DISPLAY_MS: f64 = 1000.0 / 60.0;

/// The settings as set (`DisplaySettings`), every field filled.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Settings {
    pub frame_cap: usize,
    pub battery_saver: bool,
    pub render_scale: f32,
    pub cloth_subdivisions: u32,
    pub auto_quality: bool,
    pub auto_min_scale: f32,
    pub auto_preset_steps: u8,
    /// The player's graphics preset, drawn in place of RenderSettings'
    /// quality fields; `None` draws the game's.
    pub quality: Option<QualityPreset>,
    /// The player wants the page fullscreen.
    pub fullscreen: bool,
    /// The GPU adapter the next load asks for: low power (the integrated
    /// GPU of a laptop with two) instead of high performance.
    pub low_power: bool,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            frame_cap: 0,
            battery_saver: false,
            render_scale: 1.0,
            cloth_subdivisions: 0,
            auto_quality: true,
            auto_min_scale: og_render::MIN_RENDER_SCALE,
            auto_preset_steps: 1,
            quality: None,
            fullscreen: false,
            low_power: false,
        }
    }
}

impl Settings {
    /// A `DisplaySettings` desc over `base` (the settings now): fields left
    /// out (or null) keep `base`'s values, but `quality: null`, which is the
    /// game's own quality; a value out of its bounds takes its nearest
    /// bound.
    pub fn parse(d: &Value, base: &Settings) -> Result<Settings, String> {
        if !d.is_object() && !d.is_null() {
            return Err(format!("display settings are an object, not {d}"));
        }
        let t: wire::DisplaySettings = wire::value(d, "settings")?;
        // Patch: a field left out (or null) keeps `base`'s value.
        let given = |k: &str| d.get(k).is_some_and(|v| !v.is_null());
        let frame_cap = if given("frameCap") {
            t.frame_cap as usize
        } else {
            base.frame_cap
        };
        // `quality: null` is the game's own quality.
        let quality = if d.get("quality").is_some() {
            t.quality.map(|q| PRESETS[q as usize].1)
        } else {
            base.quality
        };
        let low_power = if given("powerPreference") {
            t.power_preference == wire::PowerPreference::LowPower
        } else {
            base.low_power
        };
        let scale = |v: f32| v.clamp(og_render::MIN_RENDER_SCALE, og_render::MAX_RENDER_SCALE);
        Ok(Settings {
            frame_cap,
            quality,
            low_power,
            fullscreen: if given("fullscreen") {
                t.fullscreen
            } else {
                base.fullscreen
            },
            battery_saver: if given("batterySaver") {
                t.battery_saver
            } else {
                base.battery_saver
            },
            render_scale: scale(if given("renderScale") {
                t.render_scale
            } else {
                base.render_scale
            }),
            cloth_subdivisions: if given("clothSubdivisions") {
                t.cloth_subdivisions.min(8)
            } else {
                base.cloth_subdivisions
            },
            auto_quality: if given("autoQuality") {
                t.auto_quality
            } else {
                base.auto_quality
            },
            auto_min_scale: scale(if given("autoMinScale") {
                t.auto_min_scale
            } else {
                base.auto_min_scale
            }),
            auto_preset_steps: if given("autoPresetSteps") {
                t.auto_preset_steps.min(2) as u8
            } else {
                base.auto_preset_steps
            },
        })
    }

    fn wire(&self) -> wire::DisplaySettings {
        wire::DisplaySettings {
            frame_cap: wire::FrameCap::ALL[self.frame_cap],
            battery_saver: self.battery_saver,
            render_scale: self.render_scale,
            cloth_subdivisions: self.cloth_subdivisions,
            auto_quality: self.auto_quality,
            auto_min_scale: self.auto_min_scale,
            auto_preset_steps: u32::from(self.auto_preset_steps),
            quality: self.quality.map(|q| {
                let i = PRESETS.iter().position(|(_, p)| *p == q).unwrap_or(0);
                wire::GraphicsQuality::ALL[i]
            }),
            fullscreen: self.fullscreen,
            power_preference: power(self.low_power),
        }
    }

    /// The most frames a second (0: the display's rate) on a device of
    /// this class (`phone`: a coarse pointer).
    pub fn max_fps(&self, phone: bool) -> u32 {
        let cap = match FRAME_CAPS[self.frame_cap] {
            "display" => 0,
            "fps60" => 60,
            "fps30" => 30,
            "fps120" => 120,
            "fps144" => 144,
            _ if phone => 60,
            _ => 0,
        };
        if self.battery_saver { SAVER_FPS } else { cap }
    }

    /// The render scale at the top of automatic quality's ladder (and the
    /// one drawn without it).
    pub fn max_scale(&self) -> f32 {
        if self.battery_saver {
            self.render_scale.min(SAVER_SCALE)
        } else {
            self.render_scale
        }
    }

    pub fn auto(&self) -> bool {
        self.auto_quality || self.battery_saver
    }
}

/// What the engine keeps of its display.
#[derive(Default)]
pub(crate) struct DisplayState {
    pub settings: Settings,
    /// The display's interval as the host measured it, ms (0: unknown).
    pub display_ms: f64,
    pub auto: Option<AutoQuality>,
    /// The host steps this frame for a script or a harness (`og run
    /// --commands`, the web Worker's probe `frame`), not for the display:
    /// it draws at the configured quality and is no sample of automatic
    /// quality, whose controller is kept for the next display frame.
    pub scripted: bool,
    /// Capture-only: the host's frames step the ticks and draw nothing;
    /// what is seen is a probe capture's ([`set_capture_only`]).
    pub capture_only: bool,
    /// The render scale and preset steps the last frame drew with.
    pub scale: f32,
    pub step: u8,
    /// The page is fullscreen now, as the host last reported it.
    pub fullscreen: bool,
    /// The power preference the host's GPU adapter was made with (`true`:
    /// low power); `None` where the host made none by preference (a
    /// native run).
    pub booted_low_power: Option<bool>,
    /// The host's mouse (`og.input.setMouse`, `og.input.mouse`).
    pub mouse: Mouse,
}

/// The host's mouse while the pointer is locked: the player's raw-input
/// setting and what the browser granted (IDL `input.MouseInfo`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Mouse {
    pub raw: bool,
    pub raw_active: bool,
    pub raw_unsupported: bool,
}

impl Default for Mouse {
    fn default() -> Mouse {
        Mouse {
            raw: true,
            raw_active: false,
            raw_unsupported: false,
        }
    }
}

impl DisplayState {
    fn bounds(&self, phone: bool) -> AutoQualityBounds {
        let display = if self.display_ms > 0.0 {
            self.display_ms
        } else {
            UNKNOWN_DISPLAY_MS
        };
        let fps = self.settings.max_fps(phone);
        let max_scale = self.settings.max_scale();
        AutoQualityBounds {
            max_scale,
            min_scale: self.settings.auto_min_scale.min(max_scale),
            preset_steps: self.settings.auto_preset_steps,
            // A cap faster than the display is the display's interval.
            budget_ms: if fps > 0 {
                (1000.0 / f64::from(fps)).max(display)
            } else {
                display
            },
        }
    }
}

fn phone(rs: &RenderState) -> bool {
    rs.viewport.as_ref().is_some_and(|v| v.coarse_pointer)
}

/// Before a frame is drawn, after extraction: on a presenting host, the
/// render scale and the preset cap automatic quality chose (or the
/// settings' scale without it). Elsewhere nothing: scale 1, the game's
/// quality.
pub(crate) fn before_render(rs: &mut RenderState) {
    let level = rs.display.settings.cloth_subdivisions;
    rs.scene.objects.cloth_subdivisions = if level == 0 { 2 } else { level };
    // The player's preset replaces the game's quality fields on every host,
    // as RenderSettings' own do; automatic quality's steps cap it below.
    if let Some(q) = rs.display.settings.quality {
        rs.scene.quality = og_render::RenderQuality::preset(q);
    }
    let Some(presents) = rs.backend.as_ref().map(|b| b.presents()) else {
        return;
    };
    if !presents {
        return;
    }
    let phone = phone(rs);
    let d = &mut rs.display;
    let (scale, step) = if d.scripted {
        (d.settings.max_scale(), 0)
    } else if d.settings.auto() {
        let bounds = d.bounds(phone);
        let auto = d.auto.get_or_insert_with(|| AutoQuality::new(bounds));
        if auto.bounds() != bounds {
            auto.set_bounds(bounds);
        }
        (auto.scale(), auto.step())
    } else {
        d.auto = None;
        (d.settings.max_scale(), 0)
    };
    d.scale = scale;
    d.step = step;
    if step > 0 {
        rs.scene.objects.cloth_subdivisions = rs
            .scene
            .objects
            .cloth_subdivisions
            .min(if step >= 2 { 1 } else { 2 });
        let preset = if step >= 2 {
            QualityPreset::Low
        } else {
            QualityPreset::Medium
        };
        rs.scene.quality = rs.scene.quality.capped(preset);
    }
    if let Some(b) = rs.backend.as_mut() {
        b.set_render_scale(scale);
    }
}

/// After a drawn frame: its interval (ms since the last frame), the
/// engine's work on it and the GPU's time where measured, for automatic
/// quality.
pub(crate) fn after_frame(
    rs: &mut RenderState,
    interval_ms: f64,
    cpu_ms: f64,
    gpu_ms: Option<f64>,
) {
    if rs.display.scripted || interval_ms <= 0.0 {
        return;
    }
    if let Some(auto) = rs.display.auto.as_mut() {
        auto.sample(FrameSample {
            interval_ms,
            cpu_ms,
            gpu_ms,
        });
    }
}

/// The frame cap resolved for engine `id` (0: the display's rate): the
/// frame answer's `maxFps`.
pub(crate) fn max_fps(rs: &RenderState) -> u32 {
    rs.display.settings.max_fps(phone(rs))
}

/// Brackets the frames a host steps for a script or a harness, not for the
/// display: a scripted run's frame time is its machine's load, so letting
/// automatic quality read it made the same script draw at another render
/// scale from run to run (the web tiers' looks and Meridian, 2026-11-06).
/// Host-only, beside [`set_display_interval`]; the game API is unchanged.
pub fn set_scripted_frames(id: u32, on: bool) {
    if let Ok(eng) = engine(id) {
        eng.render.borrow_mut().display.scripted = on;
    }
}

/// Capture-only frames (a script's or a harness's run that reads frames
/// only through the probe's captures): a frame steps its ticks, and with
/// them every presentation state stepped by the tick (og.motion, the
/// particles' histories), but draws nothing. A capture is a function of
/// the world and the presentation checkpoint (docs/design.md §2.4), so its
/// PNG is the one a run drawing every frame captures. Host-only.
pub fn set_capture_only(id: u32, on: bool) {
    if let Ok(eng) = engine(id) {
        eng.render.borrow_mut().display.capture_only = on;
    }
}

/// The display's interval as the host measured it (the web Worker's
/// requestAnimationFrame), ms; 0 or less is unknown.
pub fn set_display_interval(id: u32, ms: f64) {
    if let Ok(eng) = engine(id)
        && let Ok(mut rs) = eng.render.try_borrow_mut()
    {
        rs.display.display_ms = if ms.is_finite() { ms.max(0.0) } else { 0.0 };
    }
}

/// The page went fullscreen or left it (the web shell's
/// `fullscreenchange`): `DisplayInfo.fullscreen`, and the player's setting
/// follows it, so a player who leaves with Escape finds Fullscreen off on
/// the settings page and the shell does not ask for it again.
pub fn set_fullscreen(id: u32, on: bool) {
    if let Ok(eng) = engine(id)
        && let Ok(mut rs) = eng.render.try_borrow_mut()
    {
        rs.display.fullscreen = on;
        rs.display.settings.fullscreen = on;
    }
}

/// The power preference the host's GPU adapter was made with (`true`: low
/// power), at the boot: the settings take it too, so the page reports no
/// restart until the player picks the other.
pub fn set_power_preference(id: u32, low_power: bool) {
    if let Ok(eng) = engine(id)
        && let Ok(mut rs) = eng.render.try_borrow_mut()
    {
        rs.display.booted_low_power = Some(low_power);
        rs.display.settings.low_power = low_power;
    }
}

/// What the browser granted the pointer lock (the web shell's
/// requestPointerLock): raw movement now (`active`), and whether it
/// refused unadjustedMovement (`unsupported`).
pub fn set_mouse_report(id: u32, active: bool, unsupported: bool) {
    if let Ok(eng) = engine(id)
        && let Ok(mut rs) = eng.render.try_borrow_mut()
    {
        rs.display.mouse.raw_active = active;
        rs.display.mouse.raw_unsupported = unsupported;
    }
}

/// What the frame answer tells the page: fullscreen wanted, the power
/// preference for the next load, raw mouse input.
pub(crate) fn page_asks(rs: &RenderState) -> Value {
    json!({
        "fullscreen": rs.display.settings.fullscreen,
        "lowPower": rs.display.settings.low_power,
        "rawMouse": rs.display.mouse.raw,
    })
}

/// Destroy the renderer's device (a test hook: the device-loss checks of
/// the web host, `og_web_lose_device`); the next frame finds it lost and
/// the host makes the renderer again. False without a renderer.
pub fn lose_device(id: u32) -> bool {
    engine(id).is_ok_and(|eng| {
        eng.render
            .try_borrow_mut()
            .is_ok_and(|mut rs| rs.backend.as_mut().is_some_and(|b| b.lose_device()))
    })
}

/// The renderer's GPU memory as `GpuMemory`, or null without one.
pub(crate) fn gpu_memory(rs: &RenderState) -> Value {
    gpu(rs).to_json()
}

fn gpu(rs: &RenderState) -> Option<wire::GpuMemory> {
    let m = rs.backend.as_ref().and_then(|b| b.gpu_memory())?;
    Some(wire::GpuMemory {
        textures: m.textures as f64,
        meshes: m.meshes as f64,
        render_targets: m.render_targets as f64,
        buffers: m.buffers as f64,
        total: m.total() as f64,
    })
}

/// `og.render.setDisplay`.
pub fn og_render_set_display(world: u32, settings: Option<String>) -> String {
    let r = (|| -> Result<(), String> {
        let eng = engine(world)?;
        let d = super::parse(settings.as_deref())?;
        let mut rs = eng
            .render
            .try_borrow_mut()
            .map_err(|_| "the renderer is busy drawing".to_string())?;
        let s = Settings::parse(&d, &rs.display.settings)?;
        rs.display.settings = s;
        Ok(())
    })();
    match r {
        Ok(()) => ok_void(),
        Err(e) => refuse(e),
    }
}

/// `og.render.resetDisplay`: every setting back to its default.
pub fn og_render_reset_display(world: u32) -> String {
    let r = engine(world).and_then(|eng| {
        let mut rs = eng
            .render
            .try_borrow_mut()
            .map_err(|_| "the renderer is busy drawing".to_string())?;
        rs.display.settings = Settings::default();
        Ok(())
    });
    match r {
        Ok(()) => ok_void(),
        Err(e) => refuse(e),
    }
}

/// `og.render.display`.
pub fn og_render_display(world: u32) -> String {
    let Ok(eng) = engine(world) else {
        return Value::Null.to_string();
    };
    let Ok(rs) = eng.render.try_borrow() else {
        return Value::Null.to_string();
    };
    let d = &rs.display;
    let presenting = rs.backend.as_ref().is_some_and(|b| b.presents());
    let (w, h) = rs.backend.as_ref().map_or((0, 0), |b| b.scene_size());
    wire::text_of(&wire::DisplayInfo {
        settings: d.settings.wire(),
        presenting,
        max_fps: max_fps(&rs),
        display_hz: if d.display_ms > 0.0 {
            (1000.0 / d.display_ms) as f32
        } else {
            0.0
        },
        render_scale: if presenting { d.scale } else { 1.0 },
        preset_steps: if presenting { u32::from(d.step) } else { 0 },
        scene_width: w,
        scene_height: h,
        frame_ms: d.auto.as_ref().map_or(0.0, |a| a.cost_ms()) as f32,
        device_losses: rs.backend.as_ref().map_or(0, |b| b.device_losses()),
        gpu: gpu(&rs),
        // A browser presents every frame on the display's refresh and has
        // no switch for it; a native run presents nothing.
        vsync: presenting,
        fullscreen: d.fullscreen,
        power_preference: power(d.booted_low_power.unwrap_or(false)),
        restart_needed: d
            .booted_low_power
            .is_some_and(|b| b != d.settings.low_power),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The device-class defaults and the battery saver: auto is 60 on a
    /// phone and the display's rate on a desktop; the saver is 30 and at
    /// most 0.75 of the scale whatever else is set.
    #[test]
    fn the_frame_cap_resolves_by_device_class_and_battery() {
        let s = Settings::default();
        assert_eq!((s.max_fps(true), s.max_fps(false)), (60, 0));
        let d = Settings::parse(&json!({ "frameCap": "display" }), &Settings::default()).unwrap();
        assert_eq!(d.max_fps(true), 0);
        let saver = Settings::parse(&json!({ "frameCap": "display", "batterySaver": true, "renderScale": 1.5, "autoQuality": false }), &Settings::default()).unwrap();
        assert_eq!(saver.max_fps(false), 30);
        assert_eq!(saver.max_scale(), 0.75);
        assert!(saver.auto());
        assert!(Settings::parse(&json!({ "frameCap": "fps90" }), &Settings::default()).is_err());
        let wild = Settings::parse(
            &json!({ "renderScale": 9, "autoMinScale": 0, "autoPresetSteps": 7 }),
            &Settings::default(),
        )
        .unwrap();
        assert_eq!(
            (
                wild.render_scale,
                wild.auto_min_scale,
                wild.auto_preset_steps
            ),
            (2.0, 0.5, 2)
        );
    }

    /// The budget is the cap's interval, never shorter than the display's
    /// (60 on a 30 Hz display is 33 ms), and the display's at no cap.
    #[test]
    fn the_budget_is_the_cap_or_the_display() {
        let mut d = DisplayState {
            display_ms: 1000.0 / 120.0,
            ..Default::default()
        };
        assert!((d.bounds(true).budget_ms - 1000.0 / 60.0).abs() < 1e-9);
        assert!((d.bounds(false).budget_ms - 1000.0 / 120.0).abs() < 1e-9);
        d.display_ms = 1000.0 / 30.0;
        assert!((d.bounds(true).budget_ms - 1000.0 / 30.0).abs() < 1e-9);
        d.display_ms = 0.0;
        assert!((d.bounds(false).budget_ms - UNKNOWN_DISPLAY_MS).abs() < 1e-9);
    }
}
