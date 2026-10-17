//! The container's contract, through its public boundary.
//!
//! Ways it could fail, each covered below: the writer emits different bytes
//! for the same calls (uninitialised padding, map ordering); a payload comes
//! back changed or at an unaligned offset; a stale blob (other stamp, type or
//! schema version) is accepted, or refused without the path and both values;
//! a truncated or overrun file is accepted; a table is viewed as the wrong row
//! type; an in-place view is attempted on misaligned bytes.

use og_blob::{AlignedBytes, Blob, BlobError, BlobWriter, Expect, HEADER_SIZE};
use std::borrow::Cow;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct Row {
    a: u32,
    b: f32,
}

const ROW_TYPE: u64 = 0x1234_5678_9abc_def0;

fn sample(stamp: u64) -> Vec<u8> {
    let rows = [
        Row { a: 1, b: 0.5 },
        Row { a: 2, b: -1.0 },
        Row { a: 3, b: 7.25 },
    ];
    BlobWriter::new(0xaaaa, 3, stamp)
        .raw("meta", vec![9u8; 5], 4)
        .table("rows", &rows, ROW_TYPE)
        .raw("payload", (0..=255u8).collect::<Vec<_>>(), 256)
        .raw("empty", Vec::new(), 16)
        .finish()
}

#[test]
fn the_same_calls_write_the_same_bytes() {
    assert_eq!(sample(7), sample(7));
    assert_ne!(sample(7), sample(8));
}

#[test]
fn sections_come_back_unchanged_and_aligned() {
    let bytes = AlignedBytes::from_slice(&sample(7));
    let blob = Blob::parse(&bytes, "x.ogb").unwrap();
    assert_eq!(blob.header().type_hash, 0xaaaa);
    assert_eq!(blob.header().schema_version, 3);
    assert_eq!(blob.header().cook_stamp, 7);
    assert_eq!(blob.header().max_align, 256);
    assert_eq!(blob.raw("meta").unwrap(), &[9u8; 5]);
    assert_eq!(
        blob.raw("payload").unwrap(),
        (0..=255u8).collect::<Vec<_>>().as_slice()
    );
    assert_eq!(blob.raw("empty").unwrap(), &[] as &[u8]);
    for s in blob.sections() {
        assert_eq!(s.offset % s.align as u64, 0, "{} at {}", s.name, s.offset);
        assert!(s.offset as usize >= HEADER_SIZE);
    }
    let names: Vec<_> = blob.sections().iter().map(|s| s.name).collect();
    assert_eq!(names, ["meta", "rows", "payload", "empty"]);
    assert!(matches!(
        blob.raw("nope"),
        Err(BlobError::MissingSection { .. })
    ));
}

#[test]
fn tables_are_viewed_in_place_when_aligned_and_copied_otherwise() {
    let bytes = AlignedBytes::from_slice(&sample(7));
    let blob = Blob::parse(&bytes, "x.ogb").unwrap();
    let rows = blob.table::<Row>("rows", ROW_TYPE).unwrap();
    assert!(matches!(rows, Cow::Borrowed(_)));
    assert_eq!(rows[2], Row { a: 3, b: 7.25 });
    assert!(blob.table_view::<Row>("rows", ROW_TYPE).unwrap().is_some());

    // The same blob one byte off its alignment: still readable, by copy.
    let mut shifted = vec![0u8; 1];
    shifted.extend_from_slice(&sample(7));
    let blob = Blob::parse(&shifted[1..], "x.ogb").unwrap();
    let rows = blob.table::<Row>("rows", ROW_TYPE).unwrap();
    assert!(matches!(rows, Cow::Owned(_)));
    assert_eq!(rows[1], Row { a: 2, b: -1.0 });
    assert!(blob.table_view::<Row>("rows", ROW_TYPE).unwrap().is_none());

    // Another row type or size is refused.
    assert!(matches!(
        blob.table::<Row>("rows", ROW_TYPE + 1),
        Err(BlobError::SectionType { .. })
    ));
    assert!(matches!(
        blob.table::<u32>("rows", ROW_TYPE),
        Err(BlobError::SectionType { .. })
    ));
}

#[test]
fn a_stale_blob_is_refused_with_the_path_and_both_stamps() {
    let bytes = sample(0x1111);
    let expect = Expect {
        type_hash: 0xaaaa,
        schema_version: 3,
        stamp: Some(0x2222),
    };
    let err = Blob::open(&bytes, "cooked/models/ship.glb.ogb", &expect).unwrap_err();
    assert_eq!(
        err,
        BlobError::StampMismatch {
            path: "cooked/models/ship.glb.ogb".into(),
            expected: 0x2222,
            found: 0x1111
        }
    );
    let text = err.to_string();
    assert!(text.contains("cooked/models/ship.glb.ogb"), "{text}");
    assert!(
        text.contains("0000000000001111") && text.contains("0000000000002222"),
        "{text}"
    );

    let ok = Blob::open(
        &bytes,
        "p",
        &Expect {
            stamp: Some(0x1111),
            ..expect
        },
    );
    assert!(ok.is_ok());
    assert!(matches!(
        Blob::open(
            &bytes,
            "p",
            &Expect {
                type_hash: 1,
                stamp: None,
                ..expect
            }
        ),
        Err(BlobError::TypeMismatch {
            expected: 1,
            found: 0xaaaa,
            ..
        })
    ));
    assert!(matches!(
        Blob::open(
            &bytes,
            "p",
            &Expect {
                schema_version: 4,
                stamp: None,
                ..expect
            }
        ),
        Err(BlobError::SchemaVersion {
            expected: 4,
            found: 3,
            ..
        })
    ));
}

#[test]
fn damaged_files_are_refused() {
    let bytes = sample(1);
    // Truncated anywhere: refused, never a panic.
    for len in [0, 10, HEADER_SIZE, bytes.len() - 1] {
        assert!(
            matches!(
                Blob::parse(&bytes[..len], "t"),
                Err(BlobError::Malformed { .. })
            ),
            "accepted {len} bytes"
        );
    }
    // Wrong magic.
    let mut b = bytes.clone();
    b[1] = b'X';
    assert!(matches!(
        Blob::parse(&b, "t"),
        Err(BlobError::Malformed { .. })
    ));
    // Unknown container version.
    let mut b = bytes.clone();
    b[8] = 9;
    assert!(matches!(
        Blob::parse(&b, "t"),
        Err(BlobError::ContainerVersion { found: 9, .. })
    ));
    // A section offset moved past the end of the file.
    let mut b = bytes.clone();
    let entry = HEADER_SIZE + 48 * 2;
    b[entry + 8..entry + 16].copy_from_slice(&(bytes.len() as u64).to_le_bytes());
    assert!(matches!(
        Blob::parse(&b, "t"),
        Err(BlobError::Malformed { .. })
    ));
    // A section offset that breaks its alignment.
    let mut b = bytes.clone();
    let at = u64::from_le_bytes(b[entry + 8..entry + 16].try_into().unwrap()) + 1;
    b[entry + 8..entry + 16].copy_from_slice(&at.to_le_bytes());
    assert!(matches!(
        Blob::parse(&b, "t"),
        Err(BlobError::Malformed { .. })
    ));
    // Every single-byte corruption of the header and table is refused or
    // parses into something structurally valid; none panics.
    let table_end = HEADER_SIZE + 48 * 4;
    for i in 0..table_end {
        let mut b = bytes.clone();
        b[i] ^= 0xff;
        let _ = Blob::parse(&b, "t");
    }
}
