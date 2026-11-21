//! The UI's bindings (UiApi, I18nApi).
//! Expanded for a host by [`crate::host_bindings!`].

#[doc(hidden)]
#[macro_export]
macro_rules! __og_host_bindings_ui {
    ($host:ty) => {
        impl $crate::host_impl::api::UiApi for $host {
            fn og_ui_configure(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_ui_configure(world, desc))
            }
            fn og_ui_viewport(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_ui_viewport(world))
            }
            fn og_ui_theme(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_ui_theme(world))
            }
            fn og_ui_frame_theme(&self) -> String {
                $crate::og_ui_frame_theme()
            }
            fn og_ui_frame_viewport(&self) -> String {
                $crate::og_ui_frame_viewport()
            }
            fn og_ui_has_icon(&self, name: String) -> bool {
                $crate::og_ui_has_icon(name)
            }
            fn og_ui_seconds(&self) -> f64 {
                $crate::og_ui_seconds()
            }
            fn og_ui_frame_direction(&self) -> String {
                $crate::og_ui_frame_direction()
            }
            fn og_ui_frame_pointer(&self) -> String {
                $crate::host_impl::json($crate::og_ui_frame_pointer())
            }

            fn og_ui_frame_cursor(&self) -> String {
                $crate::og_ui_frame_cursor()
            }
            fn og_ui_feedback(&self, event: String, rise: f64) {
                $crate::og_ui_feedback(event, rise)
            }
            fn og_ui_set_haptics(&self, intensity: f64) {
                $crate::og_ui_set_haptics(intensity)
            }
            fn og_ui_set_virtual_cursor(&self, desc: String) {
                $crate::og_ui_set_virtual_cursor(desc)
            }
            fn og_ui_virtual_cursor(&self) -> String {
                $crate::host_impl::json($crate::og_ui_virtual_cursor())
            }

            fn og_ui_tick(&self) -> u32 {
                $crate::og_ui_tick()
            }
            fn og_ui_preview_viewport(&self, world: u32, viewport: String) {
                $crate::og_ui_preview_viewport(world, viewport)
            }
        }

        impl $crate::host_impl::api::I18nApi for $host {
            fn og_i18n_load(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_i18n_load(world, path))
            }
            fn og_i18n_set_locale(&self, world: u32, locale: String) -> String {
                $crate::host_impl::json($crate::og_i18n_set_locale(world, locale))
            }
            fn og_i18n_locale(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_i18n_locale(world))
            }
            fn og_i18n_t(&self, world: u32, key: String, args: Option<String>) -> String {
                $crate::og_i18n_t(world, key, args)
            }
            fn og_i18n_has(&self, world: u32, key: String) -> bool {
                $crate::og_i18n_has(world, key)
            }
            fn og_i18n_format_number(
                &self,
                world: u32,
                value: f64,
                format: Option<String>,
            ) -> String {
                $crate::og_i18n_format_number(world, value, format)
            }
            fn og_i18n_format_compact(
                &self,
                world: u32,
                value: f64,
                decimals: Option<u32>,
            ) -> String {
                $crate::og_i18n_format_compact(world, value, decimals)
            }
            fn og_i18n_format_currency(&self, world: u32, amount: f64, currency: String) -> String {
                $crate::og_i18n_format_currency(world, amount, currency)
            }
            fn og_i18n_format_date(
                &self,
                world: u32,
                time_ms: f64,
                format: Option<String>,
            ) -> String {
                $crate::og_i18n_format_date(world, time_ms, format)
            }
            fn og_i18n_format_time(
                &self,
                world: u32,
                time_ms: f64,
                format: Option<String>,
            ) -> String {
                $crate::og_i18n_format_time(world, time_ms, format)
            }
            fn og_i18n_format_duration(
                &self,
                world: u32,
                seconds: f64,
                style: Option<String>,
            ) -> String {
                $crate::og_i18n_format_duration(world, seconds, style)
            }
        }
    };
}
