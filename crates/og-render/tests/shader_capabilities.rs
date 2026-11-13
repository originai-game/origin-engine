//! What each lit module carries of the shared shading
//! (`og_render::Capability`), as one table read from the modules' own text.
//! A surface that gains or loses a capability changes its row here in the
//! same commit, so a gap between two kinds of surface is a line in a
//! review and not an order of parts somewhere.
#![cfg(not(feature = "profile2d"))]

use og_render::{Capability, Holds};

/// Every module with the direct light (`brdf`), without the variants the
/// list derives from another entry.
fn table() -> String {
    let mut out = format!("{:<40}", "module");
    for c in Capability::ALL {
        out.push_str(&format!(" | {:<4}", c.name()));
    }
    out.push('\n');
    for s in og_render::shader_sources() {
        if !s.wgsl.contains("fn brdf(") || s.label.ends_with(" off)") || s.label.ends_with(" out)")
        {
            continue;
        }
        out.push_str(&format!("{:<40}", s.label));
        for c in Capability::ALL {
            let held = match c.held_by(&s.wgsl) {
                Holds::Full => "full",
                Holds::Stub => "stub",
                Holds::None => "-",
            };
            out.push_str(&format!(" | {held:<width$}", width = c.name().len().max(4)));
        }
        out.truncate(out.trim_end().len());
        out.push('\n');
    }
    out
}

#[test]
fn every_lit_module_carries_what_this_table_says() {
    let expected = "\
module                                   | local lights | weather | decals | oit  | planar reflection | screen ao | lightmap | shared shade
pbr                                      | full         | full    | full   | full | stub              | full      | full     | full
pbr array maps                           | full         | full    | full   | full | stub              | full      | full     | full
pbr, two planar reflections              | full         | full    | full   | full | full              | full      | full     | full
pbr array maps, two planar reflections   | full         | full    | full   | full | full              | full      | full     | full
physical                                 | full         | full    | full   | full | stub              | full      | full     | full
wind                                     | full         | full    | full   | full | stub              | full      | full     | full
meshlet lit                              | full         | full    | full   | full | stub              | full      | full     | full
meshlet lit (planar)                     | full         | full    | full   | full | full              | full      | full     | full
stylise object                           | full         | full    | full   | full | stub              | full      | full     | full
custom full lit=true                     | full         | -       | full   | full | -                 | full      | -        | full
custom full lit=false                    | full         | -       | full   | full | -                 | full      | -        | full
sprite                                   | -            | -       | -      | full | -                 | -         | -        | -
sprite hooks                             | -            | -       | -      | full | -                 | -         | -        | -
sprite hooks (colour only)               | -            | -       | -      | full | -                 | -         | -        | -
sprite array                             | -            | -       | -      | -    | -                 | -         | -        | -
sprite mesh                              | -            | -       | -      | full | -                 | -         | -        | -
sprite skin                              | -            | -       | -      | full | -                 | -         | -        | -
tiles                                    | -            | -       | -      | full | -                 | -         | -        | -
scatter                                  | full         | -       | full   | -    | stub              | full      | -        | full
grass                                    | full         | full    | full   | -    | stub              | full      | -        | full
terrain                                  | full         | full    | full   | -    | stub              | stub      | -        | full
terrain (layer arrays)                   | full         | full    | full   | -    | stub              | stub      | -        | full
water                                    | full         | -       | -      | -    | -                 | stub      | -        | full
fluid 3d                                 | full         | -       | -      | -    | -                 | stub      | -        | full
volumetric                               | full         | -       | -      | -    | -                 | -         | -        | -
volumetric raymarch                      | full         | -       | -      | -    | -                 | -         | -        | -
particles                                | full         | -       | -      | full | -                 | stub      | -        | full
pbr lean                                 | full         | full    | full   | full | stub              | full      | full     | full
pbr lean, two planar reflections         | full         | full    | full   | full | full              | full      | full     | full
";
    let table = table();
    assert!(
        table == expected,
        "the lit modules' capabilities moved; the table is now:\n{table}"
    );
}
