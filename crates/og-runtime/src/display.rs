//! How the frame is presented (`og.render.setDisplay`, `og.render.display`):
//! the frame cap, battery saver, render scale and automatic quality
//! (og-render's `AutoQuality`), and what a presenting host reports back
//! (the display's rate, device losses, GPU memory).
//!
//! Presentation only. The settings live in the engine's render state, not
//! in the world, so the world's hash, snapshots, saves and replays never
//! see them; and only a host that presents to a display
//! (`RenderBackend::presents`: the web canvas) applies the render scale and
//! automatic quality, so a native run (`og run --commands`, `og test`, the
//! reference captures) draws every frame at its fixed size and scale 1.
//! The frame cap is the host's to keep (the web Worker skips display
//! frames, `packages/engine/src/web/worker.ts`); the engine answers it with
//! every frame (`maxFps`).

use super::engine::engine;
use super::extract::RenderState;
use super::{ok_void, refuse};
use og_render::{AutoQuality, AutoQualityBounds, FrameSample, QualityPreset};
use serde_json::{Value, json};

/// The IDL `FrameCap`'s variants, in order.
const FRAME_CAPS: &[&str] = &["auto", "display", "fps60", "fps30"];

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
    pub auto_quality: bool,
    pub auto_min_scale: f32,
    pub auto_preset_steps: u8,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            frame_cap: 0,
            battery_saver: false,
            render_scale: 1.0,
            auto_quality: true,
            auto_min_scale: og_render::MIN_RENDER_SCALE,
            auto_preset_steps: 1,
        }
    }
}

fn scale(v: &Value, default: f32) -> f32 {
    v.as_f64()
        .map_or(default, |x| x as f32)
        .clamp(og_render::MIN_RENDER_SCALE, og_render::MAX_RENDER_SCALE)
}

impl Settings {
    /// A `DisplaySettings` desc: fields left out take their defaults, a
    /// value out of its bounds its nearest bound.
    pub fn parse(d: &Value) -> Result<Settings, String> {
        let base = Settings::default();
        let frame_cap = match &d["frameCap"] {
            Value::Null => 0,
            Value::String(s) => FRAME_CAPS.iter().position(|c| c == s).ok_or_else(|| {
                format!("'{s}' is not a frame cap (auto, display, fps60 or fps30)")
            })?,
            Value::Number(n) => n
                .as_u64()
                .map_or(0, |i| i as usize)
                .min(FRAME_CAPS.len() - 1),
            other => return Err(format!("frameCap {other} is not a frame cap")),
        };
        let flag = |k: &str, default: bool| d[k].as_bool().unwrap_or(default);
        Ok(Settings {
            frame_cap,
            battery_saver: flag("batterySaver", base.battery_saver),
            render_scale: scale(&d["renderScale"], base.render_scale),
            auto_quality: flag("autoQuality", base.auto_quality),
            auto_min_scale: scale(&d["autoMinScale"], base.auto_min_scale),
            auto_preset_steps: d["autoPresetSteps"]
                .as_f64()
                .map_or(base.auto_preset_steps, |x| x.clamp(0.0, 2.0) as u8),
        })
    }

    fn json(&self) -> Value {
        json!({
            "frameCap": FRAME_CAPS[self.frame_cap],
            "batterySaver": self.battery_saver,
            "renderScale": self.render_scale,
            "autoQuality": self.auto_quality,
            "autoMinScale": self.auto_min_scale,
            "autoPresetSteps": self.auto_preset_steps,
        })
    }

    /// The most frames a second (0: the display's rate) on a device of
    /// this class (`phone`: a coarse pointer).
    pub fn max_fps(&self, phone: bool) -> u32 {
        let cap = match FRAME_CAPS[self.frame_cap] {
            "display" => 0,
            "fps60" => 60,
            "fps30" => 30,
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
    /// The render scale and preset steps the last frame drew with.
    pub scale: f32,
    pub step: u8,
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
    let Some(presents) = rs.backend.as_ref().map(|b| b.presents()) else {
        return;
    };
    if !presents {
        return;
    }
    let phone = phone(rs);
    let d = &mut rs.display;
    let (scale, step) = if d.settings.auto() {
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
    if interval_ms <= 0.0 {
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

/// The display's interval as the host measured it (the web Worker's
/// requestAnimationFrame), ms; 0 or less is unknown.
pub fn set_display_interval(id: u32, ms: f64) {
    if let Ok(eng) = engine(id)
        && let Ok(mut rs) = eng.render.try_borrow_mut()
    {
        rs.display.display_ms = if ms.is_finite() { ms.max(0.0) } else { 0.0 };
    }
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
    match rs.backend.as_ref().and_then(|b| b.gpu_memory()) {
        Some(m) => json!({
            "textures": m.textures as f64,
            "meshes": m.meshes as f64,
            "renderTargets": m.render_targets as f64,
            "buffers": m.buffers as f64,
            "total": m.total() as f64,
        }),
        None => Value::Null,
    }
}

/// `og.render.setDisplay`.
pub fn og_render_set_display(world: u32, settings: Option<String>) -> String {
    let r = (|| -> Result<(), String> {
        let eng = engine(world)?;
        let d = super::parse(settings.as_deref())?;
        let s = Settings::parse(&d)?;
        let mut rs = eng
            .render
            .try_borrow_mut()
            .map_err(|_| "the renderer is busy drawing".to_string())?;
        rs.display.settings = s;
        Ok(())
    })();
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
    json!({
        "settings": d.settings.json(),
        "presenting": presenting,
        "maxFps": max_fps(&rs),
        "displayHz": if d.display_ms > 0.0 { 1000.0 / d.display_ms } else { 0.0 },
        "renderScale": if presenting { d.scale } else { 1.0 },
        "presetSteps": if presenting { d.step } else { 0 },
        "sceneWidth": w,
        "sceneHeight": h,
        "frameMs": d.auto.as_ref().map_or(0.0, |a| a.cost_ms()),
        "deviceLosses": rs.backend.as_ref().map_or(0, |b| b.device_losses()),
        "gpu": gpu_memory(&rs),
    })
    .to_string()
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
        let d = Settings::parse(&json!({ "frameCap": "display" })).unwrap();
        assert_eq!(d.max_fps(true), 0);
        let saver = Settings::parse(&json!({ "frameCap": "display", "batterySaver": true, "renderScale": 1.5, "autoQuality": false })).unwrap();
        assert_eq!(saver.max_fps(false), 30);
        assert_eq!(saver.max_scale(), 0.75);
        assert!(saver.auto());
        assert!(Settings::parse(&json!({ "frameCap": "fps90" })).is_err());
        let wild =
            Settings::parse(&json!({ "renderScale": 9, "autoMinScale": 0, "autoPresetSteps": 7 }))
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
