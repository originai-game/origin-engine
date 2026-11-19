//! The C ABI as a shell calls it: entries found by name in og_c_symbols'
//! table, arguments and answers as tagged OgValues.

use og_host_c::{OG_BOOL, OG_NUMBER, OG_SHARED, OG_STRING, OgSymbol, OgValue, og_c_symbols};
use std::ffi::CStr;

fn table() -> &'static [OgSymbol] {
    let mut n = 0;
    // SAFETY: the table lives for the process.
    unsafe { std::slice::from_raw_parts(og_c_symbols(&mut n), n) }
}

fn call(name: &str, args: &[OgValue]) -> OgValue {
    let s = table()
        .iter()
        .find(|s| unsafe { CStr::from_ptr(s.name) }.to_str() == Ok(name))
        .unwrap_or_else(|| panic!("no entry {name}"));
    assert_eq!(s.arity, args.len(), "{name}'s arity");
    let mut out = OgValue {
        tag: 0,
        len: 0,
        num: 0.0,
        ptr: std::ptr::null(),
    };
    unsafe { (s.call)(args.as_ptr(), &mut out) };
    out
}

fn number(n: f64) -> OgValue {
    OgValue {
        tag: OG_NUMBER,
        len: 0,
        num: n,
        ptr: std::ptr::null(),
    }
}

fn text(s: &str) -> OgValue {
    OgValue {
        tag: OG_STRING,
        len: s.len() as u32,
        num: 0.0,
        ptr: s.as_ptr().cast(),
    }
}

fn answer(v: &OgValue) -> String {
    assert_eq!(v.tag, OG_STRING);
    let bytes = unsafe { std::slice::from_raw_parts(v.ptr.cast::<u8>(), v.len as usize) };
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[test]
fn entries_take_and_answer_tagged_values() {
    // The table is the IDL's: the lifecycle members first, every module after.
    let names: Vec<&str> = table()
        .iter()
        .map(|s| unsafe { CStr::from_ptr(s.name) }.to_str().unwrap())
        .collect();
    assert_eq!(names[0], "og_host_set_dispatcher");
    assert!(names.contains(&"og_engine_view_add_scaled"));
    assert!(names.contains(&"og_archetype_view_entity"));

    let shared = call("og_host_shared", &[]);
    assert_eq!((shared.tag, shared.len), (OG_SHARED, 65536));
    assert!(!shared.ptr.is_null());

    // A string argument read, a string answer kept for the caller.
    let refused = answer(&call(
        "og_engine_create",
        &[number(4321.0), text(r#"{"seed":7}"#)],
    ));
    assert!(refused.starts_with(r#"{"ok":false"#), "{refused}");

    let destroyed = call("og_engine_destroy", &[number(4321.0)]);
    assert_eq!((destroyed.tag, destroyed.num), (OG_BOOL, 0.0));

    // A component number for a world that does not exist: u32::MAX as a number.
    let id = call("og_engine_component_id", &[number(4321.0), text("Nope")]);
    assert_eq!((id.tag, id.num), (OG_NUMBER, f64::from(u32::MAX)));
}
