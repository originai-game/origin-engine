//! Which ticks a frame steps a map through: one step per tick since the
//! last one it stepped, or a restart.

/// The most ticks a frame catches up on; a longer gap starts the map again.
pub const MAX_CATCH_UP: u64 = 8;

/// A frame's steps: `count` ticks from `first`, after starting again when
/// `restart`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CatchUp {
    pub restart: bool,
    pub first: u64,
    pub count: u64,
}

/// The steps of a frame at `tick` for a map last stepped at `last` (`None`
/// before any). A frame at the tick last stepped steps nothing. A first
/// frame, a jump back or a gap over [`MAX_CATCH_UP`] ticks starts again
/// and runs `warm` ticks (at least one) that end at the tick drawn, none
/// of them before tick 0.
pub(crate) fn catch_up(last: Option<u64>, tick: u64, warm: u64) -> CatchUp {
    match last {
        Some(l) if tick >= l && tick - l <= MAX_CATCH_UP => CatchUp {
            restart: false,
            first: l + 1,
            count: tick - l,
        },
        _ => {
            let warm = warm.max(1);
            CatchUp {
                restart: true,
                first: tick.saturating_sub(warm - 1),
                count: warm.min(tick.saturating_add(1)),
            }
        }
    }
}
