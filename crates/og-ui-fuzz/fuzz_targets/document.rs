//! `og_ui_fuzz::document` over arbitrary bytes.
#![no_main]

libfuzzer_sys::fuzz_target!(|bytes: &[u8]| og_ui_fuzz::document(bytes));
