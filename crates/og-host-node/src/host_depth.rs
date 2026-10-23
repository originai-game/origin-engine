//! The depth wave's modules' trait implementations (DebugApi): every
//! member is the host-neutral runtime's.

use crate::NodeHost;
use og_api::host::*;
use og_runtime as rt;

impl DebugApi for NodeHost {
    fn og_debug_line(&self, world: u32, from: String, to: String, desc: Option<String>) {
        rt::og_debug_line(world, from, to, desc)
    }
    fn og_debug_box(&self, world: u32, center: String, size: String, desc: Option<String>) {
        rt::og_debug_box(world, center, size, desc)
    }
    fn og_debug_sphere(&self, world: u32, center: String, radius: f64, desc: Option<String>) {
        rt::og_debug_sphere(world, center, radius, desc)
    }
    fn og_debug_arrow(&self, world: u32, from: String, to: String, desc: Option<String>) {
        rt::og_debug_arrow(world, from, to, desc)
    }
    fn og_debug_axes(&self, world: u32, position: String, size: f64, desc: Option<String>) {
        rt::og_debug_axes(world, position, size, desc)
    }
    fn og_debug_text(&self, world: u32, position: String, text: String, desc: Option<String>) {
        rt::og_debug_text(world, position, text, desc)
    }
    fn og_debug_clear(&self, world: u32) {
        rt::og_debug_clear(world)
    }
    fn og_debug_set_view(&self, world: u32, view: String) {
        rt::og_debug_set_view(world, view)
    }
    fn og_debug_view(&self, world: u32) -> String {
        rt::og_debug_view(world)
    }
    fn og_debug_set_enabled(&self, world: u32, enabled: bool) {
        rt::og_debug_set_enabled(world, enabled)
    }
    fn og_debug_enabled(&self, world: u32) -> bool {
        rt::og_debug_enabled(world)
    }
}
