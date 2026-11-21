// Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate.
//! The C ABI of every binding symbol: an `extern "C"` entry each,
//! taking its arguments as `OgValue`s and writing its answer to one,
//! that calls the host's implementation of its `og_api::host` trait;
//! `SYMBOLS` lists them for the shell that registers them.

#![allow(clippy::too_many_arguments, clippy::let_unit_value, unused_unsafe, unused_variables)]

use crate::CHost as Host;
use og_api::host::*;

unsafe extern "C" fn og_host_set_dispatcher(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_host_set_dispatcher(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_host_shared(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_host_shared(&Host);
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_create(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_create(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_destroy(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_destroy(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_advance(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_advance(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_command(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_command(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_frame(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_frame(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_host_begin_bundle(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_host_begin_bundle(&Host);
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_call(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_call(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_take_outputs(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_take_outputs(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_take_log(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_take_log(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_write_resource(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_write_resource(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_resource_layout(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_resource_layout(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_read_resource(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_read_resource(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_write_resource_bytes(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_write_resource_bytes(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_component_id(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_component_id(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_component_layout(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_component_layout(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_read_component(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_read_component(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_write_component(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_write_component(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_query_archetypes(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_query_archetypes(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_view_column(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_view_column(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_view_add_scaled(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_view_add_scaled(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) }, unsafe { crate::arg(args, 4) }, unsafe { crate::arg(args, 5) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_ui_patch(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_ui_patch(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_inspector_panel(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_inspector_panel(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_inspector_event(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_inspector_event(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_take(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_platform_take(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_answer(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_platform_answer(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_set_host(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_platform_set_host(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_notify(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_platform_notify(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_room_take(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_room_take(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_room_deliver(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_room_deliver(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_room_set_host(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_room_set_host(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_shared_take(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_shared_take(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_shared_deliver(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_shared_deliver(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_set_content(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_set_content(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_trace(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_trace(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_trace_take(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_trace_take(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_perf(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_perf(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_perf_report(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_perf_report(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_perf_frames(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Lifecycle>::og_engine_perf_frames(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_present_frame(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Present>::og_present_frame(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_present_resume(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Present>::og_present_resume(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_present_refresh(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Present>::og_present_refresh(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_present_fullscreen(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as Present>::og_present_fullscreen(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_reflect_define_component(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ReflectApi>::og_reflect_define_component(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_reflect_define_resource(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ReflectApi>::og_reflect_define_resource(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_reflect_define_event(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ReflectApi>::og_reflect_define_event(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_save_edit_get_version(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_save_edit_get_version(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_save_edit_types(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_save_edit_types(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_save_edit_records(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_save_edit_records(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_save_edit_set_records(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_save_edit_set_records(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_save_edit_rename_type(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_save_edit_rename_type(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_save_edit_rename_field(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_save_edit_rename_field(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_save_edit_drop_type(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_save_edit_drop_type(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_save_edit_spawn(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_save_edit_spawn(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_save_edit_text(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_save_edit_text(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_save_edit_close(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_save_edit_close(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_spatial_index_get_count(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_spatial_index_get_count(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_spatial_index_within(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_spatial_index_within(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_spatial_index_nearest(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_spatial_index_nearest(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_world_get_tick(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_world_get_tick(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_world_spawn(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_world_spawn(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_world_despawn(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_world_despawn(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_world_alive(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_world_alive(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_world_has(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_world_has(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_world_insert(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_world_insert(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_world_remove(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_world_remove(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_world_get(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_world_get(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_world_set(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_world_set(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_world_resource(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_world_resource(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_world_children(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_world_children(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_world_removed(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_world_removed(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_world_spatial(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_world_spatial(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_world_state(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_world_state(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_world_set_state(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_world_set_state(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_world_find(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_world_find(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_world_prefab_of(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_world_prefab_of(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_world_name_of(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_world_name_of(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_world_query(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_world_query(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_world_events(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_world_events(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_world_emit(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_world_emit(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_world_instantiate(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_world_instantiate(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_world_snapshot(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_world_snapshot(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_world_open_save(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_world_open_save(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_world_restore(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_world_restore(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_query_count(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_query_count(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_archetype_view_get_count(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_archetype_view_get_count(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_archetype_view_get_tick(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_archetype_view_get_tick(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_archetype_view_get_entities(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_archetype_view_get_entities(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_archetype_view_get_rows(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_archetype_view_get_rows(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_archetype_view_entity(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as WorldApi>::og_archetype_view_entity(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_hook_get_name(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as SystemApi>::og_hook_get_name(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_system_define_hook(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as SystemApi>::og_system_define_hook(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_system_get_name(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as SystemApi>::og_system_get_name(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_system_get_reads(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as SystemApi>::og_system_get_reads(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_system_get_writes(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as SystemApi>::og_system_get_writes(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_system_define_system(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as SystemApi>::og_system_define_system(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_game_get_tick(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as GameApi>::og_game_get_tick(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_game_get_systems(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as GameApi>::og_game_get_systems(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_game_game(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as GameApi>::og_game_game(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_transform_compose(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TransformApi>::og_transform_compose(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_transform_decompose(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TransformApi>::og_transform_decompose(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_transform_look_at(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TransformApi>::og_transform_look_at(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_transform_face(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TransformApi>::og_transform_face(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_transform_forward(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TransformApi>::og_transform_forward(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_transform_right(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TransformApi>::og_transform_right(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_transform_up(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TransformApi>::og_transform_up(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_transform_camera_forward(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TransformApi>::og_transform_camera_forward(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_transform_camera_right(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TransformApi>::og_transform_camera_right(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_transform_heading(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TransformApi>::og_transform_heading(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_transform_from_heading(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TransformApi>::og_transform_from_heading(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_transform_face2d(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TransformApi>::og_transform_face2d(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_transform_forward2d(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TransformApi>::og_transform_forward2d(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_transform_heading2d(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TransformApi>::og_transform_heading2d(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_transform_from_heading2d(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TransformApi>::og_transform_from_heading2d(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_transform_from_axis_angle(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TransformApi>::og_transform_from_axis_angle(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_time_set_scale(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TimeApi>::og_time_set_scale(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_time_hit_stop(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TimeApi>::og_time_hit_stop(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_time_set_group_scale(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TimeApi>::og_time_set_group_scale(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_time_group_dt(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TimeApi>::og_time_group_dt(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_rng_stream_get_seed(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RngApi>::og_rng_stream_get_seed(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_rng_stream_next(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RngApi>::og_rng_stream_next(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_rng_stream_range(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RngApi>::og_rng_stream_range(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_rng_stream_integer(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RngApi>::og_rng_stream_integer(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_rng_stream_chance(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RngApi>::og_rng_stream_chance(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_rng_stream_weighted(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RngApi>::og_rng_stream_weighted(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_rng_stream_permutation(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RngApi>::og_rng_stream_permutation(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_rng_stream_fork(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RngApi>::og_rng_stream_fork(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_rng_next(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RngApi>::og_rng_next(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_rng_range(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RngApi>::og_rng_range(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_rng_integer(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RngApi>::og_rng_integer(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_rng_chance(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RngApi>::og_rng_chance(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_rng_weighted(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RngApi>::og_rng_weighted(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_rng_permutation(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RngApi>::og_rng_permutation(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_rng_fork(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RngApi>::og_rng_fork(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_procgen_noise(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProcgenApi>::og_procgen_noise(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_procgen_noise_grid(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProcgenApi>::og_procgen_noise_grid(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_procgen_poisson_disk(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProcgenApi>::og_procgen_poisson_disk(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_procgen_bsp(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProcgenApi>::og_procgen_bsp(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_procgen_wfc(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProcgenApi>::og_procgen_wfc(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_intent(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_intent(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_take_intents(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_take_intents(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_replay_intent(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_replay_intent(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_define_action(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_define_action(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_actions(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_actions(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_action(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_action(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_is_action_down(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_is_action_down(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_is_action_just_pressed(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_is_action_just_pressed(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_is_action_just_released(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_is_action_just_released(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_is_action_held(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_is_action_held(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_action_value(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_action_value(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_action_vector(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_action_vector(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_move_relative(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_move_relative(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_set_action_maps(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_set_action_maps(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_action_maps(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_action_maps(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_rebind(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_rebind(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_conflicts(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_conflicts(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_reset_bindings(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_reset_bindings(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_set_toggle(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_set_toggle(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_save_bindings(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_save_bindings(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_load_bindings(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_load_bindings(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_start_capture(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_start_capture(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_capture(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_capture(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_cancel_capture(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_cancel_capture(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_apply_capture(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_apply_capture(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_glyph(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_glyph(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_glyphs(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_glyphs(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_control_glyph(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_control_glyph(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_device(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_device(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_pad_device(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_pad_device(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_set_deadzones(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_set_deadzones(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_set_sensitivity(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_set_sensitivity(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_sensitivity(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_sensitivity(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_set_keyboard_layout(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_set_keyboard_layout(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_keyboard_layout(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_keyboard_layout(&Host);
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_set_mouse(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_set_mouse(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_mouse(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_mouse(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_rumble(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_rumble(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_down(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_down(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_just_pressed(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_just_pressed(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_just_released(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_just_released(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_axis(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_axis(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_axis2(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_axis2(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) }, unsafe { crate::arg(args, 4) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_button_down(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_button_down(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_button_just_pressed(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_button_just_pressed(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_input_button_just_released(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_input_button_just_released(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_pointers_down(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_pointers_down(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_pointers_just_pressed(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_pointers_just_pressed(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_pointers_just_released(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_pointers_just_released(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_pointers_position(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_pointers_position(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_pointers_start(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_pointers_start(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_pointers_count(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_pointers_count(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_gamepad_connected(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_gamepad_connected(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_gamepad_down(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_gamepad_down(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_gamepad_just_pressed(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_gamepad_just_pressed(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_gamepad_just_released(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_gamepad_just_released(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_gamepad_axis(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as InputApi>::og_gamepad_axis(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_camera_look_at(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as CameraApi>::og_camera_look_at(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_camera_world_to_screen(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as CameraApi>::og_camera_world_to_screen(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_camera_screen_to_ray(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as CameraApi>::og_camera_screen_to_ray(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_camera_shake(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as CameraApi>::og_camera_shake(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_camera_punch(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as CameraApi>::og_camera_punch(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_camera_kick(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as CameraApi>::og_camera_kick(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_snap(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_snap(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_snap_camera(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_snap_camera(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_compile(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_compile(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_reveal(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_reveal(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_ribbon(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_ribbon(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_sprite(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_sprite(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_floating_text(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_floating_text(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_measure_text(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_measure_text(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_box_geometry(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_box_geometry(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_plane_geometry(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_plane_geometry(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_sphere_geometry(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_sphere_geometry(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_standard_material(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_standard_material(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_update_material(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_update_material(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_shader_material(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_shader_material(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_set_material_shader(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_set_material_shader(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_set_shader_param(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_set_shader_param(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_apply_look(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_apply_look(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_apply_quality(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_apply_quality(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_set_display(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_set_display(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_reset_display(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_reset_display(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_display(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_display(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_look(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_look(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_capabilities(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_capabilities(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_text(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_text(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_buffer_geometry(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_buffer_geometry(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_update_geometry(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_update_geometry(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_geometry_info(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_geometry_info(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_cylinder_geometry(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_cylinder_geometry(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_cone_geometry(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_cone_geometry(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_capsule_geometry(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_capsule_geometry(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_torus_geometry(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_torus_geometry(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_ring_geometry(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_ring_geometry(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_circle_geometry(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_circle_geometry(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_tube_geometry(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_tube_geometry(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_shape_geometry(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_shape_geometry(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_extrude_geometry(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_extrude_geometry(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_lathe_geometry(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_lathe_geometry(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_polyhedron_geometry(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_polyhedron_geometry(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_icosahedron_geometry(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_icosahedron_geometry(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_dodecahedron_geometry(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_dodecahedron_geometry(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_octahedron_geometry(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_octahedron_geometry(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_tetrahedron_geometry(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_tetrahedron_geometry(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_sweep_geometry(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_sweep_geometry(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_csg_geometry(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_csg_geometry(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_merge_geometries(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_merge_geometries(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_lines(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_lines(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_points(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_points(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_raycast(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_raycast(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_light2d_at(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_light2d_at(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_render_target(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_render_target(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_set_post_param(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_set_post_param(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_post_errors(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_post_errors(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_render_baked_lighting(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as RenderApi>::og_render_baked_lighting(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_bundle(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_bundle(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_bundle_info(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_bundle_info(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_progress(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_progress(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_default_bundle(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_default_bundle(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_gltf(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_gltf(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_scene(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_scene(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_prefab(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_prefab(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_font(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_font(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_font_info(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_font_info(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_sound(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_sound(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_lut(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_lut(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_environment(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_environment(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_environment_sun(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_environment_sun(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_emitter(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_emitter(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_effect(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_effect(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_shader(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_shader(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_post(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_post(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_vector(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_vector(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_ui_material(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_ui_material(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_texture(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_texture(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_texture_array(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_texture_array(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_style(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_style(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_look(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_look(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_look_info(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_look_info(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_palette(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_palette(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_sound_info(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_sound_info(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_status(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_status(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_error(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_error(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_gltf_info(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_gltf_info(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_animation(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_animation(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_rig(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_rig(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_flipbook(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_flipbook(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_tilemap(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_tilemap(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_video(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_video(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_video_info(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_video_info(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_timeline(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_timeline(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_vfx(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_vfx(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_rig2d(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_rig2d(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_image_data(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_image_data(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_gltf_data(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_gltf_data(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_environment_data(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_environment_data(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_data(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_data(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_data_json(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_data_json(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_data_bytes(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_data_bytes(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_pack(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_pack(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_environment_info(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_environment_info(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_release(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_release(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_image_info(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_image_info(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_completions(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_completions(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_load_replay_completions(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadApi>::og_load_replay_completions(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_physics_water(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PhysicsApi>::og_physics_water(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_physics_water_height(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PhysicsApi>::og_physics_water_height(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_physics_cast_ray(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PhysicsApi>::og_physics_cast_ray(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_physics_cast_ray_all(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PhysicsApi>::og_physics_cast_ray_all(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_physics_project_point(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PhysicsApi>::og_physics_project_point(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_physics_cast_shape(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PhysicsApi>::og_physics_cast_shape(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_physics_overlap(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PhysicsApi>::og_physics_overlap(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_physics_apply_force(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PhysicsApi>::og_physics_apply_force(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_physics_apply_impulse(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PhysicsApi>::og_physics_apply_impulse(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_physics_apply_torque(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PhysicsApi>::og_physics_apply_torque(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_physics_apply_torque_impulse(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PhysicsApi>::og_physics_apply_torque_impulse(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_physics_set_mesh(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PhysicsApi>::og_physics_set_mesh(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_physics_rope_points(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PhysicsApi>::og_physics_rope_points(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_physics_garment_points(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PhysicsApi>::og_physics_garment_points(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_physics_cloth_points(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PhysicsApi>::og_physics_cloth_points(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_physics_soft_body_points(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PhysicsApi>::og_physics_soft_body_points(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_physics_fluid_particles(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PhysicsApi>::og_physics_fluid_particles(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_physics_fluid3d_particles(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PhysicsApi>::og_physics_fluid3d_particles(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_physics_vehicle_wheels(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PhysicsApi>::og_physics_vehicle_wheels(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_physics_ragdoll_rig_from_skeleton(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PhysicsApi>::og_physics_ragdoll_rig_from_skeleton(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_physics_set_ragdoll_rig(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PhysicsApi>::og_physics_set_ragdoll_rig(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_physics_ragdoll_pose(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PhysicsApi>::og_physics_ragdoll_pose(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_physics_stats(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PhysicsApi>::og_physics_stats(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_atmosphere_time_of_day(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as AtmosphereApi>::og_atmosphere_time_of_day(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_atmosphere_season(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as AtmosphereApi>::og_atmosphere_season(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_anim_get_node_matrix(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as AnimApi>::og_anim_get_node_matrix(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_anim_graph(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as AnimApi>::og_anim_graph(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_anim_humanoid(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as AnimApi>::og_anim_humanoid(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_anim_retarget(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as AnimApi>::og_anim_retarget(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_anim_mirror(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as AnimApi>::og_anim_mirror(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_anim_rig2d_info(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as AnimApi>::og_anim_rig2d_info(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_anim_skeleton2d_boxes(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as AnimApi>::og_anim_skeleton2d_boxes(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_timeline_get_tween(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_timeline_get_tween(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_timeline_get_error(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_timeline_get_error(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_timeline_to(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_timeline_to(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_timeline_from(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_timeline_from(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_timeline_from_to(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_timeline_from_to(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_timeline_set(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_timeline_set(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_timeline_call(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_timeline_call(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_timeline_add_label(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_timeline_add_label(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_timeline_add(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_timeline_add(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_timeline_play(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_timeline_play(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_timeline_pause(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_timeline_pause(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_timeline_resume(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_timeline_resume(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_timeline_reverse(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_timeline_reverse(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_timeline_restart(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_timeline_restart(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_timeline_seek(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_timeline_seek(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_timeline_progress(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_timeline_progress(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_timeline_time(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_timeline_time(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_timeline_duration(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_timeline_duration(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_timeline_time_scale(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_timeline_time_scale(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_timeline_is_active(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_timeline_is_active(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_timeline_kill(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_timeline_kill(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_to(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_to(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_from(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_from(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_from_to(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_from_to(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_sequence(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_sequence(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_parallel(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_parallel(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_timeline(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_timeline(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_cancel(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_cancel(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_cancel_entity(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_cancel_entity(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_active(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_active(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_squash(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_squash(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tween_flash(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TweenApi>::og_tween_flash(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tilemap_set_tile(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TilemapApi>::og_tilemap_set_tile(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tilemap_fill(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TilemapApi>::og_tilemap_fill(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tilemap_create(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TilemapApi>::og_tilemap_create(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tilemap_resize(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TilemapApi>::og_tilemap_resize(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tilemap_neighbours(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TilemapApi>::og_tilemap_neighbours(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tilemap_get_tile(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TilemapApi>::og_tilemap_get_tile(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) }, unsafe { crate::arg(args, 4) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tilemap_cell_at(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TilemapApi>::og_tilemap_cell_at(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_tilemap_cell_center(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TilemapApi>::og_tilemap_cell_center(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_audio_play_feedback(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as AudioApi>::og_audio_play_feedback(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_audio_create_sound(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as AudioApi>::og_audio_create_sound(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_audio_output_latency(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as AudioApi>::og_audio_output_latency(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_nav_grid(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as NavApi>::og_nav_grid(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_nav_grid_from_tilemap(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as NavApi>::og_nav_grid_from_tilemap(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_nav_set_cost(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as NavApi>::og_nav_set_cost(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) }, unsafe { crate::arg(args, 4) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_nav_cell_at(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as NavApi>::og_nav_cell_at(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_nav_find_path(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as NavApi>::og_nav_find_path(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_nav_flow_field(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as NavApi>::og_nav_flow_field(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_nav_set_flow_targets(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as NavApi>::og_nav_set_flow_targets(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_nav_flow_at(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as NavApi>::og_nav_flow_at(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_nav_load(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as NavApi>::og_nav_load(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_nav_build(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as NavApi>::og_nav_build(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_nav_set_obstacle(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as NavApi>::og_nav_set_obstacle(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_nav_carve(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as NavApi>::og_nav_carve(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_nav_set_area(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as NavApi>::og_nav_set_area(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_nav_set_link(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as NavApi>::og_nav_set_link(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_nav_set_link_open(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as NavApi>::og_nav_set_link_open(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_nav_finish_link(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as NavApi>::og_nav_finish_link(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_nav_nearest(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as NavApi>::og_nav_nearest(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_nav_path_of(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as NavApi>::og_nav_path_of(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_ai_load(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as AiApi>::og_ai_load(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_ai_parse(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as AiApi>::og_ai_parse(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_ai_get(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as AiApi>::og_ai_get(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_ai_set(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as AiApi>::og_ai_set(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_ai_restart(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as AiApi>::og_ai_restart(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_ai_finish(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as AiApi>::og_ai_finish(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_ai_noise(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as AiApi>::og_ai_noise(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_ai_describe(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as AiApi>::og_ai_describe(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_story_set_saliency(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StoryApi>::og_story_set_saliency(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_story_load(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StoryApi>::og_story_load(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_story_parse(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StoryApi>::og_story_parse(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_story_start(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StoryApi>::og_story_start(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_story_view(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StoryApi>::og_story_view(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_story_graph(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StoryApi>::og_story_graph(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_story_history(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StoryApi>::og_story_history(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_story_conversations(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StoryApi>::og_story_conversations(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_story_advance(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StoryApi>::og_story_advance(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_story_choose(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StoryApi>::og_story_choose(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_story_stop(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StoryApi>::og_story_stop(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_story_bark(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StoryApi>::og_story_bark(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_story_ambient(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StoryApi>::og_story_ambient(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_story_barks(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StoryApi>::og_story_barks(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_story_get(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StoryApi>::og_story_get(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_story_set(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StoryApi>::og_story_set(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_story_reset(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StoryApi>::og_story_reset(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_story_start_quest(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StoryApi>::og_story_start_quest(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_story_progress(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StoryApi>::og_story_progress(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_story_set_stage(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StoryApi>::og_story_set_stage(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_story_complete_quest(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StoryApi>::og_story_complete_quest(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_story_fail_quest(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StoryApi>::og_story_fail_quest(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_story_quest(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StoryApi>::og_story_quest(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_story_quests(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StoryApi>::og_story_quests(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_story_track(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StoryApi>::og_story_track(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_story_mark_seen(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StoryApi>::og_story_mark_seen(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_story_mark(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StoryApi>::og_story_mark(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_store_get(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_store_get(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_store_set(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_store_set(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_catalog_head(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_catalog_head(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_store_list(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_store_list(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_leaderboard_submit(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_leaderboard_submit(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_leaderboard_top(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_leaderboard_top(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_leaderboard_around(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_leaderboard_around(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_telemetry(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_telemetry(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_point(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_point(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_ai_mark(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_ai_mark(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_load_settings(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_load_settings(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_save_settings(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_save_settings(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_save_slot(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_save_slot(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_load_slot(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_load_slot(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_products(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_products(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_purchase(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_purchase(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_entitlements(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_entitlements(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_consume(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_consume(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_grant(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_grant(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_granted(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_granted(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_ad_ready(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_ad_ready(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_show_ad(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_show_ad(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_achievement_unlock(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_achievement_unlock(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_achievement_progress(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_achievement_progress(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_achievements(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_achievements(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_player(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_player(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_friends(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_friends(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_invite_send(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_invite_send(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_invites(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_invites(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_share(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_share(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_supports(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_supports(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_notice(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_notice(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_result(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_result(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_info(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_info(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_journal(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_journal(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_now(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_now(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_room_create(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_room_create(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_room_join(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_room_join(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_quick_play(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_quick_play(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_room_leave(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_room_leave(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_room_send(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_room_send(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_room_peers(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_room_peers(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_room_info(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_room_info(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_platform_room_message(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PlatformApi>::og_platform_room_message(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_i18n_load(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as I18nApi>::og_i18n_load(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_i18n_set_locale(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as I18nApi>::og_i18n_set_locale(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_i18n_locale(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as I18nApi>::og_i18n_locale(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_i18n_t(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as I18nApi>::og_i18n_t(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_i18n_has(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as I18nApi>::og_i18n_has(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_i18n_format_number(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as I18nApi>::og_i18n_format_number(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_i18n_format_compact(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as I18nApi>::og_i18n_format_compact(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_i18n_format_currency(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as I18nApi>::og_i18n_format_currency(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_i18n_format_date(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as I18nApi>::og_i18n_format_date(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_i18n_format_time(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as I18nApi>::og_i18n_format_time(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_i18n_format_duration(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as I18nApi>::og_i18n_format_duration(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_ui_configure(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as UiApi>::og_ui_configure(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_ui_viewport(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as UiApi>::og_ui_viewport(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_ui_theme(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as UiApi>::og_ui_theme(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_ui_frame_pointer(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as UiApi>::og_ui_frame_pointer(&Host);
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_ui_frame_cursor(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as UiApi>::og_ui_frame_cursor(&Host);
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_ui_frame_viewport(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as UiApi>::og_ui_frame_viewport(&Host);
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_ui_has_icon(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as UiApi>::og_ui_has_icon(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_ui_seconds(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as UiApi>::og_ui_seconds(&Host);
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_ui_frame_direction(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as UiApi>::og_ui_frame_direction(&Host);
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_ui_frame_theme(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as UiApi>::og_ui_frame_theme(&Host);
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_ui_set_virtual_cursor(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as UiApi>::og_ui_set_virtual_cursor(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_ui_virtual_cursor(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as UiApi>::og_ui_virtual_cursor(&Host);
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_ui_feedback(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as UiApi>::og_ui_feedback(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_ui_set_haptics(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as UiApi>::og_ui_set_haptics(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_ui_tick(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as UiApi>::og_ui_tick(&Host);
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_ui_preview_viewport(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as UiApi>::og_ui_preview_viewport(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_commands_register(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as CommandsApi>::og_commands_register(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_commands_parse(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as CommandsApi>::og_commands_parse(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_commands_help(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as CommandsApi>::og_commands_help(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_commands_schedule(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as CommandsApi>::og_commands_schedule(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_runtime_probe_get_version(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_runtime_probe_get_version(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_runtime_probe_tick(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_runtime_probe_tick(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_runtime_probe_state(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_runtime_probe_state(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_runtime_probe_advance(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_runtime_probe_advance(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_runtime_probe_pause(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_runtime_probe_pause(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_runtime_probe_resume(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_runtime_probe_resume(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_runtime_probe_input(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_runtime_probe_input(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_runtime_probe_accepted(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_runtime_probe_accepted(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_runtime_probe_pick(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_runtime_probe_pick(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_runtime_probe_entity(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_runtime_probe_entity(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_runtime_probe_bounds(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_runtime_probe_bounds(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_runtime_probe_patch(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_runtime_probe_patch(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_runtime_probe_run(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_runtime_probe_run(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_probe_hello(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_engine_probe_hello(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_engine_probe_capture(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_engine_probe_capture(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) }, unsafe { crate::arg(args, 4) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_engine_probe_capture_views(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_engine_probe_capture_views(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) }, unsafe { crate::arg(args, 4) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_engine_probe_visible(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_engine_probe_visible(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_engine_probe_marks(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_engine_probe_marks(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_engine_probe_timeline(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_engine_probe_timeline(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_engine_probe_sweep(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_engine_probe_sweep(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_engine_probe_reload(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_engine_probe_reload(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_engine_probe_snapshot(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_engine_probe_snapshot(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_engine_probe_restore(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_engine_probe_restore(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_engine_probe_dump(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_engine_probe_dump(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_engine_probe_commands(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_engine_probe_commands(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_engine_probe_listen(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_engine_probe_listen(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_engine_probe_memory(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_engine_probe_memory(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_engine_probe_describe(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_engine_probe_describe(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_engine_probe_expect(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_engine_probe_expect(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_engine_probe_expectations(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_engine_probe_expectations(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_engine_probe_assertions(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_engine_probe_assertions(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_engine_probe_assertion_events(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_engine_probe_assertion_events(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_engine_probe_observe(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_engine_probe_observe(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_engine_probe_save_scene(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_engine_probe_save_scene(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_engine_probe_pick_ui(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as ProbeApi>::og_engine_probe_pick_ui(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_terrain_load(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TerrainApi>::og_terrain_load(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_terrain_sample(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TerrainApi>::og_terrain_sample(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_terrain_bounds(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TerrainApi>::og_terrain_bounds(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) }, unsafe { crate::arg(args, 4) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_terrain_edit(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TerrainApi>::og_terrain_edit(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_terrain_stamp(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TerrainApi>::og_terrain_stamp(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_terrain_unstamp(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TerrainApi>::og_terrain_unstamp(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_terrain_layer(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TerrainApi>::og_terrain_layer(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_terrain_set_layer(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TerrainApi>::og_terrain_set_layer(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_stream_status(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StreamApi>::og_stream_status(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_stream_request(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StreamApi>::og_stream_request(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_stream_release(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StreamApi>::og_stream_release(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_stream_budget(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StreamApi>::og_stream_budget(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_stream_set_budget(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as StreamApi>::og_stream_set_budget(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_video_capabilities(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as VideoApi>::og_video_capabilities(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_debug_line(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as DebugApi>::og_debug_line(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_debug_box(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as DebugApi>::og_debug_box(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_debug_sphere(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as DebugApi>::og_debug_sphere(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_debug_arrow(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as DebugApi>::og_debug_arrow(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_debug_axes(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as DebugApi>::og_debug_axes(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_debug_text(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as DebugApi>::og_debug_text(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_debug_clear(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as DebugApi>::og_debug_clear(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_debug_set_view(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as DebugApi>::og_debug_set_view(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_debug_view(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as DebugApi>::og_debug_view(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_debug_set_enabled(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as DebugApi>::og_debug_set_enabled(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_debug_enabled(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as DebugApi>::og_debug_enabled(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

#[cfg(not(feature = "publish"))]
unsafe extern "C" fn og_assert_record(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as AssertApi>::og_assert_record(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_path_create(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PathApi>::og_path_create(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_path_find(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PathApi>::og_path_find(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_path_sample(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PathApi>::og_path_sample(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_path_length(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PathApi>::og_path_length(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_path_nearest(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as PathApi>::og_path_nearest(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_timeline_captions(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TimelineApi>::og_timeline_captions(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_timeline_info(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TimelineApi>::og_timeline_info(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_timeline_play(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TimelineApi>::og_timeline_play(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_timeline_pause(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TimelineApi>::og_timeline_pause(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_timeline_seek(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TimelineApi>::og_timeline_seek(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_timeline_stop(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as TimelineApi>::og_timeline_stop(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_net_submit(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as NetApi>::og_net_submit(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_net_shape(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as NetApi>::og_net_shape(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_net_submit_input(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as NetApi>::og_net_submit_input(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_net_command(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as NetApi>::og_net_command(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_net_commands(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as NetApi>::og_net_commands(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_net_status(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as NetApi>::og_net_status(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_net_resimulating(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as NetApi>::og_net_resimulating(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_net_final_hash(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as NetApi>::og_net_final_hash(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_net_hash(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as NetApi>::og_net_hash(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_shared_shape(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as SharedApi>::og_shared_shape(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_shared_join(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as SharedApi>::og_shared_join(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_shared_leave(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as SharedApi>::og_shared_leave(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_shared_teleport(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as SharedApi>::og_shared_teleport(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_shared_send(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as SharedApi>::og_shared_send(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_shared_events(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as SharedApi>::og_shared_events(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_shared_put(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as SharedApi>::og_shared_put(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) }, unsafe { crate::arg(args, 4) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_shared_remove(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as SharedApi>::og_shared_remove(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_shared_object(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as SharedApi>::og_shared_object(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_shared_remote(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as SharedApi>::og_shared_remote(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_shared_status(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as SharedApi>::og_shared_status(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_motion_pose(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as MotionApi>::og_motion_pose(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_motion_place(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as MotionApi>::og_motion_place(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_motion_face(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as MotionApi>::og_motion_face(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_motion_walk_to(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as MotionApi>::og_motion_walk_to(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_motion_jump(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as MotionApi>::og_motion_jump(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_motion_clear(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as MotionApi>::og_motion_clear(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_motion_look_at(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as MotionApi>::og_motion_look_at(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_motion_react(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as MotionApi>::og_motion_react(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_motion_status(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as MotionApi>::og_motion_status(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_motion_capabilities(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as MotionApi>::og_motion_capabilities(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_motion_set_backend(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as MotionApi>::og_motion_set_backend(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_motion_presentation(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as MotionApi>::og_motion_presentation(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_motion_poses(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as MotionApi>::og_motion_poses(&Host);
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_destruction_damage(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as DestructionApi>::og_destruction_damage(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_destruction_explode(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as DestructionApi>::og_destruction_explode(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_destruction_fracture(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as DestructionApi>::og_destruction_fracture(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_destruction_info(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as DestructionApi>::og_destruction_info(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_destruction_material(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as DestructionApi>::og_destruction_material(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_destruction_capabilities(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as DestructionApi>::og_destruction_capabilities(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_destruction_carve2d(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as DestructionApi>::og_destruction_carve2d(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_destruction_fill2d(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as DestructionApi>::og_destruction_fill2d(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_destruction_explode2d(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as DestructionApi>::og_destruction_explode2d(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_destruction_fracture2d(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as DestructionApi>::og_destruction_fracture2d(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_destruction_cell2d(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as DestructionApi>::og_destruction_cell2d(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_vfx_spawn(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as VfxApi>::og_vfx_spawn(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_vfx_info(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as VfxApi>::og_vfx_info(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_sight_visible(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as SightApi>::og_sight_visible(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_sight_explored(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as SightApi>::og_sight_explored(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_sight_reveal(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as SightApi>::og_sight_reveal(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) }, unsafe { crate::arg(args, 3) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_sight_forget(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as SightApi>::og_sight_forget(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_film_load(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as FilmApi>::og_film_load(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_film_start(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as FilmApi>::og_film_start(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_film_view(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as FilmApi>::og_film_view(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_film_choose(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as FilmApi>::og_film_choose(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) }, unsafe { crate::arg(args, 2) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_film_play(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as FilmApi>::og_film_play(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_film_pause(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as FilmApi>::og_film_pause(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_film_skip(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as FilmApi>::og_film_skip(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_film_chart(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as FilmApi>::og_film_chart(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_film_stop(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as FilmApi>::og_film_stop(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_scene_preload(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as SceneApi>::og_scene_preload(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_scene_progress(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as SceneApi>::og_scene_progress(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_scene_release(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as SceneApi>::og_scene_release(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_scene_change(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as SceneApi>::og_scene_change(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_scene_current(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as SceneApi>::og_scene_current(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_loading_stage(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadingApi>::og_loading_stage(&Host, unsafe { crate::arg(args, 0) }, unsafe { crate::arg(args, 1) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_loading_done(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadingApi>::og_loading_done(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_loading_idle(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadingApi>::og_loading_idle(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

unsafe extern "C" fn og_loading_status(args: *const crate::OgValue, out: *mut crate::OgValue) {
    let r = <Host as LoadingApi>::og_loading_status(&Host, unsafe { crate::arg(args, 0) });
    unsafe { crate::IntoC::into_c(r, out) }
}

/// Every entry, in the order of `packages/engine/src/generated/abi.ts`.
pub static SYMBOLS: &[crate::Symbol] = &[
    crate::Symbol { name: c"og_host_set_dispatcher", arity: 1, call: og_host_set_dispatcher },
    crate::Symbol { name: c"og_host_shared", arity: 0, call: og_host_shared },
    crate::Symbol { name: c"og_engine_create", arity: 2, call: og_engine_create },
    crate::Symbol { name: c"og_engine_destroy", arity: 1, call: og_engine_destroy },
    crate::Symbol { name: c"og_engine_advance", arity: 2, call: og_engine_advance },
    crate::Symbol { name: c"og_engine_command", arity: 3, call: og_engine_command },
    crate::Symbol { name: c"og_engine_frame", arity: 2, call: og_engine_frame },
    crate::Symbol { name: c"og_host_begin_bundle", arity: 0, call: og_host_begin_bundle },
    crate::Symbol { name: c"og_engine_call", arity: 2, call: og_engine_call },
    crate::Symbol { name: c"og_engine_take_outputs", arity: 1, call: og_engine_take_outputs },
    crate::Symbol { name: c"og_engine_take_log", arity: 1, call: og_engine_take_log },
    crate::Symbol { name: c"og_engine_write_resource", arity: 3, call: og_engine_write_resource },
    crate::Symbol { name: c"og_engine_resource_layout", arity: 2, call: og_engine_resource_layout },
    crate::Symbol { name: c"og_engine_read_resource", arity: 2, call: og_engine_read_resource },
    crate::Symbol { name: c"og_engine_write_resource_bytes", arity: 3, call: og_engine_write_resource_bytes },
    crate::Symbol { name: c"og_engine_component_id", arity: 2, call: og_engine_component_id },
    crate::Symbol { name: c"og_engine_component_layout", arity: 2, call: og_engine_component_layout },
    crate::Symbol { name: c"og_engine_read_component", arity: 3, call: og_engine_read_component },
    crate::Symbol { name: c"og_engine_write_component", arity: 4, call: og_engine_write_component },
    crate::Symbol { name: c"og_engine_query_archetypes", arity: 2, call: og_engine_query_archetypes },
    crate::Symbol { name: c"og_engine_view_column", arity: 2, call: og_engine_view_column },
    crate::Symbol { name: c"og_engine_view_add_scaled", arity: 6, call: og_engine_view_add_scaled },
    crate::Symbol { name: c"og_engine_ui_patch", arity: 2, call: og_engine_ui_patch },
    crate::Symbol { name: c"og_engine_inspector_panel", arity: 1, call: og_engine_inspector_panel },
    crate::Symbol { name: c"og_engine_inspector_event", arity: 2, call: og_engine_inspector_event },
    crate::Symbol { name: c"og_platform_take", arity: 1, call: og_platform_take },
    crate::Symbol { name: c"og_platform_answer", arity: 3, call: og_platform_answer },
    crate::Symbol { name: c"og_platform_set_host", arity: 2, call: og_platform_set_host },
    crate::Symbol { name: c"og_platform_notify", arity: 2, call: og_platform_notify },
    crate::Symbol { name: c"og_room_take", arity: 1, call: og_room_take },
    crate::Symbol { name: c"og_room_deliver", arity: 2, call: og_room_deliver },
    crate::Symbol { name: c"og_room_set_host", arity: 2, call: og_room_set_host },
    crate::Symbol { name: c"og_shared_take", arity: 1, call: og_shared_take },
    crate::Symbol { name: c"og_shared_deliver", arity: 2, call: og_shared_deliver },
    crate::Symbol { name: c"og_engine_set_content", arity: 2, call: og_engine_set_content },
    crate::Symbol { name: c"og_engine_trace", arity: 2, call: og_engine_trace },
    crate::Symbol { name: c"og_engine_trace_take", arity: 2, call: og_engine_trace_take },
    crate::Symbol { name: c"og_engine_perf", arity: 2, call: og_engine_perf },
    crate::Symbol { name: c"og_engine_perf_report", arity: 2, call: og_engine_perf_report },
    crate::Symbol { name: c"og_engine_perf_frames", arity: 1, call: og_engine_perf_frames },
    crate::Symbol { name: c"og_present_frame", arity: 2, call: og_present_frame },
    crate::Symbol { name: c"og_present_resume", arity: 1, call: og_present_resume },
    crate::Symbol { name: c"og_present_refresh", arity: 2, call: og_present_refresh },
    crate::Symbol { name: c"og_present_fullscreen", arity: 2, call: og_present_fullscreen },
    crate::Symbol { name: c"og_reflect_define_component", arity: 2, call: og_reflect_define_component },
    crate::Symbol { name: c"og_reflect_define_resource", arity: 2, call: og_reflect_define_resource },
    crate::Symbol { name: c"og_reflect_define_event", arity: 2, call: og_reflect_define_event },
    crate::Symbol { name: c"og_save_edit_get_version", arity: 1, call: og_save_edit_get_version },
    crate::Symbol { name: c"og_save_edit_types", arity: 1, call: og_save_edit_types },
    crate::Symbol { name: c"og_save_edit_records", arity: 2, call: og_save_edit_records },
    crate::Symbol { name: c"og_save_edit_set_records", arity: 3, call: og_save_edit_set_records },
    crate::Symbol { name: c"og_save_edit_rename_type", arity: 3, call: og_save_edit_rename_type },
    crate::Symbol { name: c"og_save_edit_rename_field", arity: 4, call: og_save_edit_rename_field },
    crate::Symbol { name: c"og_save_edit_drop_type", arity: 2, call: og_save_edit_drop_type },
    crate::Symbol { name: c"og_save_edit_spawn", arity: 2, call: og_save_edit_spawn },
    crate::Symbol { name: c"og_save_edit_text", arity: 1, call: og_save_edit_text },
    crate::Symbol { name: c"og_save_edit_close", arity: 1, call: og_save_edit_close },
    crate::Symbol { name: c"og_spatial_index_get_count", arity: 1, call: og_spatial_index_get_count },
    crate::Symbol { name: c"og_spatial_index_within", arity: 3, call: og_spatial_index_within },
    crate::Symbol { name: c"og_spatial_index_nearest", arity: 3, call: og_spatial_index_nearest },
    crate::Symbol { name: c"og_world_get_tick", arity: 1, call: og_world_get_tick },
    crate::Symbol { name: c"og_world_spawn", arity: 2, call: og_world_spawn },
    crate::Symbol { name: c"og_world_despawn", arity: 2, call: og_world_despawn },
    crate::Symbol { name: c"og_world_alive", arity: 2, call: og_world_alive },
    crate::Symbol { name: c"og_world_has", arity: 3, call: og_world_has },
    crate::Symbol { name: c"og_world_insert", arity: 4, call: og_world_insert },
    crate::Symbol { name: c"og_world_remove", arity: 3, call: og_world_remove },
    crate::Symbol { name: c"og_world_get", arity: 3, call: og_world_get },
    crate::Symbol { name: c"og_world_set", arity: 4, call: og_world_set },
    crate::Symbol { name: c"og_world_resource", arity: 2, call: og_world_resource },
    crate::Symbol { name: c"og_world_children", arity: 2, call: og_world_children },
    crate::Symbol { name: c"og_world_removed", arity: 2, call: og_world_removed },
    crate::Symbol { name: c"og_world_spatial", arity: 2, call: og_world_spatial },
    crate::Symbol { name: c"og_world_state", arity: 2, call: og_world_state },
    crate::Symbol { name: c"og_world_set_state", arity: 3, call: og_world_set_state },
    crate::Symbol { name: c"og_world_find", arity: 2, call: og_world_find },
    crate::Symbol { name: c"og_world_prefab_of", arity: 2, call: og_world_prefab_of },
    crate::Symbol { name: c"og_world_name_of", arity: 2, call: og_world_name_of },
    crate::Symbol { name: c"og_world_query", arity: 2, call: og_world_query },
    crate::Symbol { name: c"og_world_events", arity: 2, call: og_world_events },
    crate::Symbol { name: c"og_world_emit", arity: 3, call: og_world_emit },
    crate::Symbol { name: c"og_world_instantiate", arity: 3, call: og_world_instantiate },
    crate::Symbol { name: c"og_world_snapshot", arity: 2, call: og_world_snapshot },
    crate::Symbol { name: c"og_world_open_save", arity: 2, call: og_world_open_save },
    crate::Symbol { name: c"og_world_restore", arity: 3, call: og_world_restore },
    crate::Symbol { name: c"og_query_count", arity: 1, call: og_query_count },
    crate::Symbol { name: c"og_archetype_view_get_count", arity: 1, call: og_archetype_view_get_count },
    crate::Symbol { name: c"og_archetype_view_get_tick", arity: 1, call: og_archetype_view_get_tick },
    crate::Symbol { name: c"og_archetype_view_get_entities", arity: 1, call: og_archetype_view_get_entities },
    crate::Symbol { name: c"og_archetype_view_get_rows", arity: 1, call: og_archetype_view_get_rows },
    crate::Symbol { name: c"og_archetype_view_entity", arity: 2, call: og_archetype_view_entity },
    crate::Symbol { name: c"og_hook_get_name", arity: 1, call: og_hook_get_name },
    crate::Symbol { name: c"og_system_define_hook", arity: 4, call: og_system_define_hook },
    crate::Symbol { name: c"og_system_get_name", arity: 1, call: og_system_get_name },
    crate::Symbol { name: c"og_system_get_reads", arity: 1, call: og_system_get_reads },
    crate::Symbol { name: c"og_system_get_writes", arity: 1, call: og_system_get_writes },
    crate::Symbol { name: c"og_system_define_system", arity: 3, call: og_system_define_system },
    crate::Symbol { name: c"og_game_get_tick", arity: 1, call: og_game_get_tick },
    crate::Symbol { name: c"og_game_get_systems", arity: 1, call: og_game_get_systems },
    crate::Symbol { name: c"og_game_game", arity: 1, call: og_game_game },
    crate::Symbol { name: c"og_transform_compose", arity: 3, call: og_transform_compose },
    crate::Symbol { name: c"og_transform_decompose", arity: 1, call: og_transform_decompose },
    crate::Symbol { name: c"og_transform_look_at", arity: 3, call: og_transform_look_at },
    crate::Symbol { name: c"og_transform_face", arity: 3, call: og_transform_face },
    crate::Symbol { name: c"og_transform_forward", arity: 1, call: og_transform_forward },
    crate::Symbol { name: c"og_transform_right", arity: 1, call: og_transform_right },
    crate::Symbol { name: c"og_transform_up", arity: 1, call: og_transform_up },
    crate::Symbol { name: c"og_transform_camera_forward", arity: 1, call: og_transform_camera_forward },
    crate::Symbol { name: c"og_transform_camera_right", arity: 1, call: og_transform_camera_right },
    crate::Symbol { name: c"og_transform_heading", arity: 1, call: og_transform_heading },
    crate::Symbol { name: c"og_transform_from_heading", arity: 1, call: og_transform_from_heading },
    crate::Symbol { name: c"og_transform_face2d", arity: 2, call: og_transform_face2d },
    crate::Symbol { name: c"og_transform_forward2d", arity: 1, call: og_transform_forward2d },
    crate::Symbol { name: c"og_transform_heading2d", arity: 1, call: og_transform_heading2d },
    crate::Symbol { name: c"og_transform_from_heading2d", arity: 1, call: og_transform_from_heading2d },
    crate::Symbol { name: c"og_transform_from_axis_angle", arity: 2, call: og_transform_from_axis_angle },
    crate::Symbol { name: c"og_time_set_scale", arity: 2, call: og_time_set_scale },
    crate::Symbol { name: c"og_time_hit_stop", arity: 2, call: og_time_hit_stop },
    crate::Symbol { name: c"og_time_set_group_scale", arity: 3, call: og_time_set_group_scale },
    crate::Symbol { name: c"og_time_group_dt", arity: 2, call: og_time_group_dt },
    crate::Symbol { name: c"og_rng_stream_get_seed", arity: 1, call: og_rng_stream_get_seed },
    crate::Symbol { name: c"og_rng_stream_next", arity: 1, call: og_rng_stream_next },
    crate::Symbol { name: c"og_rng_stream_range", arity: 3, call: og_rng_stream_range },
    crate::Symbol { name: c"og_rng_stream_integer", arity: 3, call: og_rng_stream_integer },
    crate::Symbol { name: c"og_rng_stream_chance", arity: 2, call: og_rng_stream_chance },
    crate::Symbol { name: c"og_rng_stream_weighted", arity: 2, call: og_rng_stream_weighted },
    crate::Symbol { name: c"og_rng_stream_permutation", arity: 2, call: og_rng_stream_permutation },
    crate::Symbol { name: c"og_rng_stream_fork", arity: 2, call: og_rng_stream_fork },
    crate::Symbol { name: c"og_rng_next", arity: 1, call: og_rng_next },
    crate::Symbol { name: c"og_rng_range", arity: 3, call: og_rng_range },
    crate::Symbol { name: c"og_rng_integer", arity: 3, call: og_rng_integer },
    crate::Symbol { name: c"og_rng_chance", arity: 2, call: og_rng_chance },
    crate::Symbol { name: c"og_rng_weighted", arity: 2, call: og_rng_weighted },
    crate::Symbol { name: c"og_rng_permutation", arity: 2, call: og_rng_permutation },
    crate::Symbol { name: c"og_rng_fork", arity: 2, call: og_rng_fork },
    crate::Symbol { name: c"og_procgen_noise", arity: 4, call: og_procgen_noise },
    crate::Symbol { name: c"og_procgen_noise_grid", arity: 1, call: og_procgen_noise_grid },
    crate::Symbol { name: c"og_procgen_poisson_disk", arity: 1, call: og_procgen_poisson_disk },
    crate::Symbol { name: c"og_procgen_bsp", arity: 1, call: og_procgen_bsp },
    crate::Symbol { name: c"og_procgen_wfc", arity: 1, call: og_procgen_wfc },
    crate::Symbol { name: c"og_input_intent", arity: 3, call: og_input_intent },
    crate::Symbol { name: c"og_input_take_intents", arity: 1, call: og_input_take_intents },
    crate::Symbol { name: c"og_input_replay_intent", arity: 2, call: og_input_replay_intent },
    crate::Symbol { name: c"og_input_define_action", arity: 2, call: og_input_define_action },
    crate::Symbol { name: c"og_input_actions", arity: 1, call: og_input_actions },
    crate::Symbol { name: c"og_input_action", arity: 2, call: og_input_action },
    crate::Symbol { name: c"og_input_is_action_down", arity: 2, call: og_input_is_action_down },
    crate::Symbol { name: c"og_input_is_action_just_pressed", arity: 2, call: og_input_is_action_just_pressed },
    crate::Symbol { name: c"og_input_is_action_just_released", arity: 2, call: og_input_is_action_just_released },
    crate::Symbol { name: c"og_input_is_action_held", arity: 2, call: og_input_is_action_held },
    crate::Symbol { name: c"og_input_action_value", arity: 2, call: og_input_action_value },
    crate::Symbol { name: c"og_input_action_vector", arity: 2, call: og_input_action_vector },
    crate::Symbol { name: c"og_input_move_relative", arity: 3, call: og_input_move_relative },
    crate::Symbol { name: c"og_input_set_action_maps", arity: 2, call: og_input_set_action_maps },
    crate::Symbol { name: c"og_input_action_maps", arity: 1, call: og_input_action_maps },
    crate::Symbol { name: c"og_input_rebind", arity: 2, call: og_input_rebind },
    crate::Symbol { name: c"og_input_conflicts", arity: 2, call: og_input_conflicts },
    crate::Symbol { name: c"og_input_reset_bindings", arity: 2, call: og_input_reset_bindings },
    crate::Symbol { name: c"og_input_set_toggle", arity: 3, call: og_input_set_toggle },
    crate::Symbol { name: c"og_input_save_bindings", arity: 1, call: og_input_save_bindings },
    crate::Symbol { name: c"og_input_load_bindings", arity: 2, call: og_input_load_bindings },
    crate::Symbol { name: c"og_input_start_capture", arity: 2, call: og_input_start_capture },
    crate::Symbol { name: c"og_input_capture", arity: 1, call: og_input_capture },
    crate::Symbol { name: c"og_input_cancel_capture", arity: 1, call: og_input_cancel_capture },
    crate::Symbol { name: c"og_input_apply_capture", arity: 2, call: og_input_apply_capture },
    crate::Symbol { name: c"og_input_glyph", arity: 3, call: og_input_glyph },
    crate::Symbol { name: c"og_input_glyphs", arity: 3, call: og_input_glyphs },
    crate::Symbol { name: c"og_input_control_glyph", arity: 2, call: og_input_control_glyph },
    crate::Symbol { name: c"og_input_device", arity: 1, call: og_input_device },
    crate::Symbol { name: c"og_input_pad_device", arity: 2, call: og_input_pad_device },
    crate::Symbol { name: c"og_input_set_deadzones", arity: 2, call: og_input_set_deadzones },
    crate::Symbol { name: c"og_input_set_sensitivity", arity: 2, call: og_input_set_sensitivity },
    crate::Symbol { name: c"og_input_sensitivity", arity: 3, call: og_input_sensitivity },
    crate::Symbol { name: c"og_input_set_keyboard_layout", arity: 1, call: og_input_set_keyboard_layout },
    crate::Symbol { name: c"og_input_keyboard_layout", arity: 0, call: og_input_keyboard_layout },
    crate::Symbol { name: c"og_input_set_mouse", arity: 2, call: og_input_set_mouse },
    crate::Symbol { name: c"og_input_mouse", arity: 1, call: og_input_mouse },
    crate::Symbol { name: c"og_input_rumble", arity: 2, call: og_input_rumble },
    crate::Symbol { name: c"og_input_down", arity: 2, call: og_input_down },
    crate::Symbol { name: c"og_input_just_pressed", arity: 2, call: og_input_just_pressed },
    crate::Symbol { name: c"og_input_just_released", arity: 2, call: og_input_just_released },
    crate::Symbol { name: c"og_input_axis", arity: 3, call: og_input_axis },
    crate::Symbol { name: c"og_input_axis2", arity: 5, call: og_input_axis2 },
    crate::Symbol { name: c"og_input_button_down", arity: 2, call: og_input_button_down },
    crate::Symbol { name: c"og_input_button_just_pressed", arity: 2, call: og_input_button_just_pressed },
    crate::Symbol { name: c"og_input_button_just_released", arity: 2, call: og_input_button_just_released },
    crate::Symbol { name: c"og_pointers_down", arity: 2, call: og_pointers_down },
    crate::Symbol { name: c"og_pointers_just_pressed", arity: 2, call: og_pointers_just_pressed },
    crate::Symbol { name: c"og_pointers_just_released", arity: 2, call: og_pointers_just_released },
    crate::Symbol { name: c"og_pointers_position", arity: 2, call: og_pointers_position },
    crate::Symbol { name: c"og_pointers_start", arity: 2, call: og_pointers_start },
    crate::Symbol { name: c"og_pointers_count", arity: 1, call: og_pointers_count },
    crate::Symbol { name: c"og_gamepad_connected", arity: 2, call: og_gamepad_connected },
    crate::Symbol { name: c"og_gamepad_down", arity: 3, call: og_gamepad_down },
    crate::Symbol { name: c"og_gamepad_just_pressed", arity: 3, call: og_gamepad_just_pressed },
    crate::Symbol { name: c"og_gamepad_just_released", arity: 3, call: og_gamepad_just_released },
    crate::Symbol { name: c"og_gamepad_axis", arity: 3, call: og_gamepad_axis },
    crate::Symbol { name: c"og_camera_look_at", arity: 2, call: og_camera_look_at },
    crate::Symbol { name: c"og_camera_world_to_screen", arity: 2, call: og_camera_world_to_screen },
    crate::Symbol { name: c"og_camera_screen_to_ray", arity: 3, call: og_camera_screen_to_ray },
    crate::Symbol { name: c"og_camera_shake", arity: 2, call: og_camera_shake },
    crate::Symbol { name: c"og_camera_punch", arity: 2, call: og_camera_punch },
    crate::Symbol { name: c"og_camera_kick", arity: 3, call: og_camera_kick },
    crate::Symbol { name: c"og_render_snap", arity: 2, call: og_render_snap },
    crate::Symbol { name: c"og_render_snap_camera", arity: 1, call: og_render_snap_camera },
    crate::Symbol { name: c"og_render_compile", arity: 1, call: og_render_compile },
    crate::Symbol { name: c"og_render_reveal", arity: 3, call: og_render_reveal },
    crate::Symbol { name: c"og_render_ribbon", arity: 2, call: og_render_ribbon },
    crate::Symbol { name: c"og_render_sprite", arity: 2, call: og_render_sprite },
    crate::Symbol { name: c"og_render_floating_text", arity: 2, call: og_render_floating_text },
    crate::Symbol { name: c"og_render_measure_text", arity: 2, call: og_render_measure_text },
    crate::Symbol { name: c"og_render_box_geometry", arity: 2, call: og_render_box_geometry },
    crate::Symbol { name: c"og_render_plane_geometry", arity: 2, call: og_render_plane_geometry },
    crate::Symbol { name: c"og_render_sphere_geometry", arity: 2, call: og_render_sphere_geometry },
    crate::Symbol { name: c"og_render_standard_material", arity: 2, call: og_render_standard_material },
    crate::Symbol { name: c"og_render_update_material", arity: 3, call: og_render_update_material },
    crate::Symbol { name: c"og_render_shader_material", arity: 3, call: og_render_shader_material },
    crate::Symbol { name: c"og_render_set_material_shader", arity: 3, call: og_render_set_material_shader },
    crate::Symbol { name: c"og_render_set_shader_param", arity: 4, call: og_render_set_shader_param },
    crate::Symbol { name: c"og_render_apply_look", arity: 2, call: og_render_apply_look },
    crate::Symbol { name: c"og_render_apply_quality", arity: 2, call: og_render_apply_quality },
    crate::Symbol { name: c"og_render_set_display", arity: 2, call: og_render_set_display },
    crate::Symbol { name: c"og_render_reset_display", arity: 1, call: og_render_reset_display },
    crate::Symbol { name: c"og_render_display", arity: 1, call: og_render_display },
    crate::Symbol { name: c"og_render_look", arity: 1, call: og_render_look },
    crate::Symbol { name: c"og_render_capabilities", arity: 1, call: og_render_capabilities },
    crate::Symbol { name: c"og_render_text", arity: 2, call: og_render_text },
    crate::Symbol { name: c"og_render_buffer_geometry", arity: 2, call: og_render_buffer_geometry },
    crate::Symbol { name: c"og_render_update_geometry", arity: 3, call: og_render_update_geometry },
    crate::Symbol { name: c"og_render_geometry_info", arity: 2, call: og_render_geometry_info },
    crate::Symbol { name: c"og_render_cylinder_geometry", arity: 2, call: og_render_cylinder_geometry },
    crate::Symbol { name: c"og_render_cone_geometry", arity: 2, call: og_render_cone_geometry },
    crate::Symbol { name: c"og_render_capsule_geometry", arity: 2, call: og_render_capsule_geometry },
    crate::Symbol { name: c"og_render_torus_geometry", arity: 2, call: og_render_torus_geometry },
    crate::Symbol { name: c"og_render_ring_geometry", arity: 2, call: og_render_ring_geometry },
    crate::Symbol { name: c"og_render_circle_geometry", arity: 2, call: og_render_circle_geometry },
    crate::Symbol { name: c"og_render_tube_geometry", arity: 2, call: og_render_tube_geometry },
    crate::Symbol { name: c"og_render_shape_geometry", arity: 2, call: og_render_shape_geometry },
    crate::Symbol { name: c"og_render_extrude_geometry", arity: 2, call: og_render_extrude_geometry },
    crate::Symbol { name: c"og_render_lathe_geometry", arity: 2, call: og_render_lathe_geometry },
    crate::Symbol { name: c"og_render_polyhedron_geometry", arity: 2, call: og_render_polyhedron_geometry },
    crate::Symbol { name: c"og_render_icosahedron_geometry", arity: 2, call: og_render_icosahedron_geometry },
    crate::Symbol { name: c"og_render_dodecahedron_geometry", arity: 2, call: og_render_dodecahedron_geometry },
    crate::Symbol { name: c"og_render_octahedron_geometry", arity: 2, call: og_render_octahedron_geometry },
    crate::Symbol { name: c"og_render_tetrahedron_geometry", arity: 2, call: og_render_tetrahedron_geometry },
    crate::Symbol { name: c"og_render_sweep_geometry", arity: 2, call: og_render_sweep_geometry },
    crate::Symbol { name: c"og_render_csg_geometry", arity: 2, call: og_render_csg_geometry },
    crate::Symbol { name: c"og_render_merge_geometries", arity: 2, call: og_render_merge_geometries },
    crate::Symbol { name: c"og_render_lines", arity: 2, call: og_render_lines },
    crate::Symbol { name: c"og_render_points", arity: 2, call: og_render_points },
    crate::Symbol { name: c"og_render_raycast", arity: 3, call: og_render_raycast },
    crate::Symbol { name: c"og_render_light2d_at", arity: 3, call: og_render_light2d_at },
    crate::Symbol { name: c"og_render_render_target", arity: 2, call: og_render_render_target },
    crate::Symbol { name: c"og_render_set_post_param", arity: 4, call: og_render_set_post_param },
    crate::Symbol { name: c"og_render_post_errors", arity: 1, call: og_render_post_errors },
    crate::Symbol { name: c"og_render_baked_lighting", arity: 1, call: og_render_baked_lighting },
    crate::Symbol { name: c"og_load_bundle", arity: 2, call: og_load_bundle },
    crate::Symbol { name: c"og_load_bundle_info", arity: 2, call: og_load_bundle_info },
    crate::Symbol { name: c"og_load_progress", arity: 2, call: og_load_progress },
    crate::Symbol { name: c"og_load_default_bundle", arity: 1, call: og_load_default_bundle },
    crate::Symbol { name: c"og_load_gltf", arity: 3, call: og_load_gltf },
    crate::Symbol { name: c"og_load_scene", arity: 2, call: og_load_scene },
    crate::Symbol { name: c"og_load_prefab", arity: 2, call: og_load_prefab },
    crate::Symbol { name: c"og_load_font", arity: 2, call: og_load_font },
    crate::Symbol { name: c"og_load_font_info", arity: 2, call: og_load_font_info },
    crate::Symbol { name: c"og_load_sound", arity: 2, call: og_load_sound },
    crate::Symbol { name: c"og_load_lut", arity: 2, call: og_load_lut },
    crate::Symbol { name: c"og_load_environment", arity: 2, call: og_load_environment },
    crate::Symbol { name: c"og_load_environment_sun", arity: 2, call: og_load_environment_sun },
    crate::Symbol { name: c"og_load_emitter", arity: 2, call: og_load_emitter },
    crate::Symbol { name: c"og_load_effect", arity: 2, call: og_load_effect },
    crate::Symbol { name: c"og_load_shader", arity: 2, call: og_load_shader },
    crate::Symbol { name: c"og_load_post", arity: 2, call: og_load_post },
    crate::Symbol { name: c"og_load_vector", arity: 2, call: og_load_vector },
    crate::Symbol { name: c"og_load_ui_material", arity: 2, call: og_load_ui_material },
    crate::Symbol { name: c"og_load_texture", arity: 2, call: og_load_texture },
    crate::Symbol { name: c"og_load_texture_array", arity: 2, call: og_load_texture_array },
    crate::Symbol { name: c"og_load_style", arity: 2, call: og_load_style },
    crate::Symbol { name: c"og_load_look", arity: 2, call: og_load_look },
    crate::Symbol { name: c"og_load_look_info", arity: 2, call: og_load_look_info },
    crate::Symbol { name: c"og_load_palette", arity: 2, call: og_load_palette },
    crate::Symbol { name: c"og_load_sound_info", arity: 2, call: og_load_sound_info },
    crate::Symbol { name: c"og_load_status", arity: 2, call: og_load_status },
    crate::Symbol { name: c"og_load_error", arity: 2, call: og_load_error },
    crate::Symbol { name: c"og_load_gltf_info", arity: 2, call: og_load_gltf_info },
    crate::Symbol { name: c"og_load_animation", arity: 2, call: og_load_animation },
    crate::Symbol { name: c"og_load_rig", arity: 2, call: og_load_rig },
    crate::Symbol { name: c"og_load_flipbook", arity: 2, call: og_load_flipbook },
    crate::Symbol { name: c"og_load_tilemap", arity: 2, call: og_load_tilemap },
    crate::Symbol { name: c"og_load_video", arity: 2, call: og_load_video },
    crate::Symbol { name: c"og_load_video_info", arity: 2, call: og_load_video_info },
    crate::Symbol { name: c"og_load_timeline", arity: 2, call: og_load_timeline },
    crate::Symbol { name: c"og_load_vfx", arity: 2, call: og_load_vfx },
    crate::Symbol { name: c"og_load_rig2d", arity: 2, call: og_load_rig2d },
    crate::Symbol { name: c"og_load_image_data", arity: 4, call: og_load_image_data },
    crate::Symbol { name: c"og_load_gltf_data", arity: 4, call: og_load_gltf_data },
    crate::Symbol { name: c"og_load_environment_data", arity: 4, call: og_load_environment_data },
    crate::Symbol { name: c"og_load_data", arity: 2, call: og_load_data },
    crate::Symbol { name: c"og_load_data_json", arity: 2, call: og_load_data_json },
    crate::Symbol { name: c"og_load_data_bytes", arity: 2, call: og_load_data_bytes },
    crate::Symbol { name: c"og_load_pack", arity: 2, call: og_load_pack },
    crate::Symbol { name: c"og_load_environment_info", arity: 2, call: og_load_environment_info },
    crate::Symbol { name: c"og_load_release", arity: 2, call: og_load_release },
    crate::Symbol { name: c"og_load_image_info", arity: 2, call: og_load_image_info },
    crate::Symbol { name: c"og_load_completions", arity: 1, call: og_load_completions },
    crate::Symbol { name: c"og_load_replay_completions", arity: 2, call: og_load_replay_completions },
    crate::Symbol { name: c"og_physics_water", arity: 2, call: og_physics_water },
    crate::Symbol { name: c"og_physics_water_height", arity: 4, call: og_physics_water_height },
    crate::Symbol { name: c"og_physics_cast_ray", arity: 2, call: og_physics_cast_ray },
    crate::Symbol { name: c"og_physics_cast_ray_all", arity: 2, call: og_physics_cast_ray_all },
    crate::Symbol { name: c"og_physics_project_point", arity: 2, call: og_physics_project_point },
    crate::Symbol { name: c"og_physics_cast_shape", arity: 2, call: og_physics_cast_shape },
    crate::Symbol { name: c"og_physics_overlap", arity: 2, call: og_physics_overlap },
    crate::Symbol { name: c"og_physics_apply_force", arity: 4, call: og_physics_apply_force },
    crate::Symbol { name: c"og_physics_apply_impulse", arity: 4, call: og_physics_apply_impulse },
    crate::Symbol { name: c"og_physics_apply_torque", arity: 3, call: og_physics_apply_torque },
    crate::Symbol { name: c"og_physics_apply_torque_impulse", arity: 3, call: og_physics_apply_torque_impulse },
    crate::Symbol { name: c"og_physics_set_mesh", arity: 3, call: og_physics_set_mesh },
    crate::Symbol { name: c"og_physics_rope_points", arity: 2, call: og_physics_rope_points },
    crate::Symbol { name: c"og_physics_garment_points", arity: 2, call: og_physics_garment_points },
    crate::Symbol { name: c"og_physics_cloth_points", arity: 2, call: og_physics_cloth_points },
    crate::Symbol { name: c"og_physics_soft_body_points", arity: 2, call: og_physics_soft_body_points },
    crate::Symbol { name: c"og_physics_fluid_particles", arity: 2, call: og_physics_fluid_particles },
    crate::Symbol { name: c"og_physics_fluid3d_particles", arity: 2, call: og_physics_fluid3d_particles },
    crate::Symbol { name: c"og_physics_vehicle_wheels", arity: 2, call: og_physics_vehicle_wheels },
    crate::Symbol { name: c"og_physics_ragdoll_rig_from_skeleton", arity: 2, call: og_physics_ragdoll_rig_from_skeleton },
    crate::Symbol { name: c"og_physics_set_ragdoll_rig", arity: 3, call: og_physics_set_ragdoll_rig },
    crate::Symbol { name: c"og_physics_ragdoll_pose", arity: 2, call: og_physics_ragdoll_pose },
    crate::Symbol { name: c"og_physics_stats", arity: 1, call: og_physics_stats },
    crate::Symbol { name: c"og_atmosphere_time_of_day", arity: 1, call: og_atmosphere_time_of_day },
    crate::Symbol { name: c"og_atmosphere_season", arity: 1, call: og_atmosphere_season },
    crate::Symbol { name: c"og_anim_get_node_matrix", arity: 3, call: og_anim_get_node_matrix },
    crate::Symbol { name: c"og_anim_graph", arity: 2, call: og_anim_graph },
    crate::Symbol { name: c"og_anim_humanoid", arity: 2, call: og_anim_humanoid },
    crate::Symbol { name: c"og_anim_retarget", arity: 2, call: og_anim_retarget },
    crate::Symbol { name: c"og_anim_mirror", arity: 2, call: og_anim_mirror },
    crate::Symbol { name: c"og_anim_rig2d_info", arity: 2, call: og_anim_rig2d_info },
    crate::Symbol { name: c"og_anim_skeleton2d_boxes", arity: 2, call: og_anim_skeleton2d_boxes },
    crate::Symbol { name: c"og_tween_timeline_get_tween", arity: 1, call: og_tween_timeline_get_tween },
    crate::Symbol { name: c"og_tween_timeline_get_error", arity: 1, call: og_tween_timeline_get_error },
    crate::Symbol { name: c"og_tween_timeline_to", arity: 4, call: og_tween_timeline_to },
    crate::Symbol { name: c"og_tween_timeline_from", arity: 4, call: og_tween_timeline_from },
    crate::Symbol { name: c"og_tween_timeline_from_to", arity: 4, call: og_tween_timeline_from_to },
    crate::Symbol { name: c"og_tween_timeline_set", arity: 4, call: og_tween_timeline_set },
    crate::Symbol { name: c"og_tween_timeline_call", arity: 3, call: og_tween_timeline_call },
    crate::Symbol { name: c"og_tween_timeline_add_label", arity: 3, call: og_tween_timeline_add_label },
    crate::Symbol { name: c"og_tween_timeline_add", arity: 3, call: og_tween_timeline_add },
    crate::Symbol { name: c"og_tween_timeline_play", arity: 2, call: og_tween_timeline_play },
    crate::Symbol { name: c"og_tween_timeline_pause", arity: 2, call: og_tween_timeline_pause },
    crate::Symbol { name: c"og_tween_timeline_resume", arity: 1, call: og_tween_timeline_resume },
    crate::Symbol { name: c"og_tween_timeline_reverse", arity: 2, call: og_tween_timeline_reverse },
    crate::Symbol { name: c"og_tween_timeline_restart", arity: 1, call: og_tween_timeline_restart },
    crate::Symbol { name: c"og_tween_timeline_seek", arity: 2, call: og_tween_timeline_seek },
    crate::Symbol { name: c"og_tween_timeline_progress", arity: 2, call: og_tween_timeline_progress },
    crate::Symbol { name: c"og_tween_timeline_time", arity: 1, call: og_tween_timeline_time },
    crate::Symbol { name: c"og_tween_timeline_duration", arity: 1, call: og_tween_timeline_duration },
    crate::Symbol { name: c"og_tween_timeline_time_scale", arity: 2, call: og_tween_timeline_time_scale },
    crate::Symbol { name: c"og_tween_timeline_is_active", arity: 1, call: og_tween_timeline_is_active },
    crate::Symbol { name: c"og_tween_timeline_kill", arity: 1, call: og_tween_timeline_kill },
    crate::Symbol { name: c"og_tween_to", arity: 2, call: og_tween_to },
    crate::Symbol { name: c"og_tween_from", arity: 2, call: og_tween_from },
    crate::Symbol { name: c"og_tween_from_to", arity: 2, call: og_tween_from_to },
    crate::Symbol { name: c"og_tween_sequence", arity: 3, call: og_tween_sequence },
    crate::Symbol { name: c"og_tween_parallel", arity: 3, call: og_tween_parallel },
    crate::Symbol { name: c"og_tween_timeline", arity: 2, call: og_tween_timeline },
    crate::Symbol { name: c"og_tween_cancel", arity: 2, call: og_tween_cancel },
    crate::Symbol { name: c"og_tween_cancel_entity", arity: 2, call: og_tween_cancel_entity },
    crate::Symbol { name: c"og_tween_active", arity: 2, call: og_tween_active },
    crate::Symbol { name: c"og_tween_squash", arity: 2, call: og_tween_squash },
    crate::Symbol { name: c"og_tween_flash", arity: 2, call: og_tween_flash },
    crate::Symbol { name: c"og_tilemap_set_tile", arity: 2, call: og_tilemap_set_tile },
    crate::Symbol { name: c"og_tilemap_fill", arity: 2, call: og_tilemap_fill },
    crate::Symbol { name: c"og_tilemap_create", arity: 2, call: og_tilemap_create },
    crate::Symbol { name: c"og_tilemap_resize", arity: 2, call: og_tilemap_resize },
    crate::Symbol { name: c"og_tilemap_neighbours", arity: 4, call: og_tilemap_neighbours },
    crate::Symbol { name: c"og_tilemap_get_tile", arity: 5, call: og_tilemap_get_tile },
    crate::Symbol { name: c"og_tilemap_cell_at", arity: 3, call: og_tilemap_cell_at },
    crate::Symbol { name: c"og_tilemap_cell_center", arity: 4, call: og_tilemap_cell_center },
    crate::Symbol { name: c"og_audio_play_feedback", arity: 3, call: og_audio_play_feedback },
    crate::Symbol { name: c"og_audio_create_sound", arity: 2, call: og_audio_create_sound },
    crate::Symbol { name: c"og_audio_output_latency", arity: 1, call: og_audio_output_latency },
    crate::Symbol { name: c"og_nav_grid", arity: 2, call: og_nav_grid },
    crate::Symbol { name: c"og_nav_grid_from_tilemap", arity: 2, call: og_nav_grid_from_tilemap },
    crate::Symbol { name: c"og_nav_set_cost", arity: 5, call: og_nav_set_cost },
    crate::Symbol { name: c"og_nav_cell_at", arity: 3, call: og_nav_cell_at },
    crate::Symbol { name: c"og_nav_find_path", arity: 2, call: og_nav_find_path },
    crate::Symbol { name: c"og_nav_flow_field", arity: 2, call: og_nav_flow_field },
    crate::Symbol { name: c"og_nav_set_flow_targets", arity: 3, call: og_nav_set_flow_targets },
    crate::Symbol { name: c"og_nav_flow_at", arity: 3, call: og_nav_flow_at },
    crate::Symbol { name: c"og_nav_load", arity: 3, call: og_nav_load },
    crate::Symbol { name: c"og_nav_build", arity: 2, call: og_nav_build },
    crate::Symbol { name: c"og_nav_set_obstacle", arity: 4, call: og_nav_set_obstacle },
    crate::Symbol { name: c"og_nav_carve", arity: 4, call: og_nav_carve },
    crate::Symbol { name: c"og_nav_set_area", arity: 4, call: og_nav_set_area },
    crate::Symbol { name: c"og_nav_set_link", arity: 4, call: og_nav_set_link },
    crate::Symbol { name: c"og_nav_set_link_open", arity: 4, call: og_nav_set_link_open },
    crate::Symbol { name: c"og_nav_finish_link", arity: 2, call: og_nav_finish_link },
    crate::Symbol { name: c"og_nav_nearest", arity: 4, call: og_nav_nearest },
    crate::Symbol { name: c"og_nav_path_of", arity: 2, call: og_nav_path_of },
    crate::Symbol { name: c"og_ai_load", arity: 2, call: og_ai_load },
    crate::Symbol { name: c"og_ai_parse", arity: 2, call: og_ai_parse },
    crate::Symbol { name: c"og_ai_get", arity: 3, call: og_ai_get },
    crate::Symbol { name: c"og_ai_set", arity: 4, call: og_ai_set },
    crate::Symbol { name: c"og_ai_restart", arity: 2, call: og_ai_restart },
    crate::Symbol { name: c"og_ai_finish", arity: 4, call: og_ai_finish },
    crate::Symbol { name: c"og_ai_noise", arity: 2, call: og_ai_noise },
    crate::Symbol { name: c"og_ai_describe", arity: 2, call: og_ai_describe },
    crate::Symbol { name: c"og_story_set_saliency", arity: 2, call: og_story_set_saliency },
    crate::Symbol { name: c"og_story_load", arity: 2, call: og_story_load },
    crate::Symbol { name: c"og_story_parse", arity: 2, call: og_story_parse },
    crate::Symbol { name: c"og_story_start", arity: 2, call: og_story_start },
    crate::Symbol { name: c"og_story_view", arity: 2, call: og_story_view },
    crate::Symbol { name: c"og_story_graph", arity: 2, call: og_story_graph },
    crate::Symbol { name: c"og_story_history", arity: 2, call: og_story_history },
    crate::Symbol { name: c"og_story_conversations", arity: 1, call: og_story_conversations },
    crate::Symbol { name: c"og_story_advance", arity: 2, call: og_story_advance },
    crate::Symbol { name: c"og_story_choose", arity: 3, call: og_story_choose },
    crate::Symbol { name: c"og_story_stop", arity: 2, call: og_story_stop },
    crate::Symbol { name: c"og_story_bark", arity: 2, call: og_story_bark },
    crate::Symbol { name: c"og_story_ambient", arity: 2, call: og_story_ambient },
    crate::Symbol { name: c"og_story_barks", arity: 1, call: og_story_barks },
    crate::Symbol { name: c"og_story_get", arity: 2, call: og_story_get },
    crate::Symbol { name: c"og_story_set", arity: 3, call: og_story_set },
    crate::Symbol { name: c"og_story_reset", arity: 1, call: og_story_reset },
    crate::Symbol { name: c"og_story_start_quest", arity: 2, call: og_story_start_quest },
    crate::Symbol { name: c"og_story_progress", arity: 4, call: og_story_progress },
    crate::Symbol { name: c"og_story_set_stage", arity: 3, call: og_story_set_stage },
    crate::Symbol { name: c"og_story_complete_quest", arity: 2, call: og_story_complete_quest },
    crate::Symbol { name: c"og_story_fail_quest", arity: 2, call: og_story_fail_quest },
    crate::Symbol { name: c"og_story_quest", arity: 2, call: og_story_quest },
    crate::Symbol { name: c"og_story_quests", arity: 1, call: og_story_quests },
    crate::Symbol { name: c"og_story_track", arity: 3, call: og_story_track },
    crate::Symbol { name: c"og_story_mark_seen", arity: 2, call: og_story_mark_seen },
    crate::Symbol { name: c"og_story_mark", arity: 3, call: og_story_mark },
    crate::Symbol { name: c"og_platform_store_get", arity: 2, call: og_platform_store_get },
    crate::Symbol { name: c"og_platform_store_set", arity: 2, call: og_platform_store_set },
    crate::Symbol { name: c"og_platform_catalog_head", arity: 2, call: og_platform_catalog_head },
    crate::Symbol { name: c"og_platform_store_list", arity: 1, call: og_platform_store_list },
    crate::Symbol { name: c"og_platform_leaderboard_submit", arity: 2, call: og_platform_leaderboard_submit },
    crate::Symbol { name: c"og_platform_leaderboard_top", arity: 2, call: og_platform_leaderboard_top },
    crate::Symbol { name: c"og_platform_leaderboard_around", arity: 2, call: og_platform_leaderboard_around },
    crate::Symbol { name: c"og_platform_telemetry", arity: 3, call: og_platform_telemetry },
    crate::Symbol { name: c"og_platform_point", arity: 2, call: og_platform_point },
    crate::Symbol { name: c"og_platform_ai_mark", arity: 2, call: og_platform_ai_mark },
    crate::Symbol { name: c"og_platform_load_settings", arity: 1, call: og_platform_load_settings },
    crate::Symbol { name: c"og_platform_save_settings", arity: 2, call: og_platform_save_settings },
    crate::Symbol { name: c"og_platform_save_slot", arity: 2, call: og_platform_save_slot },
    crate::Symbol { name: c"og_platform_load_slot", arity: 2, call: og_platform_load_slot },
    crate::Symbol { name: c"og_platform_products", arity: 2, call: og_platform_products },
    crate::Symbol { name: c"og_platform_purchase", arity: 2, call: og_platform_purchase },
    crate::Symbol { name: c"og_platform_entitlements", arity: 1, call: og_platform_entitlements },
    crate::Symbol { name: c"og_platform_consume", arity: 2, call: og_platform_consume },
    crate::Symbol { name: c"og_platform_grant", arity: 2, call: og_platform_grant },
    crate::Symbol { name: c"og_platform_granted", arity: 2, call: og_platform_granted },
    crate::Symbol { name: c"og_platform_ad_ready", arity: 2, call: og_platform_ad_ready },
    crate::Symbol { name: c"og_platform_show_ad", arity: 2, call: og_platform_show_ad },
    crate::Symbol { name: c"og_platform_achievement_unlock", arity: 2, call: og_platform_achievement_unlock },
    crate::Symbol { name: c"og_platform_achievement_progress", arity: 2, call: og_platform_achievement_progress },
    crate::Symbol { name: c"og_platform_achievements", arity: 1, call: og_platform_achievements },
    crate::Symbol { name: c"og_platform_player", arity: 1, call: og_platform_player },
    crate::Symbol { name: c"og_platform_friends", arity: 2, call: og_platform_friends },
    crate::Symbol { name: c"og_platform_invite_send", arity: 2, call: og_platform_invite_send },
    crate::Symbol { name: c"og_platform_invites", arity: 1, call: og_platform_invites },
    crate::Symbol { name: c"og_platform_share", arity: 2, call: og_platform_share },
    crate::Symbol { name: c"og_platform_supports", arity: 2, call: og_platform_supports },
    crate::Symbol { name: c"og_platform_notice", arity: 2, call: og_platform_notice },
    crate::Symbol { name: c"og_platform_result", arity: 2, call: og_platform_result },
    crate::Symbol { name: c"og_platform_info", arity: 1, call: og_platform_info },
    crate::Symbol { name: c"og_platform_journal", arity: 1, call: og_platform_journal },
    crate::Symbol { name: c"og_platform_now", arity: 1, call: og_platform_now },
    crate::Symbol { name: c"og_platform_room_create", arity: 2, call: og_platform_room_create },
    crate::Symbol { name: c"og_platform_room_join", arity: 2, call: og_platform_room_join },
    crate::Symbol { name: c"og_platform_quick_play", arity: 2, call: og_platform_quick_play },
    crate::Symbol { name: c"og_platform_room_leave", arity: 1, call: og_platform_room_leave },
    crate::Symbol { name: c"og_platform_room_send", arity: 3, call: og_platform_room_send },
    crate::Symbol { name: c"og_platform_room_peers", arity: 1, call: og_platform_room_peers },
    crate::Symbol { name: c"og_platform_room_info", arity: 1, call: og_platform_room_info },
    crate::Symbol { name: c"og_platform_room_message", arity: 2, call: og_platform_room_message },
    crate::Symbol { name: c"og_i18n_load", arity: 2, call: og_i18n_load },
    crate::Symbol { name: c"og_i18n_set_locale", arity: 2, call: og_i18n_set_locale },
    crate::Symbol { name: c"og_i18n_locale", arity: 1, call: og_i18n_locale },
    crate::Symbol { name: c"og_i18n_t", arity: 3, call: og_i18n_t },
    crate::Symbol { name: c"og_i18n_has", arity: 2, call: og_i18n_has },
    crate::Symbol { name: c"og_i18n_format_number", arity: 3, call: og_i18n_format_number },
    crate::Symbol { name: c"og_i18n_format_compact", arity: 3, call: og_i18n_format_compact },
    crate::Symbol { name: c"og_i18n_format_currency", arity: 3, call: og_i18n_format_currency },
    crate::Symbol { name: c"og_i18n_format_date", arity: 3, call: og_i18n_format_date },
    crate::Symbol { name: c"og_i18n_format_time", arity: 3, call: og_i18n_format_time },
    crate::Symbol { name: c"og_i18n_format_duration", arity: 3, call: og_i18n_format_duration },
    crate::Symbol { name: c"og_ui_configure", arity: 2, call: og_ui_configure },
    crate::Symbol { name: c"og_ui_viewport", arity: 1, call: og_ui_viewport },
    crate::Symbol { name: c"og_ui_theme", arity: 1, call: og_ui_theme },
    crate::Symbol { name: c"og_ui_frame_pointer", arity: 0, call: og_ui_frame_pointer },
    crate::Symbol { name: c"og_ui_frame_cursor", arity: 0, call: og_ui_frame_cursor },
    crate::Symbol { name: c"og_ui_frame_viewport", arity: 0, call: og_ui_frame_viewport },
    crate::Symbol { name: c"og_ui_has_icon", arity: 1, call: og_ui_has_icon },
    crate::Symbol { name: c"og_ui_seconds", arity: 0, call: og_ui_seconds },
    crate::Symbol { name: c"og_ui_frame_direction", arity: 0, call: og_ui_frame_direction },
    crate::Symbol { name: c"og_ui_frame_theme", arity: 0, call: og_ui_frame_theme },
    crate::Symbol { name: c"og_ui_set_virtual_cursor", arity: 1, call: og_ui_set_virtual_cursor },
    crate::Symbol { name: c"og_ui_virtual_cursor", arity: 0, call: og_ui_virtual_cursor },
    crate::Symbol { name: c"og_ui_feedback", arity: 2, call: og_ui_feedback },
    crate::Symbol { name: c"og_ui_set_haptics", arity: 1, call: og_ui_set_haptics },
    crate::Symbol { name: c"og_ui_tick", arity: 0, call: og_ui_tick },
    crate::Symbol { name: c"og_ui_preview_viewport", arity: 2, call: og_ui_preview_viewport },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_commands_register", arity: 3, call: og_commands_register },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_commands_parse", arity: 1, call: og_commands_parse },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_commands_help", arity: 1, call: og_commands_help },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_commands_schedule", arity: 2, call: og_commands_schedule },
    crate::Symbol { name: c"og_runtime_probe_get_version", arity: 1, call: og_runtime_probe_get_version },
    crate::Symbol { name: c"og_runtime_probe_tick", arity: 1, call: og_runtime_probe_tick },
    crate::Symbol { name: c"og_runtime_probe_state", arity: 1, call: og_runtime_probe_state },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_runtime_probe_advance", arity: 2, call: og_runtime_probe_advance },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_runtime_probe_pause", arity: 1, call: og_runtime_probe_pause },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_runtime_probe_resume", arity: 1, call: og_runtime_probe_resume },
    crate::Symbol { name: c"og_runtime_probe_input", arity: 2, call: og_runtime_probe_input },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_runtime_probe_accepted", arity: 1, call: og_runtime_probe_accepted },
    crate::Symbol { name: c"og_runtime_probe_pick", arity: 3, call: og_runtime_probe_pick },
    crate::Symbol { name: c"og_runtime_probe_entity", arity: 2, call: og_runtime_probe_entity },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_runtime_probe_bounds", arity: 2, call: og_runtime_probe_bounds },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_runtime_probe_patch", arity: 4, call: og_runtime_probe_patch },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_runtime_probe_run", arity: 2, call: og_runtime_probe_run },
    crate::Symbol { name: c"og_engine_probe_hello", arity: 1, call: og_engine_probe_hello },
    crate::Symbol { name: c"og_engine_probe_capture", arity: 5, call: og_engine_probe_capture },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_engine_probe_capture_views", arity: 5, call: og_engine_probe_capture_views },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_engine_probe_visible", arity: 2, call: og_engine_probe_visible },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_engine_probe_marks", arity: 2, call: og_engine_probe_marks },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_engine_probe_timeline", arity: 1, call: og_engine_probe_timeline },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_engine_probe_sweep", arity: 2, call: og_engine_probe_sweep },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_engine_probe_reload", arity: 1, call: og_engine_probe_reload },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_engine_probe_snapshot", arity: 1, call: og_engine_probe_snapshot },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_engine_probe_restore", arity: 2, call: og_engine_probe_restore },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_engine_probe_dump", arity: 2, call: og_engine_probe_dump },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_engine_probe_commands", arity: 2, call: og_engine_probe_commands },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_engine_probe_listen", arity: 2, call: og_engine_probe_listen },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_engine_probe_memory", arity: 1, call: og_engine_probe_memory },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_engine_probe_describe", arity: 1, call: og_engine_probe_describe },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_engine_probe_expect", arity: 2, call: og_engine_probe_expect },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_engine_probe_expectations", arity: 1, call: og_engine_probe_expectations },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_engine_probe_assertions", arity: 2, call: og_engine_probe_assertions },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_engine_probe_assertion_events", arity: 1, call: og_engine_probe_assertion_events },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_engine_probe_observe", arity: 3, call: og_engine_probe_observe },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_engine_probe_save_scene", arity: 3, call: og_engine_probe_save_scene },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_engine_probe_pick_ui", arity: 3, call: og_engine_probe_pick_ui },
    crate::Symbol { name: c"og_terrain_load", arity: 2, call: og_terrain_load },
    crate::Symbol { name: c"og_terrain_sample", arity: 2, call: og_terrain_sample },
    crate::Symbol { name: c"og_terrain_bounds", arity: 5, call: og_terrain_bounds },
    crate::Symbol { name: c"og_terrain_edit", arity: 3, call: og_terrain_edit },
    crate::Symbol { name: c"og_terrain_stamp", arity: 2, call: og_terrain_stamp },
    crate::Symbol { name: c"og_terrain_unstamp", arity: 3, call: og_terrain_unstamp },
    crate::Symbol { name: c"og_terrain_layer", arity: 3, call: og_terrain_layer },
    crate::Symbol { name: c"og_terrain_set_layer", arity: 4, call: og_terrain_set_layer },
    crate::Symbol { name: c"og_stream_status", arity: 2, call: og_stream_status },
    crate::Symbol { name: c"og_stream_request", arity: 2, call: og_stream_request },
    crate::Symbol { name: c"og_stream_release", arity: 2, call: og_stream_release },
    crate::Symbol { name: c"og_stream_budget", arity: 1, call: og_stream_budget },
    crate::Symbol { name: c"og_stream_set_budget", arity: 2, call: og_stream_set_budget },
    crate::Symbol { name: c"og_video_capabilities", arity: 1, call: og_video_capabilities },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_debug_line", arity: 4, call: og_debug_line },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_debug_box", arity: 4, call: og_debug_box },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_debug_sphere", arity: 4, call: og_debug_sphere },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_debug_arrow", arity: 4, call: og_debug_arrow },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_debug_axes", arity: 4, call: og_debug_axes },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_debug_text", arity: 4, call: og_debug_text },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_debug_clear", arity: 1, call: og_debug_clear },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_debug_set_view", arity: 2, call: og_debug_set_view },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_debug_view", arity: 1, call: og_debug_view },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_debug_set_enabled", arity: 2, call: og_debug_set_enabled },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_debug_enabled", arity: 1, call: og_debug_enabled },
    #[cfg(not(feature = "publish"))]
    crate::Symbol { name: c"og_assert_record", arity: 2, call: og_assert_record },
    crate::Symbol { name: c"og_path_create", arity: 2, call: og_path_create },
    crate::Symbol { name: c"og_path_find", arity: 2, call: og_path_find },
    crate::Symbol { name: c"og_path_sample", arity: 3, call: og_path_sample },
    crate::Symbol { name: c"og_path_length", arity: 2, call: og_path_length },
    crate::Symbol { name: c"og_path_nearest", arity: 3, call: og_path_nearest },
    crate::Symbol { name: c"og_timeline_captions", arity: 1, call: og_timeline_captions },
    crate::Symbol { name: c"og_timeline_info", arity: 2, call: og_timeline_info },
    crate::Symbol { name: c"og_timeline_play", arity: 2, call: og_timeline_play },
    crate::Symbol { name: c"og_timeline_pause", arity: 2, call: og_timeline_pause },
    crate::Symbol { name: c"og_timeline_seek", arity: 3, call: og_timeline_seek },
    crate::Symbol { name: c"og_timeline_stop", arity: 2, call: og_timeline_stop },
    crate::Symbol { name: c"og_net_submit", arity: 2, call: og_net_submit },
    crate::Symbol { name: c"og_net_shape", arity: 2, call: og_net_shape },
    crate::Symbol { name: c"og_net_submit_input", arity: 3, call: og_net_submit_input },
    crate::Symbol { name: c"og_net_command", arity: 3, call: og_net_command },
    crate::Symbol { name: c"og_net_commands", arity: 2, call: og_net_commands },
    crate::Symbol { name: c"og_net_status", arity: 1, call: og_net_status },
    crate::Symbol { name: c"og_net_resimulating", arity: 1, call: og_net_resimulating },
    crate::Symbol { name: c"og_net_final_hash", arity: 2, call: og_net_final_hash },
    crate::Symbol { name: c"og_net_hash", arity: 1, call: og_net_hash },
    crate::Symbol { name: c"og_shared_shape", arity: 2, call: og_shared_shape },
    crate::Symbol { name: c"og_shared_join", arity: 2, call: og_shared_join },
    crate::Symbol { name: c"og_shared_leave", arity: 1, call: og_shared_leave },
    crate::Symbol { name: c"og_shared_teleport", arity: 2, call: og_shared_teleport },
    crate::Symbol { name: c"og_shared_send", arity: 3, call: og_shared_send },
    crate::Symbol { name: c"og_shared_events", arity: 2, call: og_shared_events },
    crate::Symbol { name: c"og_shared_put", arity: 5, call: og_shared_put },
    crate::Symbol { name: c"og_shared_remove", arity: 3, call: og_shared_remove },
    crate::Symbol { name: c"og_shared_object", arity: 2, call: og_shared_object },
    crate::Symbol { name: c"og_shared_remote", arity: 2, call: og_shared_remote },
    crate::Symbol { name: c"og_shared_status", arity: 1, call: og_shared_status },
    crate::Symbol { name: c"og_motion_pose", arity: 3, call: og_motion_pose },
    crate::Symbol { name: c"og_motion_place", arity: 3, call: og_motion_place },
    crate::Symbol { name: c"og_motion_face", arity: 3, call: og_motion_face },
    crate::Symbol { name: c"og_motion_walk_to", arity: 3, call: og_motion_walk_to },
    crate::Symbol { name: c"og_motion_jump", arity: 3, call: og_motion_jump },
    crate::Symbol { name: c"og_motion_clear", arity: 2, call: og_motion_clear },
    crate::Symbol { name: c"og_motion_look_at", arity: 3, call: og_motion_look_at },
    crate::Symbol { name: c"og_motion_react", arity: 3, call: og_motion_react },
    crate::Symbol { name: c"og_motion_status", arity: 2, call: og_motion_status },
    crate::Symbol { name: c"og_motion_capabilities", arity: 1, call: og_motion_capabilities },
    crate::Symbol { name: c"og_motion_set_backend", arity: 2, call: og_motion_set_backend },
    crate::Symbol { name: c"og_motion_presentation", arity: 2, call: og_motion_presentation },
    crate::Symbol { name: c"og_motion_poses", arity: 0, call: og_motion_poses },
    crate::Symbol { name: c"og_destruction_damage", arity: 3, call: og_destruction_damage },
    crate::Symbol { name: c"og_destruction_explode", arity: 2, call: og_destruction_explode },
    crate::Symbol { name: c"og_destruction_fracture", arity: 2, call: og_destruction_fracture },
    crate::Symbol { name: c"og_destruction_info", arity: 2, call: og_destruction_info },
    crate::Symbol { name: c"og_destruction_material", arity: 2, call: og_destruction_material },
    crate::Symbol { name: c"og_destruction_capabilities", arity: 1, call: og_destruction_capabilities },
    crate::Symbol { name: c"og_destruction_carve2d", arity: 3, call: og_destruction_carve2d },
    crate::Symbol { name: c"og_destruction_fill2d", arity: 3, call: og_destruction_fill2d },
    crate::Symbol { name: c"og_destruction_explode2d", arity: 2, call: og_destruction_explode2d },
    crate::Symbol { name: c"og_destruction_fracture2d", arity: 3, call: og_destruction_fracture2d },
    crate::Symbol { name: c"og_destruction_cell2d", arity: 3, call: og_destruction_cell2d },
    crate::Symbol { name: c"og_vfx_spawn", arity: 2, call: og_vfx_spawn },
    crate::Symbol { name: c"og_vfx_info", arity: 2, call: og_vfx_info },
    crate::Symbol { name: c"og_sight_visible", arity: 3, call: og_sight_visible },
    crate::Symbol { name: c"og_sight_explored", arity: 3, call: og_sight_explored },
    crate::Symbol { name: c"og_sight_reveal", arity: 4, call: og_sight_reveal },
    crate::Symbol { name: c"og_sight_forget", arity: 2, call: og_sight_forget },
    crate::Symbol { name: c"og_film_load", arity: 2, call: og_film_load },
    crate::Symbol { name: c"og_film_start", arity: 2, call: og_film_start },
    crate::Symbol { name: c"og_film_view", arity: 2, call: og_film_view },
    crate::Symbol { name: c"og_film_choose", arity: 3, call: og_film_choose },
    crate::Symbol { name: c"og_film_play", arity: 2, call: og_film_play },
    crate::Symbol { name: c"og_film_pause", arity: 2, call: og_film_pause },
    crate::Symbol { name: c"og_film_skip", arity: 2, call: og_film_skip },
    crate::Symbol { name: c"og_film_chart", arity: 2, call: og_film_chart },
    crate::Symbol { name: c"og_film_stop", arity: 2, call: og_film_stop },
    crate::Symbol { name: c"og_scene_preload", arity: 2, call: og_scene_preload },
    crate::Symbol { name: c"og_scene_progress", arity: 2, call: og_scene_progress },
    crate::Symbol { name: c"og_scene_release", arity: 2, call: og_scene_release },
    crate::Symbol { name: c"og_scene_change", arity: 2, call: og_scene_change },
    crate::Symbol { name: c"og_scene_current", arity: 1, call: og_scene_current },
    crate::Symbol { name: c"og_loading_stage", arity: 2, call: og_loading_stage },
    crate::Symbol { name: c"og_loading_done", arity: 1, call: og_loading_done },
    crate::Symbol { name: c"og_loading_idle", arity: 1, call: og_loading_idle },
    crate::Symbol { name: c"og_loading_status", arity: 1, call: og_loading_status },
];
