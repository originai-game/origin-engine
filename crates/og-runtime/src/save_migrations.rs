//! Engine-owned save migrations: what an engine upgrade does to a player's
//! save so it restores on the new build instead of being refused or reset.
//!
//! A save is the whole world, so engine state sits in it beside the game's:
//! engine components and resources (og.Transform, og.Gamepad, ...) with
//! their schema versions, and the modules' state blocks (og.ui, physics,
//! ...) with their own byte versions. When a release moves one of them, it
//! registers here, by what moved and the version it moved from, the step
//! that turns the old saved form into the new one. [`upgrade`] runs every
//! step a save needs, in table order (so a block two versions behind takes
//! both of its steps), on every restore and every save a game opens for
//! its own migrations, before the restore checks the save against the
//! world: the check then sees the save as this build writes it.
//!
//! The table is the record of every engine state change since saves
//! existed; `og-api`'s layout test pins each engine
//! record's version, so a record's fields cannot change without a bump,
//! and a bump without a step here leaves old saves to the field-by-name
//! mapping (`migrate`). A record that only gained fields needs no step: a
//! restore takes it with the added fields at their declared defaults
//! (`save::engine_owned`); a field removed, renamed or retyped does.

use og_ecs::SnapshotEdit;

/// What a migration upgrades.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Target {
    /// A state block, versioned by the `u32` its bytes start with.
    Block,
    /// A component, resource or event, versioned by its schema version.
    Record,
}

/// A step's edit of the save.
type Step = fn(&mut SnapshotEdit) -> Result<(), String>;

/// An upgraded snapshot and the steps it took, for the log.
type Upgraded = (Vec<u8>, Vec<String>);

/// One engine step: `name` at version `from` becomes version `to`.
pub(crate) struct Migration {
    pub target: Target,
    pub name: &'static str,
    pub from: u32,
    pub to: u32,
    /// The engine release whose saves it upgrades, and what changed.
    pub note: &'static str,
    /// The step; `None` when the restore already takes the old form (a
    /// session record the host's value replaces).
    pub run: Option<Step>,
}

/// Every engine step, oldest first.
pub(crate) const MIGRATIONS: &[Migration] = &[
    Migration {
        target: Target::Block,
        name: "og.ui",
        from: 9,
        to: 11,
        note: "0.7.7 and earlier: hit transforms (U1), motion channels (U2), feedback sets, the layer stack (U3)",
        run: Some(ui_9_to_11),
    },
    Migration {
        target: Target::Block,
        name: "og.ui",
        from: 11,
        to: 12,
        note: "U6b: the focus returns as a stack, one per nested scope",
        run: Some(ui_upgrade),
    },
    Migration {
        target: Target::Record,
        name: "og.Gamepad",
        from: 1,
        to: 2,
        note: "0.7.7 and earlier: four pads, now eight (G1-2); a session record, so the host's value replaces the saved one",
        run: None,
    },
    Migration {
        target: Target::Record,
        name: "og.AudioSource",
        from: 1,
        to: 2,
        note: "0.7.7 and earlier: gained userBus (G2-5, the bus graph); the restore takes it at 0, the source's own bus",
        run: None,
    },
    Migration {
        target: Target::Record,
        name: "og.AudioBusFx",
        from: 1,
        to: 2,
        note: "0.7.7 and earlier: gained userBus, the equaliser, the echo and the waveshaper (G2-5); the restore takes them at their defaults, all off",
        run: None,
    },
    Migration {
        target: Target::Record,
        name: "og.RigidBody",
        from: 1,
        to: 2,
        note: "0.7.7 and earlier: gained canSleep and sleeping (G2-3, sleep); the restore takes them at true and false, so a body can sleep and is awake",
        run: None,
    },
    Migration {
        target: Target::Record,
        name: "og.Physics",
        from: 1,
        to: 2,
        note: "0.7.7 and earlier: gained sleepTicks and sleepSpeed (G2-3, sleep); the restore takes them at 0 and 0.1, sleep off as before",
        run: None,
    },
    Migration {
        target: Target::Record,
        name: "og.Joint",
        from: 1,
        to: 2,
        note: "0.7.7 and earlier: gained breakForce, breakTorque and broken (G2-3, breaking joints); the restore takes them at 0, 0 and false, a joint that never breaks",
        run: None,
    },
    Migration {
        target: Target::Record,
        name: "og.Vehicle",
        from: 1,
        to: 2,
        note: "0.7.7 and earlier: gained model, handbrake, handbrakeGrip, driftGrip and the drivetrain's fields (G2-3); the restore takes their defaults: model VEHICLE_ARCADE, the car it was",
        run: None,
    },
];

fn ui_9_to_11(edit: &mut SnapshotEdit) -> Result<(), String> {
    let bytes = edit.block("og.ui").ok_or("no og.ui block")?;
    let upgraded = og_ui::Ui::upgrade_state(bytes).map_err(|e| e.reason().to_string())?;
    edit.set_block("og.ui", upgraded)
        .map_err(|e| e.reason().to_string())
}

fn ui_upgrade(edit: &mut SnapshotEdit) -> Result<(), String> {
    let bytes = edit.block("og.ui").ok_or("no og.ui block")?;
    let upgraded = og_ui::Ui::upgrade_state(bytes).map_err(|e| e.reason().to_string())?;
    edit.set_block("og.ui", upgraded)
        .map_err(|e| e.reason().to_string())
}

/// A state block's version: the `u32` its bytes start with.
fn block_version(edit: &SnapshotEdit, name: &str) -> Option<u32> {
    let b = edit.block(name)?;
    Some(u32::from_le_bytes(b.get(..4)?.try_into().ok()?))
}

/// Upgrades a world snapshot (or a selection) taken by an older engine:
/// `Ok(None)` when nothing in it needed a step (the bytes restore as they
/// are), `Ok(Some(bytes))` the upgraded snapshot and the steps it took.
/// A step that fails refuses the save, naming the step. A snapshot this
/// module cannot read is left to the restore, which says why.
pub(crate) fn upgrade(snapshot: &[u8]) -> Result<Option<Upgraded>, String> {
    let Ok(mut edit) = SnapshotEdit::from_bytes(snapshot) else {
        return Ok(None);
    };
    let mut done = Vec::new();
    for m in MIGRATIONS {
        let at = match m.target {
            Target::Block => block_version(&edit, m.name),
            Target::Record => edit.schema(m.name).map(|s| s.version),
        };
        if at != Some(m.from) {
            continue;
        }
        let Some(run) = m.run else { continue };
        run(&mut edit).map_err(|e| {
            format!(
                "the engine's migration of {} from version {} to {} failed: {e}",
                m.name, m.from, m.to
            )
        })?;
        done.push(format!("{} {} -> {} ({})", m.name, m.from, m.to, m.note));
    }
    Ok((!done.is_empty()).then(|| (edit.to_bytes(), done)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_step_moves_a_version_forward_and_each_name_chains() {
        for m in MIGRATIONS {
            assert!(m.to > m.from, "{} {} -> {}", m.name, m.from, m.to);
        }
        for (i, m) in MIGRATIONS.iter().enumerate() {
            // A later step of the same name starts where an earlier ended.
            for later in &MIGRATIONS[i + 1..] {
                if later.name == m.name && later.target == m.target {
                    assert!(later.from >= m.to, "{} steps overlap", m.name);
                }
            }
        }
    }
}
