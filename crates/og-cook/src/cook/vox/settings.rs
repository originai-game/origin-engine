//! Voxel import settings and animation clip ranges.

/// Where a `.vox` file's origin is (`voxOrigin`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VoxOrigin {
    /// The editor's: each model about its centre (floored), at its `nTRN`
    /// translation.
    #[default]
    Center,
    /// The editor's in x and z, the whole file lifted so its lowest voxel
    /// stands on y = 0 (a prop or a character on the ground).
    Bottom,
}

/// A named range of an animated `.vox`'s frames (`voxClip walk 0 3`).
#[derive(Clone, Debug, PartialEq)]
pub struct VoxClip {
    pub name: String,
    /// The first frame and the last, both shown.
    pub first: u32,
    pub last: u32,
}

/// A `.vox` file's import settings: `model { voxScale 0.1 voxOrigin bottom }`.
#[derive(Clone, Debug, PartialEq)]
pub struct VoxSettings {
    /// Metres per voxel (default 1).
    pub scale: f32,
    pub origin: VoxOrigin,
    /// An animated file's frames per second (default 10).
    pub fps: f32,
    /// An animated file's clips; none: one clip `frames` of every frame.
    pub clips: Vec<VoxClip>,
}

/// The frame rate of an animated `.vox` without `voxFps`.
pub const DEFAULT_FPS: f32 = 10.0;

/// The clip of every frame of an animated `.vox` without `voxClip`.
pub const DEFAULT_CLIP: &str = "frames";

impl Default for VoxSettings {
    fn default() -> Self {
        VoxSettings {
            scale: 1.0,
            origin: VoxOrigin::Center,
            fps: DEFAULT_FPS,
            clips: Vec::new(),
        }
    }
}

impl VoxSettings {
    /// Reads the `model { … }` block of a `.vox`'s import settings.
    pub fn parse(block: Option<&og_core::ogt::Block>) -> Result<VoxSettings, String> {
        let mut v = VoxSettings::default();
        let Some(b) = block else {
            return Ok(v);
        };
        for e in &b.entries {
            match e.key.as_str() {
                "voxScale" => {
                    v.scale = e
                        .values
                        .first()
                        .and_then(|x| x.as_number())
                        .map(|n| n.as_f32())
                        .filter(|s| s.is_finite() && *s > 0.0)
                        .ok_or_else(|| {
                            format!(
                                "line {}: voxScale takes the metres per voxel, a number above 0 (default 1)",
                                e.line
                            )
                        })?;
                }
                "voxOrigin" => {
                    v.origin = match e.values.first().and_then(|x| x.as_word()) {
                        Some("center") => VoxOrigin::Center,
                        Some("bottom") => VoxOrigin::Bottom,
                        _ => {
                            return Err(format!(
                                "line {}: voxOrigin takes center (the editor's origin, the default) or bottom (the lowest voxel on y = 0)",
                                e.line
                            ));
                        }
                    };
                }
                "voxFps" => {
                    v.fps = e
                        .values
                        .first()
                        .and_then(|x| x.as_number())
                        .map(|n| n.as_f32())
                        .filter(|s| s.is_finite() && (1.0..=120.0).contains(s))
                        .ok_or_else(|| {
                            format!(
                                "line {}: voxFps takes an animated .vox's frames per second, 1 to 120 (default {DEFAULT_FPS})",
                                e.line
                            )
                        })?;
                }
                "voxClip" => {
                    let name = e.values.first().and_then(|x| {
                        x.as_word()
                            .map(str::to_string)
                            .or_else(|| x.as_str().map(str::to_string))
                    });
                    let frame = |i: usize| {
                        e.values
                            .get(i)
                            .and_then(|x| x.as_number())
                            .map(|n| n.as_f64())
                            .filter(|f| f.fract() == 0.0 && *f >= 0.0 && *f <= u32::MAX as f64)
                            .map(|f| f as u32)
                    };
                    match (name, frame(1), frame(2)) {
                        (Some(name), Some(first), Some(last))
                            if first <= last && e.values.len() == 3 =>
                        {
                            if v.clips.iter().any(|c| c.name == name) {
                                return Err(format!("line {}: a second voxClip '{name}'", e.line));
                            }
                            v.clips.push(VoxClip { name, first, last });
                        }
                        _ => {
                            return Err(format!(
                                "line {}: voxClip takes a name and an animated .vox's first and last frame (`voxClip walk 0 3`, both shown)",
                                e.line
                            ));
                        }
                    }
                }
                // Every model's level-of-detail settings (MeshOptions::for_model).
                "lods" | "lodError" => {}
                k => {
                    return Err(format!(
                        "line {}: unknown model setting '{k}' for a .vox (voxScale, voxOrigin, voxFps, voxClip, lods, lodError)",
                        e.line
                    ));
                }
            }
        }
        Ok(v)
    }
}
