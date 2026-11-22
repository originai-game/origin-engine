//! Origin Engine `og-play`: the mechanisms that span domains (`docs/design.md`
//! §2.1), such as a volume on a bone that hits colliders or the candidates
//! in range and in sight of an entity.
//!
//! A domain crate that depends on no other domain: it takes poses, shapes
//! and query answers as data through its own traits, and og-runtime's
//! `bind/play.rs` feeds it. It reports facts (a hit, a candidate) and never
//! decides what they mean. Its state is world state.
//!
//! Empty until its first mechanism lands.
