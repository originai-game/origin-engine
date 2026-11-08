//! What motion keeps between ticks (in [`crate::Ui::save_state`]): every
//! channel in flight, each state group's current state, the plays and
//! their clips, the leaving elements (ghosts), the rectangles FLIP and
//! shared elements compare against, and the gestures held.

use super::spring::Channel;
use crate::draw::DrawCmd;
use crate::element::Affine;
use crate::layout::Rect;
use crate::style::Srgba;
use og_core::IdMap;
use og_core::glam::Vec2;

/// A state group's current state on an element.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct GroupState {
    pub state: String,
    /// The tick it was entered.
    pub since: u64,
}

/// A timeline on an element at an offset within its play.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Clip {
    /// The timeline's name hash.
    pub timeline: u64,
    /// The element it plays on.
    pub target: u64,
    pub offset_ms: f32,
    /// Its step in a cascade or stagger (pitches rise with it).
    pub index: u32,
}

/// A play in flight.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Play {
    pub id: u32,
    /// The element it was started on (it stops when that leaves).
    pub root: u64,
    pub clips: Vec<Clip>,
    /// The tick it started (or was last seeked or resumed).
    pub start: u64,
    /// Its time at `start`, ms.
    pub at_ms: f32,
    /// Its time last tick, ms (events fire between the two).
    pub last_ms: f32,
    pub speed: f32,
    pub reverse: bool,
    /// 0 once, 1 loop, 2 ping-pong.
    pub repeat: u8,
    pub hold: bool,
    pub paused: bool,
    /// Its length, ms.
    pub length_ms: f32,
    /// It reached its end (held, or removed next tick).
    pub ended: bool,
}

/// An element's drawing, frozen: what a leaving element or a shared
/// element's old self draws while it plays.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Frozen {
    /// Its commands, each with its map relative to the element's own.
    pub cmds: Vec<(DrawCmd, Affine)>,
    /// Its laid-out rectangle and its map onto the screen.
    pub rect: Rect,
    pub world: Affine,
    pub alpha: f32,
    /// Its parent's key (0: a layer or the root).
    pub parent: u64,
    pub path: String,
    /// Its exit (an empty one for a shared element without one).
    pub exit: Option<super::PresenceSpec>,
    pub match_key: u64,
    /// Its corner radius, for a shared element's morph.
    pub radius: f32,
}

/// An element that left the tree and is playing its exit (or morphing into
/// its shared element): drawn from its frozen commands, inert, not in hit
/// tests or the accessibility tree.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Ghost {
    pub key: u64,
    pub frozen: Frozen,
    /// The exit's channels by channel index.
    pub channels: Vec<(u8, Channel)>,
    pub since: u64,
    /// The element it morphs into (a shared element), 0: none.
    pub morph_to: u64,
    /// How long it plays, ticks.
    pub ticks: u32,
}

/// A shared element's last place.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Matched {
    pub key: u64,
    /// On the screen (transforms applied).
    pub rect: Rect,
    pub radius: f32,
    /// The tick it was last seen.
    pub tick: u64,
}

/// A drag (and a pinch) on an element.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Gesture {
    /// The press is held on it; `dragging` once it moved past the threshold.
    pub held: bool,
    pub dragging: bool,
    /// Where the press started, and the offset then.
    pub start: Vec2,
    pub start_offset: Vec2,
    /// 0 free, 1 x, 2 y (an axis lock decided).
    pub lock: u8,
    /// Velocity, CSS px per second (smoothed).
    pub velocity: Vec2,
    /// The pointer last tick.
    pub last: Vec2,
    /// The offset now (springs take it over on release).
    pub offset: Vec2,
    /// The snap point chosen by the last release (-1: none).
    pub snapped: i32,
    pub dismissed: bool,
    /// The drop target under the drag (0: none), and the one it was
    /// dropped on this tick.
    pub over: u64,
    pub dropped_on: u64,
    /// A pinch: its scale and turn, and the spread and angle it started at.
    pub scale: f32,
    pub rotation: f32,
    pub pinch: bool,
    pub pinch_spread: f32,
    pub pinch_angle: f32,
    pub pinch_scale: f32,
    pub pinch_rotation: f32,
    /// The tick it was last used.
    pub tick: u64,
}

/// A burst an event started.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct BurstRun {
    pub key: u64,
    pub since: u64,
    pub count: u32,
    pub radius: f32,
    pub color: Option<Srgba>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct MotionState {
    /// Channels by `channel_key(element, channel)`: looked up for every
    /// channel of every animated element each tick, so a hash map; the
    /// codec writes it in key order.
    pub channels: crate::id::KeyMap<Channel>,
    /// State groups by `group_key(element, group)`.
    pub groups: IdMap<u64, GroupState>,
    /// States a timeline event set, by group key.
    pub set_states: IdMap<u64, String>,
    /// States the probe or an agent forced, by group key.
    pub forced: IdMap<u64, String>,
    pub plays: Vec<Play>,
    pub next_play: u32,
    pub ghosts: Vec<Ghost>,
    /// Last tick's drawing of elements with an exit or a match key.
    pub frozen: IdMap<u64, Frozen>,
    /// Last tick's laid-out rectangles of elements with a layout motion
    /// (relative to the nearest such ancestor).
    pub flip: IdMap<u64, Rect>,
    /// Shared elements by match key hash.
    pub matches: IdMap<u64, Matched>,
    /// Elements with an entrance seen last tick.
    pub seen: IdMap<u64, u64>,
    pub gestures: IdMap<u64, Gesture>,
    pub bursts: Vec<BurstRun>,
    /// Running CSS animations (saved beside the layer stack).
    pub css: Vec<super::css::CssRun>,
}

/// The key of channel `c` of element `key`.
pub(crate) fn channel_key(key: u64, c: usize) -> u64 {
    crate::id::child_key(key, CHANNEL_NAMES[c])
}

/// The key of group `group` of element `key`.
pub(crate) fn group_key(key: u64, group: &str) -> u64 {
    crate::id::child_key(key, &format!("state:{group}"))
}

/// A name's hash (timelines, match keys).
pub(crate) fn name_hash(name: &str) -> u64 {
    crate::id::child_key(0x6d6f_7469_6f6e, name)
}

const CHANNEL_NAMES: [&str; crate::element::CHANNELS + 6] = [
    "~x", "~y", "~sx", "~sy", "~rot", "~kx", "~ky", "~opacity", "~radius", "~bgr", "~bgg", "~bgb",
    "~bga", "~inkr", "~inkg", "~inkb", "~inka", "~z", "~inert", "~flipx", "~flipy", "~flipsx",
    "~flipsy", "~gx", "~gy",
];

/// The FLIP channels after the props' own.
pub(crate) const FLIP_X: usize = crate::element::CHANNELS;
pub(crate) const FLIP_Y: usize = crate::element::CHANNELS + 1;
pub(crate) const FLIP_SX: usize = crate::element::CHANNELS + 2;
pub(crate) const FLIP_SY: usize = crate::element::CHANNELS + 3;
/// A drag's offset, springing after release.
pub(crate) const DRAG_X: usize = crate::element::CHANNELS + 4;
pub(crate) const DRAG_Y: usize = crate::element::CHANNELS + 5;
