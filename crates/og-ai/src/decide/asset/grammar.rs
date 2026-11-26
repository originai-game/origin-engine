//! Field discovery and accepted names for the behaviour reader.
use og_core::ogt::{Block, line::Line};
macro_rules! rows {
    ($($scope:expr, $key:literal, $args:literal, $default:literal, $doc:literal);* $(;)?) => {
        &[$(Line::of($scope, $key, $args, $default, $doc, "")),*]
    };
}
pub const LINES: &[Line] = rows![
    &["root"], "name", "\"<name>\"", "behaviour", "Asset name.";
    &["root"], "key", "<name> <number | boolean | x y z>", "", "Initial blackboard value; builtin keys cannot be redefined.";
    &["root"], "route", "<name> { point <x> <y> <z> ... }", "", "Nonempty patrol route, metres.";
    &["root"], "tree", "{ <node> ... }", "", "Exactly one root node; exclusive with machine.";
    &["root"], "machine", "{ initial <state>; state ... }", "", "State machine, exclusive with tree.";
    &["nodes"], "ReactiveFallback", "[<label>] { <nodes> }", "", "First child that does not fail, reevaluated.";
    &["nodes"], "ReactiveSequence", "[<label>] { <nodes> }", "", "First child that does not succeed, reevaluated.";
    &["nodes"], "Sequence", "[<label>] { <nodes> }", "", "Ordered steps, remembers its running child.";
    &["nodes"], "parallel", "[<label>] { policy all | one; <nodes> }", "all", "Nonempty children with explicit completion policy.";
    &["nodes"], "utility", "[<label>] { inertia <number>; option <name> { score ...; <nodes> } }", "0", "Scores the game's options; inertia adds to the running option.";
    &["nodes"], "Inverter", "{ <node> }", "", "One child, invert success/failure.";
    &["nodes"], "ForceSuccess", "{ <node> }", "", "One child; force successful completion.";
    &["nodes"], "repeat", "[<u32>] { <node> }", "0", "One child; zero repeats forever.";
    &["nodes"], "cooldown", "<seconds> { <node> }", "required", "One child with a cooldown.";
    &["nodes"], "timeout", "<seconds> { <node> }", "required", "One child with a deadline.";
    &["nodes", "to"], "check", "<operand> [is | not | below | above | atMost | atLeast <value or operand>]", "truthy", "Reads blackboard or reflected field. Values are number, bool or xyz.";
    &["nodes"], "use", "\"<behaviour asset>\"", "", "Inline a tree; tracked transitive dependencies, cycles fail.";
    &["nodes"], "move_to", "<point key | x y z> { speed ... }", "", "Move toward the explicit point.";
    &["nodes"], "chase", "[<target key>] { speed ...; reach ... }", "target", "Chase the supplied target.";
    &["nodes"], "flee", "[<target key>] { speed ...; distance ... }", "target", "Move away from the supplied target.";
    &["nodes"], "patrol", "<route> { speed ...; wait ...; loop ... }", "", "Follow the declared route.";
    &["nodes"], "wander", "{ speed ...; radius ...; wait ... }", "", "Seeded movement within the stated radius.";
    &["nodes"], "wait", "<seconds>", "required", "Wait for this duration.";
    &["nodes", "instant"], "stop", "", "", "Stop movement.";
    &["nodes", "instant"], "set", "<key> <value>", "", "Set a non-builtin blackboard key.";
    &["nodes", "instant"], "command", "<snake_case command> [<args> ...] { cooldown ... }", "", "Call an explicitly registered game command.";
    &["nodes"], "action", "<name> [<args> ...]", "", "Game-owned action name under 64 bytes, arguments at most 255 bytes.";
    &["move_to", "chase", "flee", "patrol", "wander"], "speed", "<metres/second | operand>", "agent", "Fixed speed or a blackboard/reflected field read at runtime.";
    &["chase"], "reach", "<metres>", "0.5", "Stop distance.";
    &["flee"], "distance", "<metres>", "6", "Requested separation.";
    &["patrol", "wander"], "wait", "<seconds>", "0", "Wait between movement targets.";
    &["patrol"], "loop", "true | false", "true", "Repeat the route.";
    &["wander"], "radius", "<metres>", "4", "Wander radius.";
    &["command"], "cooldown", "<seconds>", "0", "Minimum command interval.";
    &["machine"], "initial", "<state>", "first state", "Initial state name.";
    &["machine"], "state", "<name> { <nodes>; entry ...; exit ...; to ... }", "", "Named state, body is an implicit ordered sequence.";
    &["state"], "entry", "{ <instant actions> }", "", "Only set, stop and command.";
    &["state"], "exit", "{ <instant actions> }", "", "Only set, stop and command.";
    &["state"], "to", "<state> { check ...; after ... }", "", "Transition to an existing state.";
    &["to"], "after", "<seconds>", "0", "Minimum time in state.";
    &["option"], "score", "<number> | <operand> linear | inverse | quadratic | sqrt | smooth | step <lo> <hi>", "required", "At least one score and node per utility option."
];

pub(super) fn check(b: &Block, scope: &str) -> Result<(), String> {
    for e in &b.entries {
        if !LINES
            .iter()
            .any(|f| f.within.contains(&scope) && f.key == e.key)
        {
            return Err(format!(
                "line {}: '{}' is not a {scope} field",
                e.line, e.key
            ));
        }
    }
    Ok(())
}
