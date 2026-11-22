//! Origin Engine `og-play`: the mechanisms that span domains (`docs/design.md`
//! §2.1), such as a volume on a bone that hits colliders or the candidates
//! in range and in sight of an entity.
//!
//! A domain crate that depends on no other domain: it takes poses, shapes
//! and query answers as data through its own traits, and og-runtime's
//! `bind/play.rs` feeds it. It reports facts (a hit, a candidate) and never
//! decides what they mean. Its state is world state. Plain f32 arithmetic,
//! sqrt and og-core's deterministic transcendentals, in a fixed order, so
//! every host computes the same bits.
//!
//! | module | what it owns |
//! |---|---|
//! | [`shape`] | [`shape::Shape`], a volume's shape; [`shape::Pose`]; [`shape::Motion`], the screw between two poses |
//! | [`contact`] | [`contact::touch`]: whether two placed solids overlap, and where |
//! | [`hits`] | [`hits::Swings`]: hit volumes swept between ticks against hurt volumes and colliders, each target once a swing |
//! | [`candidates`] | [`candidates::candidates`]: what is in range and in a cone, nearest first |

pub mod candidates;
pub mod contact;
pub mod hits;
pub mod shape;
