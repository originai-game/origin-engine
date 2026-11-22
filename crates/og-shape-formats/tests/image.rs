//! Image headers read without decoding: sizes and the mip chain's memory.

use og_shape::tools::*;

#[test]
fn reads_sizes_and_counts_the_mip_chain() {
    let png = og_shape_formats::png::write::png(&og_shape::tools::Image {
        width: 6,
        height: 3,
        rgba: vec![0; 6 * 3 * 4],
    });
    let info = ImageInfo::from_bytes("a", &png);
    assert_eq!(
        (info.format.as_str(), info.width, info.height),
        ("png", Some(6), Some(3))
    );
    // 6×3, 3×1, 1×1.
    assert_eq!(info.decoded_bytes(), Some((72, 72 + 12 + 4)));
    // A baseline JPEG header: SOI, an APP0 segment, SOF0 with 640×480.
    let mut jpg = vec![0xFF, 0xD8, 0xFF, 0xE0, 0, 4, 0, 0];
    jpg.extend([
        0xFF, 0xC0, 0, 11, 8, 0x01, 0xE0, 0x02, 0x80, 3, 0, 0, 0, 0, 0,
    ]);
    let info = ImageInfo::from_bytes("b", &jpg);
    assert_eq!((info.width, info.height), (Some(640), Some(480)));
    assert!(ImageInfo::from_bytes("c", b"nope").error.is_some());
}
