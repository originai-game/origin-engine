//! Audio events (`og.AudioEvent`): the `og.audioEvents` system, before
//! `og.audio`. A triggered event picks one of its sounds (weighted random,
//! round-robin or shuffle), jitters its volume and pitch, honours its
//! cooldown and voice limit, and spawns a voice entity: an
//! `og.AudioSource`, an `og.AudioEventVoice` and an `og.Parent` to the
//! event's entity (so a spatial event is heard where its entity is). Voices
//! that ended are despawned the tick after.
//!
//! Every random draw comes from the engine's seeded stream for audio
//! (`og.AudioMix.rng`, an [`og_core::Rng`] forked from the world's seed at
//! install): world state, so a replay, a restore and a second run with the
//! same seed choose the same sounds at the same pitches, and the game's
//! own `og.Rng` sequence is not perturbed by sounds it plays.

use crate::schema::{
    AudioEvent, AudioEventVoice, AudioMix, AudioSource, BUS_COUNT, MAX_BUSES, Parent, Transform,
    event_order,
};
use og_core::{Result, Rng};
use og_ecs::{Entity, QueryDesc, SystemContext, SystemDesc, World};

/// The system's name in the schedule.
pub const EVENTS_SYSTEM: &str = "og.audioEvents";

/// The label the audio random stream is forked from the world's seed with.
pub const RNG_LABEL: &str = "og.audio";

pub(crate) fn install(world: &mut World) -> Result<()> {
    world.add_system(
        SystemDesc::new(EVENTS_SYSTEM)
            .write::<AudioEvent>()
            .write::<AudioEventVoice>()
            .write::<AudioSource>()
            .write::<Parent>()
            .write::<Transform>()
            .write::<AudioMix>()
            // 0: events, 1: their voices.
            .query(QueryDesc::new().write::<AudioEvent>())
            .query(
                QueryDesc::new()
                    .read::<AudioEventVoice>()
                    .write::<AudioSource>(),
            ),
        run,
    )?;
    Ok(())
}

/// A voice of an event, as the tick finds it.
struct Live {
    e: Entity,
    play: u32,
}

/// The positions of `sounds` that hold a sound.
fn valid(ev: &AudioEvent) -> Vec<usize> {
    let s = ev.sounds.as_slice();
    (0..s.len()).filter(|&i| s[i] != 0).collect()
}

/// The position of the sound this play takes, drawing from `rng` as the
/// order needs (round-robin draws nothing).
fn pick(ev: &mut AudioEvent, rng: &mut Rng) -> Option<usize> {
    let ok = valid(ev);
    if ok.is_empty() {
        return None;
    }
    let i = match ev.order {
        event_order::ROUND_ROBIN => {
            if ev.played == 0 {
                ok[0]
            } else {
                *ok.iter()
                    .find(|&&i| i > ev.index as usize)
                    .unwrap_or(&ok[0])
            }
        }
        event_order::SHUFFLE => {
            let mut left: Vec<usize> = ok
                .iter()
                .copied()
                .filter(|&i| ev.bag & (1 << i) == 0)
                .collect();
            if left.is_empty() {
                // A new round; never the one just played first.
                ev.bag = 0;
                left = ok
                    .iter()
                    .copied()
                    .filter(|&i| ok.len() == 1 || ev.played == 0 || i != ev.index as usize)
                    .collect();
            }
            let k = rng.integer(0, left.len() as i64 - 1) as usize;
            ev.bag |= 1 << left[k];
            left[k]
        }
        _ => {
            let w = ev.weights.as_slice();
            let weight = |i: usize| w.get(i).copied().unwrap_or(1.0).max(0.0);
            let total: f64 = ok.iter().map(|&i| f64::from(weight(i))).sum();
            let r = rng.next_f64();
            if total > 0.0 {
                let mut at = r * total;
                let mut chosen = *ok.last().expect("not empty");
                for &i in &ok {
                    let wi = f64::from(weight(i));
                    if at < wi {
                        chosen = i;
                        break;
                    }
                    at -= wi;
                }
                chosen
            } else {
                ok[(r * ok.len() as f64) as usize % ok.len()]
            }
        }
    };
    Some(i)
}

fn run(ctx: &mut SystemContext<'_>) -> Result<()> {
    let time = ctx.time();
    let rate = f64::from((1.0 / time.dt).round().max(1.0));
    let now = time.tick as f64 / rate;
    let mut mix = *ctx.resource::<AudioMix>()?;
    let mut rng = Rng::default();
    rng.restore(mix.rng);

    // The voices: ended ones (and those whose event is gone) despawned,
    // the others counted per event.
    let voices: Vec<Entity> = ctx.query(1)?.flat_map(|a| a.entities().to_vec()).collect();
    let mut live: Vec<(u64, Live)> = Vec::new();
    for e in voices {
        let v = ctx.get::<AudioEventVoice>(e)?;
        let s = ctx.get::<AudioSource>(e)?;
        let event = Entity::from_bits(v.event);
        if !s.playing || !ctx.is_alive(event) || ctx.get::<AudioEvent>(event).is_err() {
            ctx.despawn(e)?;
            continue;
        }
        live.push((v.event, Live { e, play: v.play }));
    }

    let events: Vec<Entity> = ctx.query(0)?.flat_map(|a| a.entities().to_vec()).collect();
    for e in events {
        let mut ev = ctx.get::<AudioEvent>(e)?;
        let before = ev;
        let mine: Vec<&Live> = live
            .iter()
            .filter(|(k, _)| *k == e.to_bits())
            .map(|(_, l)| l)
            .collect();
        let mut count = mine.len() as u32;
        if ev.trigger {
            ev.trigger = false;
            let cooling = ev.played > 0 && now - ev.last < f64::from(ev.cooldown.max(0.0));
            let full = ev.max_voices > 0 && count >= ev.max_voices;
            let go = !cooling && (!full || ev.steal);
            if go && full {
                // Stop the oldest; og.audio fades it this tick.
                if let Some(oldest) = mine.iter().min_by_key(|l| l.play) {
                    let s = ctx.get::<AudioSource>(oldest.e)?;
                    ctx.set(
                        oldest.e,
                        AudioSource {
                            playing: false,
                            ..s
                        },
                    )?;
                    count -= 1;
                }
            }
            if go && let Some(i) = pick(&mut ev, &mut rng) {
                let mut volume = ev.volume.max(0.0);
                if ev.volume_jitter > 0.0 {
                    let db = rng.range_f32(-ev.volume_jitter, ev.volume_jitter);
                    volume *= og_core::math::exp(db * (std::f32::consts::LN_10 / 20.0));
                }
                let mut pitch = ev.pitch;
                if ev.pitch_jitter > 0.0 {
                    let semis = rng.range_f32(-ev.pitch_jitter, ev.pitch_jitter);
                    pitch *= og_core::math::exp(semis * (std::f32::consts::LN_2 / 12.0));
                }
                let user_bus = if (BUS_COUNT as u32..MAX_BUSES as u32).contains(&ev.user_bus) {
                    ev.user_bus
                } else {
                    0
                };
                ev.played += 1;
                ev.index = i as u32;
                ev.last = now;
                let sound = ev.sounds.as_slice()[i];
                ctx.spawn((
                    AudioSource {
                        sound,
                        bus: ev.bus.min(BUS_COUNT as u32 - 1),
                        volume,
                        pitch,
                        spatial: ev.spatial,
                        ref_distance: ev.ref_distance,
                        max_distance: ev.max_distance,
                        priority: ev.priority,
                        user_bus,
                        ..AudioSource::default()
                    },
                    AudioEventVoice {
                        event: e.to_bits(),
                        sound,
                        play: ev.played,
                    },
                    Parent {
                        entity: e.to_bits(),
                    },
                    Transform::default(),
                ))?;
                count += 1;
            }
        }
        ev.voices = count;
        if ev != before {
            ctx.set(e, ev)?;
        }
    }
    if rng.save() != mix.rng {
        mix.rng = rng.save();
        *ctx.resource_mut::<AudioMix>()? = mix;
    }
    Ok(())
}
