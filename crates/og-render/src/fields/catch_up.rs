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

#[cfg(test)]
mod tests {
    use super::*;

    /// A frame steps each tick since the last once, and a tick already
    /// stepped not again.
    #[test]
    fn a_frame_steps_the_ticks_since_the_last() {
        let step = |last, tick| catch_up(Some(last), tick, 90);
        assert_eq!(
            step(100, 103),
            CatchUp {
                restart: false,
                first: 101,
                count: 3
            }
        );
        assert_eq!(step(100, 100).count, 0);
        assert!(!step(100, 100 + MAX_CATCH_UP).restart);
    }

    /// A first frame, a jump back and a long gap start again, warm up to
    /// the tick drawn and never step a tick before the world's first.
    #[test]
    fn a_restart_warms_up_to_the_tick_drawn() {
        let fresh = catch_up(None, 1000, 90);
        assert_eq!(
            fresh,
            CatchUp {
                restart: true,
                first: 911,
                count: 90
            }
        );
        assert_eq!(catch_up(Some(1001), 1000, 90), fresh);
        assert_eq!(catch_up(Some(1000 - MAX_CATCH_UP - 1), 1000, 90), fresh);
        assert_eq!(
            catch_up(None, 4, 90),
            CatchUp {
                restart: true,
                first: 0,
                count: 5
            }
        );
        assert_eq!(
            catch_up(None, 7, 0),
            CatchUp {
                restart: true,
                first: 7,
                count: 1
            }
        );
    }
}
