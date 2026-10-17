# og-probe conformance and command-channel test run

Revision fa2045d (og-probe uncommitted in the tree), native aarch64-apple-darwin, Rust 1.97.0.

```text
$ cargo test -p og-probe
     Running unittests src/lib.rs (target/agent-probe/debug/deps/og_probe-0c01f76a15184694)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/commands.rs (target/agent-probe/debug/deps/commands-87baa0cf41bfe212)
test the_grammar_is_strict_about_times_and_names ... ok
test help_lists_every_action_core_and_registered ... ok
test an_unknown_action_warns_and_the_script_continues ... ok
test pointer_and_key_actions_go_through_the_input_layer ... ok
test teleport_set_frame_and_dump_entity_write_through_the_schema ... ok
test steps_run_on_simulation_time_even_inside_an_advance ... ok
test a_scripted_run_seeds_advances_dumps_and_quits ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/conformance.rs (target/agent-probe/debug/deps/conformance-6c1788ced9b04b40)
test a_failing_system_is_a_fault_and_the_probe_still_answers ... ok
test reload_swaps_the_bundle_and_keeps_the_world ... ok
test a_sweep_yields_one_frame_per_value_and_restores_the_world ... ok
test every_member_answers_as_protocol_5_says ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/publish.rs (target/agent-probe/debug/deps/publish-bb38e3889108fc96)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/websocket.rs (target/agent-probe/debug/deps/websocket-9375e8edb89ab56f)
test other_paths_and_non_loopback_addresses_are_refused ... ok
test a_real_client_is_answered_on_the_world_thread ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ cargo test -p og-probe --features publish --test publish
test only_player_facing_members_answer ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ cargo fmt -p og-probe --check && cargo clippy -p og-probe --all-targets [--all-features|--no-default-features] -- -D warnings
clean (all three feature sets)
$ cargo check -p og-probe --target wasm32-unknown-unknown [--no-default-features]
clean (both)
```

## Each covered path broken once, then restored

`a scratch mutation script` applied one change at a time to `crates/og-probe/src`, ran the named suite, and restored the file:

```text
caught by conformance                    unknown method answers null
caught by conformance                    patch ignores the tunable flag
caught by conformance                    patch skips the range check
caught by conformance                    run accepts > 256 operations
caught by conformance                    run does not stop at a refusal
caught by conformance                    sweep does not restore afterwards
caught by conformance                    sweep does not restore between values
caught by commands                       advance does not run due steps
caught by commands                       an unknown action stops the script
caught by commands                       help omits registered actions
caught by commands                       parse accepts descending times
caught by commands                       input edges never expire
caught by commands                       seed does nothing
caught by conformance                    dump_world counts wrong
caught by conformance                    reload does not re-register dropped types
caught by conformance                    restore ignores unknown ids
caught by websocket                      server accepts any path
caught by publish                        publish keeps every member
caught by conformance                    capture accepts a lone width
caught by commands                       touch targets ignore the path
caught by websocket                      server answers off the world thread (replaces a too-weak 'serve one call per frame' mutation, which is correct behaviour)
```
