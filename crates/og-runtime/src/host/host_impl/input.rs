//! Input's bindings (InputApi).
//! Expanded for a host by [`crate::host_bindings!`].

#[doc(hidden)]
#[macro_export]
macro_rules! __og_host_bindings_input {
    ($host:ty) => {
        impl $crate::host_impl::api::InputApi for $host {
            fn og_input_intent(&self, world: u32, event: String, payload: String) -> String {
                $crate::host_impl::answer($crate::og_input_intent(world, event, payload))
            }
            fn og_input_take_intents(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_input_take_intents(world))
            }
            fn og_input_replay_intent(&self, world: u32, intent: String) -> String {
                $crate::host_impl::answer($crate::og_input_replay_intent(world, intent))
            }
            fn og_input_define_action(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_input_define_action(world, desc))
            }
            fn og_input_actions(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_input_actions(world))
            }
            fn og_input_action(&self, world: u32, name: String) -> String {
                $crate::host_impl::json($crate::og_input_action(world, name))
            }
            fn og_input_is_action_pressed(&self, world: u32, name: String) -> bool {
                $crate::og_input_is_action_pressed(world, name)
            }
            fn og_input_is_action_just_pressed(&self, world: u32, name: String) -> bool {
                $crate::og_input_is_action_just_pressed(world, name)
            }
            fn og_input_is_action_just_released(&self, world: u32, name: String) -> bool {
                $crate::og_input_is_action_just_released(world, name)
            }
            fn og_input_action_held(&self, world: u32, name: String) -> bool {
                $crate::og_input_action_held(world, name)
            }
            fn og_input_action_value(&self, world: u32, name: String) -> f64 {
                $crate::og_input_action_value(world, name)
            }
            fn og_input_action_vector(&self, world: u32, name: String) -> String {
                $crate::host_impl::json($crate::og_input_action_vector(world, name))
            }
            fn og_input_move_relative(&self, world: u32, name: String, camera: String) -> String {
                $crate::host_impl::json($crate::og_input_move_relative(world, name, camera))
            }
            fn og_input_set_action_maps(&self, world: u32, contexts: String) {
                $crate::og_input_set_action_maps(world, contexts)
            }
            fn og_input_action_maps(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_input_action_maps(world))
            }
            fn og_input_rebind(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_input_rebind(world, desc))
            }
            fn og_input_conflicts(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_input_conflicts(world, desc))
            }
            fn og_input_reset_bindings(&self, world: u32, action: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_input_reset_bindings(world, action))
            }
            fn og_input_set_toggle(&self, world: u32, action: String, on: bool) -> String {
                $crate::host_impl::answer($crate::og_input_set_toggle(world, action, on))
            }
            fn og_input_save_bindings(&self, world: u32) -> String {
                $crate::og_input_save_bindings(world)
            }
            fn og_input_load_bindings(&self, world: u32, text: String) -> String {
                $crate::host_impl::answer($crate::og_input_load_bindings(world, text))
            }
            fn og_input_start_capture(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_input_start_capture(world, desc))
            }
            fn og_input_capture(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_input_capture(world))
            }
            fn og_input_cancel_capture(&self, world: u32) {
                $crate::og_input_cancel_capture(world)
            }
            fn og_input_apply_capture(&self, world: u32, policy: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_input_apply_capture(world, policy))
            }
            fn og_input_glyph(&self, world: u32, action: String, device: Option<String>) -> String {
                $crate::host_impl::json($crate::og_input_glyph(world, action, device))
            }
            fn og_input_glyphs(
                &self,
                world: u32,
                action: String,
                device: Option<String>,
            ) -> String {
                $crate::host_impl::json($crate::og_input_glyphs(world, action, device))
            }
            fn og_input_control_glyph(&self, control: String, device: Option<String>) -> String {
                $crate::host_impl::json($crate::og_input_control_glyph(control, device))
            }
            fn og_input_device(&self, world: u32) -> String {
                $crate::og_input_device(world)
            }
            fn og_input_pad_device(&self, world: u32, pad: u32) -> String {
                $crate::host_impl::json($crate::og_input_pad_device(world, pad))
            }
            fn og_input_set_deadzones(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_input_set_deadzones(world, desc))
            }
            fn og_input_set_sensitivity(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_input_set_sensitivity(world, desc))
            }
            fn og_input_sensitivity(
                &self,
                world: u32,
                action: String,
                device: Option<String>,
            ) -> String {
                $crate::host_impl::json($crate::og_input_sensitivity(world, action, device))
            }
            fn og_input_set_keyboard_layout(&self, labels: String) {
                $crate::og_input_set_keyboard_layout(labels)
            }
            fn og_input_keyboard_layout(&self) -> String {
                $crate::host_impl::json($crate::og_input_keyboard_layout())
            }
            fn og_input_set_mouse(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_input_set_mouse(world, desc))
            }
            fn og_input_mouse(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_input_mouse(world))
            }
            fn og_input_rumble(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_input_rumble(world, desc))
            }
            fn og_input_down(&self, world: u32, key: String) -> bool {
                $crate::og_input_down(world, key)
            }
            fn og_input_pressed(&self, world: u32, key: String) -> bool {
                $crate::og_input_pressed(world, key)
            }
            fn og_input_released(&self, world: u32, key: String) -> bool {
                $crate::og_input_released(world, key)
            }
            fn og_input_axis(&self, world: u32, negative: String, positive: String) -> f64 {
                $crate::og_input_axis(world, negative, positive)
            }
            fn og_input_axis2(
                &self,
                world: u32,
                left: String,
                right: String,
                down: String,
                up: String,
            ) -> String {
                $crate::host_impl::json($crate::og_input_axis2(world, left, right, down, up))
            }
            fn og_input_button_down(&self, world: u32, button: String) -> bool {
                $crate::og_input_button_down(world, button)
            }
            fn og_input_button_pressed(&self, world: u32, button: String) -> bool {
                $crate::og_input_button_pressed(world, button)
            }
            fn og_input_button_released(&self, world: u32, button: String) -> bool {
                $crate::og_input_button_released(world, button)
            }
            fn og_pointers_down(&self, world: u32, id: u32) -> bool {
                $crate::og_pointers_down(world, id)
            }
            fn og_pointers_pressed(&self, world: u32, id: u32) -> bool {
                $crate::og_pointers_pressed(world, id)
            }
            fn og_pointers_released(&self, world: u32, id: u32) -> bool {
                $crate::og_pointers_released(world, id)
            }
            fn og_pointers_position(&self, world: u32, id: u32) -> String {
                $crate::host_impl::json($crate::og_pointers_position(world, id))
            }
            fn og_pointers_start(&self, world: u32, id: u32) -> String {
                $crate::host_impl::json($crate::og_pointers_start(world, id))
            }
            fn og_pointers_count(&self, world: u32) -> u32 {
                $crate::og_pointers_count(world)
            }
            fn og_gamepad_connected(&self, world: u32, pad: Option<u32>) -> bool {
                $crate::og_gamepad_connected(world, pad)
            }
            fn og_gamepad_down(&self, world: u32, button: String, pad: Option<u32>) -> bool {
                $crate::og_gamepad_down(world, button, pad)
            }
            fn og_gamepad_pressed(&self, world: u32, button: String, pad: Option<u32>) -> bool {
                $crate::og_gamepad_pressed(world, button, pad)
            }
            fn og_gamepad_released(&self, world: u32, button: String, pad: Option<u32>) -> bool {
                $crate::og_gamepad_released(world, button, pad)
            }
            fn og_gamepad_axis(&self, world: u32, axis: String, pad: Option<u32>) -> f64 {
                $crate::og_gamepad_axis(world, axis, pad)
            }
        }
    };
}
