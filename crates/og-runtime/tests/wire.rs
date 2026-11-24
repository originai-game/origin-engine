//! The typed wire readers (src/wire.rs over the generated types): what the
//! IDL refuses, the engine refuses.

#[test]
fn a_system_desc_the_idl_refuses_refuses_the_engine() {
    use og_runtime::{og_engine_create, og_game_game, og_host_begin_bundle};
    og_host_begin_bundle();
    // Read by hand, a `when` the IDL does not name ran as `during`.
    og_runtime::og_system_define_system("late".into(), r#"{"when":"after"}"#.into(), 0);
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let answer = og_engine_create(game, r#"{}"#.into());
    assert!(
        answer.contains("og.system 'late': desc.when: 'after' is not a StateWhen"),
        "{answer}"
    );
}
