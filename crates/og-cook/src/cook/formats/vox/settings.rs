//! Native voxel import controls and inclusive animation clip ranges.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VoxOrigin {
    #[default]
    Center,
    Bottom,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VoxClip {
    pub name: String,
    pub first: u32,
    pub last: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VoxSettings {
    pub scale: f32,
    pub origin: VoxOrigin,
    pub fps: f32,
    pub clips: Vec<VoxClip>,
}

pub const DEFAULT_FPS: f32 = 10.0;
pub const DEFAULT_CLIP: &str = "frames";

impl Default for VoxSettings {
    fn default() -> Self {
        Self {
            scale: 1.0,
            origin: VoxOrigin::Center,
            fps: DEFAULT_FPS,
            clips: Vec::new(),
        }
    }
}

impl VoxSettings {
    pub fn from_input(
        input: Option<&crate::cook::source_schemas::world::import::Model>,
    ) -> Result<Self, String> {
        use crate::cook::source_schemas::world::import;
        let Some(d) = input else {
            return Ok(Self::default());
        };
        let mut clips: Vec<VoxClip> = Vec::new();
        for clip in &d.vox_clips {
            if clip.first > clip.last {
                return Err(format!(
                    "voxClips '{}': first must not exceed last",
                    clip.name
                ));
            }
            if clips.iter().any(|c| c.name == clip.name) {
                return Err(format!("duplicate voxClips name '{}'", clip.name));
            }
            clips.push(VoxClip {
                name: clip.name.clone(),
                first: clip.first,
                last: clip.last,
            });
        }
        Ok(Self {
            scale: d.vox_scale.unwrap_or(1.0),
            origin: match d.vox_origin {
                Some(import::VoxOrigin::Bottom) => VoxOrigin::Bottom,
                _ => VoxOrigin::Center,
            },
            fps: d.vox_fps.unwrap_or(DEFAULT_FPS),
            clips,
        })
    }
}
