//! The musical clock: `og.Music` (tempo, metre, where bar 1 starts) and
//! the `og.music` system, which writes each clock's bar, beat and the times
//! of the next beat and bar line, and sends `og.MusicBeat` for every beat in
//! the tick's interval. Times are world seconds (tick / tick rate), so a
//! source with `startAt = nextBar` starts on the bar line to the sample,
//! and stems started on one bar stay in step.

use crate::schema::{Music, MusicBeat};
use og_core::Result;
use og_ecs::{Entity, QueryDesc, SystemDesc, World};

/// The system's name in the schedule (just before og.audio).
pub const MUSIC_SYSTEM: &str = "og.music";

pub(crate) fn register(world: &mut World) -> Result<()> {
    world.register::<Music>()?;
    world.register::<MusicBeat>()?;
    Ok(())
}

/// Where a clock is at world time `t`: the beat index (from 0 at `start`,
/// negative before it) and the beat length in seconds.
fn beat_at(m: &Music, t: f64) -> (f64, f64) {
    let len = 60.0 / f64::from(m.bpm.clamp(20.0, 400.0));
    ((t - m.start) / len, len)
}

/// The world time of the first beat at or after `t`, and of the first bar line.
pub fn next_times(m: &Music, t: f64) -> (f64, f64) {
    let (b, len) = beat_at(m, t);
    let per_bar = f64::from(m.beats_per_bar.max(1));
    // A hair of tolerance so a beat exactly at `t` counts as at `t`.
    let beat = (b - 1e-9).ceil().max(0.0);
    let bar = ((b - 1e-9) / per_bar).ceil().max(0.0) * per_bar;
    (m.start + beat * len, m.start + bar * len)
}

pub(crate) fn install(world: &mut World) -> Result<()> {
    world.add_system(
        SystemDesc::new(MUSIC_SYSTEM)
            .write::<Music>()
            .write::<MusicBeat>()
            .query(QueryDesc::new().write::<Music>()),
        |ctx| {
            let time = ctx.time();
            let rate = f64::from((1.0 / time.dt).round().max(1.0));
            let (t0, t1) = (time.tick as f64 / rate, (time.tick + 1) as f64 / rate);
            let _ = t0;
            let clocks: Vec<Entity> = ctx.query(0)?.flat_map(|a| a.entities().to_vec()).collect();
            for e in clocks {
                let mut m = ctx.get::<Music>(e)?;
                let per_bar = u64::from(m.beats_per_bar.max(1));
                let (b0, len) = beat_at(&m, t0);
                // The beats whose time falls in this tick (each in exactly one tick).
                let tick_of = |t: f64| (t * rate + 1e-7).floor() as u64;
                let mut n = (b0.floor() - 1.0).max(0.0);
                loop {
                    let t = m.start + n * len;
                    let k = tick_of(t);
                    if k > time.tick {
                        break;
                    }
                    if k == time.tick {
                        let i = n as u64;
                        ctx.send(MusicBeat {
                            entity: e.to_bits(),
                            bar: (i / per_bar + 1) as u32,
                            beat: (i % per_bar + 1) as u32,
                            time: t,
                        })?;
                    }
                    n += 1.0;
                }
                // Where it stands as the next tick starts, and what comes next.
                let (b1, _) = beat_at(&m, t1);
                if b1 < 0.0 {
                    m.bar = 0;
                    m.beat = 0;
                } else {
                    let k = (b1 + 1e-9).floor() as u64;
                    m.bar = (k / per_bar + 1) as u32;
                    m.beat = (k % per_bar + 1) as u32;
                }
                let (beat, bar) = next_times(&m, t1);
                m.next_beat = beat;
                m.next_bar = bar;
                ctx.set(e, m)?;
            }
            Ok(())
        },
    )?;
    Ok(())
}
