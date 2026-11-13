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
            fn og_ui_define_mass(&self, name: String, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_ui_define_mass(name, desc))
            }
            fn og_ui_spawn_mass(&self, template: u32, instances: String) {
                $crate::og_ui_spawn_mass(template, instances)
            }
            fn og_ui_mass_count(&self, template: u32) -> u32 {
                $crate::og_ui_mass_count(template)
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
            fn og_ui_set_focus(&self, path: String) {
                $crate::og_ui_set_focus(path)
            }
            fn og_ui_scroll_into_view(&self, path: String, center: bool) {
                $crate::og_ui_scroll_into_view(path, center)
            }
            fn og_ui_bounds(&self, path: String) -> String {
                $crate::host_impl::json($crate::og_ui_bounds(path))
            }
            fn og_ui_scroll_state(&self, path: String) -> String {
                $crate::host_impl::json($crate::og_ui_scroll_state(path))
            }
            fn og_ui_toast(&self, world: u32, text: String, desc: Option<String>) -> u32 {
                $crate::og_ui_toast(world, text, desc)
            }
            fn og_ui_dismiss_toast(&self, world: u32, toast_id: u32) {
                $crate::og_ui_dismiss_toast(world, toast_id)
            }
            fn og_ui_navigate(&self, world: u32, intent: String) {
                $crate::og_ui_navigate(world, intent)
            }
            fn og_ui_back(&self) -> bool {
                $crate::og_ui_back()
            }
            fn og_ui_set_navigation(&self, on: bool) {
                $crate::og_ui_set_navigation(on)
            }
            fn og_ui_feedback(&self, event: String, rise: f64) {
                $crate::og_ui_feedback(event, rise)
            }
            fn og_ui_set_haptics(&self, intensity: f64) {
                $crate::og_ui_set_haptics(intensity)
            }
            fn og_ui_set_players(&self, count: u32) {
                $crate::og_ui_set_players(count)
            }
            fn og_ui_set_virtual_cursor(&self, desc: String) {
                $crate::og_ui_set_virtual_cursor(desc)
            }
            fn og_ui_virtual_cursor(&self) -> String {
                $crate::host_impl::json($crate::og_ui_virtual_cursor())
            }
            fn og_ui_set_hud_edit(&self, on: bool) {
                $crate::og_ui_set_hud_edit(on)
            }
            fn og_ui_set_hud_player(&self, player: u32) {
                $crate::og_ui_set_hud_player(player)
            }
            fn og_ui_hud_edit(&self) -> String {
                $crate::host_impl::json($crate::og_ui_hud_edit())
            }
            fn og_ui_hud_layout(&self, player: u32) -> String {
                $crate::og_ui_hud_layout(player)
            }
            fn og_ui_set_hud_layout(&self, player: u32, layout: String) -> bool {
                $crate::og_ui_set_hud_layout(player, layout)
            }
            fn og_ui_hud_select(&self, key: String) {
                $crate::og_ui_hud_select(key)
            }
            fn og_ui_hud_step(&self, scale: i32, opacity: i32) {
                $crate::og_ui_hud_step(scale, opacity)
            }
            fn og_ui_hud_reset(&self, key: String) {
                $crate::og_ui_hud_reset(key)
            }
            fn og_ui_focus(&self, pad: u32) -> String {
                $crate::og_ui_focus(pad)
            }

            fn og_ui_a11y(&self) -> String {
                $crate::host_impl::json($crate::og_ui_a11y())
            }
            fn og_ui_tick(&self) -> u32 {
                $crate::og_ui_tick()
            }
            fn og_ui_preview_viewport(&self, world: u32, viewport: String) {
                $crate::og_ui_preview_viewport(world, viewport)
            }
            fn og_ui_define_timeline(&self, name: String, timeline: Option<String>) {
                $crate::og_ui_define_timeline(name, timeline)
            }
            fn og_ui_play(&self, path: String, choreo: String, desc: Option<String>) -> u32 {
                $crate::og_ui_play(path, choreo, desc)
            }
            fn og_ui_play_timeline(
                &self,
                path: String,
                timeline: String,
                desc: Option<String>,
            ) -> u32 {
                $crate::og_ui_play_timeline(path, timeline, desc)
            }
            fn og_ui_seek(&self, play: u32, ms: f64) -> bool {
                $crate::og_ui_seek(play, ms)
            }
            fn og_ui_pause(&self, play: u32) -> bool {
                $crate::og_ui_pause(play)
            }
            fn og_ui_resume(&self, play: u32) -> bool {
                $crate::og_ui_resume(play)
            }
            fn og_ui_reverse(&self, play: u32) -> bool {
                $crate::og_ui_reverse(play)
            }
            fn og_ui_stop(&self, play: u32) -> bool {
                $crate::og_ui_stop(play)
            }
            fn og_ui_play_info(&self, play: u32) -> String {
                $crate::host_impl::json($crate::og_ui_play_info(play))
            }
            fn og_ui_force_state(&self, path: String, group: String, state: String) {
                $crate::og_ui_force_state(path, group, state)
            }

            fn og_ui_visual_state(&self, path: String, group: String) -> String {
                $crate::og_ui_visual_state(path, group)
            }
            fn og_ui_animate(&self, id: String, target: f64, transition: String) -> f64 {
                $crate::og_ui_animate(id, target, transition)
            }
            fn og_ui_gesture(&self, path: String) -> String {
                $crate::host_impl::json($crate::og_ui_gesture(path))
            }
            fn og_ui_motion_events(&self) -> String {
                $crate::host_impl::json($crate::og_ui_motion_events())
            }
            fn og_ui_overlaps(&self) -> String {
                $crate::host_impl::json($crate::og_ui_overlaps())
            }
            fn og_ui_layout_stats(&self) -> String {
                $crate::host_impl::json($crate::og_ui_layout_stats())
            }
            fn og_ui_push(&self, world: u32, screen: String, vm: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_ui_push(world, screen, vm))
            }
            fn og_ui_pop(&self, world: u32) -> String {
                $crate::og_ui_pop(world)
            }
            fn og_ui_replace(&self, world: u32, screen: String, vm: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_ui_replace(world, screen, vm))
            }
            fn og_ui_clear(&self, world: u32) {
                $crate::og_ui_clear(world)
            }
            fn og_ui_layers(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_ui_layers(world))
            }
            fn og_ui_screen_vm(&self, world: u32, screen: String) -> String {
                $crate::host_impl::json($crate::og_ui_screen_vm(world, screen))
            }
            fn og_ui_doc_events(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_ui_doc_events(world))
            }
            fn og_ui_game_input(&self, world: u32) -> bool {
                $crate::og_ui_game_input(world)
            }
            fn og_ui_paused(&self, world: u32) -> bool {
                $crate::og_ui_paused(world)
            }
            fn og_ui_define_action(&self, world: u32, desc: String) -> String {
                $crate::host_impl::answer($crate::og_ui_define_action(world, desc))
            }
            fn og_ui_action(&self, name: String) -> bool {
                $crate::og_ui_action(name)
            }
            fn og_ui_action_held(&self, name: String) -> bool {
                $crate::og_ui_action_held(name)
            }
            fn og_ui_glyph(&self, world: u32, action: String) -> String {
                $crate::host_impl::json($crate::og_ui_glyph(world, action))
            }
            fn og_ui_frame_glyph(&self, action: String) -> String {
                $crate::host_impl::json($crate::og_ui_frame_glyph(action))
            }
            fn og_ui_device(&self, world: u32) -> String {
                $crate::og_ui_device(world)
            }
            fn og_ui_file_picker(&self, world: u32) -> bool {
                $crate::og_ui_file_picker(world)
            }
            fn og_ui_set_device(&self, world: u32, device: String) {
                $crate::og_ui_set_device(world, device)
            }
            fn og_ui_action_bar(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_ui_action_bar(world))
            }
            fn og_ui_document_schema(&self) -> String {
                $crate::host_impl::json($crate::og_ui_document_schema())
            }
            fn og_ui_set_prop(
                &self,
                world: u32,
                path: String,
                prop: String,
                value: String,
            ) -> String {
                $crate::host_impl::answer($crate::og_ui_set_prop(world, path, prop, value))
            }
            fn og_ui_save_edits(&self, world: u32) -> String {
                $crate::host_impl::answer($crate::og_ui_save_edits(world))
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
                seconds: f64,
                format: Option<String>,
            ) -> String {
                $crate::og_i18n_format_date(world, seconds, format)
            }
            fn og_i18n_format_time(
                &self,
                world: u32,
                seconds: f64,
                format: Option<String>,
            ) -> String {
                $crate::og_i18n_format_time(world, seconds, format)
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
