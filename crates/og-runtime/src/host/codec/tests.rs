use super::*;
use og_api::host::{FieldDecl, RecordDecl};

fn field(
    name: &'static str,
    column: &'static str,
    default: &'static str,
    capacity: u32,
) -> FieldDecl {
    FieldDecl {
        name,
        column,
        default,
        readonly: false,
        tunable: false,
        min: None,
        max: None,
        step: None,
        unit: "",
        label: "",
        doc: "",
        variants: &[],
        capacity,
        list: 0,
        worldspace: "",
    }
}

/// Every engine record's schema carries its declared defaults, so a
/// save from before a field was added restores it at that default
/// (og-ecs fills added fields from the schema): the bytes the schema's
/// defaults pack to are the record's encoded defaults, field by field,
/// not zero (`og.Camera.fov` is 60).
#[test]
fn an_engine_records_schema_defaults_are_its_declared_ones() {
    let mut nonzero = 0;
    for decl in og_api::host::RECORDS {
        let Ok(layout) = Layout::from_decl(decl) else {
            continue;
        };
        let encoded = layout.encode(None, None).unwrap();
        let schema = layout.schema();
        let mut at = 0;
        for f in &schema.fields {
            let size = f.ty.size();
            let want = &encoded[at..at + size];
            at += size;
            if f.ty.is_string() || f.ty.is_list() {
                continue;
            }
            assert_eq!(f.default_bytes(), want, "{}.{}", schema.name, f.name);
            nonzero += usize::from(want.iter().any(|&b| b != 0));
        }
        assert_eq!(at, encoded.len(), "{}", schema.name);
    }
    assert!(nonzero > 20, "only {nonzero} non-zero defaults checked");
}

/// A resource read every frame is decoded once while its bytes stay
/// the same (one JSON shared by the layout's clones) and again when
/// they change, always what a fresh decode answers.
#[test]
fn a_kept_decode_is_redone_only_when_the_bytes_change() {
    let decl = og_api::host::RECORDS
        .iter()
        .find(|d| d.schema == "og.Camera")
        .expect("og.Camera");
    let layout = Layout::from_decl(decl).unwrap();
    let clone = layout.clone();
    let a = layout.encode(None, None).unwrap();
    let b = layout
        .encode(Some(&a), Some(&json!({ "fov": 35.0 })))
        .unwrap();
    assert_ne!(a, b);
    let first = layout.decode_kept(a.clone());
    assert_eq!(*first, layout.decode(&a));
    let again = clone.decode_kept(a.clone());
    assert!(std::sync::Arc::ptr_eq(&first, &again), "decoded again");
    let changed = layout.decode_kept(b.clone());
    assert!(!std::sync::Arc::ptr_eq(&first, &changed));
    assert_eq!(*changed, layout.decode(&b));
    assert_eq!(changed["fov"], json!(35.0));
    assert_eq!(*layout.decode_kept(a.clone()), layout.decode(&a));
}

/// An engine record's text field (`str.24` in the IDL) is stored at the
/// IDL's capacity, and its default encodes and decodes as the text.
#[test]
fn a_row_reads_what_the_decoded_json_answers() {
    // Every engine record, at its defaults and with its bytes scrambled
    // (NaNs, infinities, enum indices past their variants, odd text).
    let mut seed = 0x9e37_79b9_7f4a_7c15_u64;
    let mut checked = 0;
    for decl in og_api::host::RECORDS {
        let Ok(layout) = Layout::from_decl(decl) else {
            continue;
        };
        let defaults = layout.encode(None, None).unwrap();
        let scrambled: Vec<u8> = (0..defaults.len())
            .map(|_| {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                seed as u8
            })
            .collect();
        for bytes in [&defaults, &scrambled] {
            let value = layout.decode(bytes);
            let row = Row::new(&layout, bytes);
            let names = layout.fields.iter().map(|f| f.name.as_str());
            for key in names.chain(["missing"]) {
                let at = format!("{}.{key}", layout.schema);
                // The same reads by name and by a slot resolved once.
                let slot = Slot::of(&layout, String::leak(key.to_owned()));
                assert_eq!(
                    row.num(key).map(f64::to_bits),
                    value.num(key).map(f64::to_bits),
                    "{at} num"
                );
                assert_eq!(
                    row.num(&slot).map(f64::to_bits),
                    value.num(key).map(f64::to_bits),
                    "{at} num by slot"
                );
                assert_eq!(row.int(key), value.int(key), "{at} int");
                assert_eq!(row.int(&slot), value.int(key), "{at} int by slot");
                assert_eq!(row.uint(key), value.uint(key), "{at} uint");
                assert_eq!(row.uint(&slot), value.uint(key), "{at} uint by slot");
                assert_eq!(row.flag(key), value.flag(key), "{at} flag");
                assert_eq!(row.flag(&slot), value.flag(key), "{at} flag by slot");
                assert_eq!(row.text(key), value.text(key), "{at} text");
                assert_eq!(row.text(&slot), value.text(key), "{at} text by slot");
                for lane in ["x", "y", "z", "w", "r", "g", "b", "elements"] {
                    assert_eq!(
                        row.lane(key, lane).map(f64::to_bits),
                        value.lane(key, lane).map(f64::to_bits),
                        "{at}.{lane}"
                    );
                    assert_eq!(
                        row.lane(&slot, lane).map(f64::to_bits),
                        value.lane(key, lane).map(f64::to_bits),
                        "{at}.{lane} by slot"
                    );
                }
                checked += 1;
            }
        }
    }
    assert!(checked > 500, "{checked} fields read");
}

/// A row read from its archetype's columns answers what a `Row` over
/// its packed bytes does, whichever way the archetype's values are
/// held, and packs to those bytes.
#[test]
fn a_column_row_reads_what_the_packed_row_answers() {
    use std::cell::{Ref, RefCell};
    let mut seed = 0x2545_f491_4f6c_dd1d_u64;
    let mut checked = 0;
    for decl in og_api::host::RECORDS {
        let Ok(layout) = Layout::from_decl(decl) else {
            continue;
        };
        // Three rows that differ: the defaults and two scrambled ones.
        let mut rows = vec![layout.encode(None, None).unwrap()];
        for _ in 0..2 {
            rows.push(
                (0..layout.size())
                    .map(|_| {
                        seed ^= seed << 13;
                        seed ^= seed >> 7;
                        seed ^= seed << 17;
                        seed as u8
                    })
                    .collect(),
            );
        }
        let columns: Vec<RefCell<Vec<u8>>> = (0..layout.fields.len())
            .map(|i| {
                let at = layout.offset(i);
                let size = layout.fields[i].ty.size();
                RefCell::new(
                    rows.iter()
                        .flat_map(|r| r[at..at + size].to_vec())
                        .collect(),
                )
            })
            .collect();
        let by_column = Cells::Columns(
            columns
                .iter()
                .zip(layout.fields.iter())
                .map(|(c, f)| Col::of(Ref::map(c.borrow(), |c| c.as_slice()), f))
                .collect(),
        );
        let by_row = Cells::Rows {
            bytes: rows.concat(),
            stride: layout.size(),
        };
        for (cells, held) in [(&by_column, "columns"), (&by_row, "rows")] {
            for (i, bytes) in rows.iter().enumerate() {
                let row = Row::new(&layout, bytes);
                let col = ColRow::new(&layout, cells, i);
                let mut packed = Vec::new();
                col.pack(&mut packed);
                assert_eq!(&packed, bytes, "{} row {i} of {held}", layout.schema);
                let names = layout.fields.iter().map(|f| f.name.as_str());
                for key in names.chain(["missing"]) {
                    let at = format!("{}.{key} row {i} of {held}", layout.schema);
                    let slot = Slot::of(&layout, String::leak(key.to_owned()));
                    assert_eq!(
                        col.num(&slot).map(f64::to_bits),
                        row.num(key).map(f64::to_bits),
                        "{at} num"
                    );
                    assert_eq!(col.int(&slot), row.int(key), "{at} int");
                    assert_eq!(col.uint(key), row.uint(key), "{at} uint");
                    assert_eq!(col.flag(&slot), row.flag(key), "{at} flag");
                    assert_eq!(col.text(&slot), row.text(key), "{at} text");
                    for lane in ["x", "y", "z", "w", "r", "g", "b", "elements"] {
                        assert_eq!(
                            col.lane(&slot, lane).map(f64::to_bits),
                            row.lane(key, lane).map(f64::to_bits),
                            "{at}.{lane}"
                        );
                    }
                    checked += 1;
                }
            }
        }
    }
    assert!(checked > 3000, "{checked} fields read");
}

#[test]
fn an_engine_record_text_field_takes_the_idl_capacity() {
    let fields: &'static [FieldDecl] = Box::leak(Box::new([
        field("label", "str", "\"Ada\"", 24),
        field("size", "f32", "1", 0),
    ]));
    let decl = RecordDecl {
        kind: "component",
        name: "Tag",
        schema: "og.Tag",
        fields,
    };
    let layout = Layout::from_decl(&decl).unwrap();
    assert_eq!(layout.fields[0].ty, FieldType::Str(24));
    assert_eq!(layout.fields[1].ty, FieldType::F32);
    let bytes = layout.encode(None, None).unwrap();
    assert_eq!(bytes.len(), 24 + 4);
    assert_eq!(&bytes[..3], b"Ada");
    assert!(bytes[3..24].iter().all(|b| *b == 0));
}

/// A list field (`List.Post.8` in the IDL) is its element's kind at the
/// IDL's capacity: an array of handles in and out, `[]` and `[0]` apart,
/// refused past the capacity with the fix and never truncated.
#[test]
fn an_engine_record_list_field_takes_the_idl_capacity() {
    let mut passes = field("passes", "Post", "[]", 0);
    passes.list = 3;
    let mut weights = field("weights", "f32", "[]", 0);
    weights.list = 2;
    let fields: &'static [FieldDecl] = Box::leak(Box::new([passes, weights]));
    let decl = RecordDecl {
        kind: "resource",
        name: "Chain",
        schema: "og.Chain",
        fields,
    };
    let layout = Layout::from_decl(&decl).unwrap();
    assert_eq!(
        layout.fields[0].ty,
        FieldType::List(og_core::Scalar::U64, 3)
    );
    assert_eq!(
        layout.fields[1].ty,
        FieldType::List(og_core::Scalar::F32, 2)
    );
    let empty = layout.encode(None, None).unwrap();
    assert_eq!(empty.len(), 8 * 4 + 4 * 3);
    assert_eq!(
        layout.decode(&empty),
        json!({ "passes": [], "weights": [] })
    );
    let big = 4_294_967_298u64;
    let value = json!({ "passes": [big, 0], "weights": [0.5] });
    let bytes = layout.encode(None, Some(&value)).unwrap();
    assert_eq!(layout.decode(&bytes), value);
    assert_eq!(bytes[..8], 2u64.to_le_bytes());
    let zero = layout
        .encode(None, Some(&json!({ "passes": [0] })))
        .unwrap();
    assert_ne!(zero, empty, "[0] is not []");
    let long = layout
        .encode(Some(&bytes), Some(&json!({ "passes": [1, 2, 3, 4] })))
        .unwrap_err();
    assert!(
        long.contains("og.Chain.passes': 4 elements and the field holds 3")
            && long.contains("list(Asset(), 4)"),
        "{long}"
    );
    let bad = layout
        .encode(None, Some(&json!({ "weights": "x" })))
        .unwrap_err();
    assert!(bad.contains("a list is an array"), "{bad}");
    // A row reads no list field: its JSON is an array.
    let row = Row::new(&layout, &bytes);
    assert_eq!((row.num("passes"), row.uint("passes")), (None, None));
}

/// PostEffects.custom holds what og-render runs, so the two cannot drift.
#[test]
fn post_effects_custom_is_a_list_of_every_pass_og_render_runs() {
    let post = og_api::host::RECORDS
        .iter()
        .find(|r| r.schema == "og.PostEffects")
        .unwrap();
    let custom = post.fields.iter().find(|f| f.name == "custom").unwrap();
    assert_eq!(
        (custom.column, custom.list as usize),
        ("Post", og_render::MAX_CUSTOM_POSTS)
    );
    assert!(
        !post
            .fields
            .iter()
            .any(|f| f.name.starts_with("custom") && f.name != "custom")
    );
}
