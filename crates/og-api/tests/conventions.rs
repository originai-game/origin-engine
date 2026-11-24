//! The names of the API keep their promises (the `conventions` guide): a
//! field's name says its unit and its type, so a reader who has learned
//! one member writes the next one right. Checked over every field of
//! every struct and record of the IDL.
//!
//! A member that breaks a rule is either renamed or listed below with its
//! reason; a listed member that keeps the rule again fails too, so the
//! lists never outlive their debts.

use og_idl::model::{Api, Field, Item, TypeRef};
use std::collections::BTreeSet;
use std::path::Path;

/// Part files whose names are a contract with another repository
/// (AGENTS.md §Ownership): the platform reads and writes these members by
/// name, so they are renamed there first.
const CONTRACTS: [&str; 2] = ["probe.ogt", "platform.ogt"];

/// Members that break a rule for a reason that stands, as (`Owner.field`,
/// the rule, why).
const EXCEPTIONS: [(&str, &str, &str); 8] = [
    // A tick stamp, not a duration: the tick something happened at.
    (
        "LockstepSeat.since",
        "ticks",
        "the tick a seat's state began, not a number of ticks",
    ),
    (
        "LockstepSeat.until",
        "ticks",
        "the tick a seat's state ends, not a number of ticks",
    ),
    (
        "LockstepStatus.acknowledged",
        "ticks",
        "the last tick the peers acknowledged, not a number of ticks",
    ),
    (
        "WorldRestored.from",
        "ticks",
        "the tick the world was restored from, not a number of ticks",
    ),
    // The art contract's angles are degrees (conventions §2): the yaw an
    // isometric or dimetric view is drawn at, as an artist names it.
    (
        "ArtCamera.yaw",
        "rad",
        "an angle of the art contract, which is in degrees",
    ),
    // `damping` that is not a decay rate: a joint's and a reverb's.
    (
        "Joint.damping",
        "damping",
        "a joint spring's damping coefficient, N·s/m",
    ),
    (
        "ReverbZone.damping",
        "damping",
        "a reverb's damping of high frequencies, not of motion",
    ),
    // `alpha` as a flag, the `type video` key it reports.
    (
        "VideoInfo.alpha",
        "opacity",
        "whether a video's frames carry the source's alpha (`alpha true`), not how opaque it is",
    ),
];

/// Debts: members that break a rule and are to be renamed, in modules
/// this test's stream did not own, as (`Owner.field`, the rule, who).
///
/// None today: the `ui-*.ogt` modules keep every rule. The visual-effects
/// stream owns `vfx.ogt` and og.ParticleSystem, og.Trail, og.Beam,
/// og.Decal, SpriteBlend and StripBlend; none of their fields breaks a rule
/// today, and one that comes to is listed here, not renamed from outside
/// that stream.
const DEBTS: [(&str, &str, &str); 0] = [];

/// Every field of the IDL as (part file, owner, field).
fn fields(api: &Api) -> Vec<(&str, &str, &Field)> {
    let mut out = Vec::new();
    for item in api.modules.iter().flat_map(|m| m.items.iter()) {
        let (owner, fields) = match item {
            Item::Struct(s) => (s.name.as_str(), &s.fields),
            Item::Record(r) => (r.name.as_str(), &r.fields),
            _ => continue,
        };
        for f in fields {
            out.push((api.locate(f.line).0, owner, f));
        }
    }
    out
}

/// Whether a type is `name` or holds it (`Option.Quaternion`).
fn holds(ty: &TypeRef, name: &str) -> bool {
    ty.name() == name || ty.args().iter().any(|a| holds(a, name))
}

/// The rules a field of this name, type and unit breaks, each as (rule,
/// what the rule asks).
fn broken(name: &str, ty: &TypeRef, unit: Option<&str>) -> Vec<(&'static str, &'static str)> {
    let mut out = Vec::new();
    if unit == Some("ms") && !(name == "ms" || name.ends_with("Ms")) {
        out.push((
            "ms",
            "a field in milliseconds is named `ms` or ends in `Ms`",
        ));
    }
    if unit == Some("ticks") && !(name.contains("Tick") || name.contains("tick")) {
        out.push(("ticks", "a field in ticks has `Tick` or `tick` in its name"));
    }
    if holds(ty, "Quaternion") && !(name == "quaternion" || name.ends_with("Quaternion")) {
        out.push((
            "quaternion",
            "a Quaternion is named `quaternion` or ends in `Quaternion`",
        ));
    }
    if matches!(name, "rotation" | "heading" | "yaw") && unit.is_some_and(|u| u != "rad") {
        out.push(("rad", "a rotation, heading or yaw with a unit is in `rad`"));
    }
    if (name == "fov" || name.starts_with("slope")) && unit != Some("deg") {
        out.push(("deg", "a fov and a slope carry `unit deg`"));
    }
    if name == "damping" && unit != Some("perSecond") {
        out.push((
            "damping",
            "`damping` is a decay rate and carries `unit perSecond`",
        ));
    }
    if name == "alpha" || name.ends_with("Alpha") {
        out.push((
            "opacity",
            "how opaque a thing is is named `opacity` or ends in `Opacity`",
        ));
    }
    if ty.name() == "bool" && matches!(name, "pressed" | "released") {
        out.push((
            "press",
            "a press's edges are `justPressed` and `justReleased`; `down` is its state",
        ));
    }
    out
}

#[test]
fn every_field_name_keeps_its_convention() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let api = og_idl::read(&root.join(og_idl::IDL_PATH)).unwrap();
    let listed: BTreeSet<(&str, &str)> = EXCEPTIONS
        .iter()
        .chain(&DEBTS)
        .map(|(member, rule, _)| (*member, *rule))
        .collect();
    assert_eq!(
        listed.len(),
        EXCEPTIONS.len() + DEBTS.len(),
        "a member is listed twice for one rule"
    );

    let mut failures = Vec::new();
    let mut met = BTreeSet::new();
    let mut checked = 0;
    for (file, owner, f) in fields(&api) {
        checked += 1;
        if CONTRACTS.contains(&file) {
            continue;
        }
        let member = format!("{owner}.{}", f.name);
        for (rule, asks) in broken(&f.name, &f.ty, f.unit.as_deref()) {
            match listed.iter().find(|(m, r)| *m == member && *r == rule) {
                Some(key) => {
                    met.insert(*key);
                }
                None => failures.push(format!(
                    "{file}: {member} ({}{}): {asks}; rename it, or list it in \
                     crates/og-api/tests/conventions.rs with its reason",
                    f.ty.name(),
                    f.unit
                        .as_deref()
                        .map(|u| format!(", unit {u}"))
                        .unwrap_or_default(),
                )),
            }
        }
    }
    // A listed member that no longer breaks its rule (renamed, or gone):
    // its line goes too.
    for key in listed.difference(&met) {
        failures.push(format!(
            "{} is listed for the `{}` rule and does not break it: remove its line from \
             crates/og-api/tests/conventions.rs",
            key.0, key.1
        ));
    }
    assert!(checked > 1000, "the IDL's fields were not read: {checked}");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
