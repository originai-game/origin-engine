//! What each lit module carries of the shared shading
//! (`og_render::Capability`), as one table read from the modules' own text.
//! A surface that gains or loses a capability changes its row here in the
//! same commit, so a gap between two kinds of surface is a line in a
//! review and not an order of parts somewhere.
//!
//! A cell says what the surface does, not what its module contains: a
//! capability whose code no entry point reaches is `idle`, and no module
//! may hold one (the part is dead weight and the row would claim what the
//! surface does not do). The two a material shader's hooks may call
//! (`Capability::HOOKS_CALL`) are there for the hooks.
//!
//! The empty cells that are a decision, not a debt: the water and the 3D
//! fluid take no decals, puddles or snow (a mark on water is its own foam
//! or wake, and the rain rings it through weather_rain.wgsl, which is not
//! this capability); the water reads the screen's occlusion through its
//! own binding; only meshes are lightmapped. The guides say the same to
//! an author (render.md, looks.md). One is a debt: the terrain's stub for
//! the screen's occlusion (its pipelines have no texture slot left), so
//! what stands on the ground darkens no ground around its foot.
#![cfg(not(feature = "profile2d"))]

use naga::Statement as St;
use og_render::{Capability, Holds};
use std::collections::HashSet;

/// Calls `f` with every statement of a block, nested ones too.
fn each<'a>(block: &'a naga::Block, f: &mut dyn FnMut(&'a St)) {
    for s in block.iter() {
        f(s);
        match s {
            St::Block(b) => each(b, f),
            St::If { accept, reject, .. } => {
                each(accept, f);
                each(reject, f);
            }
            St::Switch { cases, .. } => cases.iter().for_each(|c| each(&c.body, f)),
            St::Loop {
                body, continuing, ..
            } => {
                each(body, f);
                each(continuing, f);
            }
            _ => {}
        }
    }
}

/// The names of the functions `wgsl`'s entry points reach.
fn reached(label: &str, wgsl: &str) -> HashSet<String> {
    let module = naga::front::wgsl::parse_str(wgsl)
        .unwrap_or_else(|e| panic!("{label}: {}", e.emit_to_string(wgsl)));
    let mut seen = HashSet::new();
    let mut open = Vec::new();
    let calls = |body: &naga::Block, open: &mut Vec<naga::Handle<naga::Function>>| {
        each(body, &mut |s| {
            if let St::Call { function, .. } = s {
                open.push(*function);
            }
        });
    };
    for ep in &module.entry_points {
        calls(&ep.function.body, &mut open);
    }
    while let Some(f) = open.pop() {
        if seen.insert(f) {
            calls(&module.functions[f].body, &mut open);
        }
    }
    seen.into_iter()
        .filter_map(|f| module.functions[f].name.clone())
        .collect()
}

/// Every module with the direct light (`brdf`), without the variants the
/// list derives from another entry and without the unlit full shader (the
/// lit one's module, whose lit path it never takes: its colour is its own).
fn table() -> String {
    let mut out = format!("{:<40}", "module");
    for c in Capability::ALL {
        out.push_str(&format!(" | {:<4}", c.name()));
    }
    out.push('\n');
    for s in og_render::shader_sources() {
        if !s.wgsl.contains("fn brdf(")
            || s.label.ends_with(" off)")
            || s.label.ends_with(" out)")
            || s.label.ends_with(" lit=false")
        {
            continue;
        }
        out.push_str(&format!("{:<40}", s.label));
        let reached = reached(&s.label, &s.wgsl);
        for c in Capability::ALL {
            let held = match c.held_by(&s.wgsl) {
                Holds::Full
                    if !Capability::HOOKS_CALL.contains(&c) && !reached.contains(c.function()) =>
                {
                    "idle"
                }
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
custom full lit=true                     | full         | full    | full   | full | -                 | full      | -        | full
sprite                                   | -            | -       | -      | full | -                 | -         | -        | -
sprite hooks                             | -            | -       | -      | full | -                 | -         | -        | -
sprite hooks (colour only)               | -            | -       | -      | full | -                 | -         | -        | -
sprite array                             | -            | -       | -      | -    | -                 | -         | -        | -
sprite mesh                              | -            | -       | -      | full | -                 | -         | -        | -
sprite skin                              | -            | -       | -      | full | -                 | -         | -        | -
tiles                                    | -            | -       | -      | full | -                 | -         | -        | -
scatter                                  | full         | full    | full   | -    | stub              | full      | -        | full
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
volumetric (water)                       | full         | -       | -      | -    | -                 | -         | -        | -
volumetric raymarch (water)              | full         | -       | -      | -    | -                 | -         | -        | -
";
    let table = table();
    assert!(
        !table.contains("idle"),
        "a module holds a capability's code that none of its entry points reaches:\n{table}"
    );
    assert!(
        table == expected,
        "the lit modules' capabilities moved; the table is now:\n{table}"
    );
}
