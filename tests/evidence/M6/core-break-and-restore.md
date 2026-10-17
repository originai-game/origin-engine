# M6 Core: each new test breaks when the behaviour it guards breaks

Each row is one deliberate defect applied to the source, the named test run
alone (`cargo test -p <crate> --test <file> <filter>`), and the source
restored. Every row failed as it must; with the source restored all tests
pass (68 tests in og-core, og-core-derive, og-ecs, og-input).

| # | defect | test | result |
|---|---|---|---|
| 0 | `bool` stored as 4 bytes | og-ecs field_types `a_bool_column_round_trips_through_snapshot_and_hash` | fails |
| 1 | bool byte check accepts 2..9 | same | fails |
| 2 | the hash stream skips one-byte (bool) columns | same | fails |
| 3 | string types allowed as components and resources | og-ecs field_types `strings_ride_on_events_only` | fails |
| 4 | migration fills added fields with zeros, not defaults | og-ecs migrate `a_migrating_restore_maps_fields_by_name` | fails |
| 5 | migration copies enum indices instead of mapping variant names | same | fails |
| 6 | migration maps fields by position instead of name | same | fails |
| 7 | `system_info` ignores a query's `without` | og-ecs introspection `system_info_reports_declarations_and_what_each_query_matched` | fails |
| 8 | matched counts never recorded | same | fails |
| 9 | names left out of the snapshot | og-ecs introspection `names_address_entities_and_survive_snapshots` | fails |
| 10 | despawn keeps the entity's name | same | fails |
| 11 | an overwriting insert bumps the structural counter | og-ecs introspection `the_structural_counter_moves_exactly_on_structural_operations` | fails |
| 12 | `remove` does not bump the counter | same | fails |
| 13 | `SystemContext::set_bytes` needs only read access | og-ecs packed_access `undeclared_byte_access_is_refused_and_named` | fails |
| 14 | key pressed edges never expire | og-input edges `edges_last_one_tick_over_a_scripted_sequence` | fails |
| 15 | `press` never releases | same | fails |
| 16 | a sub-tick tap loses its pressed edge | same | fails |
| 17 | end-of-tick releases raise no `KeyUp`/`PointerUp` | same | fails |
| 18 | release state kept in the sink instead of the world | og-input edges `injected_input_is_world_state_and_replays_exactly` | fails |
| 19 | ring decoder reads `x` from the `y` word | og-input ring `the_decoder_reads_what_the_web_shell_wrote` | fails |
| 20 | ring decoder's slot index off by one | same | fails |

Determinism: `tools/wasm-determinism/run.sh` agrees on every build with the
M1 numbers (`core-cross-target-determinism.md`): the snapshot moved to
format 2 (field table and names), but the world hash covers the state
stream, which for a world without names is byte for byte the M1 snapshot,
so `tests/evidence/M1/cross-target-determinism.md` needed no refresh.
