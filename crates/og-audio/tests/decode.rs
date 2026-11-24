//! The native QOA decoder against the web's: both decode one fixture to
//! one pinned hash, so the two hosts cannot drift apart.

use og_audio::qoa;

/// The fixture the web's decoder pins too (packages/engine/tests/
/// audio-decoder.test.ts): the two decoders share one .qoa and one pinned
/// hash read from `frames.fnv` beside it, so they cannot drift apart.
#[test]
fn the_fixture_hashes_as_the_web_decoder_pins_it() {
    let base = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../packages/engine/tests/fixtures/qoa"
    );
    let file = std::fs::read(format!("{base}/frames.qoa")).unwrap();
    let want = u64::from_str_radix(
        std::fs::read_to_string(format!("{base}/frames.fnv"))
            .unwrap()
            .trim(),
        16,
    )
    .unwrap();
    let hash = |samples: &[i16]| {
        og_core::hash::hash_bytes(
            &samples
                .iter()
                .flat_map(|s| s.to_le_bytes())
                .collect::<Vec<u8>>(),
        )
    };
    let whole = qoa::decode(&file).unwrap();
    assert_eq!(whole.samples.len(), 12_000 * 2);
    assert_eq!(hash(&whole.samples), want);
    // Frame by frame, as the bank's jobs carry the file: `qoaf`, the
    // frame's length, the frame.
    let mut files = Vec::new();
    let mut pos = 8;
    while pos < file.len() {
        let fh = u64::from_be_bytes(file[pos..pos + 8].try_into().unwrap());
        let len = ((fh >> 16) & 0xffff) as usize;
        let size = (fh & 0xffff) as usize;
        let mut one = Vec::with_capacity(8 + size);
        one.extend_from_slice(&((0x716f_6166u64 << 32) | len as u64).to_be_bytes());
        one.extend_from_slice(&file[pos..pos + size]);
        files.push(one);
        pos += size;
    }
    assert_eq!(files.len(), 3);
    let mut samples = Vec::new();
    for f in &files {
        samples.extend(qoa::decode(f).unwrap().samples);
    }
    assert_eq!(hash(&samples), want);
}
