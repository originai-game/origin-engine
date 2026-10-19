# Phase 2, UI round: break and restore

Each new test of the round was run once against the code it covers broken
on purpose, seen to fail for the right reason, and run again after the code
was restored.

| Test | Broken how | Failed with | Restored |
|---|---|---|---|
| `packages/engine/tests/ui-sound.test.ts` (row 15) | `crates/og-runtime/src/modules.rs` as on main: `og.ui.sound` scheduled after `og_audio::install` | `expect(received).toBeLessThan(expected)`: the hover blip's first frame 72800, expected < 72800 (its input's tick is frames 72000..72800; the blip started one tick late) | passes, first frames 72000 and 120000 |
| `crates/og-ui/tests/skin.rs` `keyboard_and_gamepad_focus_moves_sound_as_the_pointer_does` (row 41) | the `Hover` sound of a focus move in `crates/og-ui/src/nav.rs` removed | `Tab onto A sounds`: left `[]`, right `[Hover]` | passes |
