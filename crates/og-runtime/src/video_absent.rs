//! The IDL's `video` module in a build without og-runtime's `video`
//! feature: og-video is not linked, and this module stands in for
//! `video.rs`. `og.load.video` and `og.load.videoInfo` answer [`ABSENT`],
//! `og.video.capabilities` answers that nothing streams, and a document's
//! `<video>` or `<audio>` (which asks its player through og.MediaIntent,
//! the module's event) is refused with it when the document loads.
//!
//! The world is the full build's while the game never names footage: the
//! og.video system is scheduled at the same place with the same
//! declarations and runs nothing (a world with change filters saves every
//! system's name and the change tick, which moves once per system run).
//! og.VideoPlayer and og.MediaIntent stay lazy records in IDL order
//! (super::lazy) and add no state block in either build; naming either is
//! refused with [`ABSENT`].

use super::codec::Layout;
use super::extract::RenderState;
use og_ecs::{SystemContext, SystemDesc, World};
use std::collections::BTreeMap;

/// Why footage answers nothing here.
pub(crate) const ABSENT: &str = "og.video is not built into this engine: og-runtime was built without its `video` feature (a default feature; og-host-web and og-host-node pass it through as `video`); og.load.video, og.VideoPlayer and a document's <video> or <audio> need it";

/// The system's name (the full module's).
pub(crate) const VIDEO_SYSTEM: &str = "og.video";
pub(crate) const PLAYER: &str = "og.VideoPlayer";
/// og.VideoPlayer's schema version (the full module's: the engine's type
/// table is the same in both builds).
pub(crate) const PLAYER_VERSION: u32 = 2;
const AUDIO: &str = "og.AudioSource";
pub(crate) const MEDIA_INTENT: &str = "og.MediaIntent";

/// Lazy og.VideoPlayer or og.MediaIntent was named: refused.
pub(crate) fn refuse_lazy(
    _world: &mut World,
    _mount: super::engine::Mount<'_>,
) -> og_core::Result<()> {
    Err(og_core::Error::new(og_core::ErrorKind::Invalid, ABSENT))
}

/// No player plays here.
pub(crate) fn media_state(_engine: u32, _player: u64) -> Option<(og_ui::bind::MediaState, f64)> {
    None
}

/// Never reached with requests: a document with a `<video>` or `<audio>`
/// is refused when it loads.
pub(crate) fn send_media(
    _eng: &crate::engine::Engine,
    _ctx: &SystemContext<'_>,
    _ops: &[og_ui::bind::MediaOp],
) -> Result<(), String> {
    Err(ABSENT.to_string())
}

/// og.video on one engine: nothing here.
#[derive(Default)]
pub(crate) struct ModuleState;

/// The og.video system as the full module schedules it (its name and
/// declarations), running nothing.
pub(crate) fn install(
    world: &mut World,
    _mount: super::engine::Mount<'_>,
    _layouts: &BTreeMap<String, Layout>,
) -> Result<(), String> {
    world
        .add_system(
            SystemDesc {
                name: VIDEO_SYSTEM.into(),
                writes: vec![AUDIO.into()],
                ..Default::default()
            },
            |_| Ok(()),
        )
        .map_err(|e| e.reason().to_string())
}

pub(crate) fn extract(_world: &World, _rs: &mut RenderState) {}

/// `og.load.video`.
pub fn og_load_video(_world: u32, _path: String) -> Result<crate::wire::VideoInfo, String> {
    Err(ABSENT.into())
}

/// `og.load.videoInfo`.
pub fn og_load_video_info(_world: u32, _video: f64) -> Result<crate::wire::VideoInfo, String> {
    Err(ABSENT.into())
}

/// `og.video.capabilities`: nothing streams (as the full module answers
/// without a decoder).
pub fn og_video_capabilities(_world: u32) -> crate::wire::VideoCapabilities {
    crate::wire::VideoCapabilities {
        streamed: false,
        decoder: "none".into(),
        hardware: false,
        max_width: 0,
        max_height: 0,
    }
}
