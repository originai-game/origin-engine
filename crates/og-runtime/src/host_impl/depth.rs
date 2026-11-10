//! The depth wave's modules' bindings (DebugApi, AssertApi).
//! Expanded for a host by [`crate::host_bindings!`].

#[doc(hidden)]
#[macro_export]
macro_rules! __og_host_bindings_depth {
    ($host:ty) => {
        impl $crate::host_impl::api::DebugApi for $host {
            fn og_debug_line(&self, world: u32, from: String, to: String, desc: Option<String>) {
                $crate::og_debug_line(world, from, to, desc)
            }
            fn og_debug_box(&self, world: u32, center: String, size: String, desc: Option<String>) {
                $crate::og_debug_box(world, center, size, desc)
            }
            fn og_debug_sphere(
                &self,
                world: u32,
                center: String,
                radius: f64,
                desc: Option<String>,
            ) {
                $crate::og_debug_sphere(world, center, radius, desc)
            }
            fn og_debug_arrow(&self, world: u32, from: String, to: String, desc: Option<String>) {
                $crate::og_debug_arrow(world, from, to, desc)
            }
            fn og_debug_axes(&self, world: u32, position: String, size: f64, desc: Option<String>) {
                $crate::og_debug_axes(world, position, size, desc)
            }
            fn og_debug_text(
                &self,
                world: u32,
                position: String,
                text: String,
                desc: Option<String>,
            ) {
                $crate::og_debug_text(world, position, text, desc)
            }
            fn og_debug_clear(&self, world: u32) {
                $crate::og_debug_clear(world)
            }
            fn og_debug_set_view(&self, world: u32, view: String) {
                $crate::og_debug_set_view(world, view)
            }
            fn og_debug_view(&self, world: u32) -> String {
                $crate::og_debug_view(world)
            }
            fn og_debug_set_enabled(&self, world: u32, enabled: bool) {
                $crate::og_debug_set_enabled(world, enabled)
            }
            fn og_debug_enabled(&self, world: u32) -> bool {
                $crate::og_debug_enabled(world)
            }
        }

        impl $crate::host_impl::api::AssertApi for $host {
            fn og_assert_record(&self, world: u32, record: String) -> u32 {
                $crate::og_assert_record(world, record)
            }
        }
    };
}
